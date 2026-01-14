//! Error and warning types for XM validation.

use std::fmt;

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
