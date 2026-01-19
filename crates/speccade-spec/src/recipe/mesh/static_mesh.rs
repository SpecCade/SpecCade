//! Static mesh recipe definitions.

use serde::{Deserialize, Serialize};

use super::common::{CollisionMeshSettings, MaterialSlot, MeshConstraints, MeshExportSettings, NavmeshSettings, NormalsSettings};
use super::modifiers::{MeshModifier, UvProjection};
use super::primitives::MeshPrimitive;

/// Decimate method for LOD generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LodDecimateMethod {
    /// Collapse edges (default, best quality).
    #[default]
    Collapse,
    /// Planar decimation (good for architectural meshes).
    Planar,
}

/// A single LOD level specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LodLevel {
    /// LOD level index (0 = highest detail / original).
    pub level: u8,
    /// Target triangle count for this LOD level.
    /// If `None`, this is the original mesh (typically LOD0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_tris: Option<u32>,
}

/// LOD chain configuration for multi-LOD mesh export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LodChainSettings {
    /// List of LOD levels to generate.
    pub levels: Vec<LodLevel>,
    /// Decimation method to use for LOD generation.
    #[serde(default, skip_serializing_if = "is_default_decimate_method")]
    pub decimate_method: LodDecimateMethod,
}

fn is_default_decimate_method(method: &LodDecimateMethod) -> bool {
    *method == LodDecimateMethod::default()
}

/// Parameters for the `static_mesh.blender_primitives_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    /// Normals automation settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normals: Option<NormalsSettings>,
    /// Material slot definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_slots: Vec<MaterialSlot>,
    /// GLB export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<MeshExportSettings>,
    /// Mesh constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<MeshConstraints>,
    /// LOD chain configuration for multi-LOD mesh export.
    /// When specified, generates multiple mesh LODs using decimation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lod_chain: Option<LodChainSettings>,
    /// Collision mesh generation settings.
    /// When specified, generates a collision mesh alongside the primary mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collision_mesh: Option<CollisionMeshSettings>,
    /// Navmesh analysis settings.
    /// When specified, analyzes mesh geometry for walkability and emits
    /// navmesh metadata (walkable/non-walkable face counts, stair detection).
    /// Note: This produces classification metadata only, not actual navmesh generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navmesh: Option<NavmeshSettings>,
}

