//! Flanger effect with LFO-modulated delay and feedback.
//!
//! Flanger is similar to chorus but uses shorter base delays and a feedback path
//! to create comb filter resonance and the characteristic "jet" sound.

use crate::error::{AudioError, AudioResult};
use crate::mixer::StereoOutput;
use std::f64::consts::PI;

const TWO_PI: f64 = 2.0 * PI;

/// Delay line with interpolated read for modulation and feedback support.
struct FlangerDelayLine {
    buffer: Vec<f64>,
    write_pos: usize,
}

impl FlangerDelayLine {
    fn new(max_delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_delay_samples],
            write_pos: 0,
        }
    }

    fn write(&mut self, sample: f64) {
        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
    }

    fn read_interpolated(&self, delay_samples: f64) -> f64 {
        let delay_clamped = delay_samples.max(0.0).min(self.buffer.len() as f64 - 1.0);
        let delay_int = delay_clamped.floor() as usize;
        let delay_frac = delay_clamped - delay_int as f64;

        let read_pos1 = (self.write_pos + self.buffer.len() - delay_int - 1) % self.buffer.len();
        let read_pos2 = (self.write_pos + self.buffer.len() - delay_int - 2) % self.buffer.len();

        let sample1 = self.buffer[read_pos1];
        let sample2 = self.buffer[read_pos2];

        // Linear interpolation
        sample1 * (1.0 - delay_frac) + sample2 * delay_frac
    }
}

/// Applies flanger effect to stereo audio.
///
/// Flanger uses short modulated delays with feedback to create the classic
/// "jet" or "swoosh" sound. Distinguished from chorus by shorter base delays
/// and feedback path creating comb filter resonance.
///
/// # Arguments
/// * `stereo` - Stereo audio to process in-place
/// * `rate` - LFO rate in Hz (0.1-10.0)
/// * `depth` - Modulation depth (0.0-1.0)
/// * `feedback` - Feedback amount (-0.99 to 0.99)
/// * `delay_ms` - Base delay time in milliseconds (1-20 typical)
/// * `wet` - Wet/dry mix (0.0-1.0)
/// * `sample_rate` - Sample rate in Hz
pub fn apply(
    stereo: &mut StereoOutput,
    rate: f64,
    depth: f64,
    feedback: f64,
    delay_ms: f64,
    wet: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(0.1..=10.0).contains(&rate) {
        return Err(AudioError::invalid_param(
            "flanger.rate",
            format!("must be 0.1-10.0 Hz, got {}", rate),
        ));
    }
    if !(0.0..=1.0).contains(&depth) {
        return Err(AudioError::invalid_param(
            "flanger.depth",
            format!("must be 0.0-1.0, got {}", depth),
        ));
    }
    if !(-0.99..=0.99).contains(&feedback) {
        return Err(AudioError::invalid_param(
            "flanger.feedback",
            format!("must be -0.99 to 0.99, got {}", feedback),
        ));
    }
    if !(0.1..=50.0).contains(&delay_ms) {
        return Err(AudioError::invalid_param(
            "flanger.delay_ms",
            format!("must be 0.1-50.0 ms, got {}", delay_ms),
        ));
    }
    if !(0.0..=1.0).contains(&wet) {
        return Err(AudioError::invalid_param(
            "flanger.wet",
            format!("must be 0.0-1.0, got {}", wet),
        ));
    }

    // Flanger parameters
    // Depth maps to approximately 5ms of modulation range
    let max_modulation_ms = 5.0;
    let base_delay_samples = (delay_ms / 1000.0) * sample_rate;
    let max_modulation_samples = (max_modulation_ms / 1000.0) * sample_rate;

    // Buffer needs to accommodate base delay + full modulation range
    let buffer_size = (base_delay_samples + max_modulation_samples).ceil() as usize + 2;

    let mut delay_left = FlangerDelayLine::new(buffer_size);
    let mut delay_right = FlangerDelayLine::new(buffer_size);

    let num_samples = stereo.left.len();
    let mut output_left = Vec::with_capacity(num_samples);
    let mut output_right = Vec::with_capacity(num_samples);

    let dry = 1.0 - wet;

    // Clamp feedback for stability (already validated but belt-and-suspenders)
    let feedback_clamped = feedback.clamp(-0.99, 0.99);

    // Stereo LFO phase offset for width (quarter cycle offset)
    let stereo_phase_offset = 0.25;

    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        let t = i as f64 / sample_rate;

        // LFO for left channel (sine wave, 0 to 1 range)
        let lfo_left = ((TWO_PI * rate * t).sin() * 0.5 + 0.5) * depth;

        // LFO for right channel with phase offset for stereo width
        let lfo_right =
            ((TWO_PI * rate * t + TWO_PI * stereo_phase_offset).sin() * 0.5 + 0.5) * depth;

        // Calculate modulated delay times
        let modulated_delay_left = base_delay_samples + lfo_left * max_modulation_samples;
        let modulated_delay_right = base_delay_samples + lfo_right * max_modulation_samples;

        // Read delayed samples
        let delayed_left = delay_left.read_interpolated(modulated_delay_left);
        let delayed_right = delay_right.read_interpolated(modulated_delay_right);

        // Apply feedback: input + feedback * delayed_sample goes into delay line
        let feedback_left = in_left + feedback_clamped * delayed_left;
        let feedback_right = in_right + feedback_clamped * delayed_right;

        // Write to delay lines (with feedback)
        delay_left.write(feedback_left);
        delay_right.write(feedback_right);

        // Mix wet/dry
        let out_left = dry * in_left + wet * delayed_left;
        let out_right = dry * in_right + wet * delayed_right;

        output_left.push(out_left);
        output_right.push(out_right);
    }

    stereo.left = output_left;
    stereo.right = output_right;

    Ok(())
}

