//! Ambient Occlusion (AO) map generator.

use super::GrayscaleBuffer;

/// AO map generator.
pub struct AoGenerator {
    /// AO strength (0.0 = no AO, 1.0 = full AO).
    pub strength: f64,
    /// Sample radius in pixels.
    pub radius: u32,
    /// Number of samples for quality.
    pub samples: u32,
}

impl AoGenerator {
    /// Create a new AO generator.
    pub fn new() -> Self {
        Self {
            strength: 1.0,
            radius: 4,
            samples: 8,
        }
    }

    /// Set the AO strength.
    pub fn with_strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }

    /// Set the sample radius.
    pub fn with_radius(mut self, radius: u32) -> Self {
        self.radius = radius;
        self
    }

    /// Set the number of samples.
    pub fn with_samples(mut self, samples: u32) -> Self {
        self.samples = samples;
        self
    }

    /// Generate AO from a height map.
    ///
    /// This uses a simple horizon-based approach where lower areas
    /// surrounded by higher areas get darker.
    pub fn generate_from_height(&self, height_map: &GrayscaleBuffer) -> GrayscaleBuffer {
        let width = height_map.width;
        let height = height_map.height;
        let mut buffer = GrayscaleBuffer::new(width, height, 1.0);

        for y in 0..height {
            for x in 0..width {
                let ao = self.calculate_ao(height_map, x as i32, y as i32);
                buffer.set(x, y, ao);
            }
        }

        buffer
    }

    /// Calculate AO at a specific pixel.
    fn calculate_ao(&self, height_map: &GrayscaleBuffer, x: i32, y: i32) -> f64 {
        let center_height = height_map.get_wrapped(x, y);

        let mut occlusion = 0.0;
        let mut sample_count = 0;

        // Sample in a circular pattern
        for i in 0..self.samples {
            let angle = (i as f64 / self.samples as f64) * std::f64::consts::PI * 2.0;

            for r in 1..=self.radius {
                let dx = (angle.cos() * r as f64).round() as i32;
                let dy = (angle.sin() * r as f64).round() as i32;

                let sample_height = height_map.get_wrapped(x + dx, y + dy);

                // If the sample is higher than center, it occludes
                if sample_height > center_height {
                    let height_diff = sample_height - center_height;
                    let distance = r as f64;
                    // Closer samples and larger height differences contribute more
                    let contribution = height_diff / (distance * 0.5 + 1.0);
                    occlusion += contribution;
                }

                sample_count += 1;
            }
        }

        // Normalize occlusion
        let max_occlusion = sample_count as f64;
        let normalized_occlusion = (occlusion / max_occlusion).clamp(0.0, 1.0);

        // Apply strength and return (1.0 = no occlusion, 0.0 = full occlusion)
        1.0 - normalized_occlusion * self.strength
    }

    /// Generate AO from a pattern (e.g., crevices are darker).
    pub fn generate_from_pattern(
        &self,
        pattern: &GrayscaleBuffer,
        invert: bool,
    ) -> GrayscaleBuffer {
        let mut height_map = pattern.clone();

        if invert {
            for i in 0..height_map.data.len() {
                height_map.data[i] = 1.0 - height_map.data[i];
            }
        }

        self.generate_from_height(&height_map)
    }

    /// Generate a simple cavity-based AO (concave areas are darker).
    pub fn generate_cavity_ao(&self, height_map: &GrayscaleBuffer) -> GrayscaleBuffer {
        let width = height_map.width;
        let height = height_map.height;
        let mut buffer = GrayscaleBuffer::new(width, height, 1.0);

        for y in 0..height {
            for x in 0..width {
                let center = height_map.get(x, y);

                // Sample neighbors
                let mut neighbor_sum = 0.0;
                let mut count = 0;

                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        neighbor_sum += height_map.get_wrapped(x as i32 + dx, y as i32 + dy);
                        count += 1;
                    }
                }

                let neighbor_avg = neighbor_sum / count as f64;

                // If center is lower than average neighbors, it's a cavity
                let cavity = (neighbor_avg - center).max(0.0);

                // Convert cavity to AO
                let ao = 1.0 - (cavity * self.strength * 2.0).clamp(0.0, 1.0);

                buffer.set(x, y, ao);
            }
        }

        buffer
    }

    /// Combine two AO maps by multiplication.
    pub fn combine_multiply(&self, a: &GrayscaleBuffer, b: &GrayscaleBuffer) -> GrayscaleBuffer {
        let width = a.width.min(b.width);
        let height = a.height.min(b.height);
        let mut result = GrayscaleBuffer::new(width, height, 1.0);

        for y in 0..height {
            for x in 0..width {
                let ao = a.get(x, y) * b.get(x, y);
                result.set(x, y, ao);
            }
        }

        result
    }
}

impl Default for AoGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate AO map with sensible defaults.
pub fn generate_ao_from_height(
    height_map: &GrayscaleBuffer,
    strength: f64,
) -> GrayscaleBuffer {
    AoGenerator::new()
        .with_strength(strength)
        .generate_from_height(height_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ao_flat() {
        // Flat height map should produce uniform AO
        let height_map = GrayscaleBuffer::new(64, 64, 0.5);
        let generator = AoGenerator::new();
        let ao_map = generator.generate_from_height(&height_map);

        // Check that AO is relatively uniform
        let first = ao_map.get(0, 0);
        for y in 0..64 {
            for x in 0..64 {
                let v = ao_map.get(x, y);
                // Should be very close to 1.0 for flat surface
                assert!(v >= 0.9, "Flat surface should have minimal AO");
            }
        }
    }

    #[test]
    fn test_ao_cavity() {
        // Create a height map with a pit in the center
        let mut height_map = GrayscaleBuffer::new(64, 64, 1.0);

        // Create a pit at center
        for y in 28..36 {
            for x in 28..36 {
                height_map.set(x, y, 0.0);
            }
        }

        let generator = AoGenerator::new().with_strength(1.0);
        let ao_map = generator.generate_from_height(&height_map);

        // Center (pit) should have more occlusion than edges
        let center_ao = ao_map.get(32, 32);
        let edge_ao = ao_map.get(0, 0);

        assert!(center_ao < edge_ao, "Pit should have more occlusion than flat area");
    }

    #[test]
    fn test_ao_deterministic() {
        let height_map = GrayscaleBuffer::new(64, 64, 0.5);

        let gen1 = AoGenerator::new();
        let gen2 = AoGenerator::new();

        let buf1 = gen1.generate_from_height(&height_map);
        let buf2 = gen2.generate_from_height(&height_map);

        for y in 0..64 {
            for x in 0..64 {
                assert_eq!(buf1.get(x, y), buf2.get(x, y));
            }
        }
    }
}
