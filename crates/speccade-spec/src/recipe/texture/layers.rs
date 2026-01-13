//! Texture layer types for procedural texture generation.

use serde::{Deserialize, Serialize};

use super::common::{GradientDirection, NoiseConfig, StripeDirection, TextureMapType};

/// Procedural texture layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
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
    /// Stain/blotch overlay.
    Stains {
        /// Noise configuration for stain placement.
        noise: NoiseConfig,
        /// Threshold for stain coverage (0.0 to 1.0).
        threshold: f64,
        /// Stain color as [R, G, B].
        color: [f64; 3],
        /// Which maps this layer affects.
        affects: Vec<TextureMapType>,
        /// Layer strength (0.0 to 1.0).
        strength: f64,
    },
    /// Water streaks/drips overlay.
    WaterStreaks {
        /// Noise configuration for streak placement.
        noise: NoiseConfig,
        /// Threshold for streak coverage (0.0 to 1.0).
        threshold: f64,
        /// Streak direction: "horizontal" or "vertical".
        direction: StripeDirection,
        /// Streak color as [R, G, B].
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
    /// Checkerboard layer.
    Checkerboard {
        /// Tile size in pixels.
        tile_size: u32,
        /// First tile value (0.0 to 1.0).
        color1: f64,
        /// Second tile value (0.0 to 1.0).
        color2: f64,
        /// Which maps this layer affects.
        affects: Vec<TextureMapType>,
        /// Layer strength (0.0 to 1.0).
        strength: f64,
    },
    /// Pitting/porous surface detail layer.
    Pitting {
        /// Noise configuration used to distribute pits.
        noise: NoiseConfig,
        /// Threshold for pit coverage (0.0 to 1.0).
        threshold: f64,
        /// Pit depth (0.0 to 1.0).
        depth: f64,
        /// Which maps this layer affects.
        affects: Vec<TextureMapType>,
    },
    /// Weave/fabric surface detail layer.
    Weave {
        /// Thread width in pixels.
        thread_width: u32,
        /// Gap between threads in pixels.
        gap: u32,
        /// Weave depth (0.0 to 1.0).
        depth: f64,
        /// Which maps this layer affects.
        affects: Vec<TextureMapType>,
    },
}
