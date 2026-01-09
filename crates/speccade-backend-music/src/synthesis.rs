//! Instrument sample generation using deterministic synthesis.
//!
//! This module provides waveform generation for tracker instruments.
//! All synthesis is deterministic given the same parameters and seed.

use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use speccade_spec::recipe::audio_sfx::Envelope;
use speccade_spec::recipe::music::InstrumentSynthesis;
use std::f64::consts::PI;

use crate::note::midi_to_freq;

/// Default number of samples to generate for an instrument.
pub const DEFAULT_SAMPLE_LENGTH: usize = 22050; // 1 second at 22050 Hz

/// Generate instrument sample data using the specified synthesis method.
///
/// # Arguments
/// * `synthesis` - The synthesis type and parameters
/// * `envelope` - ADSR envelope for amplitude shaping
/// * `sample_rate` - Sample rate in Hz
/// * `duration_samples` - Number of samples to generate
/// * `seed` - Seed for deterministic random generation
///
/// # Returns
/// Vector of 16-bit signed PCM samples as bytes (little-endian)
pub fn generate_instrument_sample(
    synthesis: &InstrumentSynthesis,
    envelope: &Envelope,
    sample_rate: u32,
    duration_samples: usize,
    seed: u32,
) -> Vec<u8> {
    let samples = match synthesis {
        InstrumentSynthesis::Pulse { duty_cycle } => {
            generate_pulse_wave(*duty_cycle, sample_rate, duration_samples)
        }
        InstrumentSynthesis::Triangle => {
            generate_triangle_wave(sample_rate, duration_samples)
        }
        InstrumentSynthesis::Sawtooth => {
            generate_sawtooth_wave(sample_rate, duration_samples)
        }
        InstrumentSynthesis::Sine => {
            generate_sine_wave(sample_rate, duration_samples)
        }
        InstrumentSynthesis::Noise { periodic } => {
            generate_noise(*periodic, sample_rate, duration_samples, seed)
        }
        InstrumentSynthesis::Sample { .. } => {
            // For sample-based instruments, return empty data
            // The actual sample loading is handled elsewhere
            vec![0.0; duration_samples]
        }
    };

    // Apply envelope
    let samples = apply_envelope(&samples, envelope, sample_rate);

    // Convert to 16-bit signed PCM bytes
    samples_to_bytes(&samples)
}

/// Generate a single cycle waveform for the given synthesis type.
/// This is used for generating short loopable samples.
///
/// # Arguments
/// * `synthesis` - The synthesis type and parameters
/// * `samples_per_cycle` - Number of samples in one cycle
/// * `seed` - Seed for deterministic random generation
///
/// # Returns
/// Vector of f64 samples normalized to [-1.0, 1.0]
pub fn generate_single_cycle(
    synthesis: &InstrumentSynthesis,
    samples_per_cycle: usize,
    seed: u32,
) -> Vec<f64> {
    match synthesis {
        InstrumentSynthesis::Pulse { duty_cycle } => {
            (0..samples_per_cycle)
                .map(|i| {
                    let t = i as f64 / samples_per_cycle as f64;
                    if t < *duty_cycle { 1.0 } else { -1.0 }
                })
                .collect()
        }
        InstrumentSynthesis::Triangle => {
            (0..samples_per_cycle)
                .map(|i| {
                    let t = i as f64 / samples_per_cycle as f64;
                    if t < 0.5 {
                        4.0 * t - 1.0
                    } else {
                        3.0 - 4.0 * t
                    }
                })
                .collect()
        }
        InstrumentSynthesis::Sawtooth => {
            (0..samples_per_cycle)
                .map(|i| {
                    let t = i as f64 / samples_per_cycle as f64;
                    2.0 * t - 1.0
                })
                .collect()
        }
        InstrumentSynthesis::Sine => {
            (0..samples_per_cycle)
                .map(|i| {
                    let t = i as f64 / samples_per_cycle as f64;
                    (2.0 * PI * t).sin()
                })
                .collect()
        }
        InstrumentSynthesis::Noise { periodic } => {
            let mut rng = create_rng(seed);
            if *periodic {
                // Generate periodic noise (loopable)
                (0..samples_per_cycle)
                    .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
                    .collect()
            } else {
                // For non-periodic noise, still generate samples
                (0..samples_per_cycle)
                    .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
                    .collect()
            }
        }
        InstrumentSynthesis::Sample { .. } => {
            // Return silence for sample-based instruments
            vec![0.0; samples_per_cycle]
        }
    }
}

