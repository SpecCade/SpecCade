//! Compose (Pattern IR) expansion for tracker music.

use std::collections::{BTreeMap, HashMap};

use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use thiserror::Error;

use speccade_spec::recipe::audio::parse_note_name;
use speccade_spec::recipe::music::{
    BeatDelta, BeatPos, CellTemplate, ChannelRef, ChordSpec, ComposePattern, Harmony, HarmonyScale,
    InstrumentRef, MergePolicy, MusicTrackerSongComposeV1Params, MusicTrackerSongV1Params,
    PatternExpr, PatternNote, PitchSeq, PitchSeqKind, Seq, SeqMode, TimeBase, TimeExpr,
    TrackerPattern, TransformOp,
};
use speccade_spec::BackendError;

const MAX_RECURSION_DEPTH: usize = 64;
const MAX_CELLS_PER_PATTERN: usize = 50_000;
const MAX_TIME_LIST_LEN: usize = 50_000;
const MAX_PATTERN_STRING_LEN: usize = 100_000;

type CellKey = (i32, u8);
type CellMap = BTreeMap<CellKey, Cell>;

#[derive(Debug, Clone, PartialEq)]
struct Cell {
    note: Option<String>,
    inst: Option<u8>,
    vol: Option<u8>,
    effect: Option<u8>,
    param: Option<u8>,
    effect_name: Option<String>,
    effect_xy: Option<[u8; 2]>,
}

impl Cell {
    fn from_template(template: &CellTemplate, inst: Option<u8>) -> Self {
        Self {
            note: template.note.clone(),
            inst,
            vol: template.vol,
            effect: template.effect,
            param: template.param,
            effect_name: template.effect_name.clone(),
            effect_xy: template.effect_xy,
        }
    }

    fn from_pattern_note(note: &PatternNote) -> Self {
        Self {
            note: Some(note.note.clone()),
            inst: Some(note.inst),
            vol: note.vol,
            effect: note.effect,
            param: note.param,
            effect_name: note.effect_name.clone(),
            effect_xy: note.effect_xy,
        }
    }
}

/// Errors that can occur during compose expansion.
#[derive(Debug, Error)]
pub enum ExpandError {
    #[error("unknown ref '{name}' in pattern '{pattern}'")]
    UnknownRef { pattern: String, name: String },
    #[error("ref cycle detected: {cycle}")]
    RefCycle { cycle: String },
    #[error("recursion depth exceeded (max {max}) in pattern '{pattern}'")]
    RecursionLimit { pattern: String, max: usize },
    #[error("invalid time expression in pattern '{pattern}': {message}")]
    InvalidTime { pattern: String, message: String },
    #[error("invalid pattern expression in pattern '{pattern}': {message}")]
    InvalidExpr { pattern: String, message: String },
    #[error("unknown channel alias '{alias}' in pattern '{pattern}'")]
    UnknownChannelAlias { pattern: String, alias: String },
    #[error("unknown instrument alias '{alias}' in pattern '{pattern}'")]
    UnknownInstrumentAlias { pattern: String, alias: String },
    #[error(
        "merge conflict at row {row}, channel {channel} on field '{field}' in pattern '{pattern}'"
    )]
    MergeConflict {
        pattern: String,
        row: i32,
        channel: u8,
        field: &'static str,
    },
    #[error("cell out of bounds (row {row}, channel {channel}) in pattern '{pattern}'")]
    CellOutOfBounds {
        pattern: String,
        row: i32,
        channel: u8,
    },
    #[error("instrument index {inst} out of range (len {len}) in pattern '{pattern}'")]
    InvalidInstrument {
        pattern: String,
        inst: u8,
        len: usize,
    },
    #[error("missing instrument for cell at row {row}, channel {channel} in pattern '{pattern}'")]
    MissingInstrument {
        pattern: String,
        row: i32,
        channel: u8,
    },
    #[error("expanded cell limit exceeded in pattern '{pattern}' (max {max})")]
    CellLimit { pattern: String, max: usize },
    #[error("time list limit exceeded in pattern '{pattern}' (max {max})")]
    TimeListLimit { pattern: String, max: usize },
    #[error("pattern string too long in pattern '{pattern}' (max {max})")]
    PatternStringLimit { pattern: String, max: usize },
}

impl BackendError for ExpandError {
    fn code(&self) -> &'static str {
        match self {
            ExpandError::UnknownRef { .. } => "MUSIC_COMPOSE_001",
            ExpandError::RefCycle { .. } => "MUSIC_COMPOSE_002",
            ExpandError::RecursionLimit { .. } => "MUSIC_COMPOSE_003",
            ExpandError::InvalidTime { .. } => "MUSIC_COMPOSE_004",
            ExpandError::InvalidExpr { .. } => "MUSIC_COMPOSE_005",
            ExpandError::UnknownChannelAlias { .. } => "MUSIC_COMPOSE_013",
            ExpandError::UnknownInstrumentAlias { .. } => "MUSIC_COMPOSE_014",
            ExpandError::MergeConflict { .. } => "MUSIC_COMPOSE_006",
            ExpandError::CellOutOfBounds { .. } => "MUSIC_COMPOSE_007",
            ExpandError::InvalidInstrument { .. } => "MUSIC_COMPOSE_008",
            ExpandError::MissingInstrument { .. } => "MUSIC_COMPOSE_009",
            ExpandError::CellLimit { .. } => "MUSIC_COMPOSE_010",
            ExpandError::TimeListLimit { .. } => "MUSIC_COMPOSE_011",
            ExpandError::PatternStringLimit { .. } => "MUSIC_COMPOSE_012",
        }
    }

    fn category(&self) -> &'static str {
        "music"
    }
}

#[derive(Debug, Clone)]
struct KeyContext {
    root_pc: u8,
    _scale: HarmonyScale,
    scale_intervals: [i32; 7],
}

#[derive(Debug, Clone)]
struct ParsedChord {
    root_pc: u8,
    intervals: Vec<u8>,
    _bass_pc: Option<u8>,
}

#[derive(Debug, Clone)]
struct ChordAt {
    at_row: i64,
    idx: usize,
    chord: ParsedChord,
}

struct Expander<'a> {
    params: &'a MusicTrackerSongComposeV1Params,
    pattern_name: &'a str,
    defs: &'a HashMap<String, PatternExpr>,
    ref_stack: Vec<String>,
    seed: u32,
    pattern_rows: u16,
    timebase: Option<TimeBase>,
}

impl<'a> Expander<'a> {
    fn new(
        params: &'a MusicTrackerSongComposeV1Params,
        pattern_name: &'a str,
        defs: &'a HashMap<String, PatternExpr>,
        seed: u32,
        pattern_rows: u16,
        timebase: Option<TimeBase>,
    ) -> Self {
        Self {
            params,
            pattern_name,
            defs,
            ref_stack: Vec::new(),
            seed,
            pattern_rows,
            timebase,
        }
    }

    fn resolve_channel(&self, channel: &ChannelRef, row: i32) -> Result<u8, ExpandError> {
        let resolved = match channel {
            ChannelRef::Index(idx) => *idx,
            ChannelRef::Name(alias) => *self.params.channel_ids.get(alias).ok_or_else(|| {
                ExpandError::UnknownChannelAlias {
                    pattern: self.pattern_name.to_string(),
                    alias: alias.clone(),
                }
            })?,
        };

        if resolved >= self.params.channels {
            return Err(ExpandError::CellOutOfBounds {
                pattern: self.pattern_name.to_string(),
                row,
                channel: resolved,
            });
        }

        Ok(resolved)
    }

    fn resolve_instrument(&self, inst: &InstrumentRef) -> Result<u8, ExpandError> {
        let resolved = match inst {
            InstrumentRef::Index(idx) => *idx,
            InstrumentRef::Name(alias) => {
                *self.params.instrument_ids.get(alias).ok_or_else(|| {
                    ExpandError::UnknownInstrumentAlias {
                        pattern: self.pattern_name.to_string(),
                        alias: alias.clone(),
                    }
                })?
            }
        };

        if resolved as usize >= self.params.instruments.len() {
            return Err(ExpandError::InvalidInstrument {
                pattern: self.pattern_name.to_string(),
                inst: resolved,
                len: self.params.instruments.len(),
            });
        }

        Ok(resolved)
    }

    fn timebase(&self) -> Result<&TimeBase, ExpandError> {
        self.timebase
            .as_ref()
            .ok_or_else(|| ExpandError::InvalidTime {
                pattern: self.pattern_name.to_string(),
                message: "beat-based time expressions require a timebase".to_string(),
            })
    }

    fn beat_pos_to_row(&self, pos: &BeatPos) -> Result<i64, ExpandError> {
        let timebase = self.timebase()?;
        if timebase.beats_per_bar == 0 || timebase.rows_per_beat == 0 {
            return Err(ExpandError::InvalidTime {
                pattern: self.pattern_name.to_string(),
                message: "timebase beats_per_bar and rows_per_beat must be > 0".to_string(),
            });
        }
        if pos.beat >= timebase.beats_per_bar {
            return Err(ExpandError::InvalidTime {
                pattern: self.pattern_name.to_string(),
                message: format!(
                    "beat position beat {} out of range (beats_per_bar {})",
                    pos.beat, timebase.beats_per_bar
                ),
            });
        }
        if pos.sub >= timebase.rows_per_beat {
            return Err(ExpandError::InvalidTime {
                pattern: self.pattern_name.to_string(),
                message: format!(
                    "beat position sub {} out of range (rows_per_beat {})",
                    pos.sub, timebase.rows_per_beat
                ),
            });
        }

        let total_beats = (pos.bar as u64)
            .checked_mul(timebase.beats_per_bar as u64)
            .and_then(|v| v.checked_add(pos.beat as u64))
            .ok_or_else(|| ExpandError::InvalidTime {
                pattern: self.pattern_name.to_string(),
                message: "beat position overflow".to_string(),
            })?;
        let row = total_beats
            .checked_mul(timebase.rows_per_beat as u64)
            .and_then(|v| v.checked_add(pos.sub as u64))
            .ok_or_else(|| ExpandError::InvalidTime {
                pattern: self.pattern_name.to_string(),
                message: "beat position overflow".to_string(),
            })?;

        let row_i64 = i64::try_from(row).map_err(|_| ExpandError::InvalidTime {
            pattern: self.pattern_name.to_string(),
            message: "beat position overflow".to_string(),
        })?;
        if row_i64 < 0 || row_i64 >= self.pattern_rows as i64 {
            return Err(ExpandError::InvalidTime {
                pattern: self.pattern_name.to_string(),
                message: format!(
                    "beat position maps to row {} outside pattern rows {}",
                    row_i64, self.pattern_rows
                ),
            });
        }

        Ok(row_i64)
    }

