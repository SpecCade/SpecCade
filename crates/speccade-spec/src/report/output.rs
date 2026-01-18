//! Output result and metrics types for reports.

use crate::output::{OutputFormat, OutputKind};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Result entry for a single output artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputResult {
    /// The kind of output (primary, metadata, preview).
    pub kind: OutputKind,
    /// The file format.
    pub format: OutputFormat,
    /// Relative path where the artifact was written.
    pub path: PathBuf,
    /// Hex-encoded BLAKE3 hash (for Tier 1 outputs only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Validation metrics (for Tier 2 outputs only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<OutputMetrics>,
    /// Whether this output was generated in preview mode (truncated for fast iteration).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
}

impl OutputResult {
    /// Creates a new Tier 1 output result with a hash.
    pub fn tier1(kind: OutputKind, format: OutputFormat, path: PathBuf, hash: String) -> Self {
        Self {
            kind,
            format,
            path,
            hash: Some(hash),
            metrics: None,
            preview: None,
        }
    }

    /// Creates a new Tier 2 output result with metrics.
    pub fn tier2(
        kind: OutputKind,
        format: OutputFormat,
        path: PathBuf,
        metrics: OutputMetrics,
    ) -> Self {
        Self {
            kind,
            format,
            path,
            hash: None,
            metrics: Some(metrics),
            preview: None,
        }
    }
}

/// Validation metrics for Tier 2 outputs (GLB meshes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputMetrics {
    // ========== Topology metrics ==========
    /// Number of vertices in the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertex_count: Option<u32>,
    /// Number of faces (polygons) in the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_count: Option<u32>,
    /// Number of edges in the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_count: Option<u32>,
    /// Number of triangles in the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triangle_count: Option<u32>,
    /// Number of quad faces in the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quad_count: Option<u32>,
    /// Percentage of faces that are quads (0.0-100.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quad_percentage: Option<f64>,

    // ========== Manifold metrics ==========
    /// Whether the mesh is manifold (watertight).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifold: Option<bool>,
    /// Number of non-manifold edges.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_manifold_edge_count: Option<u32>,
    /// Number of degenerate faces (zero area or invalid topology).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degenerate_face_count: Option<u32>,

    // ========== UV metrics ==========
    /// Number of UV islands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_island_count: Option<u32>,
    /// UV coverage ratio (0.0-1.0), how much of UV space is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_coverage: Option<f64>,
    /// Percentage of UV space that overlaps (0.0-100.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_overlap_percentage: Option<f64>,

    // ========== Bounds metrics ==========
    /// Bounding box of the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    /// Minimum corner of bounding box [x, y, z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds_min: Option<[f64; 3]>,
    /// Maximum corner of bounding box [x, y, z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds_max: Option<[f64; 3]>,

    // ========== Skeleton metrics ==========
    /// Number of bones in the skeleton.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone_count: Option<u32>,
    /// Maximum number of bone influences per vertex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bone_influences: Option<u32>,

    // ========== Material metrics ==========
    /// Number of material slots.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_slot_count: Option<u32>,

    // ========== Animation metrics ==========
    /// Number of animation frames.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation_frame_count: Option<u32>,
    /// Duration of animation in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation_duration_seconds: Option<f32>,
}

impl OutputMetrics {
    /// Creates a new empty metrics object.
    pub fn new() -> Self {
        Self {
            vertex_count: None,
            face_count: None,
            edge_count: None,
            triangle_count: None,
            quad_count: None,
            quad_percentage: None,
            manifold: None,
            non_manifold_edge_count: None,
            degenerate_face_count: None,
            uv_island_count: None,
            uv_coverage: None,
            uv_overlap_percentage: None,
            bounding_box: None,
            bounds_min: None,
            bounds_max: None,
            bone_count: None,
            max_bone_influences: None,
            material_slot_count: None,
            animation_frame_count: None,
            animation_duration_seconds: None,
        }
    }

    /// Sets the vertex count.
    pub fn with_vertex_count(mut self, count: u32) -> Self {
        self.vertex_count = Some(count);
        self
    }

    /// Sets the face count.
    pub fn with_face_count(mut self, count: u32) -> Self {
        self.face_count = Some(count);
        self
    }

    /// Sets the edge count.
    pub fn with_edge_count(mut self, count: u32) -> Self {
        self.edge_count = Some(count);
        self
    }

    /// Sets the triangle count.
    pub fn with_triangle_count(mut self, count: u32) -> Self {
        self.triangle_count = Some(count);
        self
    }

    /// Sets the quad count.
    pub fn with_quad_count(mut self, count: u32) -> Self {
        self.quad_count = Some(count);
        self
    }

    /// Sets the quad percentage.
    pub fn with_quad_percentage(mut self, percentage: f64) -> Self {
        self.quad_percentage = Some(percentage);
        self
    }

    /// Sets whether the mesh is manifold.
    pub fn with_manifold(mut self, manifold: bool) -> Self {
        self.manifold = Some(manifold);
        self
    }

    /// Sets the non-manifold edge count.
    pub fn with_non_manifold_edge_count(mut self, count: u32) -> Self {
        self.non_manifold_edge_count = Some(count);
        self
    }

    /// Sets the degenerate face count.
    pub fn with_degenerate_face_count(mut self, count: u32) -> Self {
        self.degenerate_face_count = Some(count);
        self
    }

    /// Sets the UV island count.
    pub fn with_uv_island_count(mut self, count: u32) -> Self {
        self.uv_island_count = Some(count);
        self
    }

    /// Sets the UV coverage.
    pub fn with_uv_coverage(mut self, coverage: f64) -> Self {
        self.uv_coverage = Some(coverage);
        self
    }

    /// Sets the UV overlap percentage.
    pub fn with_uv_overlap_percentage(mut self, percentage: f64) -> Self {
        self.uv_overlap_percentage = Some(percentage);
        self
    }

    /// Sets the bounding box.
    pub fn with_bounding_box(mut self, bbox: BoundingBox) -> Self {
        self.bounding_box = Some(bbox);
        self
    }

    /// Sets the bounds min.
    pub fn with_bounds_min(mut self, min: [f64; 3]) -> Self {
        self.bounds_min = Some(min);
        self
    }

    /// Sets the bounds max.
    pub fn with_bounds_max(mut self, max: [f64; 3]) -> Self {
        self.bounds_max = Some(max);
        self
    }

    /// Sets the bone count.
    pub fn with_bone_count(mut self, count: u32) -> Self {
        self.bone_count = Some(count);
        self
    }

    /// Sets the maximum bone influences.
    pub fn with_max_bone_influences(mut self, max: u32) -> Self {
        self.max_bone_influences = Some(max);
        self
    }

    /// Sets the material slot count.
    pub fn with_material_slot_count(mut self, count: u32) -> Self {
        self.material_slot_count = Some(count);
        self
    }

    /// Sets the animation frame count.
    pub fn with_animation_frame_count(mut self, count: u32) -> Self {
        self.animation_frame_count = Some(count);
        self
    }

    /// Sets the animation duration.
    pub fn with_animation_duration_seconds(mut self, duration: f32) -> Self {
        self.animation_duration_seconds = Some(duration);
        self
    }
}

impl Default for OutputMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundingBox {
    /// Minimum corner (x, y, z).
    pub min: [f32; 3],
    /// Maximum corner (x, y, z).
    pub max: [f32; 3],
}

impl BoundingBox {
    /// Creates a new bounding box.
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }
}
