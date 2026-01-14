//! Compose (Pattern IR) expansion for tracker music.

use std::collections::HashMap;

use speccade_spec::recipe::music::{
    MusicTrackerSongComposeV1Params, MusicTrackerSongV1Params, PatternNote, TrackerPattern,
};

mod error;
mod expander;
mod expander_context;
mod expander_eval;
mod expander_time;
mod harmony;
mod merge;
mod seq;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API.
pub use error::ExpandError;

use expander::Expander;

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
