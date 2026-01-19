//! Splat set texture recipe types.
//!
//! `texture.splat_set_v1` produces terrain splat/blend textures with multiple
//! material layers (grass, dirt, rock, etc.), blend masks (RGBA splat masks),
//! per-layer PBR outputs, and macro variation overlays.

use serde::{Deserialize, Serialize};

/// Parameters for the `texture.splat_set_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureSplatSetV1Params {
    /// Texture resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Terrain layers (max 4 per splat mask).
    pub layers: Vec<SplatLayer>,
    /// Mask generation mode for determining layer blending.
    #[serde(default)]
    pub mask_mode: SplatMaskMode,
    /// Noise scale for noise-based mask generation (used when mask_mode is "noise").
    #[serde(default = "default_noise_scale")]
    pub noise_scale: f64,
    /// Whether to generate macro variation overlay texture.
    #[serde(default)]
    pub macro_variation: bool,
    /// Macro variation scale (relative to texture size).
    #[serde(default = "default_macro_scale")]
    pub macro_scale: f64,
    /// Macro variation intensity (0.0 = none, 1.0 = full).
    #[serde(default = "default_macro_intensity")]
    pub macro_intensity: f64,
}

fn default_noise_scale() -> f64 {
    0.1
}

fn default_macro_scale() -> f64 {
    0.05
}

fn default_macro_intensity() -> f64 {
    0.3
}

/// A single terrain layer in the splat set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SplatLayer {
    /// Unique layer identifier (e.g., "grass", "dirt", "rock").
    pub id: String,
    /// Base albedo color [r, g, b, a] (0.0-1.0).
    pub albedo_color: [f64; 4],
    /// Normal map strength (0.0 = flat, 1.0 = full detail).
    #[serde(default = "default_normal_strength")]
    pub normal_strength: f64,
    /// Roughness value (0.0 = smooth/shiny, 1.0 = rough/matte).
    #[serde(default = "default_roughness")]
    pub roughness: f64,
    /// Detail noise scale for the layer's texture variation.
    #[serde(default = "default_detail_scale")]
    pub detail_scale: f64,
    /// Detail noise intensity.
    #[serde(default = "default_detail_intensity")]
    pub detail_intensity: f64,
}

fn default_normal_strength() -> f64 {
    1.0
}

fn default_roughness() -> f64 {
    0.8
}

fn default_detail_scale() -> f64 {
    0.2
}

fn default_detail_intensity() -> f64 {
    0.3
}

/// Mode for generating splat mask blending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplatMaskMode {
    /// Pure noise-based blending (uniform distribution).
    #[default]
    Noise,
    /// Height-based blending (lower layers at bottom, higher at top).
    Height,
    /// Slope-based blending (flat areas vs steep areas).
    Slope,
    /// Combined height and slope blending.
    HeightSlope,
}

/// Metadata output for a generated splat set.
///
/// This is written to `{asset_id}.splat.json` alongside the texture outputs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplatSetOutputMetadata {
    /// Texture resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Layer definitions with their assigned mask channels.
    pub layers: Vec<SplatLayerMetadata>,
    /// Mask mode used for generation.
    pub mask_mode: String,
    /// Whether macro variation texture was generated.
    pub has_macro_variation: bool,
    /// Number of splat mask textures generated.
    pub splat_mask_count: u32,
}

