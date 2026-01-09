//! XM instrument structures and writing.

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Write};

/// XM instrument header size (for instruments with samples).
pub const XM_INSTRUMENT_HEADER_SIZE: u32 = 263;

/// XM sample header size.
pub const XM_SAMPLE_HEADER_SIZE: u32 = 40;

/// Volume envelope point.
#[derive(Debug, Clone, Copy, Default)]
pub struct XmEnvelopePoint {
    /// Frame number (0-65535).
    pub frame: u16,
    /// Value (0-64 for volume, 0-64 for panning).
    pub value: u16,
}

/// Volume or panning envelope.
#[derive(Debug, Clone, Default)]
pub struct XmEnvelope {
    /// Envelope points (up to 12).
    pub points: Vec<XmEnvelopePoint>,
    /// Sustain point index.
    pub sustain_point: u8,
    /// Loop start point index.
    pub loop_start: u8,
    /// Loop end point index.
    pub loop_end: u8,
    /// Envelope enabled.
    pub enabled: bool,
    /// Sustain enabled.
    pub sustain_enabled: bool,
    /// Loop enabled.
    pub loop_enabled: bool,
}

impl XmEnvelope {
    /// Create a simple ADSR-style volume envelope.
    pub fn adsr(attack_frames: u16, decay_frames: u16, sustain_level: u8, release_frames: u16) -> Self {
        let mut points = Vec::new();

        // Attack: 0 -> 64
        points.push(XmEnvelopePoint { frame: 0, value: 0 });
        points.push(XmEnvelopePoint {
            frame: attack_frames,
            value: 64,
        });

        // Decay: 64 -> sustain
        let decay_end = attack_frames + decay_frames;
        points.push(XmEnvelopePoint {
            frame: decay_end,
            value: sustain_level as u16,
        });

        // Release point (sustain holds here)
        let release_start = decay_end + 100; // Hold sustain for a bit
        points.push(XmEnvelopePoint {
            frame: release_start,
            value: sustain_level as u16,
        });

        // Release: sustain -> 0
        points.push(XmEnvelopePoint {
            frame: release_start + release_frames,
            value: 0,
        });

        Self {
            points,
            sustain_point: 2, // Sustain at decay end
            loop_start: 0,
            loop_end: 0,
            enabled: true,
            sustain_enabled: true,
            loop_enabled: false,
        }
    }

    /// Get the flags byte for this envelope.
    pub fn flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.enabled {
            flags |= 1;
        }
        if self.sustain_enabled {
            flags |= 2;
        }
        if self.loop_enabled {
            flags |= 4;
        }
        flags
    }
}

/// XM instrument with embedded sample.
#[derive(Debug, Clone, Default)]
pub struct XmInstrument {
    /// Instrument name (22 characters max).
    pub name: String,
    /// Volume envelope.
    pub volume_envelope: XmEnvelope,
    /// Panning envelope.
    pub panning_envelope: XmEnvelope,
    /// Vibrato type (0=sine, 1=square, 2=ramp down, 3=ramp up).
    pub vibrato_type: u8,
    /// Vibrato sweep.
    pub vibrato_sweep: u8,
    /// Vibrato depth.
    pub vibrato_depth: u8,
    /// Vibrato rate.
    pub vibrato_rate: u8,
    /// Volume fadeout (0-4095).
    pub volume_fadeout: u16,
    /// Sample data.
    pub sample: XmSample,
}

impl XmInstrument {
    /// Create a new instrument with the given name and sample.
    pub fn new(name: &str, sample: XmSample) -> Self {
        Self {
            name: name.to_string(),
            sample,
            ..Default::default()
        }
    }

    /// Set the volume envelope.
    pub fn with_volume_envelope(mut self, envelope: XmEnvelope) -> Self {
        self.volume_envelope = envelope;
        self
    }

    /// Write the instrument to a writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // Instrument header size
        writer.write_u32::<LittleEndian>(XM_INSTRUMENT_HEADER_SIZE)?;