/// Applies flanger effect with external LFO modulation of delay time.
///
/// This variant allows post-FX LFO modulation of the base delay time,
/// adding additional motion to the flanger effect.
///
/// # Arguments
/// * `stereo` - Stereo audio to process in-place
/// * `rate` - Internal LFO rate in Hz (0.1-10.0)
/// * `depth` - Internal modulation depth (0.0-1.0)
/// * `feedback` - Feedback amount (-0.99 to 0.99)
/// * `base_delay_ms` - Base delay time in milliseconds
/// * `delay_lfo_curve` - Pre-computed LFO curve for delay modulation (0.0-1.0 range)
/// * `delay_lfo_amount_ms` - Maximum delay modulation in milliseconds
/// * `wet` - Wet/dry mix (0.0-1.0)
/// * `sample_rate` - Sample rate in Hz
#[allow(clippy::too_many_arguments)]
pub fn apply_with_modulation(
    stereo: &mut StereoOutput,
    rate: f64,
    depth: f64,
    feedback: f64,
    base_delay_ms: &f64,
    delay_lfo_curve: &[f64],
    delay_lfo_amount_ms: f64,
    wet: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(0.1..=10.0).contains(&rate) {
        return Err(AudioError::invalid_param(
            "flanger.rate",
            format!("must be 0.1-10.0 Hz, got {}", rate),
        ));
    }
    if !(0.0..=1.0).contains(&depth) {
        return Err(AudioError::invalid_param(
            "flanger.depth",
            format!("must be 0.0-1.0, got {}", depth),
        ));
    }
    if !(-0.99..=0.99).contains(&feedback) {
        return Err(AudioError::invalid_param(
            "flanger.feedback",
            format!("must be -0.99 to 0.99, got {}", feedback),
        ));
    }
    if !(0.1..=50.0).contains(base_delay_ms) {
        return Err(AudioError::invalid_param(
            "flanger.delay_ms",
            format!("must be 0.1-50.0 ms, got {}", base_delay_ms),
        ));
    }
    if !(0.0..=1.0).contains(&wet) {
        return Err(AudioError::invalid_param(
            "flanger.wet",
            format!("must be 0.0-1.0, got {}", wet),
        ));
    }

    // Calculate maximum possible delay for buffer sizing
    let max_delay_ms = base_delay_ms + delay_lfo_amount_ms.abs();
    let max_modulation_ms = 5.0; // Internal LFO modulation range
    let max_delay_samples = ((max_delay_ms + max_modulation_ms) / 1000.0) * sample_rate;
    let buffer_size = max_delay_samples.ceil() as usize + 2;

    let mut delay_left = FlangerDelayLine::new(buffer_size);
    let mut delay_right = FlangerDelayLine::new(buffer_size);

    let num_samples = stereo.left.len();
    let mut output_left = Vec::with_capacity(num_samples);
    let mut output_right = Vec::with_capacity(num_samples);

    let dry = 1.0 - wet;
    let feedback_clamped = feedback.clamp(-0.99, 0.99);
    let stereo_phase_offset = 0.25;
    let max_modulation_samples = (max_modulation_ms / 1000.0) * sample_rate;

    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // Get external LFO modulated delay time
        let lfo_value = delay_lfo_curve.get(i).copied().unwrap_or(0.5);
        let bipolar = (lfo_value - 0.5) * 2.0;
        let modulated_base_delay_ms =
            (*base_delay_ms + bipolar * delay_lfo_amount_ms).clamp(0.1, 50.0);
        let modulated_base_delay_samples = (modulated_base_delay_ms / 1000.0) * sample_rate;

        let t = i as f64 / sample_rate;

        // Internal LFO for left channel
        let lfo_left = ((TWO_PI * rate * t).sin() * 0.5 + 0.5) * depth;
        let lfo_right =
            ((TWO_PI * rate * t + TWO_PI * stereo_phase_offset).sin() * 0.5 + 0.5) * depth;

        // Calculate total modulated delay times
        let total_delay_left = modulated_base_delay_samples + lfo_left * max_modulation_samples;
        let total_delay_right = modulated_base_delay_samples + lfo_right * max_modulation_samples;

        // Read delayed samples
        let delayed_left = delay_left.read_interpolated(total_delay_left);
        let delayed_right = delay_right.read_interpolated(total_delay_right);

        // Apply feedback
        let feedback_left = in_left + feedback_clamped * delayed_left;
        let feedback_right = in_right + feedback_clamped * delayed_right;

        // Write to delay lines
        delay_left.write(feedback_left);
        delay_right.write(feedback_right);

        // Mix wet/dry
        let out_left = dry * in_left + wet * delayed_left;
        let out_right = dry * in_right + wet * delayed_right;

        output_left.push(out_left);
        output_right.push(out_right);
    }

    stereo.left = output_left;
    stereo.right = output_right;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stereo(len: usize) -> StereoOutput {
        // Simple sine wave for testing
        let samples: Vec<f64> = (0..len).map(|i| (i as f64 * 0.1).sin() * 0.5).collect();
        StereoOutput {
            left: samples.clone(),
            right: samples,
        }
    }

    #[test]
    fn test_flanger_basic() {
        let mut stereo = make_stereo(44100);
        let result = apply(&mut stereo, 0.5, 0.5, 0.5, 5.0, 0.5, 44100.0);
        assert!(result.is_ok());
        assert_eq!(stereo.left.len(), 44100);
        assert_eq!(stereo.right.len(), 44100);
    }

    #[test]
    fn test_flanger_negative_feedback() {
        let mut stereo = make_stereo(44100);
        let result = apply(&mut stereo, 1.0, 0.7, -0.7, 3.0, 0.6, 44100.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_flanger_invalid_rate() {
        let mut stereo = make_stereo(1000);
        let result = apply(&mut stereo, 0.05, 0.5, 0.5, 5.0, 0.5, 44100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_flanger_invalid_feedback() {
        let mut stereo = make_stereo(1000);
        let result = apply(&mut stereo, 0.5, 0.5, 1.5, 5.0, 0.5, 44100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_flanger_zero_wet() {
        let mut stereo = make_stereo(1000);
        let original_left = stereo.left.clone();
        let result = apply(&mut stereo, 0.5, 0.5, 0.5, 5.0, 0.0, 44100.0);
        assert!(result.is_ok());
        // With zero wet, output should equal input
        for (orig, out) in original_left.iter().zip(stereo.left.iter()) {
            assert!((orig - out).abs() < 1e-10);
        }
    }

    #[test]
    fn test_flanger_determinism() {
        let mut stereo1 = make_stereo(4410);
        let mut stereo2 = make_stereo(4410);

        apply(&mut stereo1, 0.5, 0.5, 0.5, 5.0, 0.5, 44100.0).unwrap();
        apply(&mut stereo2, 0.5, 0.5, 0.5, 5.0, 0.5, 44100.0).unwrap();

        // Results should be identical
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
}
