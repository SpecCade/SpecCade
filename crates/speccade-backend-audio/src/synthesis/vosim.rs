//! VOSIM synthesis implementation.
//!
//! VOSIM (Voice Simulation) synthesis generates formant-rich sounds using squared-sine
//! pulse trains. Each fundamental period contains N pulses at the formant frequency,
//! with exponential decay within the period. This creates vowel-like and robotic timbres
//! efficiently without requiring filter banks.
//!
//! The classic VOSIM algorithm:
//! - Each fundamental period contains N squared-sine pulses at the formant frequency
//! - Pulses decay exponentially within each period
//! - Formula: output = sin^2(2 * PI * formant_freq * t) * decay(t)
//! - The number of pulses determines formant character

use rand::Rng;
use rand_pcg::Pcg32;
use std::f64::consts::PI;

use crate::synthesis::Synthesizer;

/// VOSIM synthesizer.
#[derive(Debug, Clone)]
pub struct VosimSynth {
    /// Fundamental frequency (pitch) in Hz.
    frequency: f64,
    /// Formant frequency (spectral peak) in Hz.
    formant_freq: f64,
    /// Number of pulses per period (1-16).
    pulses: u8,
    /// Noise amount for breathiness (0.0-1.0).
    breathiness: f64,
}

impl VosimSynth {
    /// Creates a new VOSIM synthesizer.
    ///
    /// # Arguments
    /// * `frequency` - Fundamental frequency (pitch) in Hz
    /// * `formant_freq` - Formant frequency (spectral peak) in Hz
    /// * `pulses` - Number of pulses per period (1-16)
    /// * `breathiness` - Noise amount for breathiness (0.0-1.0)
    pub fn new(frequency: f64, formant_freq: f64, pulses: u8, breathiness: f64) -> Self {
        Self {
            frequency,
            formant_freq,
            pulses: pulses.clamp(1, 16),
            breathiness: breathiness.clamp(0.0, 1.0),
        }
    }
}

