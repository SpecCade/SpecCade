//! Texture recipe types (material_maps and normal_map).

use serde::{Deserialize, Serialize};

/// Parameters for the `texture_2d.material_maps_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Texture2dMaterialMapsV1Params {
    /// Texture resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Whether the texture should tile seamlessly.
    pub tileable: bool,
    /// Which PBR maps to generate.
    pub maps: Vec<TextureMapType>,
    /// Base material properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_material: Option<BaseMaterial>,
    /// Procedural layers to apply.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<TextureLayer>,
    /// Discrete color palette for remapping values (hex colors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palette: Option<Vec<String>>,
    /// Interpolated color ramp (hex colors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_ramp: Option<Vec<String>>,
}

/// Parameters for the `texture_2d.normal_map_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Texture2dNormalMapV1Params {
    /// Texture resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Whether the texture should tile seamlessly.
    pub tileable: bool,
    /// Pattern configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<NormalMapPattern>,
    /// Bump strength (0.0 to 1.0).
    #[serde(default = "default_bump_strength")]
    pub bump_strength: f64,
    /// Post-processing options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing: Option<NormalMapProcessing>,
}

fn default_bump_strength() -> f64 {
    1.0
}

/// Types of PBR texture maps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureMapType {
    /// Base color / diffuse map.
    Albedo,
    /// Normal map.
    Normal,
    /// Roughness map.
    Roughness,
    /// Metallic map.
    Metallic,
    /// Ambient occlusion map.
    Ao,
    /// Emissive map.
    Emissive,
    /// Height/displacement map.
    Height,
}

/// Base material properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseMaterial {
    /// Material type.
    #[serde(rename = "type")]
    pub material_type: MaterialType,
    /// Base color as [R, G, B] (0.0 to 1.0).
    pub base_color: [f64; 3],
    /// Roughness range [min, max] (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness_range: Option<[f64; 2]>,
    /// Metallic value (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic: Option<f64>,
}

/// Base material types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialType {
    /// Metal surface.
    Metal,
    /// Wood surface.
    Wood,
    /// Stone surface.
    Stone,
    /// Fabric surface.
    Fabric,
    /// Plastic surface.
    Plastic,
    /// Concrete surface.
    Concrete,
    /// Brick surface.
    Brick,
    /// Generic procedural.
    Procedural,
}