        // Instrument name (22 bytes)
        let name_bytes = self.name.as_bytes();
        let mut name_buf = [0u8; 22];
        let copy_len = name_bytes.len().min(22);
        name_buf[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        writer.write_all(&name_buf)?;

        // Instrument type (always 0)
        writer.write_u8(0)?;

        // Number of samples (always 1 for our purposes)
        writer.write_u16::<LittleEndian>(1)?;

        // Sample header size
        writer.write_u32::<LittleEndian>(XM_SAMPLE_HEADER_SIZE)?;

        // Note-sample mapping table (96 bytes)
        // Maps each note to sample 0
        writer.write_all(&[0u8; 96])?;

        // Volume envelope (48 bytes: 12 points * 4 bytes)
        for i in 0..12 {
            if i < self.volume_envelope.points.len() {
                let point = &self.volume_envelope.points[i];
                writer.write_u16::<LittleEndian>(point.frame)?;
                writer.write_u16::<LittleEndian>(point.value)?;
            } else {
                writer.write_u32::<LittleEndian>(0)?;
            }
        }

        // Panning envelope (48 bytes: 12 points * 4 bytes)
        for i in 0..12 {
            if i < self.panning_envelope.points.len() {
                let point = &self.panning_envelope.points[i];
                writer.write_u16::<LittleEndian>(point.frame)?;
                writer.write_u16::<LittleEndian>(point.value)?;
            } else {
                writer.write_u32::<LittleEndian>(0)?;
            }
        }

        // Number of volume envelope points
        writer.write_u8(self.volume_envelope.points.len().min(12) as u8)?;

        // Number of panning envelope points
        writer.write_u8(self.panning_envelope.points.len().min(12) as u8)?;

        // Volume sustain point
        writer.write_u8(self.volume_envelope.sustain_point)?;

        // Volume loop start
        writer.write_u8(self.volume_envelope.loop_start)?;

        // Volume loop end
        writer.write_u8(self.volume_envelope.loop_end)?;

        // Panning sustain point
        writer.write_u8(self.panning_envelope.sustain_point)?;

        // Panning loop start
        writer.write_u8(self.panning_envelope.loop_start)?;

        // Panning loop end
        writer.write_u8(self.panning_envelope.loop_end)?;

        // Volume envelope flags
        writer.write_u8(self.volume_envelope.flags())?;

        // Panning envelope flags
        writer.write_u8(self.panning_envelope.flags())?;

        // Vibrato parameters
        writer.write_u8(self.vibrato_type)?;
        writer.write_u8(self.vibrato_sweep)?;
        writer.write_u8(self.vibrato_depth)?;
        writer.write_u8(self.vibrato_rate)?;

        // Volume fadeout
        writer.write_u16::<LittleEndian>(self.volume_fadeout)?;

        // Reserved (22 bytes)
        writer.write_all(&[0u8; 22])?;

        // Write sample header
        self.sample.write_header(writer)?;

        // Write sample data
        self.sample.write_data(writer)?;

        Ok(())
    }
}

/// XM sample data.
#[derive(Debug, Clone, Default)]
pub struct XmSample {
    /// Sample name (22 characters max).
    pub name: String,
    /// Sample data (16-bit signed PCM, will be delta-encoded).
    pub data: Vec<u8>,
    /// Loop start position (in samples).
    pub loop_start: u32,
    /// Loop length (in samples).
    pub loop_length: u32,
    /// Loop type (0=none, 1=forward, 2=ping-pong).
    pub loop_type: u8,
    /// Default volume (0-64).
    pub volume: u8,
    /// Finetune (-128 to 127).
    pub finetune: i8,
    /// Relative note (semitones from C-4).
    pub relative_note: i8,
    /// Panning (0-255, 128=center).
    pub panning: u8,
    /// Is 16-bit sample.
    pub is_16bit: bool,
}

impl XmSample {
    /// Create a new sample with the given data.
    pub fn new(name: &str, data: Vec<u8>, is_16bit: bool) -> Self {
        Self {
            name: name.to_string(),
            data,
            loop_start: 0,
            loop_length: 0,
            loop_type: 0,
            volume: 64,
            finetune: 0,
            relative_note: 0,
            panning: 128,
            is_16bit,
        }
    }

    /// Set loop parameters.
    pub fn with_loop(mut self, start: u32, length: u32, loop_type: u8) -> Self {
        self.loop_start = start;
        self.loop_length = length;
        self.loop_type = loop_type;
        self
    }

