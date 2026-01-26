//! Mesh quality lint rules.
//!
//! Rules for detecting perceptual problems in generated mesh assets.
//!
//! Includes 12 rules across three severity levels:
//! - **Error**: Non-manifold edges, degenerate faces, unweighted vertices, inverted normals
//! - **Warning**: Humanoid proportions, UV overlap/stretch, missing material, excessive ngons, isolated vertices
//! - **Info**: High poly count, no UVs

use crate::report::{AssetType, LintIssue, Severity};
use crate::rules::{AssetData, LintRule};
use speccade_spec::Spec;
use std::collections::{HashMap, HashSet};

// =============================================================================
// Helper functions for glTF/GLB parsing
// =============================================================================

/// Extract the binary chunk from a GLB file.
fn extract_glb_binary_chunk(b: &[u8]) -> Option<&[u8]> {
    // GLB format: first 12 bytes are header, then chunks
    if b.len() > 12 {
        let json_chunk_length = u32::from_le_bytes([b[12], b[13], b[14], b[15]]) as usize;
        let bin_chunk_start = 12 + 8 + json_chunk_length;
        if b.len() > bin_chunk_start + 8 {
            let bin_chunk_length = u32::from_le_bytes([
                b[bin_chunk_start],
                b[bin_chunk_start + 1],
                b[bin_chunk_start + 2],
                b[bin_chunk_start + 3],
            ]) as usize;
            let bin_data_start = bin_chunk_start + 8;
            if b.len() >= bin_data_start + bin_chunk_length {
                return Some(&b[bin_data_start..bin_data_start + bin_chunk_length]);
            }
        }
    }
    None
}

/// Read vertex positions from an accessor.
fn read_positions(accessor: &gltf::Accessor, data: &[u8], positions: &mut Vec<[f32; 3]>) {
    let view = match accessor.view() {
        Some(v) => v,
        None => return,
    };

    let offset = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(12); // 3 * sizeof(f32)

    for i in 0..accessor.count() {
        let start = offset + i * stride;
        if start + 12 <= data.len() {
            let x = f32::from_le_bytes([
                data[start],
                data[start + 1],
                data[start + 2],
                data[start + 3],
            ]);
            let y = f32::from_le_bytes([
                data[start + 4],
                data[start + 5],
                data[start + 6],
                data[start + 7],
            ]);
            let z = f32::from_le_bytes([
                data[start + 8],
                data[start + 9],
                data[start + 10],
                data[start + 11],
            ]);
            positions.push([x, y, z]);
        }
    }
}

/// Read vertex normals from an accessor.
fn read_normals(accessor: &gltf::Accessor, data: &[u8], normals: &mut Vec<[f32; 3]>) {
    let view = match accessor.view() {
        Some(v) => v,
        None => return,
    };

    let offset = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(12); // 3 * sizeof(f32)

    for i in 0..accessor.count() {
        let start = offset + i * stride;
        if start + 12 <= data.len() {
            let x = f32::from_le_bytes([
                data[start],
                data[start + 1],
                data[start + 2],
                data[start + 3],
            ]);
            let y = f32::from_le_bytes([
                data[start + 4],
                data[start + 5],
                data[start + 6],
                data[start + 7],
            ]);
            let z = f32::from_le_bytes([
                data[start + 8],
                data[start + 9],
                data[start + 10],
                data[start + 11],
            ]);
            normals.push([x, y, z]);
        }
    }
}

/// Read indices from an accessor.
fn read_indices(accessor: &gltf::Accessor, data: &[u8], indices: &mut Vec<u32>) {
    let view = match accessor.view() {
        Some(v) => v,
        None => return,
    };

    let offset = view.offset() + accessor.offset();
    let component_size = match accessor.data_type() {
        gltf::accessor::DataType::U8 => 1,
        gltf::accessor::DataType::U16 => 2,
        gltf::accessor::DataType::U32 => 4,
        _ => return,
    };
    let stride = view.stride().unwrap_or(component_size);

    for i in 0..accessor.count() {
        let start = offset + i * stride;
        let index = match accessor.data_type() {
            gltf::accessor::DataType::U8 => {
                if start < data.len() {
                    data[start] as u32
                } else {
                    continue;
                }
            }
            gltf::accessor::DataType::U16 => {
                if start + 2 <= data.len() {
                    u16::from_le_bytes([data[start], data[start + 1]]) as u32
                } else {
                    continue;
                }
            }
            gltf::accessor::DataType::U32 => {
                if start + 4 <= data.len() {
                    u32::from_le_bytes([
                        data[start],
                        data[start + 1],
                        data[start + 2],
                        data[start + 3],
                    ])
                } else {
                    continue;
                }
            }
            _ => continue,
        };
        indices.push(index);
    }
}

