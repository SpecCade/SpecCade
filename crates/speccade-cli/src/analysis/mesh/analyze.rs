//! Core mesh analysis logic.
//!
//! This module contains the main analysis functions and helper utilities
//! for extracting metrics from GLB/glTF files.

use super::accessors::{extract_glb_binary_chunk, read_indices, read_positions, read_uvs};
use super::types::{
    MeshAnimationMetrics, MeshBoundsMetrics, MeshFormatMetadata, MeshManifoldMetrics,
    MeshMaterialMetrics, MeshSkeletonMetrics, MeshTopologyMetrics, MeshUvMetrics,
};
use super::{MeshAnalysisError, MeshMetrics, FLOAT_PRECISION};
use std::collections::{HashMap, HashSet};

/// Round a float to the specified number of decimal places.
pub(super) fn round_f64(value: f64, decimals: i32) -> f64 {
    let multiplier = 10_f64.powi(decimals);
    (value * multiplier).round() / multiplier
}

/// Analyze a GLB file and return metrics.
pub fn analyze_glb(glb_data: &[u8]) -> Result<MeshMetrics, MeshAnalysisError> {
    let gltf = gltf::Gltf::from_slice(glb_data).map_err(|e| MeshAnalysisError::ParseError {
        message: e.to_string(),
    })?;

    analyze_gltf_document(&gltf, Some(glb_data))
}

/// Analyze a glTF JSON file (without embedded buffer data).
pub fn analyze_gltf_json(json_data: &[u8]) -> Result<MeshMetrics, MeshAnalysisError> {
    let gltf = gltf::Gltf::from_slice(json_data).map_err(|e| MeshAnalysisError::ParseError {
        message: e.to_string(),
    })?;

    analyze_gltf_document(&gltf, None)
}

/// Analyze a glTF document.
fn analyze_gltf_document(
    gltf: &gltf::Gltf,
    blob: Option<&[u8]>,
) -> Result<MeshMetrics, MeshAnalysisError> {
    let doc = &gltf.document;

    // Format metadata - use the document API for asset info
    let format_metadata = MeshFormatMetadata {
        format: if blob.is_some() { "glb" } else { "gltf" }.to_string(),
        gltf_version: "2.0".to_string(), // glTF crate only supports 2.0
        generator: None,                 // Not easily accessible via document API
        mesh_count: doc.meshes().count() as u32,
        node_count: doc.nodes().count() as u32,
    };

    // Collect all vertex positions and face indices for analysis
    let mut all_positions: Vec<[f32; 3]> = Vec::new();
    let mut all_indices: Vec<u32> = Vec::new();
    let mut all_uvs: Vec<[f32; 2]> = Vec::new();
    let mut total_triangles: u32 = 0;
    let mut uv_set_count: u32 = 0;

    // Get the blob data for reading accessors
    let blob_data = blob.and_then(|b| extract_glb_binary_chunk(b));

    // Iterate through all meshes
    for mesh in doc.meshes() {
        for primitive in mesh.primitives() {
            // Count triangles
            if let Some(indices_accessor) = primitive.indices() {
                let index_count = indices_accessor.count();
                total_triangles += (index_count / 3) as u32;

                // Try to read actual indices if we have blob data
                if let Some(data) = blob_data {
                    read_indices(&indices_accessor, data, &mut all_indices);
                }
            } else {
                // Non-indexed geometry: count from position accessor
                if let Some(positions) = primitive.get(&gltf::Semantic::Positions) {
                    total_triangles += (positions.count() / 3) as u32;
                }
            }

            // Read positions
            if let Some(positions_accessor) = primitive.get(&gltf::Semantic::Positions) {
                if let Some(data) = blob_data {
                    read_positions(&positions_accessor, data, &mut all_positions);
                }
            }

            // Check for UV coordinates
            if primitive.get(&gltf::Semantic::TexCoords(0)).is_some() {
                uv_set_count = uv_set_count.max(1);
                if let Some(uv_accessor) = primitive.get(&gltf::Semantic::TexCoords(0)) {
                    if let Some(data) = blob_data {
                        read_uvs(&uv_accessor, data, &mut all_uvs);
                    }
                }
            }
            if primitive.get(&gltf::Semantic::TexCoords(1)).is_some() {
                uv_set_count = uv_set_count.max(2);
            }
        }
    }

    let vertex_count = all_positions.len() as u32;
    let face_count = total_triangles;

    if vertex_count == 0 && face_count == 0 {
        return Err(MeshAnalysisError::EmptyMesh);
    }

    // Estimate edge count from triangles (Euler formula approximation)
    let edge_count = if total_triangles > 0 {
        // For a closed mesh: E = F * 3 / 2 (each edge shared by 2 faces)
        // For an open mesh or with boundary: slightly higher
        (total_triangles * 3 / 2) + (total_triangles / 10)
    } else {
        0
    };

    // Topology metrics
    let topology = MeshTopologyMetrics {
        vertex_count,
        face_count,
        edge_count,
        triangle_count: total_triangles,
        quad_count: 0, // glTF uses triangles
        quad_percentage: 0.0,
    };

    // Manifold analysis
    let manifold_metrics = analyze_manifold(&all_positions, &all_indices);

    // UV analysis
    let uv_metrics = analyze_uvs(&all_uvs, uv_set_count);

    // Bounds analysis
    let bounds = calculate_bounds(&all_positions);

    // Skeleton analysis
    let skeleton = analyze_skeleton(doc);

    // Material analysis
    let materials = MeshMaterialMetrics {
        material_count: doc.materials().count() as u32,
        texture_count: doc.textures().count() as u32,
    };

    // Animation analysis
    let animation = analyze_animations(doc);

    Ok(MeshMetrics {
        format: format_metadata,
        topology,
        manifold: manifold_metrics,
        uv: uv_metrics,
        bounds,
        skeleton,
        materials,
        animation,
    })
}