#[cfg(test)]
mod tests {
    use super::super::common::NormalsPreset;
    use super::super::modifiers::UvProjectionMethod;
    use super::*;

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
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: None,
            navmesh: None,
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
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: None,
            navmesh: None,
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
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: None,
            navmesh: None,
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
                texel_density: None,
                uv_margin: None,
                lightmap_uv: None,
            }),
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: None,
            navmesh: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("uv_projection"));
        assert!(json.contains("smart"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.uv_projection.is_some());
    }

    #[test]
    fn test_mesh_params_with_normals_auto_smooth() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Cube,
            dimensions: [1.0, 1.0, 1.0],
            modifiers: vec![],
            uv_projection: None,
            normals: Some(NormalsSettings {
                preset: NormalsPreset::AutoSmooth,
                angle: Some(30.0),
                keep_sharp: None,
            }),
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: None,
            navmesh: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("normals"));
        assert!(json.contains("auto_smooth"));
        assert!(json.contains("\"angle\":30.0"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.normals.is_some());
        let normals = parsed.normals.unwrap();
        assert_eq!(normals.preset, NormalsPreset::AutoSmooth);
        assert_eq!(normals.angle, Some(30.0));
    }

    #[test]
    fn test_mesh_params_with_normals_weighted() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::IcoSphere,
            dimensions: [1.0, 1.0, 1.0],
            modifiers: vec![],
            uv_projection: None,
            normals: Some(NormalsSettings {
                preset: NormalsPreset::WeightedNormals,
                angle: None,
                keep_sharp: Some(true),
            }),
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: None,
            navmesh: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("weighted_normals"));
        assert!(json.contains("\"keep_sharp\":true"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        let normals = parsed.normals.unwrap();
        assert_eq!(normals.preset, NormalsPreset::WeightedNormals);
        assert_eq!(normals.keep_sharp, Some(true));
    }

    #[test]
    fn test_mesh_params_with_normals_hard_edge() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Cylinder,
            dimensions: [1.0, 1.0, 2.0],
            modifiers: vec![],
            uv_projection: None,
            normals: Some(NormalsSettings {
                preset: NormalsPreset::HardEdgeByAngle,
                angle: Some(45.0),
                keep_sharp: Some(false),
            }),
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: None,
            navmesh: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("hard_edge_by_angle"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        let normals = parsed.normals.unwrap();
        assert_eq!(normals.preset, NormalsPreset::HardEdgeByAngle);
        assert_eq!(normals.angle, Some(45.0));
    }

    #[test]
    fn test_mesh_params_with_export_settings() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Torus,
            dimensions: [1.5, 1.5, 0.5],
            modifiers: vec![],
            uv_projection: None,
            normals: None,
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
            lod_chain: None,
            collision_mesh: None,
            navmesh: None,
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
                texel_density: None,
                uv_margin: None,
                lightmap_uv: None,
            }),
            normals: Some(NormalsSettings {
                preset: NormalsPreset::AutoSmooth,
                angle: Some(60.0),
                keep_sharp: Some(true),
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
            lod_chain: None,
            collision_mesh: None,
            navmesh: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.base_primitive, MeshPrimitive::IcoSphere);
        assert_eq!(parsed.dimensions, [1.0, 1.0, 1.0]);
        assert_eq!(parsed.modifiers.len(), 2);
        assert!(parsed.uv_projection.is_some());
        assert!(parsed.normals.is_some());
        assert!(parsed.export.is_some());
        assert!(parsed.constraints.is_some());
    }

    // ========================================================================
    // LOD Chain Tests
    // ========================================================================

    #[test]
    fn test_lod_level_basic() {
        let level = LodLevel {
            level: 0,
            target_tris: None,
        };

        let json = serde_json::to_string(&level).unwrap();
        assert!(json.contains("\"level\":0"));
        assert!(!json.contains("target_tris"));

        let parsed: LodLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.level, 0);
        assert_eq!(parsed.target_tris, None);
    }

    #[test]
    fn test_lod_level_with_target() {
        let level = LodLevel {
            level: 1,
            target_tris: Some(500),
        };

        let json = serde_json::to_string(&level).unwrap();
        assert!(json.contains("\"level\":1"));
        assert!(json.contains("\"target_tris\":500"));

        let parsed: LodLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.level, 1);
        assert_eq!(parsed.target_tris, Some(500));
    }

    #[test]
    fn test_lod_decimate_method_collapse() {
        let method = LodDecimateMethod::Collapse;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"collapse\"");

        let parsed: LodDecimateMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, LodDecimateMethod::Collapse);
    }

    #[test]
    fn test_lod_decimate_method_planar() {
        let method = LodDecimateMethod::Planar;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"planar\"");

        let parsed: LodDecimateMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, LodDecimateMethod::Planar);
    }

    #[test]
    fn test_lod_chain_settings_basic() {
        let chain = LodChainSettings {
            levels: vec![
                LodLevel {
                    level: 0,
                    target_tris: None,
                },
                LodLevel {
                    level: 1,
                    target_tris: Some(500),
                },
                LodLevel {
                    level: 2,
                    target_tris: Some(100),
                },
            ],
            decimate_method: LodDecimateMethod::Collapse,
        };

        let json = serde_json::to_string(&chain).unwrap();
        assert!(json.contains("\"levels\""));
        assert!(json.contains("\"level\":0"));
        assert!(json.contains("\"level\":1"));
        assert!(json.contains("\"level\":2"));
        assert!(json.contains("\"target_tris\":500"));
        assert!(json.contains("\"target_tris\":100"));
        // Default decimate_method should not be serialized
        assert!(!json.contains("decimate_method"));

        let parsed: LodChainSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.levels.len(), 3);
        assert_eq!(parsed.levels[0].level, 0);
        assert_eq!(parsed.levels[1].target_tris, Some(500));
        assert_eq!(parsed.decimate_method, LodDecimateMethod::Collapse);
    }

    #[test]
    fn test_lod_chain_settings_planar() {
        let chain = LodChainSettings {
            levels: vec![LodLevel {
                level: 0,
                target_tris: None,
            }],
            decimate_method: LodDecimateMethod::Planar,
        };

        let json = serde_json::to_string(&chain).unwrap();
        assert!(json.contains("\"decimate_method\":\"planar\""));

        let parsed: LodChainSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.decimate_method, LodDecimateMethod::Planar);
    }

    #[test]
    fn test_mesh_params_with_lod_chain() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::IcoSphere,
            dimensions: [1.0, 1.0, 1.0],
            modifiers: vec![MeshModifier::Subdivision {
                levels: 2,
                render_levels: 2,
            }],
            uv_projection: None,
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: Some(LodChainSettings {
                levels: vec![
                    LodLevel {
                        level: 0,
                        target_tris: None,
                    },
                    LodLevel {
                        level: 1,
                        target_tris: Some(500),
                    },
                    LodLevel {
                        level: 2,
                        target_tris: Some(100),
                    },
                ],
                decimate_method: LodDecimateMethod::Collapse,
            }),
            collision_mesh: None,
            navmesh: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("lod_chain"));
        assert!(json.contains("\"target_tris\":500"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.lod_chain.is_some());
        let lod_chain = parsed.lod_chain.unwrap();
        assert_eq!(lod_chain.levels.len(), 3);
        assert_eq!(lod_chain.levels[0].target_tris, None);
        assert_eq!(lod_chain.levels[1].target_tris, Some(500));
        assert_eq!(lod_chain.levels[2].target_tris, Some(100));
    }

    #[test]
    fn test_lod_chain_from_json() {
        let json = r#"{
            "levels": [
                { "level": 0, "target_tris": null },
                { "level": 1, "target_tris": 500 },
                { "level": 2, "target_tris": 100 }
            ]
        }"#;

        let parsed: LodChainSettings = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.levels.len(), 3);
        assert_eq!(parsed.levels[0].target_tris, None);
        assert_eq!(parsed.levels[1].target_tris, Some(500));
        assert_eq!(parsed.levels[2].target_tris, Some(100));
        assert_eq!(parsed.decimate_method, LodDecimateMethod::Collapse);
    }

    #[test]
    fn test_lod_chain_rejects_unknown_fields() {
        let json = r#"{
            "levels": [{ "level": 0 }],
            "unknown_field": true
        }"#;

        let result: Result<LodChainSettings, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_lod_level_rejects_unknown_fields() {
        let json = r#"{ "level": 0, "unknown_field": true }"#;
        let result: Result<LodLevel, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ========================================================================
    // Collision Mesh Integration Tests
    // ========================================================================

    #[test]
    fn test_mesh_params_with_collision_mesh_convex_hull() {
        use super::super::common::{CollisionMeshSettings, CollisionType};

        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Cube,
            dimensions: [1.0, 1.0, 1.0],
            modifiers: vec![],
            uv_projection: None,
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: Some(CollisionMeshSettings {
                collision_type: CollisionType::ConvexHull,
                target_faces: None,
                output_suffix: "_col".to_string(),
            }),
            navmesh: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("collision_mesh"));
        assert!(json.contains("convex_hull"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.collision_mesh.is_some());
        let collision = parsed.collision_mesh.unwrap();
        assert_eq!(collision.collision_type, CollisionType::ConvexHull);
        assert_eq!(collision.output_suffix, "_col");
    }

    #[test]
    fn test_mesh_params_with_collision_mesh_simplified() {
        use super::super::common::{CollisionMeshSettings, CollisionType};

        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::IcoSphere,
            dimensions: [1.0, 1.0, 1.0],
            modifiers: vec![MeshModifier::Subdivision {
                levels: 2,
                render_levels: 2,
            }],
            uv_projection: None,
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: Some(CollisionMeshSettings {
                collision_type: CollisionType::SimplifiedMesh,
                target_faces: Some(64),
                output_suffix: "_col".to_string(),
            }),
            navmesh: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("collision_mesh"));
        assert!(json.contains("simplified_mesh"));
        assert!(json.contains("\"target_faces\":64"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.collision_mesh.is_some());
        let collision = parsed.collision_mesh.unwrap();
        assert_eq!(collision.collision_type, CollisionType::SimplifiedMesh);
        assert_eq!(collision.target_faces, Some(64));
    }

    #[test]
    fn test_mesh_params_with_collision_mesh_box() {
        use super::super::common::{CollisionMeshSettings, CollisionType};

        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Cylinder,
            dimensions: [1.0, 1.0, 2.0],
            modifiers: vec![],
            uv_projection: None,
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: Some(CollisionMeshSettings {
                collision_type: CollisionType::Box,
                target_faces: None,
                output_suffix: "_box".to_string(),
            }),
            navmesh: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"collision_type\":\"box\""));
        assert!(json.contains("\"output_suffix\":\"_box\""));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.collision_mesh.is_some());
        let collision = parsed.collision_mesh.unwrap();
        assert_eq!(collision.collision_type, CollisionType::Box);
    }

    #[test]
    fn test_mesh_params_with_lod_and_collision() {
        use super::super::common::{CollisionMeshSettings, CollisionType};

        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::IcoSphere,
            dimensions: [1.0, 1.0, 1.0],
            modifiers: vec![MeshModifier::Subdivision {
                levels: 2,
                render_levels: 2,
            }],
            uv_projection: None,
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: Some(LodChainSettings {
                levels: vec![
                    LodLevel {
                        level: 0,
                        target_tris: None,
                    },
                    LodLevel {
                        level: 1,
                        target_tris: Some(500),
                    },
                ],
                decimate_method: LodDecimateMethod::Collapse,
            }),
            collision_mesh: Some(CollisionMeshSettings {
                collision_type: CollisionType::ConvexHull,
                target_faces: None,
                output_suffix: "_col".to_string(),
            }),
            navmesh: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("lod_chain"));
        assert!(json.contains("collision_mesh"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.lod_chain.is_some());
        assert!(parsed.collision_mesh.is_some());
    }

    // ========================================================================
    // Navmesh Integration Tests
    // ========================================================================

    #[test]
    fn test_mesh_params_with_navmesh_basic() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Plane,
            dimensions: [10.0, 10.0, 0.0],
            modifiers: vec![],
            uv_projection: None,
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: None,
            navmesh: Some(NavmeshSettings::default()),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("navmesh"));
        assert!(json.contains("\"walkable_slope_max\":45.0"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.navmesh.is_some());
        let navmesh = parsed.navmesh.unwrap();
        assert_eq!(navmesh.walkable_slope_max, 45.0);
        assert!(!navmesh.stair_detection);
    }

    #[test]
    fn test_mesh_params_with_navmesh_stair_detection() {
        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Cube,
            dimensions: [2.0, 2.0, 1.0],
            modifiers: vec![],
            uv_projection: None,
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: None,
            navmesh: Some(NavmeshSettings {
                walkable_slope_max: 60.0,
                stair_detection: true,
                stair_step_height: Some(0.25),
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"stair_detection\":true"));
        assert!(json.contains("\"stair_step_height\":0.25"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        let navmesh = parsed.navmesh.unwrap();
        assert_eq!(navmesh.walkable_slope_max, 60.0);
        assert!(navmesh.stair_detection);
        assert_eq!(navmesh.stair_step_height, Some(0.25));
    }

    #[test]
    fn test_mesh_params_with_navmesh_and_collision() {
        use super::super::common::{CollisionMeshSettings, CollisionType};

        let params = StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Cube,
            dimensions: [5.0, 5.0, 1.0],
            modifiers: vec![],
            uv_projection: None,
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: Some(CollisionMeshSettings {
                collision_type: CollisionType::SimplifiedMesh,
                target_faces: Some(32),
                output_suffix: "_col".to_string(),
            }),
            navmesh: Some(NavmeshSettings {
                walkable_slope_max: 45.0,
                stair_detection: false,
                stair_step_height: None,
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("collision_mesh"));
        assert!(json.contains("navmesh"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.collision_mesh.is_some());
        assert!(parsed.navmesh.is_some());
    }

    #[test]
    fn test_mesh_params_navmesh_from_json() {
        let json = r#"{
            "base_primitive": "plane",
            "dimensions": [10.0, 10.0, 0.0],
            "navmesh": {
                "walkable_slope_max": 30.0,
                "stair_detection": true,
                "stair_step_height": 0.3
            }
        }"#;

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(json).unwrap();
        assert!(parsed.navmesh.is_some());
        let navmesh = parsed.navmesh.unwrap();
        assert_eq!(navmesh.walkable_slope_max, 30.0);
        assert!(navmesh.stair_detection);
        assert_eq!(navmesh.stair_step_height, Some(0.3));
    }
}
