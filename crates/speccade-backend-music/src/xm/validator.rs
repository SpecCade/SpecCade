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

use std::fmt;

// Re-use constants from header module to avoid duplication
use super::header::{
    XM_HEADER_SIZE, XM_MAGIC, XM_MAX_CHANNELS as XM_MAX_CHANNELS_U8, XM_MAX_INSTRUMENTS,
    XM_MAX_PATTERNS, XM_MAX_PATTERN_ROWS, XM_VERSION,
};
use super::instrument::XM_SAMPLE_HEADER_SIZE;

// Aliases for consistency within validator
/// XM ID text (alias for XM_MAGIC).
pub const XM_ID_TEXT: &[u8; 17] = XM_MAGIC;

/// XM version 1.04 (alias for XM_VERSION).
pub const XM_VERSION_104: u16 = XM_VERSION;

/// Standard header size (alias for XM_HEADER_SIZE).
pub const XM_STANDARD_HEADER_SIZE: u32 = XM_HEADER_SIZE;

/// Maximum number of channels as u16 for comparison.
pub const XM_VALIDATOR_MAX_CHANNELS: u16 = XM_MAX_CHANNELS_U8 as u16;

// Additional validation-specific constants

/// Required byte at offset 0x25 (37).
pub const XM_MAGIC_BYTE: u8 = 0x1A;

/// Minimum number of channels.
pub const XM_MIN_CHANNELS: u16 = 1;

/// Minimum pattern rows.
pub const XM_MIN_PATTERN_ROWS: u16 = 1;

/// Maximum envelope points.
pub const XM_MAX_ENVELOPE_POINTS: u8 = 12;

/// Maximum volume value.
pub const XM_MAX_VOLUME: u8 = 64;

/// Maximum panning value.
pub const XM_MAX_PANNING: u8 = 255;

/// Maximum BPM value.
pub const XM_MAX_BPM: u16 = 255;

/// Minimum BPM value.
pub const XM_MIN_BPM: u16 = 32;

/// Maximum tempo (ticks per row).
pub const XM_MAX_TEMPO: u16 = 31;

/// Minimum tempo.
pub const XM_MIN_TEMPO: u16 = 1;

/// Note off value.
pub const XM_NOTE_OFF: u8 = 97;

/// Maximum note value (excluding note-off).
pub const XM_MAX_NOTE: u8 = 96;

/// Minimum note value.
pub const XM_MIN_NOTE: u8 = 1;

/// Minimum file size for a valid XM file (header only).
pub const XM_MIN_FILE_SIZE: usize = 60;

/// Full header size including pattern order table.
pub const XM_FULL_HEADER_SIZE: usize = 336; // 60 + 276 = 336

/// Pattern header size.
pub const XM_PATTERN_HEADER_SIZE: u32 = 9;

/// Packing byte high bit.
pub const XM_PACKING_FLAG: u8 = 0x80;

/// XM format validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmFormatError {
    /// File is too small to be a valid XM file.
    FileTooSmall { size: usize, minimum: usize },
    /// Invalid ID text at offset 0.
    InvalidIdText { found: Vec<u8> },
    /// Missing magic byte 0x1A at offset 0x25.
    MissingMagicByte { found: u8, offset: usize },
    /// Unsupported XM version.
    UnsupportedVersion { version: u16 },
    /// Invalid header size.
    InvalidHeaderSize { size: u32, expected: u32 },
    /// Invalid number of channels.
    InvalidChannelCount { channels: u16, min: u16, max: u16 },
    /// Invalid number of patterns.
    InvalidPatternCount { patterns: u16, max: u16 },
    /// Invalid number of instruments.
    InvalidInstrumentCount { instruments: u16, max: u16 },
    /// Invalid song length (order count).
    InvalidSongLength { length: u16 },
    /// Restart position exceeds song length.
    InvalidRestartPosition { restart: u16, song_length: u16 },
    /// Invalid tempo value.
    InvalidTempo { tempo: u16, min: u16, max: u16 },
    /// Invalid BPM value.
    InvalidBpm { bpm: u16, min: u16, max: u16 },
    /// Pattern order references non-existent pattern.
    InvalidPatternOrder {
        position: usize,
        pattern: u8,
        max_pattern: u16,
    },
    /// Pattern header error.
    PatternError {
        pattern_index: usize,
        message: String,
    },
    /// Instrument header error.
    InstrumentError {
        instrument_index: usize,
        message: String,
    },
    /// Sample error.
    SampleError {
        instrument: usize,
        sample: usize,
        message: String,
    },
    /// Envelope error.
    EnvelopeError {
        instrument: usize,
        envelope_type: &'static str,
        message: String,
    },
    /// File truncated.
    FileTruncated { expected: usize, actual: usize },
    /// Invalid note value.
    InvalidNoteValue {
        value: u8,
        pattern: usize,
        row: usize,
        channel: usize,
    },
    /// Invalid volume value.
    InvalidVolume { value: u8, context: String },
    /// Invalid packing type.
    InvalidPackingType { found: u8, pattern: usize },
    /// Invalid sample flags.
    InvalidSampleFlags {
        flags: u8,
        instrument: usize,
        sample: usize,
    },
    /// Invalid loop type.
    InvalidLoopType {
        loop_type: u8,
        instrument: usize,
        sample: usize,
    },
    /// Loop extends beyond sample.
    InvalidLoopBounds {
        instrument: usize,
        sample: usize,
        loop_start: u32,
        loop_length: u32,
        sample_length: u32,
    },
}

