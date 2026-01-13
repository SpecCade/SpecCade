//! Shared synthesis types for audio asset generation.
//!
//! This module contains types shared between SFX and instrument audio recipes.

use serde::{Deserialize, Serialize};

/// Basic waveform types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Waveform {
    /// Sine wave.
    Sine,
    /// Square wave.
    Square,
    /// Sawtooth wave.
    Sawtooth,
    /// Triangle wave.
    Triangle,
    /// Pulse wave with variable duty cycle.
    Pulse,
}

/// Frequency sweep parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreqSweep {
    /// Target frequency at end of sweep.
    pub end_freq: f64,
    /// Sweep curve type.
    pub curve: SweepCurve,
}

/// Sweep curve type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SweepCurve {
    /// Linear interpolation.
    Linear,
    /// Exponential interpolation.
    Exponential,
    /// Logarithmic interpolation.
    Logarithmic,
}

/// ADSR envelope parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    /// Attack time in seconds.
    pub attack: f64,
    /// Decay time in seconds.
    pub decay: f64,
    /// Sustain level (0.0 to 1.0).
    pub sustain: f64,
    /// Release time in seconds.
    pub release: f64,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
        }
    }
}

/// Pitch envelope for modulating frequency over time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PitchEnvelope {
    /// Attack time in seconds.
    pub attack: f64,
    /// Decay time in seconds.
    pub decay: f64,
    /// Sustain level (0.0 to 1.0).
    pub sustain: f64,
    /// Release time in seconds.
    pub release: f64,
    /// Pitch depth in semitones (can be positive or negative).
    pub depth: f64,
}

impl Default for PitchEnvelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
            depth: 0.0,
        }
    }
}

/// Configuration for a single oscillator in a multi-oscillator stack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OscillatorConfig {
    /// Waveform type.
    pub waveform: Waveform,
    /// Volume/amplitude of this oscillator (0.0 to 1.0).
    #[serde(default = "default_oscillator_volume")]
    pub volume: f64,
    /// Detune amount in cents (100 cents = 1 semitone).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detune: Option<f64>,
    /// Phase offset in radians (0 to 2*PI).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<f64>,
    /// Duty cycle for square/pulse waves (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duty: Option<f64>,
}

fn default_oscillator_volume() -> f64 {
    1.0
}

/// Noise type for noise-based synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseType {
    /// White noise (equal energy per frequency).
    White,
    /// Pink noise (1/f spectrum).
    Pink,
    /// Brown noise (1/f^2 spectrum).
    Brown,
}

/// Filter configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Filter {
    /// Low-pass filter.
    Lowpass {
        /// Cutoff frequency in Hz.
        cutoff: f64,
        /// Resonance (Q factor).
        resonance: f64,
        /// Optional target cutoff frequency for sweep.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cutoff_end: Option<f64>,
    },
    /// High-pass filter.
    Highpass {
        /// Cutoff frequency in Hz.
        cutoff: f64,
        /// Resonance (Q factor).
        resonance: f64,
        /// Optional target cutoff frequency for sweep.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cutoff_end: Option<f64>,
    },
    /// Band-pass filter.
    Bandpass {
        /// Center frequency in Hz.
        center: f64,
        /// Resonance (Q factor).
        resonance: f64,
        /// Optional target center frequency for sweep.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        center_end: Option<f64>,
    },
}

/// Note specification - can be MIDI number or note name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NoteSpec {
    /// MIDI note number (0-127).
    MidiNote(u8),
    /// Note name (e.g., "C4", "A#3", "Bb5").
    NoteName(String),
}

impl NoteSpec {
    /// Converts to frequency in Hz.
    pub fn to_frequency(&self) -> f64 {
        match self {
            NoteSpec::MidiNote(n) => midi_to_frequency(*n),
            NoteSpec::NoteName(name) => parse_note_name(name)
                .map(midi_to_frequency)
                .unwrap_or(261.63),
        }
    }
}

impl Default for NoteSpec {
    fn default() -> Self {
        NoteSpec::NoteName("C4".to_string())
    }
}

/// Converts a MIDI note number to frequency in Hz.
pub fn midi_to_frequency(midi_note: u8) -> f64 {
    440.0 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0)
}

