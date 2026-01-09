//! IT sample structures and writing.

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Write};

/// IT sample magic identifier.
pub const IT_SAMPLE_MAGIC: &[u8; 4] = b"IMPS";

/// IT sample header size.
pub const IT_SAMPLE_HEADER_SIZE: usize = 80;

/// Sample flags.
pub mod sample_flags {
    /// Sample has data.
    pub const HAS_DATA: u8 = 0x01;
    /// 16-bit sample.
    pub const BITS_16: u8 = 0x02;
    /// Stereo sample.
    pub const STEREO: u8 = 0x04;
    /// Compressed sample.
    pub const COMPRESSED: u8 = 0x08;
    /// Loop enabled.
    pub const LOOP: u8 = 0x10;
    /// Sustain loop enabled.
    pub const SUSTAIN_LOOP: u8 = 0x20;
    /// Ping-pong loop.
    pub const LOOP_PINGPONG: u8 = 0x40;
    /// Ping-pong sustain loop.
    pub const SUSTAIN_PINGPONG: u8 = 0x80;
}

/// Convert flags.
pub mod convert_flags {
    /// Signed samples.
    pub const SIGNED: u8 = 0x01;
    /// Big-endian (not typically used).
    pub const BIG_ENDIAN: u8 = 0x02;
    /// Delta encoded.
    pub const DELTA: u8 = 0x04;
    /// Byte delta (8-bit only).
    pub const BYTE_DELTA: u8 = 0x08;
    /// TX-Wave 12-bit.
    pub const TXWAVE: u8 = 0x10;
    /// Left/Right stereo prompt.
    pub const STEREO_PROMPT: u8 = 0x20;
}

/// IT sample definition.
#[derive(Debug, Clone)]
pub struct ItSample {
    /// Sample name (26 characters max).
    pub name: String,
    /// DOS filename (12 characters max).
    pub filename: String,
    /// Global volume (0-64).
    pub global_volume: u8,
    /// Sample flags.
    pub flags: u8,
    /// Default volume (0-64).
    pub default_volume: u8,
    /// Default panning (0-64, or None for disabled).
    pub default_pan: Option<u8>,
    /// Sample length in samples.
    pub length: u32,
    /// Loop begin (in samples).
    pub loop_begin: u32,
    /// Loop end (in samples).
    pub loop_end: u32,
    /// C-5 speed (sample rate for C-5 playback).
    pub c5_speed: u32,
    /// Sustain loop begin.
    pub sustain_loop_begin: u32,
    /// Sustain loop end.
    pub sustain_loop_end: u32,
    /// Vibrato speed.
    pub vibrato_speed: u8,
    /// Vibrato depth.
    pub vibrato_depth: u8,
    /// Vibrato rate.
    pub vibrato_rate: u8,
    /// Vibrato type (0=sine, 1=ramp down, 2=square, 3=random).
    pub vibrato_type: u8,
    /// Sample data (16-bit signed PCM).
    pub data: Vec<u8>,
}

impl Default for ItSample {
    fn default() -> Self {
        Self {
            name: String::new(),
            filename: String::new(),
            global_volume: 64,
            flags: sample_flags::HAS_DATA | sample_flags::BITS_16,
            default_volume: 64,
            default_pan: None,
            length: 0,
            loop_begin: 0,
            loop_end: 0,
            c5_speed: 22050,
            sustain_loop_begin: 0,
            sustain_loop_end: 0,
            vibrato_speed: 0,
            vibrato_depth: 0,
            vibrato_rate: 0,
            vibrato_type: 0,
            data: Vec::new(),
        }
    }
}

impl ItSample {
    /// Create a new sample with the given name and data.
    pub fn new(name: &str, data: Vec<u8>, sample_rate: u32) -> Self {
        let length = (data.len() / 2) as u32; // 16-bit samples
        Self {
            name: name.to_string(),
            data,
            length,
            c5_speed: sample_rate,
            flags: sample_flags::HAS_DATA | sample_flags::BITS_16,
            ..Default::default()
        }
    }

    /// Set loop parameters.
    pub fn with_loop(mut self, begin: u32, end: u32, pingpong: bool) -> Self {
        self.flags |= sample_flags::LOOP;
        if pingpong {
            self.flags |= sample_flags::LOOP_PINGPONG;
        }
        self.loop_begin = begin;
        self.loop_end = end;
        self
    }

    /// Get the length of sample data in samples (not bytes).
    pub fn length_samples(&self) -> u32 {
        self.length
    }

