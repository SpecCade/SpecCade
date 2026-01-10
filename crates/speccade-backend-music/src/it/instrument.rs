//! IT instrument structures and writing.

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Write};

/// IT instrument magic identifier.
pub const IT_INSTRUMENT_MAGIC: &[u8; 4] = b"IMPI";

/// IT instrument header size.
pub const IT_INSTRUMENT_SIZE: usize = 554;

/// NNA (New Note Action) modes.
pub mod nna {
    /// Cut previous note immediately.
    pub const CUT: u8 = 0;
    /// Continue previous note.
    pub const CONTINUE: u8 = 1;
    /// Note-off previous note.
    pub const OFF: u8 = 2;
    /// Fade out previous note.
    pub const FADE: u8 = 3;
}

/// DCT (Duplicate Check Type) modes.
pub mod dct {
    /// Off.
    pub const OFF: u8 = 0;
    /// Note.
    pub const NOTE: u8 = 1;
    /// Sample.
    pub const SAMPLE: u8 = 2;
    /// Instrument.
    pub const INSTRUMENT: u8 = 3;
}

/// DCA (Duplicate Check Action) modes.
pub mod dca {
    /// Cut.
    pub const CUT: u8 = 0;
    /// Note-off.
    pub const OFF: u8 = 1;
    /// Fade.
    pub const FADE: u8 = 2;
}

/// Envelope flags.
pub mod env_flags {
    /// Envelope enabled.
    pub const ENABLED: u8 = 0x01;
    /// Loop enabled.
    pub const LOOP: u8 = 0x02;
    /// Sustain loop enabled.
    pub const SUSTAIN_LOOP: u8 = 0x04;
    /// Carry envelope (for pitch envelope).
    pub const CARRY: u8 = 0x08;
    /// Filter envelope (for pitch envelope).
    pub const FILTER: u8 = 0x80;
}

/// IT envelope point.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItEnvelopePoint {
    /// Tick position.
    pub tick: u16,
    /// Value (0-64 for volume/panning, -32 to 32 for pitch).
    pub value: i8,
}

/// IT envelope.
#[derive(Debug, Clone, Default)]
pub struct ItEnvelope {
    /// Envelope flags.
    pub flags: u8,
    /// Envelope points (up to 25).
    pub points: Vec<ItEnvelopePoint>,
    /// Loop begin point index.
    pub loop_begin: u8,
    /// Loop end point index.
    pub loop_end: u8,
    /// Sustain begin point index.
    pub sustain_begin: u8,
    /// Sustain end point index.
    pub sustain_end: u8,
}

impl ItEnvelope {
    /// Create a simple ADSR volume envelope.
    pub fn adsr_volume(attack: u16, decay: u16, sustain: u8, release: u16) -> Self {
        let mut points = Vec::new();

        // Attack: 0 -> 64
        points.push(ItEnvelopePoint { tick: 0, value: 0 });
        points.push(ItEnvelopePoint {
            tick: attack,
            value: 64,
        });

        // Decay: 64 -> sustain
        let decay_end = attack + decay;
        points.push(ItEnvelopePoint {
            tick: decay_end,
            value: sustain as i8,
        });

        // Hold sustain
        let sustain_end = decay_end + 100;
        points.push(ItEnvelopePoint {
            tick: sustain_end,
            value: sustain as i8,
        });

        // Release: sustain -> 0
        points.push(ItEnvelopePoint {
            tick: sustain_end + release,
            value: 0,
        });

        Self {
            flags: env_flags::ENABLED | env_flags::SUSTAIN_LOOP,
            points,
            loop_begin: 0,
            loop_end: 0,
            sustain_begin: 2,
            sustain_end: 3,
        }
    }