/// Parses a note name (e.g., "C4", "A#3", "Bb5") to a MIDI note number.
pub fn parse_note_name(name: &str) -> Option<u8> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    let mut chars = name.chars();
    let note_letter = chars.next()?.to_ascii_uppercase();

    let base_semitone = match note_letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };

    let rest: String = chars.collect();
    let (accidental_offset, octave_str) = if let Some(stripped) = rest.strip_prefix('#') {
        (1i32, stripped)
    } else if let Some(stripped) = rest.strip_prefix('s') {
        (1i32, stripped)
    } else if let Some(stripped) = rest.strip_prefix('b') {
        (-1i32, stripped)
    } else {
        (0i32, rest.as_str())
    };

    let octave: i32 = octave_str.parse().ok()?;

    // MIDI note = (octave + 1) * 12 + semitone
    // C4 = 60, A4 = 69
    let midi = (octave + 1) * 12 + base_semitone + accidental_offset;

    if (0..=127).contains(&midi) {
        Some(midi as u8)
    } else {
        None
    }
}

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
        waveform: PdWaveform,
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
        modes: Vec<ModalMode>,
        /// Excitation type (how the object is struck/excited).
        excitation: ModalExcitation,
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
        carrier_type: VocoderCarrierType,
        /// Number of filter bands (8-32 typical).
        num_bands: usize,
        /// Band spacing mode (linear or logarithmic).
        band_spacing: VocoderBandSpacing,
        /// Envelope attack time in seconds (how fast bands respond).
        envelope_attack: f64,
        /// Envelope release time in seconds (how fast bands decay).
        envelope_release: f64,
        /// Formant animation rate in Hz (cycles per second for envelope patterns).
        #[serde(default = "default_formant_rate")]
        formant_rate: f64,
        /// Optional custom band configurations (overrides num_bands if provided).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        bands: Vec<VocoderBand>,
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
        formants: Vec<FormantConfig>,
        /// Vowel preset to use (if formants not provided).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vowel: Option<FormantVowel>,
        /// Optional second vowel for morphing transitions.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vowel_morph: Option<FormantVowel>,
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
        sources: [VectorSource; 4],
        /// Static X position (0.0-1.0, used if path is empty).
        #[serde(default = "default_vector_position")]
        position_x: f64,
        /// Static Y position (0.0-1.0, used if path is empty).
        #[serde(default = "default_vector_position")]
        position_y: f64,
        /// Optional animated path (sequence of positions with durations).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        path: Vec<VectorPathPoint>,
        /// Whether the path should loop.
        #[serde(default)]
        path_loop: bool,
        /// Interpolation curve for path animation.
        #[serde(default = "default_linear_curve")]
        path_curve: SweepCurve,
    },
}

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
    pub curve: SweepCurve,
}

fn default_linear_curve() -> SweepCurve {
    SweepCurve::Linear
}

fn default_formant_rate() -> f64 {
    2.0
}

fn default_vector_position() -> f64 {
    0.5
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

/// LFO (Low Frequency Oscillator) configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LfoConfig {
    /// Waveform type for the LFO.
    pub waveform: Waveform,
    /// LFO rate in Hz (typically 0.1-20 Hz).
    pub rate: f64,
    /// Modulation depth (0.0-1.0).
    pub depth: f64,
    /// Initial phase offset (0.0-1.0, optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<f64>,
}

/// Modulation target specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "target", rename_all = "snake_case")]
pub enum ModulationTarget {
    /// Modulate pitch (vibrato).
    Pitch {
        /// Maximum pitch deviation in semitones.
        semitones: f64,
    },
    /// Modulate volume (tremolo).
    Volume,
    /// Modulate filter cutoff frequency.
    FilterCutoff {
        /// Maximum cutoff frequency change in Hz.
        amount: f64,
    },
    /// Modulate stereo pan.
    Pan,
}

