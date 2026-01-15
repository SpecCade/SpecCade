//! Delay effect with feedback and ping-pong stereo support.

use crate::error::{AudioError, AudioResult};
use crate::mixer::StereoOutput;

/// Ring buffer for delay line.
struct DelayLine {
    buffer: Vec<f64>,
    write_pos: usize,
}

impl DelayLine {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            write_pos: 0,
        }
    }

    fn write(&mut self, sample: f64) {
        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
    }

    fn read(&self, delay_samples: usize) -> f64 {
        let read_pos = (self.write_pos + self.buffer.len() - delay_samples) % self.buffer.len();
        self.buffer[read_pos]
    }

    fn read_interpolated(&self, delay_samples: f64) -> f64 {
        let delay_int = delay_samples.floor() as usize;
        let delay_frac = delay_samples - delay_int as f64;

        let sample1 = self.read(delay_int);
        let sample2 = self.read(delay_int + 1);

        // Linear interpolation
        sample1 * (1.0 - delay_frac) + sample2 * delay_frac
    }
}

/// Applies delay effect to stereo audio.
pub fn apply(
    stereo: &mut StereoOutput,
    time_ms: f64,
    feedback: f64,
    wet: f64,
    ping_pong: bool,
    sample_rate: f64,
) -> AudioResult<()> {
    // Create a constant time curve for the non-modulated case
    let num_samples = stereo.left.len();
    let time_curve = vec![time_ms; num_samples];
    apply_with_modulation(stereo, &time_curve, feedback, wet, ping_pong, sample_rate)
}

/// Applies delay effect to stereo audio with per-sample time modulation.
///
/// # Arguments
/// * `stereo` - Stereo audio to process
/// * `time_curve` - Per-sample delay time in milliseconds
/// * `feedback` - Feedback amount (0.0-0.95)
/// * `wet` - Wet/dry mix (0.0-1.0)
/// * `ping_pong` - Enable ping-pong stereo delay
/// * `sample_rate` - Sample rate in Hz
pub fn apply_with_modulation(
    stereo: &mut StereoOutput,
    time_curve: &[f64],
    feedback: f64,
    wet: f64,
    ping_pong: bool,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(0.0..=0.95).contains(&feedback) {
        return Err(AudioError::invalid_param(
            "delay.feedback",
            format!("must be 0.0-0.95, got {}", feedback),
        ));
    }
    if !(0.0..=1.0).contains(&wet) {
        return Err(AudioError::invalid_param(
            "delay.wet",
            format!("must be 0.0-1.0, got {}", wet),
        ));
    }

    // Find max delay time to size the buffer
    let max_time_ms = time_curve
        .iter()
        .copied()
        .fold(1.0_f64, |a, b| a.max(b))
        .clamp(1.0, 2000.0);
    let max_delay_samples = (max_time_ms / 1000.0) * sample_rate;
    let delay_buffer_size = (max_delay_samples.ceil() as usize + 2).max(4);

    let mut delay_left = DelayLine::new(delay_buffer_size);
    let mut delay_right = DelayLine::new(delay_buffer_size);

    let num_samples = stereo.left.len();
    let mut output_left = Vec::with_capacity(num_samples);
    let mut output_right = Vec::with_capacity(num_samples);

    let dry = 1.0 - wet;

    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // Get modulated delay time for this sample
        let time_ms = time_curve.get(i).copied().unwrap_or(1.0).clamp(1.0, 2000.0);
        let delay_samples = (time_ms / 1000.0) * sample_rate;

        if ping_pong {
            // Ping-pong: left delay feeds right, right delay feeds left
            let delayed_right = delay_right.read_interpolated(delay_samples);
            let delayed_left = delay_left.read_interpolated(delay_samples);

            let fb_left = in_left + delayed_right * feedback;
            let fb_right = in_right + delayed_left * feedback;

            delay_left.write(fb_left);
            delay_right.write(fb_right);

            output_left.push(in_left * dry + delayed_left * wet);
            output_right.push(in_right * dry + delayed_right * wet);
        } else {
            // Normal stereo delay
            let delayed_left = delay_left.read_interpolated(delay_samples);
            let delayed_right = delay_right.read_interpolated(delay_samples);

            let fb_left = in_left + delayed_left * feedback;
            let fb_right = in_right + delayed_right * feedback;

            delay_left.write(fb_left);
            delay_right.write(fb_right);

            output_left.push(in_left * dry + delayed_left * wet);
            output_right.push(in_right * dry + delayed_right * wet);
        }
    }

    stereo.left = output_left;
    stereo.right = output_right;

    Ok(())
}
