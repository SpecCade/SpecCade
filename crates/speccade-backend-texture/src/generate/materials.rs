//! Material pattern application for texture generation.
//!
//! This module handles applying material-specific base patterns to height maps
//! based on the material type (brick, wood, metal, etc.).

use speccade_spec::recipe::texture::{BaseMaterial, MaterialType};

use crate::maps::GrayscaleBuffer;
use crate::noise::{Fbm, Noise2D, PerlinNoise};
use crate::pattern::{BrickPattern, WoodGrainPattern};

use super::helpers::{apply_pattern_to_buffer, apply_transform};

/// Apply material-specific base pattern to height map.
pub fn apply_material_pattern(
    height_map: &mut GrayscaleBuffer,
    material: &BaseMaterial,
    width: u32,
    height: u32,
    seed: u32,
) {
    match material.material_type {
        MaterialType::Brick => {
            let mut brick = BrickPattern::new(width, height).with_seed(seed);

            // Apply brick pattern params from spec if provided
            if let Some(ref params) = material.brick_pattern {
                brick = brick
                    .with_brick_size(params.brick_width, params.brick_height)
                    .with_row_offset(params.offset);

                // Mortar depth comes from normal_params if provided
                let mortar_depth = material
                    .normal_params
                    .as_ref()
                    .map(|np| np.mortar_depth)
                    .unwrap_or(0.3);

                brick = brick.with_mortar(params.mortar_width, mortar_depth);
            }

            apply_pattern_to_buffer(&brick, height_map);
        }
        MaterialType::Wood => {
            let wood = WoodGrainPattern::new(width, height, seed);
            apply_pattern_to_buffer(&wood, height_map);
        }
        MaterialType::Metal | MaterialType::Stone | MaterialType::Concrete => {
            // Add noise-based height variation
            let noise = Fbm::new(PerlinNoise::new(seed))
                .with_octaves(4)
                .with_persistence(0.5);

            apply_transform(height_map, |x, y, _| {
                let nx = x as f64 * 0.02;
                let ny = y as f64 * 0.02;
                noise.sample_01(nx, ny)
            });
        }
        _ => {
            // Default: slight noise variation
            let noise = Fbm::new(PerlinNoise::new(seed))
                .with_octaves(2)
                .with_persistence(0.5);

            apply_transform(height_map, |x, y, _| {
                let nx = x as f64 * 0.01;
                let ny = y as f64 * 0.01;
                let v = 0.5 + noise.sample(nx, ny) * 0.1;
                v.clamp(0.0, 1.0)
            });
        }
    }
}
