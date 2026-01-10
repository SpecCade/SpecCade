//! Brick pattern generator.

use super::{DetailedPattern2D, Pattern2D, PatternSample};
use crate::rng::DeterministicRng;

/// Brick pattern configuration.
#[derive(Debug, Clone)]
pub struct BrickPattern {
    /// Brick width in pixels.
    pub brick_width: u32,
    /// Brick height in pixels.
    pub brick_height: u32,
    /// Mortar width in pixels.
    pub mortar_width: u32,
    /// Mortar depth (0.0 = flush, 1.0 = deep).
    pub mortar_depth: f64,
    /// Brick color/height variation.
    pub brick_variation: f64,
    /// Row offset (0.5 = half brick offset for standard brick pattern).
    pub row_offset: f64,
    /// Seed for variation.
    pub seed: u32,
    /// Total width for tiling calculations.
    pub total_width: u32,
    /// Total height for tiling calculations.
    pub total_height: u32,
}

impl BrickPattern {
    /// Create a new brick pattern with default settings.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            brick_width: 64,
            brick_height: 32,
            mortar_width: 4,
            mortar_depth: 0.3,
            brick_variation: 0.1,
            row_offset: 0.5,
            seed: 42,
            total_width: width,
            total_height: height,
        }
    }

    /// Set brick dimensions.
    pub fn with_brick_size(mut self, width: u32, height: u32) -> Self {
        self.brick_width = width;
        self.brick_height = height;
        self
    }

    /// Set mortar properties.
    pub fn with_mortar(mut self, width: u32, depth: f64) -> Self {
        self.mortar_width = width;
        self.mortar_depth = depth;
        self
    }

    /// Set variation amount.
    pub fn with_variation(mut self, variation: f64) -> Self {
        self.brick_variation = variation;
        self
    }

    /// Set row offset.
    pub fn with_row_offset(mut self, offset: f64) -> Self {
        self.row_offset = offset;
        self
    }

    /// Set seed.
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    /// Get the brick index for a given pixel coordinate.
    fn get_brick_index(&self, x: u32, y: u32) -> (i32, i32) {
        let row_height = self.brick_height + self.mortar_width;
        let col_width = self.brick_width + self.mortar_width;

        let row = (y / row_height) as i32;

        // Apply row offset
        let offset = if row % 2 == 1 {
            (self.row_offset * col_width as f64) as u32
        } else {
            0
        };

        let adjusted_x = (x + self.total_width - offset) % self.total_width;
        let col = (adjusted_x / col_width) as i32;

        (col, row)
    }

    /// Get brick variation value for a given brick.
    fn get_brick_variation(&self, col: i32, row: i32) -> f64 {
        let brick_seed = self
            .seed
            .wrapping_add((col as u32).wrapping_mul(374761393))
            .wrapping_add((row as u32).wrapping_mul(668265263));

        let mut rng = DeterministicRng::new(brick_seed);
        rng.gen_f64() * self.brick_variation
    }

    /// Check if a pixel is in the mortar.
    fn is_mortar(&self, x: u32, y: u32) -> bool {
        let row_height = self.brick_height + self.mortar_width;
        let col_width = self.brick_width + self.mortar_width;

        let row = y / row_height;

        // Apply row offset
        let offset = if row % 2 == 1 {
            (self.row_offset * col_width as f64) as u32
        } else {
            0
        };

        let adjusted_x = (x + self.total_width - offset) % self.total_width;

        // Position within the brick+mortar cell
        let local_x = adjusted_x % col_width;
        let local_y = y % row_height;

        // Check if in mortar region
        local_x >= self.brick_width || local_y >= self.brick_height
    }

    /// Get the distance from the edge of the brick (for beveling).
    fn edge_distance(&self, x: u32, y: u32) -> f64 {
        if self.is_mortar(x, y) {
            return 0.0;
        }

        let row_height = self.brick_height + self.mortar_width;
        let col_width = self.brick_width + self.mortar_width;

        let row = y / row_height;

        let offset = if row % 2 == 1 {
            (self.row_offset * col_width as f64) as u32
        } else {
            0
        };

        let adjusted_x = (x + self.total_width - offset) % self.total_width;

        let local_x = adjusted_x % col_width;
        let local_y = y % row_height;

        // Distance from each edge
        let dist_left = local_x as f64;
        let dist_right = (self.brick_width - 1 - local_x) as f64;
        let dist_top = local_y as f64;
        let dist_bottom = (self.brick_height - 1 - local_y) as f64;

        // Minimum distance to any edge
        let min_dist = dist_left.min(dist_right).min(dist_top).min(dist_bottom);

        // Normalize to brick size
        let max_dist = (self.brick_width.min(self.brick_height) / 2) as f64;
        (min_dist / max_dist).clamp(0.0, 1.0)
    }
}

impl Pattern2D for BrickPattern {
    fn sample(&self, x: u32, y: u32) -> f64 {
        self.sample_detailed(x, y).height
    }
}

impl DetailedPattern2D for BrickPattern {
    fn sample_detailed(&self, x: u32, y: u32) -> PatternSample {
        if self.is_mortar(x, y) {
            return PatternSample {
                height: 1.0 - self.mortar_depth,
                mask: 0.0, // Mortar mask
                secondary: 0.0,
            };
        }

        let (col, row) = self.get_brick_index(x, y);
        let variation = self.get_brick_variation(col, row);
        let edge_dist = self.edge_distance(x, y);

        // Slight bevel at edges
        let bevel_amount = 0.1;
        let bevel = if edge_dist < 0.2 {
            1.0 - (0.2 - edge_dist) * bevel_amount / 0.2
        } else {
            1.0
        };

        PatternSample {
            height: (1.0 + variation) * bevel,
            mask: 1.0,                            // Brick mask
            secondary: (col + row * 1000) as f64, // Brick ID for per-brick effects
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brick_pattern_deterministic() {
        let pattern1 = BrickPattern::new(256, 256).with_seed(42);
        let pattern2 = BrickPattern::new(256, 256).with_seed(42);

        for y in 0..100 {
            for x in 0..100 {
                assert_eq!(pattern1.sample(x, y), pattern2.sample(x, y));
            }
        }
    }

    #[test]
    fn test_brick_mortar_detection() {
        let pattern = BrickPattern::new(256, 256)
            .with_brick_size(64, 32)
            .with_mortar(4, 0.3);

        // Point well inside a brick
        let sample = pattern.sample_detailed(10, 10);
        assert_eq!(sample.mask, 1.0);

        // Point in mortar (right edge of first brick)
        let sample = pattern.sample_detailed(65, 10);
        assert_eq!(sample.mask, 0.0);
    }

    #[test]
    fn test_brick_variation() {
        let pattern = BrickPattern::new(256, 256)
            .with_variation(0.2)
            .with_seed(42);

        // Sample from different bricks
        let v1 = pattern.sample(10, 10);
        let v2 = pattern.sample(100, 10);

        // They should be slightly different due to variation
        // (unless they happen to be the same brick)
        // Just verify both are valid
        assert!((0.0..=1.5).contains(&v1));
        assert!((0.0..=1.5).contains(&v2));
    }
}
