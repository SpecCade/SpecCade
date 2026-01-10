//! Stripes pattern generator.

use super::Pattern2D;

/// Generates horizontal or vertical stripe patterns.
pub struct StripesPattern {
    width: u32,
    height: u32,
    stripe_width: u32,
    color1: f64,
    color2: f64,
    horizontal: bool,
}

impl StripesPattern {
    /// Create a new vertical stripes pattern.
    pub fn new_vertical(width: u32, height: u32, stripe_width: u32, color1: f64, color2: f64) -> Self {
        Self {
            width,
            height,
            stripe_width,
            color1,
            color2,
            horizontal: false,
        }
    }

    /// Create a new horizontal stripes pattern.
    pub fn new_horizontal(width: u32, height: u32, stripe_width: u32, color1: f64, color2: f64) -> Self {
        Self {
            width,
            height,
            stripe_width,
            color1,
            color2,
            horizontal: true,
        }
    }
}

impl Pattern2D for StripesPattern {
    fn sample(&self, x: u32, y: u32) -> f64 {
        let coord = if self.horizontal { y } else { x };
        let stripe_index = (coord / self.stripe_width) % 2;

        if stripe_index == 0 {
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
    fn test_vertical_stripes() {
        let pattern = StripesPattern::new_vertical(64, 64, 16, 0.0, 1.0);

        // First stripe (0-15) should be color1
        assert_eq!(pattern.sample(0, 0), 0.0);
        assert_eq!(pattern.sample(15, 0), 0.0);

        // Second stripe (16-31) should be color2
        assert_eq!(pattern.sample(16, 0), 1.0);
        assert_eq!(pattern.sample(31, 0), 1.0);

        // Third stripe (32-47) should be color1
        assert_eq!(pattern.sample(32, 0), 0.0);
    }

    #[test]
    fn test_horizontal_stripes() {
        let pattern = StripesPattern::new_horizontal(64, 64, 16, 0.0, 1.0);

        // First stripe (0-15) should be color1
        assert_eq!(pattern.sample(0, 0), 0.0);
        assert_eq!(pattern.sample(0, 15), 0.0);

        // Second stripe (16-31) should be color2
        assert_eq!(pattern.sample(0, 16), 1.0);
        assert_eq!(pattern.sample(0, 31), 1.0);

        // Third stripe (32-47) should be color1
        assert_eq!(pattern.sample(0, 32), 0.0);
    }
}
