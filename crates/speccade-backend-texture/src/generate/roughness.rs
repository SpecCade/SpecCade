//! Roughness map generation.
//!
//! Generates roughness maps from height data and applies layers like
//! scratches, stains, and streaks.

use speccade_spec::recipe::texture::{TextureLayer, TextureMapType};

use crate::maps::{GrayscaleBuffer, RoughnessGenerator};
use crate::pattern::ScratchesPattern;
use crate::png::{self, PngConfig};
use crate::rng::DeterministicRng;

use super::helpers::apply_pattern_to_buffer;
use super::masks::{build_streak_mask, build_threshold_mask};
use super::{GenerateError, MapResult};

/// Generate roughness map.
pub fn generate_roughness_map(
    height_map: &GrayscaleBuffer,
    layers: &[TextureLayer],
    roughness_range: [f64; 2],
    width: u32,
    height: u32,
    seed: u32,
) -> Result<MapResult, GenerateError> {
    let base_roughness = (roughness_range[0] + roughness_range[1]) / 2.0;
    let generator = RoughnessGenerator::new(base_roughness, seed)
        .with_range(roughness_range[0], roughness_range[1]);

    let mut buffer = generator.generate_from_height(height_map, true);

    // Apply scratch layers (scratches increase roughness)
    for (i, layer) in layers.iter().enumerate() {
        if let TextureLayer::Scratches {
            affects, strength, ..
        } = layer
        {
            if affects.contains(&TextureMapType::Roughness) {
                let layer_seed = DeterministicRng::derive_layer_seed(seed, i as u32);
                let scratches = ScratchesPattern::new(width, height, layer_seed);

                let mut scratch_map = GrayscaleBuffer::new(width, height, 1.0);
                apply_pattern_to_buffer(&scratches, &mut scratch_map);

                generator.apply_scratches(&mut buffer, &scratch_map, 1.0 - strength);
            }
        }
    }

    for (i, layer) in layers.iter().enumerate() {
        let layer_seed = DeterministicRng::derive_layer_seed(seed, i as u32);

        match layer {
            TextureLayer::Stains {
                noise,
                threshold,
                affects,
                strength,
                ..
            } => {
                if affects.contains(&TextureMapType::Roughness) {
                    let mask = build_threshold_mask(
                        width, height, noise, layer_seed, *threshold, *strength,
                    );

                    for y in 0..height {
                        for x in 0..width {
                            let t = mask.get(x, y);
                            if t <= 0.0 {
                                continue;
                            }
                            let current = buffer.get(x, y);
                            let blended = current + t * (1.0 - current);
                            buffer.set(x, y, blended.clamp(0.0, 1.0));
                        }
                    }
                }
            }
            TextureLayer::WaterStreaks {
                noise,
                threshold,
                direction,
                affects,
                strength,
                ..
            } => {
                if affects.contains(&TextureMapType::Roughness) {
                    let mask = build_streak_mask(
                        width, height, noise, layer_seed, *threshold, *strength, *direction,
                    );

                    for y in 0..height {
                        for x in 0..width {
                            let t = mask.get(x, y);
                            if t <= 0.0 {
                                continue;
                            }
                            let current = buffer.get(x, y);
                            let blended = current * (1.0 - t);
                            buffer.set(x, y, blended.clamp(0.0, 1.0));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let config = PngConfig::default();
    let (data, hash) = png::write_grayscale_to_vec_with_hash(&buffer, &config)?;

    Ok(MapResult {
        map_type: TextureMapType::Roughness,
        data,
        width,
        height,
        hash,
        is_color: false,
    })
}
