//! Error types for audio backend.

use speccade_spec::BackendError;
use thiserror::Error;

/// Result type for audio operations.
pub type AudioResult<T> = Result<T, AudioError>;

/// Errors that can occur during audio generation.
#[derive(Debug, Error)]
pub enum AudioError {
    /// Missing recipe in spec.
    #[error("spec does not contain a recipe")]
    MissingRecipe,

    /// Invalid recipe type.
    #[error("invalid recipe type: expected {expected}, found {found}")]
    InvalidRecipeType {
        /// Expected recipe kind.
        expected: String,
        /// Found recipe kind.
        found: String,
    },

    /// Invalid sample rate.
    #[error("invalid sample rate: {rate}")]
    InvalidSampleRate {
        /// The invalid sample rate.
        rate: u32,
    },

    /// Invalid duration.
    #[error("invalid duration: {duration} seconds")]
    InvalidDuration {
        /// The invalid duration.
        duration: f64,
    },

    /// Invalid frequency.
    #[error("invalid frequency: {freq} Hz")]
    InvalidFrequency {
        /// The invalid frequency.
        freq: f64,
    },

    /// Invalid parameter value.
    #[error("invalid parameter '{name}': {message}")]
    InvalidParameter {
        /// Parameter name.
        name: String,
        /// Error message.
        message: String,
    },

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Internal synthesis error.
    #[error("synthesis error: {message}")]
    Synthesis {
        /// Error message.
        message: String,
    },
}

impl AudioError {
    /// Creates an invalid parameter error.
    pub fn invalid_param(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidParameter {
            name: name.into(),
            message: message.into(),
        }
    }

    /// Creates a synthesis error.
    pub fn synthesis(message: impl Into<String>) -> Self {
        Self::Synthesis {
            message: message.into(),
        }
    }
}

impl BackendError for AudioError {
    fn code(&self) -> &'static str {
        match self {
            AudioError::MissingRecipe => "AUDIO_001",
            AudioError::InvalidRecipeType { .. } => "AUDIO_002",
            AudioError::InvalidSampleRate { .. } => "AUDIO_003",
            AudioError::InvalidDuration { .. } => "AUDIO_004",
            AudioError::InvalidFrequency { .. } => "AUDIO_005",
            AudioError::InvalidParameter { .. } => "AUDIO_006",
            AudioError::Io(_) => "AUDIO_007",
            AudioError::Synthesis { .. } => "AUDIO_008",
        }
    }

    fn category(&self) -> &'static str {
        "audio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_param_helper() {
        let err = AudioError::invalid_param("gain", "must be between 0 and 1");
        assert!(err.to_string().contains("gain"));
        assert!(err.to_string().contains("between 0 and 1"));
    }

    #[test]
    fn test_synthesis_helper() {
        let err = AudioError::synthesis("oscillator blew up");
        assert!(err.to_string().contains("oscillator blew up"));
    }
}
