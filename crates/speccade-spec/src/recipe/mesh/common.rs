//! Common/shared mesh types.

use serde::{Deserialize, Serialize};

/// Normals automation preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalsPreset {
    /// Auto-smooth normals based on angle threshold.
    AutoSmooth,
    /// Weighted normals based on face area.
    WeightedNormals,
    /// Hard edges at angles above threshold.
    HardEdgeByAngle,
    /// Flat shading (faceted).
    Flat,
    /// Smooth shading (interpolated).
    Smooth,
}

/// Normals generation settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalsSettings {
    /// Normals preset to apply.
    pub preset: NormalsPreset,
    /// Angle threshold in degrees (used by auto_smooth and hard_edge_by_angle).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<f64>,
    /// Keep existing sharp edges marked in the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_sharp: Option<bool>,
}

/// Material slot definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

pub(crate) fn default_true() -> bool {
    true
}

/// Type of bake map to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BakeType {
    /// Normal map (tangent-space by default).
    Normal,
    /// Ambient occlusion map.
    Ao,
    /// Curvature map (convex/concave edges).
    Curvature,
    /// Combined map (RGB channels: AO, Curvature, Metallic or similar packing).
    Combined,
}

/// Baking settings for high-to-low mesh map transfer.
///
/// Used to bake normal maps, AO, curvature, etc. from a high-poly source
/// onto the UVs of a low-poly target mesh.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BakingSettings {
    /// Types of maps to bake.
    pub bake_types: Vec<BakeType>,
    /// Ray distance for ray casting from low to high poly.
    /// Default: 0.1
    #[serde(default = "default_ray_distance")]
    pub ray_distance: f64,
    /// Margin (dilation) in pixels for mip-safe edges.
    /// Default: 16
    #[serde(default = "default_margin")]
    pub margin: u32,
    /// Resolution of baked textures [width, height].
    /// Default: [1024, 1024]
    #[serde(default = "default_bake_resolution")]
    pub resolution: [u32; 2],
    /// Optional path or reference to high-poly source mesh.
    /// If not specified, bakes from the mesh itself (e.g., for AO).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high_poly_source: Option<String>,
}

fn default_ray_distance() -> f64 {
    0.1
}

fn default_margin() -> u32 {
    16
}

fn default_bake_resolution() -> [u32; 2] {
    [1024, 1024]
}

impl Default for BakingSettings {
    fn default() -> Self {
        Self {
            bake_types: vec![BakeType::Normal],
            ray_distance: default_ray_distance(),
            margin: default_margin(),
            resolution: default_bake_resolution(),
            high_poly_source: None,
        }
    }
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
#[serde(deny_unknown_fields)]
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

/// Collision mesh type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CollisionType {
    /// Convex hull collision (fast, wraps around the mesh).
    #[default]
    ConvexHull,
    /// Simplified mesh collision (preserves general shape, reduced triangles).
    SimplifiedMesh,
    /// Bounding box collision (axis-aligned box).
    Box,
}

/// Collision mesh generation settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollisionMeshSettings {
    /// Type of collision mesh to generate.
    #[serde(default)]
    pub collision_type: CollisionType,
    /// Target face count for simplified mesh (only used with SimplifiedMesh type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_faces: Option<u32>,
    /// Filename suffix for collision mesh (default: "_col").
    #[serde(default = "default_collision_suffix")]
    pub output_suffix: String,
}

fn default_collision_suffix() -> String {
    "_col".to_string()
}

impl Default for CollisionMeshSettings {
    fn default() -> Self {
        Self {
            collision_type: CollisionType::ConvexHull,
            target_faces: None,
            output_suffix: default_collision_suffix(),
        }
    }
}

