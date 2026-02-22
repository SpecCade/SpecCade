//! Gabor noise implementation.
//!
//! This is a deterministic sparse-convolution Gabor field suitable for
//! anisotropic, fiber-like texture detail.

use std::f64::consts::PI;

use super::Noise2D;

/// 2D Gabor noise generator.
#[derive(Clone)]
pub struct GaborNoise {
    seed: u32,
    frequency: f64,
    sigma: f64,
    impulses_per_cell: u8,
}

impl GaborNoise {
    /// Create a new Gabor noise generator with deterministic defaults.
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            frequency: 0.9,
            sigma: 1.2,
            impulses_per_cell: 3,
        }
    }

    #[inline]
    fn splitmix64(mut x: u64) -> u64 {
        x = x.wrapping_add(0x9e3779b97f4a7c15);
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        x ^ (x >> 31)
    }

    #[inline]
    fn hash_u32(&self, cx: i32, cy: i32, stream: u64) -> u32 {
        let mut x = self.seed as u64;
        x ^= (cx as i64 as u64).wrapping_mul(0x9e3779b185ebca87);
        x ^= (cy as i64 as u64).wrapping_mul(0xc2b2ae3d27d4eb4f);
        x ^= stream.wrapping_mul(0x165667b19e3779f9);
        (Self::splitmix64(x) >> 32) as u32
    }

    #[inline]
    fn hash_unit(&self, cx: i32, cy: i32, stream: u64) -> f64 {
        let h = self.hash_u32(cx, cy, stream);
        (h as f64) / (u32::MAX as f64)
    }

    #[inline]
    fn fast_floor(x: f64) -> i32 {
        if x >= 0.0 {
            x as i32
        } else {
            x as i32 - 1
        }
    }
}

impl Noise2D for GaborNoise {
    fn sample(&self, x: f64, y: f64) -> f64 {
        let cx0 = Self::fast_floor(x);
        let cy0 = Self::fast_floor(y);

        let mut sum = 0.0;
        let mut weight_sum = 0.0;

        // Sparse convolution over neighboring lattice cells.
        for cy in (cy0 - 1)..=(cy0 + 1) {
            for cx in (cx0 - 1)..=(cx0 + 1) {
                for k in 0..self.impulses_per_cell {
                    let base = (k as u64) * 8;

                    let ox = self.hash_unit(cx, cy, base + 1);
                    let oy = self.hash_unit(cx, cy, base + 2);
                    let theta = self.hash_unit(cx, cy, base + 3) * (2.0 * PI);
                    let phase = self.hash_unit(cx, cy, base + 4) * (2.0 * PI);
                    let amp = if (self.hash_u32(cx, cy, base + 5) & 1) == 0 {
                        1.0
                    } else {
                        -1.0
                    };

                    let px = cx as f64 + ox;
                    let py = cy as f64 + oy;
                    let dx = x - px;
                    let dy = y - py;
                    let r2 = dx * dx + dy * dy;

                    // Compact neighborhood support for performance.
                    if r2 > 4.0 {
                        continue;
                    }

                    let dir = dx * theta.cos() + dy * theta.sin();
                    let envelope = (-(PI * self.sigma * self.sigma) * r2).exp();
                    let carrier = ((2.0 * PI * self.frequency * dir) + phase).cos();
                    let contrib = amp * envelope * carrier;

                    sum += contrib;
                    weight_sum += envelope.abs();
                }
            }
        }

        if weight_sum <= f64::EPSILON {
            0.0
        } else {
            // Keep output in a stable ~[-1,1] range.
            (sum / weight_sum.max(1e-9)).tanh()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gabor_deterministic() {
        let n1 = GaborNoise::new(42);
        let n2 = GaborNoise::new(42);
        for i in 0..64 {
            let x = i as f64 * 0.07;
            let y = i as f64 * 0.11;
            assert_eq!(n1.sample(x, y), n2.sample(x, y));
        }
    }

    #[test]
    fn test_gabor_different_seeds_differ() {
        let n1 = GaborNoise::new(42);
        let n2 = GaborNoise::new(43);
        let a = n1.sample(0.37, 1.19);
        let b = n2.sample(0.37, 1.19);
        assert_ne!(a, b);
    }

    #[test]
    fn test_gabor_range_is_bounded() {
        let noise = GaborNoise::new(123);
        for y in 0..32 {
            for x in 0..32 {
                let v = noise.sample(x as f64 * 0.1, y as f64 * 0.1);
                assert!(
                    (-1.0..=1.0).contains(&v),
                    "value {} at ({}, {}) should be in [-1, 1]",
                    v,
                    x,
                    y
                );
            }
        }
    }
}