impl fmt::Display for XmFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmFormatError::FileTooSmall { size, minimum } => {
                write!(
                    f,
                    "File too small: {} bytes (minimum {} required)",
                    size, minimum
                )
            }
            XmFormatError::InvalidIdText { found } => {
                write!(
                    f,
                    "Invalid ID text at offset 0: expected 'Extended Module: ', got {:?}",
                    String::from_utf8_lossy(found)
                )
            }
            XmFormatError::MissingMagicByte { found, offset } => {
                write!(
                    f,
                    "Missing magic byte at offset {}: expected 0x1A, got 0x{:02X}",
                    offset, found
                )
            }
            XmFormatError::UnsupportedVersion { version } => {
                write!(
                    f,
                    "Unsupported XM version: 0x{:04X} (expected 0x0104)",
                    version
                )
            }
            XmFormatError::InvalidHeaderSize { size, expected } => {
                write!(f, "Invalid header size: {} (expected {})", size, expected)
            }
            XmFormatError::InvalidChannelCount { channels, min, max } => {
                write!(
                    f,
                    "Invalid channel count: {} (must be {}-{})",
                    channels, min, max
                )
            }
            XmFormatError::InvalidPatternCount { patterns, max } => {
                write!(f, "Invalid pattern count: {} (maximum {})", patterns, max)
            }
            XmFormatError::InvalidInstrumentCount { instruments, max } => {
                write!(
                    f,
                    "Invalid instrument count: {} (maximum {})",
                    instruments, max
                )
            }
            XmFormatError::InvalidSongLength { length } => {
                write!(f, "Invalid song length: {} (must be 1-256)", length)
            }
            XmFormatError::InvalidRestartPosition {
                restart,
                song_length,
            } => {
                write!(
                    f,
                    "Restart position {} exceeds song length {}",
                    restart, song_length
                )
            }
            XmFormatError::InvalidTempo { tempo, min, max } => {
                write!(f, "Invalid tempo: {} (must be {}-{})", tempo, min, max)
            }
            XmFormatError::InvalidBpm { bpm, min, max } => {
                write!(f, "Invalid BPM: {} (must be {}-{})", bpm, min, max)
            }
            XmFormatError::InvalidPatternOrder {
                position,
                pattern,
                max_pattern,
            } => {
                write!(
                    f,
                    "Pattern order[{}] = {} exceeds pattern count {}",
                    position, pattern, max_pattern
                )
            }
            XmFormatError::PatternError {
                pattern_index,
                message,
            } => {
                write!(f, "Pattern {} error: {}", pattern_index, message)
            }
            XmFormatError::InstrumentError {
                instrument_index,
                message,
            } => {
                write!(f, "Instrument {} error: {}", instrument_index, message)
            }
            XmFormatError::SampleError {
                instrument,
                sample,
                message,
            } => {
                write!(
                    f,
                    "Instrument {} sample {} error: {}",
                    instrument, sample, message
                )
            }
            XmFormatError::EnvelopeError {
                instrument,
                envelope_type,
                message,
            } => {
                write!(
                    f,
                    "Instrument {} {} envelope error: {}",
                    instrument, envelope_type, message
                )
            }
            XmFormatError::FileTruncated { expected, actual } => {
                write!(
                    f,
                    "File truncated: expected {} bytes, got {}",
                    expected, actual
                )
            }
            XmFormatError::InvalidNoteValue {
                value,
                pattern,
                row,
                channel,
            } => {
                write!(
                    f,
                    "Invalid note value {} at pattern {}, row {}, channel {}",
                    value, pattern, row, channel
                )
            }
            XmFormatError::InvalidVolume { value, context } => {
                write!(f, "Invalid volume {} in {}", value, context)
            }
            XmFormatError::InvalidPackingType { found, pattern } => {
                write!(
                    f,
                    "Invalid packing type {} in pattern {} (must be 0)",
                    found, pattern
                )
            }
            XmFormatError::InvalidSampleFlags {
                flags,
                instrument,
                sample,
            } => {
                write!(
                    f,
                    "Invalid sample flags 0x{:02X} in instrument {} sample {}",
                    flags, instrument, sample
                )
            }
            XmFormatError::InvalidLoopType {
                loop_type,
                instrument,
                sample,
            } => {
                write!(
                    f,
                    "Invalid loop type {} in instrument {} sample {} (must be 0-2)",
                    loop_type, instrument, sample
                )
            }
            XmFormatError::InvalidLoopBounds {
                instrument,
                sample,
                loop_start,
                loop_length,
                sample_length,
            } => {
                write!(f, "Invalid loop bounds in instrument {} sample {}: start {} + length {} > sample length {}",
                    instrument, sample, loop_start, loop_length, sample_length)
            }
        }
    }
}

impl std::error::Error for XmFormatError {}

/// Warning type for non-critical issues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmWarning {
    /// Non-standard header size (but acceptable).
    NonStandardHeaderSize { size: u32, standard: u32 },
    /// Unusual tempo/BPM values.
    UnusualTempoBpm { tempo: u16, bpm: u16 },
    /// Empty pattern.
    EmptyPattern { pattern_index: usize },
    /// Instrument with no samples.
    InstrumentWithoutSamples { instrument_index: usize },
    /// Envelope with zero points but enabled.
    EmptyEnabledEnvelope {
        instrument: usize,
        envelope_type: &'static str,
    },
    /// Non-standard tracker name.
    NonStandardTrackerName { name: String },
    /// High envelope frame values (compatible but unusual).
    HighEnvelopeFrameValue {
        instrument: usize,
        envelope_type: &'static str,
        frame: u16,
    },
    /// Sample with unusual panning.
    UnusualSamplePanning {
        instrument: usize,
        sample: usize,
        panning: u8,
    },
}

impl fmt::Display for XmWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmWarning::NonStandardHeaderSize { size, standard } => {
                write!(
                    f,
                    "Non-standard header size {} (standard: {})",
                    size, standard
                )
            }
            XmWarning::UnusualTempoBpm { tempo, bpm } => {
                write!(f, "Unusual tempo/BPM combination: {}/{}", tempo, bpm)
            }
            XmWarning::EmptyPattern { pattern_index } => {
                write!(f, "Pattern {} is empty", pattern_index)
            }
            XmWarning::InstrumentWithoutSamples { instrument_index } => {
                write!(f, "Instrument {} has no samples", instrument_index)
            }
            XmWarning::EmptyEnabledEnvelope {
                instrument,
                envelope_type,
            } => {
                write!(
                    f,
                    "Instrument {} has {} envelope enabled but zero points",
                    instrument, envelope_type
                )
            }
            XmWarning::NonStandardTrackerName { name } => {
                write!(f, "Non-standard tracker name: '{}'", name)
            }
            XmWarning::HighEnvelopeFrameValue {
                instrument,
                envelope_type,
                frame,
            } => {
                write!(
                    f,
                    "Instrument {} {} envelope has high frame value {}",
                    instrument, envelope_type, frame
                )
            }
            XmWarning::UnusualSamplePanning {
                instrument,
                sample,
                panning,
            } => {
                write!(
                    f,
                    "Instrument {} sample {} has unusual panning: {}",
                    instrument, sample, panning
                )
            }
        }
    }
}

/// Detailed information extracted from XM header.
#[derive(Debug, Clone)]
pub struct XmHeaderInfo {
    /// Module name (up to 20 characters).
    pub name: String,
    /// Tracker name (up to 20 characters).
    pub tracker_name: String,
    /// XM format version.
    pub version: u16,
    /// Header size (from offset 60).
    pub header_size: u32,
    /// Song length (number of pattern positions).
    pub song_length: u16,
    /// Restart position.
    pub restart_position: u16,
    /// Number of channels.
    pub num_channels: u16,
    /// Number of patterns.
    pub num_patterns: u16,
    /// Number of instruments.
    pub num_instruments: u16,
    /// Flags (bit 0: frequency table type).
    pub flags: u16,
    /// Uses linear frequency table (vs Amiga).
    pub linear_frequency_table: bool,
    /// Default tempo (ticks per row).
    pub default_tempo: u16,
    /// Default BPM.
    pub default_bpm: u16,
    /// Pattern order table (256 entries).
    pub pattern_order: [u8; 256],
}

/// Information about a pattern.
#[derive(Debug, Clone)]
pub struct XmPatternInfo {
    /// Pattern index.
    pub index: usize,
    /// Header length.
    pub header_length: u32,
    /// Packing type (should be 0).
    pub packing_type: u8,
    /// Number of rows.
    pub num_rows: u16,
    /// Packed data size.
    pub packed_size: u16,
    /// Is pattern empty.
    pub is_empty: bool,
}

/// Information about an envelope.
#[derive(Debug, Clone)]
pub struct XmEnvelopeInfo {
    /// Envelope type ("volume" or "panning").
    pub envelope_type: &'static str,
    /// Number of points.
    pub num_points: u8,
    /// Sustain point index.
    pub sustain_point: u8,
    /// Loop start point.
    pub loop_start: u8,
    /// Loop end point.
    pub loop_end: u8,
    /// Envelope enabled.
    pub enabled: bool,
    /// Sustain enabled.
    pub sustain_enabled: bool,
    /// Loop enabled.
    pub loop_enabled: bool,
    /// Envelope points (frame, value).
    pub points: Vec<(u16, u16)>,
}

