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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_map_type_serde() {
        let map = TextureMapType::Albedo;
        let json = serde_json::to_string(&map).unwrap();
        assert_eq!(json, "\"albedo\"");

        let parsed: TextureMapType = serde_json::from_str("\"normal\"").unwrap();
        assert_eq!(parsed, TextureMapType::Normal);
    }

    #[test]
    fn test_texture_layer_serde() {
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
    fn test_normal_map_pattern_serde() {
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
}
