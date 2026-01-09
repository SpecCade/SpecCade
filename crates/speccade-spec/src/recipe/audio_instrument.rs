//! Audio instrument recipe types.

use serde::{Deserialize, Serialize};

use super::audio_sfx::{Envelope, Synthesis};

/// Parameters for the `audio_instrument.synth_patch_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioInstrumentSynthPatchV1Params {
    /// Duration of the instrument sample in seconds.
    #[serde(default = "default_note_duration")]
    pub note_duration_seconds: f64,
    /// Sample rate in Hz (22050, 44100, or 48000).
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    /// Synthesis configuration.
    pub synthesis: Synthesis,
    /// ADSR envelope.
    pub envelope: Envelope,
    /// Optional notes to generate (MIDI note numbers or note names).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<NoteSpec>>,
    /// Whether to generate loop points.
    #[serde(default)]
    pub generate_loop_points: bool,
}

fn default_note_duration() -> f64 {
    1.0
}

fn default_sample_rate() -> u32 {
    44100
}

/// Note specification for multi-note instrument samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NoteSpec {
    /// MIDI note number (0-127).
    MidiNote(u8),
    /// Note name (e.g., "C4", "A#3").
    NoteName(String),
}

impl NoteSpec {
    /// Converts the note spec to a MIDI note number.
    pub fn to_midi_note(&self) -> Option<u8> {
        match self {
            NoteSpec::MidiNote(n) => Some(*n),
            NoteSpec::NoteName(name) => parse_note_name(name),
        }
    }

    /// Converts the note spec to a frequency in Hz.
    pub fn to_frequency(&self) -> Option<f64> {
        self.to_midi_note().map(midi_to_frequency)
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_midi_to_frequency() {
        // A4 = 440 Hz
        assert!((midi_to_frequency(69) - 440.0).abs() < 0.001);
        // C4 ~= 261.63 Hz
        assert!((midi_to_frequency(60) - 261.63).abs() < 0.1);
    }

    #[test]
    fn test_note_spec_to_frequency() {
        let note = NoteSpec::NoteName("A4".to_string());
        let freq = note.to_frequency().unwrap();
        assert!((freq - 440.0).abs() < 0.001);

        let note = NoteSpec::MidiNote(69);
        let freq = note.to_frequency().unwrap();
        assert!((freq - 440.0).abs() < 0.001);
    }
}