/// Metadata for a single splat layer in the output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplatLayerMetadata {
    /// Layer identifier.
    pub id: String,
    /// Which splat mask texture this layer is in (0-indexed).
    pub mask_index: u32,
    /// Which channel in the mask (r=0, g=1, b=2, a=3).
    pub mask_channel: u32,
    /// Base roughness value.
    pub roughness: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splat_layer_roundtrip() {
        let layer = SplatLayer {
            id: "grass".to_string(),
            albedo_color: [0.2, 0.5, 0.1, 1.0],
            normal_strength: 0.8,
            roughness: 0.9,
            detail_scale: 0.15,
            detail_intensity: 0.25,
        };

        let json = serde_json::to_string_pretty(&layer).unwrap();
        let parsed: SplatLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layer);
    }

    #[test]
    fn splat_layer_defaults() {
        let json = r#"
        {
          "id": "grass",
          "albedo_color": [0.2, 0.5, 0.1, 1.0]
        }
        "#;

        let layer: SplatLayer = serde_json::from_str(json).unwrap();
        assert_eq!(layer.normal_strength, 1.0);
        assert_eq!(layer.roughness, 0.8);
        assert_eq!(layer.detail_scale, 0.2);
        assert_eq!(layer.detail_intensity, 0.3);
    }

    #[test]
    fn splat_set_params_roundtrip() {
        let params = TextureSplatSetV1Params {
            resolution: [512, 512],
            layers: vec![
                SplatLayer {
                    id: "grass".to_string(),
                    albedo_color: [0.2, 0.5, 0.1, 1.0],
                    normal_strength: 1.0,
                    roughness: 0.8,
                    detail_scale: 0.2,
                    detail_intensity: 0.3,
                },
                SplatLayer {
                    id: "dirt".to_string(),
                    albedo_color: [0.4, 0.3, 0.2, 1.0],
                    normal_strength: 0.9,
                    roughness: 0.9,
                    detail_scale: 0.15,
                    detail_intensity: 0.4,
                },
            ],
            mask_mode: SplatMaskMode::Noise,
            noise_scale: 0.1,
            macro_variation: true,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: TextureSplatSetV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn splat_set_params_from_json() {
        let json = r#"
        {
          "resolution": [512, 512],
          "layers": [
            { "id": "grass", "albedo_color": [0.2, 0.5, 0.1, 1.0], "roughness": 0.8 },
            { "id": "dirt", "albedo_color": [0.4, 0.3, 0.2, 1.0], "roughness": 0.9 },
            { "id": "rock", "albedo_color": [0.5, 0.5, 0.5, 1.0], "roughness": 0.7 }
          ],
          "mask_mode": "noise",
          "noise_scale": 0.1,
          "macro_variation": true
        }
        "#;

        let params: TextureSplatSetV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.resolution, [512, 512]);
        assert_eq!(params.layers.len(), 3);
        assert_eq!(params.layers[0].id, "grass");
        assert_eq!(params.layers[1].id, "dirt");
        assert_eq!(params.layers[2].id, "rock");
        assert_eq!(params.mask_mode, SplatMaskMode::Noise);
        assert!(params.macro_variation);
    }

    #[test]
    fn splat_set_params_defaults() {
        let json = r#"
        {
          "resolution": [256, 256],
          "layers": [
            { "id": "base", "albedo_color": [0.5, 0.5, 0.5, 1.0] }
          ]
        }
        "#;

        let params: TextureSplatSetV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.mask_mode, SplatMaskMode::Noise);
        assert_eq!(params.noise_scale, 0.1);
        assert!(!params.macro_variation);
        assert_eq!(params.macro_scale, 0.05);
        assert_eq!(params.macro_intensity, 0.3);
    }

    #[test]
    fn splat_set_mask_modes() {
        for (json_str, expected) in [
            ("\"noise\"", SplatMaskMode::Noise),
            ("\"height\"", SplatMaskMode::Height),
            ("\"slope\"", SplatMaskMode::Slope),
            ("\"height_slope\"", SplatMaskMode::HeightSlope),
        ] {
            let mode: SplatMaskMode = serde_json::from_str(json_str).unwrap();
            assert_eq!(mode, expected);
        }
    }

    #[test]
    fn splat_set_output_metadata_roundtrip() {
        let metadata = SplatSetOutputMetadata {
            resolution: [512, 512],
            layers: vec![
                SplatLayerMetadata {
                    id: "grass".to_string(),
                    mask_index: 0,
                    mask_channel: 0,
                    roughness: 0.8,
                },
                SplatLayerMetadata {
                    id: "dirt".to_string(),
                    mask_index: 0,
                    mask_channel: 1,
                    roughness: 0.9,
                },
            ],
            mask_mode: "noise".to_string(),
            has_macro_variation: true,
            splat_mask_count: 1,
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        let parsed: SplatSetOutputMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, metadata);
    }

    #[test]
    fn splat_set_max_four_layers_per_mask() {
        // Ensure we can have 4 layers (RGBA channels)
        let json = r#"
        {
          "resolution": [256, 256],
          "layers": [
            { "id": "grass", "albedo_color": [0.2, 0.5, 0.1, 1.0] },
            { "id": "dirt", "albedo_color": [0.4, 0.3, 0.2, 1.0] },
            { "id": "rock", "albedo_color": [0.5, 0.5, 0.5, 1.0] },
            { "id": "sand", "albedo_color": [0.8, 0.7, 0.5, 1.0] }
          ]
        }
        "#;

        let params: TextureSplatSetV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.layers.len(), 4);
    }
}
