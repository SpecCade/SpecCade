//! Mesh analysis module for extracting geometric metrics from GLB/glTF files.
//!
//! This module provides deterministic mesh analysis for Tier-2 mesh/character/animation
//! assets. All metrics are computed to produce byte-identical JSON output across runs
//! on the same input.

mod accessors;
mod analyze;
#[cfg(test)]
mod tests;
mod types;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Re-export public types
pub use analyze::{analyze_glb, analyze_gltf_json};
pub use types::{
    MeshAnimationMetrics, MeshBoundsMetrics, MeshFormatMetadata, MeshManifoldMetrics,
    MeshMaterialMetrics, MeshSkeletonMetrics, MeshTopologyMetrics, MeshUvMetrics,
};

/// Precision for floating point values in output (6 decimal places).
const FLOAT_PRECISION: i32 = 6;

/// Mesh analysis result containing all extracted metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshMetrics {
    /// Format metadata
    pub format: MeshFormatMetadata,
    /// Topology metrics
    pub topology: MeshTopologyMetrics,
    /// Manifold quality metrics
    pub manifold: MeshManifoldMetrics,
    /// UV mapping metrics
    pub uv: MeshUvMetrics,
    /// Bounding box metrics
    pub bounds: MeshBoundsMetrics,
    /// Skeleton metrics (if armature present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton: Option<MeshSkeletonMetrics>,
    /// Material metrics
    pub materials: MeshMaterialMetrics,
    /// Animation metrics (if animations present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<MeshAnimationMetrics>,
}

/// Error type for mesh analysis.
#[derive(Debug, Clone)]
pub enum MeshAnalysisError {
    /// File read error
    FileRead { message: String },
    /// Invalid glTF/GLB format
    InvalidFormat { message: String },
    /// Empty mesh (no geometry)
    EmptyMesh,
    /// Parse error
    ParseError { message: String },
}

impl std::fmt::Display for MeshAnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshAnalysisError::FileRead { message } => {
                write!(f, "Failed to read mesh file: {}", message)
            }
            MeshAnalysisError::InvalidFormat { message } => {
                write!(f, "Invalid glTF/GLB format: {}", message)
            }
            MeshAnalysisError::EmptyMesh => {
                write!(f, "Mesh has no geometry")
            }
            MeshAnalysisError::ParseError { message } => {
                write!(f, "Parse error: {}", message)
            }
        }
    }
}

impl std::error::Error for MeshAnalysisError {}

