//! Static mesh recipe types.

use serde::{Deserialize, Serialize};

/// Parameters for the `static_mesh.blender_primitives_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticMeshBlenderPrimitivesV1Params {
    /// Base Blender primitive.
    pub base_primitive: MeshPrimitive,
    /// Dimensions [X, Y, Z] in Blender units.
    pub dimensions: [f64; 3],
    /// Blender modifiers to apply.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<MeshModifier>,
    /// UV unwrapping method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_projection: Option<UvProjection>,
    /// Material slot definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_slots: Vec<MaterialSlot>,
    /// GLB export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<MeshExportSettings>,
    /// Mesh constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<MeshConstraints>,
}

/// Base mesh primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshPrimitive {
    /// Cube/box.
    Cube,
    /// UV sphere.
    Sphere,
    /// Cylinder.
    Cylinder,
    /// Cone.
    Cone,
    /// Torus.
    Torus,
    /// Plane.
    Plane,
    /// Ico sphere.
    IcoSphere,
}

/// Blender modifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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
}

/// UV projection method.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
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

/// Material slot definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialSlot {
    /// Material name.
    pub name: String,
    /// Base color as [R, G, B, A] (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_color: Option<[f64; 4]>,
    /// Metallic value (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic: Option<f64>,
    /// Roughness value (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness: Option<f64>,
    /// Emissive color as [R, G, B].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive: Option<[f64; 3]>,
    /// Emissive strength.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive_strength: Option<f64>,
}

/// GLB/glTF export settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshExportSettings {
    /// Apply all modifiers before export.
    #[serde(default = "default_true")]
    pub apply_modifiers: bool,
    /// Triangulate mesh before export.
    #[serde(default = "default_true")]
    pub triangulate: bool,
    /// Include vertex normals.
    #[serde(default = "default_true")]
    pub include_normals: bool,
    /// Include UV coordinates.
    #[serde(default = "default_true")]
    pub include_uvs: bool,
    /// Include vertex colors.
    #[serde(default)]
    pub include_vertex_colors: bool,
    /// Export tangents for normal mapping.
    #[serde(default)]
    pub tangents: bool,
}

fn default_true() -> bool {
    true
}

impl Default for MeshExportSettings {
    fn default() -> Self {
        Self {
            apply_modifiers: true,
            triangulate: true,
            include_normals: true,
            include_uvs: true,
            include_vertex_colors: false,
            tangents: false,
        }
    }
}

