//! Audio signal analysis utilities for testing speccade audio generation.
//!
//! This module provides functions to verify audio output quality beyond simple
//! "not empty" checks. It includes signal analysis functions and WAV parsing helpers.
//!
//! ## Example
//!
//! ```rust,no_run
//! use speccade_tests::audio_analysis::{parse_wav_samples, calculate_rms, is_silent};
//!
//! let wav_data = std::fs::read("output.wav").unwrap();
//! let samples = parse_wav_samples(&wav_data).unwrap();
//!
//! let rms = calculate_rms(&samples);
//! assert!(!is_silent(&samples, 0.001));
//! ```

mod channel;
mod error;
mod signal;
mod wav;

#[cfg(test)]
mod tests;

// Re-export public API
pub use channel::{left_channel, right_channel, stereo_to_mono};
pub use error::{AudioAnalysisError, WavHeader};
pub use signal::{
    calculate_rms, detect_clipping, has_audio_content, is_silent, peak_amplitude,
    zero_crossing_rate,
};
pub use wav::{parse_wav_header, parse_wav_samples};
