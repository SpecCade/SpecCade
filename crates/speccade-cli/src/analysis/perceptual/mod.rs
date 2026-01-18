//! Perceptual comparison metrics for images and audio.
//!
//! This module provides deterministic implementations of:
//! - SSIM (Structural Similarity Index) for image comparison
//! - DeltaE (CIE76) for color difference in Lab space
//! - Histogram comparison for texture analysis
//! - Spectral similarity for audio comparison

mod audio;
mod color;
mod ssim;

#[cfg(test)]
mod tests;

use crate::analysis::audio::AudioMetrics;
use crate::analysis::texture::TextureMetrics;

// Re-export public items
pub use color::HistogramDiff;
pub use ssim::calculate_ssim;

// Re-export for internal use
pub use audio::calculate_spectral_correlation;
pub use color::{calculate_delta_e, calculate_histogram_diff};

/// Precision for floating point values in output.
const FLOAT_PRECISION: i32 = 6;

/// Round a float to the specified number of decimal places.
fn round_f64(value: f64, decimals: i32) -> f64 {
    let multiplier = 10_f64.powi(decimals);
    (value * multiplier).round() / multiplier
}

/// Result of image perceptual comparison.
#[derive(Debug, Clone)]
pub struct ImageCompareResult {
    /// SSIM (Structural Similarity Index), range [0, 1] where 1 = identical
    pub ssim: f64,
    /// Mean DeltaE (CIE76) color difference
    pub delta_e_mean: f64,
    /// Maximum DeltaE color difference
    pub delta_e_max: f64,
    /// Histogram difference per channel (absolute mean difference)
    pub histogram_diff: HistogramDiff,
}

/// Result of audio perceptual comparison.
#[derive(Debug, Clone)]
pub struct AudioCompareResult {
    /// Spectral centroid correlation coefficient [-1, 1]
    pub spectral_correlation: f64,
    /// RMS level difference in dB
    pub rms_delta_db: f64,
    /// Peak level difference in dB
    pub peak_delta_db: f64,
    /// Loudness (RMS) difference as percentage
    pub loudness_delta_percent: f64,
}

/// Compare two images and return perceptual metrics.
pub fn compare_images(
    pixels_a: &[u8],
    pixels_b: &[u8],
    width: u32,
    height: u32,
    channels: u8,
    metrics_a: &TextureMetrics,
    metrics_b: &TextureMetrics,
) -> ImageCompareResult {
    let ssim = calculate_ssim(pixels_a, pixels_b, width, height, channels);
    let (delta_e_mean, delta_e_max) =
        calculate_delta_e(pixels_a, pixels_b, width, height, channels);
    let histogram_diff = calculate_histogram_diff(metrics_a, metrics_b);

    ImageCompareResult {
        ssim,
        delta_e_mean,
        delta_e_max,
        histogram_diff,
    }
}

/// Compare two audio files and return perceptual metrics.
pub fn compare_audio(
    samples_a: &[f32],
    samples_b: &[f32],
    sample_rate: u32,
    metrics_a: &AudioMetrics,
    metrics_b: &AudioMetrics,
) -> AudioCompareResult {
    let spectral_correlation = calculate_spectral_correlation(samples_a, samples_b, sample_rate);

    let rms_delta_db = round_f64(
        metrics_a.quality.rms_db - metrics_b.quality.rms_db,
        FLOAT_PRECISION,
    );

    let peak_delta_db = round_f64(
        metrics_a.quality.peak_db - metrics_b.quality.peak_db,
        FLOAT_PRECISION,
    );

    // Calculate loudness delta as percentage
    // RMS in dB, convert to linear for percentage comparison
    let rms_a_linear = 10.0_f64.powf(metrics_a.quality.rms_db / 20.0);
    let rms_b_linear = 10.0_f64.powf(metrics_b.quality.rms_db / 20.0);

    let loudness_delta_percent = if rms_b_linear > 0.0 {
        round_f64(
            ((rms_a_linear - rms_b_linear) / rms_b_linear) * 100.0,
            FLOAT_PRECISION,
        )
    } else if rms_a_linear > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    AudioCompareResult {
        spectral_correlation,
        rms_delta_db,
        peak_delta_db,
        loudness_delta_percent,
    }
}
