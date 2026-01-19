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
}