    fn beat_delta_to_rows(&self, delta: &BeatDelta) -> Result<i64, ExpandError> {
        let timebase = self.timebase()?;
        if timebase.rows_per_beat == 0 {
            return Err(ExpandError::InvalidTime {
                pattern: self.pattern_name.to_string(),
                message: "timebase rows_per_beat must be > 0".to_string(),
            });
        }

        let beats_component = (delta.beats as i64)
            .checked_mul(timebase.rows_per_beat as i64)
            .ok_or_else(|| ExpandError::InvalidTime {
                pattern: self.pattern_name.to_string(),
                message: "beat delta overflow".to_string(),
            })?;
        beats_component
            .checked_add(delta.sub as i64)
            .ok_or_else(|| ExpandError::InvalidTime {
                pattern: self.pattern_name.to_string(),
                message: "beat delta overflow".to_string(),
            })
    }

    fn harmony(&self) -> Result<&Harmony, ExpandError> {
        self.params
            .harmony
            .as_ref()
            .ok_or_else(|| ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: "pitch_seq requires a harmony block in params".to_string(),
            })
    }

    fn parse_pitch_class(&self, name: &str) -> Result<u8, ExpandError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: "pitch class cannot be empty".to_string(),
            });
        }

        let bytes = trimmed.as_bytes();
        let note_letter = bytes[0] as char;
        let base_semitone: i32 = match note_letter.to_ascii_uppercase() {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => -1,
        };

        if base_semitone >= 0 {
            let mut idx = 1usize;
            let mut accidental: i32 = 0;
            if idx < bytes.len() {
                match bytes[idx] as char {
                    '#' | 's' => {
                        accidental = 1;
                        idx += 1;
                    }
                    'b' => {
                        accidental = -1;
                        idx += 1;
                    }
                    _ => {}
                }
            }
            if idx == bytes.len() {
                let pc = (base_semitone + accidental).rem_euclid(12) as u8;
                return Ok(pc);
            }
        }

        let midi = parse_note_name(trimmed).or_else(|| parse_note_name(&trimmed.replace('-', "")));
        let Some(midi) = midi else {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: format!("invalid pitch class '{}'", trimmed),
            });
        };

        Ok(midi % 12)
    }

    fn key_context(&self) -> Result<KeyContext, ExpandError> {
        let harmony = self.harmony()?;
        let root_pc = self.parse_pitch_class(&harmony.key.root)?;
        let scale_intervals = match harmony.key.scale {
            HarmonyScale::Major => [0, 2, 4, 5, 7, 9, 11],
            HarmonyScale::Minor => [0, 2, 3, 5, 7, 8, 10],
        };
        Ok(KeyContext {
            root_pc,
            _scale: harmony.key.scale,
            scale_intervals,
        })
    }

    fn chord_context(&self) -> Result<Vec<ChordAt>, ExpandError> {
        let harmony = self.harmony()?;
        if harmony.chords.is_empty() {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: "pitch_seq chord_tone requires harmony.chords".to_string(),
            });
        }

        let mut chords = Vec::with_capacity(harmony.chords.len());
        for (idx, entry) in harmony.chords.iter().enumerate() {
            let at_row = self.beat_pos_to_row(&entry.at)?;
            let chord = self.parse_chord_spec(&entry.chord)?;
            chords.push(ChordAt { at_row, idx, chord });
        }
        chords.sort_by(|a, b| a.at_row.cmp(&b.at_row).then(a.idx.cmp(&b.idx)));
        Ok(chords)
    }

    fn select_chord<'c>(
        &self,
        chords: &'c [ChordAt],
        row: i32,
    ) -> Result<&'c ParsedChord, ExpandError> {
        if chords.is_empty() {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: "no chords available".to_string(),
            });
        }

        let row_i64 = row as i64;
        let mut selected: Option<&ChordAt> = None;
        for chord in chords {
            if chord.at_row <= row_i64 {
                selected = Some(chord);
            } else {
                break;
            }
        }

        selected
            .map(|c| &c.chord)
            .ok_or_else(|| ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: format!("no chord defined at or before row {}", row),
            })
    }

    fn parse_chord_spec(&self, spec: &ChordSpec) -> Result<ParsedChord, ExpandError> {
        match spec {
            ChordSpec::Symbol(s) => self.parse_chord_symbol(&s.symbol),
            ChordSpec::Intervals(i) => {
                let root_pc = self.parse_pitch_class(&i.root)?;
                if !i.intervals.contains(&0) {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "chord interval form must include 0".to_string(),
                    });
                }
                let mut intervals = i.intervals.clone();
                intervals.sort_unstable();
                intervals.dedup();
                let bass_pc = i
                    .bass
                    .as_ref()
                    .map(|b| self.parse_pitch_class(b))
                    .transpose()?;
                Ok(ParsedChord {
                    root_pc,
                    intervals,
                    _bass_pc: bass_pc,
                })
            }
        }
    }

    fn parse_chord_symbol(&self, symbol: &str) -> Result<ParsedChord, ExpandError> {
        let symbol = symbol.trim();
        if symbol.is_empty() {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: "chord symbol cannot be empty".to_string(),
            });
        }

        let (main, bass) = match symbol.split_once('/') {
            Some((a, b)) => {
                if b.contains('/') {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: format!("invalid chord symbol '{}'", symbol),
                    });
                }
                (a.trim(), Some(b.trim()))
            }
            None => (symbol, None),
        };

        let bass_pc = bass.map(|b| self.parse_pitch_class(b)).transpose()?;

        let bytes = main.as_bytes();
        if bytes.is_empty() {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: format!("invalid chord symbol '{}'", symbol),
            });
        }
        let mut root_len = 1usize;
        if root_len < bytes.len() {
            match bytes[root_len] as char {
                '#' | 'b' | 's' => root_len += 1,
                _ => {}
            }
        }
        let root_str = &main[..root_len];
        let root_pc = self.parse_pitch_class(root_str)?;

        let rest_lower = main[root_len..].trim().to_ascii_lowercase();
        let mut rest = rest_lower.as_str();

        let mut intervals: Vec<u8> = vec![0, 4, 7];
        let mut base_third: Option<u8> = Some(4);
        let mut base_fifth: Option<u8> = Some(7);

        // Special combined forms (imply triad + seventh).
        if let Some(stripped) = rest.strip_prefix("m7b5") {
            intervals = vec![0, 3, 6, 10];
            base_third = Some(3);
            base_fifth = Some(6);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("ø7") {
            intervals = vec![0, 3, 6, 10];
            base_third = Some(3);
            base_fifth = Some(6);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("dim7") {
            intervals = vec![0, 3, 6, 9];
            base_third = Some(3);
            base_fifth = Some(6);
            rest = stripped;
        } else {
            // Quality (triad).
            if let Some(stripped) = rest.strip_prefix("sus2") {
                intervals = vec![0, 2, 7];
                base_third = Some(2);
                base_fifth = Some(7);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("sus4") {
                intervals = vec![0, 5, 7];
                base_third = Some(5);
                base_fifth = Some(7);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("sus") {
                intervals = vec![0, 5, 7];
                base_third = Some(5);
                base_fifth = Some(7);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("dim") {
                intervals = vec![0, 3, 6];
                base_third = Some(3);
                base_fifth = Some(6);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("aug") {
                intervals = vec![0, 4, 8];
                base_third = Some(4);
                base_fifth = Some(8);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("+") {
                intervals = vec![0, 4, 8];
                base_third = Some(4);
                base_fifth = Some(8);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("5") {
                intervals = vec![0, 7];
                base_third = None;
                base_fifth = Some(7);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("min") {
                intervals = vec![0, 3, 7];
                base_third = Some(3);
                base_fifth = Some(7);
                rest = stripped;
            } else if rest.starts_with('m') && !rest.starts_with("maj") {
                intervals = vec![0, 3, 7];
                base_third = Some(3);
                base_fifth = Some(7);
                rest = &rest[1..];
            }

            // Sevenths.
            if let Some(stripped) = rest.strip_prefix("maj7") {
                intervals.push(11);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("7") {
                intervals.push(10);
                rest = stripped;
            }
        }

        let ensure_seventh = |intervals: &mut Vec<u8>| {
            if !intervals.iter().any(|v| matches!(*v, 9..=11)) {
                intervals.push(10);
            }
        };
        let ensure_ninth = |intervals: &mut Vec<u8>| {
            if !intervals.iter().any(|v| matches!(*v, 13..=15)) {
                intervals.push(14);
            }
        };
        let ensure_eleven = |intervals: &mut Vec<u8>| {
            if !intervals.iter().any(|v| matches!(*v, 17 | 18)) {
                intervals.push(17);
            }
        };

        // Extensions / adds / alterations / omissions (prefix scan).
        while !rest.is_empty() {
            if let Some(stripped) = rest.strip_prefix("add13") {
                intervals.push(21);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("add11") {
                intervals.push(17);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("add9") {
                intervals.push(14);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("no3") {
                if let Some(third) = base_third {
                    intervals.retain(|v| *v != third);
                }
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("no5") {
                if let Some(fifth) = base_fifth {
                    intervals.retain(|v| *v != fifth);
                }
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("#11") {
                intervals.retain(|v| *v != 17 && *v != 18);
                intervals.push(18);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("b13") {
                intervals.retain(|v| *v != 20 && *v != 21);
                intervals.push(20);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("b9") {
                intervals.retain(|v| !matches!(*v, 13..=15));
                intervals.push(13);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("#9") {
                intervals.retain(|v| !matches!(*v, 13..=15));
                intervals.push(15);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("b5") {
                intervals.retain(|v| !matches!(*v, 6..=8));
                intervals.push(6);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("#5") {
                intervals.retain(|v| !matches!(*v, 6..=8));
                intervals.push(8);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("maj7") {
                intervals.push(11);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("7") {
                intervals.push(10);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("6") {
                intervals.push(9);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("13") {
                ensure_seventh(&mut intervals);
                ensure_ninth(&mut intervals);
                ensure_eleven(&mut intervals);
                intervals.push(21);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("11") {
                ensure_seventh(&mut intervals);
                ensure_ninth(&mut intervals);
                intervals.push(17);
                rest = stripped;
            } else if let Some(stripped) = rest.strip_prefix("9") {
                ensure_seventh(&mut intervals);
                intervals.push(14);
                rest = stripped;
            } else {
                return Err(ExpandError::InvalidExpr {
                    pattern: self.pattern_name.to_string(),
                    message: format!("unsupported chord token '{}' in symbol '{}'", rest, symbol),
                });
            }
        }

        intervals.sort_unstable();
        intervals.dedup();
        if !intervals.contains(&0) {
            intervals.insert(0, 0);
        }

        Ok(ParsedChord {
            root_pc,
            intervals,
            _bass_pc: bass_pc,
        })
    }

    fn parse_degree_value(&self, value: &str) -> Result<(i32, i32), ExpandError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: "degree value cannot be empty".to_string(),
            });
        }

        let mut accidental = 0i32;
        let mut rest = trimmed;
        while let Some(stripped) = rest.strip_prefix('b') {
            accidental -= 1;
            rest = stripped;
        }
        while let Some(stripped) = rest.strip_prefix('#') {
            accidental += 1;
            rest = stripped;
        }

        let degree = rest.parse::<i32>().map_err(|_| ExpandError::InvalidExpr {
            pattern: self.pattern_name.to_string(),
            message: format!("invalid degree value '{}'", trimmed),
        })?;

        Ok((degree, accidental))
    }

    fn midi_from_pitch(&self, root_pc: u8, octave: i32, semitones: i32) -> Result<u8, ExpandError> {
        let midi = (octave + 1)
            .checked_mul(12)
            .and_then(|v| v.checked_add(root_pc as i32))
            .and_then(|v| v.checked_add(semitones))
            .ok_or_else(|| ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: "pitch -> MIDI overflow".to_string(),
            })?;
        if !(0..=127).contains(&midi) {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: format!("pitch produced out-of-range MIDI note {}", midi),
            });
        }
        Ok(midi as u8)
    }

    fn scale_degree_note_name(
        &self,
        key: &KeyContext,
        value: &str,
        octave: i32,
        allow_accidentals: bool,
    ) -> Result<String, ExpandError> {
        let (degree, accidental) = self.parse_degree_value(value)?;
        if !(1..=7).contains(&degree) {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: format!("scale_degree value '{}' must be 1..=7", value),
            });
        }
        if accidental != 0 && !allow_accidentals {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: format!(
                    "scale_degree value '{}' uses accidentals (set allow_accidentals=true to permit)",
                    value
                ),
            });
        }

        let base = key.scale_intervals[(degree - 1) as usize];
        let semitones = base + accidental;
        if semitones < 0 {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: format!("scale_degree value '{}' underflows", value),
            });
        }

        let midi = self.midi_from_pitch(key.root_pc, octave, semitones)?;
        Ok(midi_to_note_name(midi))
    }

    fn chord_tone_note_name(
        &self,
        chord: &ParsedChord,
        value: &str,
        octave: i32,
    ) -> Result<String, ExpandError> {
        let (degree, accidental) = self.parse_degree_value(value)?;
        if accidental != 0 {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: format!("chord_tone value '{}' does not support accidentals", value),
            });
        }

        let candidates: &'static [u8] = match degree {
            1 => &[0],
            3 => &[4, 3],
            5 => &[7, 6, 8],
            7 => &[10, 11, 9],
            9 => &[14, 13, 15],
            11 => &[17, 18],
            13 => &[21, 20],
            _ => {
                return Err(ExpandError::InvalidExpr {
                    pattern: self.pattern_name.to_string(),
                    message: format!(
                        "chord_tone value '{}' must be one of 1,3,5,7,9,11,13",
                        value
                    ),
                })
            }
        };

        let Some(interval) = candidates
            .iter()
            .copied()
            .find(|cand| chord.intervals.contains(cand))
        else {
            return Err(ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: format!(
                    "chord_tone value '{}' not present in chord intervals {:?}",
                    value, chord.intervals
                ),
            });
        };

        let midi = self.midi_from_pitch(chord.root_pc, octave, interval as i32)?;
        Ok(midi_to_note_name(midi))
    }

    fn expand_pattern(&mut self, pattern: &ComposePattern) -> Result<CellMap, ExpandError> {
        let mut map = self.eval_expr(&pattern.program, 0)?;

        // Merge hand-authored data/notes on top (last_wins).
        let manual = self.manual_cells(pattern)?;
        self.merge_maps(&mut map, manual, MergePolicy::LastWins)?;

        if map.len() > MAX_CELLS_PER_PATTERN {
            return Err(ExpandError::CellLimit {
                pattern: self.pattern_name.to_string(),
                max: MAX_CELLS_PER_PATTERN,
            });
        }

        Ok(map)
    }

    fn manual_cells(&self, pattern: &ComposePattern) -> Result<CellMap, ExpandError> {
        let mut map = CellMap::new();

        if let Some(ref data) = pattern.data {
            for note in data {
                let channel = note.channel.unwrap_or(0);
                let key = (note.row as i32, channel);
                let cell = Cell::from_pattern_note(note);
                insert_cell_merge(
                    &mut map,
                    key,
                    cell,
                    MergePolicy::MergeFields,
                    self.pattern_name,
                )?;
            }
        }

        if let Some(ref notes) = pattern.notes {
            for (channel_str, items) in notes {
                let channel = channel_str
                    .parse::<u8>()
                    .map_err(|_| ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: format!("invalid channel key '{}'", channel_str),
                    })?;
                for note in items {
                    let key = (note.row as i32, channel);
                    let cell = Cell::from_pattern_note(note);
                    insert_cell_merge(
                        &mut map,
                        key,
                        cell,
                        MergePolicy::MergeFields,
                        self.pattern_name,
                    )?;
                }
            }
        }

        Ok(map)
    }

    fn eval_expr(&mut self, expr: &PatternExpr, depth: usize) -> Result<CellMap, ExpandError> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(ExpandError::RecursionLimit {
                pattern: self.pattern_name.to_string(),
                max: MAX_RECURSION_DEPTH,
            });
        }

        match expr {
            PatternExpr::Stack { merge, parts } => {
                let mut map = CellMap::new();
                for part in parts {
                    let part_map = self.eval_expr(part, depth + 1)?;
                    self.merge_maps(&mut map, part_map, *merge)?;
                    if map.len() > MAX_CELLS_PER_PATTERN {
                        return Err(ExpandError::CellLimit {
                            pattern: self.pattern_name.to_string(),
                            max: MAX_CELLS_PER_PATTERN,
                        });
                    }
                }
                Ok(map)
            }
            PatternExpr::Concat { parts } => {
                let mut map = CellMap::new();
                let mut offset = 0i32;
                let mut total_len = 0i32;
                for part in parts {
                    if part.len_rows == 0 {
                        return Err(ExpandError::InvalidExpr {
                            pattern: self.pattern_name.to_string(),
                            message: "concat parts must have len_rows > 0".to_string(),
                        });
                    }
                    total_len += part.len_rows as i32;
                }
                if total_len > 0 && total_len > self.pattern_rows() as i32 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: format!(
                            "concat length {} exceeds pattern rows {}",
                            total_len,
                            self.pattern_rows()
                        ),
                    });
                }
                for part in parts {
                    let part_len = part.len_rows as i32;
                    let mut part_map = self.eval_expr(&part.body, depth + 1)?;
                    // Ensure part stays within its declared window.
                    for (key, _) in part_map.iter() {
                        let row = key.0;
                        if row < 0 || row >= part_len {
                            return Err(ExpandError::CellOutOfBounds {
                                pattern: self.pattern_name.to_string(),
                                row,
                                channel: key.1,
                            });
                        }
                    }
                    shift_map(&mut part_map, offset);
                    self.merge_maps(&mut map, part_map, MergePolicy::Error)?;
                    offset += part_len;
                }
                Ok(map)
            }
            PatternExpr::Repeat {
                times,
                len_rows,
                body,
            } => {
                if *len_rows == 0 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "repeat len_rows must be > 0".to_string(),
                    });
                }
                let total_len = (*times as i32) * (*len_rows as i32);
                if total_len > 0 && total_len > self.pattern_rows() as i32 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: format!(
                            "repeat length {} exceeds pattern rows {}",
                            total_len,
                            self.pattern_rows()
                        ),
                    });
                }
                let base = self.eval_expr(body, depth + 1)?;
                for (key, _) in base.iter() {
                    let row = key.0;
                    if row < 0 || row >= *len_rows as i32 {
                        return Err(ExpandError::CellOutOfBounds {
                            pattern: self.pattern_name.to_string(),
                            row,
                            channel: key.1,
                        });
                    }
                }
                let mut map = CellMap::new();
                for i in 0..*times {
                    let mut part = base.clone();
                    shift_map(&mut part, (i as i32) * (*len_rows as i32));
                    self.merge_maps(&mut map, part, MergePolicy::Error)?;
                }
                Ok(map)
            }
            PatternExpr::Shift { rows, body } => {
                let mut map = self.eval_expr(body, depth + 1)?;
                shift_map(&mut map, *rows);
                Ok(map)
            }
            PatternExpr::Slice { start, len, body } => {
                if *len < 0 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "slice len must be >= 0".to_string(),
                    });
                }
                let map = self.eval_expr(body, depth + 1)?;
                let end = start.saturating_add(*len);
                let filtered = map
                    .into_iter()
                    .filter(|((row, _), _)| *row >= *start && *row < end)
                    .collect();
                Ok(filtered)
            }
            PatternExpr::Ref { name } => {
                if self.ref_stack.contains(name) {
                    let mut cycle = self.ref_stack.join(" -> ");
                    if !cycle.is_empty() {
                        cycle.push_str(" -> ");
                    }
                    cycle.push_str(name);
                    return Err(ExpandError::RefCycle { cycle });
                }
                let expr = self.defs.get(name).ok_or_else(|| ExpandError::UnknownRef {
                    pattern: self.pattern_name.to_string(),
                    name: name.clone(),
                })?;
                self.ref_stack.push(name.clone());
                let result = self.eval_expr(expr, depth + 1);
                self.ref_stack.pop();
                result
            }
            PatternExpr::Emit { at, cell } => {
                let positions = self.eval_time_expr(at)?;
                let mut map = CellMap::new();
                for row in positions {
                    let channel = self.resolve_channel(&cell.channel, row)?;
                    let inst = cell
                        .inst
                        .as_ref()
                        .map(|inst_ref| self.resolve_instrument(inst_ref))
                        .transpose()?;
                    let key = (row, channel);
                    let cell = Cell::from_template(cell, inst);
                    insert_cell_merge(
                        &mut map,
                        key,
                        cell,
                        MergePolicy::MergeFields,
                        self.pattern_name,
                    )?;
                }
                Ok(map)
            }
            PatternExpr::EmitSeq {
                at,
                cell,
                note_seq,
                pitch_seq,
                inst_seq,
                vol_seq,
                effect_seq,
                param_seq,
                effect_name_seq,
                effect_xy_seq,
            } => {
                let positions = self.eval_time_expr(at)?;
                let mut map = CellMap::new();
                let count = positions.len();

                if note_seq.is_some() && pitch_seq.is_some() {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "emit_seq must specify at most one of note_seq or pitch_seq"
                            .to_string(),
                    });
                }

                let note_seq = SeqAccessor::new(note_seq.as_ref(), count, self.pattern_name)?;
                let pitch_seq =
                    PitchSeqAccessor::new(pitch_seq.as_ref(), count, self.pattern_name)?;
                let inst_seq = SeqAccessor::new(inst_seq.as_ref(), count, self.pattern_name)?;
                let vol_seq = SeqAccessor::new(vol_seq.as_ref(), count, self.pattern_name)?;
                let effect_seq = SeqAccessor::new(effect_seq.as_ref(), count, self.pattern_name)?;
                let param_seq = SeqAccessor::new(param_seq.as_ref(), count, self.pattern_name)?;
                let effect_name_seq =
                    SeqAccessor::new(effect_name_seq.as_ref(), count, self.pattern_name)?;
                let effect_xy_seq =
                    SeqAccessor::new(effect_xy_seq.as_ref(), count, self.pattern_name)?;

                let key_ctx = pitch_seq
                    .seq()
                    .filter(|seq| seq.kind == PitchSeqKind::ScaleDegree)
                    .map(|_| self.key_context())
                    .transpose()?;
                let chord_ctx = pitch_seq
                    .seq()
                    .filter(|seq| seq.kind == PitchSeqKind::ChordTone)
                    .map(|_| self.chord_context())
                    .transpose()?;

                let base_inst = cell
                    .inst
                    .as_ref()
                    .map(|inst_ref| self.resolve_instrument(inst_ref))
                    .transpose()?;
                for (idx, row) in positions.into_iter().enumerate() {
                    let channel = self.resolve_channel(&cell.channel, row)?;
                    let mut cell = Cell::from_template(cell, base_inst);
                    if let Some(value) = note_seq.value(idx)? {
                        cell.note = Some(value);
                    }
                    if let Some(seq) = pitch_seq.seq() {
                        if let Some(value) = pitch_seq.value(idx)? {
                            let note = match seq.kind {
                                PitchSeqKind::ScaleDegree => {
                                    let key_ctx = key_ctx.as_ref().ok_or_else(|| {
                                        ExpandError::InvalidExpr {
                                            pattern: self.pattern_name.to_string(),
                                            message: "pitch_seq scale_degree requires harmony.key"
                                                .to_string(),
                                        }
                                    })?;
                                    self.scale_degree_note_name(
                                        key_ctx,
                                        &value,
                                        seq.octave,
                                        seq.allow_accidentals,
                                    )?
                                }
                                PitchSeqKind::ChordTone => {
                                    let chord_ctx = chord_ctx.as_ref().ok_or_else(|| {
                                        ExpandError::InvalidExpr {
                                            pattern: self.pattern_name.to_string(),
                                            message: "pitch_seq chord_tone requires harmony.chords"
                                                .to_string(),
                                        }
                                    })?;
                                    let chord = self.select_chord(chord_ctx, row)?;
                                    self.chord_tone_note_name(chord, &value, seq.octave)?
                                }
                            };
                            cell.note = Some(note);
                        }
                    }
                    if let Some(value) = inst_seq.value(idx)? {
                        cell.inst = Some(self.resolve_instrument(&value)?);
                    }
                    if let Some(value) = vol_seq.value(idx)? {
                        cell.vol = Some(value);
                    }
                    if let Some(value) = effect_seq.value(idx)? {
                        cell.effect = Some(value);
                    }
                    if let Some(value) = param_seq.value(idx)? {
                        cell.param = Some(value);
                    }
                    if let Some(value) = effect_name_seq.value(idx)? {
                        cell.effect_name = Some(value);
                    }
                    if let Some(value) = effect_xy_seq.value(idx)? {
                        cell.effect_xy = Some(value);
                    }

                    let key = (row, channel);
                    insert_cell_merge(
                        &mut map,
                        key,
                        cell,
                        MergePolicy::MergeFields,
                        self.pattern_name,
                    )?;
                }
                Ok(map)
            }
            PatternExpr::Transform { ops, body } => {
                let mut map = self.eval_expr(body, depth + 1)?;
                for (_key, cell) in map.iter_mut() {
                    apply_transforms(cell, ops, self.pattern_name)?;
                }
                Ok(map)
            }
            PatternExpr::Prob {
                p_permille,
                seed_salt,
                body,
            } => {
                if *p_permille > 1000 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "prob p_permille must be <= 1000".to_string(),
                    });
                }
                let map = self.eval_expr(body, depth + 1)?;
                if *p_permille == 0 {
                    return Ok(map);
                }
                if *p_permille >= 1000 {
                    return Ok(CellMap::new());
                }
                let mut rng = rng_for(self.seed, self.pattern_name, seed_salt);
                let filtered = map
                    .into_iter()
                    .filter(|_| rng.gen_range(0u16..1000u16) >= *p_permille)
                    .collect();
                Ok(filtered)
            }
            PatternExpr::Choose { seed_salt, choices } => {
                if choices.is_empty() {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "choose requires at least one choice".to_string(),
                    });
                }
                let total_weight: u32 = choices.iter().map(|c| c.weight).sum();
                if total_weight == 0 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "choose weights must sum to > 0".to_string(),
                    });
                }
                let mut rng = rng_for(self.seed, self.pattern_name, seed_salt);
                let mut roll = rng.gen_range(0u32..total_weight);
                for choice in choices {
                    if roll < choice.weight {
                        return self.eval_expr(&choice.body, depth + 1);
                    }
                    roll = roll.saturating_sub(choice.weight);
                }
                // Fallback (shouldn't happen).
                self.eval_expr(&choices[0].body, depth + 1)
            }
        }
    }

    fn eval_time_expr(&self, expr: &TimeExpr) -> Result<Vec<i32>, ExpandError> {
        match expr {
            TimeExpr::Range { start, step, count } => {
                if *step == 0 {
                    return Err(ExpandError::InvalidTime {
                        pattern: self.pattern_name.to_string(),
                        message: "range step must be non-zero".to_string(),
                    });
                }
                let count = *count as usize;
                if count > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }
                let mut rows = Vec::with_capacity(count);
                for i in 0..count {
                    let row = *start as i64 + (*step as i64) * (i as i64);
                    rows.push(row as i32);
                }
                Ok(rows)
            }
            TimeExpr::List { rows } => {
                if rows.len() > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }
                Ok(rows.clone())
            }
            TimeExpr::BeatList { beats } => {
                if beats.len() > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }
                let mut rows = Vec::with_capacity(beats.len());
                for pos in beats {
                    rows.push(self.beat_pos_to_row(pos)? as i32);
                }
                Ok(rows)
            }
            TimeExpr::BeatRange { start, step, count } => {
                let count = *count as usize;
                if count > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }

                let start_row = self.beat_pos_to_row(start)?;
                let delta_rows = self.beat_delta_to_rows(step)?;
                if delta_rows == 0 {
                    return Err(ExpandError::InvalidTime {
                        pattern: self.pattern_name.to_string(),
                        message: "beat_range step must be non-zero".to_string(),
                    });
                }

                let mut rows = Vec::with_capacity(count);
                for i in 0..count {
                    let offset = delta_rows.checked_mul(i as i64).ok_or_else(|| {
                        ExpandError::InvalidTime {
                            pattern: self.pattern_name.to_string(),
                            message: "beat_range overflow".to_string(),
                        }
                    })?;
                    let row =
                        start_row
                            .checked_add(offset)
                            .ok_or_else(|| ExpandError::InvalidTime {
                                pattern: self.pattern_name.to_string(),
                                message: "beat_range overflow".to_string(),
                            })?;
                    if row < 0 || row >= self.pattern_rows as i64 {
                        return Err(ExpandError::InvalidTime {
                            pattern: self.pattern_name.to_string(),
                            message: format!(
                                "beat_range produced row {} outside pattern rows {}",
                                row, self.pattern_rows
                            ),
                        });
                    }
                    rows.push(row as i32);
                }
                Ok(rows)
            }
            TimeExpr::Euclid {
                pulses,
                steps,
                rotate,
                stride,
                offset,
            } => {
                let steps_usize = *steps as usize;
                let pulses_usize = *pulses as usize;
                if *steps == 0 {
                    return Err(ExpandError::InvalidTime {
                        pattern: self.pattern_name.to_string(),
                        message: "euclid steps must be > 0".to_string(),
                    });
                }
                if pulses_usize > steps_usize {
                    return Err(ExpandError::InvalidTime {
                        pattern: self.pattern_name.to_string(),
                        message: "euclid pulses must be <= steps".to_string(),
                    });
                }
                if steps_usize > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }
                let pattern = bjorklund(steps_usize, pulses_usize);
                let rot = modulo(*rotate, *steps as i32);
                let mut rows = Vec::with_capacity(pulses_usize);
                for (idx, hit) in pattern.iter().enumerate() {
                    if *hit {
                        let rotated = (idx as i32 + rot) % (*steps as i32);
                        let row = rotated * *stride + *offset;
                        rows.push(row);
                    }
                }
                Ok(rows)
            }
            TimeExpr::Pattern {
                pattern,
                stride,
                offset,
            } => {
                if pattern.len() > MAX_PATTERN_STRING_LEN {
                    return Err(ExpandError::PatternStringLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_PATTERN_STRING_LEN,
                    });
                }
                let mut rows = Vec::new();
                let mut idx = 0i32;
                for ch in pattern.chars() {
                    match ch {
                        'x' | 'X' => {
                            let row = idx * *stride + *offset;
                            rows.push(row);
                            idx += 1;
                        }
                        '.' => {
                            idx += 1;
                        }
                        c if c.is_whitespace() => {}
                        _ => {
                            return Err(ExpandError::InvalidTime {
                                pattern: self.pattern_name.to_string(),
                                message: format!("invalid pattern character '{}'", ch),
                            })
                        }
                    }
                }
                if rows.len() > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }
                Ok(rows)
            }
        }
    }

    fn merge_maps(
        &self,
        base: &mut CellMap,
        incoming: CellMap,
        policy: MergePolicy,
    ) -> Result<(), ExpandError> {
        for (key, cell) in incoming {
            insert_cell_merge(base, key, cell, policy, self.pattern_name)?;
        }
        Ok(())
    }

    fn pattern_rows(&self) -> u16 {
        self.pattern_rows
    }
}

