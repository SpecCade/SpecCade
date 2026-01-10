//! Gradient pattern generator.

use super::Pattern2D;

/// Generates linear or radial gradient patterns.
pub struct GradientPattern {
    width: u32,
    height: u32,
    gradient_type: GradientType,
}

enum GradientType {
    Horizontal { start: f64, end: f64 },
    Vertical { start: f64, end: f64 },
    Radial { center: [f64; 2], inner: f64, outer: f64 },
}

impl GradientPattern {
    /// Create a new horizontal gradient (left to right).
    pub fn new_horizontal(width: u32, height: u32, start: f64, end: f64) -> Self {
        Self {
            width,
            height,
            gradient_type: GradientType::Horizontal { start, end },
        }
    }

    /// Create a new vertical gradient (top to bottom).
    pub fn new_vertical(width: u32, height: u32, start: f64, end: f64) -> Self {
        Self {
            width,
            height,
            gradient_type: GradientType::Vertical { start, end },
        }
    }

    /// Create a new radial gradient (center outward).
    pub fn new_radial(width: u32, height: u32, center: [f64; 2], inner: f64, outer: f64) -> Self {
        Self {
            width,
            height,
            gradient_type: GradientType::Radial { center, inner, outer },
        }
    }
}

impl Pattern2D for GradientPattern {
    fn sample(&self, x: u32, y: u32) -> f64 {
        match &self.gradient_type {
            GradientType::Horizontal { start, end } => {
                let t = x as f64 / (self.width as f64 - 1.0).max(1.0);
                start + (end - start) * t
            }
            GradientType::Vertical { start, end } => {
                let t = y as f64 / (self.height as f64 - 1.0).max(1.0);
                start + (end - start) * t
            }
            GradientType::Radial { center, inner, outer } => {
                // Normalized coordinates [0, 1]
                let nx = x as f64 / (self.width as f64 - 1.0).max(1.0);
                let ny = y as f64 / (self.height as f64 - 1.0).max(1.0);

                // Distance from center
                let dx = nx - center[0];
                let dy = ny - center[1];
                let dist = (dx * dx + dy * dy).sqrt();

                // Normalize to [0, 1] range based on max possible distance
                let max_dist = ((center[0].max(1.0 - center[0])).powi(2)
                              + (center[1].max(1.0 - center[1])).powi(2)).sqrt();
                let t = (dist / max_dist).clamp(0.0, 1.0);

                // Interpolate between inner and outer
                inner + (outer - inner) * t
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal_gradient() {
        let pattern = GradientPattern::new_horizontal(100, 100, 0.0, 1.0);

        // Left edge should be start value
        assert!((pattern.sample(0, 0) - 0.0).abs() < 0.01);

        // Middle should be ~0.5
        let mid = pattern.sample(50, 0);
        assert!((mid - 0.5).abs() < 0.01);

        // Right edge should be end value
        assert!((pattern.sample(99, 0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_vertical_gradient() {
        let pattern = GradientPattern::new_vertical(100, 100, 0.0, 1.0);

        // Top edge should be start value
        assert!((pattern.sample(0, 0) - 0.0).abs() < 0.01);

        // Middle should be ~0.5
        let mid = pattern.sample(0, 50);
        assert!((mid - 0.5).abs() < 0.01);

        // Bottom edge should be end value
        assert!((pattern.sample(0, 99) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_radial_gradient() {
        let pattern = GradientPattern::new_radial(100, 100, [0.5, 0.5], 1.0, 0.0);

        // Center should be inner value
        let center_val = pattern.sample(50, 50);
        assert!((center_val - 1.0).abs() < 0.1);

        // Edges should be closer to outer value
        let edge_val = pattern.sample(0, 0);
        assert!(edge_val < 0.5);
    }
}