    /// Write the sample header to a writer.
    pub fn write_header<W: Write>(&self, writer: &mut W, sample_data_offset: u32) -> io::Result<()> {
        // Magic
        writer.write_all(IT_SAMPLE_MAGIC)?;

        // DOS filename (12 bytes)
        let filename_bytes = self.filename.as_bytes();
        let mut filename_buf = [0u8; 12];
        let copy_len = filename_bytes.len().min(12);
        filename_buf[..copy_len].copy_from_slice(&filename_bytes[..copy_len]);
        writer.write_all(&filename_buf)?;

        // Reserved byte
        writer.write_u8(0)?;

        // Global volume
        writer.write_u8(self.global_volume)?;

        // Flags
        writer.write_u8(self.flags)?;

        // Default volume
        writer.write_u8(self.default_volume)?;

        // Sample name (26 bytes)
        let name_bytes = self.name.as_bytes();
        let mut name_buf = [0u8; 26];
        let copy_len = name_bytes.len().min(26);
        name_buf[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        writer.write_all(&name_buf)?;

        // Convert flags (signed samples)
        writer.write_u8(convert_flags::SIGNED)?;

        // Default pan (bit 7 set = use pan)
        let dfp = match self.default_pan {
            Some(pan) => pan | 0x80,
            None => 0,
        };
        writer.write_u8(dfp)?;

        // Length (in samples)
        writer.write_u32::<LittleEndian>(self.length)?;

        // Loop begin
        writer.write_u32::<LittleEndian>(self.loop_begin)?;

        // Loop end
        writer.write_u32::<LittleEndian>(self.loop_end)?;

        // C-5 speed
        writer.write_u32::<LittleEndian>(self.c5_speed)?;

        // Sustain loop begin
        writer.write_u32::<LittleEndian>(self.sustain_loop_begin)?;

        // Sustain loop end
        writer.write_u32::<LittleEndian>(self.sustain_loop_end)?;

        // Sample data pointer (offset in file)
        writer.write_u32::<LittleEndian>(sample_data_offset)?;

        // Vibrato parameters
        writer.write_u8(self.vibrato_speed)?;
        writer.write_u8(self.vibrato_depth)?;
        writer.write_u8(self.vibrato_rate)?;
        writer.write_u8(self.vibrato_type)?;

        Ok(())
    }

    /// Write the sample data to a writer.
    /// IT uses signed samples directly (no delta encoding by default).
    pub fn write_data<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.data)?;
        Ok(())
    }
}

/// Convert unsigned 8-bit samples to signed 8-bit.
pub fn convert_u8_to_s8(data: &[u8]) -> Vec<u8> {
    data.iter().map(|&s| (s as i8).wrapping_sub(-128) as u8).collect()
}

/// Convert f32 samples to 16-bit signed PCM bytes.
pub fn convert_f32_to_s16_bytes(samples: &[f32]) -> Vec<u8> {
    let mut output = Vec::with_capacity(samples.len() * 2);

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_sample = (clamped * 32767.0) as i16;
        output.extend_from_slice(&i16_sample.to_le_bytes());
    }

    output
}

/// Convert f64 samples to 16-bit signed PCM bytes.
pub fn convert_f64_to_s16_bytes(samples: &[f64]) -> Vec<u8> {
    let mut output = Vec::with_capacity(samples.len() * 2);

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_sample = (clamped * 32767.0) as i16;
        output.extend_from_slice(&i16_sample.to_le_bytes());
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_creation() {
        let data = vec![0u8; 1000];
        let sample = ItSample::new("Test", data, 22050);

        assert_eq!(sample.length, 500); // 500 16-bit samples
        assert_eq!(sample.c5_speed, 22050);
        assert!(sample.flags & sample_flags::HAS_DATA != 0);
        assert!(sample.flags & sample_flags::BITS_16 != 0);
    }

    #[test]
    fn test_sample_header_write() {
        let data = vec![0u8; 100];
        let sample = ItSample::new("Test", data, 22050);

        let mut buf = Vec::new();
        sample.write_header(&mut buf, 1000).unwrap();

        // Header should be 80 bytes
        assert_eq!(buf.len(), IT_SAMPLE_HEADER_SIZE);

        // Check magic
        assert_eq!(&buf[0..4], IT_SAMPLE_MAGIC);
    }

    #[test]
    fn test_with_loop() {
        let data = vec![0u8; 1000];
        let sample = ItSample::new("Test", data, 22050).with_loop(0, 100, false);

        assert!(sample.flags & sample_flags::LOOP != 0);
        assert_eq!(sample.loop_begin, 0);
        assert_eq!(sample.loop_end, 100);
    }

    #[test]
    fn test_convert_f32() {
        let samples = vec![0.0f32, 0.5, 1.0, -1.0];
        let bytes = convert_f32_to_s16_bytes(&samples);

        assert_eq!(bytes.len(), 8);

        let s0 = i16::from_le_bytes([bytes[0], bytes[1]]);
        assert_eq!(s0, 0);

        let s2 = i16::from_le_bytes([bytes[4], bytes[5]]);
        assert_eq!(s2, 32767);
    }
}
