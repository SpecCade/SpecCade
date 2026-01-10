//! Worley (Voronoi/Cellular) noise implementation.
//!
//! Pure Rust implementation of 2D Worley noise for deterministic output.

use super::Noise2D;
use crate::rng::DeterministicRng;

/// Distance function for Worley noise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceFunction {
    /// Euclidean distance (sqrt(dx^2 + dy^2)).
    Euclidean,
    /// Manhattan distance (|dx| + |dy|).
    Manhattan,
    /// Chebyshev distance (max(|dx|, |dy|)).
    Chebyshev,
}

/// Return value type for Worley noise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorleyReturn {
    /// Return the distance to the nearest point (F1).
    F1,
    /// Return the distance to the second nearest point (F2).
    F2,
    /// Return F2 - F1 (cell edges).
    F2MinusF1,
    /// Return F1 + F2 / 2 (soft cells).
    F1PlusF2,
}

/// 2D Worley (cellular) noise generator.
#[derive(Clone)]
pub struct WorleyNoise {
    /// Base seed for generating cell points.
    seed: u32,
    /// Jitter amount (0.0 = regular grid, 1.0 = full jitter).
    jitter: f64,
    /// Distance function to use.
    distance_fn: DistanceFunction,
    /// What to return.
    return_type: WorleyReturn,
}

impl WorleyNoise {
    /// Create a new Worley noise generator with default settings.
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            jitter: 1.0,
            distance_fn: DistanceFunction::Euclidean,
            return_type: WorleyReturn::F1,
        }
    }

    /// Set the jitter amount (0.0 to 1.0).
    pub fn with_jitter(mut self, jitter: f64) -> Self {
        self.jitter = jitter.clamp(0.0, 1.0);
        self
    }

    /// Set the distance function.
    pub fn with_distance_function(mut self, func: DistanceFunction) -> Self {
        self.distance_fn = func;
        self
    }

    /// Set the return type.
    pub fn with_return_type(mut self, return_type: WorleyReturn) -> Self {
        self.return_type = return_type;
        self
    }

    /// Hash function for cell coordinates to get deterministic point positions.
    fn cell_point(&self, cell_x: i32, cell_y: i32) -> (f64, f64) {
        // Create a unique seed for this cell
        let cell_seed = self
            .seed
            .wrapping_add((cell_x as u32).wrapping_mul(374761393))
            .wrapping_add((cell_y as u32).wrapping_mul(668265263));

        let mut rng = DeterministicRng::new(cell_seed);

        // Generate point within cell with jitter
        let px = cell_x as f64 + 0.5 + (rng.gen_f64() - 0.5) * self.jitter;
        let py = cell_y as f64 + 0.5 + (rng.gen_f64() - 0.5) * self.jitter;

        (px, py)
    }

    /// Compute distance between two points.
    fn distance(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let dx = x1 - x2;
        let dy = y1 - y2;

        match self.distance_fn {
            DistanceFunction::Euclidean => (dx * dx + dy * dy).sqrt(),
            DistanceFunction::Manhattan => dx.abs() + dy.abs(),
            DistanceFunction::Chebyshev => dx.abs().max(dy.abs()),
        }
    }

    /// Fast floor function.
    #[inline]
    fn fast_floor(x: f64) -> i32 {
        if x >= 0.0 {
            x as i32
        } else {
            x as i32 - 1
        }
    }
}

impl Noise2D for WorleyNoise {
    fn sample(&self, x: f64, y: f64) -> f64 {
        // Find the cell containing the point
        let cell_x = Self::fast_floor(x);
        let cell_y = Self::fast_floor(y);

        // Track the two closest distances
        let mut f1 = f64::MAX;
        let mut f2 = f64::MAX;

        // Check the 3x3 neighborhood of cells
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cx = cell_x + dx;
                let cy = cell_y + dy;

                let (px, py) = self.cell_point(cx, cy);
                let dist = self.distance(x, y, px, py);

                if dist < f1 {
                    f2 = f1;
                    f1 = dist;
                } else if dist < f2 {
                    f2 = dist;
                }
            }
        }

        // Return based on return type
        let result = match self.return_type {
            WorleyReturn::F1 => f1,
            WorleyReturn::F2 => f2,
            WorleyReturn::F2MinusF1 => f2 - f1,
            WorleyReturn::F1PlusF2 => (f1 + f2) * 0.5,
        };

        // Normalize to roughly [-1, 1] range
        // For F1 with Euclidean distance, max distance is sqrt(2) / 2 from cell center
        result * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worley_deterministic() {
        let noise1 = WorleyNoise::new(42);
        let noise2 = WorleyNoise::new(42);

        for i in 0..100 {
            let x = i as f64 * 0.1;
            let y = i as f64 * 0.13;
            assert_eq!(noise1.sample(x, y), noise2.sample(x, y));
        }
    }

    #[test]
    fn test_worley_different_seeds() {
        let noise1 = WorleyNoise::new(42);
        let noise2 = WorleyNoise::new(43);

        let mut different = false;
        for i in 0..10 {
            let x = i as f64 * 0.1;
            let y = i as f64 * 0.13;
            if noise1.sample(x, y) != noise2.sample(x, y) {
                different = true;
                break;
            }
        }
        assert!(different);
    }

    #[test]
    fn test_worley_return_types() {
        let seed = 42;
        let f1 = WorleyNoise::new(seed).with_return_type(WorleyReturn::F1);
        let f2 = WorleyNoise::new(seed).with_return_type(WorleyReturn::F2);

        let x = 1.5;
        let y = 2.3;

        // F2 should always be >= F1
        let v1 = f1.sample(x, y);
        let v2 = f2.sample(x, y);

        // After the +1 normalization, F2 should still be >= F1
        // (both are shifted by the same amount)
        assert!(v2 >= v1 || (v2 - v1).abs() < 1e-10);
    }
}
