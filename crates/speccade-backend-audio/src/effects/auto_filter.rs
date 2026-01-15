//! Auto-filter / envelope follower effect implementation.
//!
//! Implements a dynamic filter that sweeps cutoff based on input signal level.
//! Creates auto-wah and envelope-controlled filter effects.

use crate::error::{AudioError, AudioResult};
use crate::filter::{BiquadCoeffs, BiquadFilter};
use crate::mixer::StereoOutput;

/// Applies auto-filter (envelope follower) effect to stereo audio.
///
/// The effect tracks the input signal level with an envelope follower and uses
/// that to modulate a lowpass filter's cutoff frequency. Higher signal levels
/// result in higher cutoff frequencies, creating dynamic "wah" effects.
///
/// # Arguments
/// * `stereo` - Stereo audio buffer to process in place
/// * `sensitivity` - How much signal level affects filter (0.0-1.0)
/// * `attack_ms` - Envelope attack time in ms (0.1-100)
/// * `release_ms` - Envelope release time in ms (10-1000)
/// * `depth` - Filter sweep range (0.0-1.0)
/// * `base_frequency` - Base cutoff when signal is quiet (100-8000 Hz)
/// * `sample_rate` - Sample rate in Hz
pub fn apply(
    stereo: &mut StereoOutput,
    sensitivity: f64,
    attack_ms: f64,
    release_ms: f64,
    depth: f64,
    base_frequency: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(0.0..=1.0).contains(&sensitivity) {
        return Err(AudioError::invalid_param(
            "auto_filter.sensitivity",
            format!("must be 0.0-1.0, got {}", sensitivity),
        ));
    }
    if !(0.1..=100.0).contains(&attack_ms) {
        return Err(AudioError::invalid_param(
            "auto_filter.attack_ms",
            format!("must be 0.1-100, got {}", attack_ms),
        ));
    }
    if !(10.0..=1000.0).contains(&release_ms) {
        return Err(AudioError::invalid_param(
            "auto_filter.release_ms",
            format!("must be 10-1000, got {}", release_ms),
        ));
    }
    if !(0.0..=1.0).contains(&depth) {
        return Err(AudioError::invalid_param(
            "auto_filter.depth",
            format!("must be 0.0-1.0, got {}", depth),
        ));
    }
    if !(100.0..=8000.0).contains(&base_frequency) {
        return Err(AudioError::invalid_param(
            "auto_filter.base_frequency",
            format!("must be 100-8000, got {}", base_frequency),
        ));
    }

    let num_samples = stereo.left.len();
    if num_samples == 0 {
        return Ok(());
    }

    // Convert time constants to coefficients
    let attack_coeff = (-1.0 / (attack_ms * 0.001 * sample_rate)).exp();
    let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();

    // Maximum cutoff frequency for sweep
    const MAX_FREQUENCY: f64 = 20000.0;

    // Use moderate resonance for musical response without instability
    const RESONANCE: f64 = 1.5;

    // Envelope state
    let mut envelope = 0.0;

    // Initialize filters for left and right channels
    let initial_coeffs = BiquadCoeffs::lowpass(base_frequency, RESONANCE, sample_rate);
    let mut filter_left = BiquadFilter::new(initial_coeffs);
    let mut filter_right = BiquadFilter::new(initial_coeffs);

    // Process each sample
    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // Calculate input level (peak detection)
        let input_level = in_left.abs().max(in_right.abs());

        // Envelope follower
        let target = input_level;
        if target > envelope {
            envelope = attack_coeff * envelope + (1.0 - attack_coeff) * target;
        } else {
            envelope = release_coeff * envelope + (1.0 - release_coeff) * target;
        }

        // Scale envelope by sensitivity and depth
        let modulation = envelope * sensitivity * depth;

        // Calculate cutoff frequency
        // Formula: cutoff = base_frequency + modulation * (max_freq - base_frequency)
        let cutoff = base_frequency + modulation * (MAX_FREQUENCY - base_frequency);
        let cutoff = cutoff.clamp(20.0, MAX_FREQUENCY);

        // Update filter coefficients with new cutoff
        let coeffs = BiquadCoeffs::lowpass(cutoff, RESONANCE, sample_rate);
        filter_left.set_coeffs(coeffs);
        filter_right.set_coeffs(coeffs);

        // Apply filter
        stereo.left[i] = filter_left.process(in_left);
        stereo.right[i] = filter_right.process(in_right);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stereo_buffer(len: usize, value: f64) -> StereoOutput {
        StereoOutput {
            left: vec![value; len],
            right: vec![value; len],
        }
    }

    #[test]
    fn test_auto_filter_processes() {
        let mut stereo = make_stereo_buffer(1000, 0.5);

        let result = apply(&mut stereo, 0.8, 5.0, 100.0, 0.7, 500.0, 44100.0);

        assert!(result.is_ok());
        // Output should be finite
        assert!(stereo.left[500].is_finite());
        assert!(stereo.right[500].is_finite());
    }

    #[test]
    fn test_auto_filter_with_varying_input() {
        // Create input with attack/decay pattern
        let mut stereo = StereoOutput {
            left: vec![0.0; 2000],
            right: vec![0.0; 2000],
        };

        // First half: ramp up
        for i in 0..1000 {
            let level = (i as f64 / 1000.0) * 0.8;
            stereo.left[i] = level;
            stereo.right[i] = level;
        }
        // Second half: ramp down
        for i in 1000..2000 {
            let level = ((2000 - i) as f64 / 1000.0) * 0.8;
            stereo.left[i] = level;
            stereo.right[i] = level;
        }

        let result = apply(&mut stereo, 1.0, 1.0, 50.0, 1.0, 200.0, 44100.0);

        assert!(result.is_ok());
        // All output should be finite
        for i in 0..2000 {
            assert!(stereo.left[i].is_finite());
            assert!(stereo.right[i].is_finite());
        }
    }

    #[test]
    fn test_auto_filter_zero_depth() {
        let mut stereo = make_stereo_buffer(1000, 0.5);
        let original_left = stereo.left.clone();

        // With depth = 0, filter should stay at base_frequency (no modulation)
        let result = apply(&mut stereo, 1.0, 5.0, 100.0, 0.0, 1000.0, 44100.0);

        assert!(result.is_ok());
        // Output should be filtered but not modulated
        // (constant lowpass at 1000 Hz)
        assert!(stereo.left[999].is_finite());
        // With constant input and constant filter, output should settle
        // to approximately the input value (lowpass passes DC)
        let last_sample = stereo.left[999];
        assert!(
            (last_sample - original_left[999]).abs() < 0.2,
            "Expected output near input, got {}",
            last_sample
        );
    }

    #[test]
    fn test_auto_filter_zero_sensitivity() {
        let mut stereo = make_stereo_buffer(1000, 0.5);

        // With sensitivity = 0, envelope has no effect
        let result = apply(&mut stereo, 0.0, 5.0, 100.0, 1.0, 1000.0, 44100.0);

        assert!(result.is_ok());
        assert!(stereo.left[999].is_finite());
    }

    #[test]
    fn test_auto_filter_param_validation() {
        let mut stereo = make_stereo_buffer(100, 0.5);

        // Invalid sensitivity
        assert!(apply(&mut stereo, -0.1, 5.0, 100.0, 0.5, 500.0, 44100.0).is_err());
        assert!(apply(&mut stereo, 1.5, 5.0, 100.0, 0.5, 500.0, 44100.0).is_err());

        // Invalid attack_ms
        assert!(apply(&mut stereo, 0.5, 0.05, 100.0, 0.5, 500.0, 44100.0).is_err());
        assert!(apply(&mut stereo, 0.5, 150.0, 100.0, 0.5, 500.0, 44100.0).is_err());

        // Invalid release_ms
        assert!(apply(&mut stereo, 0.5, 5.0, 5.0, 0.5, 500.0, 44100.0).is_err());
        assert!(apply(&mut stereo, 0.5, 5.0, 1500.0, 0.5, 500.0, 44100.0).is_err());

        // Invalid depth
        assert!(apply(&mut stereo, 0.5, 5.0, 100.0, -0.1, 500.0, 44100.0).is_err());
        assert!(apply(&mut stereo, 0.5, 5.0, 100.0, 1.5, 500.0, 44100.0).is_err());

        // Invalid base_frequency
        assert!(apply(&mut stereo, 0.5, 5.0, 100.0, 0.5, 50.0, 44100.0).is_err());
        assert!(apply(&mut stereo, 0.5, 5.0, 100.0, 0.5, 10000.0, 44100.0).is_err());
    }

    #[test]
    fn test_auto_filter_empty_buffer() {
        let mut stereo = StereoOutput {
            left: vec![],
            right: vec![],
        };

        let result = apply(&mut stereo, 0.8, 5.0, 100.0, 0.7, 500.0, 44100.0);

        assert!(result.is_ok());
    }

    #[test]
    fn test_auto_filter_stereo_independence() {
        // Left channel loud, right channel quiet
        let mut stereo = StereoOutput {
            left: vec![0.8; 1000],
            right: vec![0.1; 1000],
        };

        let result = apply(&mut stereo, 1.0, 1.0, 50.0, 1.0, 200.0, 44100.0);

        assert!(result.is_ok());
        // Both channels should be processed
        assert!(stereo.left[999].is_finite());
        assert!(stereo.right[999].is_finite());
    }
}
