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

    /// Reaction-diffusion field (Gray-Scott, grayscale).
    ReactionDiffusion {
        /// Number of simulation steps.
        #[serde(default = "default_rd_steps")]
        steps: u32,
        /// Feed rate.
        #[serde(default = "default_rd_feed")]
        feed: f64,
        /// Kill rate.
        #[serde(default = "default_rd_kill")]
        kill: f64,
        /// Diffusion rate for chemical A.
        #[serde(default = "default_rd_diffuse_a")]
        diffuse_a: f64,
        /// Diffusion rate for chemical B.
        #[serde(default = "default_rd_diffuse_b")]
        diffuse_b: f64,
        /// Simulation time step.
        #[serde(default = "default_rd_dt")]
        dt: f64,
        /// Initial seeded B-chemical density.
        #[serde(default = "default_rd_seed_density")]
        seed_density: f64,
    },

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

    // ---------------------------------------------------------------------
    // Stochastic tiling ops
    // ---------------------------------------------------------------------
    /// Wang tile edge-matching for seamless random tiling.
    ///
    /// Uses a simplified 2-edge Wang tile system (corner colors) to create
    /// seamless random tiling from an input texture. The input is subdivided
    /// into tiles, and edge-matching ensures seamless boundaries.
    WangTiles {
        /// Input texture node to tile.
        input: String,
        /// Number of tiles in [x, y] (e.g., [4, 4] for 16 total tiles).
        tile_count: [u32; 2],
        /// Blend width at tile edges as fraction of tile size (0.0-0.5).
        #[serde(default = "default_wang_blend_width")]
        blend_width: f64,
    },

    /// Random stamp/splat placement with overlap handling (texture bombing).
    ///
    /// Places randomized stamps of the input texture across the output,
    /// with configurable density, scale variation, rotation, and blend mode.
    TextureBomb {
        /// Input texture node to scatter.
        input: String,
        /// Density of stamps (0.0-1.0), controls how many stamps per area.
        density: f64,
        /// Scale variation range [min, max] (e.g., [0.8, 1.2]).
        #[serde(default = "default_scale_variation")]
        scale_variation: [f64; 2],
        /// Rotation variation in degrees (0 = no rotation, 360 = full random).
        #[serde(default)]
        rotation_variation: f64,
        /// Blend mode for overlapping stamps: "max", "add", "average".
        #[serde(default = "default_bomb_blend_mode")]
        blend_mode: String,
    },
}

fn default_normal_strength() -> f64 {
    1.0
}

fn default_wang_blend_width() -> f64 {
    0.1
}

fn default_scale_variation() -> [f64; 2] {
    [1.0, 1.0]
}

fn default_bomb_blend_mode() -> String {
    "max".to_string()
}

fn default_rd_steps() -> u32 {
    120
}

fn default_rd_feed() -> f64 {
    0.055
}

fn default_rd_kill() -> f64 {
    0.062
}

fn default_rd_diffuse_a() -> f64 {
    1.0
}

fn default_rd_diffuse_b() -> f64 {
    0.5
}

fn default_rd_dt() -> f64 {
    1.0
}

