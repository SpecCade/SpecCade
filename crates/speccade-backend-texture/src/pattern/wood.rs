//! Wood grain pattern generator.

use super::Pattern2D;
use crate::noise::{PerlinNoise, Noise2D, Fbm};

/// Wood grain pattern configuration.
#[derive(Clone)]
pub struct WoodGrainPattern {
    /// Number of rings.
    pub ring_count: u32,
    /// Ring distortion amount.
    pub distortion: f64,
    /// Turbulence amount.
    pub turbulence: f64,
    /// Scale of the distortion noise.
    pub noise_scale: f64,
    /// Seed for random elements.
    pub seed: u32,
    /// Total width.
    width: u32,
    /// Total height.
    height: u32,
    /// Cached noise generator.
    noise: Fbm<PerlinNoise>,
}

impl WoodGrainPattern {
    /// Create a new wood grain pattern.
    pub fn new(width: u32, height: u32, seed: u32) -> Self {
        let noise = Fbm::new(PerlinNoise::new(seed))
            .with_octaves(4)
            .with_persistence(0.5)
            .with_lacunarity(2.0);

        Self {
            ring_count: 8,
            distortion: 0.3,
            turbulence: 0.1,
            noise_scale: 0.02,
            seed,
            width,
            height,
            noise,
        }
    }

    /// Set the number of rings.
    pub fn with_ring_count(mut self, count: u32) -> Self {
        self.ring_count = count;
        self
    }

    /// Set the distortion amount.
    pub fn with_distortion(mut self, distortion: f64) -> Self {
        self.distortion = distortion;
        self
    }

    /// Set the turbulence amount.
    pub fn with_turbulence(mut self, turbulence: f64) -> Self {
        self.turbulence = turbulence;
        self
    }

    /// Set the noise scale.
    pub fn with_noise_scale(mut self, scale: f64) -> Self {
        self.noise_scale = scale;
        self
    }
}

impl Pattern2D for WoodGrainPattern {
    fn sample(&self, x: u32, y: u32) -> f64 {
        // Normalize coordinates to [0, 1]
        let nx = x as f64 / self.width as f64;
        let ny = y as f64 / self.height as f64;

        // Center coordinates
        let cx = nx - 0.5;
        let cy = ny - 0.5;

        // Sample noise for distortion
        let noise_x = x as f64 * self.noise_scale;
        let noise_y = y as f64 * self.noise_scale;
        let distort = self.noise.sample(noise_x, noise_y) * self.distortion;

        // Turbulence for fine detail
        let turb = self.noise.sample(noise_x * 4.0, noise_y * 4.0) * self.turbulence;

        // Distance from center (for rings)
        let dist = (cx * cx + cy * cy).sqrt();

        // Create rings with distortion
        let ring_value = (dist * self.ring_count as f64 + distort * 10.0 + turb) % 1.0;

        // Create smooth rings using sine
        let grain = (ring_value * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5;

        // Add fine grain detail
        let fine_grain = self.noise.sample(noise_x * 8.0, noise_y * 0.5);
        let detail = fine_grain * 0.05;

        (grain + detail).clamp(0.0, 1.0)
    }
}

/// Linear wood grain pattern (plank-style).
#[derive(Clone)]
pub struct LinearWoodGrainPattern {
    /// Base line spacing.
    pub line_spacing: f64,
    /// Noise scale for variation.
    pub noise_scale: f64,
    /// Amount of waviness in the grain.
    pub waviness: f64,
    /// Seed.
    pub seed: u32,
    /// Width.
    width: u32,
    /// Height.
    height: u32,
    /// Noise generator.
    noise: Fbm<PerlinNoise>,
}

impl LinearWoodGrainPattern {
    /// Create a new linear wood grain pattern.
    pub fn new(width: u32, height: u32, seed: u32) -> Self {
        let noise = Fbm::new(PerlinNoise::new(seed))
            .with_octaves(3)
            .with_persistence(0.5);

        Self {
            line_spacing: 8.0,
            noise_scale: 0.01,
            waviness: 0.2,
            seed,
            width,
            height,
            noise,
        }
    }

    /// Set line spacing.
    pub fn with_line_spacing(mut self, spacing: f64) -> Self {
        self.line_spacing = spacing;
        self
    }

    /// Set waviness.
    pub fn with_waviness(mut self, waviness: f64) -> Self {
        self.waviness = waviness;
        self
    }
}

impl Pattern2D for LinearWoodGrainPattern {
    fn sample(&self, x: u32, y: u32) -> f64 {
        let noise_x = x as f64 * self.noise_scale;
        let noise_y = y as f64 * self.noise_scale;

        // Waviness in the grain direction
        let wave = self.noise.sample(noise_x * 0.5, noise_y * 2.0) * self.waviness;

        // Main grain lines running horizontally
        let y_offset = y as f64 + wave * self.line_spacing * 5.0;
        let line_value = (y_offset / self.line_spacing) % 1.0;

        // Create soft grain lines using sine
        let grain = (line_value * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5;

        // Add some noise variation
        let noise_detail = self.noise.sample(noise_x * 4.0, noise_y * 4.0) * 0.1;

        (grain * 0.3 + 0.5 + noise_detail).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wood_grain_deterministic() {
        let pattern1 = WoodGrainPattern::new(256, 256, 42);
        let pattern2 = WoodGrainPattern::new(256, 256, 42);

        for y in 0..50 {
            for x in 0..50 {
                assert_eq!(pattern1.sample(x, y), pattern2.sample(x, y));
            }
        }
    }

    #[test]
    fn test_wood_grain_range() {
        let pattern = WoodGrainPattern::new(256, 256, 42);

        for y in 0..100 {
            for x in 0..100 {
                let v = pattern.sample(x, y);
                assert!(v >= 0.0 && v <= 1.0);
            }
        }
    }

    #[test]
    fn test_linear_wood_grain() {
        let pattern = LinearWoodGrainPattern::new(256, 256, 42);

        for y in 0..100 {
            for x in 0..100 {
                let v = pattern.sample(x, y);
                assert!(v >= 0.0 && v <= 1.0);
            }
        }
    }
}
