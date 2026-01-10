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
