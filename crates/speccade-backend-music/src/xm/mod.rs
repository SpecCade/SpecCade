//! XM (FastTracker II Extended Module) format writer and validator.
//!
//! This module provides deterministic generation of XM tracker files from SpecCade specs,
//! as well as comprehensive validation against the official XM file format specification.
//!
//! # XM Format Overview
//!
//! XM is a module format created by Triton for FastTracker II. Key features:
//! - Up to 32 channels
//! - Up to 128 instruments with embedded samples
//! - Up to 256 patterns
//! - Volume and panning envelopes
//! - Linear frequency table for precise pitch control
//!
//! # Validation
//!
//! The `validator` module provides thorough validation of XM files:
//!
//! ```rust,ignore
//! use speccade_backend_music::xm::{XmValidator, XmValidationReport};
//!
//! let data = std::fs::read("song.xm")?;
//! let report = XmValidator::validate(&data)?;
//!
//! if report.valid {
//!     println!("Valid XM file: {} channels, {} patterns",
//!         report.header.as_ref().unwrap().num_channels,
//!         report.patterns.len());
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
mod validator;
mod writer;

pub use header::*;
pub use instrument::*;
pub use pattern::*;
pub use sample::*;
pub use validator::*;
pub use writer::*;
