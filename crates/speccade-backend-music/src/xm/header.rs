//! XM file header structures and constants.

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Write};

/// XM file magic identifier.
pub const XM_MAGIC: &[u8; 17] = b"Extended Module: ";

/// XM format version (1.04).
pub const XM_VERSION: u16 = 0x0104;

/// Header size (fixed at 276 bytes for version 1.04).
pub const XM_HEADER_SIZE: u32 = 276;

/// Maximum number of channels in XM format.
pub const XM_MAX_CHANNELS: u8 = 32;

/// Maximum number of patterns in XM format.
pub const XM_MAX_PATTERNS: u16 = 256;

/// Maximum number of instruments in XM format.
pub const XM_MAX_INSTRUMENTS: u16 = 128;

/// Maximum pattern length in rows.
pub const XM_MAX_PATTERN_ROWS: u16 = 256;

/// XM module header data.
#[derive(Debug, Clone)]
pub struct XmHeader {
    /// Song name (20 characters max).
    pub name: String,
    /// Number of positions in order table.
    pub song_length: u16,
    /// Restart position for looping.
    pub restart_position: u16,
    /// Number of channels.
    pub num_channels: u16,
    /// Number of patterns.
    pub num_patterns: u16,
    /// Number of instruments.
    pub num_instruments: u16,
    /// Flags (bit 0: linear frequency table).
    pub flags: u16,
    /// Default tempo (ticks per row).
    pub default_speed: u16,
    /// Default BPM.
    pub default_bpm: u16,
    /// Pattern order table (256 entries).
    pub order_table: [u8; 256],
}

impl Default for XmHeader {
    fn default() -> Self {
        Self {
            name: String::new(),
            song_length: 1,
            restart_position: 0,
            num_channels: 4,
            num_patterns: 1,
            num_instruments: 0,
            flags: 1, // Linear frequency table enabled
            default_speed: 6,
            default_bpm: 125,
            order_table: [0; 256],
        }
    }
}

impl XmHeader {
    /// Create a new XM header with the given parameters.
    pub fn new(
        name: &str,
        num_channels: u8,
        num_patterns: u16,
        num_instruments: u16,
        speed: u8,
        bpm: u16,
    ) -> Self {
        Self {
            name: name.to_string(),
            song_length: 1,
            restart_position: 0,
            num_channels: num_channels as u16,
            num_patterns,
            num_instruments,
            flags: 1, // Linear frequency table
            default_speed: speed as u16,
            default_bpm: bpm,
            order_table: [0; 256],
        }
    }

    /// Set the order table from a slice.
    pub fn set_order_table(&mut self, orders: &[u8]) {
        self.song_length = orders.len().min(256) as u16;
        for (i, &order) in orders.iter().enumerate().take(256) {
            self.order_table[i] = order;
        }
    }

    /// Write the header to a writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // Magic identifier
        writer.write_all(XM_MAGIC)?;

        // Song name (20 bytes, null-padded)
        let name_bytes = self.name.as_bytes();
        let mut name_buf = [0u8; 20];
        let copy_len = name_bytes.len().min(20);
        name_buf[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        writer.write_all(&name_buf)?;

        // 0x1A byte
        writer.write_u8(0x1A)?;

        // Tracker name (20 bytes)
        let tracker_name = b"SpecCade XM Writer  ";
        writer.write_all(tracker_name)?;

        // Version
        writer.write_u16::<LittleEndian>(XM_VERSION)?;

        // Header size
        writer.write_u32::<LittleEndian>(XM_HEADER_SIZE)?;

        // Song length (number of pattern positions)
        writer.write_u16::<LittleEndian>(self.song_length)?;

        // Restart position
        writer.write_u16::<LittleEndian>(self.restart_position)?;

        // Number of channels
        writer.write_u16::<LittleEndian>(self.num_channels)?;

        // Number of patterns
        writer.write_u16::<LittleEndian>(self.num_patterns)?;

        // Number of instruments
        writer.write_u16::<LittleEndian>(self.num_instruments)?;

        // Flags
        writer.write_u16::<LittleEndian>(self.flags)?;

        // Default speed
        writer.write_u16::<LittleEndian>(self.default_speed)?;

        // Default BPM
        writer.write_u16::<LittleEndian>(self.default_bpm)?;

        // Order table (256 bytes)
        writer.write_all(&self.order_table)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_write() {
        let header = XmHeader {
            name: "Test Song".to_string(),
            num_channels: 8,
            num_patterns: 4,
            num_instruments: 2,
            ..Default::default()
        };

        let mut buf = Vec::new();
        header.write(&mut buf).unwrap();

        // Header should be exactly 60 bytes + 256 order table = 336 bytes
        // Actually: 17 magic + 20 name + 1 0x1A + 20 tracker + 2 version + 4 header_size
        //         + 2 song_length + 2 restart + 2 channels + 2 patterns + 2 instruments
        //         + 2 flags + 2 speed + 2 bpm + 256 order = 336 bytes
        assert_eq!(buf.len(), 336);

        // Check magic
        assert_eq!(&buf[0..17], XM_MAGIC);
    }
}
