//! IT (Impulse Tracker) format writer and validator.
//!
//! This module provides deterministic generation of IT tracker files from SpecCade specs,
//! as well as comprehensive validation against the ITTECH.TXT specification.
//!
//! # IT Format Overview
//!
//! IT is a module format created by Jeffrey Lim for Impulse Tracker. Key features:
//! - Up to 64 channels
//! - Separate instrument and sample definitions
//! - NNA (New Note Actions) for polyphonic instruments
//! - Volume, panning, and pitch envelopes
//! - Resonant filters
//!
//! # Validation
//!
//! The `validator` module provides comprehensive validation of IT files against the
//! official ITTECH.TXT specification:
//!
//! ```rust,ignore
//! use speccade_backend_music::it::validator::ItValidator;
//!
//! let data = std::fs::read("song.it")?;
//! let report = ItValidator::validate(&data)?;
//!
//! if report.is_valid {
//!     println!("Valid IT file: {}", report.header.unwrap().name);
//! } else {
//!     for error in &report.errors {
//!         eprintln!("Error: {}", error);
//!     }
//! }
//! ```

mod header;
mod instrument;
mod pattern;
mod sample;
pub mod validator;
mod writer;

pub use header::*;
pub use instrument::*;
pub use pattern::*;
pub use sample::*;
pub use validator::{ItErrorCategory, ItFormatError, ItValidationReport, ItValidator};
pub use writer::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validator_rejects_too_small_files() {
        let err = ItValidator::validate(&[0u8; 10]).unwrap_err();
        assert_eq!(err.category, ItErrorCategory::Structure);
        assert!(err.to_string().contains("File too small"));
    }

    #[test]
    fn validator_accepts_writer_output() {
        let mut module = ItModule::new("Test", 4, 6, 125);
        module.add_pattern(ItPattern::empty(64, 4));
        module.add_instrument(ItInstrument::new("Inst1"));
        module.add_sample(ItSample::new("Sample1", vec![0u8; 100], 22050));
        module.set_orders(&[0]);

        let bytes = module.to_bytes().unwrap();
        let report = ItValidator::validate(&bytes).unwrap();
        assert!(report.is_valid, "errors: {:?}", report.errors);
        assert!(report.header.is_some());
    }
}