/// Analyze manifold properties of the mesh.
pub(super) fn analyze_manifold(positions: &[[f32; 3]], indices: &[u32]) -> MeshManifoldMetrics {
    if indices.is_empty() || positions.is_empty() {
        return MeshManifoldMetrics {
            manifold: true,
            non_manifold_edge_count: 0,
            degenerate_face_count: 0,
        };
    }

    // Count edge usage (each edge should be used by exactly 2 faces for manifold)
    let mut edge_count_map: HashMap<(u32, u32), u32> = HashMap::new();
    let mut degenerate_count = 0u32;

    for tri in indices.chunks(3) {
        if tri.len() != 3 {
            continue;
        }

        let v0 = tri[0];
        let v1 = tri[1];
        let v2 = tri[2];

        // Check for degenerate triangle (duplicate vertices)
        if v0 == v1 || v1 == v2 || v0 == v2 {
            degenerate_count += 1;
            continue;
        }

        // Check for zero-area triangle
        if (v0 as usize) < positions.len()
            && (v1 as usize) < positions.len()
            && (v2 as usize) < positions.len()
        {
            let p0 = positions[v0 as usize];
            let p1 = positions[v1 as usize];
            let p2 = positions[v2 as usize];
            let area = triangle_area_3d(p0, p1, p2);
            if area < 1e-10 {
                degenerate_count += 1;
            }
        }

        // Count edges
        for (a, b) in [(v0, v1), (v1, v2), (v2, v0)] {
            let edge = if a < b { (a, b) } else { (b, a) };
            *edge_count_map.entry(edge).or_insert(0) += 1;
        }
    }

    // Count non-manifold edges (not shared by exactly 2 faces)
    let non_manifold_edges = edge_count_map.values().filter(|&&c| c != 2).count() as u32;

    MeshManifoldMetrics {
        manifold: non_manifold_edges == 0 && degenerate_count == 0,
        non_manifold_edge_count: non_manifold_edges,
        degenerate_face_count: degenerate_count,
    }
}

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

/// Analyze UV coordinates.
fn analyze_uvs(uvs: &[[f32; 2]], uv_set_count: u32) -> MeshUvMetrics {
    if uvs.is_empty() {
        return MeshUvMetrics {
            has_uvs: uv_set_count > 0,
            uv_set_count,
            uv_island_count: 0,
            uv_coverage: 0.0,
            uv_overlap_percentage: 0.0,
        };
    }

    // Estimate UV island count by counting disconnected UV regions
    // This is a simplified approximation
    let island_count = estimate_uv_islands(uvs);

    // Calculate UV coverage and overlap
    let (coverage, overlap) = calculate_uv_coverage(uvs);

    MeshUvMetrics {
        has_uvs: true,
        uv_set_count: uv_set_count.max(1),
        uv_island_count: island_count,
        uv_coverage: round_f64(coverage, FLOAT_PRECISION),
        uv_overlap_percentage: round_f64(overlap, FLOAT_PRECISION),
    }
}

/// Estimate UV island count.
pub(super) fn estimate_uv_islands(uvs: &[[f32; 2]]) -> u32 {
    if uvs.is_empty() {
        return 0;
    }

    // Simple grid-based island detection
    // Quantize UV space into cells and count connected components
    let grid_size = 32;
    let mut grid: HashSet<(i32, i32)> = HashSet::new();

    for uv in uvs {
        let cell_x = ((uv[0].clamp(0.0, 1.0) * grid_size as f32) as i32).min(grid_size - 1);
        let cell_y = ((uv[1].clamp(0.0, 1.0) * grid_size as f32) as i32).min(grid_size - 1);
        grid.insert((cell_x, cell_y));
    }

    // Count connected components in the grid
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut islands = 0u32;

    for &cell in &grid {
        if visited.contains(&cell) {
            continue;
        }

        // BFS to find connected cells
        let mut stack = vec![cell];
        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            // Check 4-connected neighbors
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let neighbor = (current.0 + dx, current.1 + dy);
                if grid.contains(&neighbor) && !visited.contains(&neighbor) {
                    stack.push(neighbor);
                }
            }
        }
        islands += 1;
    }

    islands.max(1)
}

