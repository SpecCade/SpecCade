//! Formant synthesis implementation.
//!
//! Formant synthesis creates vowel and voice sounds using resonant filter banks
//! tuned to formant frequencies. Human vowels are characterized by formant
//! frequencies (F1, F2, F3, etc.) - resonant peaks in the spectrum.
//!
//! Typical formant ranges:
//! - F1 (200-900 Hz): Related to tongue height (open/closed)
//! - F2 (700-2600 Hz): Related to tongue position (front/back)
//! - F3 (1800-3500 Hz): Speaker characteristics

use std::f64::consts::PI;

use rand::Rng;
use rand_pcg::Pcg32;

use crate::filter::{BiquadCoeffs, BiquadFilter};
use crate::oscillator::PhaseAccumulator;

use super::Synthesizer;

mod vowel;
pub use vowel::{Formant, VowelMorph, VowelPreset, VowelTarget};

#[cfg(test)]
mod tests;

/// Formant synthesizer.
///
/// Creates vowel and voice sounds using parallel resonant filters applied
/// to an excitation source (typically a pulse train or noise).
#[derive(Debug, Clone)]
pub struct FormantSynth {
    /// Base pitch frequency of the voice in Hz.
    pub frequency: f64,
    /// Formant configurations (if provided, overrides vowel).
    pub formants: Vec<Formant>,
    /// Vowel preset to use (if formants not provided).
    pub vowel: Option<VowelPreset>,
    /// Optional vowel morph configuration.
    pub vowel_morph: Option<VowelMorph>,
    /// Amount of noise mixed in (0.0-1.0) for breathiness.
    pub breathiness: f64,
}

impl FormantSynth {
    /// Creates a new formant synthesizer with custom formants.
    ///
    /// # Arguments
    /// * `frequency` - Base pitch frequency in Hz
    /// * `formants` - Custom formant configurations
    pub fn new(frequency: f64, formants: Vec<Formant>) -> Self {
        Self {
            frequency: frequency.max(20.0),
            formants,
            vowel: None,
            vowel_morph: None,
            breathiness: 0.0,
        }
    }

    /// Creates a formant synthesizer with a vowel preset.
    ///
    /// # Arguments
    /// * `frequency` - Base pitch frequency in Hz
    /// * `vowel` - Vowel preset to use
    pub fn with_vowel(frequency: f64, vowel: VowelPreset) -> Self {
        Self {
            frequency: frequency.max(20.0),
            formants: Vec::new(),
            vowel: Some(vowel),
            vowel_morph: None,
            breathiness: 0.0,
        }
    }

    /// Sets the breathiness amount.
    pub fn with_breathiness(mut self, breathiness: f64) -> Self {
        self.breathiness = breathiness.clamp(0.0, 1.0);
        self
    }

    /// Sets a vowel morph target.
    pub fn with_vowel_morph(mut self, target_vowel: VowelPreset, amount: f64) -> Self {
        self.vowel_morph = Some(VowelMorph::new(target_vowel.to_vowel_target(), amount));
        self
    }

    /// Preset for vowel /a/ (ah) sound.
    pub fn vowel_a(frequency: f64) -> Self {
        Self::with_vowel(frequency, VowelPreset::A)
    }

    /// Preset for vowel /i/ (ee) sound.
    pub fn vowel_i(frequency: f64) -> Self {
        Self::with_vowel(frequency, VowelPreset::I)
    }

    /// Preset for vowel /u/ (oo) sound.
    pub fn vowel_u(frequency: f64) -> Self {
        Self::with_vowel(frequency, VowelPreset::U)
    }

    /// Preset for choir "ah" sound.
    ///
    /// A warm, resonant choir vowel with slight breathiness.
    pub fn choir_ah(frequency: f64) -> Self {
        Self::with_vowel(frequency, VowelPreset::A).with_breathiness(0.15)
    }

    /// Preset for creature growl sound.
    ///
    /// Low-pitched growling sound with heavy breathiness.
    pub fn creature_growl(frequency: f64) -> Self {
        // Use lower formants for a more guttural sound
        let formants = vec![
            Formant::new(300.0, 1.0, 3.0),
            Formant::new(600.0, 0.8, 4.0),
            Formant::new(1200.0, 0.4, 5.0),
            Formant::new(2000.0, 0.2, 6.0),
        ];
        Self::new(frequency, formants).with_breathiness(0.4)
    }

