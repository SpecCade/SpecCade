//! Perlin noise implementation.
//!
//! Pure Rust implementation of 2D Perlin noise for deterministic output.

use super::{Noise2D, quintic, lerp};
use crate::rng::DeterministicRng;

/// 2D Perlin noise generator.
#[derive(Clone)]
pub struct PerlinNoise {
    /// Permutation table (256 values, doubled for wrapping).
    perm: [u8; 512],
}

impl PerlinNoise {
    /// Gradient vectors for 2D.
    const GRAD2: [[f64; 2]; 8] = [
        [1.0, 0.0],
        [-1.0, 0.0],
        [0.0, 1.0],
        [0.0, -1.0],
        [1.0, 1.0],
        [-1.0, 1.0],
        [1.0, -1.0],
        [-1.0, -1.0],
    ];

    /// Create a new Perlin noise generator with the given seed.
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

    /// Hash function for grid coordinates.
    #[inline]
    fn hash(&self, x: i32, y: i32) -> usize {
        let xi = (x & 255) as usize;
        let yi = (y & 255) as usize;
        self.perm[xi + self.perm[yi] as usize] as usize
    }

    /// Compute gradient dot product.
    #[inline]
    fn grad(&self, hash: usize, x: f64, y: f64) -> f64 {
        let g = &Self::GRAD2[hash & 7];
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

impl Noise2D for PerlinNoise {
    fn sample(&self, x: f64, y: f64) -> f64 {
        // Grid cell coordinates
        let x0 = Self::fast_floor(x);
        let y0 = Self::fast_floor(y);
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        // Fractional parts
        let fx = x - x0 as f64;
        let fy = y - y0 as f64;

        // Smoothed interpolation weights (quintic for smoother results)
        let u = quintic(fx);
        let v = quintic(fy);

        // Hash the four corners
        let h00 = self.hash(x0, y0);
        let h10 = self.hash(x1, y0);
        let h01 = self.hash(x0, y1);
        let h11 = self.hash(x1, y1);

        // Gradient dot products at corners
        let n00 = self.grad(h00, fx, fy);
        let n10 = self.grad(h10, fx - 1.0, fy);
        let n01 = self.grad(h01, fx, fy - 1.0);
        let n11 = self.grad(h11, fx - 1.0, fy - 1.0);

        // Bilinear interpolation
        let nx0 = lerp(n00, n10, u);
        let nx1 = lerp(n01, n11, u);
        lerp(nx0, nx1, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perlin_deterministic() {
        let noise1 = PerlinNoise::new(42);
        let noise2 = PerlinNoise::new(42);

        for i in 0..100 {
            let x = i as f64 * 0.1;
            let y = i as f64 * 0.13;
            assert_eq!(noise1.sample(x, y), noise2.sample(x, y));
        }
    }

    #[test]
    fn test_perlin_range() {
        let noise = PerlinNoise::new(42);
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

        // Perlin noise values should be roughly in [-1, 1]
        assert!(min >= -1.5);
        assert!(max <= 1.5);
    }
}
