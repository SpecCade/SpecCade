//! Shrinkwrap mesh recipe definitions.
//!
//! This module provides the `static_mesh.shrinkwrap_v1` recipe for wrapping
//! one mesh onto another (e.g., armor onto body, clothing onto character)
//! using Blender's shrinkwrap modifier (Tier 2 backend).

use serde::{Deserialize, Serialize};

use super::common::MeshExportSettings;

/// Maximum smooth iterations allowed for post-wrap smoothing.
pub const MAX_SMOOTH_ITERATIONS: u8 = 10;

/// Minimum offset value (can be zero or positive).
pub const MIN_OFFSET: f64 = 0.0;

/// Maximum offset value.
pub const MAX_OFFSET: f64 = 1.0;

/// Default offset value.
pub const DEFAULT_OFFSET: f64 = 0.0;

/// Default smooth iterations.
pub const DEFAULT_SMOOTH_ITERATIONS: u8 = 0;

/// Shrinkwrap mode (corresponds to Blender's shrinkwrap modifier modes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ShrinkwrapMode {
    /// Snap to the nearest point on the target surface.
    /// Best for general wrapping like armor plates.
    #[default]
    NearestSurface,
    /// Project vertices along their normals onto the target.
    /// Best for cloth-like draping effects.
    Project,
    /// Snap to the nearest vertex on the target mesh.
    /// Best for low-poly or stylized fits.
    NearestVertex,
}

/// Validation settings for post-shrinkwrap mesh quality checks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShrinkwrapValidation {
    /// Maximum allowed self-intersections (0 = fail on any intersection).
    /// Self-intersections occur when the mesh folds into itself.
    #[serde(default)]
    pub max_self_intersections: u32,
    /// Minimum face area threshold. Faces smaller than this are considered degenerate.
    /// Default: 0.0001
    #[serde(
        default = "default_min_face_area",
        skip_serializing_if = "is_default_min_face_area"
    )]
    pub min_face_area: f64,
}

fn default_min_face_area() -> f64 {
    0.0001
}

fn is_default_min_face_area(v: &f64) -> bool {
    (*v - 0.0001).abs() < f64::EPSILON
}

impl Default for ShrinkwrapValidation {
    fn default() -> Self {
        Self {
            max_self_intersections: 0,
            min_face_area: default_min_face_area(),
        }
    }
}

/// Parameters for the `static_mesh.shrinkwrap_v1` recipe.
///
/// This recipe wraps a source mesh (wrap_mesh) onto a target mesh (base_mesh)
/// using Blender's shrinkwrap modifier. Useful for:
/// - Fitting armor plates onto character bodies
/// - Draping cloth/clothing onto characters
/// - Attaching accessories to curved surfaces
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticMeshShrinkwrapV1Params {
    /// Asset ID or path to the base/target mesh (the surface to wrap onto).
    /// This is the "body" mesh that the wrap mesh will conform to.
    pub base_mesh: String,
    /// Asset ID or path to the wrap mesh (the mesh that will be wrapped).
    /// This is the "armor" or "clothing" mesh that will be deformed.
    pub wrap_mesh: String,
    /// Shrinkwrap projection mode.
    #[serde(default)]
    pub mode: ShrinkwrapMode,
    /// Offset distance from the target surface.
    /// Positive values push the wrapped mesh away from the surface,
    /// creating a gap (useful for clothing layers or armor clearance).
    #[serde(default = "default_offset", skip_serializing_if = "is_default_offset")]
    pub offset: f64,
    /// Number of smooth iterations to apply after shrinkwrap (0-10).
    /// Helps blend the wrapped mesh and reduce sharp deformations.
    #[serde(default, skip_serializing_if = "is_default_smooth_iterations")]
    pub smooth_iterations: u8,
    /// Smooth factor per iteration (0.0 to 1.0).
    /// Higher values produce stronger smoothing per iteration.
    #[serde(
        default = "default_smooth_factor",
        skip_serializing_if = "is_default_smooth_factor"
    )]
    pub smooth_factor: f64,
    /// Optional validation settings for mesh quality post-wrap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ShrinkwrapValidation>,
    /// GLB export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<MeshExportSettings>,
}

fn default_offset() -> f64 {
    DEFAULT_OFFSET
}

fn is_default_offset(v: &f64) -> bool {
    (*v - DEFAULT_OFFSET).abs() < f64::EPSILON
}

fn is_default_smooth_iterations(v: &u8) -> bool {
    *v == DEFAULT_SMOOTH_ITERATIONS
}

fn default_smooth_factor() -> f64 {
    0.5
}

