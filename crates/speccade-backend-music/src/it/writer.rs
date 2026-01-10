//! IT file writer - assembles all components into a complete IT file.

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Cursor, Seek, Write};

use super::header::ItHeader;
use super::instrument::ItInstrument;
use super::pattern::ItPattern;
use super::sample::ItSample;

/// IT module containing all song data.
#[derive(Debug, Clone)]
pub struct ItModule {
    /// Module header.
    pub header: ItHeader,
    /// Patterns.
    pub patterns: Vec<ItPattern>,
    /// Instruments.
    pub instruments: Vec<ItInstrument>,
    /// Samples.
    pub samples: Vec<ItSample>,
    /// Optional song message.
    pub message: Option<String>,
}

impl ItModule {
    /// Create a new IT module with the given parameters.
    pub fn new(name: &str, num_channels: u8, speed: u8, bpm: u8) -> Self {
        Self {
            header: ItHeader::new(name, num_channels, speed, bpm),
            patterns: Vec::new(),
            instruments: Vec::new(),
            samples: Vec::new(),
            message: None,
        }
    }

    /// Add a pattern to the module.
    pub fn add_pattern(&mut self, pattern: ItPattern) {
        self.patterns.push(pattern);
        self.header.pattern_count = self.patterns.len() as u16;
    }

    /// Add an instrument to the module.
    pub fn add_instrument(&mut self, instrument: ItInstrument) {
        self.instruments.push(instrument);
        self.header.instrument_count = self.instruments.len() as u16;
    }

    /// Add a sample to the module.
    pub fn add_sample(&mut self, sample: ItSample) {
        self.samples.push(sample);
        self.header.sample_count = self.samples.len() as u16;
    }

    /// Set the order list (pattern playback order).
    pub fn set_orders(&mut self, orders: &[u8]) {
        self.header.set_orders(orders);
    }

    /// Set the song message.
    pub fn set_message(&mut self, message: &str) {
        self.message = Some(message.to_string());
        self.header.special |= super::header::special::MESSAGE;
        self.header.message_length = message.len() as u16 + 1;
    }

    /// Write the complete IT module to a writer.
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> io::Result<()> {
        // Calculate offsets
        let header_size = 192;
        let orders_size = self.header.orders.len();
        let num_instruments = self.instruments.len();
        let num_samples = self.samples.len();
        let num_patterns = self.patterns.len();

        // Offset table starts after header + orders
        let offset_table_start = header_size + orders_size;

        // Offset table size: (instruments + samples + patterns) * 4 bytes each
        let offset_table_size = (num_instruments + num_samples + num_patterns) * 4;

        // Message offset (if present)
        let message_offset = if self.message.is_some() {
            offset_table_start + offset_table_size
        } else {
            0
        };

        let message_size = self.message.as_ref().map(|m| m.len() + 1).unwrap_or(0);

        // Instruments start after message
        let instruments_start = offset_table_start + offset_table_size + message_size;

        // Calculate instrument offsets (each instrument is 554 bytes)
        let instrument_offsets: Vec<u32> = (0..num_instruments)
            .map(|i| (instruments_start + i * super::instrument::IT_INSTRUMENT_SIZE) as u32)
            .collect();

        // Samples start after instruments
        let samples_start =
            instruments_start + num_instruments * super::instrument::IT_INSTRUMENT_SIZE;

        // Calculate sample header offsets (each header is 80 bytes)
        let sample_header_offsets: Vec<u32> = (0..num_samples)
            .map(|i| (samples_start + i * super::sample::IT_SAMPLE_HEADER_SIZE) as u32)
            .collect();

        // Patterns start after sample headers
        let patterns_start = samples_start + num_samples * super::sample::IT_SAMPLE_HEADER_SIZE;

        // Calculate pattern offsets (variable size due to packing)
        let mut pattern_offsets = Vec::with_capacity(num_patterns);
        let mut packed_patterns = Vec::with_capacity(num_patterns);
        let mut current_offset = patterns_start;

        for pattern in &self.patterns {
            pattern_offsets.push(current_offset as u32);
            let packed = pattern.pack(
                self.header
                    .channel_pan
                    .iter()
                    .filter(|&&p| p != 128)
                    .count() as u8,
            );
            let pattern_size = 8 + packed.len(); // 8-byte header + data
            current_offset += pattern_size;
            packed_patterns.push(packed);
        }

        // Sample data starts after patterns
        let sample_data_start = current_offset;

        // Calculate sample data offsets
        let mut sample_data_offsets = Vec::with_capacity(num_samples);
        let mut current_data_offset = sample_data_start;
        for sample in &self.samples {
            sample_data_offsets.push(current_data_offset as u32);
            current_data_offset += sample.data.len();
        }

        // Update header with message offset
        let mut header = self.header.clone();
        if self.message.is_some() {
            header.message_offset = message_offset as u32;
        }

        // Write header
        header.write(writer)?;

        // Write orders
        writer.write_all(&header.orders)?;

        // Write instrument offset table
        for offset in &instrument_offsets {
            writer.write_u32::<LittleEndian>(*offset)?;
        }

        // Write sample offset table
        for offset in &sample_header_offsets {
            writer.write_u32::<LittleEndian>(*offset)?;
        }

        // Write pattern offset table
        for offset in &pattern_offsets {
            writer.write_u32::<LittleEndian>(*offset)?;
        }

        // Write message (if present)
        if let Some(ref message) = self.message {
            writer.write_all(message.as_bytes())?;
            writer.write_u8(0)?; // Null terminator
        }

        // Write instruments
        for instrument in &self.instruments {
            instrument.write(writer)?;
        }

        // Write sample headers
        for (i, sample) in self.samples.iter().enumerate() {
            sample.write_header(writer, sample_data_offsets[i])?;
        }

        // Write patterns
        for (i, pattern) in self.patterns.iter().enumerate() {
            let packed = &packed_patterns[i];
            writer.write_u16::<LittleEndian>(packed.len() as u16)?;
            writer.write_u16::<LittleEndian>(pattern.num_rows)?;
            writer.write_all(&[0u8; 4])?; // Reserved
            writer.write_all(packed)?;
        }

        // Write sample data
        for sample in &self.samples {
            sample.write_data(writer)?;
        }

        Ok(())
    }