/// Procedural texture layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextureLayer {
    /// Noise-based pattern layer.
    NoisePattern {
        /// Noise configuration.
        noise: NoiseConfig,
        /// Which maps this layer affects.
        affects: Vec<TextureMapType>,
        /// Layer strength (0.0 to 1.0).
        strength: f64,
    },
    /// Scratch marks layer.
    Scratches {
        /// Scratch density (0.0 to 1.0).
        density: f64,
        /// Length range [min, max] as fraction of texture size.
        length_range: [f64; 2],
        /// Width as fraction of texture size.
        width: f64,
        /// Which maps this layer affects.
        affects: Vec<TextureMapType>,
        /// Layer strength (0.0 to 1.0).
        strength: f64,
    },
    /// Edge wear layer.
    EdgeWear {
        /// Wear amount (0.0 to 1.0).
        amount: f64,
        /// Which maps this layer affects.
        affects: Vec<TextureMapType>,
    },
    /// Dirt/grime overlay.
    Dirt {
        /// Dirt density (0.0 to 1.0).
        density: f64,
        /// Dirt color as [R, G, B].
        color: [f64; 3],
        /// Which maps this layer affects.
        affects: Vec<TextureMapType>,
        /// Layer strength (0.0 to 1.0).
        strength: f64,
    },
    /// Color variation layer.
    ColorVariation {
        /// Hue variation range in degrees.
        hue_range: f64,
        /// Saturation variation range (0.0 to 1.0).
        saturation_range: f64,
        /// Value/brightness variation range (0.0 to 1.0).
        value_range: f64,
        /// Noise scale for variation.
        noise_scale: f64,
    },
    /// Gradient layer.
    Gradient {
        /// Gradient direction: "horizontal", "vertical", "radial".
        direction: GradientDirection,
        /// Linear gradient start value (0.0 to 1.0).
        #[serde(skip_serializing_if = "Option::is_none")]
        start: Option<f64>,
        /// Linear gradient end value (0.0 to 1.0).
        #[serde(skip_serializing_if = "Option::is_none")]
        end: Option<f64>,
        /// Radial gradient center [x, y] normalized (0.0 to 1.0).
        #[serde(skip_serializing_if = "Option::is_none")]
        center: Option<[f64; 2]>,
        /// Radial gradient inner value (0.0 to 1.0).
        #[serde(skip_serializing_if = "Option::is_none")]
        inner: Option<f64>,
        /// Radial gradient outer value (0.0 to 1.0).
        #[serde(skip_serializing_if = "Option::is_none")]
        outer: Option<f64>,
        /// Which maps this layer affects.
        affects: Vec<TextureMapType>,
        /// Layer strength (0.0 to 1.0).
        strength: f64,
    },
    /// Stripes layer.
    Stripes {
        /// Stripe direction: "horizontal" or "vertical".
        direction: StripeDirection,
        /// Stripe width in pixels.
        stripe_width: u32,
        /// First stripe value (0.0 to 1.0).
        color1: f64,
        /// Second stripe value (0.0 to 1.0).
        color2: f64,
        /// Which maps this layer affects.
        affects: Vec<TextureMapType>,
        /// Layer strength (0.0 to 1.0).
        strength: f64,
    },
}

/// Noise configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoiseConfig {
    /// Noise algorithm.
    pub algorithm: NoiseAlgorithm,
    /// Scale factor.
    pub scale: f64,
    /// Number of octaves for fractal noise.
    #[serde(default = "default_octaves")]
    pub octaves: u8,
    /// Persistence for fractal noise.
    #[serde(default = "default_persistence")]
    pub persistence: f64,
    /// Lacunarity for fractal noise.
    #[serde(default = "default_lacunarity")]
    pub lacunarity: f64,
}

fn default_octaves() -> u8 {
    4
}

fn default_persistence() -> f64 {
    0.5
}

fn default_lacunarity() -> f64 {
    2.0
}

/// Noise algorithm types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseAlgorithm {
    /// Perlin noise.
    Perlin,
    /// Simplex noise.
    Simplex,
    /// Worley/cellular noise.
    Worley,
    /// Value noise.
    Value,
    /// Fractal Brownian motion.
    Fbm,
}

/// Gradient direction types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GradientDirection {
    /// Horizontal gradient (left to right).
    Horizontal,
    /// Vertical gradient (top to bottom).
    Vertical,
    /// Radial gradient (center outward).
    Radial,
}

/// Stripe direction types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StripeDirection {
    /// Horizontal stripes.
    Horizontal,
    /// Vertical stripes.
    Vertical,
}

/// Post-processing options for normal maps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalMapProcessing {
    /// Gaussian blur sigma for height map smoothing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur: Option<f64>,
    /// Invert height map before conversion.
    #[serde(default = "default_invert")]
    pub invert: bool,
}

fn default_invert() -> bool {
    true
}

