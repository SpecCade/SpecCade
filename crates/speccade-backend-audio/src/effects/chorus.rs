//! Chorus and phaser effects with LFO modulation.

use crate::error::{AudioError, AudioResult};
use crate::mixer::StereoOutput;
use std::f64::consts::PI;

const TWO_PI: f64 = 2.0 * PI;

/// Delay line with interpolated read for modulation.
struct ModulatedDelayLine {
    buffer: Vec<f64>,
    write_pos: usize,
}

impl ModulatedDelayLine {
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

/// Applies chorus effect to stereo audio.
pub fn apply(
    stereo: &mut StereoOutput,
    rate: f64,
    depth: f64,
    wet: f64,
    voices: u8,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(0.1..=10.0).contains(&rate) {
        return Err(AudioError::invalid_param(
            "chorus.rate",
            format!("must be 0.1-10.0 Hz, got {}", rate),
        ));
    }
    if !(0.0..=1.0).contains(&depth) {
        return Err(AudioError::invalid_param(
            "chorus.depth",
            format!("must be 0.0-1.0, got {}", depth),
        ));
    }
    if !(0.0..=1.0).contains(&wet) {
        return Err(AudioError::invalid_param(
            "chorus.wet",
            format!("must be 0.0-1.0, got {}", wet),
        ));
    }
    if !(1..=4).contains(&voices) {
        return Err(AudioError::invalid_param(
            "chorus.voices",
            format!("must be 1-4, got {}", voices),
        ));
    }

    // Chorus parameters
    let base_delay_ms = 20.0; // Base delay in milliseconds
    let max_delay_ms = 40.0; // Maximum delay variation

    let base_delay_samples = (base_delay_ms / 1000.0) * sample_rate;
    let max_delay_samples = (max_delay_ms / 1000.0) * sample_rate;
    let buffer_size = (base_delay_samples + max_delay_samples).ceil() as usize + 2;

    let mut delay_left = ModulatedDelayLine::new(buffer_size);
    let mut delay_right = ModulatedDelayLine::new(buffer_size);

    let num_samples = stereo.left.len();
    let mut output_left = Vec::with_capacity(num_samples);
    let mut output_right = Vec::with_capacity(num_samples);

    let dry = 1.0 - wet;
    let voice_gain = wet / voices as f64;

    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        delay_left.write(in_left);
        delay_right.write(in_right);

        let mut out_left = in_left * dry;
        let mut out_right = in_right * dry;

        // Generate multiple voices with phase offsets
        for voice_idx in 0..voices {
            let phase_offset = (voice_idx as f64 * TWO_PI) / voices as f64;
            let t = i as f64 / sample_rate;
            let lfo = ((TWO_PI * rate * t) + phase_offset).sin();

            // Modulate delay time
            let modulation = depth * max_delay_samples * (lfo * 0.5 + 0.5);
            let delay_samples = base_delay_samples + modulation;

            // Slightly different LFO for stereo width
            let lfo_right = ((TWO_PI * rate * t) + phase_offset + 0.5).sin();
            let modulation_right = depth * max_delay_samples * (lfo_right * 0.5 + 0.5);
            let delay_samples_right = base_delay_samples + modulation_right;

            out_left += delay_left.read_interpolated(delay_samples) * voice_gain;
            out_right += delay_right.read_interpolated(delay_samples_right) * voice_gain;
        }

        output_left.push(out_left);
        output_right.push(out_right);
    }

    stereo.left = output_left;
    stereo.right = output_right;

    Ok(())
}

/// Simple allpass filter for phaser.
struct AllpassFilter {
    feedback: f64,
    buffer: Vec<f64>,
    index: usize,
}

impl AllpassFilter {
    fn new(delay_samples: usize, feedback: f64) -> Self {
        Self {
            feedback,
            buffer: vec![0.0; delay_samples.max(1)],
            index: 0,
        }
    }

    fn process(&mut self, input: f64) -> f64 {
        let delayed = self.buffer[self.index];
        let output = -input + delayed;

        self.buffer[self.index] = input + delayed * self.feedback;

        self.index = (self.index + 1) % self.buffer.len();

        output
    }
}

/// Applies phaser effect to stereo audio.
pub fn apply_phaser(
    stereo: &mut StereoOutput,
    rate: f64,
    depth: f64,
    stages: u8,
    wet: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(0.1..=10.0).contains(&rate) {
        return Err(AudioError::invalid_param(
            "phaser.rate",
            format!("must be 0.1-10.0 Hz, got {}", rate),
        ));
    }
    if !(0.0..=1.0).contains(&depth) {
        return Err(AudioError::invalid_param(
            "phaser.depth",
            format!("must be 0.0-1.0, got {}", depth),
        ));
    }
    if !(2..=12).contains(&stages) {
        return Err(AudioError::invalid_param(
            "phaser.stages",
            format!("must be 2-12, got {}", stages),
        ));
    }
    if !(0.0..=1.0).contains(&wet) {
        return Err(AudioError::invalid_param(
            "phaser.wet",
            format!("must be 0.0-1.0, got {}", wet),
        ));
    }

    // Create allpass filter stages
    let min_delay = 2;
    let max_delay = 10;

    let num_samples = stereo.left.len();
    let mut output_left = Vec::with_capacity(num_samples);
    let mut output_right = Vec::with_capacity(num_samples);

    let dry = 1.0 - wet;

    // Create separate allpass chains for left and right
    let mut allpasses_left: Vec<_> = (0..stages)
        .map(|i| {
            let delay = min_delay + (i as usize * (max_delay - min_delay)) / stages.max(1) as usize;
            AllpassFilter::new(delay, 0.7)
        })
        .collect();

    let mut allpasses_right: Vec<_> = (0..stages)
        .map(|i| {
            let delay = min_delay + (i as usize * (max_delay - min_delay)) / stages.max(1) as usize;
            AllpassFilter::new(delay + 1, 0.7) // Slight offset for stereo
        })
        .collect();

    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // LFO modulation (for future use if we want to modulate allpass parameters)
        let _t = i as f64 / sample_rate;
        let _lfo = ((TWO_PI * rate * _t).sin() * 0.5 + 0.5) * depth;

        // Process through allpass chain
        let mut processed_left = in_left;
        for allpass in &mut allpasses_left {
            processed_left = allpass.process(processed_left);
        }

        let mut processed_right = in_right;
        for allpass in &mut allpasses_right {
            processed_right = allpass.process(processed_right);
        }

        // Mix wet/dry
        output_left.push(in_left * dry + processed_left * wet);
        output_right.push(in_right * dry + processed_right * wet);
    }

    stereo.left = output_left;
    stereo.right = output_right;

    Ok(())
}
