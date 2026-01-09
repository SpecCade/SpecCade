//! Deterministic RNG wrapper using PCG32.
//!
//! All texture generation MUST use this module for random number generation
//! to ensure deterministic output per the SpecCade determinism policy.

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

/// Wrapper around PCG32 for deterministic random number generation.
#[derive(Clone)]
pub struct DeterministicRng {
    inner: Pcg32,
}

impl DeterministicRng {
    /// Create a new RNG from a 32-bit seed.
    ///
    /// The seed is expanded to 64 bits by duplicating the bits as per
    /// the SpecCade determinism policy.
    pub fn new(seed: u32) -> Self {
        // Expand 32-bit seed to 64-bit for PCG32 state
        let seed64 = (seed as u64) | ((seed as u64) << 32);
        Self {
            inner: Pcg32::seed_from_u64(seed64),
        }
    }

    /// Derive a seed for a specific layer using BLAKE3.
    pub fn derive_layer_seed(base_seed: u32, layer_index: u32) -> u32 {
        let mut input = Vec::with_capacity(8);
        input.extend_from_slice(&base_seed.to_le_bytes());
        input.extend_from_slice(&layer_index.to_le_bytes());
        let hash = blake3::hash(&input);
        let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
        u32::from_le_bytes(bytes)
    }

    /// Derive a seed for a specific variant using BLAKE3.
    pub fn derive_variant_seed(base_seed: u32, variant_id: &str) -> u32 {
        let mut input = Vec::with_capacity(4 + variant_id.len());
        input.extend_from_slice(&base_seed.to_le_bytes());
        input.extend_from_slice(variant_id.as_bytes());
        let hash = blake3::hash(&input);
        let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
        u32::from_le_bytes(bytes)
    }

    /// Generate a random f64 in the range [0.0, 1.0).
    #[inline]
    pub fn gen_f64(&mut self) -> f64 {
        self.inner.gen::<f64>()
    }

    /// Generate a random f32 in the range [0.0, 1.0).
    #[inline]
    pub fn gen_f32(&mut self) -> f32 {
        self.inner.gen::<f32>()
    }

    /// Generate a random u32.
    #[inline]
    pub fn gen_u32(&mut self) -> u32 {
        self.inner.gen::<u32>()
    }

    /// Generate a random value in the given range.
    #[inline]
    pub fn gen_range<T, R>(&mut self, range: R) -> T
    where
        T: rand::distributions::uniform::SampleUniform,
        R: rand::distributions::uniform::SampleRange<T>,
    {
        self.inner.gen_range(range)
    }

    /// Generate a random f64 in the range [-1.0, 1.0).
    #[inline]
    pub fn gen_signed_f64(&mut self) -> f64 {
        self.gen_f64() * 2.0 - 1.0
    }

    /// Generate a random f32 in the range [-1.0, 1.0).
    #[inline]
    pub fn gen_signed_f32(&mut self) -> f32 {
        self.gen_f32() * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_output() {
        let mut rng1 = DeterministicRng::new(42);
        let mut rng2 = DeterministicRng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.gen_f64(), rng2.gen_f64());
        }
    }

    #[test]
    fn test_different_seeds_produce_different_output() {
        let mut rng1 = DeterministicRng::new(42);
        let mut rng2 = DeterministicRng::new(43);

        // At least one of the first 10 values should differ
        let mut any_different = false;
        for _ in 0..10 {
            if rng1.gen_f64() != rng2.gen_f64() {
                any_different = true;
                break;
            }
        }
        assert!(any_different);
    }

    #[test]
    fn test_derive_layer_seed() {
        let seed0 = DeterministicRng::derive_layer_seed(42, 0);
        let seed1 = DeterministicRng::derive_layer_seed(42, 1);
        assert_ne!(seed0, seed1);

        // Same inputs produce same output
        let seed0_again = DeterministicRng::derive_layer_seed(42, 0);
        assert_eq!(seed0, seed0_again);
    }

    #[test]
    fn test_derive_variant_seed() {
        let soft = DeterministicRng::derive_variant_seed(42, "soft");
        let hard = DeterministicRng::derive_variant_seed(42, "hard");
        assert_ne!(soft, hard);

        // Same inputs produce same output
        let soft_again = DeterministicRng::derive_variant_seed(42, "soft");
        assert_eq!(soft, soft_again);
    }
}
