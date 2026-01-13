//! Granular synthesis implementation.
//!
//! Granular synthesis generates sound by combining many short audio fragments called "grains".
//! Each grain can have random variations in pitch, position, and stereo placement.

use rand::Rng;
use rand_pcg::Pcg32;
use speccade_spec::recipe::audio::{GranularSource, NoiseType, Waveform};

use crate::oscillator::PhaseAccumulator;
use crate::synthesis::Synthesizer;

use std::f64::consts::PI;

/// Granular synthesizer.
#[derive(Debug, Clone)]
pub struct GranularSynth {
    /// Source material for grains.
    source: GranularSource,
    /// Grain size in milliseconds.
    grain_size_ms: f64,
    /// Grains per second.
    grain_density: f64,
    /// Random pitch variation in semitones.
    pitch_spread: f64,
    /// Random position jitter (0.0-1.0).
    position_spread: f64,
    /// Stereo spread (0.0-1.0).
    pan_spread: f64,
}

impl GranularSynth {
    /// Creates a new granular synthesizer.
    ///
    /// # Arguments
    /// * `source` - Source material for grains
    /// * `grain_size_ms` - Grain size in milliseconds (10-500ms)
    /// * `grain_density` - Grains per second (1-100)
    /// * `pitch_spread` - Random pitch variation in semitones
    /// * `position_spread` - Random position jitter (0.0-1.0)
    /// * `pan_spread` - Stereo spread (0.0-1.0)
    pub fn new(
        source: GranularSource,
        grain_size_ms: f64,
        grain_density: f64,
        pitch_spread: f64,
        position_spread: f64,
        pan_spread: f64,
    ) -> Self {
        Self {
            source,
            grain_size_ms,
            grain_density,
            pitch_spread,
            position_spread,
            pan_spread,
        }
    }
}

impl Synthesizer for GranularSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        // For stereo output, we'll generate interleaved samples [L, R, L, R, ...]
        // For mono output (pan_spread == 0.0), we'll generate [M, M, M, ...]
        let is_stereo = self.pan_spread > 0.0;
        let output_samples = if is_stereo {
            num_samples * 2
        } else {
            num_samples
        };

        let mut output = vec![0.0; output_samples];

        // Calculate grain parameters
        let grain_size_samples = ((self.grain_size_ms / 1000.0) * sample_rate) as usize;
        if grain_size_samples == 0 {
            return output;
        }

        // Calculate interval between grain starts
        let grain_interval_samples = (sample_rate / self.grain_density) as usize;
        if grain_interval_samples == 0 {
            return output;
        }

        // Generate grains
        let mut current_sample = 0;
        while current_sample < num_samples {
            // Apply position jitter
            let jitter = if self.position_spread > 0.0 {
                let jitter_range =
                    (self.position_spread * grain_interval_samples as f64 * 0.5) as i32;
                rng.gen_range(-jitter_range..=jitter_range)
            } else {
                0
            };

            let grain_start =
                (current_sample as i32 + jitter).max(0).min(num_samples as i32 - 1) as usize;

            // Generate pitch shift for this grain
            let pitch_shift = if self.pitch_spread > 0.0 {
                let semitones = rng.gen_range(-self.pitch_spread..=self.pitch_spread);
                2.0_f64.powf(semitones / 12.0)
            } else {
                1.0
            };

            // Generate pan for this grain (0.0 = center, -1.0 = left, 1.0 = right)
            let pan = if is_stereo {
                rng.gen_range(-self.pan_spread..=self.pan_spread)
            } else {
                0.0
            };

            // Convert pan to left/right gains
            let (left_gain, right_gain) = if is_stereo {
                // Equal power panning
                let pan_angle = (pan + 1.0) * 0.5 * PI / 2.0;
                (pan_angle.cos(), pan_angle.sin())
            } else {
                (1.0, 1.0)
            };

            // Generate grain samples
            let grain_samples = self.generate_grain(grain_size_samples, sample_rate, pitch_shift, rng);

            // Apply Hann window and add to output buffer (overlap-add)
            for (i, &grain_sample) in grain_samples.iter().enumerate() {
                let output_idx = grain_start + i;
                if output_idx >= num_samples {
                    break;
                }

                // Apply Hann window
                let window = 0.5 * (1.0 - (2.0 * PI * i as f64 / grain_size_samples as f64).cos());
                let windowed_sample = grain_sample * window;

                if is_stereo {
                    // Stereo output
                    let left_idx = output_idx * 2;
                    let right_idx = output_idx * 2 + 1;
                    output[left_idx] += windowed_sample * left_gain;
                    output[right_idx] += windowed_sample * right_gain;
                } else {
                    // Mono output
                    output[output_idx] += windowed_sample;
                }
            }

            current_sample += grain_interval_samples;
        }

        // Normalize by approximate grain overlap
        let overlap_factor = (grain_size_samples as f64 / grain_interval_samples as f64).max(1.0);
        let normalization = 1.0 / overlap_factor.sqrt();

        for sample in &mut output {
            *sample *= normalization;
        }

        output
    }
}

