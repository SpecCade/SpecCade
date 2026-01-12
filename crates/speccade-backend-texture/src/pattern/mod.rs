//! Pattern generation primitives.
//!
//! Patterns are deterministic functions that generate structural features
//! like bricks, checkerboards, wood grain, etc.

mod brick;
mod checker;
mod edge_wear;
mod gradient;
mod scratches;
mod stripes;
mod wood;

pub use brick::BrickPattern;
pub use checker::CheckerPattern;
pub use edge_wear::EdgeWearPattern;
pub use gradient::GradientPattern;
pub use scratches::ScratchesPattern;
pub use stripes::StripesPattern;
pub use wood::WoodGrainPattern;

/// Trait for 2D pattern generators.
///
/// Patterns sample at pixel coordinates and return a value in [0, 1].
pub trait Pattern2D {
    /// Sample the pattern at a given pixel coordinate.
    /// Returns a value in [0.0, 1.0].
    fn sample(&self, x: u32, y: u32) -> f64;

    /// Sample with normalized coordinates [0, 1].
    fn sample_normalized(&self, u: f64, v: f64, width: u32, height: u32) -> f64 {
        let x = (u * width as f64).floor() as u32;
        let y = (v * height as f64).floor() as u32;
        self.sample(x.min(width - 1), y.min(height - 1))
    }
}

/// A pattern that combines height/mask information.
#[derive(Debug, Clone, Copy)]
pub struct PatternSample {
    /// Height value (0.0 = lowest, 1.0 = highest).
    pub height: f64,
    /// Mask value (0.0 = fully masked, 1.0 = fully visible).
    pub mask: f64,
    /// Optional secondary value (e.g., material ID).
    pub secondary: f64,
}

impl Default for PatternSample {
    fn default() -> Self {
        Self {
            height: 0.5,
            mask: 1.0,
            secondary: 0.0,
        }
    }
}

/// Trait for patterns that return detailed samples.
pub trait DetailedPattern2D {
    /// Sample the pattern at a given pixel coordinate.
    fn sample_detailed(&self, x: u32, y: u32) -> PatternSample;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoPattern;

    impl Pattern2D for EchoPattern {
        fn sample(&self, x: u32, y: u32) -> f64 {
            (x * 1000 + y) as f64
        }
    }

    #[test]
    fn sample_normalized_clamps_to_last_pixel() {
        let p = EchoPattern;
        let width = 4;
        let height = 3;

        // u/v at exactly 1.0 would otherwise compute x=width, y=height.
        let v = p.sample_normalized(1.0, 1.0, width, height);
        assert_eq!(v, ((width - 1) * 1000 + (height - 1)) as f64);
    }

    #[test]
    fn sample_normalized_uses_floor() {
        let p = EchoPattern;
        let width = 4;
        let height = 4;

        // u=0.49 -> floor(0.49*4)=1
        let v = p.sample_normalized(0.49, 0.0, width, height);
        assert_eq!(v, (1 * 1000 + 0) as f64);
    }

    #[test]
    fn pattern_sample_default_is_sane() {
        let s = PatternSample::default();
        assert_eq!(s.height, 0.5);
        assert_eq!(s.mask, 1.0);
        assert_eq!(s.secondary, 0.0);
    }
}
