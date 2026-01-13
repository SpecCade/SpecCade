//! Phase Distortion (PD) synthesis - Casio CZ style.
//!
//! Phase distortion synthesis creates complex timbres by warping the phase
//! of a waveform non-linearly. Instead of reading a sine wave at a constant
//! rate, the read position is distorted over time, creating evolving timbres
//! with rich harmonics.
//!
//! The basic formula is:
//! - `warped_phase = distort(phase, distortion_amount)`
//! - `output = sin(warped_phase * 2π)`
//!
//! The distortion amount typically decays over time, similar to a filter
//! envelope, creating sounds that start bright and evolve to pure tones.

use std::f64::consts::PI;

use rand_pcg::Pcg32;

use super::{FrequencySweep, Synthesizer};

/// Phase distortion waveform shape.
///
/// Different distortion curves produce different timbral characteristics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PdWaveform {
    /// Resonant-like tone using power curve distortion.
    /// `warped = phase^(1 + distortion)` creates resonant filter-like tones.
    Resonant,
    /// Sawtooth-like asymmetric distortion.
    /// Compresses one half of the waveform, creating saw-like harmonics.
    Sawtooth,
    /// Pulse-like distortion with sharp phase transition.
    /// Creates square/pulse wave characteristics with adjustable edge.
    Pulse,
}

/// Phase Distortion synthesis parameters.
///
/// PD synthesis works by warping the phase of a carrier oscillator to create
/// harmonically rich tones. The distortion amount controls timbre brightness
/// and typically decays over time like a filter envelope.
#[derive(Debug, Clone)]
pub struct PhaseDistortionSynth {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Initial distortion amount (0.0 = pure sine, higher = more harmonics).
    /// Typical range: 0.0 to 10.0
    pub distortion: f64,
    /// Distortion decay rate (higher = faster decay to pure sine).
    /// The distortion decays as: distortion(t) = distortion * exp(-decay * t)
    pub distortion_decay: f64,
    /// Waveform shape determining the distortion curve.
    pub waveform: PdWaveform,
    /// Optional frequency sweep.
    pub freq_sweep: Option<FrequencySweep>,
}

impl PhaseDistortionSynth {
    /// Creates a new Phase Distortion synthesizer.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `distortion` - Initial distortion amount (0.0 = pure sine)
    /// * `waveform` - Distortion curve shape
    pub fn new(frequency: f64, distortion: f64, waveform: PdWaveform) -> Self {
        Self {
            frequency,
            distortion: distortion.max(0.0),
            distortion_decay: 0.0,
            waveform,
            freq_sweep: None,
        }
    }

    /// Sets the distortion decay rate.
    ///
    /// The distortion decays exponentially: distortion(t) = distortion * exp(-decay * t)
    /// Higher values = faster decay to pure sine wave.
    pub fn with_distortion_decay(mut self, decay: f64) -> Self {
        self.distortion_decay = decay.max(0.0);
        self
    }

    /// Sets a frequency sweep.
    pub fn with_sweep(mut self, sweep: FrequencySweep) -> Self {
        self.freq_sweep = Some(sweep);
        self
    }

    /// Creates a CZ-style bass preset.
    ///
    /// Deep bass with resonant character and moderate decay.
    /// Good for bass lines and low-end sounds.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz (typical: 40-120 Hz)
    pub fn cz_bass(frequency: f64) -> Self {
        Self {
            frequency,
            distortion: 4.0,
            distortion_decay: 6.0,
            waveform: PdWaveform::Resonant,
            freq_sweep: None,
        }
    }

    /// Creates a CZ-style organ preset.
    ///
    /// Sustained organ-like tone with moderate distortion.
    /// The sustained distortion gives a bright, organ-like character.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz (typical: 200-800 Hz)
    pub fn cz_organ(frequency: f64) -> Self {
        Self {
            frequency,
            distortion: 2.5,
            distortion_decay: 0.5, // Slow decay for sustained brightness
            waveform: PdWaveform::Sawtooth,
            freq_sweep: None,
        }
    }

