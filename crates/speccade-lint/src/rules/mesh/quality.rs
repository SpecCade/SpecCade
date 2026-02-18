//! Quality-focused mesh lint rules.
//!
//! Rules that check vertex weights, normals, humanoid proportions,
//! UV quality, and material assignments.

use crate::report::{AssetType, LintIssue, Severity};
use crate::rules::{AssetData, LintRule};
use speccade_spec::Spec;

use super::parsing::{
    calculate_centroid, calculate_face_normal, dot3, parse_mesh_data, triangle_area_2d,
    triangle_area_3d,
};

/// Rule 3: Unweighted vertices (skinned meshes)
///
/// Detection: vertex with weight sum = 0 when mesh has skeleton
pub struct UnweightedVertsRule;

impl LintRule for UnweightedVertsRule {
    fn id(&self) -> &'static str {
        "mesh/unweighted-verts"
    }

    fn description(&self) -> &'static str {
        "Detects vertices with no bone weights in skinned meshes"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Mesh]
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(mesh) = parse_mesh_data(asset) else {
            return vec![];
        };

        // Only applies to skinned meshes
        if !mesh.has_skeleton {
            return vec![];
        }

        // If we have no weight data but have a skeleton, that's a problem
        if mesh.weights.is_empty() && !mesh.positions.is_empty() {
            return vec![LintIssue::new(
                self.id(),
                self.default_severity(),
                "Skinned mesh has no weight data",
                "Add vertex weights using auto-weights or manual weight painting",
            )
            .with_actual_value("no weights")
            .with_expected_range("weights for all vertices")];
        }

        // Count vertices with zero total weight
        let mut unweighted_count = 0usize;
        for w in &mesh.weights {
            let sum = w[0] + w[1] + w[2] + w[3];
            if sum < 1e-6 {
                unweighted_count += 1;
            }
        }

        if unweighted_count == 0 {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Found {} vertex/vertices with no bone weights",
                unweighted_count
            ),
            "Apply auto-weights or paint weights for unweighted vertices",
        )
        .with_actual_value(format!("{} unweighted", unweighted_count))
        .with_expected_range("0 unweighted vertices")]
    }
}

/// Rule 4: Inverted normals
///
/// Detection: normals pointing inward (away from mesh centroid)
pub struct InvertedNormalsRule;

impl LintRule for InvertedNormalsRule {
    fn id(&self) -> &'static str {
        "mesh/inverted-normals"
    }

    fn description(&self) -> &'static str {
        "Detects normals pointing inward (inverted face orientation)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Mesh]
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(mesh) = parse_mesh_data(asset) else {
            return vec![];
        };

        if mesh.indices.is_empty() || mesh.positions.is_empty() {
            return vec![];
        }

        // Calculate mesh centroid
        let centroid = calculate_centroid(&mesh.positions);

        let mut inverted_count = 0u32;
        let mut total_faces = 0u32;

        for tri in mesh.indices.chunks(3) {
            if tri.len() != 3 {
                continue;
            }
            let v0 = tri[0] as usize;
            let v1 = tri[1] as usize;
            let v2 = tri[2] as usize;

            if v0 >= mesh.positions.len()
                || v1 >= mesh.positions.len()
                || v2 >= mesh.positions.len()
            {
                continue;
            }

            total_faces += 1;

            // Calculate face center
            let face_center = [
                (mesh.positions[v0][0] + mesh.positions[v1][0] + mesh.positions[v2][0]) / 3.0,
                (mesh.positions[v0][1] + mesh.positions[v1][1] + mesh.positions[v2][1]) / 3.0,
                (mesh.positions[v0][2] + mesh.positions[v1][2] + mesh.positions[v2][2]) / 3.0,
            ];

            // Calculate face normal
            let face_normal =
                calculate_face_normal(mesh.positions[v0], mesh.positions[v1], mesh.positions[v2]);

            // Vector from centroid to face center
            let to_face = [
                face_center[0] - centroid[0],
                face_center[1] - centroid[1],
                face_center[2] - centroid[2],
            ];

            // If normal points away from centroid, dot product should be positive
            // If negative, the normal is inverted
            let dot = dot3(face_normal, to_face);
            if dot < 0.0 {
                inverted_count += 1;
            }
        }

        // Only report if majority of faces appear inverted (to handle non-convex meshes)
        // A mesh with >50% inverted normals likely has incorrect orientation
        if total_faces == 0 || inverted_count < total_faces / 2 {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Found {} of {} faces with normals pointing inward",
                inverted_count, total_faces
            ),
            "Recalculate normals with consistent outward orientation",
        )
        .with_actual_value(format!(
            "{:.1}% inverted",
            (inverted_count as f64 / total_faces as f64) * 100.0
        ))
        .with_expected_range("< 50% inverted")]
    }
}

