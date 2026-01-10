//! SpecCade Music Backend - Deterministic XM/IT Tracker Module Generation
//!
//! This crate provides deterministic generation of tracker module files (XM and IT formats)
//! from SpecCade music specifications. It implements Tier 1 determinism guarantees as
//! defined in the SpecCade determinism policy.
//!
//! # Features
//!
//! - **XM Format (FastTracker II)**: Up to 32 channels, 128 instruments, volume/panning envelopes
//! - **IT Format (Impulse Tracker)**: Up to 64 channels, NNA for polyphony, pitch envelopes
//! - **Deterministic Synthesis**: Instrument samples generated from specs using seeded RNG
//! - **Full Hash Validation**: BLAKE3 hashes for Tier 1 validation
//!
//! # Determinism
//!
//! All operations in this crate are fully deterministic. Given the same spec and seed,
//! the output will be byte-identical. This is achieved through:
//!
//! - PCG32 random number generator (seeded via BLAKE3 hash derivation)
//! - Deterministic waveform generation
//! - Fixed-format binary output
//!
//! # Example
//!
//! ```ignore
//! use speccade_backend_music::generate::generate_music;
//! use speccade_spec::recipe::music::{MusicTrackerSongV1Params, TrackerFormat};
//!
//! // Create a music spec
//! let params = MusicTrackerSongV1Params {
//!     format: TrackerFormat::Xm,
//!     bpm: 120,
//!     speed: 6,
//!     channels: 4,
//!     r#loop: true,
//!     instruments: vec![...],
//!     patterns: HashMap::new(),
//!     arrangement: vec![...],
//! };
//!
//! // Generate with seed
//! let result = generate_music(&params, 42)?;
//!
//! // Write to file
//! std::fs::write(format!("song.{}", result.extension), &result.data)?;
//!
//! // Verify hash
//! println!("Generated hash: {}", result.hash);
//! ```
//!
//! # Module Structure
//!
//! - [`note`]: Note/frequency conversion utilities
//! - [`synthesis`]: Instrument sample generation
//! - [`xm`]: XM (FastTracker II) format writer
//! - [`it`]: IT (Impulse Tracker) format writer
//! - [`generate`]: Main generation entry point

pub mod envelope;
pub mod generate;
pub mod it;
pub mod it_gen;
pub mod note;
pub mod synthesis;
pub mod xm;
pub mod xm_gen;

// Re-export main types
pub use generate::{generate_music, GenerateError, GenerateResult};
pub use note::{
    calculate_pitch_correction, freq_to_midi, it_note_to_name, midi_to_freq, note_name_to_it,
    note_name_to_xm, xm_note_to_name, DEFAULT_SAMPLE_RATE,
};

/// Crate version for backend identification.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Backend identifier for cache keys.
pub const BACKEND_ID: &str = "speccade-backend-music";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_backend_id() {
        assert_eq!(BACKEND_ID, "speccade-backend-music");
    }
}
