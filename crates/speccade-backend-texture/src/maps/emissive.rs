//! Emissive map generator.

use super::{TextureBuffer, GrayscaleBuffer};
use crate::color::Color;
use crate::noise::{Noise2D, Fbm, PerlinNoise};

/// Emissive map generator.
pub struct EmissiveGenerator {
    /// Emissive color.
    pub color: Color,
    /// Emission intensity (multiplier).
    pub intensity: f64,
    /// Seed for variation.
    pub seed: u32,
}

impl EmissiveGenerator {
    /// Create a new emissive generator.
    pub fn new(color: Color, seed: u32) -> Self {
        Self {
            color,
            intensity: 1.0,
            seed,
        }
    }

    /// Set the intensity.
    pub fn with_intensity(mut self, intensity: f64) -> Self {
        self.intensity = intensity;
        self
    }

    /// Generate an emissive map with no emission (black).
    pub fn generate_none(&self, width: u32, height: u32) -> TextureBuffer {
        TextureBuffer::new(width, height, Color::black())
    }

    /// Generate a uniform emissive map.
    pub fn generate_uniform(&self, width: u32, height: u32) -> TextureBuffer {
        let color = Color::rgb(
            self.color.r * self.intensity,
            self.color.g * self.intensity,
            self.color.b * self.intensity,
        );
        TextureBuffer::new(width, height, color)
    }

    /// Generate emissive from a mask (where mask > 0, emit light).
    pub fn generate_from_mask(
        &self,
        mask: &GrayscaleBuffer,
        threshold: f64,
    ) -> TextureBuffer {
        let mut buffer = TextureBuffer::new(mask.width, mask.height, Color::black());

        for y in 0..mask.height {
            for x in 0..mask.width {
                let m = mask.get(x, y);
                if m > threshold {
                    let strength = ((m - threshold) / (1.0 - threshold)) * self.intensity;
                    let color = Color::rgb(
                        self.color.r * strength,
                        self.color.g * strength,
                        self.color.b * strength,
                    );
                    buffer.set(x, y, color);
                }
            }
        }

        buffer
    }

    /// Generate emissive with pulsing/flickering effect baked in.
    /// This creates variation across the texture, not animation.
    pub fn generate_with_variation(&self, width: u32, height: u32) -> TextureBuffer {
        let mut buffer = TextureBuffer::new(width, height, Color::black());

        let noise = Fbm::new(PerlinNoise::new(self.seed))
            .with_octaves(3)
            .with_persistence(0.5);

        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 * 0.05;
                let ny = y as f64 * 0.05;

                let noise_val = noise.sample_01(nx, ny);
                let strength = noise_val * self.intensity;

                let color = Color::rgb(
                    self.color.r * strength,
                    self.color.g * strength,
                    self.color.b * strength,
                );
                buffer.set(x, y, color);
            }
        }

        buffer
    }

    /// Generate emissive for hot metal/glowing edges effect.
    pub fn generate_hot_glow(
        &self,
        heat_map: &GrayscaleBuffer,
        cold_color: Color,
        hot_color: Color,
    ) -> TextureBuffer {
        let mut buffer = TextureBuffer::new(heat_map.width, heat_map.height, Color::black());

        for y in 0..heat_map.height {
            for x in 0..heat_map.width {
                let heat = heat_map.get(x, y);

                if heat > 0.01 {
                    // Interpolate between cold and hot color based on heat
                    let color = cold_color.lerp(&hot_color, heat);
                    let emissive = Color::rgb(
                        color.r * heat * self.intensity,
                        color.g * heat * self.intensity,
                        color.b * heat * self.intensity,
                    );
                    buffer.set(x, y, emissive);
                }
            }
        }

        buffer
    }

    /// Generate emissive for glowing lines/circuits.
    pub fn generate_circuit_glow(
        &self,
        circuit_mask: &GrayscaleBuffer,
        glow_radius: u32,
    ) -> TextureBuffer {
        let width = circuit_mask.width;
        let height = circuit_mask.height;
        let mut buffer = TextureBuffer::new(width, height, Color::black());

        // First pass: identify circuit pixels
        // Second pass: add glow around them
        for y in 0..height {
            for x in 0..width {
                let center_value = circuit_mask.get(x, y);

                if center_value > 0.5 {
                    // This is a circuit pixel - emit at full strength
                    let color = Color::rgb(
                        self.color.r * self.intensity,
                        self.color.g * self.intensity,
                        self.color.b * self.intensity,
                    );
                    buffer.set(x, y, color);
                } else {
                    // Check for nearby circuit pixels for glow
                    let mut max_glow: f64 = 0.0;

                    for dy in -(glow_radius as i32)..=(glow_radius as i32) {
                        for dx in -(glow_radius as i32)..=(glow_radius as i32) {
                            let sx = (x as i32 + dx).rem_euclid(width as i32) as u32;
                            let sy = (y as i32 + dy).rem_euclid(height as i32) as u32;

                            if circuit_mask.get(sx, sy) > 0.5 {
                                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                                if dist <= glow_radius as f64 {
                                    let falloff = 1.0 - dist / glow_radius as f64;
                                    max_glow = max_glow.max(falloff);
                                }
                            }
                        }
                    }

                    if max_glow > 0.0 {
                        let glow_intensity = max_glow * max_glow * self.intensity * 0.5;
                        let color = Color::rgb(
                            self.color.r * glow_intensity,
                            self.color.g * glow_intensity,
                            self.color.b * glow_intensity,
                        );
                        buffer.set(x, y, color);
                    }
                }
            }
        }

        buffer
    }
}

