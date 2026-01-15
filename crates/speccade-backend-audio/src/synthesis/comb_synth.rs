//! Comb filter synthesis for resonant metallic tones.
//!
//! Uses a delay-line comb filter with feedback to create pitched resonant sounds.
//! The delay line length determines the pitch (sample_rate / frequency).
//! An excitation signal is fed through the comb filter to produce metallic,
//! resonant, and bell-like timbres.
//!
//! Distinct from:
//! - Karplus-Strong: Uses lowpass filtering in the feedback loop for plucked string sounds
//! - Metallic synthesis: Uses inharmonic additive partials

use rand::Rng;
use rand_pcg::Pcg32;
use speccade_spec::recipe::audio::CombExcitation;

use super::Synthesizer;

/// Comb filter synthesis parameters.
#[derive(Debug, Clone)]
pub struct CombFilterSynth {
    /// Base frequency in Hz (determines delay line length).
    pub frequency: f64,
    /// Feedback decay amount (0.0-1.0). Higher values = longer resonance.
    pub decay: f64,
    /// Excitation type for the comb filter.
    pub excitation: CombExcitation,
}

impl CombFilterSynth {
    /// Creates a new comb filter synthesizer.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `decay` - Feedback decay (0.0-1.0)
    /// * `excitation` - Type of excitation signal
    pub fn new(frequency: f64, decay: f64, excitation: CombExcitation) -> Self {
        Self {
            frequency,
            decay: decay.clamp(0.0, 0.999),
            excitation,
        }
    }

    /// Creates a metallic bell-like preset.
    pub fn bell(frequency: f64) -> Self {
        Self {
            frequency,
            decay: 0.95,
            excitation: CombExcitation::Impulse,
        }
    }

    /// Creates a resonant metallic preset with noise excitation.
    pub fn resonant(frequency: f64) -> Self {
        Self {
            frequency,
            decay: 0.9,
            excitation: CombExcitation::Noise,
        }
    }

    /// Creates a harsh metallic preset with sawtooth excitation.
    pub fn harsh(frequency: f64) -> Self {
        Self {
            frequency,
            decay: 0.85,
            excitation: CombExcitation::Saw,
        }
    }
}

impl Synthesizer for CombFilterSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        // Handle edge case: zero or negative frequency
        if self.frequency <= 0.0 {
            return vec![0.0; num_samples];
        }

        // Calculate delay line length based on frequency
        let delay_length_f = sample_rate / self.frequency;
        if !delay_length_f.is_finite() || delay_length_f <= 0.0 {
            return vec![0.0; num_samples];
        }
        let delay_length = delay_length_f.round() as usize;
        if delay_length == 0 {
            return vec![0.0; num_samples];
        }

        // Generate excitation signal (short burst at the start)
        let excitation_length = delay_length.min(num_samples);
        let excitation: Vec<f64> = match self.excitation {
            CombExcitation::Impulse => {
                // Single impulse at the start
                let mut exc = vec![0.0; excitation_length];
                if !exc.is_empty() {
                    exc[0] = 1.0;
                }
                exc
            }
            CombExcitation::Noise => {
                // Short noise burst (one period length)
                (0..excitation_length)
                    .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
                    .collect()
            }
            CombExcitation::Saw => {
                // Short sawtooth burst (one period)
                (0..excitation_length)
                    .map(|i| {
                        let t = i as f64 / excitation_length as f64;
                        2.0 * t - 1.0
                    })
                    .collect()
            }
        };

        // Initialize delay line
        let mut delay_line = vec![0.0; delay_length];
        let mut write_pos = 0;

        // Output buffer
        let mut output = Vec::with_capacity(num_samples);

        // Create an iterator that yields excitation samples, then zeros
        let mut excitation_iter = excitation.iter().copied();

        // Process samples through the comb filter
        for _ in 0..num_samples {
            // Get input (excitation signal or zero when exhausted)
            let input = excitation_iter.next().unwrap_or(0.0);

            // Read from delay line (oldest sample)
            let delayed = delay_line[write_pos];

            // Comb filter: output = input + feedback * delayed
            let filtered = input + self.decay * delayed;

            // Write to delay line
            delay_line[write_pos] = filtered;

            // Advance write position (circular buffer)
            write_pos = (write_pos + 1) % delay_length;

            // Output the filtered signal
            output.push(filtered);
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_comb_synth_impulse() {
        let synth = CombFilterSynth::new(440.0, 0.9, CombExcitation::Impulse);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);
        // First sample should be the impulse
        assert!((samples[0] - 1.0).abs() < 0.01);
        // Should decay over time
        let early_energy: f64 = samples[0..1000].iter().map(|s| s.powi(2)).sum();
        let late_energy: f64 = samples[40000..41000].iter().map(|s| s.powi(2)).sum();
        assert!(early_energy > late_energy);
    }

    #[test]
    fn test_comb_synth_noise() {
        let synth = CombFilterSynth::new(220.0, 0.85, CombExcitation::Noise);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);
        // Should have non-zero content initially
        let early_rms: f64 =
            (samples[0..1000].iter().map(|s| s.powi(2)).sum::<f64>() / 1000.0).sqrt();
        assert!(early_rms > 0.01);
    }

    #[test]
    fn test_comb_synth_saw() {
        let synth = CombFilterSynth::new(330.0, 0.88, CombExcitation::Saw);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);
        // Should have non-zero content
        let has_content = samples.iter().any(|&s| s.abs() > 0.01);
        assert!(has_content);
    }

    #[test]
    fn test_comb_synth_determinism() {
        let synth = CombFilterSynth::new(440.0, 0.9, CombExcitation::Noise);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_comb_synth_presets() {
        let mut rng = create_rng(42);

        let bell = CombFilterSynth::bell(440.0);
        let bell_samples = bell.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(bell_samples.len(), 1000);

        let resonant = CombFilterSynth::resonant(220.0);
        let resonant_samples = resonant.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(resonant_samples.len(), 1000);

        let harsh = CombFilterSynth::harsh(330.0);
        let harsh_samples = harsh.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(harsh_samples.len(), 1000);
    }

    #[test]
    fn test_comb_synth_decay_clamp() {
        // Test that decay > 0.999 gets clamped
        let synth = CombFilterSynth::new(440.0, 1.5, CombExcitation::Impulse);
        let mut rng = create_rng(42);

        // Process and verify it doesn't explode
        let samples = synth.synthesize(10000, 44100.0, &mut rng);
        for &s in &samples {
            assert!(s.is_finite());
            assert!(s.abs() < 100.0);
        }
    }

    #[test]
    fn test_comb_synth_zero_frequency() {
        // Zero frequency should produce silence (edge case)
        let synth = CombFilterSynth::new(0.0, 0.9, CombExcitation::Impulse);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(100, 44100.0, &mut rng);

        // Should return zeros (delay_length would be inf/0)
        assert_eq!(samples.len(), 100);
        // All samples should be zero
        for &s in &samples {
            assert_eq!(s, 0.0);
        }
    }
}
