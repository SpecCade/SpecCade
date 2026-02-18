//! GLB/glTF parsing helpers and mesh data extraction.

use crate::rules::AssetData;

/// Extract the binary chunk from a GLB file.
pub(super) fn extract_glb_binary_chunk(b: &[u8]) -> Option<&[u8]> {
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

// =============================================================================
// Geometry helpers
// =============================================================================

/// Calculate 3D triangle area using cross product.
pub(super) fn triangle_area_3d(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f64 {
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
pub(super) fn triangle_area_2d(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2]) -> f64 {
    let v1 = [(p1[0] - p0[0]) as f64, (p1[1] - p0[1]) as f64];
    let v2 = [(p2[0] - p0[0]) as f64, (p2[1] - p0[1]) as f64];
    0.5 * (v1[0] * v2[1] - v1[1] * v2[0]).abs()
}

/// Calculate face normal from three vertices.
pub(super) fn calculate_face_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
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
pub(super) fn calculate_centroid(positions: &[[f32; 3]]) -> [f32; 3] {
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
    [
        (sum[0] / n) as f32,
        (sum[1] / n) as f32,
        (sum[2] / n) as f32,
    ]
}

/// Dot product of two 3D vectors.
pub(super) fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

// =============================================================================
// MeshData struct
// =============================================================================

/// Parsed mesh data for lint analysis.
pub(super) struct MeshData {
    pub(super) positions: Vec<[f32; 3]>,
    /// Stored for future normal consistency checks.
    #[allow(dead_code)]
    pub(super) normals: Vec<[f32; 3]>,
    pub(super) indices: Vec<u32>,
    pub(super) uvs: Vec<[f32; 2]>,
    pub(super) weights: Vec<[f32; 4]>,
    pub(super) has_skeleton: bool,
    pub(super) material_indices: Vec<Option<usize>>,
    pub(super) bone_names: Vec<String>,
}

impl MeshData {
    pub(super) fn from_gltf(gltf: &gltf::Gltf, blob: Option<&[u8]>) -> Self {
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
pub(super) fn parse_mesh_data(asset: &AssetData) -> Option<MeshData> {
    let gltf = gltf::Gltf::from_slice(asset.bytes).ok()?;
    Some(MeshData::from_gltf(&gltf, Some(asset.bytes)))
}