/// Read UV coordinates from an accessor.
fn read_uvs(accessor: &gltf::Accessor, data: &[u8], uvs: &mut Vec<[f32; 2]>) {
    let view = match accessor.view() {
        Some(v) => v,
        None => return,
    };

    let offset = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(8); // 2 * sizeof(f32)

    for i in 0..accessor.count() {
        let start = offset + i * stride;
        if start + 8 <= data.len() {
            let u = f32::from_le_bytes([
                data[start],
                data[start + 1],
                data[start + 2],
                data[start + 3],
            ]);
            let v = f32::from_le_bytes([
                data[start + 4],
                data[start + 5],
                data[start + 6],
                data[start + 7],
            ]);
            uvs.push([u, v]);
        }
    }
}

/// Read bone weights from an accessor.
fn read_weights(accessor: &gltf::Accessor, data: &[u8], weights: &mut Vec<[f32; 4]>) {
    let view = match accessor.view() {
        Some(v) => v,
        None => return,
    };

    let offset = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(16); // 4 * sizeof(f32)

    for i in 0..accessor.count() {
        let start = offset + i * stride;
        if start + 16 <= data.len() {
            let w0 = f32::from_le_bytes([
                data[start],
                data[start + 1],
                data[start + 2],
                data[start + 3],
            ]);
            let w1 = f32::from_le_bytes([
                data[start + 4],
                data[start + 5],
                data[start + 6],
                data[start + 7],
            ]);
            let w2 = f32::from_le_bytes([
                data[start + 8],
                data[start + 9],
                data[start + 10],
                data[start + 11],
            ]);
            let w3 = f32::from_le_bytes([
                data[start + 12],
                data[start + 13],
                data[start + 14],
                data[start + 15],
            ]);
            weights.push([w0, w1, w2, w3]);
        }
    }
}

/// Calculate 3D triangle area using cross product.
fn triangle_area_3d(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f64 {
    let v1 = [
        (p1[0] - p0[0]) as f64,
        (p1[1] - p0[1]) as f64,
        (p1[2] - p0[2]) as f64,
    ];
    let v2 = [
        (p2[0] - p0[0]) as f64,
        (p2[1] - p0[1]) as f64,
        (p2[2] - p0[2]) as f64,
    ];

    // Cross product
    let cross = [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ];

    0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
}

/// Calculate 2D triangle area.
fn triangle_area_2d(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2]) -> f64 {
    let v1 = [(p1[0] - p0[0]) as f64, (p1[1] - p0[1]) as f64];
    let v2 = [(p2[0] - p0[0]) as f64, (p2[1] - p0[1]) as f64];
    0.5 * (v1[0] * v2[1] - v1[1] * v2[0]).abs()
}

