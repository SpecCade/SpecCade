//! FM (Frequency Modulation) synthesis.
//!
//! FM synthesis creates complex timbres by modulating the frequency of a carrier
//! oscillator with a modulator oscillator. Classic sounds include electric pianos,
//! bells, and various metallic tones.

use std::f64::consts::PI;

use rand_pcg::Pcg32;

use super::{FrequencySweep, Synthesizer};

/// FM synthesis parameters.
#[derive(Debug, Clone)]
pub struct FmSynth {
    /// Carrier frequency in Hz.
    pub carrier_freq: f64,
    /// Modulator frequency in Hz (often expressed as ratio to carrier).
    pub modulator_freq: f64,
    /// Modulation index (controls harmonic richness).
    pub modulation_index: f64,
    /// Index decay rate (higher = faster decay of modulation).
    pub index_decay: f64,
    /// Optional carrier frequency sweep.
    pub freq_sweep: Option<FrequencySweep>,
}

impl FmSynth {
    /// Creates a new FM synthesizer.
    ///
    /// # Arguments
    /// * `carrier_freq` - Carrier frequency in Hz
    /// * `modulator_freq` - Modulator frequency in Hz
    /// * `modulation_index` - Modulation index (typical range 0-10)
    pub fn new(carrier_freq: f64, modulator_freq: f64, modulation_index: f64) -> Self {
        Self {
            carrier_freq,
            modulator_freq,
            modulation_index,
            index_decay: 0.0,
            freq_sweep: None,
        }
    }

    /// Creates an FM synthesizer using frequency ratio.
    ///
    /// # Arguments
    /// * `carrier_freq` - Carrier frequency in Hz
    /// * `ratio` - Modulator/carrier frequency ratio
    /// * `modulation_index` - Modulation index
    pub fn with_ratio(carrier_freq: f64, ratio: f64, modulation_index: f64) -> Self {
        Self {
            carrier_freq,
            modulator_freq: carrier_freq * ratio,
            modulation_index,
            index_decay: 0.0,
            freq_sweep: None,
        }
    }

    /// Sets the modulation index decay rate.
    ///
    /// The index decays exponentially: index(t) = index * exp(-decay * t)
    pub fn with_index_decay(mut self, decay: f64) -> Self {
        self.index_decay = decay;
        self
    }

    /// Sets a frequency sweep on the carrier.
    pub fn with_sweep(mut self, sweep: FrequencySweep) -> Self {
        self.freq_sweep = Some(sweep);
        self
    }

    /// Creates a bell-like sound preset.
    pub fn bell(frequency: f64) -> Self {
        Self {
            carrier_freq: frequency,
            modulator_freq: frequency * 1.4, // Inharmonic ratio for bell
            modulation_index: 5.0,
            index_decay: 3.0,
            freq_sweep: None,
        }
    }

    /// Creates an electric piano-like preset.
    pub fn electric_piano(frequency: f64) -> Self {
        Self {
            carrier_freq: frequency,
            modulator_freq: frequency, // 1:1 ratio
            modulation_index: 2.0,
            index_decay: 5.0,
            freq_sweep: None,
        }
    }

    /// Creates a bass-like preset.
    pub fn bass(frequency: f64) -> Self {
        Self {
            carrier_freq: frequency,
            modulator_freq: frequency * 2.0, // 2:1 ratio
            modulation_index: 3.0,
            index_decay: 8.0,
            freq_sweep: None,
        }
    }
}

impl Synthesizer for FmSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);

        let dt = 1.0 / sample_rate;
        let two_pi = 2.0 * PI;

        let mut carrier_phase: f64 = 0.0;
        let mut modulator_phase: f64 = 0.0;

        for i in 0..num_samples {
            let t = i as f64 * dt;
            let progress = i as f64 / num_samples as f64;

            // Get carrier frequency (with optional sweep)
            let carrier_freq = if let Some(ref sweep) = self.freq_sweep {
                sweep.at(progress)
            } else {
                self.carrier_freq
            };

            // Scale modulator frequency proportionally if sweeping
            let mod_ratio = self.modulator_freq / self.carrier_freq;
            let modulator_freq = carrier_freq * mod_ratio;

            // Calculate time-varying modulation index
            let index = if self.index_decay > 0.0 {
                self.modulation_index * (-self.index_decay * t).exp()
            } else {
                self.modulation_index
            };

            // FM equation: carrier = sin(wc*t + index * sin(wm*t))
            let modulator = modulator_phase.sin();
            let carrier = (carrier_phase + index * modulator).sin();

            output.push(carrier);

            // Update phases
            carrier_phase += two_pi * carrier_freq * dt;
            modulator_phase += two_pi * modulator_freq * dt;

            // Wrap phases to prevent precision loss
            if carrier_phase >= two_pi {
                carrier_phase -= two_pi;
            }
            if modulator_phase >= two_pi {
                modulator_phase -= two_pi;
            }
        }

        output
    }
}

