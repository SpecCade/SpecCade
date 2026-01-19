//! Draft compose (Pattern IR) recipe types for tracker music.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

use super::{
    ArrangementEntry, AutomationEntry, ItOptions, PatternNote, TrackerFormat, TrackerInstrument,
};

/// Channel reference (index or alias name).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChannelRef {
    Index(u8),
    Name(String),
}

impl Default for ChannelRef {
    fn default() -> Self {
        ChannelRef::Index(0)
    }
}

/// Instrument reference (index or alias name).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InstrumentRef {
    Index(u8),
    Name(String),
}

impl Default for InstrumentRef {
    fn default() -> Self {
        InstrumentRef::Index(0)
    }
}

/// Musical timebase for mapping bars/beats to tracker rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeBase {
    pub beats_per_bar: u16,
    pub rows_per_beat: u16,
}

/// Beat position within a pattern (0-indexed).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeatPos {
    pub bar: u16,
    #[serde(default)]
    pub beat: u16,
    #[serde(default)]
    pub sub: u16,
}

/// Beat delta for stepping within a pattern (can be negative).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeatDelta {
    pub beats: i32,
    pub sub: i32,
}

/// Supported key scales for harmony helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarmonyScale {
    Major,
    Minor,
}

/// Key definition for harmony helpers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarmonyKey {
    pub root: String,
    pub scale: HarmonyScale,
}

/// Symbol-form chord spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChordSpecSymbol {
    pub symbol: String,
}

/// Interval-form chord spec (escape hatch).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChordSpecIntervals {
    pub root: String,
    pub intervals: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bass: Option<String>,
}

/// Chord specification (symbol or interval form).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChordSpec {
    Symbol(ChordSpecSymbol),
    Intervals(ChordSpecIntervals),
}

/// Harmony chord change entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarmonyChordEntry {
    pub at: BeatPos,
    pub chord: ChordSpec,
}

/// Harmony helpers block (key + chord changes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Harmony {
    pub key: HarmonyKey,
    #[serde(default)]
    pub chords: Vec<HarmonyChordEntry>,
}

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
    /// Optional channel alias map (name -> channel index).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub channel_ids: HashMap<String, u8>,
    /// Optional instrument alias map (name -> instrument index).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub instrument_ids: HashMap<String, u8>,
    /// Optional timebase for mapping bars/beats to tracker rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timebase: Option<TimeBase>,
    /// Optional harmony helpers (key/chords) for pitch sequencing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harmony: Option<Harmony>,
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
    ///
    /// Mutually exclusive with `bars`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<u16>,
    /// Pattern length in bars (expanded to rows using the timebase).
    ///
    /// Mutually exclusive with `rows`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bars: Option<u16>,
    /// Optional timebase override for this pattern.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timebase: Option<TimeBase>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergePolicy {
    /// Error on any double-write to the same cell.
    Error,
    /// Fieldwise merge; error on conflicting values.
    #[default]
    MergeFields,
    /// Fieldwise merge; conflicts resolved by taking the later writer's value.
    LastWins,
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

/// Kind of pitch sequence (degree/chord-tone authoring).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PitchSeqKind {
    ScaleDegree,
    ChordTone,
}

/// Pitch sequence aligned to emitted events (compiled to note names during expansion).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PitchSeq {
    pub kind: PitchSeqKind,
    pub mode: SeqMode,
    pub values: Vec<String>,
    pub octave: i32,
    /// Whether to allow accidentals in `values` for `scale_degree`.
    #[serde(default)]
    pub allow_accidentals: bool,
}

/// Mirror axis for the `mirror` operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MirrorAxis {
    /// Mirror in time (retrograde).
    #[default]
    Time,
}

/// Filter criteria for the `filter` operator.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilterCriteria {
    /// Include only events at row >= value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_row: Option<i32>,
    /// Include only events at row < value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_row: Option<i32>,
    /// Include only events on this channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    /// If true, include only events with a note set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_note: Option<bool>,
    /// If true, include only events with an effect set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_effect: Option<bool>,
}

impl FilterCriteria {
    /// Returns true if at least one criterion is specified.
    pub fn has_any_criteria(&self) -> bool {
        self.min_row.is_some()
            || self.max_row.is_some()
            || self.channel.is_some()
            || self.has_note.is_some()
            || self.has_effect.is_some()
    }
}

