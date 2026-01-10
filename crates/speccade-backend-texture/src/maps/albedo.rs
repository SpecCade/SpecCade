//! Albedo (base color) map generator.

use super::{GrayscaleBuffer, TextureBuffer};
use crate::color::{BlendMode, Color};
use crate::noise::{Fbm, Noise2D, PerlinNoise};

/// Albedo map generator.
pub struct AlbedoGenerator {
    /// Base color.
    pub base_color: Color,
    /// Color variation amount.
    pub variation: f64,
    /// Noise scale for variation.
    pub noise_scale: f64,
    /// Seed.
    pub seed: u32,
}

impl AlbedoGenerator {
    /// Create a new albedo generator.
    pub fn new(base_color: Color, seed: u32) -> Self {
        Self {
            base_color,
            variation: 0.1,
            noise_scale: 0.02,
            seed,
        }
    }

    /// Set the variation amount.
    pub fn with_variation(mut self, variation: f64) -> Self {
        self.variation = variation;
        self
    }

    /// Set the noise scale.
    pub fn with_noise_scale(mut self, scale: f64) -> Self {
        self.noise_scale = scale;
        self
    }

    /// Generate a flat color albedo map.
    pub fn generate_flat(&self, width: u32, height: u32) -> TextureBuffer {
        TextureBuffer::new(width, height, self.base_color)
    }

    /// Generate an albedo map with noise-based color variation.
    pub fn generate_with_variation(&self, width: u32, height: u32) -> TextureBuffer {
        let mut buffer = TextureBuffer::new(width, height, self.base_color);

        let noise = Fbm::new(PerlinNoise::new(self.seed))
            .with_octaves(4)
            .with_persistence(0.5);

        let (h, s, v) = self.base_color.to_hsv();

        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 * self.noise_scale;
                let ny = y as f64 * self.noise_scale;

                // Get noise values for each HSV component
                let h_noise = noise.sample(nx, ny) * self.variation * 30.0; // Hue variation in degrees
                let s_noise = noise.sample(nx + 100.0, ny) * self.variation;
                let v_noise = noise.sample(nx, ny + 100.0) * self.variation;

                let new_h = (h + h_noise) % 360.0;
                let new_s = (s + s_noise).clamp(0.0, 1.0);
                let new_v = (v + v_noise).clamp(0.0, 1.0);

                let color = Color::from_hsv(new_h, new_s, new_v);
                buffer.set(
                    x,
                    y,
                    Color::rgba(color.r, color.g, color.b, self.base_color.a),
                );
            }
        }

        buffer
    }

    /// Generate an albedo map from a height map (for materials where color varies with height).
    pub fn generate_from_height(
        &self,
        height_map: &GrayscaleBuffer,
        low_color: Color,
        high_color: Color,
    ) -> TextureBuffer {
        let mut buffer = TextureBuffer::new(height_map.width, height_map.height, Color::black());

        for y in 0..height_map.height {
            for x in 0..height_map.width {
                let h = height_map.get(x, y);
                let color = low_color.lerp(&high_color, h);
                buffer.set(x, y, color);
            }
        }

        buffer
    }

    /// Apply dirt/grime overlay to an existing albedo map.
    pub fn apply_dirt(&self, base: &mut TextureBuffer, density: f64, dirt_color: Color, seed: u32) {
        let noise = Fbm::new(PerlinNoise::new(seed))
            .with_octaves(4)
            .with_persistence(0.6);

        for y in 0..base.height {
            for x in 0..base.width {
                let nx = x as f64 * 0.01;
                let ny = y as f64 * 0.01;

                let dirt_amount = noise.sample_01(nx, ny);

                if dirt_amount > (1.0 - density) {
                    let strength = (dirt_amount - (1.0 - density)) / density;
                    let current = base.get(x, y);
                    let blended = BlendMode::Multiply.blend(&current, &dirt_color, strength);
                    base.set(x, y, blended);
                }
            }
        }
    }

    /// Apply color variation based on a pattern/mask.
    pub fn apply_pattern_color(
        &self,
        base: &mut TextureBuffer,
        pattern: &GrayscaleBuffer,
        pattern_color: Color,
        blend_mode: BlendMode,
    ) {
        for y in 0..base.height {
            for x in 0..base.width {
                let pattern_value = pattern.get(x, y);
                let current = base.get(x, y);
                let blended = blend_mode.blend(&current, &pattern_color, pattern_value);
                base.set(x, y, blended);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_albedo_flat() {
        let color = Color::rgb(0.8, 0.2, 0.1);
        let generator = AlbedoGenerator::new(color, 42);
        let buffer = generator.generate_flat(64, 64);

        assert_eq!(buffer.width, 64);
        assert_eq!(buffer.height, 64);

        // All pixels should be the same color
        for y in 0..64 {
            for x in 0..64 {
                let c = buffer.get(x, y);
                assert!((c.r - 0.8).abs() < 1e-10);
                assert!((c.g - 0.2).abs() < 1e-10);
                assert!((c.b - 0.1).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_albedo_with_variation() {
        let color = Color::rgb(0.5, 0.5, 0.5);
        let generator = AlbedoGenerator::new(color, 42).with_variation(0.2);
        let buffer = generator.generate_with_variation(64, 64);

        // Check for variation (not all pixels the same)
        let first = buffer.get(0, 0);
        let mut has_variation = false;

        for y in 0..64 {
            for x in 0..64 {
                let c = buffer.get(x, y);
                if (c.r - first.r).abs() > 0.01
                    || (c.g - first.g).abs() > 0.01
                    || (c.b - first.b).abs() > 0.01
                {
                    has_variation = true;
                    break;
                }
            }
        }

        assert!(has_variation, "Albedo map should have variation");
    }

    #[test]
    fn test_albedo_deterministic() {
        let color = Color::rgb(0.5, 0.5, 0.5);
        let gen1 = AlbedoGenerator::new(color, 42);
        let gen2 = AlbedoGenerator::new(color, 42);

        let buf1 = gen1.generate_with_variation(64, 64);
        let buf2 = gen2.generate_with_variation(64, 64);

        for y in 0..64 {
            for x in 0..64 {
                let c1 = buf1.get(x, y);
                let c2 = buf2.get(x, y);
                assert_eq!(c1.r, c2.r);
                assert_eq!(c1.g, c2.g);
                assert_eq!(c1.b, c2.b);
            }
        }
    }
}
