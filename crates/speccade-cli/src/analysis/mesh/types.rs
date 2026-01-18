//! Metric types for mesh analysis.
//!
//! This module contains all the nested metric structs used by `MeshMetrics`.

use serde::{Deserialize, Serialize};

/// Format metadata for the mesh file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshFormatMetadata {
    /// File format (glb or gltf)
    pub format: String,
    /// glTF version
    pub gltf_version: String,
    /// Generator string (if present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
    /// Number of meshes in the file
    pub mesh_count: u32,
    /// Number of nodes in the scene
    pub node_count: u32,
}

/// Topology metrics for mesh geometry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTopologyMetrics {
    /// Total number of vertices
    pub vertex_count: u32,
    /// Total number of faces (primitives)
    pub face_count: u32,
    /// Total number of edges (estimated from face topology)
    pub edge_count: u32,
    /// Number of triangles
    pub triangle_count: u32,
    /// Number of quads (4-vertex faces, rare in glTF which uses triangles)
    pub quad_count: u32,
    /// Percentage of faces that are quads (0.0-100.0)
    pub quad_percentage: f64,
}

/// Manifold quality metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshManifoldMetrics {
    /// Whether the mesh appears manifold (no detected issues)
    pub manifold: bool,
    /// Number of non-manifold edges (edges shared by != 2 faces)
    pub non_manifold_edge_count: u32,
    /// Number of degenerate faces (zero area triangles)
    pub degenerate_face_count: u32,
}

/// UV mapping metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshUvMetrics {
    /// Whether the mesh has UV coordinates
    pub has_uvs: bool,
    /// Number of UV sets/channels
    pub uv_set_count: u32,
    /// Estimated UV island count (approximate)
    pub uv_island_count: u32,
    /// UV coverage ratio (0.0-1.0)
    pub uv_coverage: f64,
    /// UV overlap percentage (0.0-100.0)
    pub uv_overlap_percentage: f64,
}

/// Bounding box metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshBoundsMetrics {
    /// Minimum corner [x, y, z]
    pub bounds_min: [f64; 3],
    /// Maximum corner [x, y, z]
    pub bounds_max: [f64; 3],
    /// Size in each dimension [x, y, z]
    pub size: [f64; 3],
    /// Center point [x, y, z]
    pub center: [f64; 3],
}

/// Skeleton/armature metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSkeletonMetrics {
    /// Number of bones/joints
    pub bone_count: u32,
    /// Maximum bone influences per vertex
    pub max_bone_influences: u32,
    /// Whether the skeleton has an inverse bind matrices accessor
    pub has_inverse_bind_matrices: bool,
}

/// Material metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshMaterialMetrics {
    /// Number of materials
    pub material_count: u32,
    /// Number of textures referenced
    pub texture_count: u32,
}

/// Animation metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshAnimationMetrics {
    /// Number of animation clips
    pub animation_count: u32,
    /// Total frame count (estimated from keyframes)
    pub total_frame_count: u32,
    /// Total duration in seconds (longest animation)
    pub total_duration_seconds: f64,
}
