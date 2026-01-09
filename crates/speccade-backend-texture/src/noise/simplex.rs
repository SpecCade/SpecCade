//! Simplex noise implementation.
//!
//! Pure Rust implementation of 2D simplex noise based on Stefan Gustavson's
//! reference implementation, adapted for deterministic output.

use super::Noise2D;
use crate::rng::DeterministicRng;

/// 2D Simplex noise generator.
#[derive(Clone)]
pub struct SimplexNoise {
    /// Permutation table (256 values, doubled for wrapping).
    perm: [u8; 512],
}

impl SimplexNoise {
    /// Skewing factor for 2D.
    const F2: f64 = 0.3660254037844386; // (sqrt(3) - 1) / 2
    /// Unskewing factor for 2D.
    const G2: f64 = 0.21132486540518713; // (3 - sqrt(3)) / 6

    /// Gradient vectors for 2D.
    const GRAD2: [[f64; 2]; 12] = [
        [1.0, 1.0],
        [-1.0, 1.0],
        [1.0, -1.0],
        [-1.0, -1.0],
        [1.0, 0.0],
        [-1.0, 0.0],
        [1.0, 0.0],
        [-1.0, 0.0],
        [0.0, 1.0],
        [0.0, -1.0],
        [0.0, 1.0],
        [0.0, -1.0],
    ];

    /// Create a new simplex noise generator with the given seed.
    pub fn new(seed: u32) -> Self {
        let mut rng = DeterministicRng::new(seed);

        // Initialize permutation table
        let mut perm = [0u8; 512];
        let mut source: Vec<u8> = (0..=255).collect();

        // Fisher-Yates shuffle
        for i in (1..256).rev() {
            let j = rng.gen_range(0..=i);
            source.swap(i, j);
        }

        // Double the permutation table for overflow handling
        perm[..256].copy_from_slice(&source);
        perm[256..512].copy_from_slice(&source);

        Self { perm }
    }

    /// Compute dot product of gradient and distance vector.
    #[inline]
    fn grad(&self, hash: usize, x: f64, y: f64) -> f64 {
        let g = &Self::GRAD2[hash % 12];
        g[0] * x + g[1] * y
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

impl Noise2D for SimplexNoise {
    fn sample(&self, x: f64, y: f64) -> f64 {
        // Skew the input space to determine which simplex cell we're in
        let s = (x + y) * Self::F2;
        let i = Self::fast_floor(x + s);
        let j = Self::fast_floor(y + s);

        // Unskew the cell origin back to (x, y) space
        let t = (i + j) as f64 * Self::G2;
        let x0 = x - (i as f64 - t); // The x,y distances from the cell origin
        let y0 = y - (j as f64 - t);

        // For the 2D case, the simplex shape is an equilateral triangle.
        // Determine which simplex we are in.
        let (i1, j1) = if x0 > y0 {
            (1, 0) // lower triangle, XY order: (0,0)->(1,0)->(1,1)
        } else {
            (0, 1) // upper triangle, XY order: (0,0)->(0,1)->(1,1)
        };

        // Offsets for the middle corner in (x,y) unskewed coords
        let x1 = x0 - i1 as f64 + Self::G2;
        let y1 = y0 - j1 as f64 + Self::G2;

        // Offsets for the last corner in (x,y) unskewed coords
        let x2 = x0 - 1.0 + 2.0 * Self::G2;
        let y2 = y0 - 1.0 + 2.0 * Self::G2;

        // Hash coordinates of the three simplex corners
        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;

        let gi0 = self.perm[ii + self.perm[jj] as usize] as usize;
        let gi1 = self.perm[ii + i1 + self.perm[jj + j1] as usize] as usize;
        let gi2 = self.perm[ii + 1 + self.perm[jj + 1] as usize] as usize;

        // Calculate the contribution from the three corners
        let mut n0 = 0.0;
        let mut t0 = 0.5 - x0 * x0 - y0 * y0;
        if t0 >= 0.0 {
            t0 *= t0;
            n0 = t0 * t0 * self.grad(gi0, x0, y0);
        }

        let mut n1 = 0.0;
        let mut t1 = 0.5 - x1 * x1 - y1 * y1;
        if t1 >= 0.0 {
            t1 *= t1;
            n1 = t1 * t1 * self.grad(gi1, x1, y1);
        }

        let mut n2 = 0.0;
        let mut t2 = 0.5 - x2 * x2 - y2 * y2;
        if t2 >= 0.0 {
            t2 *= t2;
            n2 = t2 * t2 * self.grad(gi2, x2, y2);
        }

        // Scale to return values in the interval [-1, 1]
        70.0 * (n0 + n1 + n2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplex_deterministic() {
        let noise1 = SimplexNoise::new(42);
        let noise2 = SimplexNoise::new(42);

        for i in 0..100 {
            let x = i as f64 * 0.1;
            let y = i as f64 * 0.13;
            assert_eq!(noise1.sample(x, y), noise2.sample(x, y));
        }
    }

    #[test]
    fn test_simplex_range() {
        let noise = SimplexNoise::new(42);
        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for i in 0..1000 {
            for j in 0..1000 {
                let x = i as f64 * 0.01;
                let y = j as f64 * 0.01;
                let v = noise.sample(x, y);
                min = min.min(v);
                max = max.max(v);
            }
        }

        // Values should be roughly in [-1, 1]
        assert!(min >= -1.5);
        assert!(max <= 1.5);
    }

    #[test]
    fn test_different_seeds() {
        let noise1 = SimplexNoise::new(42);
        let noise2 = SimplexNoise::new(43);

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
}