fn shift_map(map: &mut CellMap, offset: i32) {
    if offset == 0 {
        return;
    }
    let moved = std::mem::take(map);
    let shifted: CellMap = moved
        .into_iter()
        .map(|((row, channel), cell)| ((row + offset, channel), cell))
        .collect();
    *map = shifted;
}

fn insert_cell_merge(
    map: &mut CellMap,
    key: CellKey,
    cell: Cell,
    policy: MergePolicy,
    pattern_name: &str,
) -> Result<(), ExpandError> {
    if let Some(existing) = map.get_mut(&key) {
        if policy == MergePolicy::Error {
            return Err(ExpandError::MergeConflict {
                pattern: pattern_name.to_string(),
                row: key.0,
                channel: key.1,
                field: "cell",
            });
        }
        merge_cell_fields(existing, &cell, policy, key, pattern_name)?;
    } else {
        map.insert(key, cell);
    }
    Ok(())
}

fn merge_cell_fields(
    existing: &mut Cell,
    incoming: &Cell,
    policy: MergePolicy,
    key: CellKey,
    pattern_name: &str,
) -> Result<(), ExpandError> {
    merge_field(
        &mut existing.note,
        &incoming.note,
        policy,
        key,
        pattern_name,
        "note",
    )?;
    merge_field(
        &mut existing.inst,
        &incoming.inst,
        policy,
        key,
        pattern_name,
        "inst",
    )?;
    merge_field(
        &mut existing.vol,
        &incoming.vol,
        policy,
        key,
        pattern_name,
        "vol",
    )?;
    merge_field(
        &mut existing.effect,
        &incoming.effect,
        policy,
        key,
        pattern_name,
        "effect",
    )?;
    merge_field(
        &mut existing.param,
        &incoming.param,
        policy,
        key,
        pattern_name,
        "param",
    )?;
    merge_field(
        &mut existing.effect_name,
        &incoming.effect_name,
        policy,
        key,
        pattern_name,
        "effect_name",
    )?;
    merge_field(
        &mut existing.effect_xy,
        &incoming.effect_xy,
        policy,
        key,
        pattern_name,
        "effect_xy",
    )?;
    Ok(())
}

