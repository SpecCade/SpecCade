//! Output result and metrics types for reports.

use super::structural::StructuralMetrics;
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
    /// Number of zero-area faces (CHAR-003).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero_area_face_count: Option<u32>,

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
    /// Whether the mesh has at least one UV map (CHAR-003).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_uv_map: Option<bool>,
    /// Number of UV layers present on the mesh (MESH-002).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_layer_count: Option<u32>,
    /// Approximate texel density at a 1024x1024 reference texture (MESH-002).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texel_density: Option<f64>,

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

    // ========== Static mesh extra outputs/metrics (Blender Tier) ==========
    /// Number of generated LOD levels (MESH-004).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lod_count: Option<u32>,
    /// Per-LOD metrics (MESH-004).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lod_levels: Option<Vec<StaticMeshLodLevelMetrics>>,

    /// Collision mesh metrics (MESH-005).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collision_mesh: Option<CollisionMeshMetrics>,
    /// Collision mesh output path (relative, file name) if generated (MESH-005).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collision_mesh_path: Option<String>,

    /// Navmesh analysis metrics (MESH-006).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navmesh: Option<NavmeshMetrics>,

    /// Baking metrics (MESH-007).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baking: Option<BakingMetrics>,

    // ========== Skeleton metrics ==========
    /// Number of bones in the skeleton.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone_count: Option<u32>,
    /// Maximum number of bone influences per vertex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bone_influences: Option<u32>,
    /// Number of unweighted vertices (vertices with zero total skin weight).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unweighted_vertex_count: Option<u32>,
    /// Percentage of vertices with properly normalized weights (0.0-100.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_normalization_percentage: Option<f64>,
    /// Maximum weight sum deviation from 1.0 across all vertices (CHAR-003).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_weight_deviation: Option<f64>,

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

    // ========== Motion verification metrics (MESHVER-005) ==========
    /// Number of hinge axis violations (joints bending the wrong way).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hinge_axis_violations: Option<u32>,
    /// Number of range-of-motion violations (rotations exceeding limits).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_violations: Option<u32>,
    /// Number of velocity spikes (sudden direction reversals, "pops").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub velocity_spikes: Option<u32>,
    /// Root motion delta [X, Y, Z] from start to end of animation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_motion_delta: Option<[f32; 3]>,
    /// Root motion mode applied during export (keep, extract, bake_to_hip, lock).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_motion_mode: Option<String>,

    // ========== Structural metrics ==========
    /// Structural metrics for LLM-friendly 3D feedback.
    ///
    /// Describes geometric properties (proportions, symmetry, component
    /// relationships, skeletal structure) without encoding aesthetic opinions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structural: Option<StructuralMetrics>,
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
            zero_area_face_count: None,
            uv_island_count: None,
            uv_coverage: None,
            uv_overlap_percentage: None,
            has_uv_map: None,
            uv_layer_count: None,
            texel_density: None,
            bounding_box: None,
            bounds_min: None,
            bounds_max: None,
            lod_count: None,
            lod_levels: None,
            collision_mesh: None,
            collision_mesh_path: None,
            navmesh: None,
            baking: None,
            bone_count: None,
            max_bone_influences: None,
            unweighted_vertex_count: None,
            weight_normalization_percentage: None,
            max_weight_deviation: None,
            material_slot_count: None,
            animation_frame_count: None,
            animation_duration_seconds: None,
            hinge_axis_violations: None,
            range_violations: None,
            velocity_spikes: None,
            root_motion_delta: None,
            root_motion_mode: None,
            structural: None,
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

    /// Sets the zero-area face count.
    pub fn with_zero_area_face_count(mut self, count: u32) -> Self {
        self.zero_area_face_count = Some(count);
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

    /// Sets whether the mesh has a UV map.
    pub fn with_has_uv_map(mut self, has_uv: bool) -> Self {
        self.has_uv_map = Some(has_uv);
        self
    }

    /// Sets the UV layer count.
    pub fn with_uv_layer_count(mut self, count: u32) -> Self {
        self.uv_layer_count = Some(count);
        self
    }

    /// Sets the texel density.
    pub fn with_texel_density(mut self, density: f64) -> Self {
        self.texel_density = Some(density);
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

    /// Sets the LOD count.
    pub fn with_lod_count(mut self, count: u32) -> Self {
        self.lod_count = Some(count);
        self
    }

    /// Sets per-LOD metrics.
    pub fn with_lod_levels(mut self, levels: Vec<StaticMeshLodLevelMetrics>) -> Self {
        self.lod_levels = Some(levels);
        self
    }

    /// Sets collision mesh metrics.
    pub fn with_collision_mesh(mut self, metrics: CollisionMeshMetrics) -> Self {
        self.collision_mesh = Some(metrics);
        self
    }

    /// Sets collision mesh path.
    pub fn with_collision_mesh_path(mut self, path: impl Into<String>) -> Self {
        self.collision_mesh_path = Some(path.into());
        self
    }

    /// Sets navmesh metrics.
    pub fn with_navmesh(mut self, metrics: NavmeshMetrics) -> Self {
        self.navmesh = Some(metrics);
        self
    }

    /// Sets baking metrics.
    pub fn with_baking(mut self, metrics: BakingMetrics) -> Self {
        self.baking = Some(metrics);
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

    /// Sets the unweighted vertex count.
    pub fn with_unweighted_vertex_count(mut self, count: u32) -> Self {
        self.unweighted_vertex_count = Some(count);
        self
    }

    /// Sets the weight normalization percentage.
    pub fn with_weight_normalization_percentage(mut self, percentage: f64) -> Self {
        self.weight_normalization_percentage = Some(percentage);
        self
    }

    /// Sets the maximum weight deviation from 1.0.
    pub fn with_max_weight_deviation(mut self, deviation: f64) -> Self {
        self.max_weight_deviation = Some(deviation);
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

    /// Sets the hinge axis violations count.
    pub fn with_hinge_axis_violations(mut self, count: u32) -> Self {
        self.hinge_axis_violations = Some(count);
        self
    }

    /// Sets the range violations count.
    pub fn with_range_violations(mut self, count: u32) -> Self {
        self.range_violations = Some(count);
        self
    }

    /// Sets the velocity spikes count.
    pub fn with_velocity_spikes(mut self, count: u32) -> Self {
        self.velocity_spikes = Some(count);
        self
    }

    /// Sets the root motion delta.
    pub fn with_root_motion_delta(mut self, delta: [f32; 3]) -> Self {
        self.root_motion_delta = Some(delta);
        self
    }

    /// Sets the structural metrics.
    pub fn with_structural(mut self, structural: StructuralMetrics) -> Self {
        self.structural = Some(structural);
        self
    }
}

impl Default for OutputMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-LOD metrics for static meshes (MESH-004).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticMeshLodLevelMetrics {
    /// LOD index (0, 1, 2, ...).
    pub lod_level: u32,
    /// Target triangle count requested for this LOD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_tris: Option<u32>,
    /// Simplification ratio relative to the pre-decimate triangle count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simplification_ratio: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertex_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triangle_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quad_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quad_percentage: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_manifold_edge_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degenerate_face_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero_area_face_count: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_island_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_coverage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_overlap_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_uv_map: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_layer_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texel_density: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds_min: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds_max: Option<[f64; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_slot_count: Option<u32>,
}

/// Collision mesh metrics (MESH-005).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollisionMeshMetrics {
    pub vertex_count: u32,
    pub face_count: u32,
    pub triangle_count: u32,
    pub bounding_box: CollisionBoundingBox,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collision_type: Option<String>,
}

/// Bounding box for collision meshes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollisionBoundingBox {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

/// Navmesh analysis metrics (MESH-006).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavmeshMetrics {
    pub walkable_face_count: u32,
    pub non_walkable_face_count: u32,
    pub walkable_percentage: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stair_candidates: Option<u32>,
}

/// Baking metrics (MESH-007).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BakingMetrics {
    pub baked_maps: Vec<BakedMapInfo>,
    pub ray_distance: f64,
    pub margin: u32,
}

/// Single baked map info.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BakedMapInfo {
    #[serde(rename = "type")]
    pub bake_type: String,
    pub path: String,
    pub resolution: [u32; 2],
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
