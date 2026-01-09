//! XM pattern data structures and packing.

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Write};

use crate::note::{note_name_to_xm, xm};

/// A single note event in an XM pattern cell.
#[derive(Debug, Clone, Copy, Default)]
pub struct XmNote {
    /// Note value (0=none, 1-96=C-0..B-7, 97=note-off).
    pub note: u8,
    /// Instrument number (0=none, 1-128=instrument).
    pub instrument: u8,
    /// Volume column (0=none, 0x10-0x50=volume, etc.).
    pub volume: u8,
    /// Effect type.
    pub effect: u8,
    /// Effect parameter.
    pub effect_param: u8,
}

impl XmNote {
    /// Create a new empty note.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a note-off event.
    pub fn note_off() -> Self {
        Self {
            note: xm::NOTE_OFF,
            ..Default::default()
        }
    }

    /// Create a note from a note name string.
    pub fn from_name(name: &str, instrument: u8, volume: Option<u8>) -> Self {
        let note = note_name_to_xm(name);
        Self {
            note,
            instrument,
            volume: volume.map(|v| 0x10 + v.min(64)).unwrap_or(0),
            effect: 0,
            effect_param: 0,
        }
    }

    /// Set the effect on this note.
    pub fn with_effect(mut self, effect: u8, param: u8) -> Self {
        self.effect = effect;
        self.effect_param = param;
        self
    }

    /// Check if this note is completely empty.
    pub fn is_empty(&self) -> bool {
        self.note == 0
            && self.instrument == 0
            && self.volume == 0
            && self.effect == 0
            && self.effect_param == 0
    }
}

/// XM pattern containing rows of note data.
#[derive(Debug, Clone)]
pub struct XmPattern {
    /// Number of rows in this pattern.
    pub num_rows: u16,
    /// Note data: notes[row][channel].
    pub notes: Vec<Vec<XmNote>>,
}

impl XmPattern {
    /// Create an empty pattern with the given dimensions.
    pub fn empty(num_rows: u16, num_channels: u8) -> Self {
        let notes = (0..num_rows)
            .map(|_| (0..num_channels).map(|_| XmNote::empty()).collect())
            .collect();
        Self { num_rows, notes }
    }

    /// Set a note at the given position.
    pub fn set_note(&mut self, row: u16, channel: u8, note: XmNote) {
        if (row as usize) < self.notes.len() {
            if let Some(row_data) = self.notes.get_mut(row as usize) {
                if (channel as usize) < row_data.len() {
                    row_data[channel as usize] = note;
                }
            }
        }
    }

    /// Get a note at the given position.
    pub fn get_note(&self, row: u16, channel: u8) -> Option<&XmNote> {
        self.notes
            .get(row as usize)
            .and_then(|r| r.get(channel as usize))
    }

    /// Pack the pattern data using XM compression.
    pub fn pack(&self, num_channels: u8) -> Vec<u8> {
        let mut output = Vec::new();

        for row in &self.notes {
            for (ch_idx, note) in row.iter().enumerate() {
                if ch_idx >= num_channels as usize {
                    break;
                }

                if note.is_empty() {
                    // Empty note: just write packing byte with no data
                    output.push(0x80);
                    continue;
                }

                // Build packing flags
                let mut flags = 0x80u8;
                if note.note != 0 {
                    flags |= 0x01;
                }
                if note.instrument != 0 {
                    flags |= 0x02;
                }
                if note.volume != 0 {
                    flags |= 0x04;
                }
                if note.effect != 0 {
                    flags |= 0x08;
                }
                if note.effect_param != 0 {
                    flags |= 0x10;
                }

                output.push(flags);

                if note.note != 0 {
                    output.push(note.note);
                }
                if note.instrument != 0 {
                    output.push(note.instrument);
                }
                if note.volume != 0 {
                    output.push(note.volume);
                }
                if note.effect != 0 {
                    output.push(note.effect);
                }
                if note.effect_param != 0 {
                    output.push(note.effect_param);
                }
            }
        }

        output
    }

    /// Write the pattern to a writer (including header).
    pub fn write<W: Write>(&self, writer: &mut W, num_channels: u8) -> io::Result<()> {
        let packed_data = self.pack(num_channels);

        // Pattern header (9 bytes)
        writer.write_u32::<LittleEndian>(9)?; // Header length
        writer.write_u8(0)?; // Packing type (always 0)
        writer.write_u16::<LittleEndian>(self.num_rows)?;
        writer.write_u16::<LittleEndian>(packed_data.len() as u16)?;

        // Pattern data
        writer.write_all(&packed_data)?;

        Ok(())
    }
}

