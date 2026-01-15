//! Wavetable synthesis with morphing and unison.
//!
//! This module implements wavetable synthesis where pre-computed waveform
//! frames can be smoothly morphed between by adjusting the position parameter.

use std::f64::consts::PI;

use rand_pcg::Pcg32;
use speccade_spec::recipe::audio::WavetableSource;

use super::{SweepCurve, Synthesizer};
use crate::oscillator::PhaseAccumulator;

/// Number of samples per wavetable frame.
const FRAME_SIZE: usize = 256;

/// Number of frames per wavetable.
const NUM_FRAMES: usize = 64;

/// Wavetable synthesizer.
#[derive(Debug, Clone)]
pub struct WavetableSynth {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Position in wavetable (0.0-1.0).
    pub position: f64,
    /// Optional position sweep.
    pub position_sweep: Option<PositionSweep>,
    /// Number of unison voices (1-8).
    pub voices: u8,
    /// Detune amount in cents for unison.
    pub detune: f64,
    /// Pre-generated wavetable data.
    wavetable: Vec<Vec<f64>>,
}

/// Position sweep parameters.
#[derive(Debug, Clone, Copy)]
pub struct PositionSweep {
    /// Starting position (0.0-1.0).
    pub start_position: f64,
    /// Ending position (0.0-1.0).
    pub end_position: f64,
    /// Sweep curve type.
    pub curve: SweepCurve,
}

impl WavetableSynth {
    /// Creates a new wavetable synthesizer.
    pub fn new(
        source: WavetableSource,
        frequency: f64,
        position: f64,
        position_sweep: Option<PositionSweep>,
        voices: Option<u8>,
        detune: Option<f64>,
    ) -> Self {
        let wavetable = generate_wavetable(source);
        let voices = voices.unwrap_or(1).clamp(1, 8);
        let detune = detune.unwrap_or(0.0);

        Self {
            frequency,
            position: position.clamp(0.0, 1.0),
            position_sweep,
            voices,
            detune,
            wavetable,
        }
    }
}

impl Synthesizer for WavetableSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = vec![0.0; num_samples];

        // Calculate detune spread for unison voices
        let detune_spread: Vec<f64> = if self.voices > 1 {
            (0..self.voices)
                .map(|i| {
                    let offset = (i as f64 - (self.voices - 1) as f64 / 2.0)
                        / ((self.voices - 1) as f64 / 2.0);
                    2.0_f64.powf((offset * self.detune) / 1200.0)
                })
                .collect()
        } else {
            vec![1.0]
        };

        // Generate each unison voice
        for detune_mult in detune_spread.iter() {
            let mut phase_acc = PhaseAccumulator::new(sample_rate);
            let voice_freq = self.frequency * detune_mult;

            for (i, out_sample) in output.iter_mut().enumerate().take(num_samples) {
                // Calculate position (with optional sweep)
                let position = if let Some(ref sweep) = self.position_sweep {
                    let t = i as f64 / num_samples.max(1) as f64;
                    sweep
                        .curve
                        .interpolate(sweep.start_position, sweep.end_position, t)
                } else {
                    self.position
                };

                // Get sample from wavetable at this position
                let phase = phase_acc.advance(voice_freq);
                let sample = sample_wavetable(&self.wavetable, phase, position);

                *out_sample += sample;
            }
        }

        // Normalize by number of voices to prevent clipping
        let voice_count = self.voices as f64;
        for sample in &mut output {
            *sample /= voice_count;
        }

        output
    }
}