/// Information about a sample.
#[derive(Debug, Clone)]
pub struct XmSampleInfo {
    /// Sample index within instrument.
    pub index: usize,
    /// Sample name (up to 22 characters).
    pub name: String,
    /// Sample length in bytes.
    pub length: u32,
    /// Loop start position.
    pub loop_start: u32,
    /// Loop length.
    pub loop_length: u32,
    /// Default volume (0-64).
    pub volume: u8,
    /// Finetune (-128 to +127).
    pub finetune: i8,
    /// Sample type/flags.
    pub flags: u8,
    /// Is 16-bit sample.
    pub is_16bit: bool,
    /// Loop type (0=none, 1=forward, 2=ping-pong).
    pub loop_type: u8,
    /// Panning (0-255).
    pub panning: u8,
    /// Relative note number.
    pub relative_note: i8,
}

/// Information about an instrument.
#[derive(Debug, Clone)]
pub struct XmInstrumentInfo {
    /// Instrument index (1-based in file, 0-based here).
    pub index: usize,
    /// Instrument name (up to 22 characters).
    pub name: String,
    /// Instrument type (should be 0).
    pub instrument_type: u8,
    /// Number of samples.
    pub num_samples: u16,
    /// Sample header size.
    pub sample_header_size: u32,
    /// Volume envelope info.
    pub volume_envelope: XmEnvelopeInfo,
    /// Panning envelope info.
    pub panning_envelope: XmEnvelopeInfo,
    /// Vibrato type.
    pub vibrato_type: u8,
    /// Vibrato sweep.
    pub vibrato_sweep: u8,
    /// Vibrato depth.
    pub vibrato_depth: u8,
    /// Vibrato rate.
    pub vibrato_rate: u8,
    /// Volume fadeout.
    pub volume_fadeout: u16,
    /// Sample information.
    pub samples: Vec<XmSampleInfo>,
}

/// Complete XM validation report.
#[derive(Debug, Clone)]
pub struct XmValidationReport {
    /// Whether the file is valid.
    pub valid: bool,
    /// Header information (if parsed successfully).
    pub header: Option<XmHeaderInfo>,
    /// Pattern information.
    pub patterns: Vec<XmPatternInfo>,
    /// Instrument information.
    pub instruments: Vec<XmInstrumentInfo>,
    /// Validation errors.
    pub errors: Vec<XmFormatError>,
    /// Validation warnings.
    pub warnings: Vec<XmWarning>,
    /// Total file size.
    pub file_size: usize,
    /// Calculated expected file size.
    pub expected_size: usize,
}

impl XmValidationReport {
    /// Create a new empty report.
    fn new(file_size: usize) -> Self {
        Self {
            valid: true,
            header: None,
            patterns: Vec::new(),
            instruments: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            file_size,
            expected_size: 0,
        }
    }

    /// Add an error to the report.
    fn add_error(&mut self, error: XmFormatError) {
        self.errors.push(error);
        self.valid = false;
    }

    /// Add a warning to the report.
    fn add_warning(&mut self, warning: XmWarning) {
        self.warnings.push(warning);
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

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
        let header = Self::validate_header(data, &mut report)?;
        report.header = Some(header.clone());

        // Phase 2: Pattern validation
        // Pattern offset is 60 + header_size (header_size is calculated FROM offset 60)
        let patterns_offset = 60 + header.header_size as usize;
        let patterns_end = Self::validate_patterns(data, patterns_offset, &header, &mut report)?;

        // Phase 3: Instrument validation
        Self::validate_instruments(data, patterns_end, &header, &mut report)?;

        Ok(report)
    }

    /// Validate the XM header (offsets 0x00-0x3C and extended header).
    fn validate_header(
        data: &[u8],
        report: &mut XmValidationReport,
    ) -> Result<XmHeaderInfo, XmFormatError> {
        // Check minimum file size
        if data.len() < XM_MIN_FILE_SIZE {
            return Err(XmFormatError::FileTooSmall {
                size: data.len(),
                minimum: XM_MIN_FILE_SIZE,
            });
        }

        // Offset 0x00-0x10: ID text "Extended Module: "
        if &data[0..17] != XM_ID_TEXT {
            return Err(XmFormatError::InvalidIdText {
                found: data[0..17].to_vec(),
            });
        }

        // Offset 0x11-0x24: Module name (20 bytes)
        let name = extract_string(&data[17..37]);

        // Offset 0x25: Magic byte 0x1A
        if data[37] != XM_MAGIC_BYTE {
            return Err(XmFormatError::MissingMagicByte {
                found: data[37],
                offset: 37,
            });
        }

        // Offset 0x26-0x39: Tracker name (20 bytes)
        let tracker_name = extract_string(&data[38..58]);

        // Check for non-standard tracker name
        if !tracker_name.starts_with("FastTracker") && !tracker_name.is_empty() {
            report.add_warning(XmWarning::NonStandardTrackerName {
                name: tracker_name.clone(),
            });
        }

        // Offset 0x3A-0x3B: Version number
        let version = u16::from_le_bytes([data[58], data[59]]);
        if version != XM_VERSION_104 {
            // We'll accept it but log if it's not 1.04
            if !(0x0100..=0x0104).contains(&version) {
                return Err(XmFormatError::UnsupportedVersion { version });
            }
        }

        // Check if we have enough data for extended header
        if data.len() < XM_FULL_HEADER_SIZE {
            return Err(XmFormatError::FileTooSmall {
                size: data.len(),
                minimum: XM_FULL_HEADER_SIZE,
            });
        }

        // Offset 0x3C-0x3F: Header size (from offset 60)
        let header_size = u32::from_le_bytes([data[60], data[61], data[62], data[63]]);
        if header_size != XM_STANDARD_HEADER_SIZE {
            report.add_warning(XmWarning::NonStandardHeaderSize {
                size: header_size,
                standard: XM_STANDARD_HEADER_SIZE,
            });
        }

        // Extended header fields (offsets are relative to offset 60)
        // Song length at offset 64-65
        let song_length = u16::from_le_bytes([data[64], data[65]]);
        if song_length == 0 || song_length > 256 {
            return Err(XmFormatError::InvalidSongLength {
                length: song_length,
            });
        }

        // Restart position at offset 66-67
        let restart_position = u16::from_le_bytes([data[66], data[67]]);
        if restart_position >= song_length {
            return Err(XmFormatError::InvalidRestartPosition {
                restart: restart_position,
                song_length,
            });
        }

        // Number of channels at offset 68-69
        let num_channels = u16::from_le_bytes([data[68], data[69]]);
        if !(XM_MIN_CHANNELS..=XM_VALIDATOR_MAX_CHANNELS).contains(&num_channels) {
            return Err(XmFormatError::InvalidChannelCount {
                channels: num_channels,
                min: XM_MIN_CHANNELS,
                max: XM_VALIDATOR_MAX_CHANNELS,
            });
        }

        // Number of patterns at offset 70-71
        let num_patterns = u16::from_le_bytes([data[70], data[71]]);
        if num_patterns > XM_MAX_PATTERNS {
            return Err(XmFormatError::InvalidPatternCount {
                patterns: num_patterns,
                max: XM_MAX_PATTERNS,
            });
        }

        // Number of instruments at offset 72-73
        let num_instruments = u16::from_le_bytes([data[72], data[73]]);
        if num_instruments > XM_MAX_INSTRUMENTS {
            return Err(XmFormatError::InvalidInstrumentCount {
                instruments: num_instruments,
                max: XM_MAX_INSTRUMENTS,
            });
        }

        // Flags at offset 74-75
        let flags = u16::from_le_bytes([data[74], data[75]]);
        let linear_frequency_table = (flags & 1) != 0;

        // Default tempo at offset 76-77
        let default_tempo = u16::from_le_bytes([data[76], data[77]]);
        if !(XM_MIN_TEMPO..=XM_MAX_TEMPO).contains(&default_tempo) {
            return Err(XmFormatError::InvalidTempo {
                tempo: default_tempo,
                min: XM_MIN_TEMPO,
                max: XM_MAX_TEMPO,
            });
        }

        // Default BPM at offset 78-79
        let default_bpm = u16::from_le_bytes([data[78], data[79]]);
        if !(XM_MIN_BPM..=XM_MAX_BPM).contains(&default_bpm) {
            return Err(XmFormatError::InvalidBpm {
                bpm: default_bpm,
                min: XM_MIN_BPM,
                max: XM_MAX_BPM,
            });
        }

        // Check for unusual tempo/BPM combinations
        if default_tempo > 20 || !(50..=200).contains(&default_bpm) {
            report.add_warning(XmWarning::UnusualTempoBpm {
                tempo: default_tempo,
                bpm: default_bpm,
            });
        }

        // Pattern order table at offset 80-335 (256 bytes)
        let mut pattern_order = [0u8; 256];
        pattern_order.copy_from_slice(&data[80..336]);

        // Validate pattern order entries
        for (i, pattern) in pattern_order
            .iter()
            .copied()
            .take(song_length as usize)
            .enumerate()
        {
            if pattern as u16 >= num_patterns && num_patterns > 0 {
                return Err(XmFormatError::InvalidPatternOrder {
                    position: i,
                    pattern,
                    max_pattern: num_patterns,
                });
            }
        }

        Ok(XmHeaderInfo {
            name,
            tracker_name,
            version,
            header_size,
            song_length,
            restart_position,
            num_channels,
            num_patterns,
            num_instruments,
            flags,
            linear_frequency_table,
            default_tempo,
            default_bpm,
            pattern_order,
        })
    }

