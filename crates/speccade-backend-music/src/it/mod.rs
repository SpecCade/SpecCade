//! IT (Impulse Tracker) format writer.
//!
//! This module provides deterministic generation of IT tracker files from SpecCade specs.
//!
//! # IT Format Overview
//!
//! IT is a module format created by Jeffrey Lim for Impulse Tracker. Key features:
//! - Up to 64 channels
//! - Separate instrument and sample definitions
//! - NNA (New Note Actions) for polyphonic instruments
//! - Volume, panning, and pitch envelopes
//! - Resonant filters

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
