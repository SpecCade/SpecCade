//! Audio embedding computation.
//!
//! Provides deterministic feature vector computation for audio assets.

use rustfft::{num_complex::Complex, FftPlanner};

/// Precision for floating point values in output (6 decimal places).
const FLOAT_PRECISION: i32 = 6;

/// Number of spectral bands for audio embedding.
const SPECTRAL_BANDS: usize = 16;

/// Number of envelope frames for audio embedding.
const ENVELOPE_FRAMES: usize = 16;

/// Number of additional spectral features.
const SPECTRAL_FEATURES: usize = 16;

/// Total audio embedding dimension.
pub const EMBEDDING_DIM: usize = SPECTRAL_BANDS + ENVELOPE_FRAMES + SPECTRAL_FEATURES;

/// Round a float to the specified number of decimal places.
fn round_f64(value: f64, decimals: i32) -> f64 {
    let multiplier = 10_f64.powi(decimals);
    (value * multiplier).round() / multiplier
}

/// Compute audio embedding from mono samples.
///
/// Returns a 48-dimension feature vector capturing:
/// - Spectral energy distribution (16 bands)
/// - Temporal RMS envelope (16 frames)
/// - Spectral shape features (16 values)
pub fn compute(samples: &[f32], sample_rate: u32) -> Vec<f64> {
    let mut embedding = Vec::with_capacity(EMBEDDING_DIM);

    // Compute spectral bands (16 dimensions)
    let spectral_bands = compute_spectral_bands(samples, sample_rate);
    embedding.extend(spectral_bands);

    // Compute RMS envelope (16 dimensions)
    let rms_envelope = compute_rms_envelope(samples);
    embedding.extend(rms_envelope);

    // Compute spectral features (16 dimensions)
    let spectral_features = compute_spectral_features(samples, sample_rate);
    embedding.extend(spectral_features);

    // Round all values for determinism
    embedding
        .iter()
        .map(|&v| round_f64(v, FLOAT_PRECISION))
        .collect()
}

