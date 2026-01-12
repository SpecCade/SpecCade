//! Layer application logic for texture generation.
//!
//! This module handles applying texture layers (noise patterns, scratches,
//! edge wear, gradients, stripes) to height maps and color buffers.

use speccade_spec::recipe::texture::{GradientDirection, StripeDirection, TextureLayer};

use crate::maps::GrayscaleBuffer;
use crate::pattern::{
    CheckerPattern, EdgeWearPattern, GradientPattern, ScratchesPattern, StripesPattern,
};

use super::helpers::{apply_pattern_blended, apply_transform, create_noise_generator, BlendMode};

/// Apply a layer to the height map.
pub fn apply_layer_to_height(height_map: &mut GrayscaleBuffer, layer: &TextureLayer, seed: u32) {
    let width = height_map.width;
    let height = height_map.height;

    match layer {
        TextureLayer::NoisePattern {
            noise, strength, ..
        } => {
            let noise_gen = create_noise_generator(noise, seed);
            let scale = noise.scale;
            let strength = *strength;

            apply_transform(height_map, |x, y, current| {
                let nx = x as f64 * scale;
                let ny = y as f64 * scale;
                let noise_val = noise_gen.sample_01(nx, ny);
                current + (noise_val - 0.5) * strength
            });
        }
        TextureLayer::Scratches {
            density,
            length_range,
            width: scratch_width,
            strength,
            ..
        } => {
            let count = (density * 100.0) as u32;
            let scratches = ScratchesPattern::new(width, height, seed)
                .with_count(count)
                .with_length_range(length_range[0], length_range[1])
                .with_width(*scratch_width * width as f64)
                .with_depth(*strength);

            // Use Min blend mode: scratches cut into the surface
            apply_pattern_blended(&scratches, height_map, BlendMode::Min, 1.0);
        }
        TextureLayer::EdgeWear { amount, .. } => {
            let edge_wear = EdgeWearPattern::new(width, height, seed)
                .with_amount(*amount)
                .with_height_map(height_map.data.clone());

            // Edge wear creates worn areas: multiply by (1 - wear * 0.3)
            apply_pattern_blended(&edge_wear, height_map, BlendMode::MultiplyInverse, 0.3);
        }
        TextureLayer::Gradient {
            direction,
            start,
            end,
            center,
            inner,
            outer,
            strength,
            ..
        } => {
            let gradient = match direction {
                GradientDirection::Horizontal => {
                    let s = start.unwrap_or(0.0);
                    let e = end.unwrap_or(1.0);
                    GradientPattern::new_horizontal(width, height, s, e)
                }
                GradientDirection::Vertical => {
                    let s = start.unwrap_or(0.0);
                    let e = end.unwrap_or(1.0);
                    GradientPattern::new_vertical(width, height, s, e)
                }
                GradientDirection::Radial => {
                    let c = center.unwrap_or([0.5, 0.5]);
                    let i = inner.unwrap_or(1.0);
                    let o = outer.unwrap_or(0.0);
                    GradientPattern::new_radial(width, height, c, i, o)
                }
            };

            // Blend gradient with current value using linear interpolation
            apply_pattern_blended(&gradient, height_map, BlendMode::Lerp, *strength);
        }
        TextureLayer::Stripes {
            direction,
            stripe_width,
            color1,
            color2,
            strength,
            ..
        } => {
            let stripes = match direction {
                StripeDirection::Horizontal => {
                    StripesPattern::new_horizontal(*stripe_width, *color1, *color2)
                }
                StripeDirection::Vertical => {
                    StripesPattern::new_vertical(*stripe_width, *color1, *color2)
                }
            };

            // Blend stripes with current value using linear interpolation
            apply_pattern_blended(&stripes, height_map, BlendMode::Lerp, *strength);
        }
        TextureLayer::Checkerboard {
            tile_size,
            color1,
            color2,
            strength,
            ..
        } => {
            let checker = CheckerPattern::new(*tile_size).with_colors(*color1, *color2);

            // Blend checkerboard with current value using linear interpolation
            apply_pattern_blended(&checker, height_map, BlendMode::Lerp, *strength);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::texture::{NoiseAlgorithm, NoiseConfig, TextureMapType};

    fn make_height_map() -> GrayscaleBuffer {
        GrayscaleBuffer::new(32, 32, 0.5)
    }

    #[test]
    fn noise_pattern_strength_zero_is_noop() {
        let mut buf = make_height_map();
        let before = buf.data.clone();

        let layer = TextureLayer::NoisePattern {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Perlin,
                scale: 0.1,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            affects: vec![TextureMapType::Roughness],
            strength: 0.0,
        };

        apply_layer_to_height(&mut buf, &layer, 123);
        assert_eq!(buf.data, before);
    }

    #[test]
    fn gradient_strength_zero_is_noop() {
        let mut buf = make_height_map();
        let before = buf.data.clone();

        let layer = TextureLayer::Gradient {
            direction: GradientDirection::Horizontal,
            start: Some(0.0),
            end: Some(1.0),
            center: None,
            inner: None,
            outer: None,
            affects: vec![TextureMapType::Albedo],
            strength: 0.0,
        };

        apply_layer_to_height(&mut buf, &layer, 0);
        assert_eq!(buf.data, before);
    }

    #[test]
    fn scratches_density_zero_is_noop() {
        let mut buf = make_height_map();
        let before = buf.data.clone();

        let layer = TextureLayer::Scratches {
            density: 0.0,
            length_range: [0.1, 0.2],
            width: 0.01,
            affects: vec![TextureMapType::Roughness],
            strength: 1.0,
        };

        apply_layer_to_height(&mut buf, &layer, 999);
        assert_eq!(buf.data, before);
    }

    #[test]
    fn edge_wear_amount_zero_is_noop() {
        let mut buf = make_height_map();
        let before = buf.data.clone();

        let layer = TextureLayer::EdgeWear {
            amount: 0.0,
            affects: vec![TextureMapType::Roughness],
        };

        apply_layer_to_height(&mut buf, &layer, 999);
        assert_eq!(buf.data, before);
    }

    #[test]
    fn noise_pattern_is_deterministic_for_same_seed() {
        let layer = TextureLayer::NoisePattern {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Perlin,
                scale: 0.05,
                octaves: 3,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            affects: vec![TextureMapType::Height],
            strength: 0.8,
        };

        let mut a = make_height_map();
        let mut b = make_height_map();
        apply_layer_to_height(&mut a, &layer, 42);
        apply_layer_to_height(&mut b, &layer, 42);
        assert_eq!(a.data, b.data);
    }
}
