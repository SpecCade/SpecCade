//! Ring modulator effect implementation.
//!
//! Multiplies input audio with a carrier sine oscillator to produce
//! sum and difference frequencies (sidebands). Creates metallic,
//! robotic, and sci-fi timbres.

use crate::error::{AudioError, AudioResult};
use crate::mixer::StereoOutput;
use std::f64::consts::PI;

const TWO_PI: f64 = 2.0 * PI;

/// Applies ring modulator effect to stereo audio.
///
/// Ring modulation multiplies the input signal with a carrier oscillator,
/// creating sum and difference frequencies (sidebands). Unlike AM synthesis
/// which preserves the carrier, ring modulation produces only sidebands,
/// resulting in more complex and inharmonic spectra.
///
/// # Arguments
/// * `stereo` - Stereo audio to process in-place
/// * `frequency` - Carrier oscillator frequency in Hz (20-2000 typical)
/// * `mix` - Wet/dry mix (0.0 = dry input, 1.0 = full ring mod)
/// * `sample_rate` - Sample rate in Hz
///
/// # Algorithm
/// For each sample:
/// ```text
/// carrier = sin(2 * PI * frequency * time)
/// modulated = input * carrier
/// output = mix * modulated + (1 - mix) * input
/// ```
pub fn apply(
    stereo: &mut StereoOutput,
    frequency: f64,
    mix: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(1.0..=20000.0).contains(&frequency) {
        return Err(AudioError::invalid_param(
            "ring_modulator.frequency",
            format!("must be 1.0-20000.0 Hz, got {}", frequency),
        ));
    }
    if !(0.0..=1.0).contains(&mix) {
        return Err(AudioError::invalid_param(
            "ring_modulator.mix",
            format!("must be 0.0-1.0, got {}", mix),
        ));
    }

    let num_samples = stereo.left.len();
    if num_samples == 0 {
        return Ok(());
    }

    let dry = 1.0 - mix;

    // Phase accumulator for deterministic oscillator
    let mut phase = 0.0;
    let phase_increment = frequency / sample_rate;

    for i in 0..num_samples {
        // Generate carrier oscillator
        let carrier = (TWO_PI * phase).sin();

        // Apply ring modulation to both channels
        let wet_left = stereo.left[i] * carrier;
        let wet_right = stereo.right[i] * carrier;

        // Mix wet and dry signals
        stereo.left[i] = mix * wet_left + dry * stereo.left[i];
        stereo.right[i] = mix * wet_right + dry * stereo.right[i];

        // Advance phase (deterministic)
        phase += phase_increment;
        if phase >= 1.0 {
            phase -= 1.0;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_mod_basic() {
        let sample_rate = 44100.0;
        let num_samples = 4410; // 100ms

        // Generate a simple sine wave
        let freq = 440.0;
        let samples: Vec<f64> = (0..num_samples)
            .map(|i| (TWO_PI * freq * i as f64 / sample_rate).sin() * 0.5)
            .collect();

        let mut stereo = StereoOutput {
            left: samples.clone(),
            right: samples,
        };

        // Apply ring modulator effect
        apply(&mut stereo, 150.0, 0.8, sample_rate).unwrap();

        // Output should have same length
        assert_eq!(stereo.left.len(), num_samples);
        assert_eq!(stereo.right.len(), num_samples);

        // Output should not be silent
        let max_left = stereo.left.iter().fold(0.0_f64, |a, &b| a.max(b.abs()));
        let max_right = stereo.right.iter().fold(0.0_f64, |a, &b| a.max(b.abs()));
        assert!(max_left > 0.1);
        assert!(max_right > 0.1);
    }

    #[test]
    fn test_ring_mod_dry_passthrough() {
        let sample_rate = 44100.0;
        let num_samples = 1000;

        let samples: Vec<f64> = (0..num_samples).map(|i| i as f64 / 1000.0).collect();

        let original = samples.clone();
        let mut stereo = StereoOutput {
            left: samples.clone(),
            right: samples,
        };

        // With mix=0, output should equal input
        apply(&mut stereo, 500.0, 0.0, sample_rate).unwrap();

        for (i, (&out, &orig)) in stereo.left.iter().zip(original.iter()).enumerate() {
            assert!(
                (out - orig).abs() < 1e-10,
                "Dry signal should pass through unchanged at sample {}",
                i
            );
        }
    }

    #[test]
    fn test_ring_mod_full_wet() {
        let sample_rate = 44100.0;
        let num_samples = 4410;

        // Generate a constant signal
        let samples: Vec<f64> = vec![0.5; num_samples];

        let mut stereo = StereoOutput {
            left: samples.clone(),
            right: samples,
        };

        // Full wet ring mod
        apply(&mut stereo, 100.0, 1.0, sample_rate).unwrap();

        // Output should oscillate at carrier frequency
        // Check that it's not constant
        let mut has_variation = false;
        let first = stereo.left[0];
        for &sample in &stereo.left[1..] {
            if (sample - first).abs() > 0.01 {
                has_variation = true;
                break;
            }
        }
        assert!(
            has_variation,
            "Ring mod should create variation from carrier"
        );
    }

    #[test]
    fn test_ring_mod_stereo_coherence() {
        let sample_rate = 44100.0;
        let num_samples = 2205;

        // Identical L/R input
        let samples: Vec<f64> = (0..num_samples)
            .map(|i| (TWO_PI * 220.0 * i as f64 / sample_rate).sin() * 0.5)
            .collect();

        let mut stereo = StereoOutput {
            left: samples.clone(),
            right: samples,
        };

        apply(&mut stereo, 150.0, 0.7, sample_rate).unwrap();

        // With identical input, output should also be identical
        for i in 0..num_samples {
            assert!(
                (stereo.left[i] - stereo.right[i]).abs() < 1e-10,
                "Stereo channels should be identical for identical input"
            );
        }
    }

    #[test]
    fn test_ring_mod_parameter_validation() {
        let mut stereo = StereoOutput {
            left: vec![0.0; 100],
            right: vec![0.0; 100],
        };

        // Frequency too low
        assert!(apply(&mut stereo, 0.5, 0.5, 44100.0).is_err());

        // Frequency too high
        assert!(apply(&mut stereo, 25000.0, 0.5, 44100.0).is_err());

        // Mix out of range
        assert!(apply(&mut stereo, 500.0, -0.1, 44100.0).is_err());
        assert!(apply(&mut stereo, 500.0, 1.5, 44100.0).is_err());

        // Valid parameters should work
        assert!(apply(&mut stereo, 500.0, 0.5, 44100.0).is_ok());
    }

    #[test]
    fn test_ring_mod_deterministic() {
        let sample_rate = 44100.0;
        let num_samples = 2205;

        let samples: Vec<f64> = (0..num_samples)
            .map(|i| (TWO_PI * 330.0 * i as f64 / sample_rate).sin() * 0.5)
            .collect();

        let mut stereo1 = StereoOutput {
            left: samples.clone(),
            right: samples.clone(),
        };

        let mut stereo2 = StereoOutput {
            left: samples.clone(),
            right: samples,
        };

        apply(&mut stereo1, 200.0, 0.6, sample_rate).unwrap();
        apply(&mut stereo2, 200.0, 0.6, sample_rate).unwrap();

        // Results should be identical
        for i in 0..num_samples {
            assert!(
                (stereo1.left[i] - stereo2.left[i]).abs() < 1e-15,
                "Left channel should be deterministic"
            );
            assert!(
                (stereo1.right[i] - stereo2.right[i]).abs() < 1e-15,
                "Right channel should be deterministic"
            );
        }
    }

    #[test]
    fn test_ring_mod_empty_input() {
        let mut stereo = StereoOutput {
            left: vec![],
            right: vec![],
        };

        // Should handle empty input gracefully
        assert!(apply(&mut stereo, 500.0, 0.5, 44100.0).is_ok());
        assert!(stereo.left.is_empty());
        assert!(stereo.right.is_empty());
    }
}
