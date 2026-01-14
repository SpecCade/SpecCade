//! Pattern-based height map generation for normal maps.
//!
//! This module contains height map generators for various patterns used
//! in normal map generation. Each generator creates a grayscale height map
//! that is later converted to a normal map using Sobel operators.

use speccade_spec::recipe::texture::NormalMapPattern;

use crate::maps::GrayscaleBuffer;

mod grid_patterns;
mod texture_patterns;

pub use grid_patterns::{generate_brick_height, generate_grid_height, generate_hexagon_height};
pub use texture_patterns::{
    generate_diamond_plate_height, generate_noise_height, generate_rivets_height,
    generate_tiles_height, generate_weave_height,
};

#[cfg(test)]
mod tests;

/// Generate height map from a normal map pattern.
pub fn generate_height_from_pattern(
    pattern: &NormalMapPattern,
    width: u32,
    height: u32,
    seed: u32,
    tileable: bool,
) -> GrayscaleBuffer {
    match pattern {
        NormalMapPattern::Grid {
            cell_size,
            line_width,
            bevel,
        } => generate_grid_height(width, height, *cell_size, *line_width, *bevel),
        NormalMapPattern::Bricks {
            brick_width,
            brick_height,
            mortar_width,
            offset,
        } => generate_brick_height(
            width,
            height,
            *brick_width,
            *brick_height,
            *mortar_width,
            *offset,
            seed,
        ),
        NormalMapPattern::Hexagons { size, gap } => {
            generate_hexagon_height(width, height, *size, *gap)
        }
        NormalMapPattern::NoiseBumps { noise } => {
            generate_noise_height(width, height, noise, seed, tileable)
        }
        NormalMapPattern::DiamondPlate {
            diamond_size,
            height: plate_height,
        } => generate_diamond_plate_height(width, height, *diamond_size, *plate_height),
        NormalMapPattern::Tiles {
            tile_size,
            gap_width,
            gap_depth,
            seed: tile_seed,
        } => generate_tiles_height(
            width, height, *tile_size, *gap_width, *gap_depth, *tile_seed,
        ),
        NormalMapPattern::Rivets {
            spacing,
            radius,
            height: rivet_height,
            seed: rivet_seed,
        } => generate_rivets_height(width, height, *spacing, *radius, *rivet_height, *rivet_seed),
        NormalMapPattern::Weave {
            thread_width,
            gap,
            depth,
        } => generate_weave_height(width, height, *thread_width, *gap, *depth),
    }
}
