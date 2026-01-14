//! Mask generation utilities for texture layers.
//!
//! Provides functions to build threshold-based and streak-based masks
//! from noise patterns.

use speccade_spec::recipe::texture::{NoiseConfig, StripeDirection};

use crate::maps::GrayscaleBuffer;

use super::helpers::create_noise_generator;

/// Build a threshold-based mask from noise.
///
/// Pixels with noise values above the threshold are included in the mask,
/// with the mask strength determined by how far above the threshold they are.
pub fn build_threshold_mask(
    width: u32,
    height: u32,
    noise: &NoiseConfig,
    seed: u32,
    threshold: f64,
    strength: f64,
) -> GrayscaleBuffer {
    let threshold = threshold.clamp(0.0, 1.0);
    let strength = strength.clamp(0.0, 1.0);
    if strength <= 0.0 || threshold >= 1.0 {
        return GrayscaleBuffer::new(width, height, 0.0);
    }

    let noise_gen = create_noise_generator(noise, seed);
    let scale = noise.scale;
    let denom = (1.0 - threshold).max(1e-6);
    let mut mask = GrayscaleBuffer::new(width, height, 0.0);

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 * scale;
            let ny = y as f64 * scale;
            let noise_val = noise_gen.sample_01(nx, ny);
            if noise_val <= threshold {
                continue;
            }
            let t = (noise_val - threshold) / denom;
            mask.set(x, y, (t * strength).clamp(0.0, 1.0));
        }
    }

    mask
}

/// Build a directional streak mask from noise.
///
/// Creates streaks in horizontal or vertical direction, with strength
/// decreasing along the perpendicular direction.
pub fn build_streak_mask(
    width: u32,
    height: u32,
    noise: &NoiseConfig,
    seed: u32,
    threshold: f64,
    strength: f64,
    direction: StripeDirection,
) -> GrayscaleBuffer {
    let threshold = threshold.clamp(0.0, 1.0);
    let strength = strength.clamp(0.0, 1.0);
    if strength <= 0.0 || threshold >= 1.0 {
        return GrayscaleBuffer::new(width, height, 0.0);
    }

    let noise_gen = create_noise_generator(noise, seed);
    let scale = noise.scale;
    let denom = (1.0 - threshold).max(1e-6);
    let width_denom = width.saturating_sub(1).max(1) as f64;
    let height_denom = height.saturating_sub(1).max(1) as f64;

    let mut mask = GrayscaleBuffer::new(width, height, 0.0);

    for y in 0..height {
        for x in 0..width {
            let line_coord = match direction {
                StripeDirection::Vertical => x,
                StripeDirection::Horizontal => y,
            };
            let line_sample = noise_gen.sample_01(line_coord as f64 * scale, 0.0);
            if line_sample <= threshold {
                continue;
            }
            let line_strength = (line_sample - threshold) / denom;
            let nx = x as f64 * scale;
            let ny = y as f64 * scale;
            let variation = noise_gen.sample_01(nx, ny);
            let dir_factor = match direction {
                StripeDirection::Vertical => 1.0 - (y as f64 / height_denom),
                StripeDirection::Horizontal => 1.0 - (x as f64 / width_denom),
            };
            let t = line_strength * dir_factor * (0.5 + 0.5 * variation);
            mask.set(x, y, (t * strength).clamp(0.0, 1.0));
        }
    }

    mask
}
