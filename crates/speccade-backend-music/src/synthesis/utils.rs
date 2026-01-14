//! Utility functions for synthesis: sample conversion, RNG, resampling.

use rand::SeedableRng;
use rand_pcg::Pcg32;

/// Convert f64 samples to 16-bit signed PCM bytes (little-endian).
pub(super) fn samples_to_bytes(samples: &[f64]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(samples.len() * 2);

    for &sample in samples {
        // Clamp to [-1.0, 1.0] and convert to i16
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_sample = (clamped * 32767.0) as i16;

        // Write as little-endian bytes
        bytes.push(i16_sample as u8);
        bytes.push((i16_sample >> 8) as u8);
    }

    bytes
}

/// Create a seeded PCG32 RNG following SpecCade determinism policy.
pub(super) fn create_rng(seed: u32) -> Pcg32 {
    // Expand 32-bit seed to 64-bit for PCG32 state
    let seed64 = (seed as u64) | ((seed as u64) << 32);
    Pcg32::seed_from_u64(seed64)
}

/// Derive a seed for a specific instrument from the base seed.
///
/// Uses BLAKE3 hash for deterministic seed derivation as per SpecCade policy.
pub fn derive_instrument_seed(base_seed: u32, instrument_index: u32) -> u32 {
    speccade_spec::hash::derive_layer_seed(base_seed, instrument_index)
}

/// Resample audio using deterministic linear interpolation.
///
/// This function resamples audio from one sample rate to another using
/// linear interpolation, which is simple and deterministic.
///
/// # Arguments
/// * `samples` - Input samples (normalized to [-1.0, 1.0])
/// * `from_rate` - Source sample rate
/// * `to_rate` - Target sample rate
///
/// # Returns
/// Resampled audio at the target sample rate
#[allow(dead_code)] // Available for optional quality reduction if needed
pub(super) fn resample_linear(samples: &[f64], from_rate: u32, to_rate: u32) -> Vec<f64> {
    if samples.is_empty() {
        return Vec::new();
    }

    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = (samples.len() as f64 / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 * ratio;
        let src_idx = src_pos.floor() as usize;
        let frac = src_pos - src_idx as f64;

        // Linear interpolation between samples
        let sample = if src_idx + 1 < samples.len() {
            let s0 = samples[src_idx];
            let s1 = samples[src_idx + 1];
            s0 + (s1 - s0) * frac
        } else {
            // Last sample, no interpolation needed
            samples[src_idx.min(samples.len() - 1)]
        };

        output.push(sample);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_samples_to_bytes() {
        let samples = vec![0.0, 0.5, 1.0, -1.0];
        let bytes = samples_to_bytes(&samples);
        assert_eq!(bytes.len(), 8); // 4 samples * 2 bytes each

        // Check zero
        let s0 = i16::from_le_bytes([bytes[0], bytes[1]]);
        assert_eq!(s0, 0);

        // Check max
        let s2 = i16::from_le_bytes([bytes[4], bytes[5]]);
        assert_eq!(s2, 32767);

        // Check min
        let s3 = i16::from_le_bytes([bytes[6], bytes[7]]);
        assert_eq!(s3, -32767);
    }

    #[test]
    fn test_derive_instrument_seed_deterministic() {
        let seed1 = derive_instrument_seed(42, 0);
        let seed2 = derive_instrument_seed(42, 0);
        assert_eq!(seed1, seed2);

        let seed3 = derive_instrument_seed(42, 1);
        assert_ne!(seed1, seed3);
    }
}
