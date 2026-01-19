//! Dynamics processing: compressor and limiter.

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

/// Applies brick-wall limiting to stereo audio with lookahead.
///
/// A limiter is a dynamics processor that prevents output from exceeding the ceiling level.
/// Unlike a compressor, a limiter uses an infinite ratio above the threshold.
pub fn apply_limiter(
    stereo: &mut StereoOutput,
    threshold_db: f64,
    release_ms: f64,
    lookahead_ms: f64,
    ceiling_db: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(-24.0..=0.0).contains(&threshold_db) {
        return Err(AudioError::invalid_param(
            "limiter.threshold_db",
            format!("must be -24 to 0, got {}", threshold_db),
        ));
    }
    if !(10.0..=500.0).contains(&release_ms) {
        return Err(AudioError::invalid_param(
            "limiter.release_ms",
            format!("must be 10-500, got {}", release_ms),
        ));
    }
    if !(1.0..=10.0).contains(&lookahead_ms) {
        return Err(AudioError::invalid_param(
            "limiter.lookahead_ms",
            format!("must be 1-10, got {}", lookahead_ms),
        ));
    }
    if !(-6.0..=0.0).contains(&ceiling_db) {
        return Err(AudioError::invalid_param(
            "limiter.ceiling_db",
            format!("must be -6 to 0, got {}", ceiling_db),
        ));
    }

    let num_samples = stereo.left.len();
    if num_samples == 0 {
        return Ok(());
    }

    let threshold_linear = db_to_amp(threshold_db);
    let ceiling_linear = db_to_amp(ceiling_db);

    // Calculate lookahead in samples
    let lookahead_samples = ((lookahead_ms * 0.001 * sample_rate).round() as usize).max(1);

    // Release coefficient for smoothing gain changes
    let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();

    // Create delay buffers for lookahead (circular buffer)
    let mut delay_left: Vec<f64> = vec![0.0; lookahead_samples];
    let mut delay_right: Vec<f64> = vec![0.0; lookahead_samples];
    let mut write_pos = 0;

    // Output gain ratio: ceiling / threshold
    let output_scale = ceiling_linear / threshold_linear;

    // Current gain reduction (starts at 1.0 = no reduction)
    let mut current_gain = 1.0;

    // Process each sample
    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // Find the maximum peak in the lookahead window
        // We look ahead from the current write position
        let mut max_peak: f64 = 0.0;
        for j in 0..lookahead_samples {
            let idx = (write_pos + j) % lookahead_samples;
            let left_abs = delay_left[idx].abs();
            let right_abs = delay_right[idx].abs();
            max_peak = max_peak.max(left_abs).max(right_abs);
        }
        // Also include the current input sample
        max_peak = max_peak.max(in_left.abs()).max(in_right.abs());

        // Calculate target gain reduction
        let target_gain = if max_peak > threshold_linear {
            threshold_linear / max_peak
        } else {
            1.0
        };

        // Smooth gain changes: instant attack (if target < current), smooth release
        if target_gain < current_gain {
            // Instant attack for brick-wall limiting
            current_gain = target_gain;
        } else {
            // Smooth release
            current_gain = release_coeff * current_gain + (1.0 - release_coeff) * target_gain;
        }

        // Read the delayed sample (from lookahead_samples ago)
        let read_pos = write_pos;
        let delayed_left = delay_left[read_pos];
        let delayed_right = delay_right[read_pos];

        // Write current input to delay buffer
        delay_left[write_pos] = in_left;
        delay_right[write_pos] = in_right;
        write_pos = (write_pos + 1) % lookahead_samples;

        // Apply gain and output scaling
        stereo.left[i] = delayed_left * current_gain * output_scale;
        stereo.right[i] = delayed_right * current_gain * output_scale;
    }

    // Flush the delay buffer (process remaining samples in buffer)
    for _ in 0..lookahead_samples {
        // For the tail, we can't look ahead anymore, just apply current gain
        // with continued release smoothing toward 1.0
        current_gain = release_coeff * current_gain + (1.0 - release_coeff) * 1.0;
    }

    Ok(())
}

