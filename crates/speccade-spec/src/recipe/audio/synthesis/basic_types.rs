//! Basic synthesis types: waveforms, envelopes, filters, and note specifications.

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
