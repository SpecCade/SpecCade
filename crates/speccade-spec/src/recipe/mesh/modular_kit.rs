//! Modular kit mesh recipe definitions.
//!
//! This module provides the `static_mesh.modular_kit_v1` recipe for generating
//! modular architectural and mechanical components (walls, doors, pipes) using
//! Blender as a Tier 2 backend.

use serde::{Deserialize, Serialize};

use super::common::MeshExportSettings;

/// Maximum number of cutouts allowed in a wall.
pub const MAX_WALL_CUTOUTS: usize = 100;

/// Maximum number of pipe segments allowed.
pub const MAX_PIPE_SEGMENTS: usize = 50;

/// Parameters for the `static_mesh.modular_kit_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticMeshModularKitV1Params {
    /// Kit type to generate (wall, pipe, or door).
    pub kit_type: ModularKitType,
    /// GLB export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<MeshExportSettings>,
}

/// Kit type variants for modular mesh generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModularKitType {
    /// Wall section with optional door/window cutouts.
    Wall(WallKitParams),
    /// Pipe segment with bends and junctions.
    Pipe(PipeKitParams),
    /// Door frame with optional door panel.
    Door(DoorKitParams),
}

/// Parameters for wall kit generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WallKitParams {
    /// Wall width in units.
    pub width: f64,
    /// Wall height in units.
    pub height: f64,
    /// Wall thickness in units.
    pub thickness: f64,
    /// Optional cutouts (doors, windows).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cutouts: Vec<WallCutout>,
    /// Whether to add a baseboard.
    #[serde(default)]
    pub has_baseboard: bool,
    /// Whether to add a crown molding.
    #[serde(default)]
    pub has_crown: bool,
    /// Baseboard height (if has_baseboard is true).
    #[serde(
        default = "default_baseboard_height",
        skip_serializing_if = "is_default_baseboard_height"
    )]
    pub baseboard_height: f64,
    /// Crown height (if has_crown is true).
    #[serde(
        default = "default_crown_height",
        skip_serializing_if = "is_default_crown_height"
    )]
    pub crown_height: f64,
    /// Bevel width for edges (0 for no bevel).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bevel_width: f64,
}

fn default_baseboard_height() -> f64 {
    0.1
}

fn is_default_baseboard_height(v: &f64) -> bool {
    (*v - 0.1).abs() < f64::EPSILON
}

fn default_crown_height() -> f64 {
    0.08
}

fn is_default_crown_height(v: &f64) -> bool {
    (*v - 0.08).abs() < f64::EPSILON
}

fn is_zero(v: &f64) -> bool {
    v.abs() < f64::EPSILON
}

/// A cutout in a wall (for doors or windows).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WallCutout {
    /// Cutout type (door or window).
    pub cutout_type: CutoutType,
    /// X position of cutout center (relative to wall origin).
    pub x: f64,
    /// Y position of cutout bottom (height from floor).
    pub y: f64,
    /// Cutout width.
    pub width: f64,
    /// Cutout height.
    pub height: f64,
    /// Whether to add a frame around the cutout.
    #[serde(default)]
    pub has_frame: bool,
    /// Frame thickness (if has_frame is true).
    #[serde(
        default = "default_frame_thickness",
        skip_serializing_if = "is_default_frame_thickness"
    )]
    pub frame_thickness: f64,
}

fn default_frame_thickness() -> f64 {
    0.05
}

fn is_default_frame_thickness(v: &f64) -> bool {
    (*v - 0.05).abs() < f64::EPSILON
}

/// Type of wall cutout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutoutType {
    /// Door cutout (extends to floor).
    Door,
    /// Window cutout.
    Window,
}

/// Parameters for pipe kit generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipeKitParams {
    /// Outer diameter of the pipe.
    pub diameter: f64,
    /// Wall thickness of the pipe.
    #[serde(
        default = "default_pipe_wall_thickness",
        skip_serializing_if = "is_default_pipe_wall_thickness"
    )]
    pub wall_thickness: f64,
    /// Segments that make up the pipe.
    pub segments: Vec<PipeSegment>,
    /// Number of vertices around the pipe circumference.
    #[serde(
        default = "default_pipe_vertices",
        skip_serializing_if = "is_default_pipe_vertices"
    )]
    pub vertices: u32,
    /// Bevel width for edges (0 for no bevel).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bevel_width: f64,
}

fn default_pipe_wall_thickness() -> f64 {
    0.02
}

fn is_default_pipe_wall_thickness(v: &f64) -> bool {
    (*v - 0.02).abs() < f64::EPSILON
}

fn default_pipe_vertices() -> u32 {
    16
}

fn is_default_pipe_vertices(v: &u32) -> bool {
    *v == 16
}

