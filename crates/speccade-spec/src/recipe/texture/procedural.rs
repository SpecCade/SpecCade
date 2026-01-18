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
    // Blur/filter ops
    // ---------------------------------------------------------------------
    /// Gaussian blur (approximated via box blur).
    Blur { input: String, radius: f32 },

    /// Erode (morphological minimum within radius).
    Erode { input: String, radius: u32 },

    /// Dilate (morphological maximum within radius).
    Dilate { input: String, radius: u32 },

    // ---------------------------------------------------------------------
    // Warp/distortion ops
    // ---------------------------------------------------------------------
    /// Domain distortion using displacement node (x,y offset from grayscale).
    Warp {
        input: String,
        displacement: String,
        strength: f32,
    },

    // ---------------------------------------------------------------------
    // Blend modes
    // ---------------------------------------------------------------------
    /// Screen blend: 1 - (1 - base) * (1 - blend).
    BlendScreen { base: String, blend: String },

    /// Overlay blend: if base < 0.5: 2 * base * blend, else 1 - 2 * (1 - base) * (1 - blend).
    BlendOverlay { base: String, blend: String },

    /// Soft light blend.
    BlendSoftLight { base: String, blend: String },

    /// Difference blend: |base - blend|.
    BlendDifference { base: String, blend: String },

    // ---------------------------------------------------------------------
    // UV transforms
    // ---------------------------------------------------------------------
    /// Scale UVs before sampling.
    UvScale {
        input: String,
        scale_x: f32,
        scale_y: f32,
    },

    /// Rotate UVs before sampling (angle in radians).
    UvRotate { input: String, angle: f32 },

    /// Translate UVs before sampling.
    UvTranslate {
        input: String,
        offset_x: f32,
        offset_y: f32,
    },

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

    #[test]
    fn normal_from_height_strength_defaults_to_1() {
        let json = r#"
        {
          "resolution": [16, 16],
          "tileable": true,
          "nodes": [
            { "id": "h", "type": "constant", "value": 0.5 },
            { "id": "n", "type": "normal_from_height", "input": "h" }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();
        let node = params.nodes.iter().find(|n| n.id == "n").unwrap();

        let TextureProceduralOp::NormalFromHeight { input, strength } = &node.op else {
            panic!("expected normal_from_height op");
        };

        assert_eq!(input, "h");
        assert_eq!(*strength, 1.0);
    }

    #[test]
    fn compose_rgba_alpha_is_optional() {
        let json = r#"
        {
          "resolution": [16, 16],
          "tileable": true,
          "nodes": [
            { "id": "r", "type": "constant", "value": 0.1 },
            { "id": "g", "type": "constant", "value": 0.2 },
            { "id": "b", "type": "constant", "value": 0.3 },
            { "id": "rgba", "type": "compose_rgba", "r": "r", "g": "g", "b": "b" }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();
        let node = params.nodes.iter().find(|n| n.id == "rgba").unwrap();

        let TextureProceduralOp::ComposeRgba { r, g, b, a } = &node.op else {
            panic!("expected compose_rgba op");
        };

        assert_eq!(r, "r");
        assert_eq!(g, "g");
        assert_eq!(b, "b");
        assert_eq!(a.as_deref(), None);
    }

    #[test]
    fn blur_roundtrip() {
        let json = r#"
        {
          "resolution": [16, 16],
          "tileable": true,
          "nodes": [
            { "id": "c", "type": "constant", "value": 0.5 },
            { "id": "blurred", "type": "blur", "input": "c", "radius": 2.5 }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();
        let node = params.nodes.iter().find(|n| n.id == "blurred").unwrap();

        let TextureProceduralOp::Blur { input, radius } = &node.op else {
            panic!("expected blur op");
        };

        assert_eq!(input, "c");
        assert!((radius - 2.5).abs() < 1e-6);

        let reserialized = serde_json::to_string(&params).unwrap();
        let reparsed: TextureProceduralV1Params = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(reparsed, params);
    }

    #[test]
    fn morphology_roundtrip() {
        let json = r#"
        {
          "resolution": [16, 16],
          "tileable": true,
          "nodes": [
            { "id": "c", "type": "constant", "value": 0.5 },
            { "id": "eroded", "type": "erode", "input": "c", "radius": 2 },
            { "id": "dilated", "type": "dilate", "input": "c", "radius": 3 }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();

        let erode_node = params.nodes.iter().find(|n| n.id == "eroded").unwrap();
        let TextureProceduralOp::Erode { input, radius } = &erode_node.op else {
            panic!("expected erode op");
        };
        assert_eq!(input, "c");
        assert_eq!(*radius, 2);

        let dilate_node = params.nodes.iter().find(|n| n.id == "dilated").unwrap();
        let TextureProceduralOp::Dilate { input, radius } = &dilate_node.op else {
            panic!("expected dilate op");
        };
        assert_eq!(input, "c");
        assert_eq!(*radius, 3);

        let reserialized = serde_json::to_string(&params).unwrap();
        let reparsed: TextureProceduralV1Params = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(reparsed, params);
    }

    #[test]
    fn warp_roundtrip() {
        let json = r#"
        {
          "resolution": [16, 16],
          "tileable": true,
          "nodes": [
            { "id": "src", "type": "constant", "value": 0.5 },
            { "id": "disp", "type": "constant", "value": 0.25 },
            { "id": "warped", "type": "warp", "input": "src", "displacement": "disp", "strength": 10.0 }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();
        let node = params.nodes.iter().find(|n| n.id == "warped").unwrap();

        let TextureProceduralOp::Warp {
            input,
            displacement,
            strength,
        } = &node.op
        else {
            panic!("expected warp op");
        };

        assert_eq!(input, "src");
        assert_eq!(displacement, "disp");
        assert!((strength - 10.0).abs() < 1e-6);

        let reserialized = serde_json::to_string(&params).unwrap();
        let reparsed: TextureProceduralV1Params = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(reparsed, params);
    }

    #[test]
    fn blend_modes_roundtrip() {
        let json = r#"
        {
          "resolution": [16, 16],
          "tileable": true,
          "nodes": [
            { "id": "a", "type": "constant", "value": 0.3 },
            { "id": "b", "type": "constant", "value": 0.7 },
            { "id": "screen", "type": "blend_screen", "base": "a", "blend": "b" },
            { "id": "overlay", "type": "blend_overlay", "base": "a", "blend": "b" },
            { "id": "softlight", "type": "blend_soft_light", "base": "a", "blend": "b" },
            { "id": "diff", "type": "blend_difference", "base": "a", "blend": "b" }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();

        let screen = params.nodes.iter().find(|n| n.id == "screen").unwrap();
        assert!(
            matches!(&screen.op, TextureProceduralOp::BlendScreen { base, blend } if base == "a" && blend == "b")
        );

        let overlay = params.nodes.iter().find(|n| n.id == "overlay").unwrap();
        assert!(
            matches!(&overlay.op, TextureProceduralOp::BlendOverlay { base, blend } if base == "a" && blend == "b")
        );

        let softlight = params.nodes.iter().find(|n| n.id == "softlight").unwrap();
        assert!(
            matches!(&softlight.op, TextureProceduralOp::BlendSoftLight { base, blend } if base == "a" && blend == "b")
        );

        let diff = params.nodes.iter().find(|n| n.id == "diff").unwrap();
        assert!(
            matches!(&diff.op, TextureProceduralOp::BlendDifference { base, blend } if base == "a" && blend == "b")
        );

        let reserialized = serde_json::to_string(&params).unwrap();
        let reparsed: TextureProceduralV1Params = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(reparsed, params);
    }

    #[test]
    fn uv_transforms_roundtrip() {
        let json = r#"
        {
          "resolution": [16, 16],
          "tileable": true,
          "nodes": [
            { "id": "src", "type": "constant", "value": 0.5 },
            { "id": "scaled", "type": "uv_scale", "input": "src", "scale_x": 2.0, "scale_y": 3.0 },
            { "id": "rotated", "type": "uv_rotate", "input": "src", "angle": 1.57 },
            { "id": "translated", "type": "uv_translate", "input": "src", "offset_x": 0.25, "offset_y": 0.5 }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();

        let scaled = params.nodes.iter().find(|n| n.id == "scaled").unwrap();
        let TextureProceduralOp::UvScale {
            input,
            scale_x,
            scale_y,
        } = &scaled.op
        else {
            panic!("expected uv_scale op");
        };
        assert_eq!(input, "src");
        assert!((scale_x - 2.0).abs() < 1e-6);
        assert!((scale_y - 3.0).abs() < 1e-6);

        let rotated = params.nodes.iter().find(|n| n.id == "rotated").unwrap();
        let TextureProceduralOp::UvRotate { input, angle } = &rotated.op else {
            panic!("expected uv_rotate op");
        };
        assert_eq!(input, "src");
        assert!((angle - 1.57).abs() < 1e-6);

        let translated = params.nodes.iter().find(|n| n.id == "translated").unwrap();
        let TextureProceduralOp::UvTranslate {
            input,
            offset_x,
            offset_y,
        } = &translated.op
        else {
            panic!("expected uv_translate op");
        };
        assert_eq!(input, "src");
        assert!((offset_x - 0.25).abs() < 1e-6);
        assert!((offset_y - 0.5).abs() < 1e-6);

        let reserialized = serde_json::to_string(&params).unwrap();
        let reparsed: TextureProceduralV1Params = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(reparsed, params);
    }
}