    /// Creates a CZ-style strings preset.
    ///
    /// Smooth, evolving pad-like tone that starts bright and mellows.
    /// The pulse waveform creates a softer harmonic spectrum.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz (typical: 200-600 Hz)
    pub fn cz_strings(frequency: f64) -> Self {
        Self {
            frequency,
            distortion: 3.0,
            distortion_decay: 2.0,
            waveform: PdWaveform::Pulse,
            freq_sweep: None,
        }
    }

    /// Applies the phase distortion function based on waveform type.
    ///
    /// # Arguments
    /// * `phase` - Input phase (0.0 to 1.0)
    /// * `distortion` - Current distortion amount
    ///
    /// # Returns
    /// Distorted phase (0.0 to 1.0)
    fn distort_phase(&self, phase: f64, distortion: f64) -> f64 {
        // Normalize phase to 0.0 - 1.0 range
        let phase = phase.fract();
        let phase = if phase < 0.0 { phase + 1.0 } else { phase };

        match self.waveform {
            PdWaveform::Resonant => {
                // Power curve distortion: phase^(1 + distortion)
                // Low distortion = sine, high distortion = resonant peak
                let exponent = 1.0 + distortion;
                phase.powf(exponent)
            }
            PdWaveform::Sawtooth => {
                // Asymmetric compression creating saw-like harmonics
                // The crossover point determines where phase speeds up
                let crossover = 0.5 / (1.0 + distortion * 0.5);
                if phase < crossover {
                    // First part: stretched
                    (phase / crossover) * 0.5
                } else {
                    // Second part: compressed
                    0.5 + ((phase - crossover) / (1.0 - crossover)) * 0.5
                }
            }
            PdWaveform::Pulse => {
                // Sharp transition creating pulse-like characteristics
                // Higher distortion = sharper edge, more harmonics
                let sharpness = 1.0 + distortion * 2.0;
                let centered = phase - 0.5;
                // Apply sigmoid-like sharpening
                let shaped = 0.5 + 0.5 * (centered * sharpness).tanh() / sharpness.tanh();
                shaped.clamp(0.0, 1.0)
            }
        }
    }
}