fn merge_field<T: Clone + PartialEq>(
    target: &mut Option<T>,
    incoming: &Option<T>,
    policy: MergePolicy,
    key: CellKey,
    pattern_name: &str,
    field: &'static str,
) -> Result<(), ExpandError> {
    let Some(value) = incoming else {
        return Ok(());
    };

    match target {
        None => {
            *target = Some(value.clone());
        }
        Some(existing) => {
            if existing != value {
                match policy {
                    MergePolicy::LastWins => {
                        *existing = value.clone();
                    }
                    MergePolicy::MergeFields | MergePolicy::Error => {
                        return Err(ExpandError::MergeConflict {
                            pattern: pattern_name.to_string(),
                            row: key.0,
                            channel: key.1,
                            field,
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

fn apply_transforms(
    cell: &mut Cell,
    ops: &[TransformOp],
    pattern_name: &str,
) -> Result<(), ExpandError> {
    for op in ops {
        match op {
            TransformOp::TransposeSemitones { semitones } => {
                if let Some(ref note) = cell.note {
                    if let Some(transposed) = transpose_note(note, *semitones, pattern_name)? {
                        cell.note = Some(transposed);
                    }
                }
            }
            TransformOp::VolMul { mul, div } => {
                if *div == 0 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: pattern_name.to_string(),
                        message: "vol_mul div must be > 0".to_string(),
                    });
                }
                if let Some(vol) = cell.vol {
                    let scaled = (vol as u32).saturating_mul(*mul) / *div;
                    cell.vol = Some(std::cmp::min(scaled, 64) as u8);
                }
            }
            TransformOp::Set {
                inst,
                vol,
                effect,
                param,
                effect_name,
                effect_xy,
            } => {
                if cell.inst.is_none() {
                    cell.inst = *inst;
                }
                if cell.vol.is_none() {
                    cell.vol = *vol;
                }
                if cell.effect.is_none() {
                    cell.effect = *effect;
                }
                if cell.param.is_none() {
                    cell.param = *param;
                }
                if cell.effect_name.is_none() {
                    cell.effect_name = effect_name.clone();
                }
                if cell.effect_xy.is_none() {
                    cell.effect_xy = *effect_xy;
                }
            }
        }
    }
    Ok(())
}

fn transpose_note(
    note: &str,
    semitones: i32,
    pattern_name: &str,
) -> Result<Option<String>, ExpandError> {
    let trimmed = note.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let upper = trimmed.to_uppercase();
    if matches!(
        upper.as_str(),
        "---" | "..." | "OFF" | "===" | "^^^" | "CUT" | "FADE" | "~~~"
    ) {
        return Ok(None);
    }
    let midi = parse_note_name(trimmed)
        .or_else(|| parse_note_name(&trimmed.replace('-', "")))
        .map(|v| v as i32);
    let Some(midi) = midi else {
        return Ok(None);
    };
    let transposed = midi + semitones;
    if !(0..=127).contains(&transposed) {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: format!("transpose produced out-of-range MIDI note {}", transposed),
        });
    }
    Ok(Some(midi_to_note_name(transposed as u8)))
}

fn midi_to_note_name(midi: u8) -> String {
    const NOTES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (midi / 12) as i32 - 1;
    let note_idx = (midi % 12) as usize;
    format!("{}{}", NOTES[note_idx], octave)
}

fn bjorklund(steps: usize, pulses: usize) -> Vec<bool> {
    if pulses == 0 {
        return vec![false; steps];
    }
    if pulses >= steps {
        return vec![true; steps];
    }

    let mut pattern = Vec::new();
    let mut counts = Vec::new();
    let mut remainders = Vec::new();
    remainders.push(pulses);
    let mut divisor = steps - pulses;
    let mut level = 0usize;

    while remainders[level] > 1 {
        counts.push(divisor / remainders[level]);
        remainders.push(divisor % remainders[level]);
        divisor = remainders[level];
        level += 1;
    }
    counts.push(divisor);

    fn build(level: isize, counts: &[usize], remainders: &[usize], pattern: &mut Vec<bool>) {
        if level == -1 {
            pattern.push(false);
        } else if level == -2 {
            pattern.push(true);
        } else {
            for _ in 0..counts[level as usize] {
                build(level - 1, counts, remainders, pattern);
            }
            if remainders[level as usize] != 0 {
                build(level - 2, counts, remainders, pattern);
            }
        }
    }

    build(level as isize, &counts, &remainders, &mut pattern);
    pattern.truncate(steps);
    pattern
}

fn modulo(value: i32, modulus: i32) -> i32 {
    if modulus == 0 {
        return 0;
    }
    let mut v = value % modulus;
    if v < 0 {
        v += modulus;
    }
    v
}

struct SeqAccessor<'a, T> {
    seq: Option<&'a Seq<T>>,
    len: usize,
    pattern_name: &'a str,
}

impl<'a, T: Clone> SeqAccessor<'a, T> {
    fn new(
        seq: Option<&'a Seq<T>>,
        len: usize,
        pattern_name: &'a str,
    ) -> Result<Self, ExpandError> {
        if let Some(seq) = seq {
            match seq.mode {
                SeqMode::Cycle => {
                    if seq.values.is_empty() {
                        return Err(ExpandError::InvalidExpr {
                            pattern: pattern_name.to_string(),
                            message: "emit_seq cycle values must be non-empty".to_string(),
                        });
                    }
                }
                SeqMode::Once => {
                    if seq.values.len() != len {
                        return Err(ExpandError::InvalidExpr {
                            pattern: pattern_name.to_string(),
                            message: format!(
                                "emit_seq once values length {} != {}",
                                seq.values.len(),
                                len
                            ),
                        });
                    }
                }
            }
        }
        Ok(Self {
            seq,
            len,
            pattern_name,
        })
    }

    fn value(&self, index: usize) -> Result<Option<T>, ExpandError> {
        let Some(seq) = self.seq else {
            return Ok(None);
        };
        match seq.mode {
            SeqMode::Cycle => {
                if seq.values.is_empty() {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "emit_seq cycle values must be non-empty".to_string(),
                    });
                }
                Ok(Some(seq.values[index % seq.values.len()].clone()))
            }
            SeqMode::Once => {
                if seq.values.len() != self.len {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: format!(
                            "emit_seq once values length {} != {}",
                            seq.values.len(),
                            self.len
                        ),
                    });
                }
                Ok(Some(seq.values[index].clone()))
            }
        }
    }
}

