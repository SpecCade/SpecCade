//! XM (FastTracker II Extended Module) format writer.
//!
//! This module provides deterministic generation of XM tracker files from SpecCade specs.
//!
//! # XM Format Overview
//!
//! XM is a module format created by Triton for FastTracker II. Key features:
//! - Up to 32 channels
//! - Up to 128 instruments with embedded samples
//! - Up to 256 patterns
//! - Volume and panning envelopes
//! - Linear frequency table for precise pitch control

mod header;
mod instrument;
mod pattern;
mod sample;
mod writer;

pub use header::*;
pub use instrument::*;
pub use pattern::*;
pub use sample::*;
pub use writer::*;