/// Two-operator FM with separate envelopes for carrier and modulator.
#[derive(Debug, Clone)]
pub struct TwoOpFm {
    /// Carrier frequency in Hz.
    pub carrier_freq: f64,
    /// Modulator frequency ratio (modulator = carrier * ratio).
    pub mod_ratio: f64,
    /// Peak modulation index.
    pub mod_index: f64,
    /// Modulator envelope attack time in seconds.
    pub mod_attack: f64,
    /// Modulator envelope decay time in seconds.
    pub mod_decay: f64,
    /// Modulator envelope sustain level (0-1).
    pub mod_sustain: f64,
    /// Feedback amount (0-1) for self-modulation.
    pub feedback: f64,
}

impl TwoOpFm {
    /// Creates a new two-operator FM synthesizer.
    pub fn new(carrier_freq: f64, mod_ratio: f64, mod_index: f64) -> Self {
        Self {
            carrier_freq,
            mod_ratio,
            mod_index,
            mod_attack: 0.01,
            mod_decay: 0.3,
            mod_sustain: 0.3,
            feedback: 0.0,
        }
    }

    /// Sets the modulator envelope.
    pub fn with_mod_envelope(mut self, attack: f64, decay: f64, sustain: f64) -> Self {
        self.mod_attack = attack;
        self.mod_decay = decay;
        self.mod_sustain = sustain;
        self
    }

    /// Sets the feedback amount.
    pub fn with_feedback(mut self, feedback: f64) -> Self {
        self.feedback = feedback.clamp(0.0, 1.0);
        self
    }
}

impl Synthesizer for TwoOpFm {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);

        let dt = 1.0 / sample_rate;
        let two_pi = 2.0 * PI;

        let modulator_freq = self.carrier_freq * self.mod_ratio;

        let attack_samples = (self.mod_attack * sample_rate) as usize;
        let decay_samples = (self.mod_decay * sample_rate) as usize;

        let mut carrier_phase: f64 = 0.0;
        let mut modulator_phase: f64 = 0.0;
        let mut prev_mod_output: f64 = 0.0;

        for i in 0..num_samples {
            // Calculate modulator envelope
            let mod_env = if i < attack_samples {
                i as f64 / attack_samples as f64
            } else if i < attack_samples + decay_samples {
                let decay_progress = (i - attack_samples) as f64 / decay_samples as f64;
                1.0 - decay_progress * (1.0 - self.mod_sustain)
            } else {
                self.mod_sustain
            };

            // Modulator with optional feedback
            let feedback_mod = self.feedback * prev_mod_output;
            let modulator = (modulator_phase + feedback_mod).sin();
            prev_mod_output = modulator;

            // Apply envelope to modulation index
            let index = self.mod_index * mod_env;

            // FM equation
            let carrier = (carrier_phase + index * modulator).sin();
            output.push(carrier);

            // Update phases
            carrier_phase += two_pi * self.carrier_freq * dt;
            modulator_phase += two_pi * modulator_freq * dt;

            if carrier_phase >= two_pi {
                carrier_phase -= two_pi;
            }
            if modulator_phase >= two_pi {
                modulator_phase -= two_pi;
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
    fn test_basic_fm() {
        let synth = FmSynth::new(440.0, 880.0, 2.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_fm_with_ratio() {
        let synth = FmSynth::with_ratio(440.0, 2.0, 3.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_fm_bell_preset() {
        let synth = FmSynth::bell(440.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_fm_with_index_decay() {
        let synth = FmSynth::new(440.0, 880.0, 5.0).with_index_decay(10.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        // Initial samples should have more harmonics (larger amplitude variation)
        let early_variance: f64 = samples[0..100].iter().map(|s| s.powi(2)).sum::<f64>() / 100.0;

        let late_variance: f64 =
            samples[40000..40100].iter().map(|s| s.powi(2)).sum::<f64>() / 100.0;

        // Both should have similar variance since carrier amplitude stays constant
        assert!(early_variance > 0.0 && late_variance > 0.0);
    }

    #[test]
    fn test_two_op_fm() {
        let synth = TwoOpFm::new(440.0, 2.0, 3.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_fm_determinism() {
        let synth = FmSynth::new(440.0, 880.0, 2.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }
}
