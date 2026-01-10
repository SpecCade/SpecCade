//! IT pattern data structures and packing.

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Write};

use crate::note::{it, note_name_to_it};

/// A single note event in an IT pattern cell.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItNote {
    /// Note value (0-119 for C-0..B-9, 254=cut, 255=off).
    pub note: u8,
    /// Instrument number (0=none, 1-99=instrument).
    pub instrument: u8,
    /// Volume column (0-64).
    pub volume: u8,
    /// Effect command (A-Z = 1-26).
    pub effect: u8,
    /// Effect parameter.
    pub effect_param: u8,
}

impl ItNote {
    /// Create a new empty note.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a note-off event.
    pub fn note_off() -> Self {
        Self {
            note: it::NOTE_OFF,
            ..Default::default()
        }
    }

    /// Create a note-cut event.
    pub fn note_cut() -> Self {
        Self {
            note: it::NOTE_CUT,
            ..Default::default()
        }
    }

    /// Create a note from a note name string.
    pub fn from_name(name: &str, instrument: u8, volume: u8) -> Self {
        let note = note_name_to_it(name);
        Self {
            note,
            instrument,
            volume: volume.min(64),
            effect: 0,
            effect_param: 0,
        }
    }

    /// Create a note from a MIDI note number.
    pub fn from_midi(midi_note: u8, instrument: u8, volume: u8) -> Self {
        Self {
            note: midi_note.min(119),
            instrument,
            volume: volume.min(64),
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

    /// Check if this note has any data that needs to be written.
    pub fn has_data(&self) -> bool {
        !self.is_empty()
    }
}

/// IT pattern containing rows of note data.
#[derive(Debug, Clone)]
pub struct ItPattern {
    /// Number of rows in this pattern.
    pub num_rows: u16,
    /// Note data: notes[row][channel].
    pub notes: Vec<Vec<ItNote>>,
}

impl ItPattern {
    /// Create an empty pattern with the given dimensions.
    pub fn empty(num_rows: u16, num_channels: u8) -> Self {
        let notes = (0..num_rows)
            .map(|_| (0..num_channels).map(|_| ItNote::empty()).collect())
            .collect();
        Self { num_rows, notes }
    }

    /// Set a note at the given position.
    pub fn set_note(&mut self, row: u16, channel: u8, note: ItNote) {
        if (row as usize) < self.notes.len() {
            if let Some(row_data) = self.notes.get_mut(row as usize) {
                if (channel as usize) < row_data.len() {
                    row_data[channel as usize] = note;
                }
            }
        }
    }

    /// Get a note at the given position.
    pub fn get_note(&self, row: u16, channel: u8) -> Option<&ItNote> {
        self.notes
            .get(row as usize)
            .and_then(|r| r.get(channel as usize))
    }

    /// Pack the pattern data using IT compression.
    pub fn pack(&self, num_channels: u8) -> Vec<u8> {
        let mut output = Vec::new();

        // Previous values for each channel (for compression)
        let mut prev_note = [0u8; 64];
        let mut prev_instrument = [0u8; 64];
        let mut prev_volume = [0u8; 64];
        let mut prev_effect = [0u8; 64];
        let mut prev_effect_param = [0u8; 64];

        for row in &self.notes {
            for (channel, note) in row.iter().enumerate().take(num_channels as usize) {
                // Skip completely empty entries
                if !note.has_data() {
                    continue;
                }

                // Build mask variable
                let mut mask = 0u8;

                // Check what needs to be written
                if note.note != 0 && note.note != prev_note[channel] {
                    mask |= 0x01;
                    prev_note[channel] = note.note;
                } else if note.note != 0 {
                    mask |= 0x10; // Use previous note
                }

                if note.instrument != 0 && note.instrument != prev_instrument[channel] {
                    mask |= 0x02;
                    prev_instrument[channel] = note.instrument;
                } else if note.instrument != 0 {
                    mask |= 0x20; // Use previous instrument
                }

                // Volume - IT uses explicit volume values
                let has_note_data = note.note != 0 || note.instrument != 0;
                if note.volume > 0 && note.volume != prev_volume[channel] {
                    mask |= 0x04;
                    prev_volume[channel] = note.volume;
                } else if note.volume > 0 && note.volume == prev_volume[channel] && has_note_data {
                    mask |= 0x40; // Use previous volume
                }

                if (note.effect != 0 || note.effect_param != 0)
                    && (note.effect != prev_effect[channel]
                        || note.effect_param != prev_effect_param[channel])
                {
                    mask |= 0x08;
                    prev_effect[channel] = note.effect;
                    prev_effect_param[channel] = note.effect_param;
                } else if note.effect != 0 || note.effect_param != 0 {
                    mask |= 0x80; // Use previous effect
                }

                if mask == 0 {
                    continue;
                }

                // Write channel variable (channel + 1, bit 7 set to indicate mask follows)
                output.push((channel as u8 + 1) | 0x80);
                output.push(mask);

                // Write data based on mask
                if mask & 0x01 != 0 {
                    output.push(note.note);
                }
                if mask & 0x02 != 0 {
                    output.push(note.instrument);
                }
                if mask & 0x04 != 0 {
                    output.push(note.volume);
                }
                if mask & 0x08 != 0 {
                    output.push(note.effect);
                    output.push(note.effect_param);
                }
            }

            // End of row marker
            output.push(0);
        }

        output
    }

    /// Write the pattern to a writer (including header).
    pub fn write<W: Write>(&self, writer: &mut W, num_channels: u8) -> io::Result<()> {
        let packed_data = self.pack(num_channels);

        // Pattern header (8 bytes)
        writer.write_u16::<LittleEndian>(packed_data.len() as u16)?;
        writer.write_u16::<LittleEndian>(self.num_rows)?;
        writer.write_all(&[0u8; 4])?; // Reserved

        // Pattern data
        writer.write_all(&packed_data)?;

        Ok(())
    }
}

/// IT effect types (letter-based: A=1, B=2, etc.).
pub mod effects {
    /// Set speed (Axx).
    pub const SET_SPEED: u8 = 1;
    /// Position jump (Bxx).
    pub const POSITION_JUMP: u8 = 2;
    /// Pattern break (Cxx).
    pub const PATTERN_BREAK: u8 = 3;
    /// Volume slide (Dxy).
    pub const VOLUME_SLIDE: u8 = 4;
    /// Portamento down (Exx).
    pub const PORTA_DOWN: u8 = 5;
    /// Portamento up (Fxx).
    pub const PORTA_UP: u8 = 6;
    /// Tone portamento (Gxx).
    pub const TONE_PORTA: u8 = 7;
    /// Vibrato (Hxy).
    pub const VIBRATO: u8 = 8;
    /// Tremor (Ixy).
    pub const TREMOR: u8 = 9;
    /// Arpeggio (Jxy).
    pub const ARPEGGIO: u8 = 10;
    /// Vibrato + volume slide (Kxy).
    pub const VIBRATO_VOL_SLIDE: u8 = 11;
    /// Tone porta + volume slide (Lxy).
    pub const TONE_PORTA_VOL_SLIDE: u8 = 12;
    /// Set channel volume (Mxx).
    pub const SET_CHANNEL_VOL: u8 = 13;
    /// Channel volume slide (Nxy).
    pub const CHANNEL_VOL_SLIDE: u8 = 14;
    /// Sample offset (Oxx).
    pub const SAMPLE_OFFSET: u8 = 15;
    /// Panning slide (Pxy).
    pub const PANNING_SLIDE: u8 = 16;
    /// Retrigger note (Qxy).
    pub const RETRIGGER: u8 = 17;
    /// Tremolo (Rxy).
    pub const TREMOLO: u8 = 18;
    /// Extended effects (Sxy).
    pub const EXTENDED: u8 = 19;
    /// Set tempo (Txx).
    pub const TEMPO: u8 = 20;
    /// Fine vibrato (Uxy).
    pub const FINE_VIBRATO: u8 = 21;
    /// Set global volume (Vxx).
    pub const SET_GLOBAL_VOL: u8 = 22;
    /// Global volume slide (Wxy).
    pub const GLOBAL_VOL_SLIDE: u8 = 23;
    /// Set panning (Xxx).
    pub const SET_PANNING: u8 = 24;
    /// Panbrello (Yxy).
    pub const PANBRELLO: u8 = 25;
    /// MIDI macro (Zxx).
    pub const MIDI_MACRO: u8 = 26;
}

/// Convert an effect name to IT effect code.
pub fn effect_name_to_code(name: &str) -> Option<u8> {
    match name.to_lowercase().as_str() {
        "set_speed" | "speed" => Some(effects::SET_SPEED),
        "position_jump" | "jump" => Some(effects::POSITION_JUMP),
        "pattern_break" | "break" => Some(effects::PATTERN_BREAK),
        "volume_slide" | "vol_slide" => Some(effects::VOLUME_SLIDE),
        "porta_down" | "portamento_down" => Some(effects::PORTA_DOWN),
        "porta_up" | "portamento_up" => Some(effects::PORTA_UP),
        "tone_porta" | "tone_portamento" => Some(effects::TONE_PORTA),
        "vibrato" => Some(effects::VIBRATO),
        "arpeggio" => Some(effects::ARPEGGIO),
        "tempo" | "set_tempo" => Some(effects::TEMPO),
        "set_panning" | "panning" => Some(effects::SET_PANNING),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation() {
        let note = ItNote::from_name("C4", 1, 64);
        assert_eq!(note.note, 48);
        assert_eq!(note.instrument, 1);
        assert_eq!(note.volume, 64);
    }

    #[test]
    fn test_pattern_packing() {
        let mut pattern = ItPattern::empty(4, 2);
        pattern.set_note(0, 0, ItNote::from_name("C4", 1, 64));
        pattern.set_note(2, 1, ItNote::from_name("E4", 1, 48));

        let packed = pattern.pack(2);
        assert!(!packed.is_empty());

        // Should have row end markers
        assert!(packed.iter().filter(|&&b| b == 0).count() >= 4);
    }

    #[test]
    fn test_empty_note() {
        let note = ItNote::empty();
        assert!(note.is_empty());
        assert!(!note.has_data());
    }

    #[test]
    fn test_special_notes() {
        assert_eq!(ItNote::note_off().note, it::NOTE_OFF);
        assert_eq!(ItNote::note_cut().note, it::NOTE_CUT);
    }
}
