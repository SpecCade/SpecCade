//! Granular delay effect for shimmer and pitchy delays.
//!
//! This effect uses pitch-shifted grains read from a delay buffer to create
//! ethereal, shimmering delay textures. Each grain is windowed with a Hann
//! envelope and pitch-shifted via resampling.

use crate::effects::delay_line::DelayLine;
use crate::error::{AudioError, AudioResult};
use crate::mixer::StereoOutput;
use crate::rng::{create_rng, derive_component_seed};
use rand::Rng;
use std::f64::consts::PI;

/// Maximum supported delay time in milliseconds.
const MAX_DELAY_MS: f64 = 2000.0;

/// Minimum supported delay time in milliseconds.
const MIN_DELAY_MS: f64 = 10.0;

/// Minimum grain size in milliseconds.
const MIN_GRAIN_SIZE_MS: f64 = 10.0;

/// Maximum grain size in milliseconds.
const MAX_GRAIN_SIZE_MS: f64 = 200.0;

/// Maximum pitch shift in semitones.
const MAX_PITCH_SEMITONES: f64 = 24.0;

/// Jitter amount for grain start timing (fraction of grain interval).
const GRAIN_TIMING_JITTER: f64 = 0.2;

/// Applies granular delay effect to stereo audio.
///
/// # Arguments
/// * `stereo` - Stereo audio to process
/// * `time_ms` - Delay time in milliseconds (10-2000)
/// * `feedback` - Feedback amount (0.0-0.95)
/// * `grain_size_ms` - Grain window size in milliseconds (10-200)
/// * `pitch_semitones` - Pitch shift per grain pass in semitones (-24 to +24)
/// * `wet` - Wet/dry mix (0.0-1.0)
/// * `sample_rate` - Sample rate in Hz
/// * `seed` - RNG seed for deterministic grain timing jitter
#[allow(clippy::too_many_arguments)]
pub fn apply(
    stereo: &mut StereoOutput,
    time_ms: f64,
    feedback: f64,
    grain_size_ms: f64,
    pitch_semitones: f64,
    wet: f64,
    sample_rate: f64,
    seed: u32,
) -> AudioResult<()> {
    let num_samples = stereo.left.len();
    let time_curve = vec![time_ms; num_samples];
    apply_with_modulation(
        stereo,
        &time_curve,
        feedback,
        grain_size_ms,
        pitch_semitones,
        wet,
        sample_rate,
        seed,
    )
}