/// Compute spectral energy in octave bands.
fn compute_spectral_bands(samples: &[f32], sample_rate: u32) -> Vec<f64> {
    if samples.len() < 64 {
        return vec![0.0; SPECTRAL_BANDS];
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

    // Compute magnitude spectrum
    let magnitudes: Vec<f64> = buffer
        .iter()
        .take(nyquist)
        .map(|c| (c.re * c.re + c.im * c.im).sqrt() as f64)
        .collect();

    // Define octave band edges (approximately 20Hz to 20kHz in 16 bands)
    let base_freq = 20.0;
    let max_freq = (sample_rate as f64 / 2.0).min(20000.0);
    let freq_ratio = (max_freq / base_freq).powf(1.0 / SPECTRAL_BANDS as f64);

    let mut bands = vec![0.0; SPECTRAL_BANDS];

    for (band_idx, band) in bands.iter_mut().enumerate() {
        let low_freq = base_freq * freq_ratio.powi(band_idx as i32);
        let high_freq = base_freq * freq_ratio.powi((band_idx + 1) as i32);

        let low_bin = ((low_freq / freq_resolution) as usize).max(1);
        let high_bin = ((high_freq / freq_resolution) as usize).min(nyquist);

        if low_bin < high_bin {
            let band_energy: f64 = magnitudes[low_bin..high_bin].iter().map(|m| m * m).sum();
            *band = band_energy.sqrt();
        }
    }

    // Normalize bands to [0, 1]
    let max_band = bands.iter().cloned().fold(0.0f64, f64::max);
    if max_band > 0.0 {
        for band in &mut bands {
            *band /= max_band;
        }
    }

    bands
}

/// Compute RMS envelope over time frames.
fn compute_rms_envelope(samples: &[f32]) -> Vec<f64> {
    if samples.is_empty() {
        return vec![0.0; ENVELOPE_FRAMES];
    }

    let frame_size = samples.len() / ENVELOPE_FRAMES;
    if frame_size == 0 {
        let rms = compute_frame_rms(samples);
        return vec![rms; ENVELOPE_FRAMES];
    }

    let mut envelope = Vec::with_capacity(ENVELOPE_FRAMES);
    for i in 0..ENVELOPE_FRAMES {
        let start = i * frame_size;
        let end = if i == ENVELOPE_FRAMES - 1 {
            samples.len()
        } else {
            (i + 1) * frame_size
        };
        let frame_rms = compute_frame_rms(&samples[start..end]);
        envelope.push(frame_rms);
    }

    // Normalize to [0, 1]
    let max_rms = envelope.iter().cloned().fold(0.0f64, f64::max);
    if max_rms > 0.0 {
        for rms in &mut envelope {
            *rms /= max_rms;
        }
    }

    envelope
}

/// Compute RMS for a frame of samples.
fn compute_frame_rms(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
    (sum_sq / samples.len() as f64).sqrt()
}

/// Compute additional spectral features.
fn compute_spectral_features(samples: &[f32], sample_rate: u32) -> Vec<f64> {
    if samples.len() < 64 {
        return vec![0.0; SPECTRAL_FEATURES];
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

    let magnitudes: Vec<f64> = buffer
        .iter()
        .take(nyquist)
        .map(|c| (c.re * c.re + c.im * c.im).sqrt() as f64)
        .collect();

    let total_energy: f64 = magnitudes.iter().map(|m| m * m).sum();
    let total_mag: f64 = magnitudes.iter().sum();

    let mut features = vec![0.0; SPECTRAL_FEATURES];

    // 0: Spectral centroid (normalized)
    if total_mag > 0.0 {
        let centroid: f64 = magnitudes
            .iter()
            .enumerate()
            .map(|(i, m)| i as f64 * m)
            .sum::<f64>()
            / total_mag;
        features[0] = centroid / nyquist as f64;
    }

    // 1: Spectral spread (normalized standard deviation)
    if total_mag > 0.0 {
        let centroid: f64 = magnitudes
            .iter()
            .enumerate()
            .map(|(i, m)| i as f64 * m)
            .sum::<f64>()
            / total_mag;
        let spread: f64 = magnitudes
            .iter()
            .enumerate()
            .map(|(i, m)| (i as f64 - centroid).powi(2) * m)
            .sum::<f64>()
            / total_mag;
        features[1] = spread.sqrt() / nyquist as f64;
    }

    // 2: Spectral rolloff (frequency below which 85% of energy is contained)
    if total_energy > 0.0 {
        let threshold = total_energy * 0.85;
        let mut cumulative = 0.0;
        for (i, m) in magnitudes.iter().enumerate() {
            cumulative += m * m;
            if cumulative >= threshold {
                features[2] = i as f64 / nyquist as f64;
                break;
            }
        }
    }

    // 3: Spectral flatness (geometric mean / arithmetic mean)
    if total_mag > 0.0 {
        let epsilon = 1e-10;
        let log_sum: f64 = magnitudes.iter().map(|m| (m + epsilon).ln()).sum();
        let geometric_mean = (log_sum / magnitudes.len() as f64).exp();
        let arithmetic_mean = total_mag / magnitudes.len() as f64;
        features[3] = (geometric_mean / arithmetic_mean).min(1.0);
    }

    // 4: Spectral crest (peak / RMS)
    if total_energy > 0.0 {
        let peak_mag = magnitudes.iter().cloned().fold(0.0f64, f64::max);
        let rms_mag = (total_energy / magnitudes.len() as f64).sqrt();
        features[4] = (peak_mag / rms_mag).min(10.0) / 10.0;
    }

    // 5: Zero crossing rate (normalized)
    let zcr = compute_zero_crossing_rate(samples);
    features[5] = zcr.min(0.5) * 2.0;

    // 6: Temporal peak (normalized)
    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    features[6] = peak as f64;

    // 7: Temporal RMS
    let temporal_rms = compute_frame_rms(samples);
    features[7] = temporal_rms.min(1.0);

    // 8: Crest factor (peak / RMS, normalized)
    if temporal_rms > 0.0 {
        features[8] = ((peak as f64 / temporal_rms).min(10.0)) / 10.0;
    }

    // 9: DC offset (absolute, normalized)
    let dc_offset: f64 = samples.iter().map(|&s| s as f64).sum::<f64>() / samples.len() as f64;
    features[9] = dc_offset.abs().min(1.0);

    // 10-11: Low/mid frequency energy ratios
    let low_cutoff = (250.0 / freq_resolution) as usize;
    let mid_cutoff = (4000.0 / freq_resolution) as usize;
    if total_energy > 0.0 {
        let low_energy: f64 = magnitudes
            .iter()
            .take(low_cutoff.min(nyquist))
            .map(|m| m * m)
            .sum();
        let mid_energy: f64 = magnitudes
            .iter()
            .skip(low_cutoff.min(nyquist))
            .take(mid_cutoff.saturating_sub(low_cutoff))
            .map(|m| m * m)
            .sum();
        features[10] = low_energy / total_energy;
        features[11] = mid_energy / total_energy;
    }

    // 12: High frequency energy ratio
    if total_energy > 0.0 {
        let high_energy: f64 = magnitudes
            .iter()
            .skip(mid_cutoff.min(nyquist))
            .map(|m| m * m)
            .sum();
        features[12] = high_energy / total_energy;
    }

    // 13: Spectral entropy
    if total_mag > 0.0 {
        let entropy: f64 = magnitudes
            .iter()
            .map(|m| {
                let p = m / total_mag;
                if p > 0.0 {
                    -p * p.ln()
                } else {
                    0.0
                }
            })
            .sum();
        let max_entropy = (magnitudes.len() as f64).ln();
        features[13] = if max_entropy > 0.0 {
            (entropy / max_entropy).min(1.0)
        } else {
            0.0
        };
    }

    // 14: Dominant frequency (normalized to Nyquist)
    if total_mag > 0.0 {
        let max_bin = magnitudes
            .iter()
            .enumerate()
            .skip(1)
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        features[14] = max_bin as f64 / nyquist as f64;
    }

    // 15: Bandwidth (width of spectral distribution around centroid)
    if total_mag > 0.0 {
        let centroid: f64 = magnitudes
            .iter()
            .enumerate()
            .map(|(i, m)| i as f64 * m)
            .sum::<f64>()
            / total_mag;
        let bandwidth: f64 = magnitudes
            .iter()
            .enumerate()
            .map(|(i, m)| (i as f64 - centroid).abs() * m)
            .sum::<f64>()
            / total_mag;
        features[15] = bandwidth / nyquist as f64;
    }

    features
}

/// Compute zero crossing rate.
fn compute_zero_crossing_rate(samples: &[f32]) -> f64 {
    if samples.len() < 2 {
        return 0.0;
    }
    let crossings: usize = samples
        .windows(2)
        .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
        .count();
    crossings as f64 / (samples.len() - 1) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_dimension() {
        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let embedding = compute(&samples, 44100);
        assert_eq!(embedding.len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_embedding_range() {
        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let embedding = compute(&samples, 44100);
        for (i, &v) in embedding.iter().enumerate() {
            assert!(
                (0.0..=1.0).contains(&v),
                "Embedding[{}] = {} out of range",
                i,
                v
            );
        }
    }

    #[test]
    fn test_embedding_deterministic() {
        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let e1 = compute(&samples, 44100);
        let e2 = compute(&samples, 44100);
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_embedding_empty() {
        let embedding = compute(&[], 44100);
        assert_eq!(embedding.len(), EMBEDDING_DIM);
        assert!(embedding.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_spectral_bands_normalization() {
        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin()).collect();
        let bands = compute_spectral_bands(&samples, 44100);
        assert_eq!(bands.len(), SPECTRAL_BANDS);
        let max_band = bands.iter().cloned().fold(0.0f64, f64::max);
        assert!(max_band <= 1.0);
    }

    #[test]
    fn test_rms_envelope_normalization() {
        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin()).collect();
        let envelope = compute_rms_envelope(&samples);
        assert_eq!(envelope.len(), ENVELOPE_FRAMES);
        let max_rms = envelope.iter().cloned().fold(0.0f64, f64::max);
        assert!(max_rms <= 1.0);
    }
}