/// Generate a pulse wave with the specified duty cycle.
fn generate_pulse_wave(duty_cycle: f64, sample_rate: u32, num_samples: usize) -> Vec<f64> {
    // Reference frequency for C-4 (middle C)
    let freq = midi_to_freq(60);
    let samples_per_cycle = (sample_rate as f64 / freq) as usize;

    (0..num_samples)
        .map(|i| {
            let t = (i % samples_per_cycle) as f64 / samples_per_cycle as f64;
            if t < duty_cycle { 0.8 } else { -0.8 }
        })
        .collect()
}

/// Generate a triangle wave.
fn generate_triangle_wave(sample_rate: u32, num_samples: usize) -> Vec<f64> {
    let freq = midi_to_freq(60);
    let samples_per_cycle = (sample_rate as f64 / freq) as usize;

    (0..num_samples)
        .map(|i| {
            let t = (i % samples_per_cycle) as f64 / samples_per_cycle as f64;
            if t < 0.5 {
                4.0 * t - 1.0
            } else {
                3.0 - 4.0 * t
            }
        })
        .collect()
}

/// Generate a sawtooth wave.
fn generate_sawtooth_wave(sample_rate: u32, num_samples: usize) -> Vec<f64> {
    let freq = midi_to_freq(60);
    let samples_per_cycle = (sample_rate as f64 / freq) as usize;

    (0..num_samples)
        .map(|i| {
            let t = (i % samples_per_cycle) as f64 / samples_per_cycle as f64;
            2.0 * t - 1.0
        })
        .collect()
}

/// Generate a sine wave.
fn generate_sine_wave(sample_rate: u32, num_samples: usize) -> Vec<f64> {
    let freq = midi_to_freq(60);

    (0..num_samples)
        .map(|i| {
            let t = i as f64 / sample_rate as f64;
            (2.0 * PI * freq * t).sin()
        })
        .collect()
}

/// Generate noise (white noise, optionally periodic for looping).
fn generate_noise(periodic: bool, sample_rate: u32, num_samples: usize, seed: u32) -> Vec<f64> {
    let mut rng = create_rng(seed);

    if periodic {
        // Generate a short burst of noise and loop it
        let loop_length = (sample_rate as usize / 100).max(64); // ~10ms of noise
        let noise_burst: Vec<f64> = (0..loop_length)
            .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
            .collect();

        (0..num_samples)
            .map(|i| noise_burst[i % loop_length])
            .collect()
    } else {
        (0..num_samples)
            .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
            .collect()
    }
}

/// Apply ADSR envelope to samples.
fn apply_envelope(samples: &[f64], envelope: &Envelope, sample_rate: u32) -> Vec<f64> {
    let attack_samples = (envelope.attack * sample_rate as f64) as usize;
    let decay_samples = (envelope.decay * sample_rate as f64) as usize;
    let release_samples = (envelope.release * sample_rate as f64) as usize;

    // For tracker instruments, we typically want the full sample to play
    // with just attack and decay (no sustain phase for short samples)
    let sustain_end = samples.len().saturating_sub(release_samples);

    samples
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let env_value = if i < attack_samples {
                // Attack phase: ramp from 0 to 1
                i as f64 / attack_samples as f64
            } else if i < attack_samples + decay_samples {
                // Decay phase: ramp from 1 to sustain level
                let decay_progress = (i - attack_samples) as f64 / decay_samples as f64;
                1.0 - (1.0 - envelope.sustain) * decay_progress
            } else if i < sustain_end {
                // Sustain phase
                envelope.sustain
            } else {
                // Release phase: ramp from sustain to 0
                let release_progress = (i - sustain_end) as f64 / release_samples as f64;
                envelope.sustain * (1.0 - release_progress).max(0.0)
            };

            sample * env_value
        })
        .collect()
}