/// LFO modulation configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LfoModulation {
    /// LFO configuration.
    pub config: LfoConfig,
    /// Modulation target.
    pub target: ModulationTarget,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Waveform Tests
    // ========================================================================

    #[test]
    fn test_waveform_serde() {
        let waveforms = vec![
            Waveform::Sine,
            Waveform::Square,
            Waveform::Sawtooth,
            Waveform::Triangle,
            Waveform::Pulse,
        ];

        for waveform in waveforms {
            let json = serde_json::to_string(&waveform).unwrap();
            let parsed: Waveform = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, waveform);
        }
    }

    // ========================================================================
    // Envelope Tests
    // ========================================================================

    #[test]
    fn test_envelope_default() {
        let env = Envelope::default();
        assert_eq!(env.attack, 0.01);
        assert_eq!(env.decay, 0.1);
        assert_eq!(env.sustain, 0.5);
        assert_eq!(env.release, 0.2);
    }

    #[test]
    fn test_envelope_serde() {
        let env = Envelope {
            attack: 0.05,
            decay: 0.2,
            sustain: 0.7,
            release: 0.3,
        };

        let json = serde_json::to_string(&env).unwrap();
        let parsed: Envelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, env);
    }

    // ========================================================================
    // PitchEnvelope Tests
    // ========================================================================

    #[test]
    fn test_pitch_envelope_default() {
        let env = PitchEnvelope::default();
        assert_eq!(env.depth, 0.0);
    }

    #[test]
    fn test_pitch_envelope_serde() {
        let env = PitchEnvelope {
            attack: 0.02,
            decay: 0.15,
            sustain: 0.7,
            release: 0.25,
            depth: 12.0,
        };

        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("depth"));
        let parsed: PitchEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, env);
    }

    // ========================================================================
    // Filter Tests
    // ========================================================================

    #[test]
    fn test_filter_lowpass() {
        let filter = Filter::Lowpass {
            cutoff: 2000.0,
            resonance: 0.707,
            cutoff_end: None,
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("lowpass"));
        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_highpass() {
        let filter = Filter::Highpass {
            cutoff: 500.0,
            resonance: 0.5,
            cutoff_end: Some(2000.0),
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("highpass"));
        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    // ========================================================================
    // Synthesis Tests
    // ========================================================================

    #[test]
    fn test_synthesis_oscillator() {
        let synth = Synthesis::Oscillator {
            waveform: Waveform::Sine,
            frequency: 440.0,
            freq_sweep: None,
            detune: None,
            duty: None,
        };

        let json = serde_json::to_string(&synth).unwrap();
        assert!(json.contains("oscillator"));
        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, synth);
    }

    #[test]
    fn test_synthesis_karplus_strong() {
        let synth = Synthesis::KarplusStrong {
            frequency: 220.0,
            decay: 0.996,
            blend: 0.7,
        };

        let json = serde_json::to_string(&synth).unwrap();
        assert!(json.contains("karplus_strong"));
        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, synth);
    }

    #[test]
    fn test_synthesis_noise_burst() {
        let synth = Synthesis::NoiseBurst {
            noise_type: NoiseType::White,
            filter: Some(Filter::Lowpass {
                cutoff: 1000.0,
                resonance: 0.5,
                cutoff_end: None,
            }),
        };

        let json = serde_json::to_string(&synth).unwrap();
        assert!(json.contains("noise_burst"));
        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, synth);
    }

    // ========================================================================
    // NoteSpec Tests
    // ========================================================================

    #[test]
    fn test_note_spec_midi_note() {
        let note = NoteSpec::MidiNote(69);
        let freq = note.to_frequency();
        assert!((freq - 440.0).abs() < 0.001);

        let json = serde_json::to_string(&note).unwrap();
        let parsed: NoteSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, note);
    }

    #[test]
    fn test_note_spec_note_name() {
        let note = NoteSpec::NoteName("C4".to_string());
        let freq = note.to_frequency();
        assert!((freq - 261.63).abs() < 0.1);

        let json = serde_json::to_string(&note).unwrap();
        let parsed: NoteSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, note);
    }

    #[test]
    fn test_note_spec_default() {
        let note = NoteSpec::default();
        assert_eq!(note, NoteSpec::NoteName("C4".to_string()));
    }

    #[test]
    fn test_note_spec_invalid_name() {
        let note = NoteSpec::NoteName("InvalidNote".to_string());
        let freq = note.to_frequency();
        // Should fall back to C4 (261.63 Hz)
        assert!((freq - 261.63).abs() < 0.1);
    }

    // ========================================================================
    // parse_note_name Tests
    // ========================================================================

    #[test]
    fn test_parse_note_name() {
        assert_eq!(parse_note_name("C4"), Some(60));
        assert_eq!(parse_note_name("A4"), Some(69));
        assert_eq!(parse_note_name("C#4"), Some(61));
        assert_eq!(parse_note_name("Db4"), Some(61));
        assert_eq!(parse_note_name("C0"), Some(12));
        assert_eq!(parse_note_name("G9"), Some(127));
        assert_eq!(parse_note_name(""), None);
        assert_eq!(parse_note_name("X4"), None);
    }

    #[test]
    fn test_parse_note_name_edge_cases() {
        // Lowest MIDI note
        assert_eq!(parse_note_name("C-1"), Some(0));
        // Highest MIDI note
        assert_eq!(parse_note_name("G9"), Some(127));
        // Out of range
        assert_eq!(parse_note_name("C10"), None);
        assert_eq!(parse_note_name("C-2"), None);
    }

    // ========================================================================
    // midi_to_frequency Tests
    // ========================================================================

    #[test]
    fn test_midi_to_frequency() {
        // A4 = 440 Hz
        assert!((midi_to_frequency(69) - 440.0).abs() < 0.001);
        // C4 ~= 261.63 Hz
        assert!((midi_to_frequency(60) - 261.63).abs() < 0.1);
        // A3 = 220 Hz
        assert!((midi_to_frequency(57) - 220.0).abs() < 0.001);
        // A5 = 880 Hz
        assert!((midi_to_frequency(81) - 880.0).abs() < 0.001);
    }
}