    /// Validate all patterns and return the offset after the last pattern.
    fn validate_patterns(
        data: &[u8],
        start_offset: usize,
        header: &XmHeaderInfo,
        report: &mut XmValidationReport,
    ) -> Result<usize, XmFormatError> {
        let mut offset = start_offset;

        for pattern_idx in 0..header.num_patterns as usize {
            // Check if we have enough data for pattern header
            if offset + 9 > data.len() {
                return Err(XmFormatError::FileTruncated {
                    expected: offset + 9,
                    actual: data.len(),
                });
            }

            // Pattern header length (4 bytes)
            let header_length = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);

            // Packing type (1 byte, should be 0)
            let packing_type = data[offset + 4];
            if packing_type != 0 {
                return Err(XmFormatError::InvalidPackingType {
                    found: packing_type,
                    pattern: pattern_idx,
                });
            }

            // Number of rows (2 bytes)
            let num_rows = u16::from_le_bytes([data[offset + 5], data[offset + 6]]);
            if !(XM_MIN_PATTERN_ROWS..=XM_MAX_PATTERN_ROWS).contains(&num_rows) {
                return Err(XmFormatError::PatternError {
                    pattern_index: pattern_idx,
                    message: format!(
                        "Invalid row count {} (must be {}-{})",
                        num_rows, XM_MIN_PATTERN_ROWS, XM_MAX_PATTERN_ROWS
                    ),
                });
            }

            // Packed pattern data size (2 bytes)
            let packed_size = u16::from_le_bytes([data[offset + 7], data[offset + 8]]);

            let is_empty = packed_size == 0;
            if is_empty {
                report.add_warning(XmWarning::EmptyPattern {
                    pattern_index: pattern_idx,
                });
            }

            // Validate pattern data if not empty
            if packed_size > 0 {
                let data_offset = offset + header_length as usize;
                let data_end = data_offset + packed_size as usize;

                if data_end > data.len() {
                    return Err(XmFormatError::FileTruncated {
                        expected: data_end,
                        actual: data.len(),
                    });
                }

                // Validate packed pattern data
                Self::validate_pattern_data(
                    &data[data_offset..data_end],
                    pattern_idx,
                    num_rows,
                    header.num_channels,
                    report,
                )?;
            }

            report.patterns.push(XmPatternInfo {
                index: pattern_idx,
                header_length,
                packing_type,
                num_rows,
                packed_size,
                is_empty,
            });

            // Move to next pattern
            offset += header_length as usize + packed_size as usize;
        }

