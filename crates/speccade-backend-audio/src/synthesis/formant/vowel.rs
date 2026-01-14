//! Vowel configuration and formant frequency definitions.

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
