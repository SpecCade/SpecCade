//! Roughness map generator.

use super::GrayscaleBuffer;
use crate::noise::{Fbm, Noise2D, PerlinNoise};

/// Roughness map generator.
pub struct RoughnessGenerator {
    /// Base roughness value (0.0 = smooth/mirror, 1.0 = rough/diffuse).
    pub base_roughness: f64,
    /// Roughness range [min, max].
    pub roughness_range: [f64; 2],
    /// Noise scale for variation.
    pub noise_scale: f64,
    /// Seed.
    pub seed: u32,
}

impl RoughnessGenerator {
    /// Create a new roughness generator.
    pub fn new(base_roughness: f64, seed: u32) -> Self {
        Self {
            base_roughness,
            roughness_range: [base_roughness * 0.8, base_roughness * 1.2],
            noise_scale: 0.03,
            seed,
        }
    }

    /// Set the roughness range.
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.roughness_range = [min, max];
        self
    }

    /// Set the noise scale.
    pub fn with_noise_scale(mut self, scale: f64) -> Self {
        self.noise_scale = scale;
        self
    }

    /// Generate a flat roughness map.
    pub fn generate_flat(&self, width: u32, height: u32) -> GrayscaleBuffer {
        GrayscaleBuffer::new(width, height, self.base_roughness)
    }

    /// Generate a roughness map with noise-based variation.
    pub fn generate_with_variation(&self, width: u32, height: u32) -> GrayscaleBuffer {
        let mut buffer = GrayscaleBuffer::new(width, height, self.base_roughness);

        let noise = Fbm::new(PerlinNoise::new(self.seed))
            .with_octaves(4)
            .with_persistence(0.5);

        let range = self.roughness_range[1] - self.roughness_range[0];

        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 * self.noise_scale;
                let ny = y as f64 * self.noise_scale;

                let noise_val = noise.sample_01(nx, ny);
                let roughness = self.roughness_range[0] + noise_val * range;

                buffer.set(x, y, roughness.clamp(0.0, 1.0));
            }
        }

        buffer
    }

    /// Generate roughness from a height map (recessed areas accumulate dirt = rougher).
    pub fn generate_from_height(
        &self,
        height_map: &GrayscaleBuffer,
        invert: bool,
    ) -> GrayscaleBuffer {
        let mut buffer =
            GrayscaleBuffer::new(height_map.width, height_map.height, self.base_roughness);

        let range = self.roughness_range[1] - self.roughness_range[0];

        for y in 0..height_map.height {
            for x in 0..height_map.width {
                let h = height_map.get(x, y);
                let t = if invert { 1.0 - h } else { h };
                let roughness = self.roughness_range[0] + t * range;
                buffer.set(x, y, roughness.clamp(0.0, 1.0));
            }
        }

        buffer
    }

    /// Apply scratches to a roughness map (scratches are rougher).
    pub fn apply_scratches(
        &self,
        base: &mut GrayscaleBuffer,
        scratches: &GrayscaleBuffer,
        scratch_roughness: f64,
    ) {
        for y in 0..base.height {
            for x in 0..base.width {
                let scratch_amount = 1.0 - scratches.get(x, y); // Scratches pattern returns lower values for scratches
                let current = base.get(x, y);
                let blended = current + scratch_amount * (scratch_roughness - current);
                base.set(x, y, blended.clamp(0.0, 1.0));
            }
        }
    }

    /// Apply edge wear to roughness (worn edges are shinier/smoother).
    pub fn apply_edge_wear(
        &self,
        base: &mut GrayscaleBuffer,
        edge_wear: &GrayscaleBuffer,
        worn_roughness: f64,
    ) {
        for y in 0..base.height {
            for x in 0..base.width {
                let wear_amount = edge_wear.get(x, y);
                let current = base.get(x, y);
                let blended = current + wear_amount * (worn_roughness - current);
                base.set(x, y, blended.clamp(0.0, 1.0));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roughness_flat() {
        let generator = RoughnessGenerator::new(0.5, 42);
        let buffer = generator.generate_flat(64, 64);

        for y in 0..64 {
            for x in 0..64 {
                assert!((buffer.get(x, y) - 0.5).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_roughness_range() {
        let generator = RoughnessGenerator::new(0.5, 42).with_range(0.3, 0.7);
        let buffer = generator.generate_with_variation(64, 64);

        for y in 0..64 {
            for x in 0..64 {
                let v = buffer.get(x, y);
                assert!((0.3..=0.7).contains(&v), "Value {} out of range", v);
            }
        }
    }

    #[test]
    fn test_roughness_deterministic() {
        let gen1 = RoughnessGenerator::new(0.5, 42);
        let gen2 = RoughnessGenerator::new(0.5, 42);

        let buf1 = gen1.generate_with_variation(64, 64);
        let buf2 = gen2.generate_with_variation(64, 64);

        for y in 0..64 {
            for x in 0..64 {
                assert_eq!(buf1.get(x, y), buf2.get(x, y));
            }
        }
    }
}