    /// Write envelope data (82 bytes for vol/pan, 83 bytes for pitch).
    pub fn write<W: Write>(&self, writer: &mut W, is_pitch: bool) -> io::Result<()> {
        // Flags
        writer.write_u8(self.flags)?;

        // Number of points
        let num_points = self.points.len().min(25) as u8;
        writer.write_u8(num_points)?;

        // Loop points
        writer.write_u8(self.loop_begin)?;
        writer.write_u8(self.loop_end)?;
        writer.write_u8(self.sustain_begin)?;
        writer.write_u8(self.sustain_end)?;

        // Envelope points (25 * 3 bytes = 75 bytes)
        for i in 0..25 {
            if i < self.points.len() {
                let point = &self.points[i];
                writer.write_i8(point.value)?;
                writer.write_u16::<LittleEndian>(point.tick)?;
            } else {
                writer.write_all(&[0u8; 3])?;
            }
        }

        // Reserved byte
        writer.write_u8(0)?;

        // Pitch envelope has an extra reserved byte
        if is_pitch {
            writer.write_u8(0)?;
        }

        Ok(())
    }
}

/// IT instrument definition.
#[derive(Debug, Clone)]
pub struct ItInstrument {
    /// Instrument name (26 characters max).
    pub name: String,
    /// DOS filename (12 characters max).
    pub filename: String,
    /// New Note Action.
    pub nna: u8,
    /// Duplicate Check Type.
    pub dct: u8,
    /// Duplicate Check Action.
    pub dca: u8,
    /// Fadeout value (0-1024).
    pub fadeout: u16,
    /// Pitch-pan separation (-32 to 32).
    pub pitch_pan_separation: i8,
    /// Pitch-pan center note.
    pub pitch_pan_center: u8,
    /// Global volume (0-128).
    pub global_volume: u8,
    /// Default pan (0-64, or None for disabled).
    pub default_pan: Option<u8>,
    /// Random volume variation (0-100).
    pub random_volume: u8,
    /// Random pan variation (0-64).
    pub random_pan: u8,
    /// Note-sample mapping table (120 entries of (note, sample)).
    pub note_sample_table: Vec<(u8, u8)>,
    /// Volume envelope.
    pub volume_envelope: ItEnvelope,
    /// Panning envelope.
    pub panning_envelope: ItEnvelope,
    /// Pitch/filter envelope.
    pub pitch_envelope: ItEnvelope,
    /// Initial filter cutoff (0-127, or None for disabled).
    pub filter_cutoff: Option<u8>,
    /// Initial filter resonance (0-127, or None for disabled).
    pub filter_resonance: Option<u8>,
    /// MIDI channel (0 = off).
    pub midi_channel: u8,
    /// MIDI program.
    pub midi_program: u8,
    /// MIDI bank.
    pub midi_bank: u16,
}

impl Default for ItInstrument {
    fn default() -> Self {
        // Default note-sample table: all notes map to sample 1
        let note_sample_table: Vec<(u8, u8)> = (0..120).map(|n| (n, 1)).collect();

        Self {
            name: String::new(),
            filename: String::new(),
            nna: nna::CUT,
            dct: dct::OFF,
            dca: dca::CUT,
            fadeout: 256,
            pitch_pan_separation: 0,
            pitch_pan_center: 60,
            global_volume: 128,
            default_pan: Some(32),
            random_volume: 0,
            random_pan: 0,
            note_sample_table,
            volume_envelope: ItEnvelope::default(),
            panning_envelope: ItEnvelope::default(),
            pitch_envelope: ItEnvelope::default(),
            filter_cutoff: None,
            filter_resonance: None,
            midi_channel: 0,
            midi_program: 0,
            midi_bank: 0,
        }
    }
}

impl ItInstrument {
    /// Create a new instrument with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Set the volume envelope.
    pub fn with_volume_envelope(mut self, envelope: ItEnvelope) -> Self {
        self.volume_envelope = envelope;
        self
    }

    /// Set NNA mode for polyphonic playback.
    pub fn with_nna(mut self, nna: u8, dct: u8, dca: u8) -> Self {
        self.nna = nna;
        self.dct = dct;
        self.dca = dca;
        self
    }

    /// Map all notes to a specific sample.
    pub fn map_all_to_sample(&mut self, sample_index: u8) {
        for entry in &mut self.note_sample_table {
            entry.1 = sample_index;
        }
    }

    /// Write the instrument to a writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // Magic
        writer.write_all(IT_INSTRUMENT_MAGIC)?;

