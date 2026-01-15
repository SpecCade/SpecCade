//! Rotary speaker (Leslie) effect implementation.
//!
//! Simulates a rotating speaker cabinet with:
//! - Amplitude modulation (tremolo at rotation rate)
//! - Stereo pan modulation (circular panning)
//! - Doppler effect (short modulated delay for pitch wobble)

use crate::error::{AudioError, AudioResult};
use crate::mixer::StereoOutput;
use std::f64::consts::PI;

const TWO_PI: f64 = 2.0 * PI;

/// Delay line with interpolated read for Doppler effect.
struct DelayLine {
    buffer: Vec<f64>,
    write_pos: usize,
}

impl DelayLine {
    fn new(max_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_samples.max(4)],
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

/// Applies rotary speaker effect to stereo audio.
///
/// # Arguments
/// * `stereo` - Stereo audio to process in-place
/// * `rate` - Rotation rate in Hz (0.5-10.0 typical)
/// * `depth` - Effect intensity (0.0-1.0)
/// * `wet` - Wet/dry mix (0.0-1.0)
/// * `sample_rate` - Sample rate in Hz
///
/// # Algorithm
/// The Leslie/rotary speaker effect combines:
/// 1. Amplitude modulation - tremolo at rotation rate
/// 2. Stereo pan modulation - circular panning (90 degree phase offset)
/// 3. Doppler effect - short modulated delay for pitch wobble
pub fn apply(
    stereo: &mut StereoOutput,
    rate: f64,
    depth: f64,
    wet: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    // Validate parameters
    if !(0.1..=20.0).contains(&rate) {
        return Err(AudioError::invalid_param(
            "rotary_speaker.rate",
            format!("must be 0.1-20.0 Hz, got {}", rate),
        ));
    }
    if !(0.0..=1.0).contains(&depth) {
        return Err(AudioError::invalid_param(
            "rotary_speaker.depth",
            format!("must be 0.0-1.0, got {}", depth),
        ));
    }
    if !(0.0..=1.0).contains(&wet) {
        return Err(AudioError::invalid_param(
            "rotary_speaker.wet",
            format!("must be 0.0-1.0, got {}", wet),
        ));
    }

    let num_samples = stereo.left.len();
    if num_samples == 0 {
        return Ok(());
    }

    // Doppler delay parameters (in milliseconds)
    // Base delay ~3ms, modulation range +/- 2ms scaled by depth
    let base_delay_ms = 3.0;
    let max_delay_mod_ms = 2.0;

    let base_delay_samples = (base_delay_ms / 1000.0) * sample_rate;
    let max_delay_mod_samples = (max_delay_mod_ms / 1000.0) * sample_rate;

    // Buffer size needs to accommodate max delay
    let buffer_size = (base_delay_samples + max_delay_mod_samples * depth + 2.0).ceil() as usize;

    let mut delay_left = DelayLine::new(buffer_size);
    let mut delay_right = DelayLine::new(buffer_size);

    let mut output_left = Vec::with_capacity(num_samples);
    let mut output_right = Vec::with_capacity(num_samples);

    let dry = 1.0 - wet;

    // Phase accumulator for deterministic LFO
    let mut phase = 0.0;
    let phase_increment = rate / sample_rate;

    for i in 0..num_samples {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // LFO values with 90 degree (PI/2) phase offset for stereo
        let lfo_l = (TWO_PI * phase).sin();
        let lfo_r = (TWO_PI * phase + PI / 2.0).sin();

        // Amplitude modulation (tremolo)
        // Scale depth to 0.3 for subtle tremolo effect
        let amp_mod_l = 1.0 + depth * 0.3 * lfo_l;
        let amp_mod_r = 1.0 + depth * 0.3 * lfo_r;

        // Doppler delay modulation
        let delay_samples_l = base_delay_samples + depth * max_delay_mod_samples * lfo_l;
        let delay_samples_r = base_delay_samples + depth * max_delay_mod_samples * lfo_r;

        // Write input to delay lines
        delay_left.write(in_left);
        delay_right.write(in_right);

        // Read delayed samples
        let delayed_l = delay_left.read_interpolated(delay_samples_l);
        let delayed_r = delay_right.read_interpolated(delay_samples_r);

        // Apply amplitude modulation to delayed signal
        let wet_l = delayed_l * amp_mod_l;
        let wet_r = delayed_r * amp_mod_r;

        // Mix wet and dry signals
        output_left.push(wet * wet_l + dry * in_left);
        output_right.push(wet * wet_r + dry * in_right);

        // Advance phase
        phase += phase_increment;
        if phase >= 1.0 {
            phase -= 1.0;
        }
    }

    stereo.left = output_left;
    stereo.right = output_right;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotary_speaker_basic() {
        let sample_rate = 44100.0;
        let num_samples = 4410; // 100ms

        // Generate a simple sine wave
        let freq = 440.0;
        let samples: Vec<f64> = (0..num_samples)
            .map(|i| (TWO_PI * freq * i as f64 / sample_rate).sin() * 0.5)
            .collect();

        let mut stereo = StereoOutput {
            left: samples.clone(),
            right: samples,
        };

        // Apply rotary speaker effect
        apply(&mut stereo, 5.0, 0.7, 0.5, sample_rate).unwrap();

        // Output should have same length
        assert_eq!(stereo.left.len(), num_samples);
        assert_eq!(stereo.right.len(), num_samples);

        // Output should not be silent
        let max_left = stereo.left.iter().fold(0.0_f64, |a, &b| a.max(b.abs()));
        let max_right = stereo.right.iter().fold(0.0_f64, |a, &b| a.max(b.abs()));
        assert!(max_left > 0.1);
        assert!(max_right > 0.1);
    }

    #[test]
    fn test_rotary_speaker_stereo_difference() {
        let sample_rate = 44100.0;
        let num_samples = 4410;

        // Mono input
        let samples: Vec<f64> = (0..num_samples)
            .map(|i| (TWO_PI * 220.0 * i as f64 / sample_rate).sin() * 0.5)
            .collect();

        let mut stereo = StereoOutput {
            left: samples.clone(),
            right: samples,
        };

        apply(&mut stereo, 6.0, 0.8, 1.0, sample_rate).unwrap();

        // With 90 degree phase offset, L and R should differ
        let mut has_difference = false;
        for i in 0..num_samples {
            if (stereo.left[i] - stereo.right[i]).abs() > 0.001 {
                has_difference = true;
                break;
            }
        }
        assert!(
            has_difference,
            "Rotary effect should create stereo difference"
        );
    }

    #[test]
    fn test_rotary_speaker_dry_passthrough() {
        let sample_rate = 44100.0;
        let num_samples = 1000;

        let samples: Vec<f64> = (0..num_samples).map(|i| i as f64 / 1000.0).collect();

        let mut stereo = StereoOutput {
            left: samples.clone(),
            right: samples.clone(),
        };

        // With wet=0, output should equal input
        apply(&mut stereo, 5.0, 0.5, 0.0, sample_rate).unwrap();

        for (out, orig) in stereo.left.iter().zip(samples.iter()) {
            assert!(
                (out - orig).abs() < 1e-10,
                "Dry signal should pass through unchanged"
            );
        }
    }

    #[test]
    fn test_rotary_speaker_parameter_validation() {
        let mut stereo = StereoOutput {
            left: vec![0.0; 100],
            right: vec![0.0; 100],
        };

        // Rate too low
        assert!(apply(&mut stereo, 0.05, 0.5, 0.5, 44100.0).is_err());

        // Rate too high
        assert!(apply(&mut stereo, 25.0, 0.5, 0.5, 44100.0).is_err());

        // Depth out of range
        assert!(apply(&mut stereo, 5.0, -0.1, 0.5, 44100.0).is_err());
        assert!(apply(&mut stereo, 5.0, 1.5, 0.5, 44100.0).is_err());

        // Wet out of range
        assert!(apply(&mut stereo, 5.0, 0.5, -0.1, 44100.0).is_err());
        assert!(apply(&mut stereo, 5.0, 0.5, 1.5, 44100.0).is_err());

        // Valid parameters should work
        assert!(apply(&mut stereo, 5.0, 0.5, 0.5, 44100.0).is_ok());
    }

    #[test]
    fn test_rotary_speaker_deterministic() {
        let sample_rate = 44100.0;
        let num_samples = 2205;

        let samples: Vec<f64> = (0..num_samples)
            .map(|i| (TWO_PI * 330.0 * i as f64 / sample_rate).sin() * 0.5)
            .collect();

        let mut stereo1 = StereoOutput {
            left: samples.clone(),
            right: samples.clone(),
        };

        let mut stereo2 = StereoOutput {
            left: samples.clone(),
            right: samples,
        };

        apply(&mut stereo1, 3.0, 0.6, 0.7, sample_rate).unwrap();
        apply(&mut stereo2, 3.0, 0.6, 0.7, sample_rate).unwrap();

        // Results should be identical
        for i in 0..num_samples {
            assert!(
                (stereo1.left[i] - stereo2.left[i]).abs() < 1e-15,
                "Left channel should be deterministic"
            );
            assert!(
                (stereo1.right[i] - stereo2.right[i]).abs() < 1e-15,
                "Right channel should be deterministic"
            );
        }
    }
}
