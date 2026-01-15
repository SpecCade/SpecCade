//! Modal synthesis for physically-inspired resonant sounds.
//!
//! Modal synthesis simulates struck or bowed objects (bells, chimes, metal bars,
//! wooden bars) by modeling their resonant modes. Each mode is a decaying sine
//! wave at a specific frequency ratio with its own amplitude and decay time.
//!
//! The key insight is that physical objects vibrate at multiple frequencies
//! simultaneously, and these frequencies aren't necessarily harmonic. Modal
//! synthesis allows precise control over each resonant mode.

use std::f64::consts::PI;

use rand::Rng;
use rand_pcg::Pcg32;

use super::{FrequencySweep, Synthesizer};

/// A single resonant mode in modal synthesis.
#[derive(Debug, Clone, PartialEq)]
pub struct Mode {
    /// Frequency ratio relative to the fundamental (1.0 = fundamental).
    pub freq_ratio: f64,
    /// Amplitude of this mode (0.0 to 1.0).
    pub amplitude: f64,
    /// Decay time in seconds.
    pub decay_time: f64,
}

impl Mode {
    /// Creates a new mode.
    ///
    /// # Arguments
    /// * `freq_ratio` - Frequency ratio relative to fundamental
    /// * `amplitude` - Amplitude (0.0 to 1.0)
    /// * `decay_time` - Decay time in seconds
    pub fn new(freq_ratio: f64, amplitude: f64, decay_time: f64) -> Self {
        Self {
            freq_ratio: freq_ratio.max(0.001),
            amplitude: amplitude.clamp(0.0, 1.0),
            decay_time: decay_time.max(0.001),
        }
    }
}

/// Excitation type for modal synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Excitation {
    /// Single impulse excitation (sharp attack).
    Impulse,
    /// Noise burst excitation (softer, more complex attack).
    Noise,
    /// Pluck-like excitation with quick decay.
    Pluck,
}

/// Modal synthesis using a bank of decaying resonators.
///
/// This synthesizer models physical objects by summing multiple resonant modes,
/// each with its own frequency ratio, amplitude, and decay characteristics.
#[derive(Debug, Clone)]
pub struct ModalSynth {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Bank of resonant modes.
    pub modes: Vec<Mode>,
    /// Excitation type.
    pub excitation: Excitation,
    /// Optional frequency sweep.
    pub freq_sweep: Option<FrequencySweep>,
}

impl ModalSynth {
    /// Creates a new modal synthesizer.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `modes` - Vector of Mode configurations
    /// * `excitation` - Type of excitation (Impulse, Noise, or Pluck)
    pub fn new(frequency: f64, modes: Vec<Mode>, excitation: Excitation) -> Self {
        Self {
            frequency: frequency.max(20.0),
            modes,
            excitation,
            freq_sweep: None,
        }
    }

    /// Creates a bell preset.
    ///
    /// Bells have characteristic inharmonic partials that give them
    /// their distinctive sound. The mode frequencies are based on
    /// typical church bell partial ratios.
    pub fn bell(frequency: f64) -> Self {
        let modes = vec![
            Mode::new(1.0, 1.0, 4.0),  // Fundamental (hum)
            Mode::new(2.0, 0.8, 3.5),  // Prime/strike tone
            Mode::new(2.4, 0.6, 3.0),  // Tierce (minor third)
            Mode::new(3.0, 0.5, 2.5),  // Quint (fifth)
            Mode::new(4.0, 0.4, 2.0),  // Nominal (octave)
            Mode::new(5.0, 0.25, 1.5), // Deciem
            Mode::new(6.0, 0.15, 1.2), // Upper partial
            Mode::new(8.0, 0.1, 1.0),  // High partial
        ];
        Self::new(frequency, modes, Excitation::Impulse)
    }

    /// Creates a chime preset.
    ///
    /// Chimes (tubular bells) have a different partial structure
    /// than bells, with more emphasis on the fundamental and
    /// a characteristic shimmer from higher partials.
    pub fn chime(frequency: f64) -> Self {
        let modes = vec![
            Mode::new(1.0, 1.0, 3.0),    // Fundamental
            Mode::new(2.76, 0.7, 2.5),   // Second partial (tube characteristic)
            Mode::new(5.4, 0.5, 2.0),    // Third partial
            Mode::new(8.93, 0.3, 1.5),   // Fourth partial
            Mode::new(13.34, 0.15, 1.0), // Fifth partial
            Mode::new(18.64, 0.08, 0.7), // Sixth partial
        ];
        Self::new(frequency, modes, Excitation::Impulse)
    }

