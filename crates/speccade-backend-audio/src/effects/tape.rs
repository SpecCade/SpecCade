//! Tape saturation effect with warmth, wow/flutter, and hiss.

use crate::effects::delay_line::DelayLine;
use crate::error::AudioResult;
use crate::mixer::StereoOutput;
use crate::rng::{create_rng, derive_component_seed};
use rand::Rng;
use std::f64::consts::PI;

/// Maximum delay buffer size in ms for wow/flutter modulation.
const MAX_DELAY_MS: f64 = 20.0;

/// Wow modulation depth as fraction of max delay.
const WOW_DEPTH_MS: f64 = 3.0;

/// Flutter modulation depth as fraction of max delay.
const FLUTTER_DEPTH_MS: f64 = 0.5;

/// Applies tape saturation effect to stereo audio with deterministic hiss.
///
/// # Arguments
/// * `stereo` - Stereo audio to process
/// * `drive` - Saturation amount (clamped to 1.0-20.0)
/// * `bias` - DC bias before saturation (clamped to -0.5 to 0.5)
/// * `wow_rate` - Wow LFO rate in Hz (clamped to 0.0-3.0)
/// * `flutter_rate` - Flutter LFO rate in Hz (clamped to 0.0-20.0)
/// * `hiss_level` - Tape hiss amount (clamped to 0.0-0.1)
/// * `sample_rate` - Sample rate in Hz
/// * `seed` - RNG seed for deterministic hiss generation
#[allow(clippy::too_many_arguments)]
pub fn apply(
    stereo: &mut StereoOutput,
    drive: f64,
    bias: f64,
    wow_rate: f64,
    flutter_rate: f64,
    hiss_level: f64,
    sample_rate: f64,
    seed: u32,
) -> AudioResult<()> {
    // Create a constant drive curve for the non-modulated case
    let num_samples = stereo.left.len();
    let drive_curve = vec![drive; num_samples];
    apply_with_modulation(
        stereo,
        &drive_curve,
        bias,
        wow_rate,
        flutter_rate,
        hiss_level,
        sample_rate,
        seed,
    )
}

/// Applies tape saturation effect with per-sample drive modulation.
///
/// # Arguments
/// * `stereo` - Stereo audio to process
/// * `drive_curve` - Per-sample drive values (clamped to 1.0-20.0)
/// * `bias` - DC bias before saturation (clamped to -0.5 to 0.5)
/// * `wow_rate` - Wow LFO rate in Hz (clamped to 0.0-3.0)
/// * `flutter_rate` - Flutter LFO rate in Hz (clamped to 0.0-20.0)
/// * `hiss_level` - Tape hiss amount (clamped to 0.0-0.1)
/// * `sample_rate` - Sample rate in Hz
/// * `seed` - RNG seed for deterministic hiss generation
#[allow(clippy::too_many_arguments)]
pub fn apply_with_modulation(
    stereo: &mut StereoOutput,
    drive_curve: &[f64],
    bias: f64,
    wow_rate: f64,
    flutter_rate: f64,
    hiss_level: f64,
    sample_rate: f64,
    seed: u32,
) -> AudioResult<()> {
    // Clamp parameters
    let bias = bias.clamp(-0.5, 0.5);
    let wow_rate = wow_rate.clamp(0.0, 3.0);
    let flutter_rate = flutter_rate.clamp(0.0, 20.0);
    let hiss_level = hiss_level.clamp(0.0, 0.1);

    // Calculate delay line size for wow/flutter
    let max_delay_samples = ((MAX_DELAY_MS / 1000.0) * sample_rate).ceil() as usize + 4;
    let mut delay_left = DelayLine::new(max_delay_samples);
    let mut delay_right = DelayLine::new(max_delay_samples);

    // Pre-fill delay lines with half the max delay
    let base_delay_samples = max_delay_samples / 2;
    for _ in 0..base_delay_samples {
        delay_left.write(0.0);
        delay_right.write(0.0);
    }

    // Initialize deterministic RNG for hiss
    let hiss_seed = derive_component_seed(seed, "tape_saturation_hiss");
    let mut hiss_rng = create_rng(hiss_seed);

    // Process samples
    for i in 0..stereo.left.len() {
        let time = i as f64 / sample_rate;

        // Get modulated drive for this sample
        let drive = drive_curve.get(i).copied().unwrap_or(1.0).clamp(1.0, 20.0);

        // 1. Add bias and apply tape saturation
        let biased_left = stereo.left[i] + bias;
        let biased_right = stereo.right[i] + bias;

        let saturated_left = tape_saturation_curve(biased_left, drive);
        let saturated_right = tape_saturation_curve(biased_right, drive);

        // 2. Calculate wow/flutter modulation
        let wow_mod = if wow_rate > 0.0 {
            (2.0 * PI * wow_rate * time).sin() * WOW_DEPTH_MS
        } else {
            0.0
        };

        let flutter_mod = if flutter_rate > 0.0 {
            (2.0 * PI * flutter_rate * time).sin() * FLUTTER_DEPTH_MS
        } else {
            0.0
        };

        // Calculate modulated delay in samples
        let base_delay_ms = MAX_DELAY_MS / 2.0;
        let total_delay_ms = (base_delay_ms + wow_mod + flutter_mod).clamp(0.5, MAX_DELAY_MS);
        let delay_samples = (total_delay_ms / 1000.0) * sample_rate;

        // 3. Apply modulated delay (pitch variation via delay modulation)
        let modulated_left = delay_left.read_and_write(saturated_left, delay_samples);
        let modulated_right = delay_right.read_and_write(saturated_right, delay_samples);

        // 4. Add deterministic hiss
        let hiss_sample: f64 = hiss_rng.gen::<f64>() * 2.0 - 1.0;
        stereo.left[i] = modulated_left + hiss_sample * hiss_level;
        stereo.right[i] = modulated_right + hiss_sample * hiss_level;
    }

    Ok(())
}