        Ok(offset)
    }

    /// Validate packed pattern data.
    fn validate_pattern_data(
        data: &[u8],
        pattern_idx: usize,
        num_rows: u16,
        num_channels: u16,
        report: &mut XmValidationReport,
    ) -> Result<(), XmFormatError> {
        let mut pos = 0;
        let mut row = 0;
        let mut channel = 0;

        while pos < data.len() && row < num_rows {
            let byte = data[pos];
            pos += 1;

            // Check if this is a packing byte or a note value
            if byte & XM_PACKING_FLAG != 0 {
                // Packing byte: extract flags
                let has_note = (byte & 0x01) != 0;
                let has_instrument = (byte & 0x02) != 0;
                let has_volume = (byte & 0x04) != 0;
                let has_effect = (byte & 0x08) != 0;
                let has_effect_param = (byte & 0x10) != 0;

                // Validate note if present
                if has_note {
                    if pos >= data.len() {
                        return Err(XmFormatError::FileTruncated {
                            expected: pos + 1,
                            actual: data.len(),
                        });
                    }
                    let note = data[pos];
                    pos += 1;

                    // Note must be 0 (none), 1-96 (C-0 to B-7), or 97 (key off)
                    if note != 0 && (note > XM_NOTE_OFF) {
                        report.add_error(XmFormatError::InvalidNoteValue {
                            value: note,
                            pattern: pattern_idx,
                            row: row as usize,
                            channel: channel as usize,
                        });
                    }
                }

                // Skip instrument if present
                if has_instrument {
                    if pos >= data.len() {
                        return Err(XmFormatError::FileTruncated {
                            expected: pos + 1,
                            actual: data.len(),
                        });
                    }
                    pos += 1;
                }

                // Validate volume if present
                if has_volume {
                    if pos >= data.len() {
                        return Err(XmFormatError::FileTruncated {
                            expected: pos + 1,
                            actual: data.len(),
                        });
                    }
                    let _volume = data[pos];
                    pos += 1;
                    // Volume column can have various commands, so we just validate it exists
                }

                // Skip effect if present
                if has_effect {
                    if pos >= data.len() {
                        return Err(XmFormatError::FileTruncated {
                            expected: pos + 1,
                            actual: data.len(),
                        });
                    }
                    pos += 1;
                }

                // Skip effect parameter if present
                if has_effect_param {
                    if pos >= data.len() {
                        return Err(XmFormatError::FileTruncated {
                            expected: pos + 1,
                            actual: data.len(),
                        });
                    }
                    pos += 1;
                }
            } else {
                // Uncompressed note: byte is the note value itself
                // This path is less common in properly packed XM files
                // We need 4 more bytes: instrument, volume, effect, effect_param
                if pos + 4 > data.len() {
                    return Err(XmFormatError::FileTruncated {
                        expected: pos + 4,
                        actual: data.len(),
                    });
                }

                // Validate note
                if byte != 0 && byte > XM_NOTE_OFF {
                    report.add_error(XmFormatError::InvalidNoteValue {
                        value: byte,
                        pattern: pattern_idx,
                        row: row as usize,
                        channel: channel as usize,
                    });
                }

                pos += 4; // Skip instrument, volume, effect, effect_param
            }

            // Move to next channel
            channel += 1;
            if channel >= num_channels {
                channel = 0;
                row += 1;
            }
        }

        Ok(())
    }

    /// Validate all instruments.
    fn validate_instruments(
        data: &[u8],
        start_offset: usize,
        header: &XmHeaderInfo,
        report: &mut XmValidationReport,
    ) -> Result<(), XmFormatError> {
        let mut offset = start_offset;

        for inst_idx in 0..header.num_instruments as usize {
            if offset + 4 > data.len() {
                return Err(XmFormatError::FileTruncated {
                    expected: offset + 4,
                    actual: data.len(),
                });
            }

            // Instrument header size
            let inst_size = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);

            if offset + inst_size as usize > data.len() {
                return Err(XmFormatError::FileTruncated {
                    expected: offset + inst_size as usize,
                    actual: data.len(),
                });
            }

            // Parse instrument header
            let inst_info = Self::parse_instrument_header(data, offset, inst_idx, report)?;

            // Store sample data sizes for offset calculation
            let mut total_sample_data_size: usize = 0;

            // Validate samples
            if inst_info.num_samples > 0 {
                let sample_headers_offset = offset + inst_size as usize;
                let sample_header_size = inst_info.sample_header_size;

                for sample_idx in 0..inst_info.num_samples as usize {
                    let sample_offset =
                        sample_headers_offset + sample_idx * sample_header_size as usize;

                    if sample_offset + sample_header_size as usize > data.len() {
                        return Err(XmFormatError::FileTruncated {
                            expected: sample_offset + sample_header_size as usize,
                            actual: data.len(),
                        });
                    }

                    let sample_info = Self::parse_sample_header(
                        data,
                        sample_offset,
                        inst_idx,
                        sample_idx,
                        report,
                    )?;
                    total_sample_data_size += sample_info.length as usize;

                    // Create a mutable copy of inst_info for samples
                    // Note: We'll add samples in a collected manner below
                }

                // Skip sample data
                let sample_data_offset = sample_headers_offset
                    + inst_info.num_samples as usize * sample_header_size as usize;
                offset = sample_data_offset + total_sample_data_size;
            } else {
                report.add_warning(XmWarning::InstrumentWithoutSamples {
                    instrument_index: inst_idx,
                });
                offset += inst_size as usize;
            }

            report.instruments.push(inst_info);
        }

        report.expected_size = offset;
        Ok(())
    }

    /// Parse instrument header and return instrument info.
    fn parse_instrument_header(
        data: &[u8],
        offset: usize,
        inst_idx: usize,
        report: &mut XmValidationReport,
    ) -> Result<XmInstrumentInfo, XmFormatError> {
        let inst_size = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        // Instrument name (22 bytes at offset +4)
        let name = if offset + 26 <= data.len() {
            extract_string(&data[offset + 4..offset + 26])
        } else {
            String::new()
        };

        // Instrument type at offset +26 (should be 0)
        let instrument_type = if offset + 27 <= data.len() {
            data[offset + 26]
        } else {
            0
        };

        // Number of samples at offset +27
        let num_samples = if offset + 29 <= data.len() {
            u16::from_le_bytes([data[offset + 27], data[offset + 28]])
        } else {
            0
        };

        // Sample header size at offset +29 (only if num_samples > 0)
        let sample_header_size = if num_samples > 0 && offset + 33 <= data.len() {
            u32::from_le_bytes([
                data[offset + 29],
                data[offset + 30],
                data[offset + 31],
                data[offset + 32],
            ])
        } else {
            XM_SAMPLE_HEADER_SIZE
        };

        // Parse envelopes and other data only if we have samples
        let (volume_envelope, panning_envelope) = if num_samples > 0 && inst_size >= 243 {
            Self::parse_envelopes(data, offset, inst_idx, report)?
        } else {
            (
                XmEnvelopeInfo {
                    envelope_type: "volume",
                    num_points: 0,
                    sustain_point: 0,
                    loop_start: 0,
                    loop_end: 0,
                    enabled: false,
                    sustain_enabled: false,
                    loop_enabled: false,
                    points: Vec::new(),
                },
                XmEnvelopeInfo {
                    envelope_type: "panning",
                    num_points: 0,
                    sustain_point: 0,
                    loop_start: 0,
                    loop_end: 0,
                    enabled: false,
                    sustain_enabled: false,
                    loop_enabled: false,
                    points: Vec::new(),
                },
            )
        };

        // Vibrato parameters
        let vibrato_type = if num_samples > 0 && offset + 236 <= data.len() {
            data[offset + 235]
        } else {
            0
        };
        let vibrato_sweep = if num_samples > 0 && offset + 237 <= data.len() {
            data[offset + 236]
        } else {
            0
        };
        let vibrato_depth = if num_samples > 0 && offset + 238 <= data.len() {
            data[offset + 237]
        } else {
            0
        };
        let vibrato_rate = if num_samples > 0 && offset + 239 <= data.len() {
            data[offset + 238]
        } else {
            0
        };

        // Volume fadeout
        let volume_fadeout = if num_samples > 0 && offset + 241 <= data.len() {
            u16::from_le_bytes([data[offset + 239], data[offset + 240]])
        } else {
            0
        };

        Ok(XmInstrumentInfo {
            index: inst_idx,
            name,
            instrument_type,
            num_samples,
            sample_header_size,
            volume_envelope,
            panning_envelope,
            vibrato_type,
            vibrato_sweep,
            vibrato_depth,
            vibrato_rate,
            volume_fadeout,
            samples: Vec::new(), // Filled later
        })
    }

    /// Parse volume and panning envelopes.
    fn parse_envelopes(
        data: &[u8],
        offset: usize,
        inst_idx: usize,
        report: &mut XmValidationReport,
    ) -> Result<(XmEnvelopeInfo, XmEnvelopeInfo), XmFormatError> {
        // Volume envelope points: offset + 33, 48 bytes (12 points * 4 bytes)
        let vol_points_offset = offset + 33;
        // Panning envelope points: offset + 81 (0x51), 48 bytes
        let pan_points_offset = offset + 129;

        // Number of envelope points
        let num_vol_points = data[offset + 225];
        let num_pan_points = data[offset + 226];

        // Validate point counts
        if num_vol_points > XM_MAX_ENVELOPE_POINTS {
            return Err(XmFormatError::EnvelopeError {
                instrument: inst_idx,
                envelope_type: "volume",
                message: format!(
                    "Too many points: {} (max {})",
                    num_vol_points, XM_MAX_ENVELOPE_POINTS
                ),
            });
        }
        if num_pan_points > XM_MAX_ENVELOPE_POINTS {
            return Err(XmFormatError::EnvelopeError {
                instrument: inst_idx,
                envelope_type: "panning",
                message: format!(
                    "Too many points: {} (max {})",
                    num_pan_points, XM_MAX_ENVELOPE_POINTS
                ),
            });
        }

        // Sustain/loop points
        let vol_sustain = data[offset + 227];
        let vol_loop_start = data[offset + 228];
        let vol_loop_end = data[offset + 229];
        let pan_sustain = data[offset + 230];
        let pan_loop_start = data[offset + 231];
        let pan_loop_end = data[offset + 232];

        // Envelope flags
        let vol_flags = data[offset + 233];
        let pan_flags = data[offset + 234];

        let vol_enabled = (vol_flags & 1) != 0;
        let vol_sustain_enabled = (vol_flags & 2) != 0;
        let vol_loop_enabled = (vol_flags & 4) != 0;

        let pan_enabled = (pan_flags & 1) != 0;
        let pan_sustain_enabled = (pan_flags & 2) != 0;
        let pan_loop_enabled = (pan_flags & 4) != 0;

        // Warn if envelope is enabled but has no points
        if vol_enabled && num_vol_points == 0 {
            report.add_warning(XmWarning::EmptyEnabledEnvelope {
                instrument: inst_idx,
                envelope_type: "volume",
            });
        }
        if pan_enabled && num_pan_points == 0 {
            report.add_warning(XmWarning::EmptyEnabledEnvelope {
                instrument: inst_idx,
                envelope_type: "panning",
            });
        }

        // Parse envelope points
        let mut vol_points = Vec::new();
        for i in 0..num_vol_points as usize {
            let pt_offset = vol_points_offset + i * 4;
            if pt_offset + 4 <= data.len() {
                let frame = u16::from_le_bytes([data[pt_offset], data[pt_offset + 1]]);
                let value = u16::from_le_bytes([data[pt_offset + 2], data[pt_offset + 3]]);

                // Validate value range
                if value > 64 {
                    report.add_error(XmFormatError::EnvelopeError {
                        instrument: inst_idx,
                        envelope_type: "volume",
                        message: format!("Point {} value {} exceeds maximum 64", i, value),
                    });
                }

                // Warn about high frame values
                if frame > 512 {
                    report.add_warning(XmWarning::HighEnvelopeFrameValue {
                        instrument: inst_idx,
                        envelope_type: "volume",
                        frame,
                    });
                }

                vol_points.push((frame, value));
            }
        }

        let mut pan_points = Vec::new();
        for i in 0..num_pan_points as usize {
            let pt_offset = pan_points_offset + i * 4;
            if pt_offset + 4 <= data.len() {
                let frame = u16::from_le_bytes([data[pt_offset], data[pt_offset + 1]]);
                let value = u16::from_le_bytes([data[pt_offset + 2], data[pt_offset + 3]]);

                // Panning envelope values should be 0-64 centered at 32
                if value > 64 {
                    report.add_error(XmFormatError::EnvelopeError {
                        instrument: inst_idx,
                        envelope_type: "panning",
                        message: format!("Point {} value {} exceeds maximum 64", i, value),
                    });
                }

                pan_points.push((frame, value));
            }
        }

        // Validate sustain and loop point indices
        if vol_sustain_enabled && num_vol_points > 0 && vol_sustain >= num_vol_points {
            report.add_error(XmFormatError::EnvelopeError {
                instrument: inst_idx,
                envelope_type: "volume",
                message: format!(
                    "Sustain point {} exceeds point count {}",
                    vol_sustain, num_vol_points
                ),
            });
        }
        if vol_loop_enabled
            && num_vol_points > 0
            && (vol_loop_start >= num_vol_points
                || vol_loop_end >= num_vol_points
                || vol_loop_start > vol_loop_end)
        {
            report.add_error(XmFormatError::EnvelopeError {
                instrument: inst_idx,
                envelope_type: "volume",
                message: format!(
                    "Invalid loop points: start={}, end={}, points={}",
                    vol_loop_start, vol_loop_end, num_vol_points
                ),
            });
        }

        Ok((
            XmEnvelopeInfo {
                envelope_type: "volume",
                num_points: num_vol_points,
                sustain_point: vol_sustain,
                loop_start: vol_loop_start,
                loop_end: vol_loop_end,
                enabled: vol_enabled,
                sustain_enabled: vol_sustain_enabled,
                loop_enabled: vol_loop_enabled,
                points: vol_points,
            },
            XmEnvelopeInfo {
                envelope_type: "panning",
                num_points: num_pan_points,
                sustain_point: pan_sustain,
                loop_start: pan_loop_start,
                loop_end: pan_loop_end,
                enabled: pan_enabled,
                sustain_enabled: pan_sustain_enabled,
                loop_enabled: pan_loop_enabled,
                points: pan_points,
            },
        ))
    }

    /// Parse sample header.
    fn parse_sample_header(
        data: &[u8],
        offset: usize,
        inst_idx: usize,
        sample_idx: usize,
        report: &mut XmValidationReport,
    ) -> Result<XmSampleInfo, XmFormatError> {
        // Sample length (4 bytes)
        let length = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        // Loop start (4 bytes)
        let loop_start = u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);

        // Loop length (4 bytes)
        let loop_length = u32::from_le_bytes([
            data[offset + 8],
            data[offset + 9],
            data[offset + 10],
            data[offset + 11],
        ]);

        // Volume (1 byte, 0-64)
        let volume = data[offset + 12];
        if volume > XM_MAX_VOLUME {
            report.add_error(XmFormatError::InvalidVolume {
                value: volume,
                context: format!("instrument {} sample {}", inst_idx, sample_idx),
            });
        }

        // Finetune (1 signed byte)
        let finetune = data[offset + 13] as i8;

        // Type/flags (1 byte)
        let flags = data[offset + 14];
        let loop_type = flags & 0x03;
        let is_16bit = (flags & 0x10) != 0;

        // Validate loop type
        if loop_type > 2 {
            return Err(XmFormatError::InvalidLoopType {
                loop_type,
                instrument: inst_idx,
                sample: sample_idx,
            });
        }

        // Validate loop bounds
        if loop_type != 0 && length > 0 && loop_start + loop_length > length {
            return Err(XmFormatError::InvalidLoopBounds {
                instrument: inst_idx,
                sample: sample_idx,
                loop_start,
                loop_length,
                sample_length: length,
            });
        }

        // Panning (1 byte, 0-255)
        let panning = data[offset + 15];

        // Relative note (1 signed byte)
        let relative_note = data[offset + 16] as i8;

        // Reserved byte at offset + 17

        // Sample name (22 bytes at offset + 18)
        let name = if offset + 40 <= data.len() {
            extract_string(&data[offset + 18..offset + 40])
        } else {
            String::new()
        };

        Ok(XmSampleInfo {
            index: sample_idx,
            name,
            length,
            loop_start,
            loop_length,
            volume,
            finetune,
            flags,
            is_16bit,
            loop_type,
            panning,
            relative_note,
        })
    }

    /// Quick validation that only checks header.
    pub fn validate_header_only(data: &[u8]) -> Result<XmHeaderInfo, XmFormatError> {
        let mut report = XmValidationReport::new(data.len());
        Self::validate_header(data, &mut report)
    }

    /// Check if data looks like a valid XM file (quick check).
    pub fn is_xm(data: &[u8]) -> bool {
        if data.len() < 60 {
            return false;
        }
        &data[0..17] == XM_ID_TEXT && data[37] == XM_MAGIC_BYTE
    }
}