    /// Creates a marimba preset.
    ///
    /// Marimbas have a warm, mellow tone due to their resonators
    /// which emphasize the fundamental and suppress higher partials.
    /// The bar itself produces near-harmonic partials.
    pub fn marimba(frequency: f64) -> Self {
        let modes = vec![
            Mode::new(1.0, 1.0, 1.5),   // Fundamental (strong)
            Mode::new(4.0, 0.3, 0.8),   // 2nd partial (near 4:1 for bars)
            Mode::new(9.0, 0.15, 0.5),  // 3rd partial
            Mode::new(16.0, 0.05, 0.3), // 4th partial (weak)
        ];
        Self::new(frequency, modes, Excitation::Pluck)
    }

    /// Creates a glockenspiel preset.
    ///
    /// Glockenspiels have a bright, bell-like tone with strong
    /// high partials and long sustain due to their metal construction.
    pub fn glockenspiel(frequency: f64) -> Self {
        let modes = vec![
            Mode::new(1.0, 1.0, 2.5),   // Fundamental
            Mode::new(2.71, 0.7, 2.2),  // Characteristic bar partial
            Mode::new(5.28, 0.5, 1.8),  // Higher partial
            Mode::new(8.65, 0.35, 1.4), // Bright partial
            Mode::new(12.81, 0.2, 1.0), // Very high partial
            Mode::new(17.77, 0.1, 0.7), // Shimmer
        ];
        Self::new(frequency, modes, Excitation::Impulse)
    }

    /// Creates a vibraphone preset.
    ///
    /// Vibraphones have a pure, sustained tone with motor-driven
    /// vibrato (not modeled here, just the basic tone).
    pub fn vibraphone(frequency: f64) -> Self {
        let modes = vec![
            Mode::new(1.0, 1.0, 3.5),   // Fundamental (very sustained)
            Mode::new(4.0, 0.4, 2.5),   // 2nd partial
            Mode::new(10.0, 0.15, 1.5), // 3rd partial
            Mode::new(20.0, 0.05, 0.8), // High partial
        ];
        Self::new(frequency, modes, Excitation::Pluck)
    }

    /// Creates a xylophone preset.
    ///
    /// Xylophones have a bright, dry tone with short decay
    /// due to their hard mallets and lack of resonators.
    pub fn xylophone(frequency: f64) -> Self {
        let modes = vec![
            Mode::new(1.0, 1.0, 0.8),   // Fundamental
            Mode::new(3.0, 0.6, 0.6),   // Strong 3rd harmonic
            Mode::new(6.0, 0.4, 0.4),   // Higher partial
            Mode::new(10.0, 0.2, 0.25), // Bright partial
            Mode::new(15.0, 0.1, 0.15), // Attack transient
        ];
        Self::new(frequency, modes, Excitation::Impulse)
    }

    /// Sets a frequency sweep.
    pub fn with_sweep(mut self, sweep: FrequencySweep) -> Self {
        self.freq_sweep = Some(sweep);
        self
    }

    /// Sets the excitation type.
    pub fn with_excitation(mut self, excitation: Excitation) -> Self {
        self.excitation = excitation;
        self
    }
}

