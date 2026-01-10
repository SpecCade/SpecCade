//! Common types shared across texture recipe modules.

use serde::{Deserialize, Serialize};

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

/// Noise configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

pub(crate) fn default_octaves() -> u8 {
    4
}

pub(crate) fn default_persistence() -> f64 {
    0.5
}

pub(crate) fn default_lacunarity() -> f64 {
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
