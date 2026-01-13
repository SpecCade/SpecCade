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

/// A single formant configuration.
#[derive(Debug, Clone, Copy)]
pub struct Formant {
    /// Center frequency in Hz.
    pub frequency: f64,
    /// Amplitude/gain of this formant (0.0-1.0).
    pub amplitude: f64,
    /// Bandwidth (Q factor) of the resonant filter.
    pub bandwidth: f64,
}

impl Formant {
    /// Creates a new formant configuration.
    ///
    /// # Arguments
    /// * `frequency` - Center frequency in Hz
    /// * `amplitude` - Amplitude/gain (0.0-1.0)
    /// * `bandwidth` - Bandwidth/Q factor
    pub fn new(frequency: f64, amplitude: f64, bandwidth: f64) -> Self {
        Self {
            frequency: frequency.clamp(20.0, 20000.0),
            amplitude: amplitude.clamp(0.0, 1.0),
            bandwidth: bandwidth.clamp(0.5, 20.0),
        }
    }
}

/// Vowel target with named formant frequencies.
#[derive(Debug, Clone)]
pub struct VowelTarget {
    /// Name of the vowel (e.g., "a", "i", "u").
    pub name: String,
    /// First formant frequency (F1) in Hz.
    pub f1: f64,
    /// Second formant frequency (F2) in Hz.
    pub f2: f64,
    /// Third formant frequency (F3) in Hz.
    pub f3: f64,
    /// Optional fourth formant frequency (F4) in Hz.
    pub f4: Option<f64>,
    /// Optional fifth formant frequency (F5) in Hz.
    pub f5: Option<f64>,
}

impl VowelTarget {
    /// Creates a new vowel target with 3 formants.
    pub fn new(name: &str, f1: f64, f2: f64, f3: f64) -> Self {
        Self {
            name: name.to_string(),
            f1,
            f2,
            f3,
            f4: None,
            f5: None,
        }
    }

    /// Creates a new vowel target with 5 formants.
    pub fn with_all_formants(name: &str, f1: f64, f2: f64, f3: f64, f4: f64, f5: f64) -> Self {
        Self {
            name: name.to_string(),
            f1,
            f2,
            f3,
            f4: Some(f4),
            f5: Some(f5),
        }
    }

    /// Converts vowel target to formant configurations.
    pub fn to_formants(&self) -> Vec<Formant> {
        let mut formants = vec![
            Formant::new(self.f1, 1.0, 5.0),
            Formant::new(self.f2, 0.7, 6.0),
            Formant::new(self.f3, 0.5, 7.0),
        ];

        if let Some(f4) = self.f4 {
            formants.push(Formant::new(f4, 0.3, 8.0));
        }

        if let Some(f5) = self.f5 {
            formants.push(Formant::new(f5, 0.2, 9.0));
        }

        formants
    }

    /// Preset for vowel /a/ (ah as in "father").
    pub fn vowel_a() -> Self {
        Self::with_all_formants("a", 800.0, 1200.0, 2800.0, 3500.0, 4500.0)
    }

    /// Preset for vowel /i/ (ee as in "feet").
    pub fn vowel_i() -> Self {
        Self::with_all_formants("i", 280.0, 2250.0, 2890.0, 3500.0, 4500.0)
    }

    /// Preset for vowel /u/ (oo as in "boot").
    pub fn vowel_u() -> Self {
        Self::with_all_formants("u", 310.0, 870.0, 2250.0, 3500.0, 4500.0)
    }

    /// Preset for vowel /e/ (eh as in "bed").
    pub fn vowel_e() -> Self {
        Self::with_all_formants("e", 530.0, 1840.0, 2480.0, 3500.0, 4500.0)
    }

    /// Preset for vowel /o/ (oh as in "boat").
    pub fn vowel_o() -> Self {
        Self::with_all_formants("o", 500.0, 1000.0, 2800.0, 3500.0, 4500.0)
    }
}

/// Vowel morph configuration for transitioning between vowels.
#[derive(Debug, Clone)]
pub struct VowelMorph {
    /// Target vowel to morph towards.
    pub target: VowelTarget,
    /// Morph amount (0.0 = original vowel, 1.0 = target vowel).
    pub amount: f64,
}

impl VowelMorph {
    /// Creates a new vowel morph configuration.
    pub fn new(target: VowelTarget, amount: f64) -> Self {
        Self {
            target,
            amount: amount.clamp(0.0, 1.0),
        }
    }
}

