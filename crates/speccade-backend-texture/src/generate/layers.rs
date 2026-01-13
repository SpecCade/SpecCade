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
        TextureLayer::Pitting {
            noise,
            threshold,
            depth,
            ..
        } => {
            let threshold = threshold.clamp(0.0, 1.0);
            let depth = depth.clamp(0.0, 1.0);

            if depth <= 0.0 || threshold >= 1.0 {
                return;
            }

            let noise_gen = create_noise_generator(noise, seed);
            let scale = noise.scale;
            let denom = (1.0 - threshold).max(1e-6);

            apply_transform(height_map, |x, y, current| {
                let nx = x as f64 * scale;
                let ny = y as f64 * scale;
                let noise_val = noise_gen.sample_01(nx, ny);
                if noise_val <= threshold {
                    current
                } else {
                    let t = (noise_val - threshold) / denom;
                    (current - t * depth).clamp(0.0, 1.0)
                }
            });
        }
        TextureLayer::Weave {
            thread_width,
            gap,
            depth,
            ..
        } => {
            let depth = depth.clamp(0.0, 1.0);
            if depth <= 0.0 {
                return;
            }

            apply_transform(height_map, |x, y, current| {
                let pattern_val = sample_weave_pattern(*thread_width, *gap, x, y);
                (current + (pattern_val - 0.5) * depth).clamp(0.0, 1.0)
            });
        }
        _ => {}
    }
}

fn sample_weave_pattern(thread_width: u32, gap: u32, x: u32, y: u32) -> f64 {
    let pattern_size = thread_width.saturating_add(gap).saturating_mul(2);
    if pattern_size == 0 {
        return 0.5;
    }

    let pattern_x = x % pattern_size;
    let pattern_y = y % pattern_size;
    let half_pattern = pattern_size / 2;

    let h_thread = pattern_x < thread_width
        || (pattern_x >= half_pattern && pattern_x < half_pattern + thread_width);
    let v_thread = pattern_y < thread_width
        || (pattern_y >= half_pattern && pattern_y < half_pattern + thread_width);

    let in_gap_x = pattern_x >= thread_width && pattern_x < half_pattern;
    let in_gap_y = pattern_y >= thread_width && pattern_y < half_pattern;
    let in_gap_x2 = pattern_x >= half_pattern + thread_width;
    let in_gap_y2 = pattern_y >= half_pattern + thread_width;

    if (in_gap_x || in_gap_x2) && (in_gap_y || in_gap_y2) {
        return 0.0;
    }

    if h_thread && v_thread && thread_width > 0 {
        let h_pos = if pattern_x < half_pattern {
            pattern_x
        } else {
            pattern_x - half_pattern
        };
        let v_pos = if pattern_y < half_pattern {
            pattern_y
        } else {
            pattern_y - half_pattern
        };

        let h_on_top = (pattern_x < half_pattern) == (pattern_y < half_pattern);

        if h_on_top {
            1.0 - (v_pos as f64 / thread_width as f64) * 0.3
        } else {
            1.0 - (h_pos as f64 / thread_width as f64) * 0.3
        }
    } else if h_thread || v_thread {
        0.75
    } else {
        0.5
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

    #[test]
    fn pitting_threshold_one_is_noop() {
        let mut buf = make_height_map();
        let before = buf.data.clone();

        let layer = TextureLayer::Pitting {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Worley,
                scale: 0.1,
                octaves: 2,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            threshold: 1.0,
            depth: 0.5,
            affects: vec![TextureMapType::Height],
        };

        apply_layer_to_height(&mut buf, &layer, 42);
        assert_eq!(buf.data, before);
    }

    #[test]
    fn weave_depth_zero_is_noop() {
        let mut buf = make_height_map();
        let before = buf.data.clone();

        let layer = TextureLayer::Weave {
            thread_width: 6,
            gap: 2,
            depth: 0.0,
            affects: vec![TextureMapType::Height],
        };

        apply_layer_to_height(&mut buf, &layer, 42);
        assert_eq!(buf.data, before);
    }
}
