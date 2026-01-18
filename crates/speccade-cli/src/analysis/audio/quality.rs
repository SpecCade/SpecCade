//! Quality metric calculations for audio analysis.

/// Calculate peak amplitude.
pub(super) fn calculate_peak(samples: &[f32]) -> f32 {
    samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0f32, |max, s| max.max(s))
}

/// Calculate RMS (Root Mean Square).
pub(super) fn calculate_rms(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_of_squares: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_of_squares / samples.len() as f64).sqrt()
}

/// Detect clipping (samples at or near full scale).
pub(super) fn detect_clipping(samples: &[f32]) -> bool {
    const CLIPPING_THRESHOLD: f32 = 0.999;
    samples.iter().any(|&s| s.abs() >= CLIPPING_THRESHOLD)
}

/// Calculate DC offset (mean of samples).
pub(super) fn calculate_dc_offset(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples.iter().map(|&s| s as f64).sum();
    sum / samples.len() as f64
}

/// Calculate silence ratio (proportion of samples below threshold).
pub(super) fn calculate_silence_ratio(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return 1.0;
    }
    const SILENCE_THRESHOLD: f32 = 0.001;
    let silent_count = samples
        .iter()
        .filter(|&&s| s.abs() < SILENCE_THRESHOLD)
        .count();
    silent_count as f64 / samples.len() as f64
}
