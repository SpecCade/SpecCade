//! Draft compose (Pattern IR) recipe types for tracker music.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

use super::{ArrangementEntry, AutomationEntry, ItOptions, PatternNote, TrackerFormat, TrackerInstrument};

/// Parameters for the `music.tracker_song_compose_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MusicTrackerSongComposeV1Params {
    /// Song internal name (used in IT/XM module).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Song display title (for metadata).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Tracker format: "xm" or "it".
    pub format: TrackerFormat,
    /// Beats per minute (30-300).
    pub bpm: u16,
    /// Tracker speed (ticks per row, 1-31).
    pub speed: u8,
    /// Number of channels.
    ///
    /// - XM: 1-32
    /// - IT: 1-64
    pub channels: u8,
    /// Whether the song should loop.
    ///
    /// Note: currently applied for XM via `restart_position`. IT output currently ignores looping.
    #[serde(default)]
    pub r#loop: bool,
    /// Restart position (order-table index) to jump to when looping.
    ///
    /// Note: currently used for XM only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart_position: Option<u16>,
    /// Instrument definitions.
    #[serde(default)]
    pub instruments: Vec<TrackerInstrument>,
    /// Reusable pattern fragments.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub defs: HashMap<String, PatternExpr>,
    /// Pattern definitions (Pattern IR).
    #[serde(default)]
    pub patterns: HashMap<String, ComposePattern>,
    /// Song arrangement (order of patterns).
    #[serde(default)]
    pub arrangement: Vec<ArrangementEntry>,
    /// Automation definitions (volume fades, tempo changes).
    #[serde(default)]
    pub automation: Vec<AutomationEntry>,
    /// IT-specific options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub it_options: Option<ItOptions>,
}

/// Pattern definition for compose specs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComposePattern {
    /// Number of rows in the pattern.
    pub rows: u16,
    /// Pattern IR program to expand.
    pub program: PatternExpr,
    /// Optional flat notes to merge on top (channel stored per note).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<PatternNote>>,
    /// Optional channel-keyed notes to merge on top.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<HashMap<String, Vec<PatternNote>>>,
}

/// Merge policy for overlapping cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergePolicy {
    /// Error on any double-write to the same cell.
    Error,
    /// Fieldwise merge; error on conflicting values.
    MergeFields,
    /// Fieldwise merge; conflicts resolved by taking the later writer's value.
    LastWins,
}

impl Default for MergePolicy {
    fn default() -> Self {
        MergePolicy::MergeFields
    }
}

/// Sequence mode for `emit_seq` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeqMode {
    /// Cycle through values (wraps).
    Cycle,
    /// Require exactly one value per emitted event.
    Once,
}

/// Sequence of values aligned to emitted events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seq<T> {
    pub mode: SeqMode,
    pub values: Vec<T>,
}

/// Pattern IR expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum PatternExpr {
    Stack {
        #[serde(default)]
        merge: MergePolicy,
        parts: Vec<PatternExpr>,
    },
    Concat {
        parts: Vec<ConcatPart>,
    },
    Repeat {
        times: u16,
        len_rows: u16,
        body: Box<PatternExpr>,
    },
    Shift {
        rows: i32,
        body: Box<PatternExpr>,
    },
    Slice {
        start: i32,
        len: i32,
        body: Box<PatternExpr>,
    },
    Ref {
        name: String,
    },
    Emit {
        at: TimeExpr,
        cell: CellTemplate,
    },
    EmitSeq {
        at: TimeExpr,
        cell: CellTemplate,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note_seq: Option<Seq<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        inst_seq: Option<Seq<u8>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vol_seq: Option<Seq<u8>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effect_seq: Option<Seq<u8>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        param_seq: Option<Seq<u8>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effect_name_seq: Option<Seq<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effect_xy_seq: Option<Seq<[u8; 2]>>,
    },
    Transform {
        ops: Vec<TransformOp>,
        body: Box<PatternExpr>,
    },
    Prob {
        p_permille: u16,
        seed_salt: String,
        body: Box<PatternExpr>,
    },
    Choose {
        seed_salt: String,
        choices: Vec<WeightedChoice>,
    },
}

/// Concatenation part with declared length.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConcatPart {
    pub len_rows: u16,
    pub body: PatternExpr,
}

/// Weighted choice entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeightedChoice {
    pub weight: u32,
    pub body: PatternExpr,
}

/// Time expression (row selector).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum TimeExpr {
    Range {
        start: i32,
        step: i32,
        count: u32,
    },
    List {
        rows: Vec<i32>,
    },
    Euclid {
        pulses: u32,
        steps: u32,
        #[serde(default)]
        rotate: i32,
        #[serde(default = "default_stride")]
        stride: i32,
        #[serde(default)]
        offset: i32,
    },
    Pattern {
        pattern: String,
        #[serde(default = "default_stride")]
        stride: i32,
        #[serde(default)]
        offset: i32,
    },
}

/// Transform operators for Pattern IR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransformOp {
    /// Transpose note names by semitones.
    TransposeSemitones { semitones: i32 },
    /// Multiply volume by a rational (mul/div).
    VolMul { mul: u32, div: u32 },
    /// Set missing fields.
    Set {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        inst: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vol: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effect: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        param: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effect_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effect_xy: Option<[u8; 2]>,
    },
}

/// Cell template emitted by Pattern IR (row comes from TimeExpr).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CellTemplate {
    /// Channel number (0-indexed).
    pub channel: u8,
    /// Note name (string or MIDI number); omitted means "use instrument base note".
    #[serde(default, deserialize_with = "deserialize_note_opt", skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Instrument index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inst: Option<u8>,
    /// Volume (0-64, optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

fn default_stride() -> i32 {
    1
}

fn deserialize_note_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct NoteOptVisitor;

    impl<'de> Visitor<'de> for NoteOptVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string note name, MIDI note number, or null")
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(midi_to_note_name(v as u8)))
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(midi_to_note_name(v as u8)))
        }
    }

    deserializer.deserialize_any(NoteOptVisitor)
}

fn midi_to_note_name(midi: u8) -> String {
    const NOTES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (midi / 12) as i32 - 1;
    let note_idx = (midi % 12) as usize;
    format!("{}{}", NOTES[note_idx], octave)
}
