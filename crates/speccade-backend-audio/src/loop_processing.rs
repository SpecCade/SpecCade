//! Loop point detection and crossfade processing.
//!
//! This module provides algorithms for finding optimal loop points in audio samples
//! and applying crossfades at loop boundaries to eliminate clicks.

use speccade_spec::recipe::audio::{Envelope, LoopConfig};

/// Result of loop point calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct LoopPoints {
    /// Start sample of the loop region.
    pub start: usize,
    /// End sample of the loop region.
    pub end: usize,
    /// Whether zero crossing snapping was applied.
    pub snapped_to_zero_crossing: bool,
}

/// Finds the nearest zero crossing point to the target sample.
///
/// A zero crossing occurs when the signal changes sign (crosses from positive
/// to negative or vice versa). Finding zero crossings helps eliminate clicks
/// at loop boundaries because the waveform value is near zero.
///
/// # Arguments
/// * `samples` - Audio sample buffer
/// * `target` - Target sample index to search around
/// * `tolerance` - Maximum samples to search in each direction
///
/// # Returns
/// The sample index of the nearest zero crossing, or the target if none found.
pub fn find_nearest_zero_crossing(samples: &[f64], target: usize, tolerance: usize) -> usize {
    if samples.is_empty() || target >= samples.len() {
        return target;
    }

    let search_start = target.saturating_sub(tolerance);
    let search_end = (target + tolerance).min(samples.len().saturating_sub(1));

    let mut best_idx = target;
    let mut best_distance = usize::MAX;
    let mut best_magnitude = f64::MAX;

    for i in search_start..search_end {
        // Check for zero crossing between samples[i] and samples[i+1]
        let current = samples[i];
        let next = samples[i + 1];

        // Zero crossing occurs when signs differ
        if (current >= 0.0 && next < 0.0) || (current < 0.0 && next >= 0.0) {
            // Choose the sample closer to zero
            let crossing_idx = if current.abs() < next.abs() { i } else { i + 1 };
            let distance = crossing_idx.abs_diff(target);

            // Prefer closer crossings, and among equidistant ones, prefer lower magnitude
            if distance < best_distance
                || (distance == best_distance && samples[crossing_idx].abs() < best_magnitude)
            {
                best_idx = crossing_idx;
                best_distance = distance;
                best_magnitude = samples[crossing_idx].abs();
            }
        }
    }

    // If no zero crossing found, find the sample with lowest absolute value
    if best_distance == usize::MAX {
        for (idx, &sample) in samples
            .iter()
            .enumerate()
            .take(search_end + 1)
            .skip(search_start)
        {
            let magnitude = sample.abs();
            let distance = idx.abs_diff(target);

            if magnitude < best_magnitude
                || (magnitude == best_magnitude && distance < best_distance)
            {
                best_idx = idx;
                best_magnitude = magnitude;
                best_distance = distance;
            }
        }
    }

    best_idx
}

/// Calculates loop points based on envelope and loop configuration.
///
/// The default loop start is placed after the attack+decay phases (in the sustain
/// region). The loop end defaults to the end of the audio.
///
/// # Arguments
/// * `envelope` - ADSR envelope parameters
/// * `loop_config` - Loop configuration
/// * `samples` - Audio sample buffer
/// * `sample_rate` - Sample rate in Hz
///
/// # Returns
/// Calculated loop points with optional zero crossing snapping.
pub fn calculate_loop_points(
    envelope: &Envelope,
    loop_config: &LoopConfig,
    samples: &[f64],
    sample_rate: f64,
) -> LoopPoints {
    let num_samples = samples.len();

    // Calculate default loop start (after attack + decay)
    let attack_decay_samples = ((envelope.attack + envelope.decay) * sample_rate) as usize;
    let default_start = attack_decay_samples.min(num_samples.saturating_sub(1));

    // Get start and end from config or use defaults
    let mut start = loop_config
        .start_sample
        .map(|s| s as usize)
        .unwrap_or(default_start);
    let mut end = loop_config
        .end_sample
        .map(|s| s as usize)
        .unwrap_or(num_samples.saturating_sub(1));

    // Clamp to valid range
    start = start.min(num_samples.saturating_sub(1));
    end = end.min(num_samples.saturating_sub(1));

    // Ensure start < end
    if start >= end {
        start = 0;
        end = num_samples.saturating_sub(1);
    }

    let mut snapped = false;

    // Apply zero crossing snapping if enabled
    if loop_config.snap_to_zero_crossing && !samples.is_empty() {
        let tolerance = loop_config.zero_crossing_tolerance as usize;
        let new_start = find_nearest_zero_crossing(samples, start, tolerance);
        let new_end = find_nearest_zero_crossing(samples, end, tolerance);

        // Only apply if we found better points
        if new_start != start || new_end != end {
            snapped = true;
            start = new_start;
            end = new_end;

            // Ensure start < end after snapping
            if start >= end && start > 0 {
                start = start.saturating_sub(1);
            }
        }
    }

    LoopPoints {
        start,
        end,
        snapped_to_zero_crossing: snapped,
    }
}