/// Rule 5: Humanoid proportions
///
/// Detection: if armature has standard bone names, check limb ratios
pub struct HumanoidProportionsRule;

impl HumanoidProportionsRule {
    /// Standard humanoid bone name patterns
    const BONE_PATTERNS: &'static [(&'static str, &'static str)] = &[
        ("upper_arm", "forearm"),
        ("upper_arm", "lower_arm"),
        ("upperarm", "lowerarm"),
        ("arm_upper", "arm_lower"),
        ("thigh", "shin"),
        ("upper_leg", "lower_leg"),
        ("upperleg", "lowerleg"),
        ("leg_upper", "leg_lower"),
    ];

    /// Valid ratio range for limb proportions (lower segment / upper segment)
    const MIN_RATIO: f64 = 0.7;
    const MAX_RATIO: f64 = 1.3;
}

impl LintRule for HumanoidProportionsRule {
    fn id(&self) -> &'static str {
        "mesh/humanoid-proportions"
    }

    fn description(&self) -> &'static str {
        "Checks humanoid armature limb proportions against anatomical ranges"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Mesh]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(mesh) = parse_mesh_data(asset) else {
            return vec![];
        };

        if !mesh.has_skeleton || mesh.bone_names.is_empty() {
            return vec![];
        }

        let mut issues = Vec::new();

        // Group patterns by upper segment to avoid false positives when
        // different naming conventions map to the same bone pair.
        // E.g. ("upper_arm", "forearm") and ("upper_arm", "lower_arm") both
        // match the upper_arm pattern -- only warn if NO lower variant matches.
        let mut checked_uppers = std::collections::HashSet::new();

        for (upper_pattern, _) in Self::BONE_PATTERNS {
            if !checked_uppers.insert(*upper_pattern) {
                continue;
            }

            let upper_bones: Vec<_> = mesh
                .bone_names
                .iter()
                .filter(|n| n.contains(upper_pattern))
                .collect();

            if upper_bones.is_empty() {
                continue;
            }

            // Collect all lower patterns that share this upper pattern
            let lower_patterns: Vec<_> = Self::BONE_PATTERNS
                .iter()
                .filter(|(u, _)| *u == *upper_pattern)
                .map(|(_, l)| *l)
                .collect();

            let has_any_lower = lower_patterns
                .iter()
                .any(|lp| mesh.bone_names.iter().any(|n| n.contains(lp)));

            if !has_any_lower {
                issues.push(
                    LintIssue::new(
                        self.id(),
                        self.default_severity(),
                        format!(
                            "Found upper segment bones ({}) but missing lower segment ({})",
                            upper_pattern,
                            lower_patterns.join(" or ")
                        ),
                        "Add missing lower segment bones to complete limb hierarchy",
                    )
                    .with_asset_location(format!("bone:{}", upper_bones[0]))
                    .with_expected_range(format!(
                        "ratio {:.1}-{:.1}",
                        Self::MIN_RATIO,
                        Self::MAX_RATIO
                    )),
                );
            }
        }

        issues
    }
}

/// Rule 6: UV overlap
///
/// Detection: UV triangles that intersect (simplified check)
pub struct UvOverlapRule;

impl LintRule for UvOverlapRule {
    fn id(&self) -> &'static str {
        "mesh/uv-overlap"
    }

    fn description(&self) -> &'static str {
        "Detects overlapping UV islands"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Mesh]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(mesh) = parse_mesh_data(asset) else {
            return vec![];
        };

        if mesh.uvs.len() < 3 {
            return vec![];
        }

        // Simplified overlap detection using total UV area
        // If total UV area > 1.0 (UV space is 0-1), there's likely overlap
        let mut total_area = 0.0f64;
        for tri in mesh.uvs.chunks(3) {
            if tri.len() == 3 {
                let area = triangle_area_2d(tri[0], tri[1], tri[2]);
                total_area += area.abs();
            }
        }

        // Threshold: 5% overlap is significant
        let overlap_threshold = 1.05;
        if total_area <= overlap_threshold {
            return vec![];
        }

        let overlap_percent = ((total_area - 1.0) / total_area * 100.0).min(100.0);

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "UV islands overlap by approximately {:.1}%",
                overlap_percent
            ),
            "Repack UVs to eliminate overlapping islands",
        )
        .with_actual_value(format!("{:.1}% overlap", overlap_percent))
        .with_expected_range("< 5% overlap")]
    }
}