    /// Set pitch correction.
    pub fn with_pitch_correction(mut self, finetune: i8, relative_note: i8) -> Self {
        self.finetune = finetune;
        self.relative_note = relative_note;
        self
    }

    /// Get sample length in samples (not bytes).
    pub fn length_samples(&self) -> u32 {
        if self.is_16bit {
            (self.data.len() / 2) as u32
        } else {
            self.data.len() as u32
        }
    }

    /// Write the sample header.
    pub fn write_header<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // Sample length (in bytes for 8-bit, in samples*2 for 16-bit)
        writer.write_u32::<LittleEndian>(self.data.len() as u32)?;

        // Loop start (in samples)
        writer.write_u32::<LittleEndian>(self.loop_start)?;

        // Loop length (in samples)
        writer.write_u32::<LittleEndian>(self.loop_length)?;

        // Volume
        writer.write_u8(self.volume)?;

        // Finetune
        writer.write_i8(self.finetune)?;

        // Type (bits 0-1: loop type, bit 4: 16-bit)
        let type_byte = (self.loop_type & 0x03) | if self.is_16bit { 0x10 } else { 0 };
        writer.write_u8(type_byte)?;

        // Panning
        writer.write_u8(self.panning)?;

        // Relative note
        writer.write_i8(self.relative_note)?;

        // Reserved
        writer.write_u8(0)?;

        // Sample name (22 bytes)
        let name_bytes = self.name.as_bytes();
        let mut name_buf = [0u8; 22];
        let copy_len = name_bytes.len().min(22);
        name_buf[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        writer.write_all(&name_buf)?;

        Ok(())
    }

    /// Write the sample data (delta-encoded).
    pub fn write_data<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.is_16bit {
            // Delta-encode 16-bit samples
            let encoded = delta_encode_16bit(&self.data);
            writer.write_all(&encoded)?;
        } else {
            // 8-bit samples (not delta-encoded in XM)
            writer.write_all(&self.data)?;
        }
        Ok(())
    }
}

/// Delta-encode 16-bit signed PCM samples for XM format.
fn delta_encode_16bit(data: &[u8]) -> Vec<u8> {
    let num_samples = data.len() / 2;
    let mut output = Vec::with_capacity(data.len());
    let mut prev: i16 = 0;

    for i in 0..num_samples {
        let sample = i16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
        let delta = sample.wrapping_sub(prev);
        output.extend_from_slice(&delta.to_le_bytes());
        prev = sample;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_flags() {
        let mut env = XmEnvelope::default();
        assert_eq!(env.flags(), 0);

        env.enabled = true;
        assert_eq!(env.flags(), 1);

        env.sustain_enabled = true;
        assert_eq!(env.flags(), 3);

        env.loop_enabled = true;
        assert_eq!(env.flags(), 7);
    }

    #[test]
    fn test_delta_encode() {
        // Test data: [0, 100, 200, 100]
        let data: Vec<u8> = vec![
            0, 0,     // 0
            100, 0,   // 100
            200, 0,   // 200 (as unsigned, really -56 as i16)
            100, 0,   // 100
        ];

        let encoded = delta_encode_16bit(&data);
        assert_eq!(encoded.len(), 8);

        // First delta should be 0 - 0 = 0
        let d0 = i16::from_le_bytes([encoded[0], encoded[1]]);
        assert_eq!(d0, 0);

        // Second delta should be 100 - 0 = 100
        let d1 = i16::from_le_bytes([encoded[2], encoded[3]]);
        assert_eq!(d1, 100);

        // Third delta should be 200 - 100 = 100
        let d2 = i16::from_le_bytes([encoded[4], encoded[5]]);
        assert_eq!(d2, 100);

        // Fourth delta should be 100 - 200 = -100
        let d3 = i16::from_le_bytes([encoded[6], encoded[7]]);
        assert_eq!(d3, -100);
    }

    #[test]
    fn test_adsr_envelope() {
        let env = XmEnvelope::adsr(10, 20, 32, 30);
        assert!(env.enabled);
        assert!(env.sustain_enabled);
        assert!(!env.loop_enabled);
        assert!(env.points.len() >= 4);
    }
}