/// Applies a cosine crossfade at loop boundaries.
///
/// This blends the loop end into the loop start to create a seamless transition.
/// The crossfade uses equal-power (cosine) curves for smooth amplitude preservation.
///
/// # Arguments
/// * `samples` - Audio sample buffer (modified in place)
/// * `loop_points` - Start and end of the loop region
/// * `crossfade_ms` - Duration of the crossfade in milliseconds
/// * `sample_rate` - Sample rate in Hz
///
/// # Returns
/// Number of samples in the crossfade region.
pub fn apply_loop_crossfade(
    samples: &mut [f64],
    loop_points: &LoopPoints,
    crossfade_ms: f32,
    sample_rate: f64,
) -> usize {
    if samples.is_empty() || loop_points.start >= loop_points.end {
        return 0;
    }

    let crossfade_samples = ((crossfade_ms as f64 / 1000.0) * sample_rate) as usize;
    if crossfade_samples == 0 {
        return 0;
    }

    // Calculate the actual crossfade length based on available samples
    let loop_length = loop_points.end - loop_points.start;
    let actual_crossfade = crossfade_samples.min(loop_length / 2);

    if actual_crossfade == 0 {
        return 0;
    }

    // Apply crossfade at the loop boundary
    // The end of the loop fades out while the start of the loop fades in
    for i in 0..actual_crossfade {
        let t = i as f64 / actual_crossfade as f64;

        // Equal-power crossfade using cosine curves
        let fade_out = (t * std::f64::consts::FRAC_PI_2).cos();
        let fade_in = (t * std::f64::consts::FRAC_PI_2).sin();

        // Get indices for the crossfade region
        let end_idx = loop_points.end - actual_crossfade + i;
        let start_idx = loop_points.start + i;

        if end_idx < samples.len() && start_idx < samples.len() {
            // Blend the end region with the start region
            let end_sample = samples[end_idx];
            let start_sample = samples[start_idx];

            // Apply crossfade at the end (fading out towards loop point)
            samples[end_idx] = end_sample * fade_out + start_sample * fade_in;
        }
    }

    actual_crossfade
}

/// Applies stereo crossfade at loop boundaries.
///
/// # Arguments
/// * `left` - Left channel samples (modified in place)
/// * `right` - Right channel samples (modified in place)
/// * `loop_points` - Start and end of the loop region
/// * `crossfade_ms` - Duration of the crossfade in milliseconds
/// * `sample_rate` - Sample rate in Hz
///
/// # Returns
/// Number of samples in the crossfade region.
pub fn apply_loop_crossfade_stereo(
    left: &mut [f64],
    right: &mut [f64],
    loop_points: &LoopPoints,
    crossfade_ms: f32,
    sample_rate: f64,
) -> usize {
    let left_crossfade = apply_loop_crossfade(left, loop_points, crossfade_ms, sample_rate);
    let right_crossfade = apply_loop_crossfade(right, loop_points, crossfade_ms, sample_rate);

    // Return the minimum (should be the same for both channels)
    left_crossfade.min(right_crossfade)
}