/// Convert MeshMetrics to a BTreeMap for deterministic JSON serialization.
pub fn metrics_to_btree(metrics: &MeshMetrics) -> BTreeMap<String, serde_json::Value> {
    let mut map = BTreeMap::new();

    // Animation (if present)
    if let Some(ref anim) = metrics.animation {
        let mut anim_map = BTreeMap::new();
        anim_map.insert(
            "animation_count".to_string(),
            serde_json::json!(anim.animation_count),
        );
        anim_map.insert(
            "total_duration_seconds".to_string(),
            serde_json::json!(anim.total_duration_seconds),
        );
        anim_map.insert(
            "total_frame_count".to_string(),
            serde_json::json!(anim.total_frame_count),
        );
        map.insert("animation".to_string(), serde_json::json!(anim_map));
    }

    // Bounds
    let mut bounds_map = BTreeMap::new();
    bounds_map.insert(
        "bounds_max".to_string(),
        serde_json::json!(metrics.bounds.bounds_max),
    );
    bounds_map.insert(
        "bounds_min".to_string(),
        serde_json::json!(metrics.bounds.bounds_min),
    );
    bounds_map.insert(
        "center".to_string(),
        serde_json::json!(metrics.bounds.center),
    );
    bounds_map.insert("size".to_string(), serde_json::json!(metrics.bounds.size));
    map.insert("bounds".to_string(), serde_json::json!(bounds_map));

    // Format
    let mut format_map = BTreeMap::new();
    format_map.insert(
        "format".to_string(),
        serde_json::json!(metrics.format.format),
    );
    if let Some(ref gen) = metrics.format.generator {
        format_map.insert("generator".to_string(), serde_json::json!(gen));
    }
    format_map.insert(
        "gltf_version".to_string(),
        serde_json::json!(metrics.format.gltf_version),
    );
    format_map.insert(
        "mesh_count".to_string(),
        serde_json::json!(metrics.format.mesh_count),
    );
    format_map.insert(
        "node_count".to_string(),
        serde_json::json!(metrics.format.node_count),
    );
    map.insert("format".to_string(), serde_json::json!(format_map));

    // Manifold
    let mut manifold_map = BTreeMap::new();
    manifold_map.insert(
        "degenerate_face_count".to_string(),
        serde_json::json!(metrics.manifold.degenerate_face_count),
    );
    manifold_map.insert(
        "manifold".to_string(),
        serde_json::json!(metrics.manifold.manifold),
    );
    manifold_map.insert(
        "non_manifold_edge_count".to_string(),
        serde_json::json!(metrics.manifold.non_manifold_edge_count),
    );
    map.insert("manifold".to_string(), serde_json::json!(manifold_map));

    // Materials
    let mut materials_map = BTreeMap::new();
    materials_map.insert(
        "material_count".to_string(),
        serde_json::json!(metrics.materials.material_count),
    );
    materials_map.insert(
        "texture_count".to_string(),
        serde_json::json!(metrics.materials.texture_count),
    );
    map.insert("materials".to_string(), serde_json::json!(materials_map));

    // Skeleton (if present)
    if let Some(ref skel) = metrics.skeleton {
        let mut skel_map = BTreeMap::new();
        skel_map.insert("bone_count".to_string(), serde_json::json!(skel.bone_count));
        skel_map.insert(
            "has_inverse_bind_matrices".to_string(),
            serde_json::json!(skel.has_inverse_bind_matrices),
        );
        skel_map.insert(
            "max_bone_influences".to_string(),
            serde_json::json!(skel.max_bone_influences),
        );
        map.insert("skeleton".to_string(), serde_json::json!(skel_map));
    }

    // Topology
    let mut topo_map = BTreeMap::new();
    topo_map.insert(
        "edge_count".to_string(),
        serde_json::json!(metrics.topology.edge_count),
    );
    topo_map.insert(
        "face_count".to_string(),
        serde_json::json!(metrics.topology.face_count),
    );
    topo_map.insert(
        "quad_count".to_string(),
        serde_json::json!(metrics.topology.quad_count),
    );
    topo_map.insert(
        "quad_percentage".to_string(),
        serde_json::json!(metrics.topology.quad_percentage),
    );
    topo_map.insert(
        "triangle_count".to_string(),
        serde_json::json!(metrics.topology.triangle_count),
    );
    topo_map.insert(
        "vertex_count".to_string(),
        serde_json::json!(metrics.topology.vertex_count),
    );
    map.insert("topology".to_string(), serde_json::json!(topo_map));

    // UV
    let mut uv_map = BTreeMap::new();
    uv_map.insert("has_uvs".to_string(), serde_json::json!(metrics.uv.has_uvs));
    uv_map.insert(
        "uv_coverage".to_string(),
        serde_json::json!(metrics.uv.uv_coverage),
    );
    uv_map.insert(
        "uv_island_count".to_string(),
        serde_json::json!(metrics.uv.uv_island_count),
    );
    uv_map.insert(
        "uv_overlap_percentage".to_string(),
        serde_json::json!(metrics.uv.uv_overlap_percentage),
    );
    uv_map.insert(
        "uv_set_count".to_string(),
        serde_json::json!(metrics.uv.uv_set_count),
    );
    map.insert("uv".to_string(), serde_json::json!(uv_map));

    map
}