fn default_rd_seed_density() -> f64 {
    0.03
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

    #[test]
    fn wang_tiles_roundtrip() {
        let json = r#"
        {
          "resolution": [64, 64],
          "tileable": true,
          "nodes": [
            { "id": "src", "type": "constant", "value": 0.5 },
            { "id": "tiled", "type": "wang_tiles", "input": "src", "tile_count": [4, 4] }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();
        let node = params.nodes.iter().find(|n| n.id == "tiled").unwrap();

        let TextureProceduralOp::WangTiles {
            input,
            tile_count,
            blend_width,
        } = &node.op
        else {
            panic!("expected wang_tiles op");
        };

        assert_eq!(input, "src");
        assert_eq!(*tile_count, [4, 4]);
        assert!(
            (*blend_width - 0.1).abs() < 1e-6,
            "default blend_width should be 0.1"
        );

        let reserialized = serde_json::to_string(&params).unwrap();
        let reparsed: TextureProceduralV1Params = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(reparsed, params);
    }

    #[test]
    fn wang_tiles_custom_blend_width() {
        let json = r#"
        {
          "resolution": [64, 64],
          "tileable": true,
          "nodes": [
            { "id": "src", "type": "constant", "value": 0.5 },
            { "id": "tiled", "type": "wang_tiles", "input": "src", "tile_count": [2, 2], "blend_width": 0.25 }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();
        let node = params.nodes.iter().find(|n| n.id == "tiled").unwrap();

        let TextureProceduralOp::WangTiles { blend_width, .. } = &node.op else {
            panic!("expected wang_tiles op");
        };

        assert!((*blend_width - 0.25).abs() < 1e-6);
    }

    #[test]
    fn texture_bomb_roundtrip() {
        let json = r#"
        {
          "resolution": [64, 64],
          "tileable": true,
          "nodes": [
            { "id": "src", "type": "constant", "value": 0.5 },
            { "id": "bombed", "type": "texture_bomb", "input": "src", "density": 0.5 }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();
        let node = params.nodes.iter().find(|n| n.id == "bombed").unwrap();

        let TextureProceduralOp::TextureBomb {
            input,
            density,
            scale_variation,
            rotation_variation,
            blend_mode,
        } = &node.op
        else {
            panic!("expected texture_bomb op");
        };

        assert_eq!(input, "src");
        assert!((*density - 0.5).abs() < 1e-6);
        assert_eq!(*scale_variation, [1.0, 1.0], "default scale_variation");
        assert!(
            rotation_variation.abs() < 1e-6,
            "default rotation_variation"
        );
        assert_eq!(blend_mode, "max", "default blend_mode");

        let reserialized = serde_json::to_string(&params).unwrap();
        let reparsed: TextureProceduralV1Params = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(reparsed, params);
    }

    #[test]
    fn texture_bomb_all_params() {
        let json = r#"
        {
          "resolution": [64, 64],
          "tileable": true,
          "nodes": [
            { "id": "src", "type": "constant", "value": 0.5 },
            {
              "id": "scattered",
              "type": "texture_bomb",
              "input": "src",
              "density": 0.7,
              "scale_variation": [0.8, 1.2],
              "rotation_variation": 180.0,
              "blend_mode": "add"
            }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();
        let node = params.nodes.iter().find(|n| n.id == "scattered").unwrap();

        let TextureProceduralOp::TextureBomb {
            density,
            scale_variation,
            rotation_variation,
            blend_mode,
            ..
        } = &node.op
        else {
            panic!("expected texture_bomb op");
        };

        assert!((*density - 0.7).abs() < 1e-6);
        assert!((scale_variation[0] - 0.8).abs() < 1e-6);
        assert!((scale_variation[1] - 1.2).abs() < 1e-6);
        assert!((*rotation_variation - 180.0).abs() < 1e-6);
        assert_eq!(blend_mode, "add");
    }

    #[test]
    fn reaction_diffusion_roundtrip() {
        let json = r#"
        {
          "resolution": [64, 64],
          "tileable": true,
          "nodes": [
            {
              "id": "rd",
              "type": "reaction_diffusion",
              "steps": 180,
              "feed": 0.054,
              "kill": 0.064,
              "diffuse_a": 1.0,
              "diffuse_b": 0.5,
              "dt": 1.0,
              "seed_density": 0.04
            }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();
        let node = params.nodes.iter().find(|n| n.id == "rd").unwrap();
        let TextureProceduralOp::ReactionDiffusion {
            steps,
            feed,
            kill,
            diffuse_a,
            diffuse_b,
            dt,
            seed_density,
        } = &node.op
        else {
            panic!("expected reaction_diffusion op");
        };

        assert_eq!(*steps, 180);
        assert!((*feed - 0.054).abs() < 1e-6);
        assert!((*kill - 0.064).abs() < 1e-6);
        assert!((*diffuse_a - 1.0).abs() < 1e-6);
        assert!((*diffuse_b - 0.5).abs() < 1e-6);
        assert!((*dt - 1.0).abs() < 1e-6);
        assert!((*seed_density - 0.04).abs() < 1e-6);

        let reserialized = serde_json::to_string(&params).unwrap();
        let reparsed: TextureProceduralV1Params = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(reparsed, params);
    }

    #[test]
    fn reaction_diffusion_defaults() {
        let json = r#"
        {
          "resolution": [64, 64],
          "tileable": false,
          "nodes": [
            { "id": "rd", "type": "reaction_diffusion" }
          ]
        }
        "#;

        let params: TextureProceduralV1Params = serde_json::from_str(json).unwrap();
        let node = params.nodes.iter().find(|n| n.id == "rd").unwrap();
        let TextureProceduralOp::ReactionDiffusion {
            steps,
            feed,
            kill,
            diffuse_a,
            diffuse_b,
            dt,
            seed_density,
        } = &node.op
        else {
            panic!("expected reaction_diffusion op");
        };

        assert_eq!(*steps, 120);
        assert!((*feed - 0.055).abs() < 1e-6);
        assert!((*kill - 0.062).abs() < 1e-6);
        assert!((*diffuse_a - 1.0).abs() < 1e-6);
        assert!((*diffuse_b - 0.5).abs() < 1e-6);
        assert!((*dt - 1.0).abs() < 1e-6);
        assert!((*seed_density - 0.03).abs() < 1e-6);
    }
}
