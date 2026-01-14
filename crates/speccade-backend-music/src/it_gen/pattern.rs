//! IT pattern conversion from spec format.

use speccade_spec::recipe::music::{TrackerInstrument, TrackerPattern};

use crate::generate::{resolve_pattern_note_name, GenerateError};
use crate::it::{effect_name_to_code as it_effect_name_to_code, ItNote, ItPattern};

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
            continue;
        }

        let note_name = resolve_pattern_note_name(note, instruments, "C5")?;
        let note_name = note_name.as_ref();

        let it_note = if note_name == "OFF" || note_name == "^^^" {
            ItNote::note_off()
        } else if note_name == "===" {
            ItNote::note_cut()
        } else {
            let mut n = ItNote::from_name(
                note_name,
                note.inst + 1, // IT instruments are 1-indexed
                note.vol.unwrap_or(64),
            );

            // Apply effect if present - supports both numeric effect and effect_name
            if let Some(effect_code) = note.effect {
                let param = note.param.unwrap_or(0);
                n = n.with_effect(effect_code, param);
            } else if let Some(ref effect_name) = note.effect_name {
                if let Some(code) = it_effect_name_to_code(effect_name) {
                    let param = if let Some([x, y]) = note.effect_xy {
                        (x << 4) | (y & 0x0F)
                    } else {
                        note.param.unwrap_or(0)
                    };
                    n = n.with_effect(code, param);
                }
            }

            n
        };

        it_pattern.set_note(note.row, channel, it_note);
    }

    Ok(it_pattern)
}