/// Applies granular delay effect with per-sample delay time modulation.
///
/// # Arguments
/// * `stereo` - Stereo audio to process
/// * `time_curve` - Per-sample delay time in milliseconds
/// * `feedback` - Feedback amount (0.0-0.95)
/// * `grain_size_ms` - Grain window size in milliseconds (10-200)
/// * `pitch_semitones` - Pitch shift per grain pass in semitones (-24 to +24)
/// * `wet` - Wet/dry mix (0.0-1.0)
/// * `sample_rate` - Sample rate in Hz
/// * `seed` - RNG seed for deterministic grain timing jitter
#[allow(clippy::too_many_arguments)]
pub fn apply_with_modulation(
    stereo: &mut StereoOutput,
    time_curve: &[f64],
    feedback: f64,
    grain_size_ms: f64,
    pitch_semitones: f64,
    wet: f64,
    sample_rate: f64,
    seed: u32,
) -> AudioResult<()> {
    // Validate and clamp parameters
    let feedback = feedback.clamp(0.0, 0.95);
    if !(0.0..=1.0).contains(&wet) {
        return Err(AudioError::invalid_param(
            "granular_delay.wet",
            format!("must be 0.0-1.0, got {}", wet),
        ));
    }

    let grain_size_ms = grain_size_ms.clamp(MIN_GRAIN_SIZE_MS, MAX_GRAIN_SIZE_MS);
    let pitch_semitones = pitch_semitones.clamp(-MAX_PITCH_SEMITONES, MAX_PITCH_SEMITONES);

    // Calculate pitch ratio from semitones
    let pitch_ratio = 2.0_f64.powf(pitch_semitones / 12.0);

    // Calculate buffer sizes
    let max_time_ms = time_curve
        .iter()
        .copied()
        .fold(MIN_DELAY_MS, |a, b| a.max(b))
        .clamp(MIN_DELAY_MS, MAX_DELAY_MS);
    let max_delay_samples = ((max_time_ms / 1000.0) * sample_rate).ceil() as usize;
    let grain_size_samples = ((grain_size_ms / 1000.0) * sample_rate).ceil() as usize;

    // Delay buffer needs extra room for grain reading with pitch shift
    let buffer_size = max_delay_samples + grain_size_samples * 2 + 4;
    let mut delay_left = DelayLine::new(buffer_size);
    let mut delay_right = DelayLine::new(buffer_size);

    // Initialize RNG for grain timing jitter
    let grain_seed = derive_component_seed(seed, "granular_delay");
    let mut rng = create_rng(grain_seed);

    // Pre-compute Hann window for grains
    let window = create_hann_window(grain_size_samples);

    // Grain scheduling state
    let grain_interval = grain_size_samples / 2; // 50% overlap
    let mut samples_until_grain = 0usize;

    // Active grains (we maintain 2 overlapping grains)
    let mut grains: Vec<ActiveGrain> = Vec::with_capacity(4);

    let num_samples = stereo.left.len();
    let mut output_left = Vec::with_capacity(num_samples);
    let mut output_right = Vec::with_capacity(num_samples);

    let dry = 1.0 - wet;

    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // Get modulated delay time for this sample
        let time_ms = time_curve.get(i).copied().unwrap_or(MIN_DELAY_MS);
        let time_ms = time_ms.clamp(MIN_DELAY_MS, MAX_DELAY_MS);
        let delay_samples = (time_ms / 1000.0) * sample_rate;

        // Sum output from all active grains
        let (mut grain_out_left, mut grain_out_right) = (0.0, 0.0);
        for grain in &mut grains {
            if grain.position < grain_size_samples {
                let win = window[grain.position];
                // Read from delay buffer with pitch-shifted position
                let read_pos = delay_samples + grain.read_offset;
                let left_sample = delay_left.read_interpolated(read_pos);
                let right_sample = delay_right.read_interpolated(read_pos);
                grain_out_left += left_sample * win;
                grain_out_right += right_sample * win;
                // Advance read position based on pitch ratio
                grain.read_offset += pitch_ratio;
                grain.position += 1;
            }
        }

        // Remove finished grains
        grains.retain(|g| g.position < grain_size_samples);

        // Schedule new grain if needed
        if samples_until_grain == 0 {
            // Apply jitter to grain start
            let jitter: f64 = rng.gen::<f64>() * 2.0 - 1.0;
            let jitter_samples = (jitter * GRAIN_TIMING_JITTER * grain_interval as f64) as i32;

            grains.push(ActiveGrain {
                position: 0,
                read_offset: jitter_samples.max(0) as f64,
            });

            samples_until_grain = grain_interval;
        }
        samples_until_grain = samples_until_grain.saturating_sub(1);

        // Write input + feedback to delay buffer
        let fb_left = in_left + grain_out_left * feedback;
        let fb_right = in_right + grain_out_right * feedback;
        delay_left.write(fb_left);
        delay_right.write(fb_right);

        // Mix wet/dry
        output_left.push(in_left * dry + grain_out_left * wet);
        output_right.push(in_right * dry + grain_out_right * wet);
    }

    stereo.left = output_left;
    stereo.right = output_right;

    Ok(())
}

/// Represents an active grain being processed.
struct ActiveGrain {
    /// Current sample position within the grain window.
    position: usize,
    /// Fractional read offset into the delay buffer (advances by pitch_ratio).
    read_offset: f64,
}

