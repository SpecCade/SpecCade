//! IT file header structures and constants.

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Write};

/// IT file magic identifier.
pub const IT_MAGIC: &[u8; 4] = b"IMPM";

/// IT format version (compatible with 2.14).
pub const IT_CWTT: u16 = 0x0214;

/// IT minimum compatible version.
pub const IT_CMWT: u16 = 0x0200;

/// Maximum number of channels in IT format.
pub const IT_MAX_CHANNELS: u8 = 64;

/// Maximum number of patterns in IT format.
pub const IT_MAX_PATTERNS: u16 = 200;

/// Maximum number of instruments in IT format.
pub const IT_MAX_INSTRUMENTS: u16 = 99;

/// Maximum number of samples in IT format.
pub const IT_MAX_SAMPLES: u16 = 99;

/// IT module flags.
pub mod flags {
    /// Stereo output.
    pub const STEREO: u16 = 0x01;
    /// Vol0MixOptimizations.
    pub const VOL0_MIX: u16 = 0x02;
    /// Use instruments (vs samples only).
    pub const USE_INSTRUMENTS: u16 = 0x04;
    /// Linear slides.
    pub const LINEAR_SLIDES: u16 = 0x08;
    /// Old effects (for compatibility).
    pub const OLD_EFFECTS: u16 = 0x10;
    /// Link effect G memory.
    pub const LINK_G_MEMORY: u16 = 0x20;
}

/// IT special flags.
pub mod special {
    /// Song message present.
    pub const MESSAGE: u16 = 0x01;
}

/// Order table markers.
pub mod order {
    /// End of order list.
    pub const END: u8 = 255;
    /// Skip this order.
    pub const SKIP: u8 = 254;
}

/// IT module header data.
#[derive(Debug, Clone)]
pub struct ItHeader {
    /// Song name (26 characters max).
    pub name: String,
    /// Pattern highlight (minor, major).
    pub pattern_highlight: (u8, u8),
    /// Order list length.
    pub order_count: u16,
    /// Number of instruments.
    pub instrument_count: u16,
    /// Number of samples.
    pub sample_count: u16,
    /// Number of patterns.
    pub pattern_count: u16,
    /// Created with tracker version.
    pub cwtt: u16,
    /// Compatible with version.
    pub cmwt: u16,
    /// Module flags.
    pub flags: u16,
    /// Special flags.
    pub special: u16,
    /// Global volume (0-128).
    pub global_volume: u8,
    /// Mix volume (0-128).
    pub mix_volume: u8,
    /// Initial speed (ticks per row).
    pub initial_speed: u8,
    /// Initial tempo (BPM).
    pub initial_tempo: u8,
    /// Panning separation (0-128).
    pub panning_separation: u8,
    /// Pitch wheel depth.
    pub pitch_wheel_depth: u8,
    /// Message length.
    pub message_length: u16,
    /// Message offset (0 if no message).
    pub message_offset: u32,
    /// Channel panning (64 channels).
    pub channel_pan: [u8; 64],
    /// Channel volume (64 channels).
    pub channel_vol: [u8; 64],
    /// Order list.
    pub orders: Vec<u8>,
}

impl Default for ItHeader {
    fn default() -> Self {
        // Default channel panning: center (32) for first 8, disabled (128) for rest
        let mut channel_pan = [128u8; 64];
        for pan in channel_pan.iter_mut().take(8) {
            *pan = 32; // Center
        }

        // Default channel volume: 64 for first 8, 0 for rest
        let mut channel_vol = [0u8; 64];
        for vol in channel_vol.iter_mut().take(8) {
            *vol = 64;
        }

        Self {
            name: String::new(),
            pattern_highlight: (4, 16),
            order_count: 1,
            instrument_count: 0,
            sample_count: 0,
            pattern_count: 0,
            cwtt: IT_CWTT,
            cmwt: IT_CMWT,
            flags: flags::STEREO | flags::USE_INSTRUMENTS | flags::LINEAR_SLIDES,
            special: 0,
            global_volume: 128,
            mix_volume: 48,
            initial_speed: 6,
            initial_tempo: 125,
            panning_separation: 128,
            pitch_wheel_depth: 0,
            message_length: 0,
            message_offset: 0,
            channel_pan,
            channel_vol,
            orders: vec![0],
        }
    }
}

