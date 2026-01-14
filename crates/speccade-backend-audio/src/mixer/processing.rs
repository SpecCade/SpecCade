//! Audio processing functions for normalization and clipping.

use super::types::StereoOutput;

/// Normalizes audio to prevent clipping.
///
/// # Arguments
/// * `samples` - Audio samples to normalize
/// * `headroom_db` - Headroom in dB below 0 dBFS (e.g., -3.0 for -3dB headroom)
pub fn normalize(samples: &mut [f64], headroom_db: f64) {
    let target_peak = 10.0_f64.powf(headroom_db / 20.0);
    let current_peak = samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f64, |a, b| a.max(b));

    if current_peak > 0.0 {
        let gain = target_peak / current_peak;
        for sample in samples.iter_mut() {
            *sample *= gain;
        }
    }
}

/// Normalizes stereo audio.
pub fn normalize_stereo(stereo: &mut StereoOutput, headroom_db: f64) {
    let target_peak = 10.0_f64.powf(headroom_db / 20.0);

    let left_peak = stereo
        .left
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f64, |a, b| a.max(b));
    let right_peak = stereo
        .right
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f64, |a, b| a.max(b));
    let current_peak = left_peak.max(right_peak);

    if current_peak > 0.0 {
        let gain = target_peak / current_peak;
        for sample in stereo.left.iter_mut() {
            *sample *= gain;
        }
        for sample in stereo.right.iter_mut() {
            *sample *= gain;
        }
    }
}

/// Applies soft clipping to prevent harsh digital distortion.
///
/// # Arguments
/// * `sample` - Input sample
/// * `threshold` - Threshold above which soft clipping begins (0.0 to 1.0)
///
/// # Returns
/// Soft-clipped sample
#[inline]
pub fn soft_clip(sample: f64, threshold: f64) -> f64 {
    let abs = sample.abs();
    if abs <= threshold {
        sample
    } else {
        let sign = sample.signum();
        let excess = abs - threshold;
        let compressed = threshold + (1.0 - threshold) * (1.0 - (-excess * 3.0).exp());
        sign * compressed
    }
}

/// Applies soft clipping to a buffer.
pub fn soft_clip_buffer(samples: &mut [f64], threshold: f64) {
    for sample in samples.iter_mut() {
        *sample = soft_clip(*sample, threshold);
    }
}
