//! Error types and data structures for audio analysis.

use std::fmt;

/// Error type for audio analysis operations.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioAnalysisError {
    /// WAV data is too short to contain required headers.
    DataTooShort { expected: usize, actual: usize },
    /// Missing or invalid RIFF header.
    InvalidRiffHeader,
    /// Missing or invalid WAVE format identifier.
    InvalidWaveFormat,
    /// Missing fmt chunk.
    MissingFmtChunk,
    /// Missing data chunk.
    MissingDataChunk,
    /// Unsupported audio format (only PCM supported).
    UnsupportedAudioFormat { format_code: u16 },
    /// Unsupported bits per sample.
    UnsupportedBitsPerSample { bits: u16 },
    /// Invalid or corrupted chunk.
    InvalidChunk { message: String },
}

impl fmt::Display for AudioAnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioAnalysisError::DataTooShort { expected, actual } => {
                write!(
                    f,
                    "WAV data too short: expected at least {} bytes, got {}",
                    expected, actual
                )
            }
            AudioAnalysisError::InvalidRiffHeader => {
                write!(f, "Invalid or missing RIFF header")
            }
            AudioAnalysisError::InvalidWaveFormat => {
                write!(f, "Invalid or missing WAVE format identifier")
            }
            AudioAnalysisError::MissingFmtChunk => {
                write!(f, "Missing fmt chunk in WAV file")
            }
            AudioAnalysisError::MissingDataChunk => {
                write!(f, "Missing data chunk in WAV file")
            }
            AudioAnalysisError::UnsupportedAudioFormat { format_code } => {
                write!(
                    f,
                    "Unsupported audio format code: {} (only PCM/1 supported)",
                    format_code
                )
            }
            AudioAnalysisError::UnsupportedBitsPerSample { bits } => {
                write!(
                    f,
                    "Unsupported bits per sample: {} (only 8, 16, 24, 32 supported)",
                    bits
                )
            }
            AudioAnalysisError::InvalidChunk { message } => {
                write!(f, "Invalid chunk: {}", message)
            }
        }
    }
}

impl std::error::Error for AudioAnalysisError {}

/// WAV file header information.
#[derive(Debug, Clone, PartialEq)]
pub struct WavHeader {
    /// Number of audio channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Sample rate in Hz (e.g., 44100, 48000).
    pub sample_rate: u32,
    /// Bits per sample (8, 16, 24, or 32).
    pub bits_per_sample: u16,
    /// Total number of samples per channel.
    pub num_samples: usize,
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Byte rate (bytes per second).
    pub byte_rate: u32,
    /// Block alignment (bytes per sample frame).
    pub block_align: u16,
}