/// Samples from the wavetable at a given phase and position.
///
/// # Arguments
/// * `wavetable` - The wavetable (list of frames)
/// * `phase` - Current phase in radians (0 to 2*PI)
/// * `position` - Position in wavetable (0.0-1.0)
///
/// # Returns
/// Interpolated sample value
fn sample_wavetable(wavetable: &[Vec<f64>], phase: f64, position: f64) -> f64 {
    let position = position.clamp(0.0, 1.0);

    // Calculate which frames to interpolate between
    let frame_pos = position * (NUM_FRAMES - 1) as f64;
    let frame_idx = frame_pos.floor() as usize;
    let frame_frac = frame_pos.fract();

    // Get the two frames to interpolate between
    let frame1 = &wavetable[frame_idx.min(NUM_FRAMES - 1)];
    let frame2 = &wavetable[(frame_idx + 1).min(NUM_FRAMES - 1)];

    // Calculate sample position within frame
    let normalized_phase = (phase / (2.0 * PI)).fract();
    let sample_pos = normalized_phase * FRAME_SIZE as f64;
    let sample_idx = sample_pos.floor() as usize;
    let sample_frac = sample_pos.fract();

    // Linear interpolation within each frame
    let sample1_a = frame1[sample_idx % FRAME_SIZE];
    let sample1_b = frame1[(sample_idx + 1) % FRAME_SIZE];
    let sample1 = sample1_a + (sample1_b - sample1_a) * sample_frac;

    let sample2_a = frame2[sample_idx % FRAME_SIZE];
    let sample2_b = frame2[(sample_idx + 1) % FRAME_SIZE];
    let sample2 = sample2_a + (sample2_b - sample2_a) * sample_frac;

    // Linear interpolation between frames
    sample1 + (sample2 - sample1) * frame_frac
}

/// Generates a complete wavetable for the given source.
///
/// Returns a vector of frames, each containing FRAME_SIZE samples.
fn generate_wavetable(source: WavetableSource) -> Vec<Vec<f64>> {
    match source {
        WavetableSource::Basic => generate_basic_wavetable(),
        WavetableSource::Analog => generate_analog_wavetable(),
        WavetableSource::Digital => generate_digital_wavetable(),
        WavetableSource::Pwm => generate_pwm_wavetable(),
        WavetableSource::Formant => generate_formant_wavetable(),
        WavetableSource::Organ => generate_organ_wavetable(),
    }
}

/// Generates basic wavetable: sine -> saw -> square -> pulse.
fn generate_basic_wavetable() -> Vec<Vec<f64>> {
    let mut frames = Vec::with_capacity(NUM_FRAMES);

    for frame_idx in 0..NUM_FRAMES {
        let t = frame_idx as f64 / (NUM_FRAMES - 1) as f64;
        let mut frame = vec![0.0; FRAME_SIZE];

        for (i, frame_sample) in frame.iter_mut().enumerate().take(FRAME_SIZE) {
            let phase = 2.0 * PI * i as f64 / FRAME_SIZE as f64;

            let sample = if t < 0.33 {
                // Sine to saw
                let mix = t / 0.33;
                let sine = phase.sin();
                let saw = 2.0 * (phase / (2.0 * PI)) - 1.0;
                sine * (1.0 - mix) + saw * mix
            } else if t < 0.66 {
                // Saw to square
                let mix = (t - 0.33) / 0.33;
                let saw = 2.0 * (phase / (2.0 * PI)) - 1.0;
                let square = if phase < PI { 1.0 } else { -1.0 };
                saw * (1.0 - mix) + square * mix
            } else {
                // Square to pulse
                let mix = (t - 0.66) / 0.34;
                let duty = 0.5 - mix * 0.4; // Duty cycle from 0.5 to 0.1
                if phase / (2.0 * PI) < duty {
                    1.0
                } else {
                    -1.0
                }
            };

            *frame_sample = sample;
        }

        frames.push(frame);
    }

    frames
}

/// Generates analog wavetable with warm, classic waveforms.
fn generate_analog_wavetable() -> Vec<Vec<f64>> {
    let mut frames = Vec::with_capacity(NUM_FRAMES);

    for frame_idx in 0..NUM_FRAMES {
        let t = frame_idx as f64 / (NUM_FRAMES - 1) as f64;
        let mut frame = vec![0.0; FRAME_SIZE];

        for (i, frame_sample) in frame.iter_mut().enumerate().take(FRAME_SIZE) {
            let phase = 2.0 * PI * i as f64 / FRAME_SIZE as f64;

            // Mix between sine, triangle, and saw with harmonic content
            let harmonics = (t * 8.0).floor() as usize + 1;
            let mut sample = 0.0;

            for h in 1..=harmonics {
                let amp = 1.0 / h as f64;
                sample += amp * (phase * h as f64).sin();
            }

            // Normalize
            sample *= 0.5;
            *frame_sample = sample.clamp(-1.0, 1.0);
        }

        frames.push(frame);
    }

    frames
}