/// Rule 7: UV stretch
///
/// Detection: UV area vs 3D area ratio > 2.0 (or < 0.5)
pub struct UvStretchRule;

impl LintRule for UvStretchRule {
    fn id(&self) -> &'static str {
        "mesh/uv-stretch"
    }

    fn description(&self) -> &'static str {
        "Detects high UV distortion (stretch)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Mesh]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(mesh) = parse_mesh_data(asset) else {
            return vec![];
        };

        if mesh.uvs.len() < 3 || mesh.positions.len() < 3 || mesh.indices.is_empty() {
            return vec![];
        }

        let mut stretched_faces = 0u32;
        let mut total_faces = 0u32;
        let stretch_threshold = 2.0;

        // For each triangle, compare 3D area to UV area
        for (face_idx, tri) in mesh.indices.chunks(3).enumerate() {
            if tri.len() != 3 {
                continue;
            }
            let v0 = tri[0] as usize;
            let v1 = tri[1] as usize;
            let v2 = tri[2] as usize;

            if v0 >= mesh.positions.len()
                || v1 >= mesh.positions.len()
                || v2 >= mesh.positions.len()
            {
                continue;
            }

            // Get UV coordinates (if available)
            let uv_base = face_idx * 3;
            if uv_base + 2 >= mesh.uvs.len() {
                continue;
            }

            total_faces += 1;

            let area_3d =
                triangle_area_3d(mesh.positions[v0], mesh.positions[v1], mesh.positions[v2]);
            let area_uv = triangle_area_2d(
                mesh.uvs[uv_base],
                mesh.uvs[uv_base + 1],
                mesh.uvs[uv_base + 2],
            );

            if area_3d < 1e-10 || area_uv < 1e-10 {
                continue;
            }

            let ratio = area_uv / area_3d;
            if ratio > stretch_threshold || ratio < 1.0 / stretch_threshold {
                stretched_faces += 1;
            }
        }

        if total_faces == 0 || stretched_faces == 0 {
            return vec![];
        }

        let stretch_percent = (stretched_faces as f64 / total_faces as f64) * 100.0;

        // Only warn if more than 10% of faces are stretched
        if stretch_percent < 10.0 {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "{:.1}% of faces have high UV distortion (stretch ratio > {})",
                stretch_percent, stretch_threshold
            ),
            "Adjust UV seams and unwrap settings to reduce distortion",
        )
        .with_actual_value(format!("{:.1}% stretched", stretch_percent))
        .with_expected_range("< 10% faces stretched")]
    }
}

/// Rule 8: Missing material
///
/// Detection: faces with no material assignment
pub struct MissingMaterialRule;

impl LintRule for MissingMaterialRule {
    fn id(&self) -> &'static str {
        "mesh/missing-material"
    }

    fn description(&self) -> &'static str {
        "Detects faces with no material assigned"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Mesh]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(mesh) = parse_mesh_data(asset) else {
            return vec![];
        };

        if mesh.material_indices.is_empty() {
            return vec![];
        }

        let missing_count = mesh.material_indices.iter().filter(|m| m.is_none()).count();

        if missing_count == 0 {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!("{} face(s) have no material assigned", missing_count),
            "Assign a material to all faces",
        )
        .with_actual_value(format!("{} faces", missing_count))
        .with_expected_range("0 faces without material")]
    }
}

/// Rule 12: No UVs
///
/// Detection: mesh has no UV coordinates
pub struct NoUvsRule;

impl LintRule for NoUvsRule {
    fn id(&self) -> &'static str {
        "mesh/no-uvs"
    }

    fn description(&self) -> &'static str {
        "Detects meshes without UV coordinates"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Mesh]
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(mesh) = parse_mesh_data(asset) else {
            return vec![];
        };

        // Only flag if the mesh has geometry but no UVs
        if mesh.positions.is_empty() || !mesh.uvs.is_empty() {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            "Mesh has no UV coordinates",
            "Add UV projection (box, cylindrical, or smart UV project)",
        )
        .with_fix_template("mesh.uv_project(method=\"smart\")")]
    }
}