struct PitchSeqAccessor<'a> {
    seq: Option<&'a PitchSeq>,
    len: usize,
    pattern_name: &'a str,
}

impl<'a> PitchSeqAccessor<'a> {
    fn new(
        seq: Option<&'a PitchSeq>,
        len: usize,
        pattern_name: &'a str,
    ) -> Result<Self, ExpandError> {
        if let Some(seq) = seq {
            match seq.mode {
                SeqMode::Cycle => {
                    if seq.values.is_empty() {
                        return Err(ExpandError::InvalidExpr {
                            pattern: pattern_name.to_string(),
                            message: "emit_seq pitch_seq cycle values must be non-empty"
                                .to_string(),
                        });
                    }
                }
                SeqMode::Once => {
                    if seq.values.len() != len {
                        return Err(ExpandError::InvalidExpr {
                            pattern: pattern_name.to_string(),
                            message: format!(
                                "emit_seq pitch_seq once values length {} != {}",
                                seq.values.len(),
                                len
                            ),
                        });
                    }
                }
            }
        }

        Ok(Self {
            seq,
            len,
            pattern_name,
        })
    }

    fn seq(&self) -> Option<&'a PitchSeq> {
        self.seq
    }

    fn value(&self, index: usize) -> Result<Option<String>, ExpandError> {
        let Some(seq) = self.seq else {
            return Ok(None);
        };
        match seq.mode {
            SeqMode::Cycle => {
                if seq.values.is_empty() {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "emit_seq pitch_seq cycle values must be non-empty".to_string(),
                    });
                }
                Ok(Some(seq.values[index % seq.values.len()].clone()))
            }
            SeqMode::Once => {
                if seq.values.len() != self.len {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: format!(
                            "emit_seq pitch_seq once values length {} != {}",
                            seq.values.len(),
                            self.len
                        ),
                    });
                }
                Ok(Some(seq.values[index].clone()))
            }
        }
    }
}

