//! Edge wear pattern generator.
//!
//! Creates worn/weathered effects at edges and corners.

use super::Pattern2D;
use crate::noise::{PerlinNoise, Noise2D, Fbm};

/// Edge wear pattern configuration.
///
/// This pattern creates worn/weathered effects that accumulate at edges
/// and corners based on curvature of a height map or pattern.
#[derive(Clone)]
pub struct EdgeWearPattern {
    /// Amount of wear (0.0 to 1.0).
    pub amount: f64,
    /// Noise scale for wear variation.
    pub noise_scale: f64,
    /// Threshold for edge detection.
    pub threshold: f64,
    /// Seed for noise.
    pub seed: u32,
    /// Texture width.
    width: u32,
    /// Texture height.
    height: u32,
    /// Source height map (optional).
    height_map: Option<Vec<f64>>,
    /// Noise generator.
    noise: Fbm<PerlinNoise>,
}

impl EdgeWearPattern {
    /// Create a new edge wear pattern.
    pub fn new(width: u32, height: u32, seed: u32) -> Self {
        let noise = Fbm::new(PerlinNoise::new(seed))
            .with_octaves(4)
            .with_persistence(0.5);

        Self {
            amount: 0.5,
            noise_scale: 0.05,
            threshold: 0.3,
            seed,
            width,
            height,
            height_map: None,
            noise,
        }
    }

    /// Set the wear amount.
    pub fn with_amount(mut self, amount: f64) -> Self {
        self.amount = amount;
        self
    }

    /// Set the noise scale.
    pub fn with_noise_scale(mut self, scale: f64) -> Self {
        self.noise_scale = scale;
        self
    }

    /// Set the edge detection threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    /// Set a source height map for edge detection.
    pub fn with_height_map(mut self, height_map: Vec<f64>) -> Self {
        self.height_map = Some(height_map);
        self
    }

    /// Sample the height map (or generate procedural edges if none).
    fn sample_height(&self, x: u32, y: u32) -> f64 {
        match &self.height_map {
            Some(map) => {
                let idx = (y * self.width + x) as usize;
                if idx < map.len() {
                    map[idx]
                } else {
                    0.5
                }
            }
            None => {
                // Generate procedural height using noise
                let nx = x as f64 * 0.02;
                let ny = y as f64 * 0.02;
                self.noise.sample_01(nx, ny)
            }
        }
    }

    /// Calculate edge factor using Sobel-like edge detection.
    #[allow(clippy::needless_range_loop)]
    fn edge_factor(&self, x: u32, y: u32) -> f64 {
        // Sample 3x3 neighborhood
        let mut samples = [[0.0; 3]; 3];

        for dy in 0..3 {
            for dx in 0..3 {
                let sx = (x as i32 + dx as i32 - 1).rem_euclid(self.width as i32) as u32;
                let sy = (y as i32 + dy as i32 - 1).rem_euclid(self.height as i32) as u32;
                samples[dy][dx] = self.sample_height(sx, sy);
            }
        }

        // Sobel operators for gradient
        let gx = (samples[0][2] + 2.0 * samples[1][2] + samples[2][2])
            - (samples[0][0] + 2.0 * samples[1][0] + samples[2][0]);

        let gy = (samples[2][0] + 2.0 * samples[2][1] + samples[2][2])
            - (samples[0][0] + 2.0 * samples[0][1] + samples[0][2]);

        // Gradient magnitude
        (gx * gx + gy * gy).sqrt()
    }

    /// Calculate curvature factor (convex areas wear more).
    fn curvature_factor(&self, x: u32, y: u32) -> f64 {
        // Sample 3x3 neighborhood
        let center = self.sample_height(x, y);

        let mut neighbor_sum = 0.0;
        let mut count = 0;

        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let sx = (x as i32 + dx).rem_euclid(self.width as i32) as u32;
                let sy = (y as i32 + dy).rem_euclid(self.height as i32) as u32;

                neighbor_sum += self.sample_height(sx, sy);
                count += 1;
            }
        }

        let neighbor_avg = neighbor_sum / count as f64;

        // Positive curvature (convex) = center higher than neighbors
        (center - neighbor_avg).max(0.0)
    }
}

impl Pattern2D for EdgeWearPattern {
    fn sample(&self, x: u32, y: u32) -> f64 {
        // Get edge factor
        let edge = self.edge_factor(x, y);

        // Get curvature (convex areas)
        let curvature = self.curvature_factor(x, y);

        // Add noise variation
        let noise_x = x as f64 * self.noise_scale;
        let noise_y = y as f64 * self.noise_scale;
        let noise_val = self.noise.sample_01(noise_x, noise_y);

        // Combine edge and curvature with noise
        let wear_factor = (edge * 2.0 + curvature * 3.0) * (0.5 + noise_val * 0.5);

        // Threshold and scale
        let wear = if wear_factor > self.threshold {
            ((wear_factor - self.threshold) / (1.0 - self.threshold)) * self.amount
        } else {
            0.0
        };

        wear.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_wear_deterministic() {
        let pattern1 = EdgeWearPattern::new(256, 256, 42);
        let pattern2 = EdgeWearPattern::new(256, 256, 42);

        for y in 0..50 {
            for x in 0..50 {
                assert_eq!(pattern1.sample(x, y), pattern2.sample(x, y));
            }
        }
    }

    #[test]
    fn test_edge_wear_range() {
        let pattern = EdgeWearPattern::new(256, 256, 42);

        for y in 0..100 {
            for x in 0..100 {
                let v = pattern.sample(x, y);
                assert!(v >= 0.0 && v <= 1.0);
            }
        }
    }

    #[test]
    fn test_edge_wear_with_height_map() {
        // Create a simple step height map
        let mut height_map = vec![0.0; 256 * 256];
        for y in 0..256 {
            for x in 0..256 {
                let idx = y * 256 + x;
                height_map[idx] = if x < 128 { 0.0 } else { 1.0 };
            }
        }

        let pattern = EdgeWearPattern::new(256, 256, 42).with_height_map(height_map);

        // Sample at the edge (x=128)
        let edge_value = pattern.sample(128, 128);

        // Sample away from the edge
        let away_value = pattern.sample(64, 128);

        // Edge should have more wear
        assert!(edge_value >= away_value);
    }
}
