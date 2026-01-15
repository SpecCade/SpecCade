//! Expander struct and context resolution methods.

use std::collections::HashMap;

use speccade_spec::recipe::music::{
    BeatDelta, BeatPos, ChannelRef, InstrumentRef, MusicTrackerSongComposeV1Params, PatternExpr,
    TimeBase,
};

use super::error::ExpandError;
use super::harmony::{
    build_chord_context, build_key_context, chord_tone_note_name, scale_degree_note_name, ChordAt,
    KeyContext,
};

pub(super) const MAX_RECURSION_DEPTH: usize = 64;
pub(super) const MAX_CELLS_PER_PATTERN: usize = 50_000;
pub(super) const MAX_TIME_LIST_LEN: usize = 50_000;
pub(super) const MAX_PATTERN_STRING_LEN: usize = 100_000;

/// Pattern expander that evaluates compose expressions.
pub(super) struct Expander<'a> {
    pub(super) params: &'a MusicTrackerSongComposeV1Params,
    pub(super) pattern_name: &'a str,
    pub(super) defs: &'a HashMap<String, PatternExpr>,
    pub(super) ref_stack: Vec<String>,
    pub(super) seed: u32,
    pub(super) pattern_rows: u16,
    pub(super) timebase: Option<TimeBase>,
}

impl<'a> Expander<'a> {
    pub fn new(
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

    pub(super) fn resolve_channel(
        &self,
        channel: &ChannelRef,
        row: i32,
    ) -> Result<u8, ExpandError> {
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

    pub(super) fn resolve_instrument(&self, inst: &InstrumentRef) -> Result<u8, ExpandError> {
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

    pub(super) fn timebase(&self) -> Result<&TimeBase, ExpandError> {
        self.timebase
            .as_ref()
            .ok_or_else(|| ExpandError::InvalidTime {
                pattern: self.pattern_name.to_string(),
                message: "beat-based time expressions require a timebase".to_string(),
            })
    }

    pub(super) fn beat_pos_to_row(&self, pos: &BeatPos) -> Result<i64, ExpandError> {
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

    pub(super) fn beat_delta_to_rows(&self, delta: &BeatDelta) -> Result<i64, ExpandError> {
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

    pub(super) fn harmony(&self) -> Result<&speccade_spec::recipe::music::Harmony, ExpandError> {
        self.params
            .harmony
            .as_ref()
            .ok_or_else(|| ExpandError::InvalidExpr {
                pattern: self.pattern_name.to_string(),
                message: "pitch_seq requires a harmony block in params".to_string(),
            })
    }

    pub(super) fn key_context(&self) -> Result<KeyContext, ExpandError> {
        let harmony = self.harmony()?;
        build_key_context(harmony, self.pattern_name)
    }

    pub(super) fn chord_context(&self) -> Result<Vec<ChordAt>, ExpandError> {
        let harmony = self.harmony()?;
        build_chord_context(harmony, |pos| self.beat_pos_to_row(pos), self.pattern_name)
    }

    pub(super) fn select_chord<'c>(
        &self,
        chords: &'c [ChordAt],
        row: i32,
    ) -> Result<&'c super::harmony::ParsedChord, ExpandError> {
        super::harmony::select_chord(chords, row, self.pattern_name)
    }

    pub(super) fn scale_degree_note_name(
        &self,
        key: &KeyContext,
        value: &str,
        octave: i32,
        allow_accidentals: bool,
    ) -> Result<String, ExpandError> {
        scale_degree_note_name(key, value, octave, allow_accidentals, self.pattern_name)
    }

    pub(super) fn chord_tone_note_name(
        &self,
        chord: &super::harmony::ParsedChord,
        value: &str,
        octave: i32,
    ) -> Result<String, ExpandError> {
        chord_tone_note_name(chord, value, octave, self.pattern_name)
    }

    pub(super) fn pattern_rows(&self) -> u16 {
        self.pattern_rows
    }
}