fn rng_for(seed: u32, pattern_name: &str, seed_salt: &str) -> Pcg32 {
    let mut input = Vec::with_capacity(8 + pattern_name.len() + seed_salt.len() + 2);
    input.extend_from_slice(&seed.to_le_bytes());
    input.push(0);
    input.extend_from_slice(pattern_name.as_bytes());
    input.push(0);
    input.extend_from_slice(seed_salt.as_bytes());

    let hash = blake3::hash(&input);
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    let derived = u32::from_le_bytes(bytes);
    let seed64 = (derived as u64) | ((derived as u64) << 32);
    Pcg32::seed_from_u64(seed64)
}

/// Expand a compose params object into canonical tracker params.
pub fn expand_compose(
    params: &MusicTrackerSongComposeV1Params,
    seed: u32,
) -> Result<MusicTrackerSongV1Params, ExpandError> {
    let mut expanded_patterns = HashMap::new();
    for (name, pattern) in &params.patterns {
        let timebase = pattern.timebase.clone().or_else(|| params.timebase.clone());
        let pattern_rows = match (pattern.rows, pattern.bars) {
            (Some(rows), None) => rows,
            (None, Some(bars)) => {
                let timebase = timebase.as_ref().ok_or_else(|| ExpandError::InvalidExpr {
                    pattern: name.clone(),
                    message: "pattern uses bars but no timebase is set".to_string(),
                })?;
                if timebase.beats_per_bar == 0 || timebase.rows_per_beat == 0 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: name.clone(),
                        message: "timebase beats_per_bar and rows_per_beat must be > 0".to_string(),
                    });
                }

                let rows_u64 = (bars as u64)
                    .checked_mul(timebase.beats_per_bar as u64)
                    .and_then(|v| v.checked_mul(timebase.rows_per_beat as u64))
                    .ok_or_else(|| ExpandError::InvalidExpr {
                        pattern: name.clone(),
                        message: "pattern rows overflow".to_string(),
                    })?;
                let rows_u16 = u16::try_from(rows_u64).map_err(|_| ExpandError::InvalidExpr {
                    pattern: name.clone(),
                    message: format!("pattern rows {} out of range for u16", rows_u64),
                })?;
                rows_u16
            }
            (Some(_), Some(_)) => {
                return Err(ExpandError::InvalidExpr {
                    pattern: name.clone(),
                    message: "pattern must specify exactly one of rows or bars".to_string(),
                })
            }
            (None, None) => {
                return Err(ExpandError::InvalidExpr {
                    pattern: name.clone(),
                    message: "pattern must specify rows or bars".to_string(),
                })
            }
        };

        let mut expander = Expander::new(params, name, &params.defs, seed, pattern_rows, timebase);
        let map = expander.expand_pattern(pattern)?;

        let mut notes = Vec::with_capacity(map.len());
        for ((row, channel), cell) in map {
            if row < 0 || row >= pattern_rows as i32 {
                return Err(ExpandError::CellOutOfBounds {
                    pattern: name.clone(),
                    row,
                    channel,
                });
            }
            if channel >= params.channels {
                return Err(ExpandError::CellOutOfBounds {
                    pattern: name.clone(),
                    row,
                    channel,
                });
            }
            let inst = match cell.inst {
                Some(inst) => inst,
                None => {
                    return Err(ExpandError::MissingInstrument {
                        pattern: name.clone(),
                        row,
                        channel,
                    });
                }
            };
            if inst as usize >= params.instruments.len() {
                return Err(ExpandError::InvalidInstrument {
                    pattern: name.clone(),
                    inst,
                    len: params.instruments.len(),
                });
            }

            notes.push(PatternNote {
                row: row as u16,
                channel: Some(channel),
                note: cell.note.unwrap_or_default(),
                inst,
                vol: cell.vol,
                effect: cell.effect,
                param: cell.param,
                effect_name: cell.effect_name,
                effect_xy: cell.effect_xy,
            });
        }

        let tracker_pattern = TrackerPattern {
            rows: pattern_rows,
            notes: None,
            data: Some(notes),
        };
        expanded_patterns.insert(name.clone(), tracker_pattern);
    }

    Ok(MusicTrackerSongV1Params {
        name: params.name.clone(),
        title: params.title.clone(),
        format: params.format,
        bpm: params.bpm,
        speed: params.speed,
        channels: params.channels,
        r#loop: params.r#loop,
        restart_position: params.restart_position,
        instruments: params.instruments.clone(),
        patterns: expanded_patterns,
        arrangement: params.arrangement.clone(),
        automation: params.automation.clone(),
        it_options: params.it_options.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use speccade_spec::recipe::music::WeightedChoice;
    use speccade_spec::recipe::music::{TrackerFormat, TrackerInstrument};

    fn base_params() -> MusicTrackerSongComposeV1Params {
        MusicTrackerSongComposeV1Params {
            format: TrackerFormat::Xm,
            bpm: 120,
            speed: 6,
            channels: 4,
            instruments: vec![TrackerInstrument::default()],
            ..Default::default()
        }
    }

    #[test]
    fn time_expr_range_list_pattern() {
        let params = base_params();
        let defs = HashMap::new();
        let expander = Expander::new(&params, "p0", &defs, 1, 64, None);
        let rows = expander
            .eval_time_expr(&TimeExpr::Range {
                start: 0,
                step: 2,
                count: 4,
            })
            .unwrap();
        assert_eq!(rows, vec![0, 2, 4, 6]);

        let rows = expander
            .eval_time_expr(&TimeExpr::List {
                rows: vec![1, 3, 5],
            })
            .unwrap();
        assert_eq!(rows, vec![1, 3, 5]);

        let rows = expander
            .eval_time_expr(&TimeExpr::Pattern {
                pattern: "x.x.".to_string(),
                stride: 1,
                offset: 0,
            })
            .unwrap();
        assert_eq!(rows, vec![0, 2]);

        let rows = expander
            .eval_time_expr(&TimeExpr::Euclid {
                pulses: 3,
                steps: 8,
                rotate: 0,
                stride: 1,
                offset: 0,
            })
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|r| *r >= 0 && *r < 8));
    }

    #[test]
    fn emit_seq_cycle_vs_once() {
        let mut params = base_params();
        params.patterns.insert(
            "p0".to_string(),
            ComposePattern {
                rows: Some(4),
                bars: None,
                timebase: None,
                program: PatternExpr::EmitSeq {
                    at: TimeExpr::Range {
                        start: 0,
                        step: 1,
                        count: 4,
                    },
                    cell: Box::new(CellTemplate {
                        channel: ChannelRef::Index(0),
                        inst: Some(InstrumentRef::Index(0)),
                        ..Default::default()
                    }),
                    note_seq: Some(Seq {
                        mode: SeqMode::Cycle,
                        values: vec!["C4".to_string(), "D4".to_string()],
                    }),
                    pitch_seq: None,
                    inst_seq: None,
                    vol_seq: None,
                    effect_seq: None,
                    param_seq: None,
                    effect_name_seq: None,
                    effect_xy_seq: None,
                },
                data: None,
                notes: None,
            },
        );

        let expanded = expand_compose(&params, 1).unwrap();
        let pattern = expanded.patterns.get("p0").unwrap();
        let notes = pattern.data.as_ref().unwrap();
        assert_eq!(notes[0].note, "C4");
        assert_eq!(notes[1].note, "D4");
        assert_eq!(notes[2].note, "C4");
        assert_eq!(notes[3].note, "D4");

        params.patterns.insert(
            "p1".to_string(),
            ComposePattern {
                rows: Some(2),
                bars: None,
                timebase: None,
                program: PatternExpr::EmitSeq {
                    at: TimeExpr::Range {
                        start: 0,
                        step: 1,
                        count: 2,
                    },
                    cell: Box::new(CellTemplate {
                        channel: ChannelRef::Index(0),
                        inst: Some(InstrumentRef::Index(0)),
                        ..Default::default()
                    }),
                    note_seq: Some(Seq {
                        mode: SeqMode::Once,
                        values: vec!["E4".to_string(), "F4".to_string()],
                    }),
                    pitch_seq: None,
                    inst_seq: None,
                    vol_seq: None,
                    effect_seq: None,
                    param_seq: None,
                    effect_name_seq: None,
                    effect_xy_seq: None,
                },
                data: None,
                notes: None,
            },
        );

        let expanded = expand_compose(&params, 1).unwrap();
        let notes = expanded.patterns.get("p1").unwrap().data.as_ref().unwrap();
        assert_eq!(notes[0].note, "E4");
        assert_eq!(notes[1].note, "F4");
    }

    #[test]
    fn emit_expands_cells() {
        let mut params = base_params();
        params.patterns.insert(
            "p0".to_string(),
            ComposePattern {
                rows: Some(8),
                bars: None,
                timebase: None,
                program: PatternExpr::Emit {
                    at: TimeExpr::Range {
                        start: 0,
                        step: 2,
                        count: 3,
                    },
                    cell: CellTemplate {
                        channel: ChannelRef::Index(1),
                        note: Some("C4".to_string()),
                        inst: Some(InstrumentRef::Index(0)),
                        vol: Some(48),
                        ..Default::default()
                    },
                },
                data: None,
                notes: None,
            },
        );

        let expanded = expand_compose(&params, 1).unwrap();
        let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[0].row, 0);
        assert_eq!(notes[1].row, 2);
        assert_eq!(notes[2].row, 4);
        assert_eq!(notes[0].channel, Some(1));
        assert_eq!(notes[0].note, "C4");
        assert_eq!(notes[0].inst, 0);
        assert_eq!(notes[0].vol, Some(48));
    }

    #[test]
    fn ref_resolution_and_cycles() {
        let mut params = base_params();
        params.defs.insert(
            "beat".to_string(),
            PatternExpr::Emit {
                at: TimeExpr::List { rows: vec![0, 2] },
                cell: CellTemplate {
                    channel: ChannelRef::Index(0),
                    note: Some("C4".to_string()),
                    inst: Some(InstrumentRef::Index(0)),
                    ..Default::default()
                },
            },
        );
        params.patterns.insert(
            "p0".to_string(),
            ComposePattern {
                rows: Some(4),
                bars: None,
                timebase: None,
                program: PatternExpr::Ref {
                    name: "beat".to_string(),
                },
                data: None,
                notes: None,
            },
        );

        let expanded = expand_compose(&params, 1).unwrap();
        let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].row, 0);
        assert_eq!(notes[1].row, 2);

        let mut missing = base_params();
        missing.patterns.insert(
            "p0".to_string(),
            ComposePattern {
                rows: Some(4),
                bars: None,
                timebase: None,
                program: PatternExpr::Ref {
                    name: "nope".to_string(),
                },
                data: None,
                notes: None,
            },
        );
        let err = expand_compose(&missing, 1).unwrap_err();
        assert!(matches!(err, ExpandError::UnknownRef { .. }));

        let mut cycle = base_params();
        cycle.defs.insert(
            "a".to_string(),
            PatternExpr::Ref {
                name: "b".to_string(),
            },
        );
        cycle.defs.insert(
            "b".to_string(),
            PatternExpr::Ref {
                name: "a".to_string(),
            },
        );
        cycle.patterns.insert(
            "p0".to_string(),
            ComposePattern {
                rows: Some(4),
                bars: None,
                timebase: None,
                program: PatternExpr::Ref {
                    name: "a".to_string(),
                },
                data: None,
                notes: None,
            },
        );
        let err = expand_compose(&cycle, 1).unwrap_err();
        assert!(matches!(err, ExpandError::RefCycle { .. }));
    }

    #[test]
    fn choose_is_deterministic() {
        let mut params = base_params();
        params.patterns.insert(
            "p0".to_string(),
            ComposePattern {
                rows: Some(1),
                bars: None,
                timebase: None,
                program: PatternExpr::Choose {
                    seed_salt: "fill".to_string(),
                    choices: vec![
                        WeightedChoice {
                            weight: 1,
                            body: PatternExpr::Emit {
                                at: TimeExpr::List { rows: vec![0] },
                                cell: CellTemplate {
                                    channel: ChannelRef::Index(0),
                                    note: Some("C4".to_string()),
                                    inst: Some(InstrumentRef::Index(0)),
                                    ..Default::default()
                                },
                            },
                        },
                        WeightedChoice {
                            weight: 1,
                            body: PatternExpr::Emit {
                                at: TimeExpr::List { rows: vec![0] },
                                cell: CellTemplate {
                                    channel: ChannelRef::Index(0),
                                    note: Some("D4".to_string()),
                                    inst: Some(InstrumentRef::Index(0)),
                                    ..Default::default()
                                },
                            },
                        },
                    ],
                },
                data: None,
                notes: None,
            },
        );

        let first = expand_compose(&params, 42).unwrap();
        let second = expand_compose(&params, 42).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn seed_salt_changes_rng_stream() {
        let mut rng_a = rng_for(1, "p0", "salt_a");
        let mut rng_b = rng_for(1, "p0", "salt_b");
        let a = rng_a.gen::<u32>();
        let b = rng_b.gen::<u32>();
        assert_ne!(a, b);
    }

    #[test]
    fn merge_policies() {
        let params = base_params();
        let defs = HashMap::new();
        let expander = Expander::new(&params, "p0", &defs, 1, 64, None);
        let mut base = CellMap::new();
        let key = (0, 0);
        insert_cell_merge(
            &mut base,
            key,
            Cell {
                note: Some("C4".to_string()),
                inst: Some(0),
                vol: Some(32),
                effect: None,
                param: None,
                effect_name: None,
                effect_xy: None,
            },
            MergePolicy::MergeFields,
            "p0",
        )
        .unwrap();

        let mut incoming = CellMap::new();
        insert_cell_merge(
            &mut incoming,
            key,
            Cell {
                note: Some("C4".to_string()),
                inst: Some(0),
                vol: Some(48),
                effect: None,
                param: None,
                effect_name: None,
                effect_xy: None,
            },
            MergePolicy::MergeFields,
            "p0",
        )
        .unwrap();

        let err = expander.merge_maps(&mut base, incoming, MergePolicy::MergeFields);
        assert!(err.is_err());

        let mut incoming = CellMap::new();
        insert_cell_merge(
            &mut incoming,
            key,
            Cell {
                note: Some("C4".to_string()),
                inst: Some(0),
                vol: Some(48),
                effect: None,
                param: None,
                effect_name: None,
                effect_xy: None,
            },
            MergePolicy::MergeFields,
            "p0",
        )
        .unwrap();

        expander
            .merge_maps(&mut base, incoming, MergePolicy::LastWins)
            .unwrap();
        let cell = base.get(&key).unwrap();
        assert_eq!(cell.vol, Some(48));
    }

    #[test]
    fn transpose_skips_special_notes() {
        let mut cell = Cell {
            note: Some("---".to_string()),
            inst: Some(0),
            vol: None,
            effect: None,
            param: None,
            effect_name: None,
            effect_xy: None,
        };
        apply_transforms(
            &mut cell,
            &[TransformOp::TransposeSemitones { semitones: 12 }],
            "p0",
        )
        .unwrap();
        assert_eq!(cell.note.as_deref(), Some("---"));
    }

    #[test]
    fn transform_transpose_vol_mul_set() {
        let mut cell = Cell {
            note: Some("C4".to_string()),
            inst: None,
            vol: Some(32),
            effect: None,
            param: None,
            effect_name: None,
            effect_xy: None,
        };
        apply_transforms(
            &mut cell,
            &[
                TransformOp::TransposeSemitones { semitones: 12 },
                TransformOp::VolMul { mul: 3, div: 2 },
                TransformOp::Set {
                    inst: Some(1),
                    vol: Some(10),
                    effect: Some(2),
                    param: Some(3),
                    effect_name: Some("arpeggio".to_string()),
                    effect_xy: Some([1, 2]),
                },
            ],
            "p0",
        )
        .unwrap();

        assert_eq!(cell.note.as_deref(), Some("C5"));
        assert_eq!(cell.vol, Some(48));
        assert_eq!(cell.inst, Some(1));
        assert_eq!(cell.effect, Some(2));
        assert_eq!(cell.param, Some(3));
        assert_eq!(cell.effect_name.as_deref(), Some("arpeggio"));
        assert_eq!(cell.effect_xy, Some([1, 2]));
    }

    #[test]
    fn rng_determinism_prob() {
        let mut params = base_params();
        params.patterns.insert(
            "p0".to_string(),
            ComposePattern {
                rows: Some(8),
                bars: None,
                timebase: None,
                program: PatternExpr::Prob {
                    p_permille: 500,
                    seed_salt: "hats".to_string(),
                    body: Box::new(PatternExpr::Emit {
                        at: TimeExpr::Range {
                            start: 0,
                            step: 1,
                            count: 8,
                        },
                        cell: CellTemplate {
                            channel: ChannelRef::Index(0),
                            inst: Some(InstrumentRef::Index(0)),
                            ..Default::default()
                        },
                    }),
                },
                data: None,
                notes: None,
            },
        );

        let first = expand_compose(&params, 123).unwrap();
        let second = expand_compose(&params, 123).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn aliases_resolve_channel_and_instrument() {
        let mut params = base_params();
        params.channels = 2;
        params.instruments = vec![TrackerInstrument::default(), TrackerInstrument::default()];
        params.channel_ids.insert("kick".to_string(), 1);
        params.instrument_ids.insert("kick".to_string(), 1);

        params.patterns.insert(
            "p0".to_string(),
            ComposePattern {
                rows: Some(4),
                bars: None,
                timebase: None,
                program: PatternExpr::Emit {
                    at: TimeExpr::List { rows: vec![0] },
                    cell: CellTemplate {
                        channel: ChannelRef::Name("kick".to_string()),
                        inst: Some(InstrumentRef::Name("kick".to_string())),
                        note: Some("C4".to_string()),
                        ..Default::default()
                    },
                },
                data: None,
                notes: None,
            },
        );

        let expanded = expand_compose(&params, 1).unwrap();
        let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].channel, Some(1));
        assert_eq!(notes[0].inst, 1);
    }

    #[test]
    fn time_expr_beat_range_and_list() {
        let params = base_params();
        let defs = HashMap::new();
        let expander = Expander::new(
            &params,
            "p0",
            &defs,
            1,
            64,
            Some(TimeBase {
                beats_per_bar: 4,
                rows_per_beat: 4,
            }),
        );

        let rows = expander
            .eval_time_expr(&TimeExpr::BeatRange {
                start: BeatPos {
                    bar: 0,
                    beat: 0,
                    sub: 0,
                },
                step: BeatDelta { beats: 1, sub: 0 },
                count: 4,
            })
            .unwrap();
        assert_eq!(rows, vec![0, 4, 8, 12]);

        let rows = expander
            .eval_time_expr(&TimeExpr::BeatList {
                beats: vec![
                    BeatPos {
                        bar: 0,
                        beat: 0,
                        sub: 0,
                    },
                    BeatPos {
                        bar: 0,
                        beat: 2,
                        sub: 2,
                    },
                ],
            })
            .unwrap();
        assert_eq!(rows, vec![0, 10]);
    }

    #[test]
    fn pattern_bars_expands_to_rows() {
        let mut params = base_params();
        params.timebase = Some(TimeBase {
            beats_per_bar: 4,
            rows_per_beat: 4,
        });
        params.patterns.insert(
            "p0".to_string(),
            ComposePattern {
                rows: None,
                bars: Some(2),
                timebase: None,
                program: PatternExpr::Emit {
                    at: TimeExpr::List { rows: vec![0] },
                    cell: CellTemplate {
                        channel: ChannelRef::Index(0),
                        inst: Some(InstrumentRef::Index(0)),
                        note: Some("C4".to_string()),
                        ..Default::default()
                    },
                },
                data: None,
                notes: None,
            },
        );

        let expanded = expand_compose(&params, 1).unwrap();
        assert_eq!(expanded.patterns.get("p0").unwrap().rows, 32);
    }

    #[test]
    fn chord_symbol_parses_intervals_and_bass() {
        let params = base_params();
        let defs = HashMap::new();
        let expander = Expander::new(&params, "p0", &defs, 1, 64, None);
        let chord = expander.parse_chord_symbol("Cmaj7#11/G").unwrap();
        assert_eq!(chord.root_pc, 0);
        assert_eq!(chord._bass_pc, Some(7));
        assert_eq!(chord.intervals, vec![0, 4, 7, 11, 18]);
    }

    #[test]
    fn pitch_seq_scale_degree_maps_notes() {
        let mut params = base_params();
        params.harmony = Some(Harmony {
            key: speccade_spec::recipe::music::HarmonyKey {
                root: "A".to_string(),
                scale: HarmonyScale::Minor,
            },
            chords: vec![],
        });
        params.patterns.insert(
            "p0".to_string(),
            ComposePattern {
                rows: Some(4),
                bars: None,
                timebase: None,
                program: PatternExpr::EmitSeq {
                    at: TimeExpr::Range {
                        start: 0,
                        step: 1,
                        count: 4,
                    },
                    cell: Box::new(CellTemplate {
                        channel: ChannelRef::Index(0),
                        inst: Some(InstrumentRef::Index(0)),
                        ..Default::default()
                    }),
                    note_seq: None,
                    pitch_seq: Some(PitchSeq {
                        kind: PitchSeqKind::ScaleDegree,
                        mode: SeqMode::Once,
                        values: vec![
                            "1".to_string(),
                            "2".to_string(),
                            "3".to_string(),
                            "5".to_string(),
                        ],
                        octave: 5,
                        allow_accidentals: false,
                    }),
                    inst_seq: None,
                    vol_seq: None,
                    effect_seq: None,
                    param_seq: None,
                    effect_name_seq: None,
                    effect_xy_seq: None,
                },
                data: None,
                notes: None,
            },
        );

        let expanded = expand_compose(&params, 1).unwrap();
        let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
        assert_eq!(notes[0].note, "A5");
        assert_eq!(notes[1].note, "B5");
        assert_eq!(notes[2].note, "C6");
        assert_eq!(notes[3].note, "E6");
    }

    #[test]
    fn pitch_seq_chord_tone_maps_notes() {
        let mut params = base_params();
        params.timebase = Some(TimeBase {
            beats_per_bar: 4,
            rows_per_beat: 4,
        });
        params.harmony = Some(Harmony {
            key: speccade_spec::recipe::music::HarmonyKey {
                root: "A".to_string(),
                scale: HarmonyScale::Minor,
            },
            chords: vec![speccade_spec::recipe::music::HarmonyChordEntry {
                at: BeatPos {
                    bar: 0,
                    beat: 0,
                    sub: 0,
                },
                chord: ChordSpec::Symbol(speccade_spec::recipe::music::ChordSpecSymbol {
                    symbol: "Am7".to_string(),
                }),
            }],
        });

        params.patterns.insert(
            "p0".to_string(),
            ComposePattern {
                rows: None,
                bars: Some(1),
                timebase: None,
                program: PatternExpr::EmitSeq {
                    at: TimeExpr::Range {
                        start: 0,
                        step: 1,
                        count: 4,
                    },
                    cell: Box::new(CellTemplate {
                        channel: ChannelRef::Index(0),
                        inst: Some(InstrumentRef::Index(0)),
                        ..Default::default()
                    }),
                    note_seq: None,
                    pitch_seq: Some(PitchSeq {
                        kind: PitchSeqKind::ChordTone,
                        mode: SeqMode::Once,
                        values: vec![
                            "1".to_string(),
                            "3".to_string(),
                            "5".to_string(),
                            "7".to_string(),
                        ],
                        octave: 2,
                        allow_accidentals: false,
                    }),
                    inst_seq: None,
                    vol_seq: None,
                    effect_seq: None,
                    param_seq: None,
                    effect_name_seq: None,
                    effect_xy_seq: None,
                },
                data: None,
                notes: None,
            },
        );

        let expanded = expand_compose(&params, 1).unwrap();
        let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
        assert_eq!(notes[0].note, "A2");
        assert_eq!(notes[1].note, "C3");
        assert_eq!(notes[2].note, "E3");
        assert_eq!(notes[3].note, "G3");
    }
}