    /// Gets the effective formants for synthesis.
    fn get_effective_formants(&self) -> Vec<Formant> {
        if !self.formants.is_empty() {
            return self.formants.clone();
        }

        if let Some(vowel) = self.vowel {
            let vowel_target = vowel.to_vowel_target();

            if let Some(ref morph) = self.vowel_morph {
                // Interpolate between vowel and morph target
                let source = vowel_target.to_formants();
                let target = morph.target.to_formants();
                let amount = morph.amount;

                source
                    .iter()
                    .zip(target.iter())
                    .map(|(s, t)| {
                        Formant::new(
                            lerp(s.frequency, t.frequency, amount),
                            lerp(s.amplitude, t.amplitude, amount),
                            lerp(s.bandwidth, t.bandwidth, amount),
                        )
                    })
                    .collect()
            } else {
                vowel_target.to_formants()
            }
        } else {
            // Default to vowel /a/ if nothing specified
            VowelTarget::vowel_a().to_formants()
        }
    }

    /// Generates glottal pulse train excitation.
    fn generate_excitation(
        &self,
        num_samples: usize,
        sample_rate: f64,
        rng: &mut Pcg32,
    ) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);
        let mut phase_acc = PhaseAccumulator::new(sample_rate);

        // Glottal pulse shape parameters
        let open_phase = 0.4; // Portion of cycle where glottis is opening
        let close_phase = 0.3; // Portion of cycle where glottis is closing

        for _ in 0..num_samples {
            let phase = phase_acc.advance(self.frequency);
            let normalized_phase = phase / (2.0 * PI);

            // Generate glottal pulse waveform (Rosenberg model approximation)
            let pulse = if normalized_phase < open_phase {
                // Opening phase: rising
                let t = normalized_phase / open_phase;
                3.0 * t * t - 2.0 * t * t * t
            } else if normalized_phase < open_phase + close_phase {
                // Closing phase: falling
                let t = (normalized_phase - open_phase) / close_phase;
                1.0 - t * t
            } else {
                // Closed phase
                0.0
            };

            // Mix in noise for breathiness
            let noise = if self.breathiness > 0.0 {
                (rng.gen::<f64>() * 2.0 - 1.0) * self.breathiness
            } else {
                0.0
            };

            // Combine pulse and noise
            let excitation = pulse * (1.0 - self.breathiness * 0.5) + noise;
            output.push(excitation);
        }

        output
    }
}

impl Synthesizer for FormantSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        if num_samples == 0 {
            return Vec::new();
        }

        // Get formant configurations
        let formants = self.get_effective_formants();
        if formants.is_empty() {
            return vec![0.0; num_samples];
        }

        // Generate excitation signal
        let excitation = self.generate_excitation(num_samples, sample_rate, rng);

        // Process through parallel formant filters
        let mut output = vec![0.0; num_samples];

        for formant in &formants {
            // Skip formants that are too high for the sample rate
            if formant.frequency >= sample_rate / 2.0 || formant.frequency < 20.0 {
                continue;
            }

            // Create resonant bandpass filter for this formant
            // Use peaking EQ for formant shaping (gain boost at center frequency)
            let q = formant.bandwidth;
            let gain_db = 12.0; // Boost formant frequencies

            // Create filter coefficients
            let coeffs = BiquadCoeffs::peaking_eq(formant.frequency, q, gain_db, sample_rate);
            let mut filter = BiquadFilter::new(coeffs);

            // Process excitation through filter and add to output
            for (i, &exc) in excitation.iter().enumerate() {
                let filtered = filter.process(exc);
                output[i] += filtered * formant.amplitude;
            }
        }

        // Normalize output to [-1.0, 1.0]
        let max_val = output
            .iter()
            .map(|s| s.abs())
            .fold(0.0_f64, |a, b| a.max(b));

        if max_val > 0.0 {
            let scale = 1.0 / max_val;
            for sample in &mut output {
                *sample *= scale;
            }
        }

        output
    }
}

/// Linear interpolation helper.
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