impl Synthesizer for PhaseDistortionSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);

        let dt = 1.0 / sample_rate;
        let two_pi = 2.0 * PI;

        let mut phase: f64 = 0.0;

        for i in 0..num_samples {
            let t = i as f64 * dt;
            let progress = i as f64 / num_samples as f64;

            // Get frequency (with optional sweep)
            let frequency = if let Some(ref sweep) = self.freq_sweep {
                sweep.at(progress)
            } else {
                self.frequency
            };

            // Calculate time-varying distortion amount
            let current_distortion = if self.distortion_decay > 0.0 {
                self.distortion * (-self.distortion_decay * t).exp()
            } else {
                self.distortion
            };

            // Normalize phase to 0.0 - 1.0 for distortion function
            let normalized_phase = (phase / two_pi).fract();
            let normalized_phase = if normalized_phase < 0.0 {
                normalized_phase + 1.0
            } else {
                normalized_phase
            };

            // Apply phase distortion
            let warped_phase = self.distort_phase(normalized_phase, current_distortion);

            // Generate output sample using warped phase
            let sample = (warped_phase * two_pi).sin();

            output.push(sample);

            // Update phase
            phase += two_pi * frequency * dt;

            // Wrap phase to prevent precision loss
            if phase >= two_pi {
                phase -= two_pi;
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;
    use crate::synthesis::SweepCurve;

    #[test]
    fn test_basic_pd_synthesis() {
        let synth = PhaseDistortionSynth::new(440.0, 3.0, PdWaveform::Resonant);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!(
                (-1.0..=1.0).contains(&s),
                "Sample {} out of range [-1.0, 1.0]",
                s
            );
        }
    }

    #[test]
    fn test_pd_zero_distortion_is_sine() {
        // With zero distortion, output should be a pure sine wave
        let synth = PhaseDistortionSynth::new(440.0, 0.0, PdWaveform::Resonant);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        // Compare with expected sine wave
        let dt = 1.0 / 44100.0;
        let two_pi = 2.0 * PI;
        for (i, &sample) in samples.iter().enumerate() {
            let expected = (two_pi * 440.0 * i as f64 * dt).sin();
            assert!(
                (sample - expected).abs() < 0.001,
                "Sample {} at index {} differs from expected sine {}",
                sample,
                i,
                expected
            );
        }
    }

    #[test]
    fn test_pd_waveform_sawtooth() {
        let synth = PhaseDistortionSynth::new(440.0, 5.0, PdWaveform::Sawtooth);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_pd_waveform_pulse() {
        let synth = PhaseDistortionSynth::new(440.0, 4.0, PdWaveform::Pulse);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_pd_with_distortion_decay() {
        let synth =
            PhaseDistortionSynth::new(440.0, 5.0, PdWaveform::Resonant).with_distortion_decay(10.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);

        // Verify all samples are in range
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_pd_with_sweep() {
        let sweep = FrequencySweep::new(440.0, 880.0, SweepCurve::Linear);
        let synth =
            PhaseDistortionSynth::new(440.0, 3.0, PdWaveform::Resonant).with_sweep(sweep);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_cz_bass_preset() {
        let synth = PhaseDistortionSynth::cz_bass(80.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);
        assert_eq!(synth.frequency, 80.0);
        assert!((synth.distortion - 4.0).abs() < 0.001);
        assert_eq!(synth.waveform, PdWaveform::Resonant);

        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_cz_organ_preset() {
        let synth = PhaseDistortionSynth::cz_organ(440.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        assert_eq!(synth.frequency, 440.0);
        assert!((synth.distortion - 2.5).abs() < 0.001);
        assert_eq!(synth.waveform, PdWaveform::Sawtooth);

        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_cz_strings_preset() {
        let synth = PhaseDistortionSynth::cz_strings(330.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        assert_eq!(synth.frequency, 330.0);
        assert!((synth.distortion - 3.0).abs() < 0.001);
        assert_eq!(synth.waveform, PdWaveform::Pulse);

        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_pd_determinism() {
        let synth = PhaseDistortionSynth::new(440.0, 3.0, PdWaveform::Resonant);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2, "PD synthesis must be deterministic");
    }

    #[test]
    fn test_pd_high_distortion() {
        // Test with very high distortion to ensure no numerical issues
        let synth = PhaseDistortionSynth::new(440.0, 10.0, PdWaveform::Resonant);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!(
                (-1.0..=1.0).contains(&s),
                "Sample {} out of range at high distortion",
                s
            );
            assert!(s.is_finite(), "Sample must be finite");
        }
    }

    #[test]
    fn test_distort_phase_resonant() {
        let synth = PhaseDistortionSynth::new(440.0, 2.0, PdWaveform::Resonant);

        // At zero distortion, phase should pass through unchanged
        let phase_0 = synth.distort_phase(0.5, 0.0);
        assert!((phase_0 - 0.5).abs() < 0.001, "Zero distortion should be linear");

        // At high distortion, low phases should be compressed
        let phase_high = synth.distort_phase(0.5, 5.0);
        assert!(
            phase_high < 0.5,
            "Resonant distortion should compress phase at midpoint"
        );
    }

    #[test]
    fn test_distort_phase_sawtooth() {
        let synth = PhaseDistortionSynth::new(440.0, 2.0, PdWaveform::Sawtooth);

        // With distortion, the crossover point should shift
        let phase_early = synth.distort_phase(0.2, 3.0);
        let phase_late = synth.distort_phase(0.8, 3.0);

        // Early phase should be stretched (mapped higher than linear)
        // Late phase should still map to a valid value
        assert!(phase_early >= 0.0 && phase_early <= 1.0);
        assert!(phase_late >= 0.0 && phase_late <= 1.0);
    }

    #[test]
    fn test_distort_phase_pulse() {
        let synth = PhaseDistortionSynth::new(440.0, 2.0, PdWaveform::Pulse);

        // Pulse distortion should keep values in 0-1 range
        let phase_0 = synth.distort_phase(0.0, 5.0);
        let phase_mid = synth.distort_phase(0.5, 5.0);
        let phase_1 = synth.distort_phase(1.0, 5.0);

        assert!(phase_0 >= 0.0 && phase_0 <= 1.0);
        assert!(phase_mid >= 0.0 && phase_mid <= 1.0);
        assert!(phase_1 >= 0.0 && phase_1 <= 1.0);

        // At center, pulse distortion should be near 0.5
        assert!(
            (phase_mid - 0.5).abs() < 0.1,
            "Pulse center should be near 0.5"
        );
    }
}