/// A segment of a pipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum PipeSegment {
    /// Straight pipe section.
    Straight {
        /// Length of the straight section.
        length: f64,
    },
    /// Bend/elbow section.
    Bend {
        /// Angle of the bend in degrees.
        angle: f64,
        /// Radius of the bend (from pipe center).
        radius: f64,
    },
    /// T-junction.
    TJunction {
        /// Length of the junction arm.
        arm_length: f64,
    },
    /// Flange/connector.
    Flange {
        /// Flange outer diameter.
        outer_diameter: f64,
        /// Flange thickness.
        thickness: f64,
    },
}

/// Parameters for door kit generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DoorKitParams {
    /// Door opening width.
    pub width: f64,
    /// Door opening height.
    pub height: f64,
    /// Door frame thickness.
    pub frame_thickness: f64,
    /// Door frame depth.
    #[serde(
        default = "default_frame_depth",
        skip_serializing_if = "is_default_frame_depth"
    )]
    pub frame_depth: f64,
    /// Whether to include a door panel.
    #[serde(default)]
    pub has_door_panel: bool,
    /// Which side the door hinges on (if has_door_panel is true).
    #[serde(default, skip_serializing_if = "is_default_hinge_side")]
    pub hinge_side: HingeSide,
    /// Door panel thickness (if has_door_panel is true).
    #[serde(
        default = "default_panel_thickness",
        skip_serializing_if = "is_default_panel_thickness"
    )]
    pub panel_thickness: f64,
    /// Whether the door panel is open (if has_door_panel is true).
    #[serde(default)]
    pub is_open: bool,
    /// Open angle in degrees (if is_open is true, 0-90).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub open_angle: f64,
    /// Bevel width for edges (0 for no bevel).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bevel_width: f64,
}

fn default_frame_depth() -> f64 {
    0.1
}

fn is_default_frame_depth(v: &f64) -> bool {
    (*v - 0.1).abs() < f64::EPSILON
}

fn default_panel_thickness() -> f64 {
    0.04
}

fn is_default_panel_thickness(v: &f64) -> bool {
    (*v - 0.04).abs() < f64::EPSILON
}

/// Side on which the door hinges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HingeSide {
    /// Door hinges on the left side.
    #[default]
    Left,
    /// Door hinges on the right side.
    Right,
}

