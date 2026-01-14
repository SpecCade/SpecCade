//! Core synthesis types and simpler synthesis variants.

use serde::{Deserialize, Serialize};

use super::basic_types::{Filter, FreqSweep, NoiseType, OscillatorConfig, Waveform};

/// Synthesis type configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Synthesis {
    /// FM synthesis.
    FmSynth {
        /// Carrier frequency in Hz.
        carrier_freq: f64,
        /// Modulator frequency in Hz.
        modulator_freq: f64,
        /// Modulation index.
        modulation_index: f64,
        /// Optional frequency sweep.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
    },
    /// AM (Amplitude Modulation) synthesis.
    AmSynth {
        /// Carrier frequency in Hz.
        carrier_freq: f64,
        /// Modulator frequency in Hz.
        modulator_freq: f64,
        /// Modulation depth (0.0 to 1.0).
        modulation_depth: f64,
        /// Optional frequency sweep.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
    },
    /// Ring Modulation synthesis.
    ///
    /// Multiplies carrier and modulator directly (no DC offset),
    /// producing sum and difference frequencies for metallic/robotic timbres.
    RingModSynth {
        /// Carrier frequency in Hz.
        carrier_freq: f64,
        /// Modulator frequency in Hz.
        modulator_freq: f64,
        /// Wet/dry mix (0.0 = pure carrier, 1.0 = pure ring modulation).
        mix: f64,
        /// Optional frequency sweep.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
    },
    /// Karplus-Strong plucked string synthesis.
    KarplusStrong {
        /// Base frequency in Hz.
        frequency: f64,
        /// Decay factor (0.0 to 1.0).
        decay: f64,
        /// Blend factor for the lowpass filter.
        blend: f64,
    },
    /// Noise burst.
    NoiseBurst {
        /// Type of noise.
        noise_type: NoiseType,
        /// Optional filter.
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<Filter>,
    },
    /// Additive synthesis with multiple harmonics.
    Additive {
        /// Base frequency in Hz.
        base_freq: f64,
        /// Harmonic amplitudes (index 0 = fundamental).
        harmonics: Vec<f64>,
    },
    /// Simple waveform oscillator.
    Oscillator {
        /// Waveform type.
        waveform: Waveform,
        /// Frequency in Hz.
        frequency: f64,
        /// Optional frequency sweep.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
        /// Detune amount in cents (100 cents = 1 semitone).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detune: Option<f64>,
        /// Duty cycle for square/pulse waves (0.0 to 1.0, default 0.5).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duty: Option<f64>,
    },
    /// Multi-oscillator stack (subtractive synthesis).
    MultiOscillator {
        /// Base frequency in Hz.
        frequency: f64,
        /// Stack of oscillators to mix additively.
        oscillators: Vec<OscillatorConfig>,
        /// Optional frequency sweep applied to all oscillators.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
    },
    /// Pitched body synthesis (impact sounds with frequency sweep).
    PitchedBody {
        /// Starting frequency in Hz.
        start_freq: f64,
        /// Ending frequency in Hz.
        end_freq: f64,
    },
    /// Metallic synthesis with inharmonic partials.
    Metallic {
        /// Base frequency in Hz.
        base_freq: f64,
        /// Number of inharmonic partials.
        num_partials: usize,
        /// Inharmonicity factor (1.0 = harmonic, >1.0 = increasingly inharmonic).
        inharmonicity: f64,
    },
    /// Granular synthesis.
    Granular {
        /// Source material for grains.
        source: GranularSource,
        /// Grain size in milliseconds (10-500ms).
        grain_size_ms: f64,
        /// Grains per second (1-100).
        grain_density: f64,
        /// Random pitch variation in semitones.
        #[serde(default)]
        pitch_spread: f64,
        /// Random position jitter (0.0-1.0).
        #[serde(default)]
        position_spread: f64,
        /// Stereo spread (0.0-1.0).
        #[serde(default)]
        pan_spread: f64,
    },
    /// Wavetable synthesis with morphing.
    Wavetable {
        /// Wavetable source.
        table: WavetableSource,
        /// Base frequency in Hz.
        frequency: f64,
        /// Position in wavetable (0.0-1.0).
        #[serde(default)]
        position: f64,
        /// Optional position sweep over duration.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        position_sweep: Option<PositionSweep>,
        /// Number of unison voices (1-8).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        voices: Option<u8>,
        /// Detune amount in cents for unison.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detune: Option<f64>,
    },
    /// Phase Distortion synthesis (Casio CZ style).
    ///
    /// Creates complex timbres by warping the phase of a waveform non-linearly.
    /// The distortion amount typically decays over time, creating sounds that
    /// start bright and evolve to pure tones.
    PdSynth {
        /// Base frequency in Hz.
        frequency: f64,
        /// Initial distortion amount (0.0 = pure sine, higher = more harmonics).
        /// Typical range: 0.0 to 10.0
        distortion: f64,
        /// Distortion decay rate (higher = faster decay to pure sine).
        #[serde(default)]
        distortion_decay: f64,
        /// Waveform shape determining the distortion curve.
        waveform: super::synthesis_advanced::PdWaveform,
        /// Optional frequency sweep.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
    },
    /// Modal synthesis for struck/bowed physical objects.
    ///
    /// Simulates bells, chimes, marimbas, and other resonant objects by modeling
    /// their resonant modes. Each mode is a decaying sine wave at a specific
    /// frequency ratio with its own amplitude and decay time.
    Modal {
        /// Base frequency in Hz.
        frequency: f64,
        /// Bank of resonant modes defining the timbre.
        modes: Vec<super::synthesis_advanced::ModalMode>,
        /// Excitation type (how the object is struck/excited).
        excitation: super::synthesis_advanced::ModalExcitation,
        /// Optional frequency sweep applied to all modes.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
    },
    /// Vocoder synthesis with filter bank and formant animation.
    ///
    /// A vocoder transfers the spectral envelope from a modulator signal to a carrier.
    /// Since we're generating from scratch, we create procedural formant patterns
    /// that simulate speech-like envelope movements across frequency bands.
    Vocoder {
        /// Base frequency of carrier in Hz.
        carrier_freq: f64,
        /// Type of carrier waveform (sawtooth, pulse, or noise).
        carrier_type: super::synthesis_advanced::VocoderCarrierType,
        /// Number of filter bands (8-32 typical).
        num_bands: usize,
        /// Band spacing mode (linear or logarithmic).
        band_spacing: super::synthesis_advanced::VocoderBandSpacing,
        /// Envelope attack time in seconds (how fast bands respond).
        envelope_attack: f64,
        /// Envelope release time in seconds (how fast bands decay).
        envelope_release: f64,
        /// Formant animation rate in Hz (cycles per second for envelope patterns).
        #[serde(default = "default_formant_rate")]
        formant_rate: f64,
        /// Optional custom band configurations (overrides num_bands if provided).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        bands: Vec<super::synthesis_advanced::VocoderBand>,
    },
    /// Formant synthesis for vowel and voice sounds.
    ///
    /// Creates vowel and voice sounds using resonant filter banks tuned to formant
    /// frequencies. Human vowels are characterized by formant frequencies
    /// (F1, F2, F3, etc.) - resonant peaks in the spectrum.
    Formant {
        /// Base pitch frequency of the voice in Hz.
        frequency: f64,
        /// Optional custom formant configurations (overrides vowel preset if provided).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        formants: Vec<super::synthesis_advanced::FormantConfig>,
        /// Vowel preset to use (if formants not provided).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vowel: Option<super::synthesis_advanced::FormantVowel>,
        /// Optional second vowel for morphing transitions.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vowel_morph: Option<super::synthesis_advanced::FormantVowel>,
        /// Morph amount between vowels (0.0 = first vowel, 1.0 = second vowel).
        #[serde(default)]
        morph_amount: f64,
        /// Amount of noise mixed in for breathiness (0.0-1.0).
        #[serde(default)]
        breathiness: f64,
    },
    /// Vector synthesis with 2D crossfading between multiple sound sources.
    ///
    /// Places 2-4 sound sources at corners of a 2D space and crossfades between
    /// them based on position. The position can be animated over time to create
    /// evolving, morphing textures. Classic examples: Prophet VS, Korg Wavestation.
    Vector {
        /// Base frequency in Hz.
        frequency: f64,
        /// Four sources at corners: [A (top-left), B (top-right), C (bottom-left), D (bottom-right)].
        sources: [super::synthesis_advanced::VectorSource; 4],
        /// Static X position (0.0-1.0, used if path is empty).
        #[serde(default = "default_vector_position")]
        position_x: f64,
        /// Static Y position (0.0-1.0, used if path is empty).
        #[serde(default = "default_vector_position")]
        position_y: f64,
        /// Optional animated path (sequence of positions with durations).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        path: Vec<super::synthesis_advanced::VectorPathPoint>,
        /// Whether the path should loop.
        #[serde(default)]
        path_loop: bool,
        /// Interpolation curve for path animation.
        #[serde(default = "default_linear_curve")]
        path_curve: super::basic_types::SweepCurve,
    },
}

/// Granular synthesis source material.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GranularSource {
    /// Noise-based grains.
    Noise { noise_type: NoiseType },
    /// Tone-based grains.
    Tone { waveform: Waveform, frequency: f64 },
    /// Formant-based grains.
    Formant { frequency: f64, formant_freq: f64 },
}

/// Wavetable source for wavetable synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WavetableSource {
    /// sine -> saw -> square -> pulse morphing
    Basic,
    /// Classic analog-style waves
    Analog,
    /// Harsh digital tones
    Digital,
    /// Pulse width modulation table
    Pwm,
    /// Vocal formant-like
    Formant,
    /// Drawbar organ harmonics
    Organ,
}

/// Position sweep for wavetable synthesis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionSweep {
    /// Target position at end of sweep (0.0-1.0).
    pub end_position: f64,
    /// Sweep curve type.
    #[serde(default = "default_linear_curve")]
    pub curve: super::basic_types::SweepCurve,
}

fn default_linear_curve() -> super::basic_types::SweepCurve {
    super::basic_types::SweepCurve::Linear
}

fn default_formant_rate() -> f64 {
    2.0
}

fn default_vector_position() -> f64 {
    0.5
}
