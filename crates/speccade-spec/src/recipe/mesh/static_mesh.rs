//! Static mesh recipe definitions.

use serde::{Deserialize, Serialize};

use super::common::{MaterialSlot, MeshConstraints, MeshExportSettings, NormalsSettings};
use super::modifiers::{MeshModifier, UvProjection};
use super::primitives::MeshPrimitive;

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
            normals: None,
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
}