/// Interleave part for the `interleave` operator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterleavePart {
    pub body: PatternExpr,
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
        cell: Box<CellTemplate>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note_seq: Option<Seq<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pitch_seq: Option<PitchSeq>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        inst_seq: Option<Seq<InstrumentRef>>,
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
    /// Reverse the time ordering of events within a pattern.
    ///
    /// Events at row 0 move to row `len_rows - 1`, etc.
    Reverse {
        /// Length of the pattern in rows (must be >= 1).
        len_rows: u16,
        body: Box<PatternExpr>,
    },
    /// Mirror (retrograde) pattern in time.
    ///
    /// Currently equivalent to `reverse`; future versions may support pitch inversion.
    Mirror {
        /// Length of the pattern in rows (must be >= 1).
        len_rows: u16,
        /// Axis to mirror along (currently only `time` is supported).
        #[serde(default)]
        axis: MirrorAxis,
        body: Box<PatternExpr>,
    },
    /// Interleave events from multiple parts based on row position.
    ///
    /// Each part handles rows at its index offset within each stride-sized block.
    Interleave {
        /// Stride for interleaving (must be >= 1).
        stride: u16,
        /// Parts to interleave (max 16).
        parts: Vec<InterleavePart>,
    },
    /// Remap events to different channels.
    RemapChannel {
        /// Source channel to remap from.
        from: u8,
        /// Destination channel to remap to.
        to: u8,
        body: Box<PatternExpr>,
    },
    /// Filter events based on criteria.
    Filter {
        /// Filter criteria (at least one must be specified).
        criteria: FilterCriteria,
        body: Box<PatternExpr>,
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
    BeatList {
        beats: Vec<BeatPos>,
    },
    BeatRange {
        start: BeatPos,
        step: BeatDelta,
        count: u32,
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

/// Scale type for quantize_pitch transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantizeScale {
    Major,
    Minor,
    HarmonicMinor,
    MelodicMinor,
    PentatonicMajor,
    PentatonicMinor,
    Chromatic,
}

impl QuantizeScale {
    /// Returns the semitone intervals for this scale (relative to root).
    pub fn intervals(&self) -> &'static [u8] {
        match self {
            QuantizeScale::Major => &[0, 2, 4, 5, 7, 9, 11],
            QuantizeScale::Minor => &[0, 2, 3, 5, 7, 8, 10],
            QuantizeScale::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            QuantizeScale::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            QuantizeScale::PentatonicMajor => &[0, 2, 4, 7, 9],
            QuantizeScale::PentatonicMinor => &[0, 3, 5, 7, 10],
            QuantizeScale::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        }
    }
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
    /// Per-cell volume variation for humanization.
    ///
    /// Derives a deterministic volume value from (seed, salt, pattern_name, row, channel).
    /// Volume is clamped to 0-64.
    HumanizeVol {
        /// Minimum volume (0-64).
        min_vol: u8,
        /// Maximum volume (0-64).
        max_vol: u8,
        /// Salt for deterministic randomization.
        seed_salt: String,
    },
    /// Offbeat timing offset (swing feel).
    ///
    /// Applies note delay to offbeat positions (row % stride != 0).
    /// The delay is expressed as permille of a row (0-1000).
    Swing {
        /// Delay amount in permille of a row (0-1000).
        amount_permille: u16,
        /// Stride for determining offbeat positions.
        stride: u32,
        /// Salt for deterministic randomization.
        seed_salt: String,
    },
    /// Melodic inversion around a pivot note.
    ///
    /// Reflects each note's semitone distance from pivot.
    /// Example: If pivot is C4 and note is E4 (+4), result is G#3 (-4).
    InvertPitch {
        /// Pivot note for inversion (e.g., "C4").
        pivot: String,
    },
    /// Snap notes to a scale.
    ///
    /// Notes not in the scale are snapped to the nearest scale degree.
    /// Ties snap down (toward lower pitch).
    QuantizePitch {
        /// Scale to quantize to.
        scale: QuantizeScale,
        /// Root note of the scale (e.g., "C").
        root: String,
    },
    /// Add retriggering (ratchet) effect to notes.
    ///
    /// Adds a note-retrigger effect (E9x in XM).
    Ratchet {
        /// Retrigger divisions (1-16).
        divisions: u8,
        /// Salt for deterministic selection.
        seed_salt: String,
    },
    /// Apply arpeggio effect to notes.
    ///
    /// Adds arpeggio effect (0xy in XM/IT).
    Arpeggiate {
        /// Semitones up (x nibble, 0-15).
        semitones_up: u8,
        /// Semitones down (y nibble, 0-15).
        semitones_down: u8,
    },
}

/// Cell template emitted by Pattern IR (row comes from TimeExpr).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CellTemplate {
    /// Channel number (0-indexed).
    pub channel: ChannelRef,
    /// Note name (string or MIDI number); omitted means "use instrument base note".
    #[serde(
        default,
        deserialize_with = "deserialize_note_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub note: Option<String>,
    /// Instrument index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inst: Option<InstrumentRef>,
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