        // DOS filename (12 bytes)
        let filename_bytes = self.filename.as_bytes();
        let mut filename_buf = [0u8; 12];
        let copy_len = filename_bytes.len().min(12);
        filename_buf[..copy_len].copy_from_slice(&filename_bytes[..copy_len]);
        writer.write_all(&filename_buf)?;

        // Reserved byte
        writer.write_u8(0)?;

        // NNA, DCT, DCA
        writer.write_u8(self.nna)?;
        writer.write_u8(self.dct)?;
        writer.write_u8(self.dca)?;

        // Fadeout
        writer.write_u16::<LittleEndian>(self.fadeout)?;

        // Pitch-pan separation and center
        writer.write_i8(self.pitch_pan_separation)?;
        writer.write_u8(self.pitch_pan_center)?;

        // Global volume
        writer.write_u8(self.global_volume)?;

        // Default pan (bit 7 set = use default pan)
        let dfp = match self.default_pan {
            Some(pan) => pan | 0x80,
            None => 32, // Center, disabled
        };
        writer.write_u8(dfp)?;

        // Random volume/pan variation
        writer.write_u8(self.random_volume)?;
        writer.write_u8(self.random_pan)?;

        // Tracker version and number of samples (4 bytes reserved)
        writer.write_all(&[0u8; 4])?;

        // Instrument name (26 bytes)
        let name_bytes = self.name.as_bytes();
        let mut name_buf = [0u8; 26];
        let copy_len = name_bytes.len().min(26);
        name_buf[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        writer.write_all(&name_buf)?;

        // Initial filter cutoff (bit 7 set = use filter)
        let ifc = match self.filter_cutoff {
            Some(cutoff) => cutoff | 0x80,
            None => 0,
        };
        writer.write_u8(ifc)?;

        // Initial filter resonance (bit 7 set = use filter)
        let ifr = match self.filter_resonance {
            Some(res) => res | 0x80,
            None => 0,
        };
        writer.write_u8(ifr)?;

        // MIDI channel, program, bank
        writer.write_u8(self.midi_channel)?;
        writer.write_u8(self.midi_program)?;
        writer.write_u16::<LittleEndian>(self.midi_bank)?;

        // Note-sample table (120 entries * 2 bytes = 240 bytes)
        for &(note, sample) in &self.note_sample_table {
            writer.write_u8(note)?;
            writer.write_u8(sample)?;
        }

        // Volume envelope (82 bytes)
        self.volume_envelope.write(writer, false)?;

        // Panning envelope (82 bytes)
        self.panning_envelope.write(writer, false)?;

        // Pitch envelope (83 bytes)
        self.pitch_envelope.write(writer, true)?;

        // Padding to reach instrument size (if needed)
        // Total so far: 4+12+1+1+1+1+2+1+1+1+1+1+1+4+26+1+1+1+1+2+240+82+82+83 = 551
        // We need 3 more bytes to reach 554
        writer.write_all(&[0u8; 3])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instrument_write() {
        let instrument = ItInstrument::new("Test Instrument");

        let mut buf = Vec::new();
        instrument.write(&mut buf).unwrap();

        // Check magic
        assert_eq!(&buf[0..4], IT_INSTRUMENT_MAGIC);

        // Check size (554 bytes)
        assert_eq!(buf.len(), IT_INSTRUMENT_SIZE);
    }

    #[test]
    fn test_adsr_envelope() {
        let env = ItEnvelope::adsr_volume(10, 20, 32, 30);
        assert_eq!(env.flags & env_flags::ENABLED, env_flags::ENABLED);
        assert!(env.points.len() >= 4);
    }

    #[test]
    fn test_envelope_write() {
        let env = ItEnvelope::default();

        let mut buf = Vec::new();
        env.write(&mut buf, false).unwrap();

        // Volume/panning envelope is 82 bytes
        assert_eq!(buf.len(), 82);

        buf.clear();
        env.write(&mut buf, true).unwrap();

        // Pitch envelope is 83 bytes
        assert_eq!(buf.len(), 83);
    }
}