/// XM effect types.
pub mod effects {
    /// Arpeggio effect (0xy).
    pub const ARPEGGIO: u8 = 0x0;
    /// Portamento up (1xx).
    pub const PORTA_UP: u8 = 0x1;
    /// Portamento down (2xx).
    pub const PORTA_DOWN: u8 = 0x2;
    /// Tone portamento (3xx).
    pub const TONE_PORTA: u8 = 0x3;
    /// Vibrato (4xy).
    pub const VIBRATO: u8 = 0x4;
    /// Tone portamento + volume slide (5xy).
    pub const TONE_PORTA_VOL_SLIDE: u8 = 0x5;
    /// Vibrato + volume slide (6xy).
    pub const VIBRATO_VOL_SLIDE: u8 = 0x6;
    /// Tremolo (7xy).
    pub const TREMOLO: u8 = 0x7;
    /// Set panning (8xx).
    pub const SET_PANNING: u8 = 0x8;
    /// Sample offset (9xx).
    pub const SAMPLE_OFFSET: u8 = 0x9;
    /// Volume slide (Axy).
    pub const VOL_SLIDE: u8 = 0xA;
    /// Position jump (Bxx).
    pub const POSITION_JUMP: u8 = 0xB;
    /// Set volume (Cxx).
    pub const SET_VOLUME: u8 = 0xC;
    /// Pattern break (Dxx).
    pub const PATTERN_BREAK: u8 = 0xD;
    /// Extended effects (Exy).
    pub const EXTENDED: u8 = 0xE;
    /// Set speed/tempo (Fxx).
    pub const SET_SPEED_TEMPO: u8 = 0xF;
    /// Set global volume (Gxx).
    pub const GLOBAL_VOL: u8 = 0x10;
    /// Global volume slide (Hxy).
    pub const GLOBAL_VOL_SLIDE: u8 = 0x11;
    /// Key off (Kxx).
    pub const KEY_OFF: u8 = 0x14;
    /// Set envelope position (Lxx).
    pub const SET_ENV_POS: u8 = 0x15;
    /// Panning slide (Pxy).
    pub const PAN_SLIDE: u8 = 0x19;
    /// Multi retrigger note (Rxy).
    pub const RETRIGGER: u8 = 0x1B;
    /// Tremor (Txy).
    pub const TREMOR: u8 = 0x1D;
    /// Extra fine portamento (Xxy).
    pub const EXTRA_FINE_PORTA: u8 = 0x21;
}

/// Convert an effect name to XM effect code.
pub fn effect_name_to_code(name: &str) -> Option<u8> {
    match name.to_lowercase().as_str() {
        "arpeggio" => Some(effects::ARPEGGIO),
        "porta_up" | "portamento_up" => Some(effects::PORTA_UP),
        "porta_down" | "portamento_down" => Some(effects::PORTA_DOWN),
        "tone_porta" | "tone_portamento" => Some(effects::TONE_PORTA),
        "vibrato" => Some(effects::VIBRATO),
        "vol_slide" | "volume_slide" => Some(effects::VOL_SLIDE),
        "set_volume" => Some(effects::SET_VOLUME),
        "pattern_break" => Some(effects::PATTERN_BREAK),
        "set_speed" | "set_tempo" => Some(effects::SET_SPEED_TEMPO),
        "set_panning" | "panning" => Some(effects::SET_PANNING),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation() {
        let note = XmNote::from_name("C4", 1, Some(64));
        assert_eq!(note.note, 49);
        assert_eq!(note.instrument, 1);
        assert_eq!(note.volume, 0x10 + 64);
    }

    #[test]
    fn test_pattern_packing() {
        let mut pattern = XmPattern::empty(4, 2);
        pattern.set_note(0, 0, XmNote::from_name("C4", 1, Some(64)));
        pattern.set_note(2, 1, XmNote::from_name("E4", 1, Some(48)));

        let packed = pattern.pack(2);
        assert!(!packed.is_empty());
    }

    #[test]
    fn test_empty_note_packing() {
        let note = XmNote::empty();
        assert!(note.is_empty());
    }
}
