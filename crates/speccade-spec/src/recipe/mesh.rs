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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UvProjection {
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

    #[test]
    fn test_mesh_primitive_serde() {
        let prim = MeshPrimitive::Cube;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"cube\"");

        let parsed: MeshPrimitive = serde_json::from_str("\"cylinder\"").unwrap();
        assert_eq!(parsed, MeshPrimitive::Cylinder);
    }

    #[test]
    fn test_mesh_modifier_serde() {
        let modifier = MeshModifier::Bevel {
            width: 0.02,
            segments: 2,
        };

        let json = serde_json::to_string(&modifier).unwrap();
        assert!(json.contains("bevel"));
        assert!(json.contains("width"));

        let parsed: MeshModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, modifier);
    }

    #[test]
    fn test_export_settings_defaults() {
        let json = "{}";
        let settings: MeshExportSettings = serde_json::from_str(json).unwrap();
        assert!(settings.apply_modifiers);
        assert!(settings.triangulate);
        assert!(settings.include_normals);
        assert!(settings.include_uvs);
        assert!(!settings.include_vertex_colors);
    }
}