/// Creates a Hann window of the given size.
fn create_hann_window(size: usize) -> Vec<f64> {
    (0..size)
        .map(|i| {
            let phase = i as f64 / size as f64;
            0.5 * (1.0 - (2.0 * PI * phase).cos())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_granular_delay_determinism() {
        let sample_rate = 44100.0;
        let seed = 42u32;

        let mut stereo1 = StereoOutput {
            left: vec![0.5; 2000],
            right: vec![-0.3; 2000],
        };
        let mut stereo2 = StereoOutput {
            left: vec![0.5; 2000],
            right: vec![-0.3; 2000],
        };

        apply(&mut stereo1, 100.0, 0.5, 50.0, 12.0, 0.5, sample_rate, seed).unwrap();
        apply(&mut stereo2, 100.0, 0.5, 50.0, 12.0, 0.5, sample_rate, seed).unwrap();

        assert_eq!(stereo1.left, stereo2.left);
        assert_eq!(stereo1.right, stereo2.right);
    }

    #[test]
    fn test_granular_delay_different_seeds() {
        let sample_rate = 44100.0;
        let num_samples = 10000; // Longer duration to allow jitter to manifest

        // Use varied input so that jitter timing differences are visible
        let left_input: Vec<f64> = (0..num_samples)
            .map(|i| (i as f64 * 0.1).sin() * 0.5)
            .collect();
        let right_input: Vec<f64> = (0..num_samples)
            .map(|i| (i as f64 * 0.15).sin() * 0.3)
            .collect();

        let mut stereo1 = StereoOutput {
            left: left_input.clone(),
            right: right_input.clone(),
        };
        let mut stereo2 = StereoOutput {
            left: left_input,
            right: right_input,
        };

        apply(&mut stereo1, 100.0, 0.5, 50.0, 12.0, 0.5, sample_rate, 42).unwrap();
        apply(&mut stereo2, 100.0, 0.5, 50.0, 12.0, 0.5, sample_rate, 43).unwrap();

        // Different seeds produce different jitter patterns
        assert_ne!(stereo1.left, stereo2.left);
    }

    #[test]
    fn test_granular_delay_parameter_clamping() {
        let sample_rate = 44100.0;
        let seed = 42u32;

        let mut stereo = StereoOutput {
            left: vec![0.5; 1000],
            right: vec![-0.3; 1000],
        };

        // Use out-of-range parameters - should not panic
        apply(
            &mut stereo,
            3000.0, // exceeds max delay
            1.0,    // exceeds max feedback
            300.0,  // exceeds max grain size
            48.0,   // exceeds max pitch
            0.5,
            sample_rate,
            seed,
        )
        .unwrap();
    }

    #[test]
    fn test_granular_delay_with_modulation() {
        let sample_rate = 44100.0;
        let seed = 42u32;

        let mut stereo = StereoOutput {
            left: vec![0.5; 2000],
            right: vec![-0.3; 2000],
        };

        // Create a time curve that varies
        let time_curve: Vec<f64> = (0..2000).map(|i| 50.0 + (i as f64 / 20.0)).collect();

        apply_with_modulation(
            &mut stereo,
            &time_curve,
            0.5,
            50.0,
            12.0,
            0.5,
            sample_rate,
            seed,
        )
        .unwrap();

        // Output should be processed (not all zeros)
        assert!(stereo.left.iter().any(|&x| x != 0.0));
    }

    #[test]
    fn test_hann_window() {
        let window = create_hann_window(100);
        assert_eq!(window.len(), 100);

        // Window should start and end near zero
        assert!(window[0] < 0.01);
        assert!(window[99] < 0.01);

        // Window should peak in the middle
        assert!(window[50] > 0.99);
    }

    #[test]
    fn test_granular_delay_dry_wet() {
        let sample_rate = 44100.0;
        let seed = 42u32;

        // Test fully dry
        let mut stereo_dry = StereoOutput {
            left: vec![0.5; 1000],
            right: vec![-0.3; 1000],
        };
        apply(
            &mut stereo_dry,
            100.0,
            0.5,
            50.0,
            12.0,
            0.0, // fully dry
            sample_rate,
            seed,
        )
        .unwrap();

        // Dry signal should be unchanged
        assert!((stereo_dry.left[0] - 0.5).abs() < 1e-10);
        assert!((stereo_dry.right[0] - (-0.3)).abs() < 1e-10);
    }
}
