//! Metallic map generator.

use super::GrayscaleBuffer;
use crate::noise::{Fbm, Noise2D, PerlinNoise};

/// Metallic map generator.
pub struct MetallicGenerator {
    /// Base metallic value (0.0 = dielectric, 1.0 = metal).
    pub base_metallic: f64,
    /// Variation amount.
    pub variation: f64,
    /// Noise scale.
    pub noise_scale: f64,
    /// Seed.
    pub seed: u32,
}

impl MetallicGenerator {
    /// Create a new metallic generator.
    pub fn new(base_metallic: f64, seed: u32) -> Self {
        Self {
            base_metallic,
            variation: 0.0,
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

    /// Generate a flat metallic map.
    pub fn generate_flat(&self, width: u32, height: u32) -> GrayscaleBuffer {
        GrayscaleBuffer::new(width, height, self.base_metallic)
    }

    /// Generate a metallic map with noise-based variation.
    /// Useful for metals with patina or oxidation.
    pub fn generate_with_variation(&self, width: u32, height: u32) -> GrayscaleBuffer {
        if self.variation < 1e-6 {
            return self.generate_flat(width, height);
        }

        let mut buffer = GrayscaleBuffer::new(width, height, self.base_metallic);

        let noise = Fbm::new(PerlinNoise::new(self.seed))
            .with_octaves(3)
            .with_persistence(0.5);

        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 * self.noise_scale;
                let ny = y as f64 * self.noise_scale;

                let noise_val = noise.sample(nx, ny);
                let metallic = self.base_metallic + noise_val * self.variation;

                buffer.set(x, y, metallic.clamp(0.0, 1.0));
            }
        }

        buffer
    }

    /// Generate metallic map from a pattern (e.g., metal parts vs non-metal).
    pub fn generate_from_pattern(
        &self,
        pattern: &GrayscaleBuffer,
        metal_value: f64,
        non_metal_value: f64,
    ) -> GrayscaleBuffer {
        let mut buffer = GrayscaleBuffer::new(pattern.width, pattern.height, 0.0);

        for y in 0..pattern.height {
            for x in 0..pattern.width {
                let t = pattern.get(x, y);
                let metallic = non_metal_value + t * (metal_value - non_metal_value);
                buffer.set(x, y, metallic);
            }
        }

        buffer
    }

    /// Apply oxidation/patina to a metallic map.
    pub fn apply_oxidation(
        &self,
        base: &mut GrayscaleBuffer,
        oxidation_map: &GrayscaleBuffer,
        oxidation_factor: f64,
    ) {
        for y in 0..base.height {
            for x in 0..base.width {
                let oxidation = oxidation_map.get(x, y) * oxidation_factor;
                let current = base.get(x, y);
                // Oxidation reduces metallicity
                let new_value = current * (1.0 - oxidation);
                base.set(x, y, new_value.clamp(0.0, 1.0));
            }
        }
    }

    /// Apply edge wear to metallic (worn edges expose more metal).
    pub fn apply_edge_wear(
        &self,
        base: &mut GrayscaleBuffer,
        edge_wear: &GrayscaleBuffer,
        exposed_metallic: f64,
    ) {
        for y in 0..base.height {
            for x in 0..base.width {
                let wear = edge_wear.get(x, y);
                let current = base.get(x, y);
                // Worn areas expose underlying metal
                let new_value = current + wear * (exposed_metallic - current);
                base.set(x, y, new_value.clamp(0.0, 1.0));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metallic_flat() {
        let generator = MetallicGenerator::new(1.0, 42);
        let buffer = generator.generate_flat(64, 64);

        for y in 0..64 {
            for x in 0..64 {
                assert!((buffer.get(x, y) - 1.0).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_metallic_with_variation() {
        let generator = MetallicGenerator::new(0.8, 42).with_variation(0.2);
        let buffer = generator.generate_with_variation(64, 64);

        // Check that values are within expected range
        for y in 0..64 {
            for x in 0..64 {
                let v = buffer.get(x, y);
                assert!((0.0..=1.0).contains(&v));
            }
        }
    }

    #[test]
    fn test_metallic_deterministic() {
        let gen1 = MetallicGenerator::new(0.8, 42).with_variation(0.1);
        let gen2 = MetallicGenerator::new(0.8, 42).with_variation(0.1);

        let buf1 = gen1.generate_with_variation(64, 64);
        let buf2 = gen2.generate_with_variation(64, 64);

        for y in 0..64 {
            for x in 0..64 {
                assert_eq!(buf1.get(x, y), buf2.get(x, y));
            }
        }
    }
}
