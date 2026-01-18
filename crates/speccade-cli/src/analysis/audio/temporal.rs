//! Temporal metric calculations for audio analysis.

use super::quality::calculate_peak;

/// Calculate zero crossing rate.
pub(super) fn calculate_zero_crossing_rate(samples: &[f32]) -> f64 {
    if samples.len() < 2 {
        return 0.0;
    }

    let crossings: usize = samples
        .windows(2)
        .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
        .count();

    crossings as f64 / (samples.len() - 1) as f64
}

/// Estimate attack time in milliseconds.
pub(super) fn calculate_attack_ms(samples: &[f32], sample_rate: u32) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }

    let peak = calculate_peak(samples);
    if peak < 0.001 {
        return 0.0;
    }

    let threshold_10 = peak * 0.1;
    let threshold_90 = peak * 0.9;

    let mut onset_sample: Option<usize> = None;
    let mut peak_sample: Option<usize> = None;

    for (i, &s) in samples.iter().enumerate() {
        if onset_sample.is_none() && s.abs() >= threshold_10 {
            onset_sample = Some(i);
        }
        if onset_sample.is_some() && s.abs() >= threshold_90 {
            peak_sample = Some(i);
            break;
        }
    }

    match (onset_sample, peak_sample) {
        (Some(onset), Some(peak)) if peak > onset => {
            let attack_samples = peak - onset;
            (attack_samples as f64 / sample_rate as f64) * 1000.0
        }
        _ => 0.0,
    }
}
