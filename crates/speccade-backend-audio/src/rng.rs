//! Deterministic RNG using PCG32 with BLAKE3 seed derivation.
//!
//! All randomness in the audio backend flows through this module to ensure
//! deterministic output. Seeds are derived using BLAKE3 hashing to ensure
//! independent random streams for different layers.

use rand::SeedableRng;
use rand_pcg::Pcg32;

/// Creates a PCG32 RNG from a 32-bit seed.
///
/// The 32-bit seed is expanded to 64 bits by duplicating the value in both
/// halves, as required by PCG32's state initialization.
///
/// # Arguments
/// * `seed` - A 32-bit seed value
///
/// # Returns
/// A deterministically initialized PCG32 generator
pub fn create_rng(seed: u32) -> Pcg32 {
    // Expand 32-bit seed to 64-bit for PCG32 state
    let seed64 = (seed as u64) | ((seed as u64) << 32);
    Pcg32::seed_from_u64(seed64)
}

/// Derives a seed for a specific layer from the base seed.
///
/// Uses BLAKE3 to hash the base seed concatenated with the layer index,
/// producing an independent seed for each layer.
///
/// # Arguments
/// * `base_seed` - The spec's base seed (u32)
/// * `layer_index` - The 0-indexed layer number
///
/// # Returns
/// A derived u32 seed for the layer
pub fn derive_layer_seed(base_seed: u32, layer_index: u32) -> u32 {
    // Concatenate base_seed and layer_index as little-endian bytes
    let mut input = Vec::with_capacity(8);
    input.extend_from_slice(&base_seed.to_le_bytes());
    input.extend_from_slice(&layer_index.to_le_bytes());

    // Hash with BLAKE3
    let hash = blake3::hash(&input);

    // Truncate to u32 (first 4 bytes, little-endian)
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    u32::from_le_bytes(bytes)
}

/// Derives a seed for a specific component from the base seed using a string key.
///
/// Uses BLAKE3 to hash the base seed concatenated with the component key,
/// producing an independent seed for each component.
///
/// # Arguments
/// * `base_seed` - The spec's base seed (u32)
/// * `key` - A string identifier for the component (e.g., "noise", "oscillator")
///
/// # Returns
/// A derived u32 seed for the component
pub fn derive_component_seed(base_seed: u32, key: &str) -> u32 {
    // Concatenate base_seed (as little-endian bytes) and key (as UTF-8)
    let mut input = Vec::with_capacity(4 + key.len());
    input.extend_from_slice(&base_seed.to_le_bytes());
    input.extend_from_slice(key.as_bytes());

    // Hash with BLAKE3
    let hash = blake3::hash(&input);

    // Truncate to u32 (first 4 bytes, little-endian)
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    u32::from_le_bytes(bytes)
}

/// Creates an RNG for a specific layer.
///
/// Convenience function that derives the layer seed and creates the RNG.
///
/// # Arguments
/// * `base_seed` - The spec's base seed (u32)
/// * `layer_index` - The 0-indexed layer number
///
/// # Returns
/// A PCG32 generator initialized with the derived layer seed
pub fn create_layer_rng(base_seed: u32, layer_index: u32) -> Pcg32 {
    let layer_seed = derive_layer_seed(base_seed, layer_index);
    create_rng(layer_seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_rng_determinism() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let values1: Vec<f32> = (0..100).map(|_| rng1.gen()).collect();
        let values2: Vec<f32> = (0..100).map(|_| rng2.gen()).collect();

        assert_eq!(values1, values2);
    }

    #[test]
    fn test_different_seeds_produce_different_sequences() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(43);

        let values1: Vec<f32> = (0..10).map(|_| rng1.gen()).collect();
        let values2: Vec<f32> = (0..10).map(|_| rng2.gen()).collect();

        assert_ne!(values1, values2);
    }

    #[test]
    fn test_layer_seed_derivation_consistency() {
        let base = 42u32;

        let seed_a = derive_layer_seed(base, 0);
        let seed_b = derive_layer_seed(base, 0);
        assert_eq!(seed_a, seed_b);

        let seed_1 = derive_layer_seed(base, 1);
        assert_ne!(seed_a, seed_1);
    }

    #[test]
    fn test_component_seed_derivation() {
        let base = 42u32;

        let seed_noise = derive_component_seed(base, "noise");
        let seed_osc = derive_component_seed(base, "oscillator");
        assert_ne!(seed_noise, seed_osc);

        // Same key produces same seed
        let seed_noise2 = derive_component_seed(base, "noise");
        assert_eq!(seed_noise, seed_noise2);
    }

    #[test]
    fn test_layer_rng_independence() {
        let base = 42u32;

        let mut rng0 = create_layer_rng(base, 0);
        let mut rng1 = create_layer_rng(base, 1);

        let values0: Vec<f32> = (0..10).map(|_| rng0.gen()).collect();
        let values1: Vec<f32> = (0..10).map(|_| rng1.gen()).collect();

        assert_ne!(values0, values1);
    }
}
