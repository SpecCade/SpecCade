//! Shared utilities for texture generation.
//!
//! This module contains common functions used by multiple texture generation
//! modules to avoid code duplication.

use speccade_spec::recipe::texture::{NoiseConfig, NoiseAlgorithm};

use crate::maps::GrayscaleBuffer;
use crate::noise::{Noise2D, Fbm, PerlinNoise, SimplexNoise, WorleyNoise};
use crate::pattern::Pattern2D;

/// Create a noise generator from configuration.
///
/// This function creates the appropriate noise generator based on the algorithm
/// specified in the config. If octaves > 1 and the algorithm is not FBM, the
/// noise will be wrapped in an FBM generator for multi-octave sampling.
///
/// # Arguments
///
/// * `config` - The noise configuration specifying algorithm and parameters
/// * `seed` - The seed for deterministic noise generation
///
/// # Returns
///
/// A boxed trait object implementing `Noise2D`
pub fn create_noise_generator(config: &NoiseConfig, seed: u32) -> Box<dyn Noise2D> {
    let base_noise: Box<dyn Noise2D> = match config.algorithm {
        NoiseAlgorithm::Perlin => Box::new(PerlinNoise::new(seed)),
        NoiseAlgorithm::Simplex => Box::new(SimplexNoise::new(seed)),
        NoiseAlgorithm::Worley => Box::new(WorleyNoise::new(seed)),
        NoiseAlgorithm::Value => Box::new(PerlinNoise::new(seed)), // Use Perlin as fallback
        NoiseAlgorithm::Fbm => {
            Box::new(
                Fbm::new(PerlinNoise::new(seed))
                    .with_octaves(config.octaves)
                    .with_persistence(config.persistence)
                    .with_lacunarity(config.lacunarity)
            )
        }
    };

    // Wrap in FBM if octaves > 1 and not already FBM
    if config.octaves > 1 && config.algorithm != NoiseAlgorithm::Fbm {
        match config.algorithm {
            NoiseAlgorithm::Perlin => {
                Box::new(
                    Fbm::new(PerlinNoise::new(seed))
                        .with_octaves(config.octaves)
                        .with_persistence(config.persistence)
                        .with_lacunarity(config.lacunarity)
                )
            }
            NoiseAlgorithm::Simplex => {
                Box::new(
                    Fbm::new(SimplexNoise::new(seed))
                        .with_octaves(config.octaves)
                        .with_persistence(config.persistence)
                        .with_lacunarity(config.lacunarity)
                )
            }
            _ => base_noise,
        }
    } else {
        base_noise
    }
}

// ============================================================================
// Pattern Application Helpers
// ============================================================================

/// Apply a pattern directly to a grayscale buffer, replacing all values.
///
/// This iterates over every pixel in the buffer and sets it to the pattern's
/// sampled value at that coordinate. This eliminates duplicated nested loops
/// throughout the codebase.
///
/// # Arguments
/// * `pattern` - The pattern to sample from (must implement `Pattern2D`)
/// * `buffer` - The grayscale buffer to write to
///
/// # Example
///
/// ```ignore
/// let pattern = BrickPattern::new(width, height);
/// let mut buffer = GrayscaleBuffer::new(width, height, 0.0);
/// sample_pattern_to_buffer(&pattern, &mut buffer);
/// ```
pub fn sample_pattern_to_buffer<P: Pattern2D>(pattern: &P, buffer: &mut GrayscaleBuffer) {
    for y in 0..buffer.height {
        for x in 0..buffer.width {
            buffer.set(x, y, pattern.sample(x, y));
        }
    }
}

/// Blend modes for combining pattern values with existing buffer values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternBlendMode {
    /// Replace the existing value entirely: `result = pattern`
    Replace,
    /// Blend with linear interpolation: `result = current * (1 - strength) + pattern * strength`
    Lerp,
    /// Add pattern offset: `result = current + (pattern - 0.5) * strength`
    Offset,
    /// Take minimum of current and pattern value: `result = min(current, pattern)`
    Min,
    /// Multiply current by pattern-based factor: `result = current * (1 - pattern * strength)`
    MultiplyInverse,
}

/// Apply a pattern to a grayscale buffer with blending.
///
/// This iterates over every pixel in the buffer and blends the pattern's
/// sampled value with the existing value according to the blend mode.
///
/// # Arguments
/// * `pattern` - The pattern to sample from (must implement `Pattern2D`)
/// * `buffer` - The grayscale buffer to read from and write to
/// * `blend_mode` - How to combine pattern values with existing values
/// * `strength` - Blend strength/opacity (0.0 to 1.0, interpretation depends on blend mode)
pub fn sample_pattern_blended<P: Pattern2D>(
    pattern: &P,
    buffer: &mut GrayscaleBuffer,
    blend_mode: PatternBlendMode,
    strength: f64,
) {
    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let pattern_val = pattern.sample(x, y);
            let current = buffer.get(x, y);
            let new_val = match blend_mode {
                PatternBlendMode::Replace => pattern_val,
                PatternBlendMode::Lerp => current * (1.0 - strength) + pattern_val * strength,
                PatternBlendMode::Offset => current + (pattern_val - 0.5) * strength,
                PatternBlendMode::Min => current.min(pattern_val),
                PatternBlendMode::MultiplyInverse => current * (1.0 - pattern_val * strength),
            };
            buffer.set(x, y, new_val.clamp(0.0, 1.0));
        }
    }
}

