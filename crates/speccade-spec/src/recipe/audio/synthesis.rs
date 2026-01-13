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
