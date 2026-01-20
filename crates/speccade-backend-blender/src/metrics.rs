//! Metrics types for Tier 2 validation.
//!
//! Blender backends produce metrics that are validated against tolerances
//! rather than requiring byte-identical output (unlike Tier 1 backends).

use serde::{Deserialize, Serialize};

/// Metrics reported by Blender for a generated mesh or animation.
///
/// These metrics are used for Tier 2 validation where determinism is
/// validated via metrics rather than file hashes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BlenderMetrics {
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
    /// Axis-aligned bounding box of the mesh.
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
    /// Number of bones in the armature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone_count: Option<u32>,

    /// Maximum bone influences per vertex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bone_influences: Option<u32>,

    /// Number of unweighted vertices (vertices with zero total skin weight, CHAR-003).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unweighted_vertex_count: Option<u32>,

    /// Percentage of vertices with normalized weights (CHAR-003).
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

    /// Animation duration in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation_duration_seconds: Option<f64>,

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
    pub root_motion_delta: Option<[f64; 3]>,
}

impl BlenderMetrics {
    /// Creates metrics for a static mesh.
    pub fn for_static_mesh(
        triangle_count: u32,
        bounding_box: BoundingBox,
        uv_island_count: u32,
        material_slot_count: u32,
    ) -> Self {
        Self {
            triangle_count: Some(triangle_count),
            bounding_box: Some(bounding_box),
            uv_island_count: Some(uv_island_count),
            material_slot_count: Some(material_slot_count),
            ..Default::default()
        }
    }

    /// Creates metrics for a skeletal mesh.
    pub fn for_skeletal_mesh(
        triangle_count: u32,
        bounding_box: BoundingBox,
        bone_count: u32,
        material_slot_count: u32,
        max_bone_influences: u32,
    ) -> Self {
        Self {
            triangle_count: Some(triangle_count),
            bounding_box: Some(bounding_box),
            bone_count: Some(bone_count),
            material_slot_count: Some(material_slot_count),
            max_bone_influences: Some(max_bone_influences),
            ..Default::default()
        }
    }

    /// Creates metrics for an animation.
    pub fn for_animation(bone_count: u32, frame_count: u32, duration_seconds: f64) -> Self {
        Self {
            bone_count: Some(bone_count),
            animation_frame_count: Some(frame_count),
            animation_duration_seconds: Some(duration_seconds),
            ..Default::default()
        }
    }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum corner [X, Y, Z].
    pub min: [f64; 3],
    /// Maximum corner [X, Y, Z].
    pub max: [f64; 3],
}

/// Per-LOD metrics for static meshes (MESH-004).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticMeshLodLevelMetrics {
    /// LOD index (0, 1, 2, ...).
    pub lod_level: u32,
    /// Target triangle count requested for this LOD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_tris: Option<u32>,
    /// Simplification ratio relative to the pre-decimate triangle count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simplification_ratio: Option<f64>,

    // Mesh metrics (subset of BlenderMetrics).
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
pub struct CollisionMeshMetrics {
    pub vertex_count: u32,
    pub face_count: u32,
    pub triangle_count: u32,
    pub bounding_box: BoundingBox,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collision_type: Option<String>,
}

/// Navmesh analysis metrics (MESH-006).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavmeshMetrics {
    pub walkable_face_count: u32,
    pub non_walkable_face_count: u32,
    pub walkable_percentage: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stair_candidates: Option<u32>,
}

/// Baking metrics (MESH-007).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BakingMetrics {
    pub baked_maps: Vec<BakedMapInfo>,
    pub ray_distance: f64,
    pub margin: u32,
}

/// Single baked map info.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BakedMapInfo {
    #[serde(rename = "type")]
    pub bake_type: String,
    pub path: String,
    pub resolution: [u32; 2],
}

impl BoundingBox {
    /// Creates a new bounding box.
    pub fn new(min: [f64; 3], max: [f64; 3]) -> Self {
        Self { min, max }
    }