impl ItHeader {
    /// Create a new IT header with the given parameters.
    pub fn new(name: &str, num_channels: u8, speed: u8, bpm: u8) -> Self {
        // Configure channels: enabled channels get center pan (32), full volume (64)
        // Disabled channels get pan 128 and volume 0
        let mut channel_pan = [128u8; 64];
        let mut channel_vol = [0u8; 64];
        for i in 0..num_channels as usize {
            channel_pan[i] = 32; // Center
            channel_vol[i] = 64; // Full volume
        }

        Self {
            name: name.to_string(),
            initial_speed: speed,
            initial_tempo: bpm,
            channel_pan,
            channel_vol,
            ..Self::default()
        }
    }

    /// Set the order list.
    pub fn set_orders(&mut self, orders: &[u8]) {
        self.orders = orders.to_vec();
        self.order_count = orders.len() as u16;
    }

    /// Calculate the size of the header in bytes.
    pub fn header_size(&self) -> usize {
        192 // Fixed header size
    }

    /// Write the header to a writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // Magic identifier
        writer.write_all(IT_MAGIC)?;

        // Song name (26 bytes)
        let name_bytes = self.name.as_bytes();
        let mut name_buf = [0u8; 26];
        let copy_len = name_bytes.len().min(26);
        name_buf[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        writer.write_all(&name_buf)?;

        // Pattern highlight (minor, major)
        writer.write_u8(self.pattern_highlight.0)?;
        writer.write_u8(self.pattern_highlight.1)?;

        // Order count
        writer.write_u16::<LittleEndian>(self.order_count)?;

        // Instrument count
        writer.write_u16::<LittleEndian>(self.instrument_count)?;

        // Sample count
        writer.write_u16::<LittleEndian>(self.sample_count)?;

        // Pattern count
        writer.write_u16::<LittleEndian>(self.pattern_count)?;

        // Cwt/v (created with tracker version)
        writer.write_u16::<LittleEndian>(self.cwtt)?;

        // Cmwt (compatible with version)
        writer.write_u16::<LittleEndian>(self.cmwt)?;

        // Flags
        writer.write_u16::<LittleEndian>(self.flags)?;

        // Special
        writer.write_u16::<LittleEndian>(self.special)?;

        // Global volume
        writer.write_u8(self.global_volume)?;

        // Mix volume
        writer.write_u8(self.mix_volume)?;

        // Initial speed
        writer.write_u8(self.initial_speed)?;

        // Initial tempo
        writer.write_u8(self.initial_tempo)?;

        // Panning separation
        writer.write_u8(self.panning_separation)?;

        // Pitch wheel depth
        writer.write_u8(self.pitch_wheel_depth)?;

        // Message length
        writer.write_u16::<LittleEndian>(self.message_length)?;

        // Message offset
        writer.write_u32::<LittleEndian>(self.message_offset)?;

        // Reserved
        writer.write_all(&[0u8; 4])?;

        // Channel panning (64 bytes)
        writer.write_all(&self.channel_pan)?;

        // Channel volume (64 bytes)
        writer.write_all(&self.channel_vol)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_write() {
        let header = ItHeader::new("Test Song", 8, 6, 125);

        let mut buf = Vec::new();
        header.write(&mut buf).unwrap();

        // Header should be exactly 192 bytes
        assert_eq!(buf.len(), 192);

        // Check magic
        assert_eq!(&buf[0..4], IT_MAGIC);
    }

    #[test]
    fn test_default_flags() {
        let header = ItHeader::default();
        assert_eq!(
            header.flags,
            flags::STEREO | flags::USE_INSTRUMENTS | flags::LINEAR_SLIDES
        );
    }
}
