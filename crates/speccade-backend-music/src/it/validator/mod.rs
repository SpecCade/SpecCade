//! Comprehensive IT (Impulse Tracker) format validator.
//!
//! This module provides thorough validation of IT files against the official
//! ITTECH.TXT specification from Impulse Tracker.
//!
//! Reference: <https://github.com/schismtracker/schismtracker/wiki/ITTECH.TXT>

mod error;
mod helpers;
mod info;
mod instrument_validator;
mod pattern_validator;
mod report;
mod sample_validator;
mod validator;

#[cfg(test)]
mod tests;

// Re-export public API
pub use error::{ItErrorCategory, ItFormatError};
pub use info::{
    ItConvertFlags, ItEnvelopeInfo, ItFlags, ItHeaderInfo, ItInstrumentInfo, ItPatternInfo,
    ItSampleFlags, ItSampleInfo, ItSpecialFlags,
};
pub use report::{ItValidationReport, ItValidationWarning};
pub use validator::ItValidator;