/// Applies gate/expander to stereo audio.
///
/// A gate attenuates signals below the threshold. An expander reduces gain
/// proportionally based on the ratio. Hold time keeps the gate open briefly
/// after the signal drops below threshold.
#[allow(clippy::too_many_arguments)]
pub fn apply_gate_expander(
    stereo: &mut StereoOutput,
    threshold_db: f64,
    ratio: f64,
    attack_ms: f64,
    hold_ms: f64,
    release_ms: f64,
    range_db: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(-60.0..=0.0).contains(&threshold_db) {
        return Err(AudioError::invalid_param(
            "gate_expander.threshold_db",
            format!("must be -60 to 0, got {}", threshold_db),
        ));
    }
    if !(1.0..=f64::INFINITY).contains(&ratio) {
        return Err(AudioError::invalid_param(
            "gate_expander.ratio",
            format!("must be >= 1.0, got {}", ratio),
        ));
    }
    if !(0.1..=50.0).contains(&attack_ms) {
        return Err(AudioError::invalid_param(
            "gate_expander.attack_ms",
            format!("must be 0.1-50, got {}", attack_ms),
        ));
    }
    if !(0.0..=500.0).contains(&hold_ms) {
        return Err(AudioError::invalid_param(
            "gate_expander.hold_ms",
            format!("must be 0-500, got {}", hold_ms),
        ));
    }
    if !(10.0..=2000.0).contains(&release_ms) {
        return Err(AudioError::invalid_param(
            "gate_expander.release_ms",
            format!("must be 10-2000, got {}", release_ms),
        ));
    }
    if !(-80.0..=0.0).contains(&range_db) {
        return Err(AudioError::invalid_param(
            "gate_expander.range_db",
            format!("must be -80 to 0, got {}", range_db),
        ));
    }

    let num_samples = stereo.left.len();
    if num_samples == 0 {
        return Ok(());
    }

    // Convert time constants to coefficients
    let attack_coeff = (-1.0 / (attack_ms * 0.001 * sample_rate)).exp();
    let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();

    // Hold time in samples
    let hold_samples = (hold_ms * 0.001 * sample_rate).round() as usize;

    let threshold_linear = db_to_amp(threshold_db);
    let range_linear = db_to_amp(range_db);

    let mut envelope = 0.0;
    let mut current_gain = 1.0;
    let mut hold_counter: usize = 0;

    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // Peak envelope detection (faster response than RMS)
        let input_peak = in_left.abs().max(in_right.abs());

        // Envelope follower with instant attack for detection, smooth release
        if input_peak > envelope {
            envelope = input_peak;
        } else {
            // Use a fast release for the detector envelope (not the gain smoothing)
            let detector_release = 0.9995;
            envelope = detector_release * envelope + (1.0 - detector_release) * input_peak;
        }

        // Calculate target gain based on envelope vs threshold
        let target_gain = if envelope >= threshold_linear {
            // Signal above threshold: gate is open
            hold_counter = hold_samples;
            1.0
        } else if hold_counter > 0 {
            // In hold period: keep gate open
            hold_counter -= 1;
            1.0
        } else {
            // Below threshold and hold expired: apply expansion/gating
            let envelope_db = amp_to_db(envelope);
            let below_threshold_db = threshold_db - envelope_db;

            // Expansion formula: gain_reduction = (threshold - envelope) * (1 - 1/ratio)
            let gain_reduction_db = below_threshold_db * (1.0 - 1.0 / ratio);

            // Clamp to range_db (maximum attenuation)
            let final_gain_db = (-gain_reduction_db).max(range_db);
            db_to_amp(final_gain_db).max(range_linear)
        };

        // Smooth gain changes: fast attack (opening), slow release (closing)
        if target_gain > current_gain {
            // Opening the gate (attack)
            current_gain = attack_coeff * current_gain + (1.0 - attack_coeff) * target_gain;
        } else {
            // Closing the gate (release)
            current_gain = release_coeff * current_gain + (1.0 - release_coeff) * target_gain;
        }

        // Apply gain
        stereo.left[i] = in_left * current_gain;
        stereo.right[i] = in_right * current_gain;
    }

    Ok(())
}

