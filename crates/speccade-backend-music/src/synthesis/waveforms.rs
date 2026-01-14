//! Basic waveform generation functions.
//!
//! This module provides low-level waveform generators for common synthesis types.
//! All waveforms are normalized to approximately [-1.0, 1.0] range.

use rand::Rng;
use std::f64::consts::PI;

use crate::note::midi_to_freq;
use super::utils::create_rng;

/// Generate a pulse wave with the specified duty cycle.
pub(super) fn generate_pulse_wave(duty_cycle: f64, sample_rate: u32, num_samples: usize) -> Vec<f64> {
    // Reference frequency for C-4 (middle C)
    let freq = midi_to_freq(60);
    let samples_per_cycle = (sample_rate as f64 / freq) as usize;

    (0..num_samples)
        .map(|i| {
            let t = (i % samples_per_cycle) as f64 / samples_per_cycle as f64;
            if t < duty_cycle {
                0.8
            } else {
                -0.8
            }
        })
        .collect()
}

/// Generate a triangle wave.
pub(super) fn generate_triangle_wave(sample_rate: u32, num_samples: usize) -> Vec<f64> {
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
pub(super) fn generate_sawtooth_wave(sample_rate: u32, num_samples: usize) -> Vec<f64> {
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
pub(super) fn generate_sine_wave(sample_rate: u32, num_samples: usize) -> Vec<f64> {
    let freq = midi_to_freq(60);

    (0..num_samples)
        .map(|i| {
            let t = i as f64 / sample_rate as f64;
            (2.0 * PI * freq * t).sin()
        })
        .collect()
}

/// Generate noise (white noise, optionally periodic for looping).
pub(super) fn generate_noise(periodic: bool, sample_rate: u32, num_samples: usize, seed: u32) -> Vec<f64> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sine_wave() {
        let samples = generate_sine_wave(22050, 1000);
        assert_eq!(samples.len(), 1000);
        // Check that values are in valid range
        for s in &samples {
            assert!((-1.0..=1.0).contains(s));
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
    fn test_generate_noise_deterministic() {
        let samples1 = generate_noise(false, 22050, 100, 42);
        let samples2 = generate_noise(false, 22050, 100, 42);
        assert_eq!(samples1, samples2);

        let samples3 = generate_noise(false, 22050, 100, 43);
        assert_ne!(samples1, samples3);
    }
}
