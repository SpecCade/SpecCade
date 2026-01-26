//! Structural metrics for LLM-friendly 3D feedback.
//!
//! This module provides types for describing geometric properties of meshes
//! without encoding aesthetic opinions. LLMs can compare these metrics against
//! their stated intent to self-correct 3D asset generation.

use serde::{Deserialize, Serialize};

/// Top-level container for all structural metrics.
///
/// Each category is optional because not all meshes have all features
/// (e.g., skeletal metrics only apply to rigged meshes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralMetrics {
    /// Universal geometry metrics (extent, proportions, centroid).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<GeometryMetrics>,

    /// Symmetry scores for each axis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symmetry: Option<SymmetryMetrics>,

    /// Component breakdown for multi-part meshes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<ComponentMetrics>,

    /// Skeletal structure metrics for rigged meshes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeletal: Option<SkeletalStructureMetrics>,

    /// Scale reference for quick size classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<ScaleReference>,
}

impl StructuralMetrics {
    /// Creates a new empty structural metrics container.
    pub fn new() -> Self {
        Self {
            geometry: None,
            symmetry: None,
            components: None,
            skeletal: None,
            scale: None,
        }
    }

    /// Sets the geometry metrics.
    pub fn with_geometry(mut self, geometry: GeometryMetrics) -> Self {
        self.geometry = Some(geometry);
        self
    }

    /// Sets the symmetry metrics.
    pub fn with_symmetry(mut self, symmetry: SymmetryMetrics) -> Self {
        self.symmetry = Some(symmetry);
        self
    }

    /// Sets the component metrics.
    pub fn with_components(mut self, components: ComponentMetrics) -> Self {
        self.components = Some(components);
        self
    }

    /// Sets the skeletal structure metrics.
    pub fn with_skeletal(mut self, skeletal: SkeletalStructureMetrics) -> Self {
        self.skeletal = Some(skeletal);
        self
    }

    /// Sets the scale reference.
    pub fn with_scale(mut self, scale: ScaleReference) -> Self {
        self.scale = Some(scale);
        self
    }
}

impl Default for StructuralMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Geometry Metrics
// ============================================================================

/// Universal geometry metrics applicable to all meshes.
///
/// Describes the overall shape, proportions, and spatial distribution
/// of the mesh geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometryMetrics {
    /// Bounding box extent [x, y, z] in meters.
    pub extent: [f64; 3],

    /// Aspect ratios between dimension pairs.
    pub aspect_ratios: AspectRatios,

    /// Which axis is longest: "X", "Y", or "Z".
    pub dominant_axis: String,

    /// Ratio of longest to shortest dimension (>= 1.0).
    pub elongation: f64,

    /// Centroid position [x, y, z] in world coordinates.
    pub centroid: [f64; 3],

    /// Centroid as fraction of bounding box [0-1, 0-1, 0-1].
    ///
    /// A value of [0.5, 0.5, 0.5] indicates the centroid is at the
    /// center of the bounding box.
    pub centroid_normalized: [f64; 3],

    /// Ratio of mesh volume to convex hull volume (0-1).
    ///
    /// A value close to 1.0 indicates a convex shape, while lower
    /// values indicate more concave or complex geometry.
    pub convex_hull_ratio: f64,
}

/// Aspect ratios between pairs of bounding box dimensions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AspectRatios {
    /// Ratio of X extent to Y extent.
    pub xy: f64,

    /// Ratio of X extent to Z extent.
    pub xz: f64,

    /// Ratio of Y extent to Z extent.
    pub yz: f64,
}

// ============================================================================
// Symmetry Metrics
// ============================================================================

/// Symmetry scores for reflection across each axis.
///
/// Each score is in the range [0, 1] where 1.0 indicates perfect symmetry
/// and 0.0 indicates no symmetry. Computed by comparing vertex positions
/// after reflection across the corresponding plane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SymmetryMetrics {
    /// Symmetry score for reflection across the YZ plane (X axis symmetry).
    pub x_axis: f64,

    /// Symmetry score for reflection across the XZ plane (Y axis symmetry).
    pub y_axis: f64,

    /// Symmetry score for reflection across the XY plane (Z axis symmetry).
    pub z_axis: f64,
}

// ============================================================================
// Component Metrics
// ============================================================================

/// Metrics for multi-part meshes with named components.
///
/// Provides per-component breakdown and spatial relationships between parts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentMetrics {
    /// Per-component breakdown.
    pub components: Vec<ComponentInfo>,

    /// Adjacency/overlap relationships between components.
    pub adjacency: Vec<ComponentAdjacency>,
}

/// Information about a single mesh component (named part).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentInfo {
    /// Component name (object name in Blender).
    pub name: String,

    /// Minimum corner of the component's bounding box [x, y, z].
    pub bounds_min: [f64; 3],

    /// Maximum corner of the component's bounding box [x, y, z].
    pub bounds_max: [f64; 3],

    /// Fraction of total mesh volume occupied by this component (0-1).
    pub volume_fraction: f64,

    /// Centroid position [x, y, z] in world coordinates.
    pub centroid: [f64; 3],

    /// Number of triangles in this component.
    pub triangle_count: u32,
}

/// Adjacency relationship between two mesh components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentAdjacency {
    /// Name of the first component.
    pub part_a: String,

    /// Name of the second component.
    pub part_b: String,

    /// Distance between the components in meters.
    ///
    /// Positive values indicate a gap, negative values indicate overlap.
    pub distance: f64,
}

// ============================================================================
// Skeletal Structure Metrics
// ============================================================================