/// Applies a tape-like saturation curve with asymmetric soft clipping.
///
/// This differs from standard tanh waveshaping by adding asymmetry and
/// a different harmonic profile that emulates analog tape saturation.
fn tape_saturation_curve(sample: f64, drive: f64) -> f64 {
    // Apply drive
    let driven = sample * drive;

    // Asymmetric soft clipping - positive and negative sides behave differently
    // This creates even harmonics typical of tape saturation
    let asymmetry = 0.1;
    let adjusted = driven + asymmetry * driven.abs();

    // Soft saturation using a modified sigmoid
    // This creates a warmer character than pure tanh
    let saturated = if adjusted >= 0.0 {
        // Positive side: gentle soft clip
        adjusted / (1.0 + adjusted.abs())
    } else {
        // Negative side: slightly harder clip
        adjusted / (1.0 + 1.2 * adjusted.abs())
    };

    // Apply makeup gain to compensate for drive
    let makeup = 1.0 / drive.sqrt();
    saturated * makeup
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tape_saturation_determinism() {
        let sample_rate = 44100.0;
        let seed = 42u32;

        // Create identical test signals
        let mut stereo1 = StereoOutput {
            left: vec![0.5; 1000],
            right: vec![-0.3; 1000],
        };
        let mut stereo2 = StereoOutput {
            left: vec![0.5; 1000],
            right: vec![-0.3; 1000],
        };

        // Apply effect with same seed
        apply(&mut stereo1, 3.0, 0.05, 1.0, 8.0, 0.02, sample_rate, seed).unwrap();

        apply(&mut stereo2, 3.0, 0.05, 1.0, 8.0, 0.02, sample_rate, seed).unwrap();

        // Output must be bit-identical
        assert_eq!(stereo1.left, stereo2.left);
        assert_eq!(stereo1.right, stereo2.right);
    }

    #[test]
    fn test_tape_saturation_different_seeds() {
        let sample_rate = 44100.0;

        let mut stereo1 = StereoOutput {
            left: vec![0.5; 1000],
            right: vec![-0.3; 1000],
        };
        let mut stereo2 = StereoOutput {
            left: vec![0.5; 1000],
            right: vec![-0.3; 1000],
        };

        // Apply effect with different seeds
        apply(&mut stereo1, 3.0, 0.05, 1.0, 8.0, 0.02, sample_rate, 42).unwrap();
        apply(&mut stereo2, 3.0, 0.05, 1.0, 8.0, 0.02, sample_rate, 43).unwrap();

        // Output should be different due to different hiss
        assert_ne!(stereo1.left, stereo2.left);
    }

    #[test]
    fn test_tape_saturation_curve_basic() {
        // Unity gain at drive=1 and small signals
        let small_signal = 0.1;
        let result = tape_saturation_curve(small_signal, 1.0);
        // Should be close to input for small signals
        assert!((result - small_signal).abs() < 0.02);

        // Saturation should limit large signals
        let large_signal = 10.0;
        let result = tape_saturation_curve(large_signal, 1.0);
        assert!(result.abs() < 1.0);
    }

    #[test]
    fn test_tape_saturation_asymmetry() {
        // Test that positive and negative sides behave differently
        let pos = tape_saturation_curve(0.5, 2.0);
        let neg = tape_saturation_curve(-0.5, 2.0);

        // Should not be perfectly symmetric
        assert!((pos.abs() - neg.abs()).abs() > 0.001);
    }

    #[test]
    fn test_tape_saturation_no_hiss() {
        let sample_rate = 44100.0;

        let mut stereo = StereoOutput {
            left: vec![0.5; 100],
            right: vec![-0.3; 100],
        };

        // Apply with zero hiss level
        apply(&mut stereo, 2.0, 0.0, 0.0, 0.0, 0.0, sample_rate, 42).unwrap();

        // Signal should still be processed (saturation applies)
        // but with no hiss, the output should be consistent
        assert!(!stereo.left.iter().all(|&x| x == 0.5));
    }

    #[test]
    fn test_tape_saturation_parameter_clamping() {
        let sample_rate = 44100.0;
        let seed = 42u32;

        let mut stereo = StereoOutput {
            left: vec![0.5; 100],
            right: vec![-0.3; 100],
        };

        // Use out-of-range parameters - should not panic
        apply(
            &mut stereo,
            100.0, // exceeds max drive of 20
            1.0,   // exceeds max bias of 0.5
            10.0,  // exceeds max wow_rate of 3.0
            50.0,  // exceeds max flutter_rate of 20.0
            0.5,   // exceeds max hiss_level of 0.1
            sample_rate,
            seed,
        )
        .unwrap();
    }

    #[test]
    fn test_tape_saturation_with_modulation() {
        let sample_rate = 44100.0;
        let seed = 42u32;

        let mut stereo = StereoOutput {
            left: vec![0.5; 1000],
            right: vec![-0.3; 1000],
        };

        // Create a drive curve that varies over time
        let drive_curve: Vec<f64> = (0..1000).map(|i| 1.0 + (i as f64 / 100.0)).collect();

        apply_with_modulation(
            &mut stereo,
            &drive_curve,
            0.05,
            1.0,
            8.0,
            0.02,
            sample_rate,
            seed,
        )
        .unwrap();

        // Output should vary (not be constant)
        let first_samples: Vec<f64> = stereo.left[..10].to_vec();
        let last_samples: Vec<f64> = stereo.left[990..].to_vec();
        assert_ne!(first_samples, last_samples);
    }
}
