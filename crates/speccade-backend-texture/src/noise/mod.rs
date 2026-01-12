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

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    struct ConstNoise(f64);

    impl Noise2D for ConstNoise {
        fn sample(&self, _x: f64, _y: f64) -> f64 {
            self.0
        }
    }

    #[test]
    fn noise2d_sample_01_maps_minus1_to_0_and_plus1_to_1() {
        assert!(approx_eq(ConstNoise(-1.0).sample_01(0.0, 0.0), 0.0));
        assert!(approx_eq(ConstNoise(1.0).sample_01(0.0, 0.0), 1.0));
        assert!(approx_eq(ConstNoise(0.0).sample_01(0.0, 0.0), 0.5));
    }

    #[test]
    fn tile_coord_wraps_into_period() {
        assert!(approx_eq(tile_coord(0.0, 4.0), 0.0));
        assert!(approx_eq(tile_coord(3.9, 4.0), 3.9));
        assert!(approx_eq(tile_coord(4.0, 4.0), 0.0));
        assert!(approx_eq(tile_coord(-0.1, 4.0), 3.9));
        assert!(approx_eq(tile_coord(-4.0, 4.0), 0.0));
    }

    #[test]
    fn smoothstep_endpoints_and_midpoint() {
        assert!(approx_eq(smoothstep(0.0), 0.0));
        assert!(approx_eq(smoothstep(1.0), 1.0));
        assert!(approx_eq(smoothstep(0.5), 0.5));
    }

    #[test]
    fn quintic_endpoints() {
        assert!(approx_eq(quintic(0.0), 0.0));
        assert!(approx_eq(quintic(1.0), 1.0));
    }

    #[test]
    fn lerp_endpoints() {
        assert!(approx_eq(lerp(10.0, 20.0, 0.0), 10.0));
        assert!(approx_eq(lerp(10.0, 20.0, 1.0), 20.0));
        assert!(approx_eq(lerp(10.0, 20.0, 0.5), 15.0));
    }
}