/// Generates digital wavetable with harsh, bright tones.
fn generate_digital_wavetable() -> Vec<Vec<f64>> {
    let mut frames = Vec::with_capacity(NUM_FRAMES);

    for frame_idx in 0..NUM_FRAMES {
        let t = frame_idx as f64 / (NUM_FRAMES - 1) as f64;
        let mut frame = vec![0.0; FRAME_SIZE];

        for (i, frame_sample) in frame.iter_mut().enumerate().take(FRAME_SIZE) {
            let phase = 2.0 * PI * i as f64 / FRAME_SIZE as f64;

            // Mix of high harmonics with sharp transitions
            let mut sample = 0.0;
            let num_harmonics = ((t * 16.0) as usize + 4).min(32);

            for h in 1..=num_harmonics {
                let amp = 1.0 / (h as f64).sqrt();
                sample += amp * (phase * h as f64).sin();
            }

            // Add some bit-crushing effect
            let bit_depth = 16.0 - t * 8.0;
            let quantize = 2.0_f64.powf(bit_depth);
            sample = (sample * quantize).round() / quantize;

            *frame_sample = sample.clamp(-1.0, 1.0);
        }

        frames.push(frame);
    }

    frames
}

/// Generates PWM wavetable with pulse width modulation.
fn generate_pwm_wavetable() -> Vec<Vec<f64>> {
    let mut frames = Vec::with_capacity(NUM_FRAMES);

    for frame_idx in 0..NUM_FRAMES {
        let t = frame_idx as f64 / (NUM_FRAMES - 1) as f64;
        let mut frame = vec![0.0; FRAME_SIZE];

        // Duty cycle sweeps from 0.05 to 0.95
        let duty = 0.05 + t * 0.9;

        for (i, frame_sample) in frame.iter_mut().enumerate().take(FRAME_SIZE) {
            let phase_norm = i as f64 / FRAME_SIZE as f64;
            *frame_sample = if phase_norm < duty { 1.0 } else { -1.0 };
        }

        frames.push(frame);
    }

    frames
}

/// Generates formant wavetable with vocal-like formants.
fn generate_formant_wavetable() -> Vec<Vec<f64>> {
    let mut frames = Vec::with_capacity(NUM_FRAMES);

    // Formant frequencies for different vowels (simplified)
    let formants = [
        (800.0, 1200.0), // "ah"
        (400.0, 2000.0), // "ee"
        (500.0, 1000.0), // "oo"
        (600.0, 1500.0), // "oh"
        (700.0, 1800.0), // "eh"
    ];

    for frame_idx in 0..NUM_FRAMES {
        let t = frame_idx as f64 / (NUM_FRAMES - 1) as f64;
        let mut frame = vec![0.0; FRAME_SIZE];

        // Interpolate between formants
        let formant_pos = t * (formants.len() - 1) as f64;
        let formant_idx = formant_pos.floor() as usize;
        let formant_frac = formant_pos.fract();

        let (f1_a, f2_a) = formants[formant_idx.min(formants.len() - 1)];
        let (f1_b, f2_b) = formants[(formant_idx + 1).min(formants.len() - 1)];

        let f1 = f1_a + (f1_b - f1_a) * formant_frac;
        let f2 = f2_a + (f2_b - f2_a) * formant_frac;

        for (i, frame_sample) in frame.iter_mut().enumerate().take(FRAME_SIZE) {
            let phase = 2.0 * PI * i as f64 / FRAME_SIZE as f64;

            // Fundamental + two formants
            let fundamental = phase.sin();
            let formant1 = 0.5 * (phase * (f1 / 100.0)).sin();
            let formant2 = 0.3 * (phase * (f2 / 100.0)).sin();

            *frame_sample = (fundamental + formant1 + formant2).clamp(-1.0, 1.0);
        }

        frames.push(frame);
    }

    frames
}

