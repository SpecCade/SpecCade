//! Pulsar synthesis implementation.
//!
//! Pulsar synthesis generates sound using synchronized grain trains ("pulsarets").
//! Each pulsaret is a windowed waveform burst at a specified frequency, emitted
//! at a fixed pulse rate. The result is a distinctive rhythmic tonal texture where
//! both the fundamental frequency AND the pulse rate are heard as separate
//! perceptual elements.

use rand_pcg::Pcg32;
use speccade_spec::recipe::audio::Waveform;
use std::f64::consts::PI;

use crate::oscillator::PhaseAccumulator;
use crate::synthesis::Synthesizer;

/// Pulsar synthesizer.
#[derive(Debug, Clone)]
pub struct PulsarSynth {
    /// Fundamental frequency of each grain in Hz.
    frequency: f64,
    /// Grains per second (pulsaret rate).
    pulse_rate: f64,
    /// Duration of each grain in milliseconds.
    grain_size_ms: f64,
    /// Waveform shape for grains.
    shape: Waveform,
}

impl PulsarSynth {
    /// Creates a new pulsar synthesizer.
    ///
    /// # Arguments
    /// * `frequency` - Fundamental frequency of each grain in Hz
    /// * `pulse_rate` - Grains per second (pulsaret rate)
    /// * `grain_size_ms` - Duration of each grain in milliseconds
    /// * `shape` - Waveform shape for grains
    pub fn new(frequency: f64, pulse_rate: f64, grain_size_ms: f64, shape: Waveform) -> Self {
        Self {
            frequency,
            pulse_rate,
            grain_size_ms,
            shape,
        }
    }

    /// Generates a single pulsaret (grain) of audio.
    ///
    /// # Arguments
    /// * `grain_size_samples` - Size of the grain in samples
    /// * `sample_rate` - Audio sample rate
    ///
    /// # Returns
    /// Vector of grain samples with Hann window applied
    fn generate_pulsaret(&self, grain_size_samples: usize, sample_rate: f64) -> Vec<f64> {
        let mut phase_acc = PhaseAccumulator::new(sample_rate);
        let mut samples = Vec::with_capacity(grain_size_samples);

        for i in 0..grain_size_samples {
            let phase = phase_acc.advance(self.frequency);

            // Generate waveform sample
            let sample = match self.shape {
                Waveform::Sine => crate::oscillator::sine(phase),
                Waveform::Square | Waveform::Pulse => crate::oscillator::square(phase, 0.5),
                Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                Waveform::Triangle => crate::oscillator::triangle(phase),
            };

            // Apply Hann window for smooth onset/offset
            let window = 0.5 * (1.0 - (2.0 * PI * i as f64 / grain_size_samples as f64).cos());
            samples.push(sample * window);
        }

        samples
    }
}

impl Synthesizer for PulsarSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = vec![0.0; num_samples];

        // Calculate grain parameters
        let grain_size_samples = ((self.grain_size_ms / 1000.0) * sample_rate).round() as usize;
        if grain_size_samples == 0 {
            return output;
        }

        // Calculate interval between grain starts (in samples)
        let pulse_interval_samples = if self.pulse_rate > 0.0 {
            (sample_rate / self.pulse_rate).round() as usize
        } else {
            return output;
        };
        if pulse_interval_samples == 0 {
            return output;
        }

        // Pre-generate a single pulsaret (all pulsarets are identical for determinism)
        let pulsaret = self.generate_pulsaret(grain_size_samples, sample_rate);

        // Place pulsarets at regular intervals
        let mut current_sample = 0;
        while current_sample < num_samples {
            // Add pulsaret at current position
            for (i, &grain_sample) in pulsaret.iter().enumerate() {
                let output_idx = current_sample + i;
                if output_idx >= num_samples {
                    break;
                }
                output[output_idx] += grain_sample;
            }

            current_sample += pulse_interval_samples;
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_pulsar_basic() {
        let mut rng = create_rng(42);

        let synth = PulsarSynth::new(440.0, 20.0, 30.0, Waveform::Sine);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        // Should produce some non-zero output
        assert!(samples.iter().any(|&s| s.abs() > 0.0));
    }

    #[test]
    fn test_pulsar_determinism() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let synth1 = PulsarSynth::new(220.0, 15.0, 50.0, Waveform::Sawtooth);
        let synth2 = PulsarSynth::new(220.0, 15.0, 50.0, Waveform::Sawtooth);

        let samples1 = synth1.synthesize(4410, 44100.0, &mut rng1);
        let samples2 = synth2.synthesize(4410, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_pulsar_different_waveforms() {
        let mut rng = create_rng(42);

        for waveform in [
            Waveform::Sine,
            Waveform::Square,
            Waveform::Sawtooth,
            Waveform::Triangle,
        ] {
            let synth = PulsarSynth::new(330.0, 10.0, 40.0, waveform);
            let samples = synth.synthesize(4410, 44100.0, &mut rng);

            assert_eq!(samples.len(), 4410);
            assert!(samples.iter().any(|&s| s.abs() > 0.0));
        }
    }

    #[test]
    fn test_pulsar_high_pulse_rate() {
        let mut rng = create_rng(42);

        // High pulse rate should produce more grains
        let synth = PulsarSynth::new(880.0, 100.0, 10.0, Waveform::Sine);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);
        assert!(samples.iter().any(|&s| s.abs() > 0.0));
    }

    #[test]
    fn test_pulsar_low_pulse_rate() {
        let mut rng = create_rng(42);

        // Low pulse rate (1 grain per second, 100ms duration, 0.1s audio)
        // This should produce 0 complete grains in 0.1s at 1 Hz rate
        // But the first grain starts at sample 0
        let synth = PulsarSynth::new(440.0, 1.0, 100.0, Waveform::Sine);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        // First grain should still produce output
        assert!(samples.iter().any(|&s| s.abs() > 0.0));
    }

    #[test]
    fn test_pulsar_zero_values() {
        let mut rng = create_rng(42);

        // Zero pulse rate should produce silence
        let synth = PulsarSynth::new(440.0, 0.0, 30.0, Waveform::Sine);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);
        assert!(samples.iter().all(|&s| s == 0.0));

        // Zero grain size should produce silence
        let synth = PulsarSynth::new(440.0, 20.0, 0.0, Waveform::Sine);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);
        assert!(samples.iter().all(|&s| s == 0.0));
    }
}