fn is_default_smooth_factor(v: &f64) -> bool {
    (*v - 0.5).abs() < f64::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // ShrinkwrapMode Tests
    // ========================================================================

    #[test]
    fn test_shrinkwrap_mode_nearest_surface() {
        let mode = ShrinkwrapMode::NearestSurface;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"nearest_surface\"");

        let parsed: ShrinkwrapMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ShrinkwrapMode::NearestSurface);
    }

    #[test]
    fn test_shrinkwrap_mode_project() {
        let mode = ShrinkwrapMode::Project;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"project\"");

        let parsed: ShrinkwrapMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ShrinkwrapMode::Project);
    }

    #[test]
    fn test_shrinkwrap_mode_nearest_vertex() {
        let mode = ShrinkwrapMode::NearestVertex;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"nearest_vertex\"");

        let parsed: ShrinkwrapMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ShrinkwrapMode::NearestVertex);
    }

    #[test]
    fn test_shrinkwrap_mode_default() {
        let mode = ShrinkwrapMode::default();
        assert_eq!(mode, ShrinkwrapMode::NearestSurface);
    }

    // ========================================================================
    // ShrinkwrapValidation Tests
    // ========================================================================

    #[test]
    fn test_shrinkwrap_validation_default() {
        let validation = ShrinkwrapValidation::default();
        assert_eq!(validation.max_self_intersections, 0);
        assert_eq!(validation.min_face_area, 0.0001);
    }

    #[test]
    fn test_shrinkwrap_validation_basic() {
        let validation = ShrinkwrapValidation {
            max_self_intersections: 5,
            min_face_area: 0.001,
        };

        let json = serde_json::to_string(&validation).unwrap();
        assert!(json.contains("\"max_self_intersections\":5"));
        assert!(json.contains("\"min_face_area\":0.001"));

        let parsed: ShrinkwrapValidation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.max_self_intersections, 5);
        assert_eq!(parsed.min_face_area, 0.001);
    }

    #[test]
    fn test_shrinkwrap_validation_from_json_defaults() {
        let json = r#"{"max_self_intersections":0}"#;
        let parsed: ShrinkwrapValidation = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.max_self_intersections, 0);
        assert_eq!(parsed.min_face_area, 0.0001);
    }

    #[test]
    fn test_shrinkwrap_validation_omits_defaults() {
        let validation = ShrinkwrapValidation {
            max_self_intersections: 0,
            min_face_area: 0.0001,
        };

        let json = serde_json::to_string(&validation).unwrap();
        assert!(!json.contains("min_face_area"));
    }

    #[test]
    fn test_shrinkwrap_validation_rejects_unknown_fields() {
        let json = r#"{"max_self_intersections":0,"unknown":true}"#;
        let result: Result<ShrinkwrapValidation, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ========================================================================
    // StaticMeshShrinkwrapV1Params Tests
    // ========================================================================

    #[test]
    fn test_shrinkwrap_params_basic() {
        let params = StaticMeshShrinkwrapV1Params {
            base_mesh: "body_torso".to_string(),
            wrap_mesh: "armor_plate".to_string(),
            mode: ShrinkwrapMode::NearestSurface,
            offset: 0.0,
            smooth_iterations: 0,
            smooth_factor: 0.5,
            validation: None,
            export: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"base_mesh\":\"body_torso\""));
        assert!(json.contains("\"wrap_mesh\":\"armor_plate\""));
        // Defaults should be omitted
        assert!(!json.contains("offset"));
        assert!(!json.contains("smooth_iterations"));
        assert!(!json.contains("smooth_factor"));

        let parsed: StaticMeshShrinkwrapV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.base_mesh, "body_torso");
        assert_eq!(parsed.wrap_mesh, "armor_plate");
        assert_eq!(parsed.mode, ShrinkwrapMode::NearestSurface);
        assert_eq!(parsed.offset, 0.0);
        assert_eq!(parsed.smooth_iterations, 0);
    }

    #[test]
    fn test_shrinkwrap_params_with_offset() {
        let params = StaticMeshShrinkwrapV1Params {
            base_mesh: "body".to_string(),
            wrap_mesh: "armor".to_string(),
            mode: ShrinkwrapMode::NearestSurface,
            offset: 0.02,
            smooth_iterations: 2,
            smooth_factor: 0.5,
            validation: None,
            export: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"offset\":0.02"));
        assert!(json.contains("\"smooth_iterations\":2"));

        let parsed: StaticMeshShrinkwrapV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.offset, 0.02);
        assert_eq!(parsed.smooth_iterations, 2);
    }

    #[test]
    fn test_shrinkwrap_params_project_mode() {
        let params = StaticMeshShrinkwrapV1Params {
            base_mesh: "body".to_string(),
            wrap_mesh: "cloth".to_string(),
            mode: ShrinkwrapMode::Project,
            offset: 0.01,
            smooth_iterations: 3,
            smooth_factor: 0.5,
            validation: None,
            export: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"mode\":\"project\""));

        let parsed: StaticMeshShrinkwrapV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mode, ShrinkwrapMode::Project);
    }

    #[test]
    fn test_shrinkwrap_params_with_validation() {
        let params = StaticMeshShrinkwrapV1Params {
            base_mesh: "body".to_string(),
            wrap_mesh: "armor".to_string(),
            mode: ShrinkwrapMode::NearestSurface,
            offset: 0.02,
            smooth_iterations: 2,
            smooth_factor: 0.5,
            validation: Some(ShrinkwrapValidation {
                max_self_intersections: 0,
                min_face_area: 0.0001,
            }),
            export: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"validation\""));
        assert!(json.contains("\"max_self_intersections\":0"));

        let parsed: StaticMeshShrinkwrapV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.validation.is_some());
        let validation = parsed.validation.unwrap();
        assert_eq!(validation.max_self_intersections, 0);
    }

    #[test]
    fn test_shrinkwrap_params_with_export() {
        let params = StaticMeshShrinkwrapV1Params {
            base_mesh: "body".to_string(),
            wrap_mesh: "armor".to_string(),
            mode: ShrinkwrapMode::NearestSurface,
            offset: 0.0,
            smooth_iterations: 0,
            smooth_factor: 0.5,
            validation: None,
            export: Some(MeshExportSettings {
                apply_modifiers: true,
                triangulate: true,
                include_normals: true,
                include_uvs: true,
                include_vertex_colors: false,
                tangents: false,
                save_blend: false,
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"export\""));
        assert!(json.contains("\"apply_modifiers\":true"));

        let parsed: StaticMeshShrinkwrapV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.export.is_some());
    }

    #[test]
    fn test_shrinkwrap_params_complete() {
        let params = StaticMeshShrinkwrapV1Params {
            base_mesh: "character_body".to_string(),
            wrap_mesh: "plate_armor".to_string(),
            mode: ShrinkwrapMode::NearestSurface,
            offset: 0.015,
            smooth_iterations: 2,
            smooth_factor: 0.6,
            validation: Some(ShrinkwrapValidation {
                max_self_intersections: 0,
                min_face_area: 0.0002,
            }),
            export: Some(MeshExportSettings::default()),
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: StaticMeshShrinkwrapV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.base_mesh, "character_body");
        assert_eq!(parsed.wrap_mesh, "plate_armor");
        assert_eq!(parsed.mode, ShrinkwrapMode::NearestSurface);
        assert_eq!(parsed.offset, 0.015);
        assert_eq!(parsed.smooth_iterations, 2);
        assert_eq!(parsed.smooth_factor, 0.6);
        assert!(parsed.validation.is_some());
        assert!(parsed.export.is_some());
    }

    #[test]
    fn test_shrinkwrap_params_from_json_minimal() {
        let json = r#"{
            "base_mesh": "body",
            "wrap_mesh": "armor"
        }"#;

        let parsed: StaticMeshShrinkwrapV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.base_mesh, "body");
        assert_eq!(parsed.wrap_mesh, "armor");
        assert_eq!(parsed.mode, ShrinkwrapMode::NearestSurface);
        assert_eq!(parsed.offset, 0.0);
        assert_eq!(parsed.smooth_iterations, 0);
        assert_eq!(parsed.smooth_factor, 0.5);
        assert!(parsed.validation.is_none());
        assert!(parsed.export.is_none());
    }

    #[test]
    fn test_shrinkwrap_params_from_json_complete() {
        let json = r#"{
            "base_mesh": "body_torso",
            "wrap_mesh": "armor_chestplate",
            "mode": "project",
            "offset": 0.02,
            "smooth_iterations": 3,
            "smooth_factor": 0.7,
            "validation": {
                "max_self_intersections": 0,
                "min_face_area": 0.0001
            },
            "export": {
                "apply_modifiers": true,
                "triangulate": true,
                "include_normals": true,
                "include_uvs": true,
                "include_vertex_colors": false,
                "tangents": true
            }
        }"#;

        let parsed: StaticMeshShrinkwrapV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.base_mesh, "body_torso");
        assert_eq!(parsed.wrap_mesh, "armor_chestplate");
        assert_eq!(parsed.mode, ShrinkwrapMode::Project);
        assert_eq!(parsed.offset, 0.02);
        assert_eq!(parsed.smooth_iterations, 3);
        assert_eq!(parsed.smooth_factor, 0.7);
        assert!(parsed.validation.is_some());
        assert!(parsed.export.is_some());
        assert!(parsed.export.unwrap().tangents);
    }

    #[test]
    fn test_shrinkwrap_params_rejects_unknown_fields() {
        let json = r#"{
            "base_mesh": "body",
            "wrap_mesh": "armor",
            "unknown_field": true
        }"#;

        let result: Result<StaticMeshShrinkwrapV1Params, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_shrinkwrap_params_all_modes() {
        for mode in [
            ShrinkwrapMode::NearestSurface,
            ShrinkwrapMode::Project,
            ShrinkwrapMode::NearestVertex,
        ] {
            let params = StaticMeshShrinkwrapV1Params {
                base_mesh: "body".to_string(),
                wrap_mesh: "armor".to_string(),
                mode,
                offset: 0.0,
                smooth_iterations: 0,
                smooth_factor: 0.5,
                validation: None,
                export: None,
            };

            let json = serde_json::to_string(&params).unwrap();
            let parsed: StaticMeshShrinkwrapV1Params = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.mode, mode);
        }
    }
}
