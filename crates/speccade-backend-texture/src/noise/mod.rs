//! Noise generation primitives.
//!
//! All noise functions are pure Rust with no external dependencies
//! and produce deterministic output given the same seed.

mod fbm;
mod perlin;
mod simplex;
mod worley;

pub use fbm::Fbm;
pub use perlin::PerlinNoise;
pub use simplex::SimplexNoise;
pub use worley::{DistanceFunction, WorleyNoise, WorleyReturn};

/// Trait for 2D noise generators.
pub trait Noise2D {
    /// Sample the noise at a given 2D coordinate.
    /// Returns a value typically in the range [-1, 1] or [0, 1] depending on the implementation.
    fn sample(&self, x: f64, y: f64) -> f64;

    /// Sample the noise and normalize to [0, 1] range.
    fn sample_01(&self, x: f64, y: f64) -> f64 {
        (self.sample(x, y) + 1.0) * 0.5
    }
}

/// Make coordinates tileable by wrapping.
#[inline]
pub fn tile_coord(coord: f64, period: f64) -> f64 {
    coord - (coord / period).floor() * period
}

/// Smooth interpolation (smoothstep).
#[inline]
pub fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Quintic interpolation (smoother than smoothstep).
#[inline]
pub fn quintic(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Linear interpolation.
#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}
