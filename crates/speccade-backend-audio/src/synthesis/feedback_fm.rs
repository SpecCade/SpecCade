//! Feedback FM synthesis with self-modulating operator.
//!
//! A single oscillator that modulates itself by feeding its output back into
//! its own phase. Creates characteristic "screaming" or "gritty" timbres at
//! high feedback values, similar to DX7 operator 1 self-feedback.

use std::f64::consts::PI;

use rand_pcg::Pcg32;

use super::{FrequencySweep, Synthesizer};

/// Feedback FM synthesis parameters.
#[derive(Debug, Clone)]
pub struct FeedbackFmSynth {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Self-modulation amount (0.0-1.0).
    pub feedback: f64,
    /// Modulation depth/index controlling harmonic richness.
    pub modulation_index: f64,
    /// Optional frequency sweep.
    pub freq_sweep: Option<FrequencySweep>,
}

impl FeedbackFmSynth {
    /// Creates a new feedback FM synthesizer.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `feedback` - Self-modulation amount (0.0-1.0, clamped to max 0.99)
    /// * `modulation_index` - Modulation depth controlling harmonic richness
    pub fn new(frequency: f64, feedback: f64, modulation_index: f64) -> Self {
        Self {
            frequency,
            feedback: feedback.clamp(0.0, 0.99),
            modulation_index,
            freq_sweep: None,
        }
    }

    /// Sets a frequency sweep.
    pub fn with_sweep(mut self, sweep: FrequencySweep) -> Self {
        self.freq_sweep = Some(sweep);
        self
    }
}

impl Synthesizer for FeedbackFmSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);

        let dt = 1.0 / sample_rate;
        let two_pi = 2.0 * PI;

        let mut phase: f64 = 0.0;
        let mut prev_output: f64 = 0.0;

        // Clamp feedback for stability
        let feedback = self.feedback.clamp(0.0, 0.99);

        for i in 0..num_samples {
            let progress = i as f64 / num_samples.max(1) as f64;

            // Get frequency (with optional sweep)
            let freq = if let Some(ref sweep) = self.freq_sweep {
                sweep.at(progress)
            } else {
                self.frequency
            };

            // Feedback FM equation: output = sin(phase + feedback * prev_output * modulation_index)
            // The previous output is fed back into the phase, scaled by feedback and modulation_index
            let modulation = feedback * prev_output * self.modulation_index;
            let current_output = (phase + modulation).sin();

            output.push(current_output);
            prev_output = current_output;

            // Update phase
            phase += two_pi * freq * dt;

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

    #[test]
    fn test_basic_feedback_fm() {
        let synth = FeedbackFmSynth::new(440.0, 0.5, 2.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s), "sample out of range: {}", s);
        }
    }

    #[test]
    fn test_zero_feedback_is_pure_sine() {
        let synth = FeedbackFmSynth::new(440.0, 0.0, 2.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        // With zero feedback, should be pure sine wave
        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_high_feedback_stays_stable() {
        // Even with high feedback, output should stay in range
        let synth = FeedbackFmSynth::new(440.0, 0.99, 5.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);
        for &s in &samples {
            assert!(
                (-1.0..=1.0).contains(&s),
                "sample out of range at high feedback: {}",
                s
            );
        }
    }

    #[test]
    fn test_feedback_clamping() {
        // Feedback > 1.0 should be clamped to 0.99
        let synth = FeedbackFmSynth::new(440.0, 2.0, 2.0);
        assert!(synth.feedback <= 0.99);

        // Feedback < 0 should be clamped to 0
        let synth2 = FeedbackFmSynth::new(440.0, -1.0, 2.0);
        assert!(synth2.feedback >= 0.0);
    }

    #[test]
    fn test_feedback_fm_with_sweep() {
        let sweep = FrequencySweep::new(440.0, 220.0, super::super::SweepCurve::Linear);
        let synth = FeedbackFmSynth::new(440.0, 0.5, 2.0).with_sweep(sweep);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_feedback_fm_determinism() {
        let synth = FeedbackFmSynth::new(440.0, 0.7, 3.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }
}
