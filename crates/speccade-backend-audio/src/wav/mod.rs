//! Deterministic WAV file writer.
//!
//! This module writes 16-bit PCM WAV files with no timestamps or variable metadata
//! to ensure deterministic output. The hash of the PCM data can be used for
//! Tier 1 validation.

mod builder;
mod format;
mod pcm;
mod result;
mod writer;

#[cfg(test)]
mod tests;

// Re-export public API
pub use builder::WavWriter;
pub use format::WavFormat;
pub use pcm::{compute_pcm_hash, extract_pcm_data};
pub use result::WavResult;
pub use writer::{samples_to_pcm16, stereo_to_pcm16, write_wav, write_wav_to_vec};
