//! Stereo widener effect for enhancing stereo image.
//!
//! Provides three processing modes:
//! - Simple: L/R crossmix for basic widening
//! - Haas: Delay-based psychoacoustic widening
//! - Mid/Side: Classic M/S processing

use crate::error::{AudioError, AudioResult};
use crate::mixer::StereoOutput;
use speccade_spec::recipe::audio::StereoWidenerMode;

/// Applies stereo widener effect to stereo audio.
///
/// # Arguments
/// * `stereo` - Stereo audio to process in-place
/// * `width` - Stereo width (0.0 = mono, 1.0 = normal, >1.0 = wider). Range: 0.0-2.0.
/// * `mode` - Processing algorithm
/// * `delay_ms` - Delay time in ms for Haas mode only (1-30 typical)
/// * `sample_rate` - Sample rate in Hz
pub fn apply(
    stereo: &mut StereoOutput,
    width: f64,
    mode: &StereoWidenerMode,
    delay_ms: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Clamp width to valid range
    let width = width.clamp(0.0, 2.0);

    // Validate delay_ms for Haas mode
    if matches!(mode, StereoWidenerMode::Haas) && !(0.1..=50.0).contains(&delay_ms) {
        return Err(AudioError::invalid_param(
            "stereo_widener.delay_ms",
            format!("must be 0.1-50.0 ms for Haas mode, got {}", delay_ms),
        ));
    }

    match mode {
        StereoWidenerMode::Simple => apply_simple(stereo, width),
        StereoWidenerMode::Haas => apply_haas(stereo, width, delay_ms, sample_rate),
        StereoWidenerMode::MidSide => apply_mid_side(stereo, width),
    }

    Ok(())
}

/// Applies stereo widener effect with LFO modulation of delay time (Haas mode).
///
/// This variant allows post-FX LFO modulation of the delay time in Haas mode,
/// adding additional motion to the stereo image.
///
/// # Arguments
/// * `stereo` - Stereo audio to process in-place
/// * `width` - Stereo width (0.0 = mono, 1.0 = normal, >1.0 = wider). Range: 0.0-2.0.
/// * `mode` - Processing algorithm
/// * `base_delay_ms` - Base delay time in ms for Haas mode
/// * `delay_lfo_curve` - Pre-computed LFO curve for delay modulation (0.0-1.0 range)
/// * `delay_lfo_amount_ms` - Maximum delay modulation in milliseconds
/// * `sample_rate` - Sample rate in Hz
pub fn apply_with_modulation(
    stereo: &mut StereoOutput,
    width: f64,
    mode: &StereoWidenerMode,
    base_delay_ms: f64,
    delay_lfo_curve: &[f64],
    delay_lfo_amount_ms: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Clamp width to valid range
    let width = width.clamp(0.0, 2.0);

    // Only Haas mode supports delay modulation
    match mode {
        StereoWidenerMode::Haas => {
            apply_haas_modulated(
                stereo,
                width,
                base_delay_ms,
                delay_lfo_curve,
                delay_lfo_amount_ms,
                sample_rate,
            );
            Ok(())
        }
        // For other modes, fall back to standard processing
        StereoWidenerMode::Simple => {
            apply_simple(stereo, width);
            Ok(())
        }
        StereoWidenerMode::MidSide => {
            apply_mid_side(stereo, width);
            Ok(())
        }
    }
}

/// Simple L/R crossmix widening.
///
/// Formula:
/// - new_L = (1 + width) * L - width * R
/// - new_R = (1 + width) * R - width * L
///
/// At width=0, output is mono (L+R)/2 behavior approximated.
/// At width=1, output equals input (normal stereo).
/// At width>1, stereo difference is amplified.
fn apply_simple(stereo: &mut StereoOutput, width: f64) {
    let num_samples = stereo.left.len();

    // For width=0, we want mono output
    // For width=1, we want unchanged stereo
    // For width>1, we want enhanced stereo

    // Scale factor for crossmix
    // At width=0: factor = 0, gain = 1 -> mono blend
    // At width=1: factor = 0, gain = 1 -> unchanged
    // At width=2: factor = 1, gain = 2 -> maximum widening

    // Remap width: 0->-1, 1->0, 2->1
    let factor = width - 1.0;

    for i in 0..num_samples {
        let left = stereo.left[i];
        let right = stereo.right[i];

        // Calculate stereo difference (side signal)
        let side = (left - right) * 0.5;
        // Calculate mono (mid signal)
        let mid = (left + right) * 0.5;

        // Reconstruct with adjusted side level
        // At factor=-1 (width=0): side is removed -> mono
        // At factor=0 (width=1): side unchanged -> original stereo
        // At factor=1 (width=2): side doubled -> enhanced stereo
        let new_side = side * (1.0 + factor);

        stereo.left[i] = mid + new_side;
        stereo.right[i] = mid - new_side;
    }
}