/// Generate a simple emissive map based on a pattern.
pub fn generate_emissive_from_pattern(
    pattern: &GrayscaleBuffer,
    color: Color,
    intensity: f64,
    threshold: f64,
) -> TextureBuffer {
    EmissiveGenerator::new(color, 42)
        .with_intensity(intensity)
        .generate_from_mask(pattern, threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emissive_none() {
        let generator = EmissiveGenerator::new(Color::rgb(1.0, 0.5, 0.0), 42);
        let buffer = generator.generate_none(64, 64);

        for y in 0..64 {
            for x in 0..64 {
                let c = buffer.get(x, y);
                assert!((c.r).abs() < 1e-10);
                assert!((c.g).abs() < 1e-10);
                assert!((c.b).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_emissive_uniform() {
        let color = Color::rgb(1.0, 0.5, 0.0);
        let generator = EmissiveGenerator::new(color, 42).with_intensity(2.0);
        let buffer = generator.generate_uniform(64, 64);

        for y in 0..64 {
            for x in 0..64 {
                let c = buffer.get(x, y);
                assert!((c.r - 2.0).abs() < 1e-10);
                assert!((c.g - 1.0).abs() < 1e-10);
                assert!((c.b).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_emissive_from_mask() {
        let mut mask = GrayscaleBuffer::new(64, 64, 0.0);
        // Set some pixels above threshold
        for y in 20..40 {
            for x in 20..40 {
                mask.set(x, y, 1.0);
            }
        }

        let color = Color::rgb(1.0, 0.0, 0.0);
        let generator = EmissiveGenerator::new(color, 42);
        let buffer = generator.generate_from_mask(&mask, 0.5);

        // Outside masked area should be black
        let outside = buffer.get(10, 10);
        assert!((outside.r).abs() < 1e-10);

        // Inside masked area should emit
        let inside = buffer.get(30, 30);
        assert!(inside.r > 0.0);
    }

    #[test]
    fn test_emissive_deterministic() {
        let color = Color::rgb(1.0, 0.5, 0.0);
        let gen1 = EmissiveGenerator::new(color, 42);
        let gen2 = EmissiveGenerator::new(color, 42);

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
