//! Transient shaper effect for controlling attack punch and sustain.
//!
//! Uses dual envelope followers with different time constants to detect
//! transients (fast envelope) versus sustained signal (slow envelope).

use crate::error::{AudioError, AudioResult};
use crate::mixer::StereoOutput;

/// Converts decibels to linear amplitude.
fn db_to_amp(db: f64) -> f64 {
    10.0_f64.powf(db / 20.0)
}

/// Applies transient shaping to stereo audio.
///
/// # Algorithm
/// Two envelope follower paths with different time constants:
/// 1. Fast envelope (attack detection) - 1ms attack, 50ms release
/// 2. Slow envelope (sustain detection) - 20ms attack, 200ms release
///
/// The difference between fast and slow envelopes detects transients.
/// Attack parameter scales transient gain, sustain parameter scales
/// the sustained portion.
///
/// # Arguments
/// * `stereo` - Stereo audio to process
/// * `attack` - Attack enhancement (-1.0 to 1.0)
/// * `sustain` - Sustain enhancement (-1.0 to 1.0)
/// * `output_gain_db` - Output makeup gain in dB (-12 to +12)
/// * `sample_rate` - Sample rate in Hz
pub fn apply(
    stereo: &mut StereoOutput,
    attack: f64,
    sustain: f64,
    output_gain_db: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(-1.0..=1.0).contains(&attack) {
        return Err(AudioError::invalid_param(
            "transient_shaper.attack",
            format!("must be -1.0 to 1.0, got {}", attack),
        ));
    }
    if !(-1.0..=1.0).contains(&sustain) {
        return Err(AudioError::invalid_param(
            "transient_shaper.sustain",
            format!("must be -1.0 to 1.0, got {}", sustain),
        ));
    }
    if !(-12.0..=12.0).contains(&output_gain_db) {
        return Err(AudioError::invalid_param(
            "transient_shaper.output_gain_db",
            format!("must be -12 to +12, got {}", output_gain_db),
        ));
    }

    let num_samples = stereo.left.len();
    if num_samples == 0 {
        return Ok(());
    }

    // Time constants for envelope followers
    // Fast envelope: 1ms attack, 50ms release (for transient detection)
    let fast_attack_ms = 1.0;
    let fast_release_ms = 50.0;
    // Slow envelope: 20ms attack, 200ms release (for sustain detection)
    let slow_attack_ms = 20.0;
    let slow_release_ms = 200.0;

    // Convert time constants to coefficients
    // coefficient = exp(-1 / (time_ms * 0.001 * sample_rate))
    let fast_attack_coeff = (-1.0 / (fast_attack_ms * 0.001 * sample_rate)).exp();
    let fast_release_coeff = (-1.0 / (fast_release_ms * 0.001 * sample_rate)).exp();
    let slow_attack_coeff = (-1.0 / (slow_attack_ms * 0.001 * sample_rate)).exp();
    let slow_release_coeff = (-1.0 / (slow_release_ms * 0.001 * sample_rate)).exp();

    let output_gain = db_to_amp(output_gain_db);

    let mut fast_env = 0.0;
    let mut slow_env = 0.0;

    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // Get the peak input level for this sample
        let input_level = in_left.abs().max(in_right.abs());

        // Fast envelope follower
        if input_level > fast_env {
            fast_env = fast_attack_coeff * fast_env + (1.0 - fast_attack_coeff) * input_level;
        } else {
            fast_env = fast_release_coeff * fast_env + (1.0 - fast_release_coeff) * input_level;
        }

        // Slow envelope follower
        if input_level > slow_env {
            slow_env = slow_attack_coeff * slow_env + (1.0 - slow_attack_coeff) * input_level;
        } else {
            slow_env = slow_release_coeff * slow_env + (1.0 - slow_release_coeff) * input_level;
        }

        // Transient detection: difference between fast and slow envelopes
        // Only positive transients (fast > slow means attack transient)
        let transient = (fast_env - slow_env).max(0.0);

        // Calculate gain modifiers
        // Attack gain: scale transients
        // attack = 1.0 means double the transient contribution
        // attack = -1.0 means remove transient contribution
        let attack_gain = 1.0 + attack * transient;

        // Sustain gain: scale based on slow envelope
        // sustain = 1.0 means boost sustained portions
        // sustain = -1.0 means reduce sustained portions
        let sustain_gain = 1.0 + sustain * slow_env;

        // Combined gain
        let total_gain = attack_gain * sustain_gain * output_gain;

        // Apply gain
        stereo.left[i] = in_left * total_gain;
        stereo.right[i] = in_right * total_gain;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transient_shaper_passthrough() {
        // With attack=0 and sustain=0, output should be approximately input * output_gain
        let mut stereo = StereoOutput {
            left: vec![0.5; 1000],
            right: vec![0.5; 1000],
        };

        apply(&mut stereo, 0.0, 0.0, 0.0, 44100.0).unwrap();

        // After envelope settling, gain should be close to 1.0
        // Check the last samples (after transient detection has settled)
        let last_left = stereo.left[999];
        assert!(
            (last_left - 0.5).abs() < 0.1,
            "Expected ~0.5, got {}",
            last_left
        );
    }

    #[test]
    fn test_transient_shaper_output_gain() {
        let mut stereo = StereoOutput {
            left: vec![0.5; 1000],
            right: vec![0.5; 1000],
        };

        // 6dB gain = ~2x amplitude
        apply(&mut stereo, 0.0, 0.0, 6.0, 44100.0).unwrap();

        // Check settled output (accounting for ~2x gain)
        let last_left = stereo.left[999];
        assert!(
            last_left > 0.8,
            "Expected amplified output, got {}",
            last_left
        );
    }

    #[test]
    fn test_transient_shaper_invalid_attack() {
        let mut stereo = StereoOutput {
            left: vec![0.5; 100],
            right: vec![0.5; 100],
        };

        let result = apply(&mut stereo, 1.5, 0.0, 0.0, 44100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_transient_shaper_invalid_sustain() {
        let mut stereo = StereoOutput {
            left: vec![0.5; 100],
            right: vec![0.5; 100],
        };

        let result = apply(&mut stereo, 0.0, -1.5, 0.0, 44100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_transient_shaper_invalid_output_gain() {
        let mut stereo = StereoOutput {
            left: vec![0.5; 100],
            right: vec![0.5; 100],
        };

        let result = apply(&mut stereo, 0.0, 0.0, 15.0, 44100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_transient_shaper_empty_input() {
        let mut stereo = StereoOutput {
            left: vec![],
            right: vec![],
        };

        let result = apply(&mut stereo, 0.5, 0.5, 0.0, 44100.0);
        assert!(result.is_ok());
    }
}
