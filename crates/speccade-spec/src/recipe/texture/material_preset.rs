//! Material preset texture recipe types.
//!
//! `texture.material_preset_v1` produces multiple PBR texture outputs (albedo, roughness,
//! metallic, normal) from a predefined material style preset with optional overrides.
//! This provides a "preset + parameterization" approach for consistent art direction.

use serde::{Deserialize, Serialize};

/// Parameters for the `texture.material_preset_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureMaterialPresetV1Params {
    /// Material preset defining the base style.
    pub preset: MaterialPresetType,
    /// Texture resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Whether textures should tile seamlessly (default: true).
    #[serde(default = "default_tileable")]
    pub tileable: bool,
    /// Optional base color override (RGB, 0.0-1.0 range).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_color: Option<[f64; 3]>,
    /// Optional roughness range override [min, max] (0.0-1.0 range).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness_range: Option<[f64; 2]>,
    /// Optional metallic value override (0.0-1.0 range).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic: Option<f64>,
    /// Optional noise scale override for detail patterns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_scale: Option<f64>,
    /// Optional pattern scale override for macro features.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_scale: Option<f64>,
}

fn default_tileable() -> bool {
    true
}

/// Material preset type defining the base PBR material style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialPresetType {
    /// Flat albedo with rim highlights and stepped roughness for stylized metal.
    ToonMetal,
    /// Wood grain pattern with warm tones and organic noise.
    StylizedWood,
    /// Dark base with bright emissive-style highlights.
    NeonGlow,
    /// Smooth, high-gloss ceramic/porcelain look.
    CeramicGlaze,
    /// Geometric patterns with metallic panels and panel lines.
    SciFiPanel,
    /// Uniform albedo with medium roughness for clean plastic surfaces.
    CleanPlastic,
    /// Rocky noise patterns with high roughness for stone surfaces.
    RoughStone,
    /// Directional anisotropic streaks for brushed metal surfaces.
    BrushedMetal,
}

impl MaterialPresetType {
    /// Returns the default base color for this preset.
    pub fn default_base_color(&self) -> [f64; 3] {
        match self {
            MaterialPresetType::ToonMetal => [0.85, 0.85, 0.9],
            MaterialPresetType::StylizedWood => [0.6, 0.4, 0.25],
            MaterialPresetType::NeonGlow => [0.1, 0.1, 0.15],
            MaterialPresetType::CeramicGlaze => [0.95, 0.95, 0.98],
            MaterialPresetType::SciFiPanel => [0.3, 0.35, 0.4],
            MaterialPresetType::CleanPlastic => [0.8, 0.8, 0.85],
            MaterialPresetType::RoughStone => [0.5, 0.45, 0.4],
            MaterialPresetType::BrushedMetal => [0.75, 0.75, 0.8],
        }
    }

    /// Returns the default roughness range [min, max] for this preset.
    pub fn default_roughness_range(&self) -> [f64; 2] {
        match self {
            MaterialPresetType::ToonMetal => [0.2, 0.5],
            MaterialPresetType::StylizedWood => [0.6, 0.9],
            MaterialPresetType::NeonGlow => [0.1, 0.3],
            MaterialPresetType::CeramicGlaze => [0.05, 0.15],
            MaterialPresetType::SciFiPanel => [0.3, 0.6],
            MaterialPresetType::CleanPlastic => [0.4, 0.6],
            MaterialPresetType::RoughStone => [0.7, 0.95],
            MaterialPresetType::BrushedMetal => [0.25, 0.45],
        }
    }

    /// Returns the default metallic value for this preset.
    pub fn default_metallic(&self) -> f64 {
        match self {
            MaterialPresetType::ToonMetal => 0.9,
            MaterialPresetType::StylizedWood => 0.0,
            MaterialPresetType::NeonGlow => 0.0,
            MaterialPresetType::CeramicGlaze => 0.0,
            MaterialPresetType::SciFiPanel => 0.8,
            MaterialPresetType::CleanPlastic => 0.0,
            MaterialPresetType::RoughStone => 0.0,
            MaterialPresetType::BrushedMetal => 0.95,
        }
    }

    /// Returns the default noise scale for this preset.
    pub fn default_noise_scale(&self) -> f64 {
        match self {
            MaterialPresetType::ToonMetal => 0.02,
            MaterialPresetType::StylizedWood => 0.05,
            MaterialPresetType::NeonGlow => 0.03,
            MaterialPresetType::CeramicGlaze => 0.01,
            MaterialPresetType::SciFiPanel => 0.04,
            MaterialPresetType::CleanPlastic => 0.01,
            MaterialPresetType::RoughStone => 0.08,
            MaterialPresetType::BrushedMetal => 0.015,
        }
    }

    /// Returns the default pattern scale for this preset.
    pub fn default_pattern_scale(&self) -> f64 {
        match self {
            MaterialPresetType::ToonMetal => 0.1,
            MaterialPresetType::StylizedWood => 0.2,
            MaterialPresetType::NeonGlow => 0.15,
            MaterialPresetType::CeramicGlaze => 0.05,
            MaterialPresetType::SciFiPanel => 0.25,
            MaterialPresetType::CleanPlastic => 0.05,
            MaterialPresetType::RoughStone => 0.15,
            MaterialPresetType::BrushedMetal => 0.1,
        }
    }
}

