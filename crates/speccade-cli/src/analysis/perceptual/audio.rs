//! Audio perceptual comparison metrics.
//!
//! Provides spectral correlation for audio comparison.

use super::{round_f64, FLOAT_PRECISION};
use rustfft::{num_complex::Complex, FftPlanner};

/// Calculate spectral similarity between two audio files using centroid correlation.
///
/// Returns a correlation coefficient in the range [-1, 1], where:
/// - 1.0 = perfectly correlated spectral content
/// - 0.0 = no correlation
/// - -1.0 = inversely correlated
pub fn calculate_spectral_correlation(
    samples_a: &[f32],
    samples_b: &[f32],
    sample_rate: u32,
) -> f64 {
    // Compute spectral centroids over time windows
    let window_size = 2048;
    let hop_size = 512;

    let centroids_a = compute_centroid_sequence(samples_a, sample_rate, window_size, hop_size);
    let centroids_b = compute_centroid_sequence(samples_b, sample_rate, window_size, hop_size);

    // Use the shorter sequence length
    let len = centroids_a.len().min(centroids_b.len());
    if len < 2 {
        return 0.0;
    }

    let a = &centroids_a[..len];
    let b = &centroids_b[..len];

    // Compute Pearson correlation coefficient
    let mean_a: f64 = a.iter().sum::<f64>() / len as f64;
    let mean_b: f64 = b.iter().sum::<f64>() / len as f64;

    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;

    for i in 0..len {
        let diff_a = a[i] - mean_a;
        let diff_b = b[i] - mean_b;
        cov += diff_a * diff_b;
        var_a += diff_a * diff_a;
        var_b += diff_b * diff_b;
    }

    let std_a = var_a.sqrt();
    let std_b = var_b.sqrt();

    if std_a > 0.0 && std_b > 0.0 {
        round_f64(cov / (std_a * std_b), FLOAT_PRECISION)
    } else {
        1.0 // Both are constant = identical
    }
}

/// Compute spectral centroid sequence over time windows.
fn compute_centroid_sequence(
    samples: &[f32],
    sample_rate: u32,
    window_size: usize,
    hop_size: usize,
) -> Vec<f64> {
    if samples.len() < window_size {
        return vec![];
    }

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(window_size);

    let num_frames = (samples.len() - window_size) / hop_size + 1;
    let mut centroids = Vec::with_capacity(num_frames);

    let freq_resolution = sample_rate as f64 / window_size as f64;

    for frame_idx in 0..num_frames {
        let start = frame_idx * hop_size;
        let window_samples = &samples[start..start + window_size];

        // Apply Hann window and compute FFT
        let mut buffer: Vec<Complex<f32>> = window_samples
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let window = 0.5
                    * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / window_size as f32).cos());
                Complex::new(s * window, 0.0)
            })
            .collect();

        fft.process(&mut buffer);

        // Calculate spectral centroid
        let nyquist = window_size / 2;
        let mut weighted_sum: f64 = 0.0;
        let mut magnitude_sum: f64 = 0.0;

        for (i, c) in buffer.iter().take(nyquist).enumerate() {
            let magnitude = (c.re * c.re + c.im * c.im).sqrt() as f64;
            let frequency = i as f64 * freq_resolution;
            weighted_sum += frequency * magnitude;
            magnitude_sum += magnitude;
        }

        let centroid = if magnitude_sum > 0.0 {
            weighted_sum / magnitude_sum
        } else {
            0.0
        };

        centroids.push(centroid);
    }

    centroids
}
