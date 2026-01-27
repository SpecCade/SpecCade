//! Main entry point for music generation from SpecCade specs.
//!
//! This module provides the public API for generating tracker modules (XM and IT)
//! from SpecCade `MusicTrackerSongV1Params` specifications.
//!
//! # Module Organization
//!
//! The generation logic is split into specialized modules:
//! - [`crate::envelope`] - Envelope conversion (ADSR to tracker format)
//! - [`crate::xm_gen`] - XM (FastTracker II) format generation
//! - [`crate::it_gen`] - IT (Impulse Tracker) format generation
//!
//! This module serves as the thin dispatcher and re-exports shared types.

use std::path::Path;

use serde::Serialize;
use speccade_spec::recipe::music::{
    MusicTrackerSongComposeV1Params, MusicTrackerSongV1Params, TrackerFormat, TrackerLoopMode,
};
use speccade_spec::BackendError;
use thiserror::Error;

use crate::compose::expand_compose;

// Re-export format-specific generators (internal use)
pub(crate) use crate::it_gen::generate_it;
pub(crate) use crate::xm_gen::generate_xm;

// Re-export internal modules
mod helpers;
mod instrument_baking;
mod loop_detection;

#[cfg(test)]
mod tests;

// Re-export key types and functions for internal use
pub(crate) use helpers::resolve_pattern_note_name;
pub(crate) use instrument_baking::bake_instrument_sample;

/// Error type for music generation.
#[derive(Debug, Error)]
pub enum GenerateError {
    /// Invalid parameter value.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Pattern not found in spec.
    #[error("Pattern not found: {0}")]
    PatternNotFound(String),

    /// IO error during writing.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Unsupported synthesis type.
    #[error("Unsupported synthesis type: {0}")]
    UnsupportedSynthesis(String),

    /// Sample loading error.
    #[error("Sample loading error: {0}")]
    SampleLoadError(String),

    /// Instrument definition error.
    #[error("Instrument error: {0}")]
    InstrumentError(String),

    /// Automation error.
    #[error("Automation error: {0}")]
    AutomationError(String),

    /// Compose expansion error.
    #[error("Compose expansion error: {0}")]
    ComposeExpand(#[from] crate::compose::ExpandError),
}

impl BackendError for GenerateError {
    fn code(&self) -> &'static str {
        match self {
            GenerateError::InvalidParameter(_) => "MUSIC_001",
            GenerateError::PatternNotFound(_) => "MUSIC_002",
            GenerateError::IoError(_) => "MUSIC_003",
            GenerateError::UnsupportedSynthesis(_) => "MUSIC_004",
            GenerateError::SampleLoadError(_) => "MUSIC_005",
            GenerateError::InstrumentError(_) => "MUSIC_006",
            GenerateError::AutomationError(_) => "MUSIC_007",
            GenerateError::ComposeExpand(err) => err.code(),
        }
    }

    fn category(&self) -> &'static str {
        "music"
    }
}

/// Loop mode actually used for an instrument sample in the generated module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChosenLoopMode {
    None,
    Forward,
    PingPong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopSeamMetrics {
    /// Absolute amplitude delta at the loop boundary.
    pub amp_jump: u32,
    /// Absolute delta between end/start slopes at the loop boundary.
    pub slope_jump: u32,
}

/// Loop diagnostics for a single tracker instrument.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MusicInstrumentLoopReport {
    pub index: u32,
    pub name: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_mode: Option<TrackerLoopMode>,

    pub chosen_mode: ChosenLoopMode,

    pub base_midi: u8,
    pub base_freq_hz: f64,
    pub sample_rate: u32,
    pub sample_len: u32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desired_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_end: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_len: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_corr: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crossfade_samples: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crossfade_ms: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seam: Option<LoopSeamMetrics>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pingpong_start_slope: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pingpong_end_slope: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dc_removed_mean: Option<i64>,

    /// Pitch deviation in cents after format-specific correction.
    /// Positive = sharp, negative = flat. Ideally < 1 cent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch_deviation_cents: Option<f64>,
}

/// Loop diagnostics for the generated tracker module.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MusicLoopReport {
    /// The generated module format ("xm" or "it").
    pub extension: String,
    pub instruments: Vec<MusicInstrumentLoopReport>,
}

/// Result of music generation.
pub struct GenerateResult {
    /// Generated tracker module bytes.
    pub data: Vec<u8>,
    /// BLAKE3 hash of the generated data.
    pub hash: String,
    /// File extension ("xm" or "it").
    pub extension: &'static str,
    /// Optional loop diagnostics, intended for developer workflows.
    pub loop_report: Option<MusicLoopReport>,
}

/// Generate a tracker module from a SpecCade music spec.
///
/// This is the main entry point for music generation. It dispatches to
/// format-specific generators based on the `format` field in params.
///
/// # Arguments
/// * `params` - Music tracker song parameters from the spec
/// * `seed` - Base seed for deterministic generation
/// * `spec_dir` - Directory containing the spec file (for resolving relative sample paths)
///
/// # Returns
/// `GenerateResult` containing the module bytes, hash, and extension
///
/// # Example
/// ```ignore
/// use speccade_backend_music::generate::generate_music;
/// use speccade_spec::recipe::music::MusicTrackerSongV1Params;
/// use std::path::Path;
///
/// let params = MusicTrackerSongV1Params { ... };
/// let spec_dir = Path::new("path/to/spec/dir");
/// let result = generate_music(&params, 42, spec_dir)?;
/// std::fs::write(format!("song.{}", result.extension), &result.data)?;
/// ```
pub fn generate_music(
    params: &MusicTrackerSongV1Params,
    seed: u32,
    spec_dir: &Path,
) -> Result<GenerateResult, GenerateError> {
    match params.format {
        TrackerFormat::Xm => generate_xm(params, seed, spec_dir),
        TrackerFormat::It => generate_it(params, seed, spec_dir),
    }
}

/// Generate a tracker module from compose (Pattern IR) params.
pub fn generate_music_compose(
    params: &MusicTrackerSongComposeV1Params,
    seed: u32,
    spec_dir: &Path,
) -> Result<GenerateResult, GenerateError> {
    let expanded = expand_compose(params, seed)?;
    generate_music(&expanded, seed, spec_dir)
}

// ============================================================================
// Shared utilities used by xm_gen and it_gen
// ============================================================================

/// Baked tracker sample data for a `TrackerInstrument`.
#[derive(Debug, Clone)]
pub(crate) struct BakedInstrumentSample {
    /// 16-bit mono PCM bytes (little-endian i16).
    pub pcm16_mono: Vec<u8>,
    /// Natural sample rate of the PCM data.
    pub sample_rate: u32,
    /// Base MIDI note the sample is tuned to.
    pub base_midi: u8,
    /// Optional loop region for sustained instruments.
    pub loop_region: Option<LoopRegion>,
}

/// Sample loop mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum LoopMode {
    Forward,
    PingPong,
}

/// Loop points for tracker samples.
///
/// In both XM and IT, loop end is stored as an absolute index (not a length) here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LoopRegion {
    pub start: u32,
    pub end: u32,
    pub mode: LoopMode,
}
