//! Multi-tap delay effect with independent delay lines per tap.

use speccade_spec::recipe::audio::DelayTap;

use crate::error::{AudioError, AudioResult};
use crate::mixer::StereoOutput;

use super::delay_line::DelayLine;

/// State for a single delay tap with its own delay line and filter.
struct TapState {
    delay_line: DelayLine,
    filter_state: f64,
}

impl TapState {
    fn new(max_samples: usize) -> Self {
        Self {
            delay_line: DelayLine::new(max_samples),
            filter_state: 0.0,
        }
    }
}

/// Applies a simple one-pole lowpass filter.
fn apply_lowpass(input: f64, state: &mut f64, alpha: f64) -> f64 {
    // One-pole lowpass: y[n] = alpha * x[n] + (1 - alpha) * y[n-1]
    *state = alpha * input + (1.0 - alpha) * *state;
    *state
}

/// Calculates the lowpass filter coefficient from cutoff frequency.
fn calculate_alpha(cutoff: f64, sample_rate: f64) -> f64 {
    if cutoff <= 0.0 || cutoff >= sample_rate * 0.5 {
        // No filtering needed
        1.0
    } else {
        // Simple approximation for one-pole lowpass
        let rc = 1.0 / (2.0 * std::f64::consts::PI * cutoff);
        let dt = 1.0 / sample_rate;
        dt / (rc + dt)
    }
}

/// Applies multi-tap delay effect to stereo audio.
///
/// # Arguments
/// * `stereo` - Stereo audio to process
/// * `taps` - Slice of delay tap configurations
/// * `sample_rate` - Sample rate in Hz
///
/// # Returns
/// Result indicating success or error
pub fn apply(stereo: &mut StereoOutput, taps: &[DelayTap], sample_rate: f64) -> AudioResult<()> {
    if taps.is_empty() {
        return Ok(());
    }

    // Validate taps
    for (i, tap) in taps.iter().enumerate() {
        if !(1.0..=2000.0).contains(&tap.time_ms) {
            return Err(AudioError::invalid_param(
                format!("multi_tap_delay.taps[{}].time_ms", i),
                format!("must be 1-2000 ms, got {}", tap.time_ms),
            ));
        }
        if !(0.0..=0.99).contains(&tap.feedback) {
            return Err(AudioError::invalid_param(
                format!("multi_tap_delay.taps[{}].feedback", i),
                format!("must be 0.0-0.99, got {}", tap.feedback),
            ));
        }
        if !(-1.0..=1.0).contains(&tap.pan) {
            return Err(AudioError::invalid_param(
                format!("multi_tap_delay.taps[{}].pan", i),
                format!("must be -1.0 to 1.0, got {}", tap.pan),
            ));
        }
        if !(0.0..=1.0).contains(&tap.level) {
            return Err(AudioError::invalid_param(
                format!("multi_tap_delay.taps[{}].level", i),
                format!("must be 0.0-1.0, got {}", tap.level),
            ));
        }
    }

    // Create a constant time curve for non-modulated case
    let num_samples = stereo.left.len();
    let time_curves: Vec<Vec<f64>> = taps
        .iter()
        .map(|tap| vec![tap.time_ms; num_samples])
        .collect();

    apply_with_modulation(stereo, taps, &time_curves, sample_rate)
}

