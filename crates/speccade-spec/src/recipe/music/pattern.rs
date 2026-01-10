//! Pattern definitions for tracker modules.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Pattern definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TrackerPattern {
    /// Number of rows in the pattern.
    #[serde(default)]
    pub rows: u16,
    /// Note data organized by channel (new format).
    /// Key is channel number (as string), value is list of notes for that channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<HashMap<String, Vec<PatternNote>>>,
    /// Note data as flat array (old format).
    /// Each note includes channel field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<PatternNote>>,
}

impl TrackerPattern {
    /// Get flattened note data with channel info.
    /// Works with both the new `notes` (channel-keyed) format and old `data` (flat array) format.
    pub fn flat_notes(&self) -> Vec<(u8, &PatternNote)> {
        let mut result = Vec::new();

        // Handle new channel-keyed format
        if let Some(ref notes_map) = self.notes {
            for (channel_str, notes) in notes_map {
                if let Ok(channel) = channel_str.parse::<u8>() {
                    for note in notes {
                        result.push((channel, note));
                    }
                }
            }
        }

        // Handle old flat array format
        if let Some(ref data) = self.data {
            for note in data {
                let channel = note.channel.unwrap_or(0);
                result.push((channel, note));
            }
        }

        // Sort by row then channel
        result.sort_by(|a, b| a.1.row.cmp(&b.1.row).then(a.0.cmp(&b.0)));
        result
    }
}

/// A single note event in a pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PatternNote {
    /// Row number (0-indexed).
    #[serde(default)]
    pub row: u16,
    /// Channel number (0-indexed). Used when notes are in flat array format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    /// Note name (e.g., "C4", "---" for note off, "..." for no note) or MIDI note number.
    #[serde(default, deserialize_with = "deserialize_note")]
    pub note: String,
    /// Instrument index.
    #[serde(default, alias = "instrument")]
    pub inst: u8,
    /// Volume (0-64, optional).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "volume")]
    pub vol: Option<u8>,
    /// Effect command number (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<u8>,
    /// Effect parameter (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<u8>,
    /// Effect name (e.g., "arpeggio", "portamento").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_name: Option<String>,
    /// Effect XY parameter as [X, Y] nibbles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_xy: Option<[u8; 2]>,
}

/// Deserialize note field that can be either a string or a MIDI note number.
fn deserialize_note<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct NoteVisitor;

    impl<'de> Visitor<'de> for NoteVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string note name or MIDI note number")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.to_string())
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(v)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
            // Convert MIDI note number to note name
            Ok(midi_to_note_name(v as u8))
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(midi_to_note_name(v as u8))
        }
    }

    deserializer.deserialize_any(NoteVisitor)
}

/// Convert MIDI note number to note name.
fn midi_to_note_name(midi: u8) -> String {
    const NOTES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (midi / 12) as i32 - 1;
    let note_idx = (midi % 12) as usize;
    format!("{}{}", NOTES[note_idx], octave)
}