/// Extract a null-terminated or space-padded string from a byte slice.
fn extract_string(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..end]).trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a minimal valid XM header for testing.
    fn create_minimal_xm(
        name: &str,
        channels: u16,
        patterns: u16,
        instruments: u16,
        tempo: u16,
        bpm: u16,
    ) -> Vec<u8> {
        let mut data = vec![0u8; XM_FULL_HEADER_SIZE];

        // ID text
        data[0..17].copy_from_slice(XM_ID_TEXT);

        // Module name
        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(20);
        data[17..17 + copy_len].copy_from_slice(&name_bytes[..copy_len]);

        // Magic byte
        data[37] = XM_MAGIC_BYTE;

        // Tracker name
        data[38..58].copy_from_slice(b"FastTracker v2.00   ");

        // Version
        data[58..60].copy_from_slice(&XM_VERSION_104.to_le_bytes());

        // Header size
        data[60..64].copy_from_slice(&XM_STANDARD_HEADER_SIZE.to_le_bytes());

        // Song length
        data[64..66].copy_from_slice(&1u16.to_le_bytes());

        // Restart position
        data[66..68].copy_from_slice(&0u16.to_le_bytes());

        // Number of channels
        data[68..70].copy_from_slice(&channels.to_le_bytes());

        // Number of patterns
        data[70..72].copy_from_slice(&patterns.to_le_bytes());

        // Number of instruments
        data[72..74].copy_from_slice(&instruments.to_le_bytes());

        // Flags
        data[74..76].copy_from_slice(&1u16.to_le_bytes());

        // Tempo
        data[76..78].copy_from_slice(&tempo.to_le_bytes());

        // BPM
        data[78..80].copy_from_slice(&bpm.to_le_bytes());

        data
    }

    /// Add an empty pattern to XM data.
    fn add_empty_pattern(data: &mut Vec<u8>, rows: u16) {
        // Pattern header length
        data.extend_from_slice(&9u32.to_le_bytes());
        // Packing type
        data.push(0);
        // Number of rows
        data.extend_from_slice(&rows.to_le_bytes());
        // Packed size (0 for empty)
        data.extend_from_slice(&0u16.to_le_bytes());
    }

    #[test]
    fn test_xm_header_format() {
        let data = create_minimal_xm("Test Song", 8, 1, 0, 6, 125);
        add_empty_pattern(&mut data.clone(), 64);

        let result = XmValidator::validate_header_only(&data);
        assert!(result.is_ok());

        let header = result.unwrap();
        assert_eq!(header.name, "Test Song");
        assert_eq!(header.num_channels, 8);
        assert_eq!(header.num_patterns, 1);
        assert_eq!(header.num_instruments, 0);
        assert_eq!(header.default_tempo, 6);
        assert_eq!(header.default_bpm, 125);
        assert!(header.linear_frequency_table);
    }

    #[test]
    fn test_xm_invalid_magic() {
        let mut data = create_minimal_xm("Test", 4, 1, 0, 6, 125);
        data[0..17].copy_from_slice(b"Not an XM file!! ");

        let result = XmValidator::validate_header_only(&data);
        assert!(matches!(result, Err(XmFormatError::InvalidIdText { .. })));
    }

    #[test]
    fn test_xm_missing_magic_byte() {
        let mut data = create_minimal_xm("Test", 4, 1, 0, 6, 125);
        data[37] = 0x00;

        let result = XmValidator::validate_header_only(&data);
        assert!(matches!(
            result,
            Err(XmFormatError::MissingMagicByte { .. })
        ));
    }

    #[test]
    fn test_xm_invalid_version() {
        let mut data = create_minimal_xm("Test", 4, 1, 0, 6, 125);
        data[58..60].copy_from_slice(&0x0200u16.to_le_bytes());

        let result = XmValidator::validate_header_only(&data);
        assert!(matches!(
            result,
            Err(XmFormatError::UnsupportedVersion { .. })
        ));
    }

    #[test]
    fn test_xm_invalid_channel_count() {
        // Too few channels
        let data = create_minimal_xm("Test", 0, 1, 0, 6, 125);
        let result = XmValidator::validate_header_only(&data);
        assert!(matches!(
            result,
            Err(XmFormatError::InvalidChannelCount { .. })
        ));

        // Too many channels
        let data = create_minimal_xm("Test", 64, 1, 0, 6, 125);
        let result = XmValidator::validate_header_only(&data);
        assert!(matches!(
            result,
            Err(XmFormatError::InvalidChannelCount { .. })
        ));
    }

    #[test]
    fn test_xm_invalid_tempo() {
        let data = create_minimal_xm("Test", 4, 1, 0, 0, 125);
        let result = XmValidator::validate_header_only(&data);
        assert!(matches!(result, Err(XmFormatError::InvalidTempo { .. })));

        let data = create_minimal_xm("Test", 4, 1, 0, 50, 125);
        let result = XmValidator::validate_header_only(&data);
        assert!(matches!(result, Err(XmFormatError::InvalidTempo { .. })));
    }

    #[test]
    fn test_xm_invalid_bpm() {
        let data = create_minimal_xm("Test", 4, 1, 0, 6, 20);
        let result = XmValidator::validate_header_only(&data);
        assert!(matches!(result, Err(XmFormatError::InvalidBpm { .. })));
    }

    #[test]
    fn test_xm_pattern_encoding() {
        let mut data = create_minimal_xm("Test", 4, 1, 0, 6, 125);
        add_empty_pattern(&mut data, 64);

        let result = XmValidator::validate(&data);
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.valid);
        assert_eq!(report.patterns.len(), 1);
        assert!(report.patterns[0].is_empty);
    }

    #[test]
    fn test_xm_pattern_with_data() {
        let mut data = create_minimal_xm("Test", 2, 1, 0, 6, 125);

        // Pattern header
        data.extend_from_slice(&9u32.to_le_bytes()); // Header length
        data.push(0); // Packing type
        data.extend_from_slice(&4u16.to_le_bytes()); // 4 rows
        data.extend_from_slice(&8u16.to_le_bytes()); // Packed size (2 channels * 4 rows = 8 empty notes)

        // Packed pattern data (all empty notes with packing flag)
        data.extend(std::iter::repeat_n(0x80, 8));

        let result = XmValidator::validate(&data);
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.valid);
        assert!(!report.patterns[0].is_empty);
    }

    #[test]
    fn test_xm_invalid_packing_type() {
        let mut data = create_minimal_xm("Test", 2, 1, 0, 6, 125);

        // Pattern header with invalid packing type
        data.extend_from_slice(&9u32.to_le_bytes());
        data.push(1); // Invalid packing type (should be 0)
        data.extend_from_slice(&4u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());

        let result = XmValidator::validate(&data);
        assert!(matches!(
            result,
            Err(XmFormatError::InvalidPackingType { .. })
        ));
    }

    #[test]
    fn test_xm_is_xm_quick_check() {
        let data = create_minimal_xm("Test", 4, 1, 0, 6, 125);
        assert!(XmValidator::is_xm(&data));

        let bad_data = vec![0u8; 100];
        assert!(!XmValidator::is_xm(&bad_data));

        let short_data = vec![0u8; 10];
        assert!(!XmValidator::is_xm(&short_data));
    }

    #[test]
    fn test_xm_file_too_small() {
        let data = vec![0u8; 50];
        let result = XmValidator::validate(&data);
        assert!(matches!(result, Err(XmFormatError::FileTooSmall { .. })));
    }

    #[test]
    fn test_xm_instrument_structure() {
        let mut data = create_minimal_xm("Test", 4, 1, 1, 6, 125);
        add_empty_pattern(&mut data, 64);

        // Add minimal instrument with no samples
        data.extend_from_slice(&29u32.to_le_bytes()); // Instrument size (minimal)
        data.extend_from_slice(b"Test Instrument\0\0\0\0\0\0\0"); // Name (22 bytes)
        data.push(0); // Instrument type
        data.extend_from_slice(&0u16.to_le_bytes()); // Number of samples

        let result = XmValidator::validate(&data);
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.valid);
        assert_eq!(report.instruments.len(), 1);
        assert!(report
            .warnings
            .iter()
            .any(|w| matches!(w, XmWarning::InstrumentWithoutSamples { .. })));
    }

    #[test]
    fn test_xm_sample_data() {
        let mut data = create_minimal_xm("Test", 4, 1, 1, 6, 125);
        add_empty_pattern(&mut data, 64);

        // Add instrument header with 1 sample
        data.extend_from_slice(&263u32.to_le_bytes()); // Standard instrument size
        data.extend_from_slice(b"Test Instrument\0\0\0\0\0\0\0"); // Name (22 bytes)
        data.push(0); // Instrument type
        data.extend_from_slice(&1u16.to_le_bytes()); // Number of samples
        data.extend_from_slice(&40u32.to_le_bytes()); // Sample header size

        // Note-sample mapping (96 bytes)
        data.extend_from_slice(&[0u8; 96]);

        // Volume envelope points (48 bytes)
        data.extend_from_slice(&[0u8; 48]);

        // Panning envelope points (48 bytes)
        data.extend_from_slice(&[0u8; 48]);

        // Envelope parameters
        data.push(0); // num vol points
        data.push(0); // num pan points
        data.push(0); // vol sustain
        data.push(0); // vol loop start
        data.push(0); // vol loop end
        data.push(0); // pan sustain
        data.push(0); // pan loop start
        data.push(0); // pan loop end
        data.push(0); // vol flags
        data.push(0); // pan flags

        // Vibrato
        data.push(0); // type
        data.push(0); // sweep
        data.push(0); // depth
        data.push(0); // rate

        // Volume fadeout
        data.extend_from_slice(&0u16.to_le_bytes());

        // Reserved (22 bytes)
        data.extend_from_slice(&[0u8; 22]);

        // Sample header (40 bytes)
        data.extend_from_slice(&100u32.to_le_bytes()); // Length
        data.extend_from_slice(&0u32.to_le_bytes()); // Loop start
        data.extend_from_slice(&0u32.to_le_bytes()); // Loop length
        data.push(64); // Volume
        data.push(0); // Finetune
        data.push(0); // Flags (no loop, 8-bit)
        data.push(128); // Panning
        data.push(0); // Relative note
        data.push(0); // Reserved
        data.extend_from_slice(b"Test Sample\0\0\0\0\0\0\0\0\0\0\0"); // Name (22 bytes)

        // Sample data (100 bytes)
        data.extend_from_slice(&[128u8; 100]);

        let result = XmValidator::validate(&data);
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.valid);
    }

    #[test]
    fn test_xm_invalid_sample_loop_bounds() {
        let mut data = create_minimal_xm("Test", 4, 1, 1, 6, 125);
        add_empty_pattern(&mut data, 64);

        // Add full instrument header with 1 sample (263 bytes)
        data.extend_from_slice(&263u32.to_le_bytes()); // Instrument size
        data.extend_from_slice(b"Test Instrument\0\0\0\0\0\0\0"); // Name (22 bytes)
        data.push(0); // Instrument type
        data.extend_from_slice(&1u16.to_le_bytes()); // Number of samples
        data.extend_from_slice(&40u32.to_le_bytes()); // Sample header size
        data.extend_from_slice(&[0u8; 96]); // Note-sample mapping (96 bytes)
        data.extend_from_slice(&[0u8; 48]); // Volume envelope points (48 bytes)
        data.extend_from_slice(&[0u8; 48]); // Panning envelope points (48 bytes)
        data.push(0); // Num vol points
        data.push(0); // Num pan points
        data.push(0); // Vol sustain
        data.push(0); // Vol loop start
        data.push(0); // Vol loop end
        data.push(0); // Pan sustain
        data.push(0); // Pan loop start
        data.push(0); // Pan loop end
        data.push(0); // Vol flags
        data.push(0); // Pan flags
        data.push(0); // Vibrato type
        data.push(0); // Vibrato sweep
        data.push(0); // Vibrato depth
        data.push(0); // Vibrato rate
        data.extend_from_slice(&0u16.to_le_bytes()); // Volume fadeout
        data.extend_from_slice(&[0u8; 22]); // Reserved (22 bytes)

        // Sample header with invalid loop (40 bytes)
        data.extend_from_slice(&100u32.to_le_bytes()); // Length: 100 bytes
        data.extend_from_slice(&50u32.to_le_bytes()); // Loop start: 50
        data.extend_from_slice(&100u32.to_le_bytes()); // Loop length: 100 (50 + 100 > 100, invalid!)
        data.push(64); // Volume
        data.push(0); // Finetune
        data.push(1); // Flags (forward loop enabled)
        data.push(128); // Panning
        data.push(0); // Relative note
        data.push(0); // Reserved
        data.extend_from_slice(&[0u8; 22]); // Sample name (22 bytes)

        // Sample data (100 bytes)
        data.extend_from_slice(&[128u8; 100]);

        let result = XmValidator::validate(&data);
        assert!(matches!(
            result,
            Err(XmFormatError::InvalidLoopBounds { .. })
        ));
    }

    #[test]
    fn test_xm_full_validation() {
        // Create a complete valid XM file
        let mut data = create_minimal_xm("Complete Test", 4, 2, 1, 6, 125);

        // Update song length and pattern order
        data[64..66].copy_from_slice(&2u16.to_le_bytes()); // 2 orders
        data[80] = 0; // Play pattern 0 first
        data[81] = 1; // Then pattern 1

        // Add two patterns
        add_empty_pattern(&mut data, 64);
        add_empty_pattern(&mut data, 32);

        // Add minimal instrument
        data.extend_from_slice(&29u32.to_le_bytes());
        data.extend_from_slice(b"Minimal Instrument\0\0\0\0");
        data.push(0);
        data.extend_from_slice(&0u16.to_le_bytes());

        let result = XmValidator::validate(&data);
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.valid);
        assert_eq!(report.patterns.len(), 2);
        assert_eq!(report.instruments.len(), 1);
        assert!(report.header.is_some());

        let header = report.header.unwrap();
        assert_eq!(header.song_length, 2);
        assert_eq!(header.num_patterns, 2);
        assert_eq!(header.pattern_order[0], 0);
        assert_eq!(header.pattern_order[1], 1);
    }

    #[test]
    fn test_xm_warnings_generated() {
        // Use unusual but valid tempo (25 is > 20 which triggers warning) and BPM (45 is < 50 which triggers warning)
        // Both are within valid ranges (tempo 1-31, BPM 32-255)
        let mut data = create_minimal_xm("Test", 4, 1, 0, 25, 45);

        // Non-standard header size (280 instead of 276)
        data[60..64].copy_from_slice(&280u32.to_le_bytes());

        // Extend data to account for larger header size
        // Header now claims to be 280 bytes, so patterns start at offset 60 + 280 = 340 instead of 336
        let additional_bytes = 4; // 280 - 276 = 4 extra bytes
        data.resize(336 + additional_bytes, 0);

        add_empty_pattern(&mut data, 64);

        let result = XmValidator::validate(&data);
        assert!(
            result.is_ok(),
            "Validation should succeed with warnings: {:?}",
            result
        );

        let report = result.unwrap();
        // Should have warnings but still be valid
        assert!(report.has_warnings(), "Should have warnings");
        assert!(
            report
                .warnings
                .iter()
                .any(|w| matches!(w, XmWarning::NonStandardHeaderSize { .. })),
            "Should have non-standard header size warning"
        );
        assert!(
            report
                .warnings
                .iter()
                .any(|w| matches!(w, XmWarning::UnusualTempoBpm { .. })),
            "Should have unusual tempo/BPM warning"
        );
    }
}
