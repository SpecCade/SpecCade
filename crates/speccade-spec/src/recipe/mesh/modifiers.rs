//! Mesh modifiers and transforms.

use serde::{Deserialize, Serialize};

/// Blender modifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MeshModifier {
    /// Bevel modifier.
    Bevel {
        /// Bevel width.
        width: f64,
        /// Number of segments.
        segments: u8,
        /// Angle limit in radians (only bevels edges with angle less than this).
        #[serde(skip_serializing_if = "Option::is_none")]
        angle_limit: Option<f64>,
    },
    /// Edge split modifier.
    EdgeSplit {
        /// Split angle in degrees.
        angle: f64,
    },
    /// Subdivision surface modifier.
    Subdivision {
        /// Subdivision levels for viewport.
        levels: u8,
        /// Subdivision levels for render.
        render_levels: u8,
    },
    /// Mirror modifier.
    Mirror {
        /// Mirror along X axis.
        #[serde(default)]
        axis_x: bool,
        /// Mirror along Y axis.
        #[serde(default)]
        axis_y: bool,
        /// Mirror along Z axis.
        #[serde(default)]
        axis_z: bool,
    },
    /// Array modifier.
    Array {
        /// Number of copies.
        count: u32,
        /// Offset between copies.
        offset: [f64; 3],
    },
    /// Solidify modifier.
    Solidify {
        /// Thickness.
        thickness: f64,
        /// Offset (-1 to 1).
        offset: f64,
    },
    /// Decimate modifier.
    Decimate {
        /// Ratio (0.0 to 1.0).
        ratio: f64,
    },
    /// Triangulate modifier.
    Triangulate {
        /// How to triangulate n-gons (polygons with more than 4 vertices).
        /// Options: "beauty", "clip", "fixed" (default: "beauty").
        #[serde(skip_serializing_if = "Option::is_none")]
        ngon_method: Option<String>,
        /// How to triangulate quads.
        /// Options: "beauty", "fixed", "shortest_diagonal", "longest_diagonal" (default: "shortest_diagonal").
        #[serde(skip_serializing_if = "Option::is_none")]
        quad_method: Option<String>,
    },
}

/// UV projection method.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum UvProjection {
    /// Simple projection method (backwards compatible).
    Simple(UvProjectionMethod),
    /// Projection with settings.
    WithSettings {
        /// Projection method.
        method: UvProjectionMethod,
        /// Angle limit in degrees (for smart projection).
        #[serde(skip_serializing_if = "Option::is_none")]
        angle_limit: Option<f64>,
        /// Cube projection size (for cube/box projection).
        #[serde(skip_serializing_if = "Option::is_none")]
        cube_size: Option<f64>,
        /// Target texel density in pixels per unit.
        /// When specified, UVs are scaled to achieve this density.
        #[serde(skip_serializing_if = "Option::is_none")]
        texel_density: Option<f64>,
        /// UV island margin/padding (0.0 to 1.0, default 0.001).
        /// Used when packing UV islands.
        #[serde(skip_serializing_if = "Option::is_none")]
        uv_margin: Option<f64>,
        /// Generate a secondary UV channel for lightmaps (UV1).
        #[serde(skip_serializing_if = "Option::is_none")]
        lightmap_uv: Option<bool>,
    },
}