/// Metadata output for a generated material preset.
///
/// This is written as a JSON sidecar alongside the texture outputs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialPresetOutputMetadata {
    /// Texture resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Whether textures tile seamlessly.
    pub tileable: bool,
    /// The preset type used.
    pub preset: String,
    /// Final base color used (after overrides).
    pub base_color: [f64; 3],
    /// Final roughness range used (after overrides).
    pub roughness_range: [f64; 2],
    /// Final metallic value used (after overrides).
    pub metallic: f64,
    /// Map types generated.
    pub generated_maps: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_preset_params_roundtrip() {
        let params = TextureMaterialPresetV1Params {
            preset: MaterialPresetType::ToonMetal,
            resolution: [512, 512],
            tileable: true,
            base_color: Some([0.9, 0.85, 0.8]),
            roughness_range: Some([0.1, 0.4]),
            metallic: Some(0.95),
            noise_scale: Some(0.03),
            pattern_scale: Some(0.12),
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: TextureMaterialPresetV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn material_preset_params_minimal() {
        let json = r#"
        {
          "preset": "toon_metal",
          "resolution": [256, 256]
        }
        "#;

        let params: TextureMaterialPresetV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.preset, MaterialPresetType::ToonMetal);
        assert_eq!(params.resolution, [256, 256]);
        assert!(params.tileable); // default
        assert!(params.base_color.is_none());
        assert!(params.roughness_range.is_none());
        assert!(params.metallic.is_none());
    }

    #[test]
    fn material_preset_type_serde() {
        let presets = vec![
            (MaterialPresetType::ToonMetal, "\"toon_metal\""),
            (MaterialPresetType::StylizedWood, "\"stylized_wood\""),
            (MaterialPresetType::NeonGlow, "\"neon_glow\""),
            (MaterialPresetType::CeramicGlaze, "\"ceramic_glaze\""),
            (MaterialPresetType::SciFiPanel, "\"sci_fi_panel\""),
            (MaterialPresetType::CleanPlastic, "\"clean_plastic\""),
            (MaterialPresetType::RoughStone, "\"rough_stone\""),
            (MaterialPresetType::BrushedMetal, "\"brushed_metal\""),
        ];

        for (preset, expected_json) in presets {
            let json = serde_json::to_string(&preset).unwrap();
            assert_eq!(json, expected_json);

            let parsed: MaterialPresetType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, preset);
        }
    }

    #[test]
    fn material_preset_params_with_overrides() {
        let json = r#"
        {
          "preset": "stylized_wood",
          "resolution": [1024, 1024],
          "tileable": false,
          "base_color": [0.7, 0.5, 0.3],
          "roughness_range": [0.5, 0.8],
          "metallic": 0.0,
          "noise_scale": 0.06,
          "pattern_scale": 0.25
        }
        "#;

        let params: TextureMaterialPresetV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.preset, MaterialPresetType::StylizedWood);
        assert!(!params.tileable);
        assert_eq!(params.base_color, Some([0.7, 0.5, 0.3]));
        assert_eq!(params.roughness_range, Some([0.5, 0.8]));
        assert_eq!(params.metallic, Some(0.0));
        assert_eq!(params.noise_scale, Some(0.06));
        assert_eq!(params.pattern_scale, Some(0.25));
    }

    #[test]
    fn material_preset_params_deny_unknown_fields() {
        let json = r#"
        {
          "preset": "clean_plastic",
          "resolution": [256, 256],
          "unknown_field": "value"
        }
        "#;

        let result: Result<TextureMaterialPresetV1Params, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn preset_defaults() {
        let preset = MaterialPresetType::ToonMetal;
        let base_color = preset.default_base_color();
        let roughness_range = preset.default_roughness_range();
        let metallic = preset.default_metallic();
        let noise_scale = preset.default_noise_scale();
        let pattern_scale = preset.default_pattern_scale();

        // Verify values are in valid ranges
        for c in base_color {
            assert!((0.0..=1.0).contains(&c));
        }
        for r in roughness_range {
            assert!((0.0..=1.0).contains(&r));
        }
        assert!((0.0..=1.0).contains(&metallic));
        assert!(noise_scale > 0.0);
        assert!(pattern_scale > 0.0);
    }

    #[test]
    fn output_metadata_roundtrip() {
        let metadata = MaterialPresetOutputMetadata {
            resolution: [512, 512],
            tileable: true,
            preset: "toon_metal".to_string(),
            base_color: [0.85, 0.85, 0.9],
            roughness_range: [0.2, 0.5],
            metallic: 0.9,
            generated_maps: vec![
                "albedo".to_string(),
                "roughness".to_string(),
                "metallic".to_string(),
                "normal".to_string(),
            ],
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        let parsed: MaterialPresetOutputMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, metadata);
    }
}