/// Mesh constraints for validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshConstraints {
    /// Maximum triangle count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_triangles: Option<u32>,
    /// Maximum number of materials.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_materials: Option<u32>,
    /// Maximum vertex count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_vertices: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // MeshPrimitive Tests - All primitive types
    // ========================================================================

    #[test]
    fn test_mesh_primitive_cube() {
        let prim = MeshPrimitive::Cube;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"cube\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Cube);
    }

    #[test]
    fn test_mesh_primitive_sphere() {
        let prim = MeshPrimitive::Sphere;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"sphere\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Sphere);
    }

    #[test]
    fn test_mesh_primitive_cylinder() {
        let prim = MeshPrimitive::Cylinder;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"cylinder\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Cylinder);
    }

    #[test]
    fn test_mesh_primitive_cone() {
        let prim = MeshPrimitive::Cone;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"cone\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Cone);
    }

    #[test]
    fn test_mesh_primitive_torus() {
        let prim = MeshPrimitive::Torus;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"torus\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Torus);
    }

    #[test]
    fn test_mesh_primitive_plane() {
        let prim = MeshPrimitive::Plane;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"plane\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Plane);
    }

    #[test]
    fn test_mesh_primitive_icosphere() {
        let prim = MeshPrimitive::IcoSphere;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"ico_sphere\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::IcoSphere);
    }

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
            angle_limit: Some(0.785398), // ~45 degrees in radians
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("angle_limit"));
        assert!(json.contains("0.785398"));

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
            } => {
                assert_eq!(method, UvProjectionMethod::Smart);
                assert_eq!(angle_limit, Some(66.0));
                assert_eq!(cube_size, None);
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
            } => {
                assert_eq!(method, UvProjectionMethod::Box);
                assert_eq!(angle_limit, None);
                assert_eq!(cube_size, Some(1.5));
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
            } => {
                assert_eq!(method, UvProjectionMethod::Smart);
                assert_eq!(angle_limit, Some(45.0));
                assert_eq!(cube_size, Some(2.0));
            }
            _ => panic!("Expected projection with settings"),
        }
    }

    // ========================================================================
    // MeshExportSettings Tests - All export parameters
    // ========================================================================

    #[test]
    fn test_export_settings_defaults() {
        let json = "{}";
        let settings: MeshExportSettings = serde_json::from_str(json).unwrap();
        assert!(settings.apply_modifiers);
        assert!(settings.triangulate);
        assert!(settings.include_normals);
        assert!(settings.include_uvs);
        assert!(!settings.include_vertex_colors);
        assert!(!settings.tangents);
    }

    #[test]
    fn test_export_settings_tangents_true() {
        let json = r#"{"tangents":true}"#;
        let settings: MeshExportSettings = serde_json::from_str(json).unwrap();
        assert!(settings.tangents);
    }

    #[test]
    fn test_export_settings_tangents_false() {
        let json = r#"{"tangents":false}"#;
        let settings: MeshExportSettings = serde_json::from_str(json).unwrap();
        assert!(!settings.tangents);
    }

    #[test]
    fn test_export_settings_all_options() {
        let settings = MeshExportSettings {
            apply_modifiers: false,
            triangulate: false,
            include_normals: false,
            include_uvs: false,
            include_vertex_colors: true,
            tangents: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: MeshExportSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, settings);
        assert!(!parsed.apply_modifiers);
        assert!(!parsed.triangulate);
        assert!(!parsed.include_normals);
        assert!(!parsed.include_uvs);
        assert!(parsed.include_vertex_colors);
        assert!(parsed.tangents);
    }

    // ========================================================================
    // StaticMeshBlenderPrimitivesV1Params Tests - Full integration
    // ========================================================================

    #[test]
    fn test_mesh_params_cube_basic() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Cube,
            dimensions: [1.0, 1.0, 1.0],
            modifiers: vec![],
            uv_projection: None,
            material_slots: vec![],
            export: None,
            constraints: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("cube"));
        assert!(json.contains("dimensions"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.base_primitive, MeshPrimitive::Cube);
    }

    #[test]
    fn test_mesh_params_sphere_with_dimensions() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Sphere,
            dimensions: [2.0, 2.0, 2.0],
            modifiers: vec![],
            uv_projection: None,
            material_slots: vec![],
            export: None,
            constraints: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"dimensions\":[2.0,2.0,2.0]"));
    }

    #[test]
    fn test_mesh_params_with_modifiers() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Cube,
            dimensions: [1.0, 1.0, 1.0],
            modifiers: vec![
                MeshModifier::Bevel {
                    width: 0.02,
                    segments: 2,
                    angle_limit: None,
                },
                MeshModifier::Subdivision {
                    levels: 2,
                    render_levels: 2,
                },
            ],
            uv_projection: None,
            material_slots: vec![],
            export: None,
            constraints: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("bevel"));
        assert!(json.contains("subdivision"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.modifiers.len(), 2);
    }

    #[test]
    fn test_mesh_params_with_uv_projection() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Cylinder,
            dimensions: [1.0, 1.0, 2.0],
            modifiers: vec![],
            uv_projection: Some(UvProjection::WithSettings {
                method: UvProjectionMethod::Smart,
                angle_limit: Some(66.0),
                cube_size: None,
            }),
            material_slots: vec![],
            export: None,
            constraints: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("uv_projection"));
        assert!(json.contains("smart"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.uv_projection.is_some());
    }

    #[test]
    fn test_mesh_params_with_export_settings() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Torus,
            dimensions: [1.5, 1.5, 0.5],
            modifiers: vec![],
            uv_projection: None,
            material_slots: vec![],
            export: Some(MeshExportSettings {
                apply_modifiers: true,
                triangulate: true,
                include_normals: true,
                include_uvs: true,
                include_vertex_colors: false,
                tangents: true,
            }),
            constraints: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("export"));
        assert!(json.contains("tangents"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.export.is_some());
        assert!(parsed.export.unwrap().tangents);
    }

    #[test]
    fn test_mesh_params_complete() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::IcoSphere,
            dimensions: [1.0, 1.0, 1.0],
            modifiers: vec![
                MeshModifier::Subdivision {
                    levels: 3,
                    render_levels: 4,
                },
                MeshModifier::Decimate { ratio: 0.5 },
            ],
            uv_projection: Some(UvProjection::WithSettings {
                method: UvProjectionMethod::Smart,
                angle_limit: Some(66.0),
                cube_size: None,
            }),
            material_slots: vec![],
            export: Some(MeshExportSettings {
                apply_modifiers: true,
                triangulate: true,
                include_normals: true,
                include_uvs: true,
                include_vertex_colors: false,
                tangents: true,
            }),
            constraints: Some(MeshConstraints {
                max_triangles: Some(1000),
                max_materials: Some(4),
                max_vertices: Some(2000),
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.base_primitive, MeshPrimitive::IcoSphere);
        assert_eq!(parsed.dimensions, [1.0, 1.0, 1.0]);
        assert_eq!(parsed.modifiers.len(), 2);
        assert!(parsed.uv_projection.is_some());
        assert!(parsed.export.is_some());
        assert!(parsed.constraints.is_some());
    }

    // ========================================================================
    // MaterialSlot Tests
    // ========================================================================

    #[test]
    fn test_material_slot_basic() {
        let slot = MaterialSlot {
            name: "Material".to_string(),
            base_color: Some([1.0, 0.0, 0.0, 1.0]),
            metallic: None,
            roughness: None,
            emissive: None,
            emissive_strength: None,
        };

        let json = serde_json::to_string(&slot).unwrap();
        assert!(json.contains("Material"));
        assert!(json.contains("base_color"));

        let parsed: MaterialSlot = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Material");
        assert_eq!(parsed.base_color, Some([1.0, 0.0, 0.0, 1.0]));
    }

    #[test]
    fn test_material_slot_with_metallic() {
        let slot = MaterialSlot {
            name: "Metal".to_string(),
            base_color: None,
            metallic: Some(1.0),
            roughness: Some(0.2),
            emissive: None,
            emissive_strength: None,
        };

        let json = serde_json::to_string(&slot).unwrap();
        assert!(json.contains("\"metallic\":1.0"));
        assert!(json.contains("\"roughness\":0.2"));
    }

    #[test]
    fn test_material_slot_with_emissive() {
        let slot = MaterialSlot {
            name: "Glow".to_string(),
            base_color: None,
            metallic: None,
            roughness: None,
            emissive: Some([1.0, 0.5, 0.0]),
            emissive_strength: Some(2.0),
        };

        let json = serde_json::to_string(&slot).unwrap();
        assert!(json.contains("emissive"));
        assert!(json.contains("emissive_strength"));

        let parsed: MaterialSlot = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.emissive, Some([1.0, 0.5, 0.0]));
        assert_eq!(parsed.emissive_strength, Some(2.0));
    }

    // ========================================================================
    // MeshConstraints Tests
    // ========================================================================

    #[test]
    fn test_mesh_constraints_max_triangles() {
        let constraints = MeshConstraints {
            max_triangles: Some(500),
            max_materials: None,
            max_vertices: None,
        };

        let json = serde_json::to_string(&constraints).unwrap();
        assert!(json.contains("\"max_triangles\":500"));

        let parsed: MeshConstraints = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.max_triangles, Some(500));
    }

    #[test]
    fn test_mesh_constraints_max_materials() {
        let constraints = MeshConstraints {
            max_triangles: None,
            max_materials: Some(2),
            max_vertices: None,
        };

        let json = serde_json::to_string(&constraints).unwrap();
        assert!(json.contains("\"max_materials\":2"));
    }

    #[test]
    fn test_mesh_constraints_max_vertices() {
        let constraints = MeshConstraints {
            max_triangles: None,
            max_materials: None,
            max_vertices: Some(1000),
        };

        let json = serde_json::to_string(&constraints).unwrap();
        assert!(json.contains("\"max_vertices\":1000"));
    }

    #[test]
    fn test_mesh_constraints_all() {
        let constraints = MeshConstraints {
            max_triangles: Some(1000),
            max_materials: Some(4),
            max_vertices: Some(2000),
        };

        let json = serde_json::to_string(&constraints).unwrap();
        let parsed: MeshConstraints = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.max_triangles, Some(1000));
        assert_eq!(parsed.max_materials, Some(4));
        assert_eq!(parsed.max_vertices, Some(2000));
    }
}