    /// Returns the size of the bounding box in each dimension.
    pub fn size(&self) -> [f64; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    /// Returns the center of the bounding box.
    pub fn center(&self) -> [f64; 3] {
        [
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
            (self.min[2] + self.max[2]) / 2.0,
        ]
    }

    /// Checks if this bounding box is within tolerance of another.
    pub fn within_tolerance(&self, other: &BoundingBox, tolerance: f64) -> bool {
        for i in 0..3 {
            if (self.min[i] - other.min[i]).abs() > tolerance {
                return false;
            }
            if (self.max[i] - other.max[i]).abs() > tolerance {
                return false;
            }
        }
        true
    }
}

/// Tolerances for Tier 2 metric validation as specified in RFC-0001.
#[derive(Debug, Clone)]
pub struct MetricTolerances {
    /// Bounding box tolerance (+/- units).
    pub bounding_box: f64,
    /// Animation duration tolerance (+/- seconds).
    pub animation_duration: f64,
}

impl Default for MetricTolerances {
    fn default() -> Self {
        Self {
            bounding_box: 0.001,
            animation_duration: 0.001,
        }
    }
}

/// Result of comparing two metric sets.
#[derive(Debug, Clone)]
pub struct MetricComparison {
    /// Whether all metrics match within tolerance.
    pub matches: bool,
    /// Details of any mismatches.
    pub mismatches: Vec<MetricMismatch>,
}

/// Description of a metric mismatch.
#[derive(Debug, Clone)]
pub struct MetricMismatch {
    /// Name of the metric.
    pub metric_name: String,
    /// Expected value (as string).
    pub expected: String,
    /// Actual value (as string).
    pub actual: String,
    /// Whether this is within tolerance (for non-exact metrics).
    pub within_tolerance: bool,
}

impl BlenderMetrics {
    /// Compares these metrics to expected metrics with tolerances.
    pub fn compare(
        &self,
        expected: &BlenderMetrics,
        tolerances: &MetricTolerances,
    ) -> MetricComparison {
        let mut mismatches = Vec::new();

        // Triangle count (exact match required)
        if let (Some(actual), Some(exp)) = (self.triangle_count, expected.triangle_count) {
            if actual != exp {
                mismatches.push(MetricMismatch {
                    metric_name: "triangle_count".to_string(),
                    expected: exp.to_string(),
                    actual: actual.to_string(),
                    within_tolerance: false,
                });
            }
        }

        // UV island count (exact match required)
        if let (Some(actual), Some(exp)) = (self.uv_island_count, expected.uv_island_count) {
            if actual != exp {
                mismatches.push(MetricMismatch {
                    metric_name: "uv_island_count".to_string(),
                    expected: exp.to_string(),
                    actual: actual.to_string(),
                    within_tolerance: false,
                });
            }
        }

        // Bone count (exact match required)
        if let (Some(actual), Some(exp)) = (self.bone_count, expected.bone_count) {
            if actual != exp {
                mismatches.push(MetricMismatch {
                    metric_name: "bone_count".to_string(),
                    expected: exp.to_string(),
                    actual: actual.to_string(),
                    within_tolerance: false,
                });
            }
        }

        // Material slot count (exact match required)
        if let (Some(actual), Some(exp)) = (self.material_slot_count, expected.material_slot_count)
        {
            if actual != exp {
                mismatches.push(MetricMismatch {
                    metric_name: "material_slot_count".to_string(),
                    expected: exp.to_string(),
                    actual: actual.to_string(),
                    within_tolerance: false,
                });
            }
        }

        // Animation frame count (exact match required)
        if let (Some(actual), Some(exp)) =
            (self.animation_frame_count, expected.animation_frame_count)
        {
            if actual != exp {
                mismatches.push(MetricMismatch {
                    metric_name: "animation_frame_count".to_string(),
                    expected: exp.to_string(),
                    actual: actual.to_string(),
                    within_tolerance: false,
                });
            }
        }

        // Bounding box (tolerance allowed)
        if let (Some(actual), Some(exp)) = (&self.bounding_box, &expected.bounding_box) {
            if !actual.within_tolerance(exp, tolerances.bounding_box) {
                mismatches.push(MetricMismatch {
                    metric_name: "bounding_box".to_string(),
                    expected: format!("min: {:?}, max: {:?}", exp.min, exp.max),
                    actual: format!("min: {:?}, max: {:?}", actual.min, actual.max),
                    within_tolerance: false,
                });
            }
        }

        // Animation duration (tolerance allowed)
        if let (Some(actual), Some(exp)) = (
            self.animation_duration_seconds,
            expected.animation_duration_seconds,
        ) {
            if (actual - exp).abs() > tolerances.animation_duration {
                mismatches.push(MetricMismatch {
                    metric_name: "animation_duration_seconds".to_string(),
                    expected: exp.to_string(),
                    actual: actual.to_string(),
                    within_tolerance: false,
                });
            }
        }

        MetricComparison {
            matches: mismatches.is_empty(),
            mismatches,
        }
    }
}

/// Report from Blender execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderReport {
    /// Whether the generation succeeded.
    pub ok: bool,
    /// Error message if generation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Metrics from the generated asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<BlenderMetrics>,
    /// Path to the generated output file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    /// Path to the generated .blend file (if save_blend was enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blend_path: Option<String>,
    /// Blender version used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blender_version: Option<String>,
    /// Execution time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl BlenderReport {
    /// Creates a successful report.
    pub fn success(metrics: BlenderMetrics, output_path: String) -> Self {
        Self {
            ok: true,
            error: None,
            metrics: Some(metrics),
            output_path: Some(output_path),
            blend_path: None,
            blender_version: None,
            duration_ms: None,
        }
    }