fn is_default_hinge_side(v: &HingeSide) -> bool {
    *v == HingeSide::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // WallKitParams Tests
    // ========================================================================

    #[test]
    fn test_wall_kit_basic() {
        let params = WallKitParams {
            width: 3.0,
            height: 2.5,
            thickness: 0.15,
            cutouts: vec![],
            has_baseboard: false,
            has_crown: false,
            baseboard_height: 0.1,
            crown_height: 0.08,
            bevel_width: 0.0,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"width\":3.0"));
        assert!(json.contains("\"height\":2.5"));
        assert!(json.contains("\"thickness\":0.15"));
        assert!(!json.contains("cutouts"));
        assert!(!json.contains("baseboard_height"));

        let parsed: WallKitParams = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.width, 3.0);
    }

    #[test]
    fn test_wall_kit_with_cutouts() {
        let params = WallKitParams {
            width: 4.0,
            height: 2.5,
            thickness: 0.2,
            cutouts: vec![
                WallCutout {
                    cutout_type: CutoutType::Door,
                    x: 1.0,
                    y: 0.0,
                    width: 0.9,
                    height: 2.1,
                    has_frame: true,
                    frame_thickness: 0.05,
                },
                WallCutout {
                    cutout_type: CutoutType::Window,
                    x: 3.0,
                    y: 1.0,
                    width: 0.8,
                    height: 0.6,
                    has_frame: true,
                    frame_thickness: 0.05,
                },
            ],
            has_baseboard: true,
            has_crown: true,
            baseboard_height: 0.12,
            crown_height: 0.1,
            bevel_width: 0.01,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"cutouts\""));
        assert!(json.contains("\"door\""));
        assert!(json.contains("\"window\""));

        let parsed: WallKitParams = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.cutouts.len(), 2);
    }

    // ========================================================================
    // PipeKitParams Tests
    // ========================================================================

    #[test]
    fn test_pipe_kit_basic() {
        let params = PipeKitParams {
            diameter: 0.1,
            wall_thickness: 0.02,
            segments: vec![PipeSegment::Straight { length: 1.0 }],
            vertices: 16,
            bevel_width: 0.0,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"diameter\":0.1"));
        assert!(json.contains("\"straight\""));

        let parsed: PipeKitParams = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.diameter, 0.1);
    }

    #[test]
    fn test_pipe_kit_with_bends() {
        let params = PipeKitParams {
            diameter: 0.15,
            wall_thickness: 0.02,
            segments: vec![
                PipeSegment::Straight { length: 0.5 },
                PipeSegment::Bend {
                    angle: 90.0,
                    radius: 0.2,
                },
                PipeSegment::Straight { length: 0.5 },
                PipeSegment::TJunction { arm_length: 0.3 },
                PipeSegment::Flange {
                    outer_diameter: 0.2,
                    thickness: 0.02,
                },
            ],
            vertices: 24,
            bevel_width: 0.005,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"bend\""));
        assert!(json.contains("\"t_junction\""));
        assert!(json.contains("\"flange\""));

        let parsed: PipeKitParams = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.segments.len(), 5);
    }

    // ========================================================================
    // DoorKitParams Tests
    // ========================================================================

    #[test]
    fn test_door_kit_frame_only() {
        let params = DoorKitParams {
            width: 0.9,
            height: 2.1,
            frame_thickness: 0.05,
            frame_depth: 0.1,
            has_door_panel: false,
            hinge_side: HingeSide::Left,
            panel_thickness: 0.04,
            is_open: false,
            open_angle: 0.0,
            bevel_width: 0.0,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"width\":0.9"));
        assert!(json.contains("\"height\":2.1"));
        assert!(!json.contains("hinge_side"));

        let parsed: DoorKitParams = serde_json::from_str(&json).unwrap();
        assert!(!parsed.has_door_panel);
    }

    #[test]
    fn test_door_kit_with_panel() {
        let params = DoorKitParams {
            width: 0.9,
            height: 2.1,
            frame_thickness: 0.05,
            frame_depth: 0.15,
            has_door_panel: true,
            hinge_side: HingeSide::Right,
            panel_thickness: 0.05,
            is_open: true,
            open_angle: 45.0,
            bevel_width: 0.005,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"has_door_panel\":true"));
        assert!(json.contains("\"hinge_side\":\"right\""));
        assert!(json.contains("\"open_angle\":45.0"));

        let parsed: DoorKitParams = serde_json::from_str(&json).unwrap();
        assert!(parsed.has_door_panel);
        assert_eq!(parsed.hinge_side, HingeSide::Right);
    }

    // ========================================================================
    // ModularKitType Tests
    // ========================================================================

    #[test]
    fn test_kit_type_wall() {
        let kit = ModularKitType::Wall(WallKitParams {
            width: 3.0,
            height: 2.5,
            thickness: 0.15,
            cutouts: vec![],
            has_baseboard: false,
            has_crown: false,
            baseboard_height: 0.1,
            crown_height: 0.08,
            bevel_width: 0.0,
        });

        let json = serde_json::to_string(&kit).unwrap();
        assert!(json.contains("\"type\":\"wall\""));

        let parsed: ModularKitType = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ModularKitType::Wall(_)));
    }

    #[test]
    fn test_kit_type_pipe() {
        let kit = ModularKitType::Pipe(PipeKitParams {
            diameter: 0.1,
            wall_thickness: 0.02,
            segments: vec![PipeSegment::Straight { length: 1.0 }],
            vertices: 16,
            bevel_width: 0.0,
        });

        let json = serde_json::to_string(&kit).unwrap();
        assert!(json.contains("\"type\":\"pipe\""));

        let parsed: ModularKitType = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ModularKitType::Pipe(_)));
    }

    #[test]
    fn test_kit_type_door() {
        let kit = ModularKitType::Door(DoorKitParams {
            width: 0.9,
            height: 2.1,
            frame_thickness: 0.05,
            frame_depth: 0.1,
            has_door_panel: true,
            hinge_side: HingeSide::Left,
            panel_thickness: 0.04,
            is_open: false,
            open_angle: 0.0,
            bevel_width: 0.0,
        });

        let json = serde_json::to_string(&kit).unwrap();
        assert!(json.contains("\"type\":\"door\""));

        let parsed: ModularKitType = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ModularKitType::Door(_)));
    }

    // ========================================================================
    // StaticMeshModularKitV1Params Tests
    // ========================================================================

    #[test]
    fn test_full_params() {
        let params = StaticMeshModularKitV1Params {
            kit_type: ModularKitType::Wall(WallKitParams {
                width: 3.0,
                height: 2.5,
                thickness: 0.15,
                cutouts: vec![],
                has_baseboard: true,
                has_crown: false,
                baseboard_height: 0.1,
                crown_height: 0.08,
                bevel_width: 0.0,
            }),
            export: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"kit_type\""));

        let parsed: StaticMeshModularKitV1Params = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed.kit_type, ModularKitType::Wall(_)));
    }

    #[test]
    fn test_rejects_unknown_fields() {
        let json = r#"{
            "kit_type": {"type": "wall", "width": 3.0, "height": 2.5, "thickness": 0.15},
            "unknown_field": true
        }"#;

        let result: Result<StaticMeshModularKitV1Params, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