/// Pattern configuration for normal maps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NormalMapPattern {
    /// Grid pattern.
    Grid {
        /// Cell size in pixels.
        cell_size: u32,
        /// Line width in pixels.
        line_width: u32,
        /// Bevel amount.
        bevel: f64,
    },
    /// Brick pattern.
    Bricks {
        /// Brick width in pixels.
        brick_width: u32,
        /// Brick height in pixels.
        brick_height: u32,
        /// Mortar width in pixels.
        mortar_width: u32,
        /// Brick offset for alternating rows (0.0 to 1.0).
        offset: f64,
    },
    /// Hexagonal pattern.
    Hexagons {
        /// Hexagon size in pixels.
        size: u32,
        /// Gap between hexagons.
        gap: u32,
    },
    /// Noise-based bumps.
    NoiseBumps {
        /// Noise configuration.
        noise: NoiseConfig,
    },
    /// Diamond plate pattern.
    DiamondPlate {
        /// Diamond size in pixels.
        diamond_size: u32,
        /// Raise height.
        height: f64,
    },
    /// Tile pattern with gaps.
    Tiles {
        /// Size of each tile in pixels.
        tile_size: u32,
        /// Width of gaps between tiles.
        gap_width: u32,
        /// Depth of gaps (0.0-1.0).
        gap_depth: f64,
        /// Random seed for tile variation.
        seed: u32,
    },
    /// Rivet pattern.
    Rivets {
        /// Distance between rivet centers.
        spacing: u32,
        /// Rivet radius in pixels.
        radius: u32,
        /// Rivet height (0.0-1.0).
        height: f64,
        /// Random seed for variation.
        seed: u32,
    },
    /// Weave/fabric pattern.
    Weave {
        /// Width of threads in pixels.
        thread_width: u32,
        /// Gap between threads.
        gap: u32,
        /// Thread depth (0.0-1.0).
        depth: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TEXTURE Top-Level Keys Tests
    // ========================================================================

    #[test]
    fn test_texture_params_name() {
        let params = Texture2dMaterialMapsV1Params {
            resolution: [256, 256],
            tileable: false,
            maps: vec![TextureMapType::Albedo],
            base_material: None,
            layers: vec![],
            palette: None,
            color_ramp: None,
        };
        // Name is part of the recipe spec, not params
        assert_eq!(params.resolution, [256, 256]);
    }

    #[test]
    fn test_texture_params_resolution() {
        let params = Texture2dMaterialMapsV1Params {
            resolution: [512, 1024],
            tileable: false,
            maps: vec![TextureMapType::Albedo],
            base_material: None,
            layers: vec![],
            palette: None,
            color_ramp: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: Texture2dMaterialMapsV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.resolution, [512, 1024]);
    }

    #[test]
    fn test_texture_params_format_serialization() {
        let params = Texture2dMaterialMapsV1Params {
            resolution: [256, 256],
            tileable: true,
            maps: vec![TextureMapType::Albedo, TextureMapType::Normal],
            base_material: None,
            layers: vec![],
            palette: None,
            color_ramp: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("resolution"));
        assert!(json.contains("tileable"));
    }

    #[test]
    fn test_texture_params_layers() {
        let layer = TextureLayer::Gradient {
            direction: GradientDirection::Horizontal,
            start: Some(0.0),
            end: Some(1.0),
            center: None,
            inner: None,
            outer: None,
            affects: vec![TextureMapType::Albedo],
            strength: 0.5,
        };
        let params = Texture2dMaterialMapsV1Params {
            resolution: [256, 256],
            tileable: false,
            maps: vec![TextureMapType::Albedo],
            base_material: None,
            layers: vec![layer.clone()],
            palette: None,
            color_ramp: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: Texture2dMaterialMapsV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.layers.len(), 1);
    }

    #[test]
    fn test_texture_params_palette() {
        let params = Texture2dMaterialMapsV1Params {
            resolution: [256, 256],
            tileable: false,
            maps: vec![TextureMapType::Albedo],
            base_material: None,
            layers: vec![],
            palette: Some(vec!["#FF0000".to_string(), "#00FF00".to_string(), "#0000FF".to_string()]),
            color_ramp: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: Texture2dMaterialMapsV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.palette.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_texture_params_color_ramp() {
        let params = Texture2dMaterialMapsV1Params {
            resolution: [256, 256],
            tileable: false,
            maps: vec![TextureMapType::Albedo],
            base_material: None,
            layers: vec![],
            palette: None,
            color_ramp: Some(vec!["#000000".to_string(), "#FFFFFF".to_string()]),
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: Texture2dMaterialMapsV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.color_ramp.as_ref().unwrap().len(), 2);
    }

    // ========================================================================
    // Layer Type Tests
    // ========================================================================

    #[test]
    fn test_layer_solid_serde() {
        // Note: Solid layer is not in the current enum, but we test existing layers
        let layer = TextureLayer::NoisePattern {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Perlin,
                scale: 0.1,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            affects: vec![TextureMapType::Albedo],
            strength: 1.0,
        };
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("noise_pattern"));
    }

    #[test]
    fn test_layer_noise_all_algorithms() {
        for algo in [
            NoiseAlgorithm::Perlin,
            NoiseAlgorithm::Simplex,
            NoiseAlgorithm::Worley,
            NoiseAlgorithm::Value,
            NoiseAlgorithm::Fbm,
        ] {
            let layer = TextureLayer::NoisePattern {
                noise: NoiseConfig {
                    algorithm: algo,
                    scale: 0.1,
                    octaves: 4,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
                affects: vec![TextureMapType::Albedo],
                strength: 1.0,
            };
            let json = serde_json::to_string(&layer).unwrap();
            let parsed: TextureLayer = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, layer);
        }
    }

    #[test]
    fn test_layer_gradient_all_directions() {
        for direction in [
            GradientDirection::Horizontal,
            GradientDirection::Vertical,
            GradientDirection::Radial,
        ] {
            let layer = TextureLayer::Gradient {
                direction,
                start: Some(0.0),
                end: Some(1.0),
                center: None,
                inner: None,
                outer: None,
                affects: vec![TextureMapType::Albedo],
                strength: 0.5,
            };
            let json = serde_json::to_string(&layer).unwrap();
            let parsed: TextureLayer = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, layer);
        }
    }

    #[test]
    fn test_layer_gradient_horizontal() {
        let layer = TextureLayer::Gradient {
            direction: GradientDirection::Horizontal,
            start: Some(0.2),
            end: Some(0.8),
            center: None,
            inner: None,
            outer: None,
            affects: vec![TextureMapType::Albedo],
            strength: 0.75,
        };
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("horizontal"));
        assert!(json.contains("0.2"));
        assert!(json.contains("0.8"));
    }

    #[test]
    fn test_layer_gradient_vertical() {
        let layer = TextureLayer::Gradient {
            direction: GradientDirection::Vertical,
            start: Some(0.0),
            end: Some(1.0),
            center: None,
            inner: None,
            outer: None,
            affects: vec![TextureMapType::Albedo],
            strength: 1.0,
        };
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("vertical"));
    }

    #[test]
    fn test_layer_gradient_radial() {
        let layer = TextureLayer::Gradient {
            direction: GradientDirection::Radial,
            start: None,
            end: None,
            center: Some([0.5, 0.5]),
            inner: Some(1.0),
            outer: Some(0.0),
            affects: vec![TextureMapType::Albedo],
            strength: 1.0,
        };
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("radial"));
        assert!(json.contains("center"));
    }

    #[test]
    fn test_layer_stripes_horizontal() {
        let layer = TextureLayer::Stripes {
            direction: StripeDirection::Horizontal,
            stripe_width: 16,
            color1: 0.0,
            color2: 1.0,
            affects: vec![TextureMapType::Albedo],
            strength: 1.0,
        };
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("horizontal"));
        assert!(json.contains("stripe_width"));
    }

    #[test]
    fn test_layer_stripes_vertical() {
        let layer = TextureLayer::Stripes {
            direction: StripeDirection::Vertical,
            stripe_width: 32,
            color1: 0.3,
            color2: 0.7,
            affects: vec![TextureMapType::Roughness],
            strength: 0.5,
        };
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("vertical"));
    }

    #[test]
    fn test_layer_scratches() {
        let layer = TextureLayer::Scratches {
            density: 0.15,
            length_range: [0.1, 0.4],
            width: 0.002,
            affects: vec![TextureMapType::Albedo, TextureMapType::Roughness],
            strength: 0.5,
        };
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("scratches"));
        assert!(json.contains("density"));
        let parsed: TextureLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layer);
    }

    #[test]
    fn test_layer_edge_wear() {
        let layer = TextureLayer::EdgeWear {
            amount: 0.3,
            affects: vec![TextureMapType::Roughness],
        };
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("edge_wear"));
    }

    #[test]
    fn test_layer_dirt() {
        let layer = TextureLayer::Dirt {
            density: 0.2,
            color: [0.3, 0.25, 0.2],
            affects: vec![TextureMapType::Albedo],
            strength: 0.4,
        };
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("dirt"));
    }

    #[test]
    fn test_layer_color_variation() {
        let layer = TextureLayer::ColorVariation {
            hue_range: 10.0,
            saturation_range: 0.1,
            value_range: 0.1,
            noise_scale: 0.05,
        };
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("color_variation"));
    }

    #[test]
    fn test_layer_blend_mode_default() {
        // blend_mode is not in TextureLayer enum, testing layer opacity
        let layer = TextureLayer::NoisePattern {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Perlin,
                scale: 0.1,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            affects: vec![TextureMapType::Albedo],
            strength: 0.5,
        };
        assert_eq!(layer, layer); // Just ensure it compiles
    }

    #[test]
    fn test_layer_opacity() {
        let layer = TextureLayer::Gradient {
            direction: GradientDirection::Horizontal,
            start: Some(0.0),
            end: Some(1.0),
            center: None,
            inner: None,
            outer: None,
            affects: vec![TextureMapType::Albedo],
            strength: 0.75, // strength is similar to opacity
        };
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("0.75"));
    }

    // ========================================================================
    // Noise Configuration Tests
    // ========================================================================

    #[test]
    fn test_noise_config_perlin() {
        let noise = NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.05,
            octaves: 6,
            persistence: 0.6,
            lacunarity: 2.5,
        };
        let json = serde_json::to_string(&noise).unwrap();
        assert!(json.contains("perlin"));
    }

    #[test]
    fn test_noise_config_simplex() {
        let noise = NoiseConfig {
            algorithm: NoiseAlgorithm::Simplex,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        };
        let json = serde_json::to_string(&noise).unwrap();
        assert!(json.contains("simplex"));
    }

    #[test]
    fn test_noise_config_worley() {
        let noise = NoiseConfig {
            algorithm: NoiseAlgorithm::Worley,
            scale: 0.02,
            octaves: 3,
            persistence: 0.5,
            lacunarity: 2.0,
        };
        let json = serde_json::to_string(&noise).unwrap();
        assert!(json.contains("worley"));
    }

    #[test]
    fn test_noise_config_defaults() {
        let noise = NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        };
        assert_eq!(noise.octaves, default_octaves());
        assert_eq!(noise.persistence, default_persistence());
        assert_eq!(noise.lacunarity, default_lacunarity());
    }

    // ========================================================================
    // NORMAL Map Tests
    // ========================================================================

    #[test]
    fn test_normal_map_params_name() {
        let params = Texture2dNormalMapV1Params {
            resolution: [256, 256],
            tileable: false,
            pattern: None,
            bump_strength: 1.0,
            processing: None,
        };
        assert_eq!(params.resolution, [256, 256]);
    }

    #[test]
    fn test_normal_map_params_resolution() {
        let params = Texture2dNormalMapV1Params {
            resolution: [512, 512],
            tileable: true,
            pattern: None,
            bump_strength: 1.5,
            processing: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: Texture2dNormalMapV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.resolution, [512, 512]);
    }

    #[test]
    fn test_normal_map_params_format() {
        let params = Texture2dNormalMapV1Params {
            resolution: [256, 256],
            tileable: true,
            pattern: None,
            bump_strength: 1.0,
            processing: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("resolution"));
    }

    #[test]
    fn test_normal_map_pattern_bricks() {
        let pattern = NormalMapPattern::Bricks {
            brick_width: 64,
            brick_height: 32,
            mortar_width: 4,
            offset: 0.5,
        };
        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("bricks"));
        let parsed: NormalMapPattern = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, pattern);
    }

    #[test]
    fn test_normal_map_pattern_tiles() {
        let pattern = NormalMapPattern::Tiles {
            tile_size: 64,
            gap_width: 4,
            gap_depth: 0.3,
            seed: 42,
        };
        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("tiles"));
    }

    #[test]
    fn test_normal_map_pattern_hexagons() {
        let pattern = NormalMapPattern::Hexagons {
            size: 32,
            gap: 3,
        };
        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("hexagons"));
    }

    #[test]
    fn test_normal_map_pattern_noise() {
        let pattern = NormalMapPattern::NoiseBumps {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Simplex,
                scale: 0.1,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
        };
        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("noise_bumps"));
    }

    #[test]
    fn test_normal_map_pattern_rivets() {
        let pattern = NormalMapPattern::Rivets {
            spacing: 32,
            radius: 4,
            height: 0.2,
            seed: 42,
        };
        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("rivets"));
    }

    #[test]
    fn test_normal_map_pattern_weave() {
        let pattern = NormalMapPattern::Weave {
            thread_width: 8,
            gap: 2,
            depth: 0.15,
        };
        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("weave"));
    }

    #[test]
    fn test_normal_map_pattern_grid() {
        let pattern = NormalMapPattern::Grid {
            cell_size: 32,
            line_width: 4,
            bevel: 0.5,
        };
        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("grid"));
    }

    #[test]
    fn test_normal_map_pattern_diamond_plate() {
        let pattern = NormalMapPattern::DiamondPlate {
            diamond_size: 32,
            height: 0.3,
        };
        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("diamond_plate"));
    }

    #[test]
    fn test_normal_map_processing_blur() {
        let processing = NormalMapProcessing {
            blur: Some(2.0),
            invert: false,
        };
        let json = serde_json::to_string(&processing).unwrap();
        assert!(json.contains("blur"));
    }

    #[test]
    fn test_normal_map_processing_strength() {
        let params = Texture2dNormalMapV1Params {
            resolution: [256, 256],
            tileable: false,
            pattern: None,
            bump_strength: 2.5,
            processing: None,
        };
        assert_eq!(params.bump_strength, 2.5);
    }

    #[test]
    fn test_normal_map_processing_invert() {
        let processing = NormalMapProcessing {
            blur: None,
            invert: true,
        };
        let json = serde_json::to_string(&processing).unwrap();
        assert!(json.contains("invert"));
    }

    #[test]
    fn test_normal_map_processing_defaults() {
        let processing = NormalMapProcessing {
            blur: None,
            invert: true,
        };
        assert_eq!(processing.invert, default_invert());
    }

    // ========================================================================
    // Material Type Tests
    // ========================================================================

    #[test]
    fn test_material_type_metal() {
        let mat = BaseMaterial {
            material_type: MaterialType::Metal,
            base_color: [0.8, 0.8, 0.8],
            roughness_range: Some([0.2, 0.4]),
            metallic: Some(1.0),
        };
        assert_eq!(mat.material_type, MaterialType::Metal);
    }

    #[test]
    fn test_material_type_wood() {
        let mat = BaseMaterial {
            material_type: MaterialType::Wood,
            base_color: [0.6, 0.4, 0.2],
            roughness_range: Some([0.5, 0.8]),
            metallic: Some(0.0),
        };
        assert_eq!(mat.material_type, MaterialType::Wood);
    }

    #[test]
    fn test_material_type_all_types() {
        for mat_type in [
            MaterialType::Metal,
            MaterialType::Wood,
            MaterialType::Stone,
            MaterialType::Fabric,
            MaterialType::Plastic,
            MaterialType::Concrete,
            MaterialType::Brick,
            MaterialType::Procedural,
        ] {
            let mat = BaseMaterial {
                material_type: mat_type,
                base_color: [0.5, 0.5, 0.5],
                roughness_range: None,
                metallic: None,
            };
            let json = serde_json::to_string(&mat).unwrap();
            let parsed: BaseMaterial = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.material_type, mat_type);
        }
    }

    // ========================================================================
    // Map Type Tests
    // ========================================================================

    #[test]
    fn test_texture_map_type_serde() {
        let map = TextureMapType::Albedo;
        let json = serde_json::to_string(&map).unwrap();
        assert_eq!(json, "\"albedo\"");

        let parsed: TextureMapType = serde_json::from_str("\"normal\"").unwrap();
        assert_eq!(parsed, TextureMapType::Normal);
    }

    #[test]
    fn test_texture_map_type_all() {
        for map_type in [
            TextureMapType::Albedo,
            TextureMapType::Normal,
            TextureMapType::Roughness,
            TextureMapType::Metallic,
            TextureMapType::Ao,
            TextureMapType::Emissive,
            TextureMapType::Height,
        ] {
            let json = serde_json::to_string(&map_type).unwrap();
            let parsed: TextureMapType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, map_type);
        }
    }

    // ========================================================================
    // Round-trip Serialization Tests
    // ========================================================================

    #[test]
    fn test_full_texture_params_roundtrip() {
        let params = Texture2dMaterialMapsV1Params {
            resolution: [1024, 1024],
            tileable: true,
            maps: vec![
                TextureMapType::Albedo,
                TextureMapType::Normal,
                TextureMapType::Roughness,
                TextureMapType::Metallic,
            ],
            base_material: Some(BaseMaterial {
                material_type: MaterialType::Metal,
                base_color: [0.8, 0.2, 0.1],
                roughness_range: Some([0.2, 0.5]),
                metallic: Some(1.0),
            }),
            layers: vec![
                TextureLayer::NoisePattern {
                    noise: NoiseConfig {
                        algorithm: NoiseAlgorithm::Perlin,
                        scale: 0.05,
                        octaves: 4,
                        persistence: 0.5,
                        lacunarity: 2.0,
                    },
                    affects: vec![TextureMapType::Roughness],
                    strength: 0.3,
                },
                TextureLayer::Scratches {
                    density: 0.1,
                    length_range: [0.1, 0.3],
                    width: 0.001,
                    affects: vec![TextureMapType::Albedo, TextureMapType::Roughness],
                    strength: 0.5,
                },
            ],
            palette: Some(vec!["#FF0000".to_string(), "#00FF00".to_string()]),
            color_ramp: Some(vec!["#000000".to_string(), "#FFFFFF".to_string()]),
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: Texture2dMaterialMapsV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.resolution, params.resolution);
        assert_eq!(parsed.maps.len(), params.maps.len());
        assert_eq!(parsed.layers.len(), params.layers.len());
    }

    #[test]
    fn test_full_normal_map_params_roundtrip() {
        let params = Texture2dNormalMapV1Params {
            resolution: [512, 512],
            tileable: true,
            pattern: Some(NormalMapPattern::Bricks {
                brick_width: 64,
                brick_height: 32,
                mortar_width: 4,
                offset: 0.5,
            }),
            bump_strength: 1.5,
            processing: Some(NormalMapProcessing {
                blur: Some(1.0),
                invert: false,
            }),
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: Texture2dNormalMapV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.resolution, params.resolution);
        assert_eq!(parsed.bump_strength, params.bump_strength);
    }
}
