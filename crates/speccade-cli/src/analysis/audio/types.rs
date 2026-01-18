//! Audio analysis types and data structures.

use serde::{Deserialize, Serialize};

/// Audio analysis result containing all extracted metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetrics {
    /// Format metadata
    pub format: AudioFormatMetadata,
    /// Quality metrics (peak, RMS, clipping, etc.)
    pub quality: AudioQualityMetrics,
    /// Temporal metrics (attack, zero crossing rate)
    pub temporal: AudioTemporalMetrics,
    /// Spectral metrics (centroid, dominant frequency)
    pub spectral: AudioSpectralMetrics,
}

/// Audio format metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFormatMetadata {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Bits per sample
    pub bits_per_sample: u16,
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Total number of samples (per channel)
    pub num_samples: usize,
}

/// Audio quality metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQualityMetrics {
    /// Peak amplitude in dB (0 dB = full scale)
    pub peak_db: f64,
    /// RMS level in dB
    pub rms_db: f64,
    /// Whether clipping was detected (samples at or near full scale)
    pub clipping_detected: bool,
    /// DC offset as a fraction of full scale
    pub dc_offset: f64,
    /// Ratio of silent samples (below threshold)
    pub silence_ratio: f64,
}

/// Audio temporal metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTemporalMetrics {
    /// Attack time in milliseconds (time from onset to peak)
    pub attack_ms: f64,
    /// Zero crossing rate (crossings per sample)
    pub zero_crossing_rate: f64,
}

/// Audio spectral metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSpectralMetrics {
    /// Spectral centroid in Hz (center of mass of spectrum)
    pub centroid_hz: f64,
    /// Dominant frequency in Hz (loudest frequency component)
    pub dominant_frequency_hz: f64,
}

/// Error type for audio analysis.
#[derive(Debug, Clone)]
pub enum AudioAnalysisError {
    /// WAV data is too short
    DataTooShort { expected: usize, actual: usize },
    /// Invalid RIFF header
    InvalidRiffHeader,
    /// Invalid WAVE format
    InvalidWaveFormat,
    /// Missing fmt chunk
    MissingFmtChunk,
    /// Missing data chunk
    MissingDataChunk,
    /// Unsupported audio format
    UnsupportedAudioFormat { format_code: u16 },
    /// Unsupported bits per sample
    UnsupportedBitsPerSample { bits: u16 },
    /// Invalid chunk
    InvalidChunk { message: String },
    /// Empty audio
    EmptyAudio,
}

impl std::fmt::Display for AudioAnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            AudioAnalysisError::EmptyAudio => {
                write!(f, "Audio file contains no samples")
            }
        }
    }
}

impl std::error::Error for AudioAnalysisError {}

/// WAV header information.
pub(super) struct WavHeader {
    pub channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub num_samples: usize,
}
