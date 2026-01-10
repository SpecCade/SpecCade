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

use speccade_spec::recipe::music::{
    InstrumentSynthesis, MusicTrackerSongV1Params, TrackerFormat, TrackerInstrument,
};
use speccade_spec::BackendError;
use thiserror::Error;

// Re-export format-specific generators (internal use)
pub(crate) use crate::it_gen::generate_it;
pub(crate) use crate::xm_gen::generate_xm;

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
        }
    }

    fn category(&self) -> &'static str {
        "music"
    }
}

/// Result of music generation.
pub struct GenerateResult {
    /// Generated tracker module bytes.
    pub data: Vec<u8>,
    /// BLAKE3 hash of the generated data.
    pub hash: String,
    /// File extension ("xm" or "it").
    pub extension: &'static str,
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

// ============================================================================
// Shared utilities used by xm_gen and it_gen
// ============================================================================

/// Get the synthesis configuration for an instrument, handling ref if present.
///
/// This resolves either an inline synthesis definition or loads from an
/// external reference file.
pub(crate) fn get_instrument_synthesis(
    instr: &TrackerInstrument,
    spec_dir: &Path,
) -> Result<InstrumentSynthesis, GenerateError> {
    if let Some(ref ref_path) = instr.r#ref {
        // Load external spec file
        load_instrument_from_ref(ref_path, spec_dir)
    } else if let Some(ref synthesis) = instr.synthesis {
        // Use inline synthesis
        Ok(synthesis.clone())
    } else {
        // Default to a basic synthesis if neither is provided
        Err(GenerateError::InstrumentError(format!(
            "Instrument '{}' must have either 'ref' or 'synthesis' defined",
            instr.name
        )))
    }
}

/// Load instrument synthesis from an external spec file.
fn load_instrument_from_ref(
    ref_path: &str,
    _spec_dir: &Path,
) -> Result<InstrumentSynthesis, GenerateError> {
    // TODO: Implement full spec file loading
    // For now, return an error as this requires the full spec parser
    Err(GenerateError::InstrumentError(format!(
        "External instrument reference loading not yet implemented: {}",
        ref_path
    )))
}

/// Extract effect code and parameter from PatternEffect, handling effect_xy nibbles.
///
/// This is used by both XM and IT pattern converters.
pub(crate) fn extract_effect_code_param<F>(
    effect: &speccade_spec::recipe::music::PatternEffect,
    name_to_code: F,
) -> Result<(Option<u8>, u8), GenerateError>
where
    F: Fn(&str) -> Option<u8>,
{
    // Calculate parameter from effect_xy nibbles if present
    let param = if let Some((x, y)) = effect.effect_xy {
        // Convert nibbles to byte: param = (x << 4) | y
        (x << 4) | y
    } else if let Some(p) = effect.param {
        p
    } else {
        0
    };

    // Get effect code from type name
    let code = if let Some(ref type_name) = effect.r#type {
        name_to_code(type_name)
    } else {
        None
    };

    Ok((code, param))
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::audio_sfx::Envelope;
    use speccade_spec::recipe::music::{ArrangementEntry, PatternNote, TrackerPattern};
    use std::collections::HashMap;

    fn create_test_params() -> MusicTrackerSongV1Params {
        let envelope = Envelope {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
        };

        let instrument = TrackerInstrument {
            name: "Test Lead".to_string(),
            r#ref: None,
            synthesis: Some(InstrumentSynthesis::Pulse { duty_cycle: 0.5 }),
            envelope: envelope.clone(),
            default_volume: Some(64),
        };

        let pattern = TrackerPattern {
            rows: 16,
            data: vec![
                PatternNote {
                    row: 0,
                    channel: 0,
                    note: "C4".to_string(),
                    instrument: 0,
                    volume: Some(64),
                    effect: None,
                },
                PatternNote {
                    row: 4,
                    channel: 0,
                    note: "E4".to_string(),
                    instrument: 0,
                    volume: Some(64),
                    effect: None,
                },
                PatternNote {
                    row: 8,
                    channel: 0,
                    note: "G4".to_string(),
                    instrument: 0,
                    volume: Some(64),
                    effect: None,
                },
                PatternNote {
                    row: 12,
                    channel: 0,
                    note: "OFF".to_string(),
                    instrument: 0,
                    volume: None,
                    effect: None,
                },
            ],
        };

        let mut patterns = HashMap::new();
        patterns.insert("intro".to_string(), pattern);

        MusicTrackerSongV1Params {
            format: TrackerFormat::Xm,
            bpm: 120,
            speed: 6,
            channels: 4,
            r#loop: true,
            instruments: vec![instrument],
            patterns,
            arrangement: vec![ArrangementEntry {
                pattern: "intro".to_string(),
                repeat: 1,
            }],
            automation: vec![],
            it_options: None,
        }
    }

    #[test]
    fn test_generate_xm() {
        let params = create_test_params();
        let spec_dir = Path::new(".");
        let result = generate_music(&params, 42, spec_dir).unwrap();

        assert_eq!(result.extension, "xm");
        assert!(!result.data.is_empty());
        assert_eq!(result.hash.len(), 64);
    }

    #[test]
    fn test_generate_it() {
        let mut params = create_test_params();
        params.format = TrackerFormat::It;

        let spec_dir = Path::new(".");
        let result = generate_music(&params, 42, spec_dir).unwrap();

        assert_eq!(result.extension, "it");
        assert!(!result.data.is_empty());
        assert_eq!(result.hash.len(), 64);
    }

    #[test]
    fn test_determinism() {
        let params = create_test_params();
        let spec_dir = Path::new(".");

        let result1 = generate_music(&params, 42, spec_dir).unwrap();
        let result2 = generate_music(&params, 42, spec_dir).unwrap();

        assert_eq!(result1.hash, result2.hash);
        assert_eq!(result1.data, result2.data);
    }

    #[test]
    fn test_different_seeds_different_output() {
        // Use noise synthesis which uses the seed
        let mut params = create_test_params();
        params.instruments[0].synthesis = Some(InstrumentSynthesis::Noise { periodic: false });

        let spec_dir = Path::new(".");
        let result1 = generate_music(&params, 42, spec_dir).unwrap();
        let result2 = generate_music(&params, 43, spec_dir).unwrap();

        // Different seeds should produce different hashes (due to noise synthesis)
        assert_ne!(result1.hash, result2.hash);
    }

    #[test]
    fn test_invalid_channels() {
        let mut params = create_test_params();
        params.channels = 100; // Invalid for XM (max 32)

        let spec_dir = Path::new(".");
        let result = generate_music(&params, 42, spec_dir);
        assert!(result.is_err());
    }
}