/// Structural metrics for rigged meshes with armatures.
///
/// Describes the bone hierarchy, mesh coverage along bones, and
/// left/right bone pair symmetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletalStructureMetrics {
    /// Maximum depth of the bone hierarchy (root = depth 1).
    pub hierarchy_depth: u32,

    /// Names of terminal (leaf) bones with no children.
    pub terminal_bones: Vec<String>,

    /// Per-bone mesh coverage information.
    pub bone_coverage: Vec<BoneCoverageInfo>,

    /// Left/right bone pair symmetry comparisons.
    pub bone_symmetry: Vec<BonePairSymmetry>,
}

/// Mesh coverage information for a single bone.
///
/// Describes how much of the mesh is influenced by this bone and
/// the proportions of the covered region.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoneCoverageInfo {
    /// Name of the bone.
    pub bone_name: String,

    /// Length of the bone in meters.
    pub bone_length: f64,

    /// Length of mesh geometry along the bone's axis in meters.
    pub mesh_length_along_bone: f64,

    /// Ratio of mesh length to bone length.
    pub coverage_ratio: f64,

    /// Average radius of mesh geometry around the bone in meters.
    pub mesh_radius_avg: f64,
}

/// Symmetry comparison between a left/right bone pair.
///
/// Used to detect asymmetric limbs or features that should be mirrored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BonePairSymmetry {
    /// Name of the left bone (e.g., "arm.L").
    pub bone_left: String,

    /// Name of the right bone (e.g., "arm.R").
    pub bone_right: String,

    /// Ratio of left bone length to right bone length.
    ///
    /// A value close to 1.0 indicates symmetric bone lengths.
    pub length_ratio: f64,

    /// Ratio of left mesh radius to right mesh radius.
    ///
    /// A value close to 1.0 indicates symmetric mesh thickness.
    pub radius_ratio: f64,
}

// ============================================================================
// Scale Reference
// ============================================================================

/// Quick scale reference for size classification.
///
/// Provides simple size checks to help LLMs verify the mesh is at
/// the intended scale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleReference {
    /// Longest dimension of the bounding box in meters.
    pub longest_dimension_m: f64,

    /// Total volume of the bounding box in cubic meters.
    pub volume_m3: f64,

    /// Whether the mesh fits within a 1 meter cube.
    pub fits_in_1m_cube: bool,

    /// Whether the mesh fits within a 10 centimeter cube (handheld object scale).
    pub fits_in_10cm_cube: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structural_metrics_serialization() {
        let metrics = StructuralMetrics::new()
            .with_geometry(GeometryMetrics {
                extent: [1.0, 2.0, 0.5],
                aspect_ratios: AspectRatios {
                    xy: 0.5,
                    xz: 2.0,
                    yz: 4.0,
                },
                dominant_axis: "Y".to_string(),
                elongation: 4.0,
                centroid: [0.5, 1.0, 0.25],
                centroid_normalized: [0.5, 0.5, 0.5],
                convex_hull_ratio: 0.85,
            })
            .with_scale(ScaleReference {
                longest_dimension_m: 2.0,
                volume_m3: 1.0,
                fits_in_1m_cube: false,
                fits_in_10cm_cube: false,
            });

        let json = serde_json::to_string_pretty(&metrics).unwrap();
        let parsed: StructuralMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics, parsed);
    }

    #[test]
    fn symmetry_metrics_serialization() {
        let symmetry = SymmetryMetrics {
            x_axis: 0.95,
            y_axis: 0.1,
            z_axis: 0.5,
        };

        let json = serde_json::to_string(&symmetry).unwrap();
        let parsed: SymmetryMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(symmetry, parsed);
    }

    #[test]
    fn component_metrics_serialization() {
        let components = ComponentMetrics {
            components: vec![
                ComponentInfo {
                    name: "body".to_string(),
                    bounds_min: [-0.5, 0.0, -0.25],
                    bounds_max: [0.5, 1.5, 0.25],
                    volume_fraction: 0.7,
                    centroid: [0.0, 0.75, 0.0],
                    triangle_count: 1000,
                },
                ComponentInfo {
                    name: "head".to_string(),
                    bounds_min: [-0.2, 1.5, -0.2],
                    bounds_max: [0.2, 1.9, 0.2],
                    volume_fraction: 0.3,
                    centroid: [0.0, 1.7, 0.0],
                    triangle_count: 500,
                },
            ],
            adjacency: vec![ComponentAdjacency {
                part_a: "body".to_string(),
                part_b: "head".to_string(),
                distance: 0.0,
            }],
        };

        let json = serde_json::to_string(&components).unwrap();
        let parsed: ComponentMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(components, parsed);
    }

    #[test]
    fn skeletal_metrics_serialization() {
        let skeletal = SkeletalStructureMetrics {
            hierarchy_depth: 4,
            terminal_bones: vec!["hand.L".to_string(), "hand.R".to_string()],
            bone_coverage: vec![BoneCoverageInfo {
                bone_name: "spine".to_string(),
                bone_length: 0.4,
                mesh_length_along_bone: 0.5,
                coverage_ratio: 1.25,
                mesh_radius_avg: 0.15,
            }],
            bone_symmetry: vec![BonePairSymmetry {
                bone_left: "arm.L".to_string(),
                bone_right: "arm.R".to_string(),
                length_ratio: 1.0,
                radius_ratio: 0.98,
            }],
        };

        let json = serde_json::to_string(&skeletal).unwrap();
        let parsed: SkeletalStructureMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(skeletal, parsed);
    }

    #[test]
    fn optional_fields_not_serialized_when_none() {
        let metrics = StructuralMetrics::new();
        let json = serde_json::to_string(&metrics).unwrap();
        // Should be an empty object since all fields are None
        assert_eq!(json, "{}");
    }
}
