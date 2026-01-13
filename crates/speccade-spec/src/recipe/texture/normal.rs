//! Normal map specific types for texture recipes.

use serde::{Deserialize, Serialize};

use super::common::NoiseConfig;

/// Legacy parameters for the `texture.normal_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureNormalV1Params {
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

/// Post-processing options for normal maps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalMapProcessing {
    /// Gaussian blur sigma for height map smoothing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur: Option<f64>,
    /// Invert height map before conversion.
    #[serde(default = "default_invert")]
    pub invert: bool,
}

fn default_invert() -> bool {
    false
}

/// Pattern configuration for normal maps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
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
