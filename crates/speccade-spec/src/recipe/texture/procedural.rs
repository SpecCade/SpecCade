//! Map-agnostic procedural texture recipe types.
//!
//! `texture.procedural_v1` is a deterministic DAG of named nodes producing
//! grayscale or RGBA images.

use serde::{Deserialize, Serialize};

use super::common::{GradientDirection, NoiseConfig, StripeDirection};

/// Parameters for the `texture.procedural_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureProceduralV1Params {
    /// Texture resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Whether the texture should tile seamlessly.
    pub tileable: bool,
    /// Graph nodes (a DAG). Each node has a stable id that can be referenced by other nodes and by outputs.
    pub nodes: Vec<TextureProceduralNode>,
}

/// A named graph node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextureProceduralNode {
    /// Stable node id.
    pub id: String,
    /// The node operation.
    #[serde(flatten)]
    pub op: TextureProceduralOp,
}

/// Graph node operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TextureProceduralOp {
    // ---------------------------------------------------------------------
    // Grayscale primitives
    // ---------------------------------------------------------------------

    /// A constant grayscale value.
    Constant { value: f64 },

    /// Noise field (grayscale).
    Noise { noise: NoiseConfig },

    /// Gradient (grayscale).
    Gradient {
        direction: GradientDirection,
        #[serde(skip_serializing_if = "Option::is_none")]
        start: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        center: Option<[f64; 2]>,
        #[serde(skip_serializing_if = "Option::is_none")]
        inner: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        outer: Option<f64>,
    },

    /// Stripes (grayscale).
    Stripes {
        direction: StripeDirection,
        stripe_width: u32,
        color1: f64,
        color2: f64,
    },

    /// Checkerboard (grayscale).
    Checkerboard {
        tile_size: u32,
        color1: f64,
        color2: f64,
    },

    // ---------------------------------------------------------------------
    // Grayscale ops
    // ---------------------------------------------------------------------

    /// Invert grayscale: `1 - x`.
    Invert { input: String },

    /// Clamp grayscale to a range.
    Clamp { input: String, min: f64, max: f64 },

    /// Add two grayscale inputs.
    Add { a: String, b: String },

    /// Multiply two grayscale inputs.
    Multiply { a: String, b: String },

    /// Lerp between `a` and `b` using `t` (all grayscale).
    Lerp { a: String, b: String, t: String },

    /// Threshold grayscale into {0,1}.
    Threshold { input: String, threshold: f64 },

    // ---------------------------------------------------------------------
    // Color ops
    // ---------------------------------------------------------------------

    /// Convert color -> grayscale (explicit luminance conversion).
    ToGrayscale { input: String },

    /// Map grayscale -> color using a hex ramp.
    ColorRamp { input: String, ramp: Vec<String> },

    /// Quantize color to nearest palette entry.
    Palette { input: String, palette: Vec<String> },

    /// Compose RGBA from grayscale channels.
    ComposeRgba {
        r: String,
        g: String,
        b: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        a: Option<String>,
    },

    /// Generate a normal map from a height field.
    NormalFromHeight {
        input: String,
        #[serde(default = "default_normal_strength")]
        strength: f64,
    },
}

fn default_normal_strength() -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::texture::{NoiseAlgorithm, NoiseConfig};

    #[test]
    fn procedural_params_roundtrip() {
        let params = TextureProceduralV1Params {
            resolution: [64, 64],
            tileable: true,
            nodes: vec![
                TextureProceduralNode {
                    id: "n".to_string(),
                    op: TextureProceduralOp::Noise {
                        noise: NoiseConfig {
                            algorithm: NoiseAlgorithm::Perlin,
                            scale: 0.1,
                            octaves: 2,
                            persistence: 0.5,
                            lacunarity: 2.0,
                        },
                    },
                },
                TextureProceduralNode {
                    id: "mask".to_string(),
                    op: TextureProceduralOp::Threshold {
                        input: "n".to_string(),
                        threshold: 0.5,
                    },
                },
            ],
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: TextureProceduralV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }
}