/// Applies multi-tap delay effect with per-sample time modulation.
///
/// # Arguments
/// * `stereo` - Stereo audio to process
/// * `taps` - Slice of delay tap configurations
/// * `time_curves` - Per-tap per-sample delay time curves in milliseconds
/// * `sample_rate` - Sample rate in Hz
///
/// # Returns
/// Result indicating success or error
pub fn apply_with_modulation(
    stereo: &mut StereoOutput,
    taps: &[DelayTap],
    time_curves: &[Vec<f64>],
    sample_rate: f64,
) -> AudioResult<()> {
    if taps.is_empty() {
        return Ok(());
    }

    let num_samples = stereo.left.len();

    // Find max delay time across all taps for buffer sizing
    let max_time_ms = time_curves
        .iter()
        .flat_map(|curve| curve.iter().copied())
        .fold(1.0_f64, |a, b| a.max(b))
        .clamp(1.0, 2000.0);
    let max_delay_samples = (max_time_ms / 1000.0) * sample_rate;
    let delay_buffer_size = (max_delay_samples.ceil() as usize + 2).max(4);

    // Initialize tap states (stable order)
    let mut tap_states: Vec<TapState> = taps
        .iter()
        .map(|_| TapState::new(delay_buffer_size))
        .collect();

    // Pre-calculate filter coefficients for each tap
    let filter_alphas: Vec<f64> = taps
        .iter()
        .map(|tap| calculate_alpha(tap.filter_cutoff, sample_rate))
        .collect();

    let mut output_left = Vec::with_capacity(num_samples);
    let mut output_right = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // Mix input to mono for delay processing
        let in_mono = (in_left + in_right) * 0.5;

        let mut wet_left = 0.0;
        let mut wet_right = 0.0;

        // Process each tap (stable iteration order)
        for (tap_idx, tap) in taps.iter().enumerate() {
            let state = &mut tap_states[tap_idx];
            let alpha = filter_alphas[tap_idx];

            // Get modulated delay time for this tap at this sample
            let time_ms: f64 = time_curves
                .get(tap_idx)
                .and_then(|curve: &Vec<f64>| curve.get(i).copied())
                .unwrap_or(tap.time_ms)
                .clamp(1.0, 2000.0);
            let delay_samples = (time_ms / 1000.0) * sample_rate;

            // Read from delay line
            let delayed = state.delay_line.read_interpolated(delay_samples);

            // Apply lowpass filter if needed
            let filtered = if alpha < 1.0 {
                apply_lowpass(delayed, &mut state.filter_state, alpha)
            } else {
                delayed
            };

            // Write input + feedback to delay line
            let feedback_sample = in_mono + filtered * tap.feedback;
            state.delay_line.write(feedback_sample);

            // Apply level and pan
            let tap_out = filtered * tap.level;

            // Constant power panning
            let pan: f64 = tap.pan.clamp(-1.0, 1.0);
            let left_gain = ((1.0 - pan) * 0.5_f64).sqrt();
            let right_gain = ((1.0 + pan) * 0.5_f64).sqrt();

            wet_left += tap_out * left_gain;
            wet_right += tap_out * right_gain;
        }

        // Mix dry and wet (50/50 default - could be made configurable)
        output_left.push(in_left + wet_left);
        output_right.push(in_right + wet_right);
    }

    stereo.left = output_left;
    stereo.right = output_right;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stereo(samples: usize) -> StereoOutput {
        let mut left = vec![0.0; samples];
        let mut right = vec![0.0; samples];
        // Put an impulse at the start
        if samples > 0 {
            left[0] = 1.0;
            right[0] = 1.0;
        }
        StereoOutput { left, right }
    }

    #[test]
    fn test_multi_tap_delay_empty_taps() {
        let mut stereo = make_stereo(100);
        let original_left = stereo.left.clone();
        let original_right = stereo.right.clone();

        apply(&mut stereo, &[], 44100.0).unwrap();

        assert_eq!(stereo.left, original_left);
        assert_eq!(stereo.right, original_right);
    }

    #[test]
    fn test_multi_tap_delay_single_tap() {
        let mut stereo = make_stereo(4410);
        let taps = vec![DelayTap {
            time_ms: 50.0,
            feedback: 0.0,
            pan: 0.0,
            level: 1.0,
            filter_cutoff: 0.0,
        }];

        apply(&mut stereo, &taps, 44100.0).unwrap();

        // Check that there's output at the expected delay position
        let delay_samples = (50.0 / 1000.0 * 44100.0) as usize;
        assert!(stereo.left[delay_samples].abs() > 0.1);
    }

    #[test]
    fn test_multi_tap_delay_panning() {
        let mut stereo = make_stereo(8820); // 200ms at 44100Hz
        let taps = vec![
            DelayTap {
                time_ms: 50.0,
                feedback: 0.0,
                pan: -1.0, // Full left
                level: 1.0,
                filter_cutoff: 0.0,
            },
            DelayTap {
                time_ms: 100.0,
                feedback: 0.0,
                pan: 1.0, // Full right
                level: 1.0,
                filter_cutoff: 0.0,
            },
        ];

        apply(&mut stereo, &taps, 44100.0).unwrap();

        let delay_samples_1 = (50.0 / 1000.0 * 44100.0) as usize;
        let delay_samples_2 = (100.0 / 1000.0 * 44100.0) as usize;

        // First tap should be mostly left
        assert!(stereo.left[delay_samples_1].abs() > stereo.right[delay_samples_1].abs());

        // Second tap should be mostly right
        assert!(stereo.right[delay_samples_2].abs() > stereo.left[delay_samples_2].abs());
    }

    #[test]
    fn test_multi_tap_delay_validation() {
        let mut stereo = make_stereo(100);

        // Invalid time_ms
        let result = apply(
            &mut stereo,
            &[DelayTap {
                time_ms: 3000.0,
                feedback: 0.0,
                pan: 0.0,
                level: 1.0,
                filter_cutoff: 0.0,
            }],
            44100.0,
        );
        assert!(result.is_err());

        // Invalid feedback
        let result = apply(
            &mut stereo,
            &[DelayTap {
                time_ms: 100.0,
                feedback: 1.5,
                pan: 0.0,
                level: 1.0,
                filter_cutoff: 0.0,
            }],
            44100.0,
        );
        assert!(result.is_err());

        // Invalid pan
        let result = apply(
            &mut stereo,
            &[DelayTap {
                time_ms: 100.0,
                feedback: 0.5,
                pan: 2.0,
                level: 1.0,
                filter_cutoff: 0.0,
            }],
            44100.0,
        );
        assert!(result.is_err());

        // Invalid level
        let result = apply(
            &mut stereo,
            &[DelayTap {
                time_ms: 100.0,
                feedback: 0.5,
                pan: 0.0,
                level: 1.5,
                filter_cutoff: 0.0,
            }],
            44100.0,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_multi_tap_delay_deterministic() {
        let taps = vec![
            DelayTap {
                time_ms: 100.0,
                feedback: 0.3,
                pan: -0.5,
                level: 0.8,
                filter_cutoff: 2000.0,
            },
            DelayTap {
                time_ms: 200.0,
                feedback: 0.2,
                pan: 0.5,
                level: 0.6,
                filter_cutoff: 0.0,
            },
        ];

        let mut stereo1 = make_stereo(4410);
        let mut stereo2 = make_stereo(4410);

        apply(&mut stereo1, &taps, 44100.0).unwrap();
        apply(&mut stereo2, &taps, 44100.0).unwrap();

        // Should produce identical output
        assert_eq!(stereo1.left, stereo2.left);
        assert_eq!(stereo1.right, stereo2.right);
    }
}
