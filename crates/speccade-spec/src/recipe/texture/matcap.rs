//! Matcap texture recipe types.
//!
//! `texture.matcap_v1` produces stylized shading textures for NPR (non-photorealistic rendering).
//! Matcaps (material capture) encode lighting and shading in a 2D texture that maps surface
//! normals to colors, providing a fast way to achieve stylized looks.

use serde::{Deserialize, Serialize};

/// Parameters for the `texture.matcap_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureMatcapV1Params {
    /// Texture resolution [width, height] in pixels (typically square, e.g., 256x256 or 512x512).
    pub resolution: [u32; 2],
    /// Matcap preset defining the base lighting/shading style.
    pub preset: MatcapPreset,
    /// Optional base color override (RGB, 0.0-1.0 range). If not specified, uses preset default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_color: Option<[f64; 3]>,
    /// Optional toon shading steps (2-16). Quantizes lighting into discrete bands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toon_steps: Option<u32>,
    /// Optional outline configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline: Option<OutlineConfig>,
    /// Optional curvature mask configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curvature_mask: Option<CurvatureMaskConfig>,
    /// Optional cavity mask configuration (darkens concave areas).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cavity_mask: Option<CavityMaskConfig>,
}

/// Matcap preset defining the base lighting and shading style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatcapPreset {
    /// Basic toon shading with clear light/shadow separation.
    ToonBasic,
    /// Toon shading with rim lighting highlight.
    ToonRim,
    /// Metallic shading with strong specular highlights.
    Metallic,
    /// Ceramic/porcelain shading with soft diffuse falloff.
    Ceramic,
    /// Clay/matte shading with no specular.
    Clay,
    /// Skin/subsurface shading with soft transitions.
    Skin,
    /// Glossy plastic with sharp highlights.
    Plastic,
    /// Velvet/fabric with anisotropic highlights.
    Velvet,
}

/// Outline configuration for edge detection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutlineConfig {
    /// Outline width in pixels (1-10).
    pub width: u32,
    /// Outline color (RGB, 0.0-1.0 range).
    pub color: [f64; 3],
}

/// Curvature mask configuration.
///
/// Highlights areas of high curvature (edges/ridges) based on procedural approximation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurvatureMaskConfig {
    /// Whether curvature masking is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Curvature strength (0.0-1.0). Higher values increase contrast.
    #[serde(default = "default_strength")]
    pub strength: f64,
}

/// Cavity mask configuration.
///
/// Darkens concave areas (cavities) based on procedural approximation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CavityMaskConfig {
    /// Whether cavity masking is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Cavity darkening strength (0.0-1.0).
    #[serde(default = "default_strength")]
    pub strength: f64,
}

fn default_enabled() -> bool {
    true
}

fn default_strength() -> f64 {
    0.5
}

impl Default for CurvatureMaskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strength: 0.5,
        }
    }
}

impl Default for CavityMaskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strength: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matcap_params_roundtrip() {
        let params = TextureMatcapV1Params {
            resolution: [512, 512],
            preset: MatcapPreset::ToonBasic,
            base_color: Some([0.8, 0.2, 0.2]),
            toon_steps: Some(4),
            outline: Some(OutlineConfig {
                width: 2,
                color: [0.0, 0.0, 0.0],
            }),
            curvature_mask: Some(CurvatureMaskConfig {
                enabled: true,
                strength: 0.6,
            }),
            cavity_mask: Some(CavityMaskConfig {
                enabled: true,
                strength: 0.4,
            }),
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: TextureMatcapV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn matcap_params_minimal() {
        let json = r#"
        {
          "resolution": [256, 256],
          "preset": "metallic"
        }
        "#;

        let params: TextureMatcapV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.resolution, [256, 256]);
        assert_eq!(params.preset, MatcapPreset::Metallic);
        assert!(params.base_color.is_none());
        assert!(params.toon_steps.is_none());
    }

    #[test]
    fn matcap_preset_serde() {
        let presets = vec![
            (MatcapPreset::ToonBasic, "\"toon_basic\""),
            (MatcapPreset::ToonRim, "\"toon_rim\""),
            (MatcapPreset::Metallic, "\"metallic\""),
            (MatcapPreset::Ceramic, "\"ceramic\""),
            (MatcapPreset::Clay, "\"clay\""),
            (MatcapPreset::Skin, "\"skin\""),
            (MatcapPreset::Plastic, "\"plastic\""),
            (MatcapPreset::Velvet, "\"velvet\""),
        ];

        for (preset, expected_json) in presets {
            let json = serde_json::to_string(&preset).unwrap();
            assert_eq!(json, expected_json);

            let parsed: MatcapPreset = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, preset);
        }
    }

    #[test]
    fn matcap_params_with_outline() {
        let json = r#"
        {
          "resolution": [512, 512],
          "preset": "toon_basic",
          "toon_steps": 3,
          "outline": {
            "width": 3,
            "color": [0.1, 0.1, 0.1]
          }
        }
        "#;

        let params: TextureMatcapV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.toon_steps, Some(3));
        assert!(params.outline.is_some());
        let outline = params.outline.unwrap();
        assert_eq!(outline.width, 3);
        assert_eq!(outline.color, [0.1, 0.1, 0.1]);
    }

    #[test]
    fn matcap_params_deny_unknown_fields() {
        let json = r#"
        {
          "resolution": [256, 256],
          "preset": "clay",
          "unknown_field": "value"
        }
        "#;

        let result: Result<TextureMatcapV1Params, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn curvature_mask_defaults() {
        let config = CurvatureMaskConfig::default();
        assert!(config.enabled);
        assert_eq!(config.strength, 0.5);
    }

    #[test]
    fn cavity_mask_defaults() {
        let config = CavityMaskConfig::default();
        assert!(config.enabled);
        assert_eq!(config.strength, 0.5);
    }
}
