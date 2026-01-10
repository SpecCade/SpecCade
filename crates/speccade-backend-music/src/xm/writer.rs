//! XM file writer - assembles all components into a complete XM file.

use std::io::{self, Write};

use super::header::XmHeader;
use super::instrument::XmInstrument;
use super::pattern::XmPattern;

/// XM module containing all song data.
#[derive(Debug, Clone)]
pub struct XmModule {
    /// Module header.
    pub header: XmHeader,
    /// Patterns.
    pub patterns: Vec<XmPattern>,
    /// Instruments.
    pub instruments: Vec<XmInstrument>,
}

impl XmModule {
    /// Create a new XM module with the given parameters.
    pub fn new(name: &str, num_channels: u8, speed: u8, bpm: u16) -> Self {
        Self {
            header: XmHeader::new(name, num_channels, 0, 0, speed, bpm),
            patterns: Vec::new(),
            instruments: Vec::new(),
        }
    }

    /// Add a pattern to the module.
    pub fn add_pattern(&mut self, pattern: XmPattern) {
        self.patterns.push(pattern);
        self.header.num_patterns = self.patterns.len() as u16;
    }

    /// Add an instrument to the module.
    pub fn add_instrument(&mut self, instrument: XmInstrument) {
        self.instruments.push(instrument);
        self.header.num_instruments = self.instruments.len() as u16;
    }

    /// Set the order table (pattern playback order).
    pub fn set_order_table(&mut self, orders: &[u8]) {
        self.header.set_order_table(orders);
    }

    /// Set the restart position for looping.
    pub fn set_restart_position(&mut self, position: u16) {
        self.header.restart_position = position;
    }

    /// Write the complete XM module to a writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // Write header
        self.header.write(writer)?;

        // Write patterns
        for pattern in &self.patterns {
            pattern.write(writer, self.header.num_channels as u8)?;
        }

        // Write instruments
        for instrument in &self.instruments {
            instrument.write(writer)?;
        }

        Ok(())
    }

    /// Write the module to a byte vector.
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.write(&mut buffer)?;
        Ok(buffer)
    }

    /// Compute the BLAKE3 hash of the module bytes.
    pub fn compute_hash(&self) -> io::Result<String> {
        let bytes = self.to_bytes()?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }
}

/// Validate an XM file has correct magic and version.
pub fn validate_xm_bytes(data: &[u8]) -> Result<(), XmValidationError> {
    if data.len() < 60 {
        return Err(XmValidationError::FileTooSmall(data.len()));
    }

    if &data[0..17] != super::header::XM_MAGIC {
        return Err(XmValidationError::InvalidMagic);
    }

    let version = u16::from_le_bytes([data[58], data[59]]);
    if version != super::header::XM_VERSION {
        return Err(XmValidationError::UnsupportedVersion(version));
    }

    Ok(())
}

/// XM validation error.
#[derive(Debug, Clone)]
pub enum XmValidationError {
    /// File is too small to be a valid XM file.
    FileTooSmall(usize),
    /// Invalid magic identifier.
    InvalidMagic,
    /// Unsupported XM version.
    UnsupportedVersion(u16),
}

impl std::fmt::Display for XmValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XmValidationError::FileTooSmall(size) => {
                write!(f, "File too small: {} bytes", size)
            }
            XmValidationError::InvalidMagic => {
                write!(f, "Invalid XM magic identifier")
            }
            XmValidationError::UnsupportedVersion(version) => {
                write!(f, "Unsupported XM version: 0x{:04X}", version)
            }
        }
    }
}

impl std::error::Error for XmValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xm::instrument::{XmInstrument, XmSample};
    use crate::xm::pattern::{XmNote, XmPattern};

    #[test]
    fn test_module_creation() {
        let mut module = XmModule::new("Test Song", 4, 6, 125);

        // Add an empty pattern
        let pattern = XmPattern::empty(64, 4);
        module.add_pattern(pattern);

        // Add an instrument with empty sample
        let sample = XmSample::new("Test", vec![], true);
        let instrument = XmInstrument::new("Lead", sample);
        module.add_instrument(instrument);

        // Set order table
        module.set_order_table(&[0]);

        // Write to bytes
        let bytes = module.to_bytes().unwrap();

        // Validate
        assert!(validate_xm_bytes(&bytes).is_ok());
    }

    #[test]
    fn test_module_with_notes() {
        let mut module = XmModule::new("Note Test", 2, 6, 120);

        // Create pattern with notes
        let mut pattern = XmPattern::empty(16, 2);
        pattern.set_note(0, 0, XmNote::from_name("C4", 1, Some(64)));
        pattern.set_note(4, 0, XmNote::from_name("E4", 1, Some(64)));
        pattern.set_note(8, 0, XmNote::from_name("G4", 1, Some(64)));
        pattern.set_note(12, 0, XmNote::note_off());
        module.add_pattern(pattern);

        // Add instrument
        let sample_data = vec![0u8; 1000]; // Silence
        let sample = XmSample::new("Lead", sample_data, true);
        let instrument = XmInstrument::new("Lead", sample);
        module.add_instrument(instrument);

        module.set_order_table(&[0]);

        let bytes = module.to_bytes().unwrap();
        assert!(validate_xm_bytes(&bytes).is_ok());
    }

    #[test]
    fn test_hash_determinism() {
        let mut module1 = XmModule::new("Hash Test", 4, 6, 125);
        module1.add_pattern(XmPattern::empty(64, 4));
        module1.set_order_table(&[0]);

        let mut module2 = XmModule::new("Hash Test", 4, 6, 125);
        module2.add_pattern(XmPattern::empty(64, 4));
        module2.set_order_table(&[0]);

        let hash1 = module1.compute_hash().unwrap();
        let hash2 = module2.compute_hash().unwrap();
        assert_eq!(hash1, hash2);
    }
}
