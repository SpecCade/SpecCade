//! Advanced synthesis types: Phase Distortion, Modal, Vocoder, Formant, and Vector synthesis.

use serde::{Deserialize, Serialize};

/// Phase distortion waveform shape.
///
/// Different distortion curves produce different timbral characteristics
/// in Phase Distortion synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdWaveform {
    /// Resonant-like tone using power curve distortion.
    /// Creates resonant filter-like tones.
    Resonant,
    /// Sawtooth-like asymmetric distortion.
    /// Compresses one half of the waveform, creating saw-like harmonics.
    Sawtooth,
    /// Pulse-like distortion with sharp phase transition.
    /// Creates square/pulse wave characteristics.
    Pulse,
}

/// A single resonant mode in modal synthesis.
///
/// Modal synthesis simulates physical objects by modeling their resonant modes.
/// Each mode represents a frequency at which the object naturally vibrates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModalMode {
    /// Frequency ratio relative to the fundamental (1.0 = fundamental).
    pub freq_ratio: f64,
    /// Amplitude of this mode (0.0 to 1.0).
    pub amplitude: f64,
    /// Decay time in seconds.
    pub decay_time: f64,
}

/// Excitation type for modal synthesis.
///
/// The excitation determines how the resonant modes are initially excited,
/// affecting the attack character of the sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModalExcitation {
    /// Single impulse excitation (sharp attack, like striking with a hard mallet).
    Impulse,
    /// Noise burst excitation (softer, more complex attack).
    Noise,
    /// Pluck-like excitation (quick attack with some harmonic content).
    Pluck,
}

/// Band spacing mode for vocoder filter bank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VocoderBandSpacing {
    /// Linear spacing between bands (equal Hz between centers).
    Linear,
    /// Logarithmic spacing (equal ratio between bands, more perceptually uniform).
    Logarithmic,
}

/// Carrier waveform type for vocoder synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VocoderCarrierType {
    /// Sawtooth wave - rich in harmonics, classic vocoder sound.
    Sawtooth,
    /// Pulse wave - hollow, more synthetic sound.
    Pulse,
    /// White noise - whispery, unvoiced consonant-like sound.
    Noise,
}

/// A single vocoder band configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VocoderBand {
    /// Center frequency of the band in Hz.
    pub center_freq: f64,
    /// Bandwidth (Q factor) of the band filter.
    pub bandwidth: f64,
    /// Envelope pattern for this band (amplitude values over time, 0.0-1.0).
    /// If empty, a default formant animation is used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub envelope_pattern: Vec<f64>,
}

/// Configuration for a single formant in formant synthesis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormantConfig {
    /// Center frequency of the formant in Hz.
    pub frequency: f64,
    /// Amplitude/gain of this formant (0.0-1.0).
    pub amplitude: f64,
    /// Bandwidth (Q factor) of the resonant filter.
    pub bandwidth: f64,
}

/// Vowel preset for formant synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormantVowel {
    /// /a/ (ah) as in "father".
    A,
    /// /i/ (ee) as in "feet".
    I,
    /// /u/ (oo) as in "boot".
    U,
    /// /e/ (eh) as in "bed".
    E,
    /// /o/ (oh) as in "boat".
    O,
}

/// Source waveform type for vector synthesis corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorSourceType {
    /// Sine wave.
    Sine,
    /// Sawtooth wave.
    Saw,
    /// Square wave with 50% duty cycle.
    Square,
    /// Triangle wave.
    Triangle,
    /// White noise.
    Noise,
    /// Wavetable-based source (uses additive harmonics for variety).
    Wavetable,
}

/// A single source in the vector synthesis grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VectorSource {
    /// Type of waveform for this source.
    pub source_type: VectorSourceType,
    /// Frequency ratio relative to the base frequency (1.0 = unison).
    #[serde(default = "default_freq_ratio")]
    pub frequency_ratio: f64,
}

fn default_freq_ratio() -> f64 {
    1.0
}

/// A point in a vector path animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VectorPathPoint {
    /// X position (0.0-1.0).
    pub x: f64,
    /// Y position (0.0-1.0).
    pub y: f64,
    /// Duration in seconds to reach this position from the previous point.
    pub duration: f64,
}
