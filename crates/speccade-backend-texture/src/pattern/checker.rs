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

        if (tile_x + tile_y) % 2 == 0 {
            self.color1
        } else {
            self.color2
        }
    }
}

/// Diagonal stripe pattern.
#[derive(Debug, Clone)]
pub struct DiagonalStripePattern {
    /// Stripe width in pixels.
    pub stripe_width: u32,
    /// Value for primary stripes.
    pub color1: f64,
    /// Value for secondary stripes.
    pub color2: f64,
    /// Angle offset (1 = 45 degrees, 2 = steeper, 0.5 = shallower).
    pub angle_factor: f64,
}

impl DiagonalStripePattern {
    /// Create a new diagonal stripe pattern.
    pub fn new(stripe_width: u32) -> Self {
        Self {
            stripe_width,
            color1: 1.0,
            color2: 0.0,
            angle_factor: 1.0,
        }
    }

    /// Set the colors/values.
    pub fn with_colors(mut self, color1: f64, color2: f64) -> Self {
        self.color1 = color1;
        self.color2 = color2;
        self
    }

    /// Set the angle factor.
    pub fn with_angle(mut self, factor: f64) -> Self {
        self.angle_factor = factor;
        self
    }
}

impl Pattern2D for DiagonalStripePattern {
    fn sample(&self, x: u32, y: u32) -> f64 {
        let pos = (x as f64 + y as f64 * self.angle_factor) as u32;
        let stripe = pos / self.stripe_width;

        if stripe % 2 == 0 {
            self.color1
        } else {
            self.color2
        }
    }
}

/// Grid pattern (lines in both directions).
#[derive(Debug, Clone)]
pub struct GridPattern {
    /// Cell size in pixels.
    pub cell_size: u32,
    /// Line width in pixels.
    pub line_width: u32,
    /// Value for lines.
    pub line_color: f64,
    /// Value for cells.
    pub cell_color: f64,
}

impl GridPattern {
    /// Create a new grid pattern.
    pub fn new(cell_size: u32, line_width: u32) -> Self {
        Self {
            cell_size,
            line_width,
            line_color: 0.0,
            cell_color: 1.0,
        }
    }

    /// Set the colors.
    pub fn with_colors(mut self, line_color: f64, cell_color: f64) -> Self {
        self.line_color = line_color;
        self.cell_color = cell_color;
        self
    }
}

impl Pattern2D for GridPattern {
    fn sample(&self, x: u32, y: u32) -> f64 {
        let local_x = x % self.cell_size;
        let local_y = y % self.cell_size;

        // Check if on a grid line
        if local_x < self.line_width || local_y < self.line_width {
            self.line_color
        } else {
            self.cell_color
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

    #[test]
    fn test_grid_lines() {
        let pattern = GridPattern::new(32, 2);

        // On a line
        assert_eq!(pattern.sample(0, 10), 0.0);
        assert_eq!(pattern.sample(10, 0), 0.0);

        // Not on a line
        assert_eq!(pattern.sample(10, 10), 1.0);
    }
}
