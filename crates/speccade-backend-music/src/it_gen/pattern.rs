//! IT pattern conversion from spec format.

use speccade_spec::recipe::music::{parse_effect_name, TrackerInstrument, TrackerPattern};

use crate::generate::{resolve_pattern_note_name, GenerateError};
use crate::it::{ItNote, ItPattern};

/// Convert a pattern from spec to IT format.
pub fn convert_pattern_to_it(
    pattern: &TrackerPattern,
    num_channels: u8,
    instruments: &[TrackerInstrument],
) -> Result<ItPattern, GenerateError> {
    let mut it_pattern = ItPattern::empty(pattern.rows, num_channels);

    // Iterate over notes organized by channel
    for (channel, note) in pattern.flat_notes() {
        if channel >= num_channels {
            return Err(GenerateError::InvalidParameter(format!(
                "pattern note channel {} exceeds configured channel count {}",
                channel, num_channels
            )));
        }
        if note.row >= pattern.rows {
            return Err(GenerateError::InvalidParameter(format!(
                "pattern note row {} is out of range for pattern rows {}",
                note.row, pattern.rows
            )));
        }
        if let Some(vol) = note.vol {
            if vol > 64 {
                return Err(GenerateError::InvalidParameter(format!(
                    "pattern note volume {} out of range (0-64) at row {}, channel {}",
                    vol, note.row, channel
                )));
            }
        }

        let note_name = resolve_pattern_note_name(note, instruments, "C5")?;
        let note_name = note_name.as_ref();

        let it_note = if note_name == "OFF" || note_name == "^^^" {
            ItNote::note_off()
        } else if note_name == "===" {
            ItNote::note_cut()
        } else {
            if note.inst as usize >= instruments.len() {
                return Err(GenerateError::InvalidParameter(format!(
                    "pattern references instrument {} but only {} instrument(s) are defined",
                    note.inst,
                    instruments.len()
                )));
            }
            let instrument_column = note.inst.checked_add(1).ok_or_else(|| {
                GenerateError::InvalidParameter(format!(
                    "pattern instrument index {} overflows IT instrument column",
                    note.inst
                ))
            })?;
            let mut n = ItNote::from_name(
                note_name,
                instrument_column, // IT instruments are 1-indexed
                note.vol.unwrap_or(64),
            );

            if note.effect.is_some() && note.effect_name.is_some() {
                return Err(GenerateError::InvalidParameter(format!(
                    "pattern note at row {}, channel {} must set either 'effect' or 'effect_name', not both",
                    note.row, channel
                )));
            }

            if let Some([x, y]) = note.effect_xy {
                if x > 0x0F || y > 0x0F {
                    return Err(GenerateError::InvalidParameter(format!(
                        "pattern note effect_xy [{}, {}] out of range (each nibble must be 0-15) at row {}, channel {}",
                        x, y, note.row, channel
                    )));
                }
            }

            // Apply effect if present.
            if let Some(effect_code) = note.effect {
                let param = if let Some([x, y]) = note.effect_xy {
                    (x << 4) | (y & 0x0F)
                } else {
                    note.param.unwrap_or(0)
                };
                n = n.with_effect(effect_code, param);
            } else if let Some(ref effect_name) = note.effect_name {
                let typed_effect = parse_effect_name(effect_name, note.param, note.effect_xy)
                    .ok_or_else(|| {
                        GenerateError::InvalidParameter(format!(
                            "unknown effect_name '{}' at row {}, channel {}",
                            effect_name, note.row, channel
                        ))
                    })?;

                typed_effect.validate_it().map_err(|e| {
                    GenerateError::InvalidParameter(format!(
                        "invalid IT effect '{}' at row {}, channel {}: {}",
                        effect_name, note.row, channel, e
                    ))
                })?;

                let (code, param) = typed_effect.to_it().ok_or_else(|| {
                    GenerateError::InvalidParameter(format!(
                        "effect '{}' is not supported in IT at row {}, channel {}",
                        effect_name, note.row, channel
                    ))
                })?;
                n = n.with_effect(code, param);
            }

            n
        };

        it_pattern.set_note(note.row, channel, it_note);
    }

    Ok(it_pattern)
}