/// Generates organ wavetable with drawbar harmonics.
fn generate_organ_wavetable() -> Vec<Vec<f64>> {
    let mut frames = Vec::with_capacity(NUM_FRAMES);

    // Drawbar settings (9 drawbars with different harmonic content)
    // Each frame uses different drawbar settings
    for frame_idx in 0..NUM_FRAMES {
        let t = frame_idx as f64 / (NUM_FRAMES - 1) as f64;
        let mut frame = vec![0.0; FRAME_SIZE];

        // Drawbar amplitudes (simulate different registrations)
        let drawbars = [
            0.5 + t * 0.5,   // 16' (sub-octave)
            0.8,             // 8' (fundamental)
            0.3 + t * 0.4,   // 4' (octave)
            0.2,             // 2 2/3' (fifth)
            0.4 * (1.0 - t), // 2' (two octaves)
            0.2,             // 1 3/5' (third)
            0.3 * t,         // 1 1/3' (fifth + octave)
            0.1,             // 1' (three octaves)
            0.2 * (1.0 - t), // 2/3' (twelfth)
        ];

        // Harmonic ratios for drawbars
        let ratios = [0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0];

        for (i, frame_sample) in frame.iter_mut().enumerate().take(FRAME_SIZE) {
            let phase = 2.0 * PI * i as f64 / FRAME_SIZE as f64;
            let mut sample = 0.0;

            for (amp, ratio) in drawbars.iter().zip(ratios.iter()) {
                sample += amp * (phase * ratio).sin();
            }

            // Normalize
            sample *= 0.2;
            *frame_sample = sample.clamp(-1.0, 1.0);
        }

        frames.push(frame);
    }

    frames
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_wavetable_basic() {
        let synth = WavetableSynth::new(WavetableSource::Basic, 440.0, 0.0, None, None, None);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Check that all samples are in range
        for &s in &samples {
            assert!(s.is_finite());
            assert!(s.abs() <= 1.1); // Allow slight overshoot
        }
    }

    #[test]
    fn test_wavetable_with_sweep() {
        let sweep = PositionSweep {
            start_position: 0.0,
            end_position: 1.0,
            curve: SweepCurve::Linear,
        };

        let synth =
            WavetableSynth::new(WavetableSource::Analog, 440.0, 0.0, Some(sweep), None, None);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_wavetable_unison() {
        let synth = WavetableSynth::new(
            WavetableSource::Analog,
            440.0,
            0.5,
            None,
            Some(4),
            Some(15.0),
        );
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_all_wavetable_sources() {
        let sources = [
            WavetableSource::Basic,
            WavetableSource::Analog,
            WavetableSource::Digital,
            WavetableSource::Pwm,
            WavetableSource::Formant,
            WavetableSource::Organ,
        ];

        for source in sources {
            let synth = WavetableSynth::new(source, 440.0, 0.5, None, None, None);
            let mut rng = create_rng(42);
            let samples = synth.synthesize(100, 44100.0, &mut rng);

            assert_eq!(samples.len(), 100);
            for &s in &samples {
                assert!(s.is_finite());
            }
        }
    }

    #[test]
    fn test_generate_wavetables() {
        // Test that all wavetable generators produce valid data
        let sources = [
            WavetableSource::Basic,
            WavetableSource::Analog,
            WavetableSource::Digital,
            WavetableSource::Pwm,
            WavetableSource::Formant,
            WavetableSource::Organ,
        ];

        for source in sources {
            let wavetable = generate_wavetable(source);
            assert_eq!(wavetable.len(), NUM_FRAMES);

            for frame in &wavetable {
                assert_eq!(frame.len(), FRAME_SIZE);
                for &sample in frame {
                    assert!(sample.is_finite());
                    assert!(sample.abs() <= 1.1); // Allow slight overshoot
                }
            }
        }
    }
}
