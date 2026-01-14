//! Instrument sample generation using deterministic synthesis.
//!
//! This module provides waveform generation for tracker instruments.
//! All synthesis is deterministic given the same parameters and seed.

mod envelope;
mod utils;
mod wav;
mod waveforms;

use rand::Rng;
use speccade_spec::recipe::audio::Envelope;
use speccade_spec::recipe::music::InstrumentSynthesis;
use std::f64::consts::PI;

use crate::note::midi_to_freq;

// Re-export public items
pub use utils::derive_instrument_seed;
pub use wav::load_wav_sample;

// Use items from submodules
use envelope::apply_envelope;
use utils::{create_rng, samples_to_bytes};
use waveforms::{
    generate_noise, generate_pulse_wave, generate_sawtooth_wave, generate_sine_wave,
    generate_triangle_wave,
};

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
        InstrumentSynthesis::Square => {
            // Square wave is a 50% duty cycle pulse
            generate_pulse_wave(0.5, sample_rate, duration_samples)
        }
        InstrumentSynthesis::Triangle => generate_triangle_wave(sample_rate, duration_samples),
        InstrumentSynthesis::Sawtooth => generate_sawtooth_wave(sample_rate, duration_samples),
        InstrumentSynthesis::Sine => generate_sine_wave(sample_rate, duration_samples),
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
        InstrumentSynthesis::Pulse { duty_cycle } => (0..samples_per_cycle)
            .map(|i| {
                let t = i as f64 / samples_per_cycle as f64;
                if t < *duty_cycle {
                    1.0
                } else {
                    -1.0
                }
            })
            .collect(),
        InstrumentSynthesis::Square => {
            // Square wave is a 50% duty cycle pulse
            (0..samples_per_cycle)
                .map(|i| {
                    let t = i as f64 / samples_per_cycle as f64;
                    if t < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                })
                .collect()
        }
        InstrumentSynthesis::Triangle => (0..samples_per_cycle)
            .map(|i| {
                let t = i as f64 / samples_per_cycle as f64;
                if t < 0.5 {
                    4.0 * t - 1.0
                } else {
                    3.0 - 4.0 * t
                }
            })
            .collect(),
        InstrumentSynthesis::Sawtooth => (0..samples_per_cycle)
            .map(|i| {
                let t = i as f64 / samples_per_cycle as f64;
                2.0 * t - 1.0
            })
            .collect(),
        InstrumentSynthesis::Sine => (0..samples_per_cycle)
            .map(|i| {
                let t = i as f64 / samples_per_cycle as f64;
                (2.0 * PI * t).sin()
            })
            .collect(),
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
    let samples: Vec<f64> = cycle.iter().cycle().take(total_samples).copied().collect();

    let bytes = samples_to_bytes(&samples);

    // For looping, the entire sample is the loop
    // Loop start and length are in samples, not bytes
    (bytes, 0, total_samples as u32)
}

/// Generate a fixed-length sample for tracker playback.
///
/// This is used for one-shot / percussive instruments where looping would create
/// obvious repetition (especially for noise).
///
/// For periodic waveforms, this repeats a single cycle to fill `duration_samples`.
/// For non-periodic noise, this generates unique random samples for the full duration.
pub fn generate_fixed_length_sample(
    synthesis: &InstrumentSynthesis,
    base_note_midi: u8,
    sample_rate: u32,
    duration_samples: usize,
    seed: u32,
) -> Vec<u8> {
    if duration_samples == 0 {
        return Vec::new();
    }

    let samples: Vec<f64> = match synthesis {
        InstrumentSynthesis::Noise { .. } => {
            // One-shot noise: do NOT repeat, or it will sound pitched / ringing.
            generate_single_cycle(synthesis, duration_samples, seed)
        }
        InstrumentSynthesis::Sample { .. } => vec![0.0; duration_samples],
        _ => {
            let freq = midi_to_freq(base_note_midi);
            let samples_per_cycle = (sample_rate as f64 / freq).round().max(1.0) as usize;
            let cycle = generate_single_cycle(synthesis, samples_per_cycle, seed);
            cycle
                .iter()
                .cycle()
                .take(duration_samples)
                .copied()
                .collect()
        }
    };

    samples_to_bytes(&samples)
}
