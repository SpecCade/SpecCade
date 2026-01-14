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
        }
    }
}

/// Validation metrics for Tier 2 outputs (GLB meshes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputMetrics {
    /// Number of triangles in the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triangle_count: Option<u32>,
    /// Bounding box of the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    /// Number of UV islands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_island_count: Option<u32>,
    /// Number of bones in the skeleton.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone_count: Option<u32>,
    /// Number of material slots.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_slot_count: Option<u32>,
    /// Maximum number of bone influences per vertex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bone_influences: Option<u32>,
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
            triangle_count: None,
            bounding_box: None,
            uv_island_count: None,
            bone_count: None,
            material_slot_count: None,
            max_bone_influences: None,
            animation_frame_count: None,
            animation_duration_seconds: None,
        }
    }

    /// Sets the triangle count.
    pub fn with_triangle_count(mut self, count: u32) -> Self {
        self.triangle_count = Some(count);
        self
    }

    /// Sets the bounding box.
    pub fn with_bounding_box(mut self, bbox: BoundingBox) -> Self {
        self.bounding_box = Some(bbox);
        self
    }

    /// Sets the UV island count.
    pub fn with_uv_island_count(mut self, count: u32) -> Self {
        self.uv_island_count = Some(count);
        self
    }

    /// Sets the bone count.
    pub fn with_bone_count(mut self, count: u32) -> Self {
        self.bone_count = Some(count);
        self
    }

    /// Sets the material slot count.
    pub fn with_material_slot_count(mut self, count: u32) -> Self {
        self.material_slot_count = Some(count);
        self
    }

    /// Sets the maximum bone influences.
    pub fn with_max_bone_influences(mut self, max: u32) -> Self {
        self.max_bone_influences = Some(max);
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