/// Measures the discontinuity at a loop point.
///
/// This is useful for testing loop quality. Returns the absolute difference
/// between the sample at loop end and the sample at loop start.
///
/// # Arguments
/// * `samples` - Audio sample buffer
/// * `loop_points` - Loop start and end points
///
/// # Returns
/// Absolute discontinuity value (0.0 = perfect continuity).
pub fn measure_loop_discontinuity(samples: &[f64], loop_points: &LoopPoints) -> f64 {
    if samples.is_empty() || loop_points.end >= samples.len() {
        return 0.0;
    }

    let start_val = samples[loop_points.start];
    let end_val = samples[loop_points.end];

    (end_val - start_val).abs()
}

/// Measures the maximum derivative (slope) at loop boundaries.
///
/// High derivatives at loop points can cause audible clicks even if the
/// values match. This measures the rate of change at the loop boundary.
///
/// # Arguments
/// * `samples` - Audio sample buffer
/// * `loop_points` - Loop start and end points
///
/// # Returns
/// Maximum absolute derivative at the loop boundary.
pub fn measure_loop_derivative(samples: &[f64], loop_points: &LoopPoints) -> f64 {
    if samples.len() < 2 || loop_points.end >= samples.len() {
        return 0.0;
    }

    let mut max_derivative: f64 = 0.0;

    // Check derivative at loop start
    if loop_points.start + 1 < samples.len() {
        let d = (samples[loop_points.start + 1] - samples[loop_points.start]).abs();
        max_derivative = max_derivative.max(d);
    }

    // Check derivative at loop end
    if loop_points.end + 1 < samples.len() {
        let d = (samples[loop_points.end + 1] - samples[loop_points.end]).abs();
        max_derivative = max_derivative.max(d);
    }

    // Check derivative across the loop boundary
    if loop_points.end < samples.len() && loop_points.start < samples.len() {
        let d = (samples[loop_points.end] - samples[loop_points.start]).abs();
        max_derivative = max_derivative.max(d);
    }

    max_derivative
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_zero_crossing_basic() {
        // Simple sine-like crossing at sample 5
        let samples = vec![-0.5, -0.3, -0.1, 0.0, 0.1, 0.3, 0.5];
        let crossing = find_nearest_zero_crossing(&samples, 3, 10);
        // Should find the zero at index 3
        assert_eq!(crossing, 3);
    }

    #[test]
    fn test_find_zero_crossing_between_samples() {
        // Zero crossing between samples 2 and 3
        let samples = vec![-0.5, -0.3, -0.1, 0.1, 0.3, 0.5];
        let crossing = find_nearest_zero_crossing(&samples, 3, 10);
        // Zero crossing is between index 2 (-0.1) and 3 (0.1), both same magnitude
        // Algorithm picks the one with smaller absolute value or closer to target
        // Since |0.1| == |-0.1|, it picks index 2 or 3 depending on implementation
        assert!(crossing == 2 || crossing == 3);
        // The value at the crossing should be near zero
        assert!(samples[crossing].abs() <= 0.1);
    }

    #[test]
    fn test_find_zero_crossing_no_crossing() {
        // All positive, no zero crossing
        let samples = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let crossing = find_nearest_zero_crossing(&samples, 3, 10);
        // Should find the smallest value (0.5 at index 0)
        assert_eq!(crossing, 0);
    }

    #[test]
    fn test_find_zero_crossing_respects_tolerance() {
        // Zero crossing at index 8, target at 3, tolerance 2
        let samples = vec![-0.5, -0.4, -0.3, -0.2, -0.1, -0.05, -0.02, -0.01, 0.01, 0.1];
        let crossing = find_nearest_zero_crossing(&samples, 3, 2);
        // Should not find the zero crossing (too far) and pick lowest in range
        assert!(crossing >= 1 && crossing <= 5);
    }

    #[test]
    fn test_calculate_loop_points_default() {
        let envelope = Envelope {
            attack: 0.01,
            decay: 0.05,
            sustain: 0.5,
            release: 0.1,
        };
        let config = LoopConfig::default();
        let samples = vec![0.0; 44100]; // 1 second at 44100 Hz

        let points = calculate_loop_points(&envelope, &config, &samples, 44100.0);

        // Default start should be after attack+decay (0.06s = 2646 samples)
        assert_eq!(points.start, 2646);
        // Default end should be at the last sample
        assert_eq!(points.end, 44099);
    }

    #[test]
    fn test_calculate_loop_points_explicit() {
        let envelope = Envelope::default();
        let config = LoopConfig::with_points(1000, 5000);
        let samples = vec![0.0; 10000];

        let points = calculate_loop_points(&envelope, &config, &samples, 44100.0);

        assert_eq!(points.start, 1000);
        assert_eq!(points.end, 5000);
    }

    #[test]
    fn test_calculate_loop_points_with_zero_crossing() {
        let envelope = Envelope::default();
        let config = LoopConfig {
            snap_to_zero_crossing: true,
            zero_crossing_tolerance: 100,
            ..LoopConfig::with_points(1000, 5000)
        };

        // Create samples with clear zero crossings near the target points
        let mut samples = vec![0.5; 10000];
        // Add clear zero crossing near 1000 (value goes from negative to positive)
        samples[998] = -0.3;
        samples[999] = -0.1;
        samples[1000] = 0.1;
        samples[1001] = 0.3;
        // Add clear zero crossing near 5000 (value goes from positive to negative)
        samples[4998] = 0.3;
        samples[4999] = 0.1;
        samples[5000] = -0.1;
        samples[5001] = -0.3;

        let points = calculate_loop_points(&envelope, &config, &samples, 44100.0);

        // Loop points should be near the zero crossings
        assert!(points.start >= 998 && points.start <= 1001);
        assert!(points.end >= 4998 && points.end <= 5001);
        // Note: snapped_to_zero_crossing is true only if we actually moved from the original point
    }

    #[test]
    fn test_apply_loop_crossfade() {
        let mut samples: Vec<f64> = (0..1000).map(|i| (i as f64 / 100.0).sin()).collect();
        let loop_points = LoopPoints {
            start: 100,
            end: 900,
            snapped_to_zero_crossing: false,
        };

        // Measure discontinuity before crossfade
        let disc_before = measure_loop_discontinuity(&samples, &loop_points);

        // Apply crossfade
        let crossfade_len = apply_loop_crossfade(&mut samples, &loop_points, 10.0, 44100.0);

        assert!(crossfade_len > 0);

        // The crossfade should reduce discontinuity
        let disc_after = measure_loop_discontinuity(&samples, &loop_points);
        assert!(
            disc_after <= disc_before + 0.1,
            "Crossfade should not increase discontinuity significantly"
        );
    }

    #[test]
    fn test_measure_loop_discontinuity() {
        // Perfect continuity
        let samples = vec![0.5, 0.5, 0.5, 0.5, 0.5];
        let points = LoopPoints {
            start: 1,
            end: 3,
            snapped_to_zero_crossing: false,
        };
        let disc = measure_loop_discontinuity(&samples, &points);
        assert!((disc - 0.0).abs() < 1e-10);

        // Some discontinuity
        let samples = vec![0.0, 0.5, 0.5, 0.8, 0.0];
        let disc = measure_loop_discontinuity(&samples, &points);
        assert!((disc - 0.3).abs() < 1e-10); // |0.8 - 0.5| = 0.3
    }

    #[test]
    fn test_zero_crossing_with_sine_wave() {
        // Generate a sine wave
        let samples: Vec<f64> = (0..1000)
            .map(|i| (2.0 * std::f64::consts::PI * i as f64 / 100.0).sin())
            .collect();

        // Find zero crossings
        let crossing = find_nearest_zero_crossing(&samples, 25, 50);

        // The sine wave crosses zero at multiples of 50 samples (100 samples per cycle)
        // Nearest to 25 should be 0 or 50
        assert!(crossing == 0 || crossing == 50);
    }

    #[test]
    fn test_loop_config_disabled() {
        let envelope = Envelope::default();
        let config = LoopConfig::disabled();
        let samples = vec![0.0; 1000];

        // Even with disabled config, calculate_loop_points should still work
        // (the caller decides whether to use the result based on config.enabled)
        let points = calculate_loop_points(&envelope, &config, &samples, 44100.0);
        assert!(points.start < points.end);
    }
}