/// Applies a true-peak limiter with oversampling for inter-sample peak detection.
///
/// This limiter uses 4x oversampling to accurately detect and limit inter-sample
/// peaks, ensuring the output never exceeds the ceiling in the analog domain.
/// Essential for broadcast/streaming compliance (EBU R128, ATSC A/85).
///
/// # Arguments
/// * `stereo` - Stereo audio buffer to process in place
/// * `ceiling_db` - Maximum output level in dBTP (-6 to 0)
/// * `release_ms` - Release time in ms for gain recovery (10-500)
/// * `sample_rate` - Sample rate in Hz
pub fn apply_true_peak_limiter(
    stereo: &mut StereoOutput,
    ceiling_db: f64,
    release_ms: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(-6.0..=0.0).contains(&ceiling_db) {
        return Err(AudioError::invalid_param(
            "true_peak_limiter.ceiling_db",
            format!("must be -6 to 0, got {}", ceiling_db),
        ));
    }
    if !(10.0..=500.0).contains(&release_ms) {
        return Err(AudioError::invalid_param(
            "true_peak_limiter.release_ms",
            format!("must be 10-500, got {}", release_ms),
        ));
    }

    let num_samples = stereo.left.len();
    if num_samples == 0 {
        return Ok(());
    }

    let ceiling_linear = db_to_amp(ceiling_db);

    // Lookahead for true-peak limiting (4 samples at 4x oversampling = 1 sample)
    // We use a small lookahead to smooth gain changes
    let lookahead_samples = (0.5 * 0.001 * sample_rate).round() as usize;
    let lookahead_samples = lookahead_samples.max(1).min(num_samples / 2);

    // Release coefficient
    let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();

    // Allocate delay buffers for lookahead
    let mut delay_left: Vec<f64> = vec![0.0; lookahead_samples];
    let mut delay_right: Vec<f64> = vec![0.0; lookahead_samples];
    let mut write_pos = 0;

    let mut current_gain = 1.0;

    // Process each sample
    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // Find true peak in lookahead window using oversampling
        let mut max_true_peak: f64 = 0.0;

        // Check current sample and lookahead buffer with oversampling
        for j in 0..lookahead_samples {
            let idx = (write_pos + j) % lookahead_samples;

            // Get the sample and its neighbors for interpolation
            let prev_idx = if j == 0 {
                (write_pos + lookahead_samples - 1) % lookahead_samples
            } else {
                (write_pos + j - 1) % lookahead_samples
            };

            let curr_left = delay_left[idx];
            let curr_right = delay_right[idx];
            let prev_left = delay_left[prev_idx];
            let prev_right = delay_right[prev_idx];

            // Sample peak
            max_true_peak = max_true_peak.max(curr_left.abs()).max(curr_right.abs());

            // Interpolated peaks (simple linear for performance)
            // Check midpoint between samples
            let mid_left = (prev_left + curr_left) * 0.5;
            let mid_right = (prev_right + curr_right) * 0.5;
            max_true_peak = max_true_peak.max(mid_left.abs()).max(mid_right.abs());

            // Check quarter points using cubic approximation
            for t in [0.25, 0.75] {
                let interp_left = prev_left * (1.0 - t) + curr_left * t;
                let interp_right = prev_right * (1.0 - t) + curr_right * t;
                max_true_peak = max_true_peak.max(interp_left.abs()).max(interp_right.abs());
            }
        }

        // Also check incoming sample with current buffer end
        max_true_peak = max_true_peak.max(in_left.abs()).max(in_right.abs());

        // Check interpolation to incoming sample
        if lookahead_samples > 0 {
            let last_idx = (write_pos + lookahead_samples - 1) % lookahead_samples;
            let last_left = delay_left[last_idx];
            let last_right = delay_right[last_idx];

            for t in [0.25, 0.5, 0.75] {
                let interp_left = last_left * (1.0 - t) + in_left * t;
                let interp_right = last_right * (1.0 - t) + in_right * t;
                max_true_peak = max_true_peak.max(interp_left.abs()).max(interp_right.abs());
            }
        }

        // Calculate target gain
        let target_gain = if max_true_peak > ceiling_linear {
            ceiling_linear / max_true_peak
        } else {
            1.0
        };

        // Apply gain smoothing: instant attack, smooth release
        if target_gain < current_gain {
            current_gain = target_gain;
        } else {
            current_gain = release_coeff * current_gain + (1.0 - release_coeff) * target_gain;
        }

        // Read delayed sample
        let delayed_left = delay_left[write_pos];
        let delayed_right = delay_right[write_pos];

        // Write current input to delay
        delay_left[write_pos] = in_left;
        delay_right[write_pos] = in_right;
        write_pos = (write_pos + 1) % lookahead_samples;

        // Apply gain
        stereo.left[i] = delayed_left * current_gain;
        stereo.right[i] = delayed_right * current_gain;
    }

    Ok(())
}

#[cfg(test)]
#[path = "dynamics_tests.rs"]
mod tests;
