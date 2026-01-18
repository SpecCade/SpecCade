//! Spectral metric calculations for audio analysis.

use rustfft::{num_complex::Complex, FftPlanner};

/// Calculate spectral centroid using FFT.
pub(super) fn calculate_spectral_centroid(samples: &[f32], sample_rate: u32) -> f64 {
    if samples.len() < 64 {
        return 0.0;
    }

    let fft_size = samples.len().next_power_of_two().min(4096);
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);

    let mut buffer: Vec<Complex<f32>> = samples
        .iter()
        .take(fft_size)
        .map(|&s| Complex::new(s, 0.0))
        .collect();

    // Zero-pad if necessary
    buffer.resize(fft_size, Complex::new(0.0, 0.0));

    // Apply Hann window
    for (i, sample) in buffer.iter_mut().enumerate() {
        let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / fft_size as f32).cos());
        *sample = Complex::new(sample.re * window, 0.0);
    }

    fft.process(&mut buffer);

    // Calculate magnitudes for positive frequencies only
    let nyquist = fft_size / 2;
    let freq_resolution = sample_rate as f64 / fft_size as f64;

    let mut weighted_sum: f64 = 0.0;
    let mut magnitude_sum: f64 = 0.0;

    for (i, c) in buffer.iter().take(nyquist).enumerate() {
        let magnitude = (c.re * c.re + c.im * c.im).sqrt() as f64;
        let frequency = i as f64 * freq_resolution;
        weighted_sum += frequency * magnitude;
        magnitude_sum += magnitude;
    }

    if magnitude_sum > 0.0 {
        weighted_sum / magnitude_sum
    } else {
        0.0
    }
}

/// Calculate dominant frequency using FFT.
pub(super) fn calculate_dominant_frequency(samples: &[f32], sample_rate: u32) -> f64 {
    if samples.len() < 64 {
        return 0.0;
    }

    let fft_size = samples.len().next_power_of_two().min(4096);
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);

    let mut buffer: Vec<Complex<f32>> = samples
        .iter()
        .take(fft_size)
        .map(|&s| Complex::new(s, 0.0))
        .collect();

    buffer.resize(fft_size, Complex::new(0.0, 0.0));

    // Apply Hann window
    for (i, sample) in buffer.iter_mut().enumerate() {
        let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / fft_size as f32).cos());
        *sample = Complex::new(sample.re * window, 0.0);
    }

    fft.process(&mut buffer);

    let nyquist = fft_size / 2;
    let freq_resolution = sample_rate as f64 / fft_size as f64;

    let mut max_magnitude: f32 = 0.0;
    let mut max_bin: usize = 0;

    // Skip DC component (bin 0) and very low frequencies
    let min_bin = (20.0 / freq_resolution).ceil() as usize;

    for (i, c) in buffer.iter().take(nyquist).enumerate().skip(min_bin) {
        let magnitude = (c.re * c.re + c.im * c.im).sqrt();
        if magnitude > max_magnitude {
            max_magnitude = magnitude;
            max_bin = i;
        }
    }

    max_bin as f64 * freq_resolution
}