/// Convert f64 samples to 16-bit signed PCM bytes (little-endian).
fn samples_to_bytes(samples: &[f64]) -> Vec<u8> {
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
fn create_rng(seed: u32) -> Pcg32 {
    // Expand 32-bit seed to 64-bit for PCG32 state
    let seed64 = (seed as u64) | ((seed as u64) << 32);
    Pcg32::seed_from_u64(seed64)
}

/// Derive a seed for a specific instrument from the base seed.
///
/// Uses BLAKE3 hash for deterministic seed derivation as per SpecCade policy.
pub fn derive_instrument_seed(base_seed: u32, instrument_index: u32) -> u32 {
    let mut input = Vec::with_capacity(8);
    input.extend_from_slice(&base_seed.to_le_bytes());
    input.extend_from_slice(&instrument_index.to_le_bytes());

    let hash = blake3::hash(&input);
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    u32::from_le_bytes(bytes)
}

/// Generate a loopable waveform sample for tracker playback.
///
/// This generates a sample that loops seamlessly by using complete waveform cycles.
///
/// # Arguments
/// * `synthesis` - The synthesis type and parameters
/// * `base_note_midi` - MIDI note number for the base pitch (e.g., 60 for C4)
/// * `sample_rate` - Sample rate in Hz
/// * `num_cycles` - Number of complete cycles to include in the sample
/// * `seed` - Seed for deterministic random generation
///
/// # Returns
/// Tuple of (sample_data, loop_start, loop_length) where sample_data is 16-bit PCM bytes
pub fn generate_loopable_sample(
    synthesis: &InstrumentSynthesis,
    base_note_midi: u8,
    sample_rate: u32,
    num_cycles: usize,
    seed: u32,
) -> (Vec<u8>, u32, u32) {
    let freq = midi_to_freq(base_note_midi);
    let samples_per_cycle = (sample_rate as f64 / freq).round() as usize;
    let total_samples = samples_per_cycle * num_cycles;

    // Generate single cycle and repeat
    let cycle = generate_single_cycle(synthesis, samples_per_cycle, seed);
    let samples: Vec<f64> = cycle
        .iter()
        .cycle()
        .take(total_samples)
        .copied()
        .collect();

    let bytes = samples_to_bytes(&samples);

    // For looping, the entire sample is the loop
    // Loop start and length are in samples, not bytes
    (bytes, 0, total_samples as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sine_wave() {
        let samples = generate_sine_wave(22050, 1000);
        assert_eq!(samples.len(), 1000);
        // Check that values are in valid range
        for s in &samples {
            assert!(*s >= -1.0 && *s <= 1.0);
        }
    }

    #[test]
    fn test_generate_pulse_wave() {
        let samples = generate_pulse_wave(0.5, 22050, 1000);
        assert_eq!(samples.len(), 1000);
        // Square wave should have only two values
        for s in &samples {
            assert!((*s - 0.8).abs() < 0.001 || (*s - (-0.8)).abs() < 0.001);
        }
    }

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

    #[test]
    fn test_generate_noise_deterministic() {
        let samples1 = generate_noise(false, 22050, 100, 42);
        let samples2 = generate_noise(false, 22050, 100, 42);
        assert_eq!(samples1, samples2);

        let samples3 = generate_noise(false, 22050, 100, 43);
        assert_ne!(samples1, samples3);
    }

    #[test]
    fn test_apply_envelope() {
        let samples: Vec<f64> = vec![1.0; 1000];
        let envelope = Envelope {
            attack: 0.01,
            decay: 0.01,
            sustain: 0.5,
            release: 0.01,
        };
        let result = apply_envelope(&samples, &envelope, 22050);
        assert_eq!(result.len(), 1000);

        // First sample should be near 0 (attack start)
        assert!(result[0] < 0.1);
    }
}