/// Preset vowel type for easy configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VowelPreset {
    /// /a/ (ah) vowel.
    A,
    /// /i/ (ee) vowel.
    I,
    /// /u/ (oo) vowel.
    U,
    /// /e/ (eh) vowel.
    E,
    /// /o/ (oh) vowel.
    O,
}

impl VowelPreset {
    /// Converts preset to vowel target.
    pub fn to_vowel_target(self) -> VowelTarget {
        match self {
            VowelPreset::A => VowelTarget::vowel_a(),
            VowelPreset::I => VowelTarget::vowel_i(),
            VowelPreset::U => VowelTarget::vowel_u(),
            VowelPreset::E => VowelTarget::vowel_e(),
            VowelPreset::O => VowelTarget::vowel_o(),
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_formant_basic() {
        let synth = FormantSynth::vowel_a(110.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s), "Sample {} out of range", s);
        }
    }

    #[test]
    fn test_formant_vowel_i() {
        let synth = FormantSynth::vowel_i(220.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_formant_vowel_u() {
        let synth = FormantSynth::vowel_u(165.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_formant_choir_ah() {
        let synth = FormantSynth::choir_ah(110.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        assert!(synth.breathiness > 0.0);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_formant_creature_growl() {
        let synth = FormantSynth::creature_growl(55.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        assert!(synth.breathiness > 0.3);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_formant_custom_formants() {
        let formants = vec![
            Formant::new(400.0, 1.0, 5.0),
            Formant::new(1000.0, 0.7, 6.0),
            Formant::new(2500.0, 0.5, 7.0),
        ];
        let synth = FormantSynth::new(110.0, formants);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_formant_determinism() {
        let synth = FormantSynth::vowel_a(110.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_formant_different_seeds() {
        let synth = FormantSynth::choir_ah(110.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(43);

        let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

        // Should be different due to noise in breathiness
        assert_ne!(samples1, samples2);
    }

    #[test]
    fn test_formant_empty_samples() {
        let synth = FormantSynth::vowel_a(110.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(0, 44100.0, &mut rng);

        assert!(samples.is_empty());
    }

    #[test]
    fn test_formant_vowel_morph() {
        let synth = FormantSynth::with_vowel(110.0, VowelPreset::A)
            .with_vowel_morph(VowelPreset::I, 0.5);

        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_formant_breathiness_range() {
        let synth1 = FormantSynth::vowel_a(110.0).with_breathiness(0.0);
        let synth2 = FormantSynth::vowel_a(110.0).with_breathiness(1.0);

        assert_eq!(synth1.breathiness, 0.0);
        assert_eq!(synth2.breathiness, 1.0);

        // Out of range should be clamped
        let synth3 = FormantSynth::vowel_a(110.0).with_breathiness(2.0);
        assert_eq!(synth3.breathiness, 1.0);
    }

    #[test]
    fn test_vowel_target_formants() {
        let vowel = VowelTarget::vowel_a();
        let formants = vowel.to_formants();

        // Should have at least 3 formants, possibly more
        assert!(formants.len() >= 3);

        // F1 for /a/ should be around 800 Hz
        assert!((formants[0].frequency - 800.0).abs() < 1.0);
    }

    #[test]
    fn test_formant_clamping() {
        let formant = Formant::new(10.0, 2.0, 0.1);
        assert!(formant.frequency >= 20.0);
        assert!(formant.amplitude <= 1.0);
        assert!(formant.bandwidth >= 0.5);

        let formant2 = Formant::new(30000.0, -0.5, 50.0);
        assert!(formant2.frequency <= 20000.0);
        assert!(formant2.amplitude >= 0.0);
        assert!(formant2.bandwidth <= 20.0);
    }

    #[test]
    fn test_all_vowel_presets() {
        let presets = [
            VowelPreset::A,
            VowelPreset::I,
            VowelPreset::U,
            VowelPreset::E,
            VowelPreset::O,
        ];

        for preset in presets {
            let synth = FormantSynth::with_vowel(110.0, preset);
            let mut rng = create_rng(42);
            let samples = synth.synthesize(1000, 44100.0, &mut rng);

            assert_eq!(samples.len(), 1000);
            for &s in &samples {
                assert!((-1.0..=1.0).contains(&s));
            }
        }
    }
}