/// Calculate UV coverage and overlap.
fn calculate_uv_coverage(uvs: &[[f32; 2]]) -> (f64, f64) {
    if uvs.len() < 3 {
        return (0.0, 0.0);
    }

    // Calculate total UV area from triangles
    let mut total_area = 0.0f64;
    for tri in uvs.chunks(3) {
        if tri.len() == 3 {
            let area = triangle_area_2d(tri[0], tri[1], tri[2]);
            total_area += area.abs();
        }
    }

    // Coverage is clamped to [0, 1]
    let coverage = total_area.min(1.0);

    // Overlap estimation: if total area > 1.0, there's overlap
    let overlap = if total_area > 1.0 {
        ((total_area - 1.0) / total_area * 100.0).min(100.0)
    } else {
        0.0
    };

    (coverage, overlap)
}

/// Calculate 2D triangle area.
pub(super) fn triangle_area_2d(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2]) -> f64 {
    let v1 = [(p1[0] - p0[0]) as f64, (p1[1] - p0[1]) as f64];
    let v2 = [(p2[0] - p0[0]) as f64, (p2[1] - p0[1]) as f64];
    0.5 * (v1[0] * v2[1] - v1[1] * v2[0]).abs()
}

/// Calculate mesh bounding box.
pub(super) fn calculate_bounds(positions: &[[f32; 3]]) -> MeshBoundsMetrics {
    if positions.is_empty() {
        return MeshBoundsMetrics {
            bounds_min: [0.0, 0.0, 0.0],
            bounds_max: [0.0, 0.0, 0.0],
            size: [0.0, 0.0, 0.0],
            center: [0.0, 0.0, 0.0],
        };
    }

    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];

    for pos in positions {
        for i in 0..3 {
            min[i] = min[i].min(pos[i] as f64);
            max[i] = max[i].max(pos[i] as f64);
        }
    }

    let size = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let center = [
        (min[0] + max[0]) / 2.0,
        (min[1] + max[1]) / 2.0,
        (min[2] + max[2]) / 2.0,
    ];

    MeshBoundsMetrics {
        bounds_min: [
            round_f64(min[0], FLOAT_PRECISION),
            round_f64(min[1], FLOAT_PRECISION),
            round_f64(min[2], FLOAT_PRECISION),
        ],
        bounds_max: [
            round_f64(max[0], FLOAT_PRECISION),
            round_f64(max[1], FLOAT_PRECISION),
            round_f64(max[2], FLOAT_PRECISION),
        ],
        size: [
            round_f64(size[0], FLOAT_PRECISION),
            round_f64(size[1], FLOAT_PRECISION),
            round_f64(size[2], FLOAT_PRECISION),
        ],
        center: [
            round_f64(center[0], FLOAT_PRECISION),
            round_f64(center[1], FLOAT_PRECISION),
            round_f64(center[2], FLOAT_PRECISION),
        ],
    }
}

/// Analyze skeleton/armature.
fn analyze_skeleton(doc: &gltf::Document) -> Option<MeshSkeletonMetrics> {
    let skins: Vec<_> = doc.skins().collect();
    if skins.is_empty() {
        return None;
    }

    let mut total_bones = 0u32;
    let mut has_ibm = false;
    let mut max_influences = 4u32; // glTF default is 4

    for skin in &skins {
        total_bones += skin.joints().count() as u32;
        if skin.inverse_bind_matrices().is_some() {
            has_ibm = true;
        }
    }

    // Check for JOINTS_0/WEIGHTS_0 attributes to determine max influences
    for mesh in doc.meshes() {
        for primitive in mesh.primitives() {
            if primitive.get(&gltf::Semantic::Joints(0)).is_some() {
                // Standard glTF supports up to 4 influences per attribute set
                // Multiple sets (JOINTS_1, etc.) can extend this
                if primitive.get(&gltf::Semantic::Joints(1)).is_some() {
                    max_influences = 8;
                }
            }
        }
    }

    Some(MeshSkeletonMetrics {
        bone_count: total_bones,
        max_bone_influences: max_influences,
        has_inverse_bind_matrices: has_ibm,
    })
}

/// Analyze animations.
fn analyze_animations(doc: &gltf::Document) -> Option<MeshAnimationMetrics> {
    let animations: Vec<_> = doc.animations().collect();
    if animations.is_empty() {
        return None;
    }

    let animation_count = animations.len() as u32;
    let mut max_duration = 0.0f64;
    let mut total_keyframes = 0u32;

    for anim in &animations {
        for channel in anim.channels() {
            let sampler = channel.sampler();
            let input = sampler.input();
            total_keyframes += input.count() as u32;

            // Get the max time value from the input accessor
            if let Some(max) = input.max() {
                if let Some(vals) = max.as_array() {
                    if let Some(t) = vals.first().and_then(|v| v.as_f64()) {
                        max_duration = max_duration.max(t);
                    }
                }
            }
        }
    }

    // Estimate frame count at 30fps
    let frame_count = (max_duration * 30.0).ceil() as u32;

    Some(MeshAnimationMetrics {
        animation_count,
        total_frame_count: frame_count.max(total_keyframes),
        total_duration_seconds: round_f64(max_duration, FLOAT_PRECISION),
    })
}