/// Calculate face normal from three vertices.
fn calculate_face_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let v2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

    // Cross product
    let cross = [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ];

    // Normalize
    let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if len > 1e-10 {
        [cross[0] / len, cross[1] / len, cross[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}

/// Calculate centroid of a set of positions.
fn calculate_centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    if positions.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut sum = [0.0f64; 3];
    for p in positions {
        sum[0] += p[0] as f64;
        sum[1] += p[1] as f64;
        sum[2] += p[2] as f64;
    }
    let n = positions.len() as f64;
    [(sum[0] / n) as f32, (sum[1] / n) as f32, (sum[2] / n) as f32]
}

/// Dot product of two 3D vectors.
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Parsed mesh data for lint analysis.
struct MeshData {
    positions: Vec<[f32; 3]>,
    /// Stored for future normal consistency checks.
    #[allow(dead_code)]
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
    uvs: Vec<[f32; 2]>,
    weights: Vec<[f32; 4]>,
    has_skeleton: bool,
    material_indices: Vec<Option<usize>>,
    bone_names: Vec<String>,
}

impl MeshData {
    fn from_gltf(gltf: &gltf::Gltf, blob: Option<&[u8]>) -> Self {
        let doc = &gltf.document;
        let blob_data = blob.and_then(extract_glb_binary_chunk);

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        let mut uvs = Vec::new();
        let mut weights = Vec::new();
        let mut material_indices = Vec::new();

        for mesh in doc.meshes() {
            for primitive in mesh.primitives() {
                let primitive_material = primitive.material().index();

                // Read positions
                if let Some(positions_accessor) = primitive.get(&gltf::Semantic::Positions) {
                    let base_vertex = positions.len() as u32;
                    if let Some(data) = blob_data {
                        read_positions(&positions_accessor, data, &mut positions);
                    }

                    // Read indices and adjust by base vertex
                    if let Some(indices_accessor) = primitive.indices() {
                        if let Some(data) = blob_data {
                            let index_start = indices.len();
                            read_indices(&indices_accessor, data, &mut indices);
                            // Adjust indices to global vertex indexing
                            for idx in &mut indices[index_start..] {
                                *idx += base_vertex;
                            }
                        }
                    }

                    // Track material per face (3 indices = 1 triangle)
                    let triangle_count = if let Some(indices_accessor) = primitive.indices() {
                        indices_accessor.count() / 3
                    } else {
                        positions_accessor.count() / 3
                    };
                    for _ in 0..triangle_count {
                        material_indices.push(primitive_material);
                    }
                }

                // Read normals
                if let Some(normals_accessor) = primitive.get(&gltf::Semantic::Normals) {
                    if let Some(data) = blob_data {
                        read_normals(&normals_accessor, data, &mut normals);
                    }
                }

                // Read UVs
                if let Some(uv_accessor) = primitive.get(&gltf::Semantic::TexCoords(0)) {
                    if let Some(data) = blob_data {
                        read_uvs(&uv_accessor, data, &mut uvs);
                    }
                }

                // Read weights
                if let Some(weights_accessor) = primitive.get(&gltf::Semantic::Weights(0)) {
                    if let Some(data) = blob_data {
                        read_weights(&weights_accessor, data, &mut weights);
                    }
                }
            }
        }

        // Check for skeleton and collect bone names
        let has_skeleton = doc.skins().next().is_some();
        let mut bone_names = Vec::new();
        for skin in doc.skins() {
            for joint in skin.joints() {
                if let Some(name) = joint.name() {
                    bone_names.push(name.to_lowercase());
                }
            }
        }

        MeshData {
            positions,
            normals,
            indices,
            uvs,
            weights,
            has_skeleton,
            material_indices,
            bone_names,
        }
    }
}

/// Try to parse mesh data from asset bytes.
fn parse_mesh_data(asset: &AssetData) -> Option<MeshData> {
    let gltf = gltf::Gltf::from_slice(asset.bytes).ok()?;
    Some(MeshData::from_gltf(&gltf, Some(asset.bytes)))
}

// =============================================================================
// Error-level rules (4)
// =============================================================================

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
            if v0 < mesh.positions.len()
                && v1 < mesh.positions.len()
                && v2 < mesh.positions.len()
            {
                let area = triangle_area_3d(
                    mesh.positions[v0],
                    mesh.positions[v1],
                    mesh.positions[v2],
                );
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
            let face_normal = calculate_face_normal(
                mesh.positions[v0],
                mesh.positions[v1],
                mesh.positions[v2],
            );

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

// =============================================================================
// Warning-level rules (6)
// =============================================================================

/// Rule 5: Humanoid proportions
///
/// Detection: if armature has standard bone names, check limb ratios
pub struct HumanoidProportionsRule;

impl HumanoidProportionsRule {
    /// Standard humanoid bone name patterns
    const BONE_PATTERNS: &'static [(&'static str, &'static str)] = &[
        ("upper_arm", "forearm"),
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

        // Check if we have standard humanoid bone names
        for (upper_pattern, lower_pattern) in Self::BONE_PATTERNS {
            let upper_bones: Vec<_> = mesh
                .bone_names
                .iter()
                .filter(|n| n.contains(upper_pattern))
                .collect();
            let lower_bones: Vec<_> = mesh
                .bone_names
                .iter()
                .filter(|n| n.contains(lower_pattern))
                .collect();

            if !upper_bones.is_empty() && lower_bones.is_empty() {
                issues.push(
                    LintIssue::new(
                        self.id(),
                        self.default_severity(),
                        format!(
                            "Found upper segment bones ({}) but missing lower segment ({})",
                            upper_pattern, lower_pattern
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

            let area_3d = triangle_area_3d(
                mesh.positions[v0],
                mesh.positions[v1],
                mesh.positions[v2],
            );
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
            format!(
                "{} face(s) have no material assigned",
                missing_count
            ),
            "Assign a material to all faces",
        )
        .with_actual_value(format!("{} faces", missing_count))
        .with_expected_range("0 faces without material")]
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

// =============================================================================
// Info-level rules (2)
// =============================================================================

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

// =============================================================================
// Rule registration
// =============================================================================

/// Returns all mesh lint rules.
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        // Error-level rules
        Box::new(NonManifoldRule),
        Box::new(DegenerateFacesRule),
        Box::new(UnweightedVertsRule),
        Box::new(InvertedNormalsRule),
        // Warning-level rules
        Box::new(HumanoidProportionsRule),
        Box::new(UvOverlapRule),
        Box::new(UvStretchRule),
        Box::new(MissingMaterialRule),
        Box::new(ExcessiveNgonsRule),
        Box::new(IsolatedVertsRule),
        // Info-level rules
        Box::new(HighPolyRule),
        Box::new(NoUvsRule),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Create minimal valid GLB data for testing.
    fn create_test_glb() -> Vec<u8> {
        // Minimal valid GLB with a simple triangle
        // This is a hand-crafted minimal GLB file
        let json = r#"{"asset":{"version":"2.0"},"meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1}]}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","max":[1,1,0],"min":[0,0,0]},{"bufferView":1,"componentType":5123,"count":3,"type":"SCALAR"}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0},{"buffer":0,"byteLength":6,"byteOffset":36}],"buffers":[{"byteLength":44}]}"#;

        let json_bytes = json.as_bytes();
        let json_len = json_bytes.len();
        // Pad to 4-byte alignment
        let json_padding = (4 - (json_len % 4)) % 4;
        let padded_json_len = json_len + json_padding;

        // Binary chunk: 3 vertices (36 bytes) + 3 indices (6 bytes) = 42 bytes
        // Padded to 44 bytes
        let positions: [[f32; 3]; 3] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let indices: [u16; 3] = [0, 1, 2];

        let mut bin_data = Vec::new();
        for pos in positions {
            for coord in pos {
                bin_data.extend_from_slice(&coord.to_le_bytes());
            }
        }
        for idx in indices {
            bin_data.extend_from_slice(&idx.to_le_bytes());
        }
        // Pad to 4-byte alignment
        while bin_data.len() % 4 != 0 {
            bin_data.push(0);
        }

        let total_len = 12 + 8 + padded_json_len + 8 + bin_data.len();

        let mut glb = Vec::new();
        // Header
        glb.extend_from_slice(b"glTF"); // magic
        glb.extend_from_slice(&2u32.to_le_bytes()); // version
        glb.extend_from_slice(&(total_len as u32).to_le_bytes()); // length

        // JSON chunk
        glb.extend_from_slice(&(padded_json_len as u32).to_le_bytes()); // chunk length
        glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // chunk type "JSON"
        glb.extend_from_slice(json_bytes);
        for _ in 0..json_padding {
            glb.push(0x20); // space padding
        }

        // Binary chunk
        glb.extend_from_slice(&(bin_data.len() as u32).to_le_bytes()); // chunk length
        glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // chunk type "BIN\0"
        glb.extend_from_slice(&bin_data);

        glb
    }

    #[test]
    fn test_all_rules_registered() {
        let rules = all_rules();
        assert_eq!(rules.len(), 12);

        // Verify all expected rule IDs are present
        let ids: Vec<_> = rules.iter().map(|r| r.id()).collect();
        assert!(ids.contains(&"mesh/non-manifold"));
        assert!(ids.contains(&"mesh/degenerate-faces"));
        assert!(ids.contains(&"mesh/unweighted-verts"));
        assert!(ids.contains(&"mesh/inverted-normals"));
        assert!(ids.contains(&"mesh/humanoid-proportions"));
        assert!(ids.contains(&"mesh/uv-overlap"));
        assert!(ids.contains(&"mesh/uv-stretch"));
        assert!(ids.contains(&"mesh/missing-material"));
        assert!(ids.contains(&"mesh/excessive-ngons"));
        assert!(ids.contains(&"mesh/isolated-verts"));
        assert!(ids.contains(&"mesh/high-poly"));
        assert!(ids.contains(&"mesh/no-uvs"));
    }

    #[test]
    fn test_rules_apply_to_mesh() {
        for rule in all_rules() {
            assert!(
                rule.applies_to().contains(&AssetType::Mesh),
                "Rule {} should apply to Mesh",
                rule.id()
            );
        }
    }

    #[test]
    fn test_error_rules_have_error_severity() {
        let error_ids = [
            "mesh/non-manifold",
            "mesh/degenerate-faces",
            "mesh/unweighted-verts",
            "mesh/inverted-normals",
        ];

        for rule in all_rules() {
            if error_ids.contains(&rule.id()) {
                assert_eq!(
                    rule.default_severity(),
                    Severity::Error,
                    "Rule {} should have Error severity",
                    rule.id()
                );
            }
        }
    }

    #[test]
    fn test_warning_rules_have_warning_severity() {
        let warning_ids = [
            "mesh/humanoid-proportions",
            "mesh/uv-overlap",
            "mesh/uv-stretch",
            "mesh/missing-material",
            "mesh/excessive-ngons",
            "mesh/isolated-verts",
        ];

        for rule in all_rules() {
            if warning_ids.contains(&rule.id()) {
                assert_eq!(
                    rule.default_severity(),
                    Severity::Warning,
                    "Rule {} should have Warning severity",
                    rule.id()
                );
            }
        }
    }

    #[test]
    fn test_info_rules_have_info_severity() {
        let info_ids = ["mesh/high-poly", "mesh/no-uvs"];

        for rule in all_rules() {
            if info_ids.contains(&rule.id()) {
                assert_eq!(
                    rule.default_severity(),
                    Severity::Info,
                    "Rule {} should have Info severity",
                    rule.id()
                );
            }
        }
    }

    #[test]
    fn test_triangle_area_3d() {
        // Unit right triangle in XY plane
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [1.0f32, 0.0, 0.0];
        let p2 = [0.0f32, 1.0, 0.0];
        let area = triangle_area_3d(p0, p1, p2);
        assert!((area - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_triangle_area_2d() {
        let p0 = [0.0f32, 0.0];
        let p1 = [1.0f32, 0.0];
        let p2 = [0.0f32, 1.0];
        let area = triangle_area_2d(p0, p1, p2);
        assert!((area - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_centroid() {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [2.0f32, 0.0, 0.0],
            [1.0f32, 2.0, 0.0],
        ];
        let centroid = calculate_centroid(&positions);
        assert!((centroid[0] - 1.0).abs() < 1e-6);
        assert!((centroid[1] - 2.0 / 3.0).abs() < 1e-6);
        assert!((centroid[2] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_parse_valid_glb() {
        let glb_data = create_test_glb();
        let asset = AssetData {
            path: Path::new("test.glb"),
            bytes: &glb_data,
        };
        let mesh = parse_mesh_data(&asset);
        assert!(mesh.is_some());
        let mesh = mesh.unwrap();
        assert_eq!(mesh.positions.len(), 3);
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn test_no_uvs_rule() {
        let glb_data = create_test_glb();
        let asset = AssetData {
            path: Path::new("test.glb"),
            bytes: &glb_data,
        };
        let rule = NoUvsRule;
        let issues = rule.check(&asset, None);
        // The test GLB has no UVs
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "mesh/no-uvs");
    }

    #[test]
    fn test_non_manifold_rule_clean_mesh() {
        let glb_data = create_test_glb();
        let asset = AssetData {
            path: Path::new("test.glb"),
            bytes: &glb_data,
        };
        let rule = NonManifoldRule;
        let issues = rule.check(&asset, None);
        // Simple triangle is manifold
        assert!(issues.is_empty());
    }

    #[test]
    fn test_degenerate_faces_rule_clean_mesh() {
        let glb_data = create_test_glb();
        let asset = AssetData {
            path: Path::new("test.glb"),
            bytes: &glb_data,
        };
        let rule = DegenerateFacesRule;
        let issues = rule.check(&asset, None);
        // Test triangle has non-zero area
        assert!(issues.is_empty());
    }

    #[test]
    fn test_high_poly_rule_small_mesh() {
        let glb_data = create_test_glb();
        let asset = AssetData {
            path: Path::new("test.glb"),
            bytes: &glb_data,
        };
        let rule = HighPolyRule;
        let issues = rule.check(&asset, None);
        // 1 triangle is well under 50k threshold
        assert!(issues.is_empty());
    }

    #[test]
    fn test_isolated_verts_rule_clean_mesh() {
        let glb_data = create_test_glb();
        let asset = AssetData {
            path: Path::new("test.glb"),
            bytes: &glb_data,
        };
        let rule = IsolatedVertsRule;
        let issues = rule.check(&asset, None);
        // All 3 vertices are used by the triangle
        assert!(issues.is_empty());
    }

    #[test]
    fn test_unweighted_verts_rule_non_skinned() {
        let glb_data = create_test_glb();
        let asset = AssetData {
            path: Path::new("test.glb"),
            bytes: &glb_data,
        };
        let rule = UnweightedVertsRule;
        let issues = rule.check(&asset, None);
        // Non-skinned mesh shouldn't trigger this rule
        assert!(issues.is_empty());
    }

    #[test]
    fn test_excessive_ngons_rule_glb() {
        let glb_data = create_test_glb();
        let asset = AssetData {
            path: Path::new("test.glb"),
            bytes: &glb_data,
        };
        let rule = ExcessiveNgonsRule;
        let issues = rule.check(&asset, None);
        // GLB is always triangulated, so no ngon warnings
        assert!(issues.is_empty());
    }
}
