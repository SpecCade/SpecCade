//! Dynamics processing: compressor.

use crate::error::{AudioError, AudioResult};
use crate::mixer::StereoOutput;

/// Converts linear amplitude to decibels.
fn amp_to_db(amp: f64) -> f64 {
    20.0 * amp.abs().max(1e-10).log10()
}

/// Converts decibels to linear amplitude.
fn db_to_amp(db: f64) -> f64 {
    10.0_f64.powf(db / 20.0)
}

/// Applies compression to stereo audio.
pub fn apply_compressor(
    stereo: &mut StereoOutput,
    threshold_db: f64,
    ratio: f64,
    attack_ms: f64,
    release_ms: f64,
    makeup_db: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(-60.0..=0.0).contains(&threshold_db) {
        return Err(AudioError::invalid_param(
            "compressor.threshold_db",
            format!("must be -60 to 0, got {}", threshold_db),
        ));
    }
    if !(1.0..=20.0).contains(&ratio) {
        return Err(AudioError::invalid_param(
            "compressor.ratio",
            format!("must be 1.0-20.0, got {}", ratio),
        ));
    }
    if !(0.1..=100.0).contains(&attack_ms) {
        return Err(AudioError::invalid_param(
            "compressor.attack_ms",
            format!("must be 0.1-100, got {}", attack_ms),
        ));
    }
    if !(10.0..=1000.0).contains(&release_ms) {
        return Err(AudioError::invalid_param(
            "compressor.release_ms",
            format!("must be 10-1000, got {}", release_ms),
        ));
    }

    // Convert time constants to coefficients
    let attack_coeff = (-1.0 / (attack_ms * 0.001 * sample_rate)).exp();
    let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();

    let makeup_gain = db_to_amp(makeup_db);

    let mut envelope = 0.0;

    let num_samples = stereo.left.len();

    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // Calculate input level (RMS of stereo)
        let input_level = ((in_left * in_left + in_right * in_right) / 2.0).sqrt();

        // Envelope follower
        let target = input_level;
        if target > envelope {
            envelope = attack_coeff * envelope + (1.0 - attack_coeff) * target;
        } else {
            envelope = release_coeff * envelope + (1.0 - release_coeff) * target;
        }

        let envelope_db = amp_to_db(envelope);

        // Calculate gain reduction
        let gain_db = if envelope_db > threshold_db {
            let over_db = envelope_db - threshold_db;
            let reduction = over_db * (1.0 - 1.0 / ratio);
            -reduction
        } else {
            0.0
        };

        let gain = db_to_amp(gain_db) * makeup_gain;

        // Apply gain
        stereo.left[i] = in_left * gain;
        stereo.right[i] = in_right * gain;
    }

    Ok(())
}