impl Synthesizer for VosimSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        let mut output = vec![0.0; num_samples];

        if self.frequency <= 0.0 || self.formant_freq <= 0.0 {
            return output;
        }

        // Fundamental period in samples
        let period_samples = (sample_rate / self.frequency).round() as usize;
        if period_samples == 0 {
            return output;
        }

        // Duration of each formant pulse in samples
        let formant_period = sample_rate / self.formant_freq;
        let pulse_duration_samples = (formant_period * self.pulses as f64).round() as usize;

        // Pre-generate noise buffer for determinism
        let noise_buffer: Vec<f64> = (0..num_samples)
            .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
            .collect();

        // Generate VOSIM waveform
        let mut sample_idx = 0;
        while sample_idx < num_samples {
            // Generate one fundamental period
            let period_end = (sample_idx + period_samples).min(num_samples);
            let active_end = (sample_idx + pulse_duration_samples).min(period_end);

            // Generate pulses within active portion of period
            let pulse_duration_max = pulse_duration_samples.max(1) as f64;
            for (local_idx, sample) in output[sample_idx..active_end].iter_mut().enumerate() {
                let t = local_idx as f64 / sample_rate;

                // Squared sine at formant frequency
                let sine_val = (2.0 * PI * self.formant_freq * t).sin();
                let squared_sine = sine_val * sine_val;

                // Exponential decay within the pulse train
                // Decay from 1.0 to ~0.1 over the pulse duration
                let pulse_progress = local_idx as f64 / pulse_duration_max;
                let decay = (-2.5 * pulse_progress).exp();

                *sample = squared_sine * decay;
            }

            // Rest of period is silent (already zero)

            sample_idx = period_end;
        }

        // Add breathiness (filtered noise)
        if self.breathiness > 0.0 {
            // Simple one-pole lowpass state for noise filtering
            let cutoff_hz = self.formant_freq * 1.5; // Filter noise around formant
            let rc = 1.0 / (2.0 * PI * cutoff_hz);
            let dt = 1.0 / sample_rate;
            let alpha = dt / (rc + dt);

            let mut filtered_noise = 0.0;
            for (sample, &noise) in output.iter_mut().zip(noise_buffer.iter()) {
                // One-pole lowpass filter
                filtered_noise = filtered_noise + alpha * (noise - filtered_noise);

                // Mix noise with VOSIM output
                *sample = *sample * (1.0 - self.breathiness * 0.5)
                    + filtered_noise * self.breathiness * 0.5;
            }
        }

        // Normalize to prevent clipping
        let max_abs = output.iter().fold(0.0f64, |m, &s| m.max(s.abs()));
        if max_abs > 0.0 {
            let scale = 1.0 / max_abs;
            for sample in output.iter_mut() {
                *sample *= scale;
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_vosim_basic() {
        let mut rng = create_rng(42);

        let synth = VosimSynth::new(110.0, 800.0, 4, 0.0);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        // Should produce non-zero output
        assert!(samples.iter().any(|&s| s.abs() > 0.0));
        // Should be normalized
        let max_abs = samples.iter().fold(0.0f64, |m, &s| m.max(s.abs()));
        assert!(max_abs <= 1.0 + 1e-10);
    }

    #[test]
    fn test_vosim_determinism() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let synth1 = VosimSynth::new(220.0, 1000.0, 6, 0.2);
        let synth2 = VosimSynth::new(220.0, 1000.0, 6, 0.2);

        let samples1 = synth1.synthesize(4410, 44100.0, &mut rng1);
        let samples2 = synth2.synthesize(4410, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_vosim_different_pulses() {
        let mut rng = create_rng(42);

        for pulses in [1, 4, 8, 16] {
            let synth = VosimSynth::new(110.0, 800.0, pulses, 0.0);
            let samples = synth.synthesize(4410, 44100.0, &mut rng);

            assert_eq!(samples.len(), 4410);
            assert!(samples.iter().any(|&s| s.abs() > 0.0));
        }
    }

    #[test]
    fn test_vosim_with_breathiness() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let synth_clean = VosimSynth::new(110.0, 800.0, 4, 0.0);
        let synth_breathy = VosimSynth::new(110.0, 800.0, 4, 0.5);

        let samples_clean = synth_clean.synthesize(4410, 44100.0, &mut rng1);
        let samples_breathy = synth_breathy.synthesize(4410, 44100.0, &mut rng2);

        // Both should produce output
        assert!(samples_clean.iter().any(|&s| s.abs() > 0.0));
        assert!(samples_breathy.iter().any(|&s| s.abs() > 0.0));

        // They should be different due to noise
        assert_ne!(samples_clean, samples_breathy);
    }

    #[test]
    fn test_vosim_clamps_pulses() {
        let mut rng = create_rng(42);

        // Test that pulses are clamped to valid range
        let synth_low = VosimSynth::new(110.0, 800.0, 0, 0.0);
        let synth_high = VosimSynth::new(110.0, 800.0, 20, 0.0);

        let samples_low = synth_low.synthesize(4410, 44100.0, &mut rng);
        let samples_high = synth_high.synthesize(4410, 44100.0, &mut rng);

        // Both should produce valid output
        assert!(samples_low.iter().any(|&s| s.abs() > 0.0));
        assert!(samples_high.iter().any(|&s| s.abs() > 0.0));
    }

    #[test]
    fn test_vosim_zero_frequency() {
        let mut rng = create_rng(42);

        let synth = VosimSynth::new(0.0, 800.0, 4, 0.0);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        // Should produce silence with zero frequency
        assert!(samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_vosim_vowel_like_formants() {
        let mut rng = create_rng(42);

        // Test with typical vowel formant frequencies
        // Vowel "a" F1 ~ 800 Hz
        let synth = VosimSynth::new(110.0, 800.0, 5, 0.1);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);
        assert!(samples.iter().any(|&s| s.abs() > 0.0));
    }
}