/// Haas effect stereo widening.
///
/// Creates stereo width by delaying the right channel slightly.
/// The width parameter controls the blend between dry and delayed signal.
fn apply_haas(stereo: &mut StereoOutput, width: f64, delay_ms: f64, sample_rate: f64) {
    let delay_samples = ((delay_ms / 1000.0) * sample_rate) as usize;

    if delay_samples == 0 {
        // No delay, just return
        return;
    }

    let num_samples = stereo.left.len();

    // Create delay buffer for right channel
    let mut delayed_right = vec![0.0; num_samples];

    for (i, delayed) in delayed_right.iter_mut().enumerate() {
        if i >= delay_samples {
            *delayed = stereo.right[i - delay_samples];
        }
        // Samples before delay_samples remain 0 (silence)
    }

    // Blend based on width
    // At width=0: mono (left channel to both)
    // At width=1: normal stereo with Haas delay
    // At width>1: enhanced Haas effect

    for (i, (out_left, out_right)) in stereo
        .left
        .iter_mut()
        .zip(stereo.right.iter_mut())
        .enumerate()
    {
        let left = *out_left;
        let right = *out_right;
        let delayed = delayed_right[i];

        if width <= 1.0 {
            // Blend from mono to full Haas
            let mono = (left + right) * 0.5;
            *out_left = mono + (left - mono) * width;
            *out_right = mono + (delayed - mono) * width;
        } else {
            // Enhanced Haas: increase the delay effect
            let enhance = width - 1.0;
            // Mix in more of the opposite channel's delay
            *out_left = left;
            *out_right = delayed + (delayed - right) * enhance;
        }
    }
}

/// Haas effect with LFO-modulated delay time.
fn apply_haas_modulated(
    stereo: &mut StereoOutput,
    width: f64,
    base_delay_ms: f64,
    delay_lfo_curve: &[f64],
    delay_lfo_amount_ms: f64,
    sample_rate: f64,
) {
    let num_samples = stereo.left.len();

    // Calculate maximum delay for buffer sizing
    let max_delay_ms = base_delay_ms + delay_lfo_amount_ms.abs();
    let max_delay_samples = ((max_delay_ms / 1000.0) * sample_rate).ceil() as usize + 1;

    // Create delay buffer
    let mut delay_buffer = vec![0.0; max_delay_samples];
    let mut write_pos = 0;

    let mut output_left = Vec::with_capacity(num_samples);
    let mut output_right = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let left = stereo.left[i];
        let right = stereo.right[i];

        // Get modulated delay time
        let lfo_value = delay_lfo_curve.get(i).copied().unwrap_or(0.5);
        let bipolar = (lfo_value - 0.5) * 2.0;
        let modulated_delay_ms = (base_delay_ms + bipolar * delay_lfo_amount_ms).clamp(0.1, 50.0);
        let delay_samples = (modulated_delay_ms / 1000.0) * sample_rate;

        // Write to delay buffer
        delay_buffer[write_pos] = right;

        // Read from delay buffer with linear interpolation
        let delay_int = delay_samples.floor() as usize;
        let delay_frac = delay_samples - delay_int as f64;

        let read_pos1 = (write_pos + max_delay_samples - delay_int) % max_delay_samples;
        let read_pos2 = (write_pos + max_delay_samples - delay_int - 1) % max_delay_samples;

        let delayed =
            delay_buffer[read_pos1] * (1.0 - delay_frac) + delay_buffer[read_pos2] * delay_frac;

        // Apply width blending
        let out_left;
        let out_right;

        if width <= 1.0 {
            let mono = (left + right) * 0.5;
            out_left = mono + (left - mono) * width;
            out_right = mono + (delayed - mono) * width;
        } else {
            let enhance = width - 1.0;
            out_left = left;
            out_right = delayed + (delayed - right) * enhance;
        }

        output_left.push(out_left);
        output_right.push(out_right);

        write_pos = (write_pos + 1) % max_delay_samples;
    }

    stereo.left = output_left;
    stereo.right = output_right;
}

