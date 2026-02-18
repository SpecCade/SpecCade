//! Topology-focused mesh lint rules.
//!
//! Rules that check mesh structure: non-manifold edges, degenerate faces,
//! isolated vertices, excessive ngons, and high polygon count.

use crate::report::{AssetType, LintIssue, Severity};
use crate::rules::{AssetData, LintRule};
use speccade_spec::Spec;
use std::collections::{HashMap, HashSet};

use super::parsing::{parse_mesh_data, triangle_area_3d};

/// Rule 1: Non-manifold edges
///
/// Detection: edge shared by more than 2 faces
pub struct NonManifoldRule;

impl LintRule for NonManifoldRule {
    fn id(&self) -> &'static str {
        "mesh/non-manifold"
    }

    fn description(&self) -> &'static str {
        "Detects non-manifold edges (edges shared by more than 2 faces)"
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

        // Count edge usage
        let mut edge_count_map: HashMap<(u32, u32), u32> = HashMap::new();

        for tri in mesh.indices.chunks(3) {
            if tri.len() != 3 {
                continue;
            }
            let v0 = tri[0];
            let v1 = tri[1];
            let v2 = tri[2];

            for (a, b) in [(v0, v1), (v1, v2), (v2, v0)] {
                let edge = if a < b { (a, b) } else { (b, a) };
                *edge_count_map.entry(edge).or_insert(0) += 1;
            }
        }

        // Find edges shared by more than 2 faces
        let non_manifold_edges: Vec<_> = edge_count_map
            .iter()
            .filter(|(_, &count)| count > 2)
            .collect();

        if non_manifold_edges.is_empty() {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Found {} non-manifold edge(s) shared by more than 2 faces",
                non_manifold_edges.len()
            ),
            "Remove duplicate faces or fix mesh topology",
        )
        .with_actual_value(format!("{} edges", non_manifold_edges.len()))
        .with_expected_range("0 non-manifold edges")]
    }
}

/// Rule 2: Degenerate faces
///
/// Detection: face area < 1e-8
pub struct DegenerateFacesRule;

impl LintRule for DegenerateFacesRule {
    fn id(&self) -> &'static str {
        "mesh/degenerate-faces"
    }

    fn description(&self) -> &'static str {
        "Detects zero-area triangles (degenerate faces)"
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

        let mut degenerate_count = 0u32;
        let threshold = 1e-8;

        for tri in mesh.indices.chunks(3) {
            if tri.len() != 3 {
                continue;
            }
            let v0 = tri[0] as usize;
            let v1 = tri[1] as usize;
            let v2 = tri[2] as usize;

            // Check duplicate vertices
            if v0 == v1 || v1 == v2 || v0 == v2 {
                degenerate_count += 1;
                continue;
            }

            // Check area
            if v0 < mesh.positions.len() && v1 < mesh.positions.len() && v2 < mesh.positions.len() {
                let area =
                    triangle_area_3d(mesh.positions[v0], mesh.positions[v1], mesh.positions[v2]);
                if area < threshold {
                    degenerate_count += 1;
                }
            }
        }

        if degenerate_count == 0 {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Found {} degenerate triangle(s) with zero or near-zero area",
                degenerate_count
            ),
            "Remove or merge vertices to eliminate zero-area faces",
        )
        .with_actual_value(format!("{} faces", degenerate_count))
        .with_expected_range("0 degenerate faces")]
    }
}

/// Rule 9: Excessive ngons
///
/// Detection: >20% faces with >4 vertices
///
/// Note: glTF uses triangles only, so this mainly checks for quads/ngons in
/// other formats or flags that the mesh was already triangulated.
pub struct ExcessiveNgonsRule;

impl LintRule for ExcessiveNgonsRule {
    fn id(&self) -> &'static str {
        "mesh/excessive-ngons"
    }

    fn description(&self) -> &'static str {
        "Detects excessive use of n-gons (faces with more than 4 vertices)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Mesh]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        // glTF always uses triangles, so we check the extension to see if this is
        // a format that might have ngons (like OBJ or FBX)
        let ext = asset
            .path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // glTF/GLB are already triangulated by the format spec
        if ext == "glb" || ext == "gltf" {
            return vec![];
        }

        // For other formats, we would need format-specific parsing
        // This is a placeholder for future OBJ/FBX support
        vec![]
    }
}

/// Rule 10: Isolated vertices
///
/// Detection: vertices not referenced by any face
pub struct IsolatedVertsRule;

impl LintRule for IsolatedVertsRule {
    fn id(&self) -> &'static str {
        "mesh/isolated-verts"
    }

    fn description(&self) -> &'static str {
        "Detects vertices not used by any face"
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

        if mesh.positions.is_empty() || mesh.indices.is_empty() {
            return vec![];
        }

        // Track which vertices are used
        let mut used_vertices: HashSet<u32> = HashSet::new();
        for &idx in &mesh.indices {
            used_vertices.insert(idx);
        }

        let total_vertices = mesh.positions.len();
        let isolated_count = total_vertices - used_vertices.len();

        if isolated_count == 0 {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Found {} isolated vertex/vertices not used by any face",
                isolated_count
            ),
            "Remove isolated vertices to reduce file size",
        )
        .with_actual_value(format!("{} isolated", isolated_count))
        .with_expected_range("0 isolated vertices")]
    }
}

/// Rule 11: High poly count
///
/// Detection: >50k triangles
pub struct HighPolyRule;

impl HighPolyRule {
    const TRIANGLE_THRESHOLD: u32 = 50_000;
}

impl LintRule for HighPolyRule {
    fn id(&self) -> &'static str {
        "mesh/high-poly"
    }

    fn description(&self) -> &'static str {
        "Warns about high polygon count meshes"
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

        let triangle_count = (mesh.indices.len() / 3) as u32;

        if triangle_count <= Self::TRIANGLE_THRESHOLD {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Mesh has {} triangles (threshold: {})",
                triangle_count,
                Self::TRIANGLE_THRESHOLD
            ),
            "Consider adding LOD levels or decimating for performance",
        )
        .with_actual_value(format!("{} triangles", triangle_count))
        .with_expected_range(format!("<= {} triangles", Self::TRIANGLE_THRESHOLD))
        .with_fix_param("triangle_count")]
    }
}
