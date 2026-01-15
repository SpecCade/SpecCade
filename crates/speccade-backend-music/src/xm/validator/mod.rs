//! Comprehensive XM (FastTracker II Extended Module) format validator.
//!
//! This module provides thorough validation of XM files against the official
//! XM file format specification (v1.04).
//!
//! # XM Format Overview
//!
//! The XM format was introduced by Triton's FastTracker II and features:
//! - Up to 32 channels
//! - Up to 128 instruments with embedded samples
//! - Up to 256 patterns
//! - Volume and panning envelopes (12 points max each)
//! - Linear or Amiga frequency tables
//! - 16-bit sample support with delta encoding
//!
//! # References
//!
//! - [The Unofficial XM File Format Specification](https://www.celersms.com/doc/XM_file_format.pdf)
//! - [MultimediaWiki XM Format](https://wiki.multimedia.cx/index.php/Fast_Tracker_2_Extended_Module)

mod constants;
mod error;
mod header;
mod instrument;
mod pattern;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use constants::{
    XM_FULL_HEADER_SIZE, XM_ID_TEXT, XM_MAGIC_BYTE, XM_MAX_BPM, XM_MAX_ENVELOPE_POINTS,
    XM_MAX_NOTE, XM_MAX_PANNING, XM_MAX_TEMPO, XM_MAX_VOLUME, XM_MIN_BPM, XM_MIN_CHANNELS,
    XM_MIN_FILE_SIZE, XM_MIN_NOTE, XM_MIN_PATTERN_ROWS, XM_MIN_TEMPO, XM_NOTE_OFF, XM_PACKING_FLAG,
    XM_PATTERN_HEADER_SIZE, XM_STANDARD_HEADER_SIZE, XM_VALIDATOR_MAX_CHANNELS, XM_VERSION_104,
};
pub use error::{XmFormatError, XmWarning};
pub use types::{
    XmEnvelopeInfo, XmHeaderInfo, XmInstrumentInfo, XmPatternInfo, XmSampleInfo, XmValidationReport,
};

/// XM file format validator.
pub struct XmValidator;

impl XmValidator {
    /// Validate an XM file from raw bytes.
    ///
    /// This performs comprehensive validation of all aspects of the XM format:
    /// - Header structure and magic bytes
    /// - Extended header fields
    /// - Pattern data
    /// - Instrument data including envelopes
    /// - Sample data
    ///
    /// Returns a detailed validation report.
    pub fn validate(data: &[u8]) -> Result<XmValidationReport, XmFormatError> {
        let mut report = XmValidationReport::new(data.len());

        // Phase 1: Header validation
        let header = header::validate_header(data, &mut report)?;
        report.header = Some(header.clone());

        // Phase 2: Pattern validation
        // Pattern offset is 60 + header_size (header_size is calculated FROM offset 60)
        let patterns_offset = 60 + header.header_size as usize;
        let patterns_end = pattern::validate_patterns(data, patterns_offset, &header, &mut report)?;

        // Phase 3: Instrument validation
        instrument::validate_instruments(data, patterns_end, &header, &mut report)?;

        Ok(report)
    }

    /// Quick validation that only checks header.
    pub fn validate_header_only(data: &[u8]) -> Result<XmHeaderInfo, XmFormatError> {
        let mut report = XmValidationReport::new(data.len());
        header::validate_header(data, &mut report)
    }

    /// Check if data looks like a valid XM file (quick check).
    pub fn is_xm(data: &[u8]) -> bool {
        if data.len() < 60 {
            return false;
        }
        &data[0..17] == constants::XM_ID_TEXT && data[37] == constants::XM_MAGIC_BYTE
    }
}