/// Apply a custom transformation to each pixel in a grayscale buffer.
///
/// This is useful when the pattern transformation is more complex than the
/// standard blend modes, such as when coordinates need to be transformed
/// before sampling.
///
/// # Arguments
/// * `buffer` - The grayscale buffer to read from and write to
/// * `transform` - A closure that takes (x, y, current_value) and returns the new value
pub fn apply_buffer_transform<F>(buffer: &mut GrayscaleBuffer, transform: F)
where
    F: Fn(u32, u32, f64) -> f64,
{
    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let current = buffer.get(x, y);
            let new_val = transform(x, y, current);
            buffer.set(x, y, new_val.clamp(0.0, 1.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_noise_generator_perlin() {
        let config = NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.1,
            octaves: 1,
            persistence: 0.5,
            lacunarity: 2.0,
        };

        let noise = create_noise_generator(&config, 42);
        let val = noise.sample_01(0.5, 0.5);
        assert!(val >= 0.0 && val <= 1.0);
    }

    #[test]
    fn test_create_noise_generator_simplex() {
        let config = NoiseConfig {
            algorithm: NoiseAlgorithm::Simplex,
            scale: 0.1,
            octaves: 1,
            persistence: 0.5,
            lacunarity: 2.0,
        };

        let noise = create_noise_generator(&config, 42);
        let val = noise.sample_01(0.5, 0.5);
        assert!(val >= 0.0 && val <= 1.0);
    }

    #[test]
    fn test_create_noise_generator_worley() {
        let config = NoiseConfig {
            algorithm: NoiseAlgorithm::Worley,
            scale: 0.1,
            octaves: 1,
            persistence: 0.5,
            lacunarity: 2.0,
        };

        let noise = create_noise_generator(&config, 42);
        let val = noise.sample_01(0.5, 0.5);
        assert!(val >= 0.0 && val <= 1.0);
    }

    #[test]
    fn test_create_noise_generator_fbm() {
        let config = NoiseConfig {
            algorithm: NoiseAlgorithm::Fbm,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        };

        let noise = create_noise_generator(&config, 42);
        let val = noise.sample_01(0.5, 0.5);
        assert!(val >= 0.0 && val <= 1.0);
    }

    #[test]
    fn test_create_noise_generator_multi_octave_perlin() {
        let config = NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        };

        let noise = create_noise_generator(&config, 42);
        let val = noise.sample_01(0.5, 0.5);
        assert!(val >= 0.0 && val <= 1.0);
    }

    #[test]
    fn test_create_noise_generator_multi_octave_simplex() {
        let config = NoiseConfig {
            algorithm: NoiseAlgorithm::Simplex,
            scale: 0.1,
            octaves: 6,
            persistence: 0.6,
            lacunarity: 2.2,
        };

        let noise = create_noise_generator(&config, 42);
        let val = noise.sample_01(0.5, 0.5);
        assert!(val >= 0.0 && val <= 1.0);
    }

    #[test]
    fn test_create_noise_generator_deterministic() {
        let config = NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        };

        let noise1 = create_noise_generator(&config, 42);
        let noise2 = create_noise_generator(&config, 42);

        // Same seed should produce same values
        assert_eq!(noise1.sample_01(0.5, 0.5), noise2.sample_01(0.5, 0.5));
        assert_eq!(noise1.sample_01(1.0, 2.0), noise2.sample_01(1.0, 2.0));
    }

    #[test]
    fn test_create_noise_generator_different_seeds() {
        let config = NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        };

        let noise1 = create_noise_generator(&config, 42);
        let noise2 = create_noise_generator(&config, 100);

        // Different seeds should produce different values
        assert_ne!(noise1.sample_01(0.5, 0.5), noise2.sample_01(0.5, 0.5));
    }

    #[test]
    fn test_sample_pattern_to_buffer() {
        use crate::pattern::CheckerPattern;

        let pattern = CheckerPattern::new(8);
        let mut buffer = GrayscaleBuffer::new(64, 64, 0.0);

        sample_pattern_to_buffer(&pattern, &mut buffer);

        // Checker pattern should produce 0.0 and 1.0 values
        let val_00 = buffer.get(0, 0);
        let val_08 = buffer.get(8, 0);

        // Adjacent checker cells should have different values
        assert!((val_00 - val_08).abs() > 0.5, "Checker pattern should alternate");
    }

    #[test]
    fn test_sample_pattern_to_buffer_deterministic() {
        use crate::pattern::CheckerPattern;

        let pattern = CheckerPattern::new(4);

        let mut buffer1 = GrayscaleBuffer::new(32, 32, 0.0);
        let mut buffer2 = GrayscaleBuffer::new(32, 32, 0.0);

        sample_pattern_to_buffer(&pattern, &mut buffer1);
        sample_pattern_to_buffer(&pattern, &mut buffer2);

        // Same pattern should produce identical results
        assert_eq!(buffer1.data, buffer2.data);
    }
}
