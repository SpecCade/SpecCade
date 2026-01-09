//! Checkerboard pattern generator.

use super::Pattern2D;

/// Checkerboard pattern configuration.
#[derive(Debug, Clone)]
pub struct CheckerPattern {
    /// Tile size in pixels.
    pub tile_size: u32,
    /// Value for "white" tiles.
    pub color1: f64,
    /// Value for "black" tiles.
    pub color2: f64,
}

impl CheckerPattern {
    /// Create a new checkerboard pattern.
    pub fn new(tile_size: u32) -> Self {
        Self {
            tile_size,
            color1: 1.0,
            color2: 0.0,
        }
    }

    /// Set the colors/values for the two tile types.
    pub fn with_colors(mut self, color1: f64, color2: f64) -> Self {
        self.color1 = color1;
        self.color2 = color2;
        self
    }
}

impl Pattern2D for CheckerPattern {
    fn sample(&self, x: u32, y: u32) -> f64 {
        let tile_x = x / self.tile_size;
        let tile_y = y / self.tile_size;

        if (tile_x + tile_y).is_multiple_of(2) {
            self.color1
        } else {
            self.color2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checker_alternates() {
        let pattern = CheckerPattern::new(32);

        // First tile should be color1
        assert_eq!(pattern.sample(0, 0), 1.0);

        // Adjacent tiles should be color2
        assert_eq!(pattern.sample(32, 0), 0.0);
        assert_eq!(pattern.sample(0, 32), 0.0);

        // Diagonal should be color1 again
        assert_eq!(pattern.sample(32, 32), 1.0);
    }
}