    /// Write the module to a byte vector.
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        self.write(&mut buffer)?;
        Ok(buffer.into_inner())
    }

    /// Compute the BLAKE3 hash of the module bytes.
    pub fn compute_hash(&self) -> io::Result<String> {
        let bytes = self.to_bytes()?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }
}

/// Validate an IT file has correct magic.
pub fn validate_it_bytes(data: &[u8]) -> Result<(), ItValidationError> {
    if data.len() < 4 {
        return Err(ItValidationError::FileTooSmall(data.len()));
    }

    if &data[0..4] != super::header::IT_MAGIC {
        return Err(ItValidationError::InvalidMagic);
    }

    Ok(())
}

/// IT validation error.
#[derive(Debug, Clone)]
pub enum ItValidationError {
    /// File is too small to be a valid IT file.
    FileTooSmall(usize),
    /// Invalid magic identifier.
    InvalidMagic,
}

impl std::fmt::Display for ItValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItValidationError::FileTooSmall(size) => {
                write!(f, "File too small: {} bytes", size)
            }
            ItValidationError::InvalidMagic => {
                write!(f, "Invalid IT magic identifier")
            }
        }
    }
}

impl std::error::Error for ItValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::it::instrument::ItInstrument;
    use crate::it::pattern::{ItNote, ItPattern};
    use crate::it::sample::ItSample;

    #[test]
    fn test_module_creation() {
        let mut module = ItModule::new("Test Song", 4, 6, 125);

        // Add an empty pattern
        let pattern = ItPattern::empty(64, 4);
        module.add_pattern(pattern);

        // Add an instrument
        let instrument = ItInstrument::new("Lead");
        module.add_instrument(instrument);

        // Add a sample
        let sample = ItSample::new("Lead Sample", vec![0u8; 100], 22050);
        module.add_sample(sample);

        // Set order table
        module.set_orders(&[0]);

        // Write to bytes
        let bytes = module.to_bytes().unwrap();

        // Validate
        assert!(validate_it_bytes(&bytes).is_ok());
    }

    #[test]
    fn test_module_with_notes() {
        let mut module = ItModule::new("Note Test", 2, 6, 120);

        // Create pattern with notes
        let mut pattern = ItPattern::empty(16, 2);
        pattern.set_note(0, 0, ItNote::from_name("C4", 1, 64));
        pattern.set_note(4, 0, ItNote::from_name("E4", 1, 64));
        pattern.set_note(8, 0, ItNote::from_name("G4", 1, 64));
        pattern.set_note(12, 0, ItNote::note_off());
        module.add_pattern(pattern);

        // Add instrument
        let instrument = ItInstrument::new("Lead");
        module.add_instrument(instrument);

        // Add sample
        let sample = ItSample::new("Lead", vec![0u8; 1000], 22050);
        module.add_sample(sample);

        module.set_orders(&[0]);

        let bytes = module.to_bytes().unwrap();
        assert!(validate_it_bytes(&bytes).is_ok());
    }

    #[test]
    fn test_hash_determinism() {
        let create_module = || {
            let mut module = ItModule::new("Hash Test", 4, 6, 125);
            module.add_pattern(ItPattern::empty(64, 4));
            let instrument = ItInstrument::new("Test");
            module.add_instrument(instrument);
            let sample = ItSample::new("Test", vec![0u8; 100], 22050);
            module.add_sample(sample);
            module.set_orders(&[0]);
            module
        };

        let module1 = create_module();
        let module2 = create_module();

        let hash1 = module1.compute_hash().unwrap();
        let hash2 = module2.compute_hash().unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_module_with_message() {
        let mut module = ItModule::new("Message Test", 4, 6, 125);
        module.add_pattern(ItPattern::empty(64, 4));
        module.set_message("Created by SpecCade");
        module.set_orders(&[0]);

        let bytes = module.to_bytes().unwrap();
        assert!(validate_it_bytes(&bytes).is_ok());
    }
}