    /// Creates a failed report.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(error.into()),
            metrics: None,
            output_path: None,
            blend_path: None,
            blender_version: None,
            duration_ms: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box_within_tolerance() {
        let bb1 = BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 2.0, 1.0]);
        let bb2 = BoundingBox::new([-1.0005, 0.0, -1.0], [1.0, 2.0005, 1.0]);

        assert!(bb1.within_tolerance(&bb2, 0.001));
        assert!(!bb1.within_tolerance(&bb2, 0.0001));
    }

    #[test]
    fn test_bounding_box_size() {
        let bb = BoundingBox::new([-1.0, 0.0, -2.0], [1.0, 3.0, 2.0]);
        let size = bb.size();
        assert_eq!(size, [2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_metrics_comparison() {
        let metrics1 = BlenderMetrics::for_static_mesh(
            100,
            BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 2.0, 1.0]),
            4,
            2,
        );

        let metrics2 = BlenderMetrics::for_static_mesh(
            100,
            BoundingBox::new([-1.0005, 0.0, -1.0], [1.0, 2.0, 1.0]),
            4,
            2,
        );

        let tolerances = MetricTolerances::default();
        let comparison = metrics1.compare(&metrics2, &tolerances);
        assert!(comparison.matches);
    }

    #[test]
    fn test_metrics_comparison_mismatch() {
        let metrics1 = BlenderMetrics::for_static_mesh(
            100,
            BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 2.0, 1.0]),
            4,
            2,
        );

        let metrics2 = BlenderMetrics::for_static_mesh(
            200, // Different triangle count
            BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 2.0, 1.0]),
            4,
            2,
        );

        let tolerances = MetricTolerances::default();
        let comparison = metrics1.compare(&metrics2, &tolerances);
        assert!(!comparison.matches);
        assert_eq!(comparison.mismatches.len(), 1);
        assert_eq!(comparison.mismatches[0].metric_name, "triangle_count");
    }

    #[test]
    fn test_blender_report_serde() {
        let report = BlenderReport::success(
            BlenderMetrics::for_static_mesh(
                100,
                BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 2.0, 1.0]),
                4,
                2,
            ),
            "output.glb".to_string(),
        );

        let json = serde_json::to_string(&report).unwrap();
        let parsed: BlenderReport = serde_json::from_str(&json).unwrap();
        assert!(parsed.ok);
        assert_eq!(parsed.output_path, Some("output.glb".to_string()));
    }
}