/// Mid/Side processing for stereo widening.
///
/// Converts to M/S domain, scales side signal, converts back.
/// - mid = (L + R) / 2
/// - side = (L - R) / 2
/// - side *= width
/// - L = mid + side, R = mid - side
fn apply_mid_side(stereo: &mut StereoOutput, width: f64) {
    let num_samples = stereo.left.len();

    for i in 0..num_samples {
        let left = stereo.left[i];
        let right = stereo.right[i];

        // Convert to M/S
        let mid = (left + right) * 0.5;
        let side = (left - right) * 0.5;

        // Scale side signal
        let scaled_side = side * width;

        // Convert back to L/R
        stereo.left[i] = mid + scaled_side;
        stereo.right[i] = mid - scaled_side;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stereo(len: usize) -> StereoOutput {
        // Simple sine wave for testing with different L/R content
        let left: Vec<f64> = (0..len).map(|i| (i as f64 * 0.1).sin() * 0.5).collect();
        let right: Vec<f64> = (0..len).map(|i| (i as f64 * 0.12).sin() * 0.5).collect();
        StereoOutput { left, right }
    }

    fn make_mono(len: usize) -> StereoOutput {
        let samples: Vec<f64> = (0..len).map(|i| (i as f64 * 0.1).sin() * 0.5).collect();
        StereoOutput {
            left: samples.clone(),
            right: samples,
        }
    }

    #[test]
    fn test_simple_width_one_unchanged() {
        let mut stereo = make_stereo(1000);
        let original_left = stereo.left.clone();
        let original_right = stereo.right.clone();

        apply(&mut stereo, 1.0, &StereoWidenerMode::Simple, 10.0, 44100.0).unwrap();

        // At width=1, output should approximately equal input
        for i in 0..stereo.left.len() {
            assert!(
                (stereo.left[i] - original_left[i]).abs() < 1e-10,
                "Left channel changed at sample {}",
                i
            );
            assert!(
                (stereo.right[i] - original_right[i]).abs() < 1e-10,
                "Right channel changed at sample {}",
                i
            );
        }
    }

    #[test]
    fn test_simple_width_zero_mono() {
        let mut stereo = make_stereo(1000);

        apply(&mut stereo, 0.0, &StereoWidenerMode::Simple, 10.0, 44100.0).unwrap();

        // At width=0, left and right should be equal (mono)
        for i in 0..stereo.left.len() {
            assert!(
                (stereo.left[i] - stereo.right[i]).abs() < 1e-10,
                "Channels differ at sample {}: L={} R={}",
                i,
                stereo.left[i],
                stereo.right[i]
            );
        }
    }

    #[test]
    fn test_mid_side_width_one_unchanged() {
        let mut stereo = make_stereo(1000);
        let original_left = stereo.left.clone();
        let original_right = stereo.right.clone();

        apply(&mut stereo, 1.0, &StereoWidenerMode::MidSide, 10.0, 44100.0).unwrap();

        // At width=1, output should equal input
        for i in 0..stereo.left.len() {
            assert!(
                (stereo.left[i] - original_left[i]).abs() < 1e-10,
                "Left channel changed at sample {}",
                i
            );
            assert!(
                (stereo.right[i] - original_right[i]).abs() < 1e-10,
                "Right channel changed at sample {}",
                i
            );
        }
    }

    #[test]
    fn test_mid_side_width_zero_mono() {
        let mut stereo = make_stereo(1000);

        apply(&mut stereo, 0.0, &StereoWidenerMode::MidSide, 10.0, 44100.0).unwrap();

        // At width=0, side is removed, so L=R=mid
        for i in 0..stereo.left.len() {
            assert!(
                (stereo.left[i] - stereo.right[i]).abs() < 1e-10,
                "Channels differ at sample {}: L={} R={}",
                i,
                stereo.left[i],
                stereo.right[i]
            );
        }
    }

    #[test]
    fn test_haas_basic() {
        let mut stereo = make_stereo(44100);

        let result = apply(&mut stereo, 1.0, &StereoWidenerMode::Haas, 10.0, 44100.0);
        assert!(result.is_ok());
        assert_eq!(stereo.left.len(), 44100);
        assert_eq!(stereo.right.len(), 44100);
    }

    #[test]
    fn test_haas_invalid_delay() {
        let mut stereo = make_stereo(1000);

        let result = apply(&mut stereo, 1.0, &StereoWidenerMode::Haas, 100.0, 44100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_width_clamped() {
        let mut stereo = make_stereo(1000);

        // Width > 2.0 should be clamped
        let result = apply(&mut stereo, 5.0, &StereoWidenerMode::Simple, 10.0, 44100.0);
        assert!(result.is_ok());

        // Width < 0.0 should be clamped
        let mut stereo2 = make_stereo(1000);
        let result2 = apply(
            &mut stereo2,
            -1.0,
            &StereoWidenerMode::Simple,
            10.0,
            44100.0,
        );
        assert!(result2.is_ok());
    }

    #[test]
    fn test_determinism() {
        let mut stereo1 = make_stereo(4410);
        let mut stereo2 = make_stereo(4410);

        apply(
            &mut stereo1,
            1.5,
            &StereoWidenerMode::MidSide,
            10.0,
            44100.0,
        )
        .unwrap();
        apply(
            &mut stereo2,
            1.5,
            &StereoWidenerMode::MidSide,
            10.0,
            44100.0,
        )
        .unwrap();

        for (a, b) in stereo1.left.iter().zip(stereo2.left.iter()) {
            assert!(
                (a - b).abs() < 1e-15,
                "Left channel mismatch: {} vs {}",
                a,
                b
            );
        }
        for (a, b) in stereo1.right.iter().zip(stereo2.right.iter()) {
            assert!(
                (a - b).abs() < 1e-15,
                "Right channel mismatch: {} vs {}",
                a,
                b
            );
        }
    }

    #[test]
    fn test_mono_input_expansion() {
        // When L=R (mono input), widening should have no effect for mid/side
        let mut stereo = make_mono(1000);

        apply(&mut stereo, 2.0, &StereoWidenerMode::MidSide, 10.0, 44100.0).unwrap();

        // For mono input, side signal is 0, so widening has no effect
        for i in 0..stereo.left.len() {
            assert!(
                (stereo.left[i] - stereo.right[i]).abs() < 1e-10,
                "Mono input should remain mono after mid/side widening"
            );
        }
    }

    #[test]
    fn test_with_modulation_haas() {
        let mut stereo = make_stereo(4410);
        let lfo_curve: Vec<f64> = (0..4410).map(|i| i as f64 / 4410.0).collect();

        let result = apply_with_modulation(
            &mut stereo,
            1.0,
            &StereoWidenerMode::Haas,
            10.0,
            &lfo_curve,
            5.0,
            44100.0,
        );

        assert!(result.is_ok());
        assert_eq!(stereo.left.len(), 4410);
        assert_eq!(stereo.right.len(), 4410);
    }

    #[test]
    fn test_with_modulation_simple_fallback() {
        // Simple mode should ignore modulation and just apply normal processing
        let mut stereo = make_stereo(1000);
        let original_left = stereo.left.clone();
        let lfo_curve: Vec<f64> = (0..1000).map(|i| i as f64 / 1000.0).collect();

        apply_with_modulation(
            &mut stereo,
            1.0,
            &StereoWidenerMode::Simple,
            10.0,
            &lfo_curve,
            5.0,
            44100.0,
        )
        .unwrap();

        // At width=1, simple mode should leave input unchanged
        for (out, orig) in stereo.left.iter().zip(original_left.iter()) {
            assert!(
                (out - orig).abs() < 1e-10,
                "Simple mode with modulation should ignore delay LFO"
            );
        }
    }
}
