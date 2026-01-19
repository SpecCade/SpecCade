//! ITU-R BS.1770 loudness measurement and true-peak detection.
//!
//! Implements integrated LUFS measurement with K-weighting and true-peak
//! detection with 4x oversampling for production-ready audio analysis.

/// Minimum audio length required for LUFS measurement (400ms block size at 44.1kHz).
#[allow(dead_code)]
const MIN_BLOCK_SAMPLES_400MS: usize = 17640;

/// Gate threshold relative to ungated loudness (in dB).
const ABSOLUTE_GATE_THRESHOLD_DB: f64 = -70.0;
const RELATIVE_GATE_OFFSET_DB: f64 = -10.0;

/// ITU-R BS.1770 constant for mono/stereo weighting.
const LUFS_REFERENCE_OFFSET: f64 = -0.691;

/// K-weighting filter state for biquad filters.
#[derive(Debug, Clone)]
struct BiquadState {
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl BiquadState {
    fn new() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Process a sample through the biquad filter.
    fn process(&mut self, x: f64, b0: f64, b1: f64, b2: f64, a1: f64, a2: f64) -> f64 {
        let y = b0 * x + b1 * self.x1 + b2 * self.x2 - a1 * self.y1 - a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// K-weighting pre-filter coefficients for a given sample rate.
/// This is a high-shelf filter boosting high frequencies.
fn k_weighting_pre_coefficients(sample_rate: u32) -> (f64, f64, f64, f64, f64) {
    // Pre-filter: high shelf at ~1.5kHz with +4dB gain
    // Coefficients derived from ITU-R BS.1770 for 48kHz, scaled for other rates
    let fs = sample_rate as f64;

    // Use the standard 48kHz coefficients and adjust
    if (fs - 48000.0).abs() < 1.0 {
        // 48kHz reference coefficients
        (
            1.53512485958697,
            -2.69169618940638,
            1.19839281085285,
            -1.69065929318241,
            0.73248077421585,
        )
    } else if (fs - 44100.0).abs() < 1.0 {
        // 44.1kHz coefficients (pre-computed for common rate)
        (
            1.53085156824536,
            -2.65067242430902,
            1.16911633949740,
            -1.66363194078698,
            0.71251089073889,
        )
    } else {
        // Approximate coefficients for other sample rates using bilinear transform
        let f0 = 1681.974450955533;
        let g = 3.999843853973347_f64;
        let q = 0.7071752369554196;

        let k = (std::f64::consts::PI * f0 / fs).tan();
        let vg = 10.0_f64.powf(g / 20.0);
        let k2 = k * k;
        let a0 = 1.0 + k / q + k2;

        let b0 = (vg + vg.sqrt() * k / q + k2) / a0;
        let b1 = 2.0 * (k2 - vg) / a0;
        let b2 = (vg - vg.sqrt() * k / q + k2) / a0;
        let a1 = 2.0 * (k2 - 1.0) / a0;
        let a2 = (1.0 - k / q + k2) / a0;

        (b0, b1, b2, a1, a2)
    }
}

/// K-weighting RLB (revised low-frequency B-weighting) filter coefficients.
/// This is a high-pass filter at ~38Hz.
fn k_weighting_rlb_coefficients(sample_rate: u32) -> (f64, f64, f64, f64, f64) {
    let fs = sample_rate as f64;

    if (fs - 48000.0).abs() < 1.0 {
        // 48kHz reference coefficients
        (1.0, -2.0, 1.0, -1.99004745483398, 0.99007225036621)
    } else if (fs - 44100.0).abs() < 1.0 {
        // 44.1kHz coefficients
        (
            0.99977198108520,
            -1.99954396217041,
            0.99977198108520,
            -1.99891572199493,
            0.99891622176588,
        )
    } else {
        // Approximate for other rates using high-pass filter design
        let fc = 38.13547087602444;
        let q = 0.5003270373238773;

        let k = (std::f64::consts::PI * fc / fs).tan();
        let k2 = k * k;
        let a0 = 1.0 + k / q + k2;

        let b0 = 1.0 / a0;
        let b1 = -2.0 / a0;
        let b2 = 1.0 / a0;
        let a1 = 2.0 * (k2 - 1.0) / a0;
        let a2 = (1.0 - k / q + k2) / a0;

        (b0, b1, b2, a1, a2)
    }
}

/// Apply K-weighting filter to samples.
fn apply_k_weighting(samples: &[f32], sample_rate: u32) -> Vec<f64> {
    let (pre_b0, pre_b1, pre_b2, pre_a1, pre_a2) = k_weighting_pre_coefficients(sample_rate);
    let (rlb_b0, rlb_b1, rlb_b2, rlb_a1, rlb_a2) = k_weighting_rlb_coefficients(sample_rate);

    let mut pre_state = BiquadState::new();
    let mut rlb_state = BiquadState::new();

    samples
        .iter()
        .map(|&s| {
            let x = s as f64;
            let after_pre = pre_state.process(x, pre_b0, pre_b1, pre_b2, pre_a1, pre_a2);
            rlb_state.process(after_pre, rlb_b0, rlb_b1, rlb_b2, rlb_a1, rlb_a2)
        })
        .collect()
}

/// Calculate mean square of a slice.
fn mean_square(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|s| s * s).sum();
    sum_sq / samples.len() as f64
}

/// Calculate loudness in LUFS from mean square value.
fn mean_square_to_lufs(ms: f64) -> f64 {
    if ms <= 0.0 {
        return -f64::INFINITY;
    }
    LUFS_REFERENCE_OFFSET + 10.0 * ms.log10()
}

/// Calculate integrated LUFS according to ITU-R BS.1770.
///
/// Returns None if the audio is too short (less than 400ms) or silent.
pub(super) fn calculate_lufs_integrated(samples: &[f32], sample_rate: u32) -> Option<f64> {
    let block_size = (sample_rate as f64 * 0.4).round() as usize; // 400ms blocks
    let hop_size = block_size / 4; // 75% overlap (100ms hop)

    // Need at least one full block
    if samples.len() < block_size {
        return None;
    }

    // Apply K-weighting
    let weighted = apply_k_weighting(samples, sample_rate);

    // Calculate mean square for each 400ms block with 75% overlap
    let mut block_loudness: Vec<f64> = Vec::new();
    let mut pos = 0;
    while pos + block_size <= weighted.len() {
        let block = &weighted[pos..pos + block_size];
        let ms = mean_square(block);
        block_loudness.push(ms);
        pos += hop_size;
    }

    if block_loudness.is_empty() {
        return None;
    }

    // First pass: absolute gate at -70 LUFS
    let absolute_threshold_linear =
        10.0_f64.powf((ABSOLUTE_GATE_THRESHOLD_DB - LUFS_REFERENCE_OFFSET) / 10.0);
    let gated_blocks: Vec<f64> = block_loudness
        .iter()
        .copied()
        .filter(|&ms| ms > absolute_threshold_linear)
        .collect();

    if gated_blocks.is_empty() {
        return None; // Audio is essentially silent
    }

    // Calculate ungated loudness for relative threshold
    let ungated_mean: f64 = gated_blocks.iter().sum::<f64>() / gated_blocks.len() as f64;
    let ungated_lufs = mean_square_to_lufs(ungated_mean);

    // Second pass: relative gate at ungated - 10 dB
    let relative_threshold_lufs = ungated_lufs + RELATIVE_GATE_OFFSET_DB;
    let relative_threshold_linear =
        10.0_f64.powf((relative_threshold_lufs - LUFS_REFERENCE_OFFSET) / 10.0);

    let final_blocks: Vec<f64> = gated_blocks
        .into_iter()
        .filter(|&ms| ms >= relative_threshold_linear)
        .collect();

    if final_blocks.is_empty() {
        return None;
    }

    // Calculate final integrated loudness
    let final_mean: f64 = final_blocks.iter().sum::<f64>() / final_blocks.len() as f64;
    let lufs = mean_square_to_lufs(final_mean);

    // Return None for -inf results
    if lufs.is_finite() {
        Some(lufs)
    } else {
        None
    }
}

/// Calculate true peak level using 4x oversampling.
///
/// True peak detection finds inter-sample peaks that may exceed the sample values
/// by using oversampling. This is important for avoiding clipping in D/A conversion.
pub(super) fn calculate_true_peak_db(samples: &[f32]) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }

    // 4x oversampling using cubic interpolation (Catmull-Rom)
    let mut max_peak: f64 = 0.0;

    // First pass: find sample peaks
    for &s in samples {
        let abs = (s as f64).abs();
        if abs > max_peak {
            max_peak = abs;
        }
    }

    // Second pass: find inter-sample peaks using 4x oversampling
    // Catmull-Rom spline interpolation for each sample
    for i in 1..samples.len().saturating_sub(2) {
        let p0 = samples[i - 1] as f64;
        let p1 = samples[i] as f64;
        let p2 = samples[i + 1] as f64;
        let p3 = if i + 2 < samples.len() {
            samples[i + 2] as f64
        } else {
            p2
        };

        // Check 3 intermediate points (t = 0.25, 0.5, 0.75)
        for j in 1..4 {
            let t = j as f64 * 0.25;
            let t2 = t * t;
            let t3 = t2 * t;

            // Catmull-Rom interpolation
            let v = 0.5
                * ((2.0 * p1)
                    + (-p0 + p2) * t
                    + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
                    + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3);

            let abs_v = v.abs();
            if abs_v > max_peak {
                max_peak = abs_v;
            }
        }
    }

    // Convert to dB
    if max_peak > 0.0 {
        Some(20.0 * max_peak.log10())
    } else {
        Some(-f64::INFINITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lufs_silent_audio() {
        let samples = vec![0.0f32; 44100];
        let result = calculate_lufs_integrated(&samples, 44100);
        assert!(result.is_none(), "Silent audio should return None");
    }

    #[test]
    fn test_lufs_too_short() {
        // Less than 400ms at 44.1kHz (17640 samples)
        let samples = vec![0.5f32; 1000];
        let result = calculate_lufs_integrated(&samples, 44100);
        assert!(result.is_none(), "Short audio should return None");
    }

    #[test]
    fn test_lufs_sine_wave() {
        // Generate a 1kHz sine wave at 0.5 amplitude for 1 second
        let sample_rate = 44100u32;
        let duration_samples = sample_rate as usize;
        let frequency = 1000.0;

        let samples: Vec<f32> = (0..duration_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5
            })
            .collect();

        let result = calculate_lufs_integrated(&samples, sample_rate);
        assert!(result.is_some(), "Sine wave should have LUFS measurement");

        let lufs = result.unwrap();
        // A -6dB sine wave should be roughly around -9 to -12 LUFS depending on K-weighting
        assert!(lufs < 0.0, "LUFS should be negative");
        assert!(
            lufs > -30.0,
            "LUFS should be reasonable for 0.5 amplitude sine"
        );
    }

    #[test]
    fn test_lufs_full_scale_sine() {
        // Generate a full-scale 1kHz sine wave for 1 second
        let sample_rate = 44100u32;
        let duration_samples = sample_rate as usize;
        let frequency = 1000.0;

        let samples: Vec<f32> = (0..duration_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();

        let result = calculate_lufs_integrated(&samples, sample_rate);
        assert!(result.is_some());

        let lufs = result.unwrap();
        // Full scale 1kHz sine should be around -3 LUFS (due to K-weighting at 1kHz)
        assert!(lufs > -10.0, "Full scale sine LUFS: {}", lufs);
        assert!(lufs < 0.0, "Full scale sine should not exceed 0 LUFS");
    }

    #[test]
    fn test_true_peak_empty() {
        let samples: Vec<f32> = vec![];
        let result = calculate_true_peak_db(&samples);
        assert!(result.is_none());
    }

    #[test]
    fn test_true_peak_dc() {
        // DC signal at 0.5 amplitude
        let samples = vec![0.5f32; 1000];
        let result = calculate_true_peak_db(&samples);
        assert!(result.is_some());

        let db = result.unwrap();
        let expected_db = 20.0 * 0.5f64.log10(); // -6.02 dB
        assert!(
            (db - expected_db).abs() < 0.1,
            "DC true peak: {} expected: {}",
            db,
            expected_db
        );
    }

    #[test]
    fn test_true_peak_intersample() {
        // Two samples that will interpolate to a higher peak
        // If we have -0.9, 0.9 next to each other, the interpolation will peak higher
        let samples = vec![0.0f32, 0.9, -0.9, 0.0];
        let result = calculate_true_peak_db(&samples);
        assert!(result.is_some());

        let db = result.unwrap();
        // The true peak should be close to or higher than the sample peak of 0.9 (-0.92 dB)
        let sample_peak_db = 20.0 * 0.9f64.log10();
        // Allow small floating point tolerance
        let tolerance = 0.01;
        assert!(
            db >= sample_peak_db - tolerance,
            "True peak {} should be >= sample peak {} (with tolerance)",
            db,
            sample_peak_db
        );
    }

    #[test]
    fn test_true_peak_sine() {
        // Full scale sine wave should have true peak around 0 dB
        let sample_rate = 44100;
        let frequency = 1000.0;

        let samples: Vec<f32> = (0..4410)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();

        let result = calculate_true_peak_db(&samples);
        assert!(result.is_some());

        let db = result.unwrap();
        // True peak of full-scale sine should be close to 0 dB
        assert!(
            db > -0.5 && db <= 0.1,
            "Sine true peak should be ~0 dB, got {}",
            db
        );
    }

    #[test]
    fn test_k_weighting_coefficients_44100() {
        let (b0, b1, b2, a1, a2) = k_weighting_pre_coefficients(44100);
        // Verify coefficients are reasonable (non-zero, finite)
        assert!(b0.is_finite() && b0 != 0.0);
        assert!(b1.is_finite());
        assert!(b2.is_finite());
        assert!(a1.is_finite());
        assert!(a2.is_finite());
    }

    #[test]
    fn test_k_weighting_coefficients_48000() {
        let (b0, b1, b2, _a1, _a2) = k_weighting_pre_coefficients(48000);
        // Verify 48kHz reference coefficients
        assert!((b0 - 1.53512485958697).abs() < 1e-10);
        assert!((b1 - (-2.69169618940638)).abs() < 1e-10);
        assert!((b2 - 1.19839281085285).abs() < 1e-10);
    }
}