impl Synthesizer for ModalSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        if self.modes.is_empty() || num_samples == 0 {
            return vec![0.0; num_samples];
        }

        let mut output = vec![0.0; num_samples];
        let dt = 1.0 / sample_rate;
        let two_pi = 2.0 * PI;

        // Generate excitation signal
        let excitation_samples = match self.excitation {
            Excitation::Impulse => {
                // Single sample impulse
                let mut exc = vec![0.0; num_samples];
                if !exc.is_empty() {
                    exc[0] = 1.0;
                }
                exc
            }
            Excitation::Noise => {
                // Short noise burst (first ~5ms)
                let burst_samples = ((sample_rate * 0.005) as usize).min(num_samples);
                let mut exc = vec![0.0; num_samples];
                for sample in exc.iter_mut().take(burst_samples) {
                    *sample = rng.gen::<f64>() * 2.0 - 1.0;
                }
                // Apply quick envelope to noise burst
                for (i, sample) in exc.iter_mut().take(burst_samples).enumerate() {
                    let env = 1.0 - (i as f64 / burst_samples as f64);
                    *sample *= env;
                }
                exc
            }
            Excitation::Pluck => {
                // Pluck-like excitation: quick attack with harmonic content
                let attack_samples = ((sample_rate * 0.002) as usize).min(num_samples);
                let mut exc = vec![0.0; num_samples];
                for (i, sample) in exc.iter_mut().take(attack_samples).enumerate() {
                    let t = i as f64 / attack_samples as f64;
                    // Combination of impulse and quick noise for pluck character
                    let impulse = if i == 0 { 1.0 } else { 0.0 };
                    let noise = (rng.gen::<f64>() * 2.0 - 1.0) * (1.0 - t);
                    *sample = impulse * 0.7 + noise * 0.3;
                }
                exc
            }
        };

        // Pre-calculate mode parameters
        let mode_params: Vec<(f64, f64, f64, f64)> = self
            .modes
            .iter()
            .map(|mode| {
                // Calculate actual frequency
                let freq = self.frequency * mode.freq_ratio;
                let amp = mode.amplitude;
                // Decay rate from decay time (time to reach ~5% amplitude)
                let decay_rate = 3.0 / mode.decay_time;
                // Random initial phase for natural sound
                let phase = rng.gen::<f64>() * two_pi;
                (freq, amp, decay_rate, phase)
            })
            .collect();

        // Synthesize each mode
        for (freq, amp, decay_rate, initial_phase) in &mode_params {
            let mut phase = *initial_phase;

            for (i, sample) in output.iter_mut().enumerate() {
                let t = i as f64 * dt;

                // Apply frequency sweep if present
                let current_freq = if let Some(ref sweep) = self.freq_sweep {
                    let progress = i as f64 / num_samples.max(1) as f64;
                    let base = sweep.at(progress);
                    base * (*freq / self.frequency)
                } else {
                    *freq
                };

                // Mode output: decaying sine
                let envelope = (-decay_rate * t).exp();
                let mode_output = (phase).sin() * amp * envelope;

                // Convolve with excitation (simplified: just multiply by excitation envelope)
                let exc_factor = excitation_samples[i];
                let exc_influence = if exc_factor.abs() > 0.0001 {
                    1.0 + exc_factor * 0.5
                } else {
                    1.0
                };

                *sample += mode_output * exc_influence;

                // Advance phase
                phase += two_pi * current_freq * dt;
                if phase >= two_pi {
                    phase -= two_pi;
                }
            }
        }

        // Normalize output to [-1.0, 1.0]
        let max = output
            .iter()
            .map(|s| s.abs())
            .fold(0.0_f64, |a, b| a.max(b));
        if max > 0.0 {
            for s in &mut output {
                *s /= max;
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
    fn test_modal_synth_basic() {
        let modes = vec![
            Mode::new(1.0, 1.0, 1.0),
            Mode::new(2.0, 0.5, 0.8),
            Mode::new(3.0, 0.25, 0.6),
        ];
        let synth = ModalSynth::new(440.0, modes, Excitation::Impulse);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_modal_bell_preset() {
        let synth = ModalSynth::bell(440.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(2000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 2000);
        assert_eq!(synth.modes.len(), 8);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_modal_chime_preset() {
        let synth = ModalSynth::chime(523.25); // C5
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1500, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1500);
        assert_eq!(synth.modes.len(), 6);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_modal_marimba_preset() {
        let synth = ModalSynth::marimba(262.0); // C4
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        assert_eq!(synth.modes.len(), 4);
        assert_eq!(synth.excitation, Excitation::Pluck);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_modal_glockenspiel_preset() {
        let synth = ModalSynth::glockenspiel(880.0); // A5
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        assert_eq!(synth.modes.len(), 6);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_modal_determinism() {
        let synth = ModalSynth::bell(440.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(500, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(500, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_modal_different_seeds() {
        let synth = ModalSynth::bell(440.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(43);

        let samples1 = synth.synthesize(500, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(500, 44100.0, &mut rng2);

        // Should be different due to random phases
        assert_ne!(samples1, samples2);
    }

    #[test]
    fn test_modal_excitation_types() {
        let modes = vec![Mode::new(1.0, 1.0, 1.0)];

        // Test each excitation type
        for excitation in [Excitation::Impulse, Excitation::Noise, Excitation::Pluck] {
            let synth = ModalSynth::new(440.0, modes.clone(), excitation);
            let mut rng = create_rng(42);
            let samples = synth.synthesize(500, 44100.0, &mut rng);

            assert_eq!(samples.len(), 500);
            for &s in &samples {
                assert!((-1.0..=1.0).contains(&s));
            }
        }
    }

    #[test]
    fn test_modal_empty_modes() {
        let synth = ModalSynth::new(440.0, vec![], Excitation::Impulse);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 100);
        for &s in &samples {
            assert_eq!(s, 0.0);
        }
    }

    #[test]
    fn test_modal_zero_samples() {
        let synth = ModalSynth::bell(440.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(0, 44100.0, &mut rng);

        assert!(samples.is_empty());
    }

    #[test]
    fn test_mode_clamping() {
        // Test that Mode constructor clamps values correctly
        let mode = Mode::new(-1.0, 2.0, -5.0);
        assert!(mode.freq_ratio > 0.0);
        assert!(mode.amplitude <= 1.0);
        assert!(mode.decay_time > 0.0);
    }

    #[test]
    fn test_modal_with_freq_sweep() {
        use crate::synthesis::SweepCurve;

        let synth = ModalSynth::bell(440.0).with_sweep(FrequencySweep::new(
            440.0,
            880.0,
            SweepCurve::Linear,
        ));

        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }
}