/// Navmesh analysis settings.
///
/// These settings control how mesh geometry is analyzed for walkability,
/// producing metadata about walkable surfaces and potential stair geometry.
/// Note: This produces classification metadata only, not actual navmesh generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavmeshSettings {
    /// Maximum slope angle in degrees for a surface to be considered walkable.
    /// Faces with normals deviating more than this from vertical are non-walkable.
    /// Default: 45.0 degrees.
    #[serde(default = "default_walkable_slope_max")]
    pub walkable_slope_max: f64,
    /// Enable stair detection analysis.
    /// When true, attempts to identify potential stair geometry.
    /// Default: false.
    #[serde(default)]
    pub stair_detection: bool,
    /// Step height threshold for stair detection in world units.
    /// Consecutive horizontal surfaces within this height difference
    /// may be classified as stairs. Only used if stair_detection is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stair_step_height: Option<f64>,
}

fn default_walkable_slope_max() -> f64 {
    45.0
}

impl Default for NavmeshSettings {
    fn default() -> Self {
        Self {
            walkable_slope_max: default_walkable_slope_max(),
            stair_detection: false,
            stair_step_height: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // ========================================================================
    // NormalsPreset Tests
    // ========================================================================

    #[test]
    fn test_normals_preset_auto_smooth() {
        let preset = NormalsPreset::AutoSmooth;
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(json, "\"auto_smooth\"");

        let parsed: NormalsPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, NormalsPreset::AutoSmooth);
    }

    #[test]
    fn test_normals_preset_weighted_normals() {
        let preset = NormalsPreset::WeightedNormals;
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(json, "\"weighted_normals\"");

        let parsed: NormalsPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, NormalsPreset::WeightedNormals);
    }