/// UV projection method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UvProjectionMethod {
    /// Box/cube projection.
    Box,
    /// Cylinder projection.
    Cylinder,
    /// Sphere projection.
    Sphere,
    /// Smart UV project.
    Smart,
    /// Lightmap pack.
    Lightmap,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // MeshModifier Tests - All modifier types and their parameters
    // ========================================================================

    #[test]
    fn test_modifier_bevel_basic() {
        let modifier = MeshModifier::Bevel {
            width: 0.02,
            segments: 2,
            angle_limit: None,
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("bevel"));
        assert!(json.contains("\"width\":0.02"));
        assert!(json.contains("\"segments\":2"));
        assert!(!json.contains("angle_limit"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_bevel_with_angle_limit() {
        let modifier = MeshModifier::Bevel {
            width: 0.05,
            segments: 4,
            angle_limit: Some(std::f64::consts::FRAC_PI_4),
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("angle_limit"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_bevel_width() {
        let modifier = MeshModifier::Bevel {
            width: 0.1,
            segments: 3,
            angle_limit: None,
        };
        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"width\":0.1"));
    }

    #[test]
    fn test_modifier_bevel_segments() {
        let modifier = MeshModifier::Bevel {
            width: 0.02,
            segments: 5,
            angle_limit: None,
        };
        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"segments\":5"));
    }

    #[test]
    fn test_modifier_subdivision() {
        let modifier = MeshModifier::Subdivision {
            levels: 2,
            render_levels: 3,
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("subdivision"));
        assert!(json.contains("\"levels\":2"));
        assert!(json.contains("\"render_levels\":3"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_subdivision_levels() {
        let modifier = MeshModifier::Subdivision {
            levels: 4,
            render_levels: 4,
        };
        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"levels\":4"));
    }

    #[test]
    fn test_modifier_decimate() {
        let modifier = MeshModifier::Decimate { ratio: 0.5 };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("decimate"));
        assert!(json.contains("\"ratio\":0.5"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_decimate_ratio() {
        let modifier = MeshModifier::Decimate { ratio: 0.75 };
        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"ratio\":0.75"));
    }

    #[test]
    fn test_modifier_mirror_axis_x() {
        let modifier = MeshModifier::Mirror {
            axis_x: true,
            axis_y: false,
            axis_z: false,
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("mirror"));
        assert!(json.contains("\"axis_x\":true"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_mirror_axis_y() {
        let modifier = MeshModifier::Mirror {
            axis_x: false,
            axis_y: true,
            axis_z: false,
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"axis_y\":true"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_mirror_axis_z() {
        let modifier = MeshModifier::Mirror {
            axis_x: false,
            axis_y: false,
            axis_z: true,
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"axis_z\":true"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_mirror_multiple_axes() {
        let modifier = MeshModifier::Mirror {
            axis_x: true,
            axis_y: true,
            axis_z: false,
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"axis_x\":true"));
        assert!(json.contains("\"axis_y\":true"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_solidify() {
        let modifier = MeshModifier::Solidify {
            thickness: 0.1,
            offset: 0.0,
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("solidify"));
        assert!(json.contains("\"thickness\":0.1"));
        assert!(json.contains("\"offset\":0"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_solidify_thickness() {
        let modifier = MeshModifier::Solidify {
            thickness: 0.25,
            offset: 0.0,
        };
        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"thickness\":0.25"));
    }

    #[test]
    fn test_modifier_solidify_offset() {
        let modifier = MeshModifier::Solidify {
            thickness: 0.1,
            offset: -0.5,
        };
        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"offset\":-0.5"));
    }

    #[test]
    fn test_modifier_edge_split() {
        let modifier = MeshModifier::EdgeSplit { angle: 30.0 };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("edge_split"));
        assert!(json.contains("\"angle\":30"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_edge_split_angle() {
        let modifier = MeshModifier::EdgeSplit { angle: 45.0 };
        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"angle\":45"));
    }

    #[test]
    fn test_modifier_array() {
        let modifier = MeshModifier::Array {
            count: 5,
            offset: [1.0, 0.0, 0.0],
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("array"));
        assert!(json.contains("\"count\":5"));
        assert!(json.contains("\"offset\":[1.0,0.0,0.0]"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_array_count() {
        let modifier = MeshModifier::Array {
            count: 10,
            offset: [0.0, 0.0, 0.0],
        };
        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"count\":10"));
    }

    #[test]
    fn test_modifier_array_offset() {
        let modifier = MeshModifier::Array {
            count: 3,
            offset: [2.0, 1.5, 0.5],
        };
        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("\"offset\":[2.0,1.5,0.5]"));
    }

    // ========================================================================
    // Triangulate Modifier Tests
    // ========================================================================

    #[test]
    fn test_modifier_triangulate_defaults() {
        let modifier = MeshModifier::Triangulate {
            ngon_method: None,
            quad_method: None,
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("triangulate"));
        // Optional fields should not be serialized when None
        assert!(!json.contains("ngon_method"));
        assert!(!json.contains("quad_method"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_triangulate_with_ngon_method() {
        let modifier = MeshModifier::Triangulate {
            ngon_method: Some("beauty".to_string()),
            quad_method: None,
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("triangulate"));
        assert!(json.contains("\"ngon_method\":\"beauty\""));
        assert!(!json.contains("quad_method"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_triangulate_with_quad_method() {
        let modifier = MeshModifier::Triangulate {
            ngon_method: None,
            quad_method: Some("shortest_diagonal".to_string()),
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("triangulate"));
        assert!(!json.contains("ngon_method"));
        assert!(json.contains("\"quad_method\":\"shortest_diagonal\""));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_triangulate_with_both_methods() {
        let modifier = MeshModifier::Triangulate {
            ngon_method: Some("clip".to_string()),
            quad_method: Some("fixed".to_string()),
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("triangulate"));
        assert!(json.contains("\"ngon_method\":\"clip\""));
        assert!(json.contains("\"quad_method\":\"fixed\""));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_modifier_triangulate_roundtrip() {
        // Test JSON deserialization from spec format
        let json =
            r#"{"type":"triangulate","ngon_method":"beauty","quad_method":"longest_diagonal"}"#;
        let parsed: MeshModifier = serde_json::from_str(json).unwrap();

        match parsed {
            MeshModifier::Triangulate {
                ngon_method,
                quad_method,
            } => {
                assert_eq!(ngon_method, Some("beauty".to_string()));
                assert_eq!(quad_method, Some("longest_diagonal".to_string()));
            }
            _ => panic!("Expected Triangulate modifier"),
        }
    }

    // ========================================================================
    // UvProjection Tests - All projection methods and parameters
    // ========================================================================

    #[test]
    fn test_uv_projection_simple_smart() {
        let json = "\"smart\"";
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::Simple(UvProjectionMethod::Smart) => (),
            _ => panic!("Expected simple smart projection"),
        }

        let serialized = serde_json::to_string(&proj).unwrap();
        assert_eq!(serialized, "\"smart\"");
    }

    #[test]
    fn test_uv_projection_simple_box() {
        let json = "\"box\"";
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::Simple(UvProjectionMethod::Box) => (),
            _ => panic!("Expected simple box projection"),
        }
    }

    #[test]
    fn test_uv_projection_simple_cylinder() {
        let json = "\"cylinder\"";
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::Simple(UvProjectionMethod::Cylinder) => (),
            _ => panic!("Expected simple cylinder projection"),
        }
    }

    #[test]
    fn test_uv_projection_simple_sphere() {
        let json = "\"sphere\"";
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::Simple(UvProjectionMethod::Sphere) => (),
            _ => panic!("Expected simple sphere projection"),
        }
    }

    #[test]
    fn test_uv_projection_simple_lightmap() {
        let json = "\"lightmap\"";
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::Simple(UvProjectionMethod::Lightmap) => (),
            _ => panic!("Expected simple lightmap projection"),
        }
    }

    #[test]
    fn test_uv_projection_smart_with_angle_limit() {
        let json = r#"{"method":"smart","angle_limit":66.0}"#;
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::WithSettings {
                method,
                angle_limit,
                cube_size,
                texel_density,
                uv_margin,
                lightmap_uv,
            } => {
                assert_eq!(method, UvProjectionMethod::Smart);
                assert_eq!(angle_limit, Some(66.0));
                assert_eq!(cube_size, None);
                assert_eq!(texel_density, None);
                assert_eq!(uv_margin, None);
                assert_eq!(lightmap_uv, None);
            }
            _ => panic!("Expected projection with settings"),
        }

        let serialized = serde_json::to_string(&proj).unwrap();
        assert!(serialized.contains("smart"));
        assert!(serialized.contains("66"));
    }

    #[test]
    fn test_uv_projection_box_with_cube_size() {
        let json = r#"{"method":"box","cube_size":1.5}"#;
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::WithSettings {
                method,
                angle_limit,
                cube_size,
                texel_density,
                uv_margin,
                lightmap_uv,
            } => {
                assert_eq!(method, UvProjectionMethod::Box);
                assert_eq!(angle_limit, None);
                assert_eq!(cube_size, Some(1.5));
                assert_eq!(texel_density, None);
                assert_eq!(uv_margin, None);
                assert_eq!(lightmap_uv, None);
            }
            _ => panic!("Expected projection with settings"),
        }
    }

    #[test]
    fn test_uv_projection_with_both_settings() {
        let json = r#"{"method":"smart","angle_limit":45.0,"cube_size":2.0}"#;
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::WithSettings {
                method,
                angle_limit,
                cube_size,
                texel_density,
                uv_margin,
                lightmap_uv,
            } => {
                assert_eq!(method, UvProjectionMethod::Smart);
                assert_eq!(angle_limit, Some(45.0));
                assert_eq!(cube_size, Some(2.0));
                assert_eq!(texel_density, None);
                assert_eq!(uv_margin, None);
                assert_eq!(lightmap_uv, None);
            }
            _ => panic!("Expected projection with settings"),
        }
    }

    // ========================================================================
    // UV Projection with Texel Density and Lightmap Tests
    // ========================================================================

    #[test]
    fn test_uv_projection_with_texel_density() {
        let json = r#"{"method":"smart","texel_density":512.0}"#;
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::WithSettings {
                method,
                texel_density,
                ..
            } => {
                assert_eq!(method, UvProjectionMethod::Smart);
                assert_eq!(texel_density, Some(512.0));
            }
            _ => panic!("Expected projection with settings"),
        }

        let serialized = serde_json::to_string(&proj).unwrap();
        assert!(serialized.contains("texel_density"));
        assert!(serialized.contains("512"));
    }

    #[test]
    fn test_uv_projection_with_uv_margin() {
        let json = r#"{"method":"smart","uv_margin":0.002}"#;
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::WithSettings {
                method, uv_margin, ..
            } => {
                assert_eq!(method, UvProjectionMethod::Smart);
                assert_eq!(uv_margin, Some(0.002));
            }
            _ => panic!("Expected projection with settings"),
        }
    }

    #[test]
    fn test_uv_projection_with_lightmap_uv() {
        let json = r#"{"method":"smart","lightmap_uv":true}"#;
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::WithSettings {
                method,
                lightmap_uv,
                ..
            } => {
                assert_eq!(method, UvProjectionMethod::Smart);
                assert_eq!(lightmap_uv, Some(true));
            }
            _ => panic!("Expected projection with settings"),
        }
    }

    #[test]
    fn test_uv_projection_complete() {
        let json = r#"{"method":"smart","angle_limit":66.0,"texel_density":1024.0,"uv_margin":0.001,"lightmap_uv":true}"#;
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::WithSettings {
                method,
                angle_limit,
                cube_size,
                texel_density,
                uv_margin,
                lightmap_uv,
            } => {
                assert_eq!(method, UvProjectionMethod::Smart);
                assert_eq!(angle_limit, Some(66.0));
                assert_eq!(cube_size, None);
                assert_eq!(texel_density, Some(1024.0));
                assert_eq!(uv_margin, Some(0.001));
                assert_eq!(lightmap_uv, Some(true));
            }
            _ => panic!("Expected projection with settings"),
        }

        let serialized = serde_json::to_string(&proj).unwrap();
        assert!(serialized.contains("texel_density"));
        assert!(serialized.contains("uv_margin"));
        assert!(serialized.contains("lightmap_uv"));
    }

    #[test]
    fn test_uv_projection_lightmap_with_margin() {
        let json = r#"{"method":"lightmap","uv_margin":0.005}"#;
        let proj: UvProjection = serde_json::from_str(json).unwrap();
        match proj {
            UvProjection::WithSettings {
                method, uv_margin, ..
            } => {
                assert_eq!(method, UvProjectionMethod::Lightmap);
                assert_eq!(uv_margin, Some(0.005));
            }
            _ => panic!("Expected projection with settings"),
        }
    }
}