impl GranularSynth {
    /// Generates a single grain of audio.
    ///
    /// # Arguments
    /// * `grain_size_samples` - Size of the grain in samples
    /// * `sample_rate` - Audio sample rate
    /// * `pitch_shift` - Pitch shift multiplier (1.0 = no shift)
    /// * `rng` - Random number generator
    ///
    /// # Returns
    /// Vector of grain samples
    fn generate_grain(
        &self,
        grain_size_samples: usize,
        sample_rate: f64,
        pitch_shift: f64,
        rng: &mut Pcg32,
    ) -> Vec<f64> {
        match &self.source {
            GranularSource::Noise { noise_type } => {
                self.generate_noise_grain(grain_size_samples, noise_type, rng)
            }
            GranularSource::Tone {
                waveform,
                frequency,
            } => self.generate_tone_grain(
                grain_size_samples,
                sample_rate,
                *frequency * pitch_shift,
                waveform,
            ),
            GranularSource::Formant {
                frequency,
                formant_freq,
            } => self.generate_formant_grain(
                grain_size_samples,
                sample_rate,
                *frequency * pitch_shift,
                *formant_freq,
            ),
        }
    }

    /// Generates a noise-based grain.
    fn generate_noise_grain(
        &self,
        grain_size_samples: usize,
        noise_type: &NoiseType,
        rng: &mut Pcg32,
    ) -> Vec<f64> {
        match noise_type {
            NoiseType::White => crate::oscillator::white_noise(rng, grain_size_samples),
            NoiseType::Pink => crate::oscillator::pink_noise(rng, grain_size_samples),
            NoiseType::Brown => crate::oscillator::brown_noise(rng, grain_size_samples),
        }
    }

    /// Generates a tone-based grain.
    fn generate_tone_grain(
        &self,
        grain_size_samples: usize,
        sample_rate: f64,
        frequency: f64,
        waveform: &Waveform,
    ) -> Vec<f64> {
        let mut phase_acc = PhaseAccumulator::new(sample_rate);
        let mut samples = Vec::with_capacity(grain_size_samples);

        for _ in 0..grain_size_samples {
            let phase = phase_acc.advance(frequency);
            let sample = match waveform {
                Waveform::Sine => crate::oscillator::sine(phase),
                Waveform::Square | Waveform::Pulse => crate::oscillator::square(phase, 0.5),
                Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                Waveform::Triangle => crate::oscillator::triangle(phase),
            };
            samples.push(sample);
        }

        samples
    }

    /// Generates a formant-based grain.
    ///
    /// Formant synthesis creates vowel-like sounds by modulating a carrier with a formant frequency.
    fn generate_formant_grain(
        &self,
        grain_size_samples: usize,
        sample_rate: f64,
        frequency: f64,
        formant_freq: f64,
    ) -> Vec<f64> {
        let mut carrier_phase = PhaseAccumulator::new(sample_rate);
        let mut formant_phase = PhaseAccumulator::new(sample_rate);
        let mut samples = Vec::with_capacity(grain_size_samples);

        for _ in 0..grain_size_samples {
            let carrier = crate::oscillator::sawtooth(carrier_phase.advance(frequency));
            let formant = crate::oscillator::sine(formant_phase.advance(formant_freq));

            // Ring modulation
            let sample = carrier * (0.5 + 0.5 * formant);
            samples.push(sample);
        }

        samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_granular_basic() {
        let mut rng = create_rng(42);
        let source = GranularSource::Tone {
            waveform: Waveform::Sine,
            frequency: 440.0,
        };

        let synth = GranularSynth::new(source, 50.0, 10.0, 0.0, 0.0, 0.0);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        assert!(samples.iter().any(|&s| s.abs() > 0.0));
    }

    #[test]
    fn test_granular_determinism() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let source = GranularSource::Noise {
            noise_type: NoiseType::White,
        };

        let synth = GranularSynth::new(source.clone(), 80.0, 20.0, 0.5, 0.3, 0.0);
        let samples1 = synth.synthesize(4410, 44100.0, &mut rng1);

        let synth2 = GranularSynth::new(source, 80.0, 20.0, 0.5, 0.3, 0.0);
        let samples2 = synth2.synthesize(4410, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_granular_stereo() {
        let mut rng = create_rng(42);
        let source = GranularSource::Tone {
            waveform: Waveform::Sawtooth,
            frequency: 220.0,
        };

        let synth = GranularSynth::new(source, 80.0, 20.0, 0.0, 0.0, 0.8);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        // Stereo output should have 2x samples (interleaved L/R)
        assert_eq!(samples.len(), 4410 * 2);
    }

    #[test]
    fn test_granular_pitch_spread() {
        let mut rng = create_rng(42);
        let source = GranularSource::Tone {
            waveform: Waveform::Sine,
            frequency: 440.0,
        };

        let synth = GranularSynth::new(source, 50.0, 10.0, 2.0, 0.0, 0.0);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        // Should produce varied pitch content
        assert!(samples.iter().any(|&s| s.abs() > 0.0));
    }

    #[test]
    fn test_granular_formant() {
        let mut rng = create_rng(42);
        let source = GranularSource::Formant {
            frequency: 100.0,
            formant_freq: 800.0,
        };

        let synth = GranularSynth::new(source, 80.0, 15.0, 0.0, 0.0, 0.0);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        assert!(samples.iter().any(|&s| s.abs() > 0.0));
    }
}