    #[test]
    fn test_normals_preset_hard_edge_by_angle() {
        let preset = NormalsPreset::HardEdgeByAngle;
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(json, "\"hard_edge_by_angle\"");

        let parsed: NormalsPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, NormalsPreset::HardEdgeByAngle);
    }

    #[test]
    fn test_normals_preset_flat() {
        let preset = NormalsPreset::Flat;
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(json, "\"flat\"");

        let parsed: NormalsPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, NormalsPreset::Flat);
    }

    #[test]
    fn test_normals_preset_smooth() {
        let preset = NormalsPreset::Smooth;
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(json, "\"smooth\"");

        let parsed: NormalsPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, NormalsPreset::Smooth);
    }

    // ========================================================================
    // NormalsSettings Tests
    // ========================================================================

    #[test]
    fn test_normals_settings_minimal() {
        let settings = NormalsSettings {
            preset: NormalsPreset::Smooth,
            angle: None,
            keep_sharp: None,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"preset\":\"smooth\""));
        assert!(!json.contains("angle"));
        assert!(!json.contains("keep_sharp"));

        let parsed: NormalsSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.preset, NormalsPreset::Smooth);
        assert_eq!(parsed.angle, None);
        assert_eq!(parsed.keep_sharp, None);
    }

    #[test]
    fn test_normals_settings_auto_smooth_with_angle() {
        let settings = NormalsSettings {
            preset: NormalsPreset::AutoSmooth,
            angle: Some(30.0),
            keep_sharp: None,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"preset\":\"auto_smooth\""));
        assert!(json.contains("\"angle\":30.0"));

        let parsed: NormalsSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.preset, NormalsPreset::AutoSmooth);
        assert_eq!(parsed.angle, Some(30.0));
    }

    #[test]
    fn test_normals_settings_weighted_with_keep_sharp() {
        let settings = NormalsSettings {
            preset: NormalsPreset::WeightedNormals,
            angle: None,
            keep_sharp: Some(true),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"preset\":\"weighted_normals\""));
        assert!(json.contains("\"keep_sharp\":true"));

        let parsed: NormalsSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.preset, NormalsPreset::WeightedNormals);
        assert_eq!(parsed.keep_sharp, Some(true));
    }

    #[test]
    fn test_normals_settings_hard_edge_complete() {
        let settings = NormalsSettings {
            preset: NormalsPreset::HardEdgeByAngle,
            angle: Some(45.0),
            keep_sharp: Some(false),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: NormalsSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.preset, NormalsPreset::HardEdgeByAngle);
        assert_eq!(parsed.angle, Some(45.0));
        assert_eq!(parsed.keep_sharp, Some(false));
    }

    #[test]
    fn test_normals_settings_from_json() {
        let json = r#"{"preset":"auto_smooth","angle":60.0,"keep_sharp":true}"#;
        let parsed: NormalsSettings = serde_json::from_str(json).unwrap();

        assert_eq!(parsed.preset, NormalsPreset::AutoSmooth);
        assert_eq!(parsed.angle, Some(60.0));
        assert_eq!(parsed.keep_sharp, Some(true));
    }

    #[test]
    fn test_normals_settings_rejects_unknown_fields() {
        let json = r#"{"preset":"smooth","unknown_field":123}"#;
        let result: Result<NormalsSettings, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ========================================================================
    // CollisionType Tests
    // ========================================================================

    #[test]
    fn test_collision_type_convex_hull() {
        let ct = CollisionType::ConvexHull;
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, "\"convex_hull\"");

        let parsed: CollisionType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, CollisionType::ConvexHull);
    }

    #[test]
    fn test_collision_type_simplified_mesh() {
        let ct = CollisionType::SimplifiedMesh;
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, "\"simplified_mesh\"");

        let parsed: CollisionType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, CollisionType::SimplifiedMesh);
    }

    #[test]
    fn test_collision_type_box() {
        let ct = CollisionType::Box;
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, "\"box\"");

        let parsed: CollisionType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, CollisionType::Box);
    }

    #[test]
    fn test_collision_type_default() {
        let ct = CollisionType::default();
        assert_eq!(ct, CollisionType::ConvexHull);
    }

    // ========================================================================
    // CollisionMeshSettings Tests
    // ========================================================================

    #[test]
    fn test_collision_mesh_settings_default() {
        let settings = CollisionMeshSettings::default();
        assert_eq!(settings.collision_type, CollisionType::ConvexHull);
        assert_eq!(settings.target_faces, None);
        assert_eq!(settings.output_suffix, "_col");
    }

    #[test]
    fn test_collision_mesh_settings_convex_hull() {
        let settings = CollisionMeshSettings {
            collision_type: CollisionType::ConvexHull,
            target_faces: None,
            output_suffix: "_col".to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"collision_type\":\"convex_hull\""));
        assert!(json.contains("\"output_suffix\":\"_col\""));
        assert!(!json.contains("target_faces"));

        let parsed: CollisionMeshSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.collision_type, CollisionType::ConvexHull);
        assert_eq!(parsed.output_suffix, "_col");
    }

    #[test]
    fn test_collision_mesh_settings_simplified_mesh() {
        let settings = CollisionMeshSettings {
            collision_type: CollisionType::SimplifiedMesh,
            target_faces: Some(64),
            output_suffix: "_col".to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"collision_type\":\"simplified_mesh\""));
        assert!(json.contains("\"target_faces\":64"));

        let parsed: CollisionMeshSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.collision_type, CollisionType::SimplifiedMesh);
        assert_eq!(parsed.target_faces, Some(64));
    }

    #[test]
    fn test_collision_mesh_settings_box() {
        let settings = CollisionMeshSettings {
            collision_type: CollisionType::Box,
            target_faces: None,
            output_suffix: "_box".to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"collision_type\":\"box\""));
        assert!(json.contains("\"output_suffix\":\"_box\""));

        let parsed: CollisionMeshSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.collision_type, CollisionType::Box);
        assert_eq!(parsed.output_suffix, "_box");
    }

    #[test]
    fn test_collision_mesh_settings_from_json_minimal() {
        let json = r#"{}"#;
        let parsed: CollisionMeshSettings = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.collision_type, CollisionType::ConvexHull);
        assert_eq!(parsed.target_faces, None);
        assert_eq!(parsed.output_suffix, "_col");
    }

    #[test]
    fn test_collision_mesh_settings_from_json_complete() {
        let json = r#"{
            "collision_type": "simplified_mesh",
            "target_faces": 128,
            "output_suffix": "_collision"
        }"#;
        let parsed: CollisionMeshSettings = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.collision_type, CollisionType::SimplifiedMesh);
        assert_eq!(parsed.target_faces, Some(128));
        assert_eq!(parsed.output_suffix, "_collision");
    }

    #[test]
    fn test_collision_mesh_settings_rejects_unknown_fields() {
        let json = r#"{"collision_type":"convex_hull","unknown_field":123}"#;
        let result: Result<CollisionMeshSettings, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ========================================================================
    // NavmeshSettings Tests
    // ========================================================================

    #[test]
    fn test_navmesh_settings_default() {
        let settings = NavmeshSettings::default();
        assert_eq!(settings.walkable_slope_max, 45.0);
        assert!(!settings.stair_detection);
        assert_eq!(settings.stair_step_height, None);
    }

    #[test]
    fn test_navmesh_settings_from_json_minimal() {
        let json = r#"{}"#;
        let parsed: NavmeshSettings = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.walkable_slope_max, 45.0);
        assert!(!parsed.stair_detection);
        assert_eq!(parsed.stair_step_height, None);
    }

    #[test]
    fn test_navmesh_settings_from_json_complete() {
        let json = r#"{
            "walkable_slope_max": 60.0,
            "stair_detection": true,
            "stair_step_height": 0.3
        }"#;
        let parsed: NavmeshSettings = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.walkable_slope_max, 60.0);
        assert!(parsed.stair_detection);
        assert_eq!(parsed.stair_step_height, Some(0.3));
    }

    #[test]
    fn test_navmesh_settings_custom_slope() {
        let settings = NavmeshSettings {
            walkable_slope_max: 30.0,
            stair_detection: false,
            stair_step_height: None,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"walkable_slope_max\":30.0"));
        assert!(!json.contains("stair_step_height"));

        let parsed: NavmeshSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.walkable_slope_max, 30.0);
        assert!(!parsed.stair_detection);
    }

    #[test]
    fn test_navmesh_settings_with_stair_detection() {
        let settings = NavmeshSettings {
            walkable_slope_max: 45.0,
            stair_detection: true,
            stair_step_height: Some(0.25),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"stair_detection\":true"));
        assert!(json.contains("\"stair_step_height\":0.25"));

        let parsed: NavmeshSettings = serde_json::from_str(&json).unwrap();
        assert!(parsed.stair_detection);
        assert_eq!(parsed.stair_step_height, Some(0.25));
    }

    #[test]
    fn test_navmesh_settings_serialization_omits_none() {
        let settings = NavmeshSettings {
            walkable_slope_max: 45.0,
            stair_detection: true,
            stair_step_height: None,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(!json.contains("stair_step_height"));
    }

    #[test]
    fn test_navmesh_settings_rejects_unknown_fields() {
        let json = r#"{"walkable_slope_max":45.0,"unknown_field":123}"#;
        let result: Result<NavmeshSettings, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_navmesh_settings_roundtrip() {
        let settings = NavmeshSettings {
            walkable_slope_max: 50.0,
            stair_detection: true,
            stair_step_height: Some(0.2),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: NavmeshSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, settings);
    }

    // ========================================================================
    // BakeType Tests
    // ========================================================================

    #[test]
    fn test_bake_type_normal() {
        let bt = BakeType::Normal;
        let json = serde_json::to_string(&bt).unwrap();
        assert_eq!(json, "\"normal\"");

        let parsed: BakeType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, BakeType::Normal);
    }

    #[test]
    fn test_bake_type_ao() {
        let bt = BakeType::Ao;
        let json = serde_json::to_string(&bt).unwrap();
        assert_eq!(json, "\"ao\"");

        let parsed: BakeType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, BakeType::Ao);
    }

    #[test]
    fn test_bake_type_curvature() {
        let bt = BakeType::Curvature;
        let json = serde_json::to_string(&bt).unwrap();
        assert_eq!(json, "\"curvature\"");

        let parsed: BakeType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, BakeType::Curvature);
    }

    #[test]
    fn test_bake_type_combined() {
        let bt = BakeType::Combined;
        let json = serde_json::to_string(&bt).unwrap();
        assert_eq!(json, "\"combined\"");

        let parsed: BakeType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, BakeType::Combined);
    }

    // ========================================================================
    // BakingSettings Tests
    // ========================================================================

    #[test]
    fn test_baking_settings_default() {
        let settings = BakingSettings::default();
        assert_eq!(settings.bake_types, vec![BakeType::Normal]);
        assert_eq!(settings.ray_distance, 0.1);
        assert_eq!(settings.margin, 16);
        assert_eq!(settings.resolution, [1024, 1024]);
        assert_eq!(settings.high_poly_source, None);
    }

    #[test]
    fn test_baking_settings_from_json_minimal() {
        let json = r#"{"bake_types":["normal"]}"#;
        let parsed: BakingSettings = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.bake_types, vec![BakeType::Normal]);
        assert_eq!(parsed.ray_distance, 0.1);
        assert_eq!(parsed.margin, 16);
        assert_eq!(parsed.resolution, [1024, 1024]);
        assert_eq!(parsed.high_poly_source, None);
    }

    #[test]
    fn test_baking_settings_from_json_complete() {
        let json = r#"{
            "bake_types": ["normal", "ao"],
            "ray_distance": 0.2,
            "margin": 32,
            "resolution": [2048, 2048],
            "high_poly_source": "meshes/high_poly.glb"
        }"#;
        let parsed: BakingSettings = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.bake_types, vec![BakeType::Normal, BakeType::Ao]);
        assert_eq!(parsed.ray_distance, 0.2);
        assert_eq!(parsed.margin, 32);
        assert_eq!(parsed.resolution, [2048, 2048]);
        assert_eq!(
            parsed.high_poly_source,
            Some("meshes/high_poly.glb".to_string())
        );
    }

    #[test]
    fn test_baking_settings_multiple_bake_types() {
        let settings = BakingSettings {
            bake_types: vec![BakeType::Normal, BakeType::Ao, BakeType::Curvature],
            ray_distance: 0.15,
            margin: 16,
            resolution: [1024, 1024],
            high_poly_source: None,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"normal\""));
        assert!(json.contains("\"ao\""));
        assert!(json.contains("\"curvature\""));

        let parsed: BakingSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.bake_types.len(), 3);
    }

    #[test]
    fn test_baking_settings_combined_bake_type() {
        let settings = BakingSettings {
            bake_types: vec![BakeType::Combined],
            ray_distance: 0.1,
            margin: 16,
            resolution: [512, 512],
            high_poly_source: None,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"combined\""));
        assert!(json.contains("\"resolution\":[512,512]"));

        let parsed: BakingSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.bake_types, vec![BakeType::Combined]);
        assert_eq!(parsed.resolution, [512, 512]);
    }

    #[test]
    fn test_baking_settings_with_high_poly_source() {
        let settings = BakingSettings {
            bake_types: vec![BakeType::Normal],
            ray_distance: 0.05,
            margin: 8,
            resolution: [4096, 4096],
            high_poly_source: Some("assets/sculpt_high.glb".to_string()),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"high_poly_source\":\"assets/sculpt_high.glb\""));

        let parsed: BakingSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.high_poly_source,
            Some("assets/sculpt_high.glb".to_string())
        );
    }

    #[test]
    fn test_baking_settings_serialization_omits_none() {
        let settings = BakingSettings {
            bake_types: vec![BakeType::Ao],
            ray_distance: 0.1,
            margin: 16,
            resolution: [1024, 1024],
            high_poly_source: None,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(!json.contains("high_poly_source"));
    }

    #[test]
    fn test_baking_settings_rejects_unknown_fields() {
        let json = r#"{"bake_types":["normal"],"unknown_field":123}"#;
        let result: Result<BakingSettings, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_baking_settings_roundtrip() {
        let settings = BakingSettings {
            bake_types: vec![BakeType::Normal, BakeType::Ao],
            ray_distance: 0.15,
            margin: 24,
            resolution: [2048, 2048],
            high_poly_source: Some("high.glb".to_string()),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: BakingSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, settings);
    }
}
