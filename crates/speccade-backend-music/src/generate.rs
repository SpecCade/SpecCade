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

/// Get the synthesis configuration for an instrument, handling ref, wav, or inline synthesis.
///
/// This resolves either an inline synthesis definition, a WAV sample path,
/// or loads from an external reference file.
pub(crate) fn get_instrument_synthesis(
    instr: &TrackerInstrument,
    _spec_dir: &Path,
) -> Result<InstrumentSynthesis, GenerateError> {
    if let Some(ref ref_path) = instr.r#ref {
        // Load external spec file
        load_instrument_from_ref(ref_path, _spec_dir)
    } else if let Some(ref wav_path) = instr.wav {
        // WAV sample at instrument level - convert to Sample synthesis
        // Use base_note from instrument level if specified
        Ok(InstrumentSynthesis::Sample {
            path: wav_path.clone(),
            base_note: instr.base_note.clone(),
        })
    } else if let Some(ref synthesis) = instr.synthesis {
        // Use inline synthesis
        Ok(synthesis.clone())
    } else {
        // Default to a basic synthesis if neither is provided
        Err(GenerateError::InstrumentError(format!(
            "Instrument '{}' must have either 'ref', 'wav', or 'synthesis' defined",
            instr.name
        )))
    }
}

/// Load instrument synthesis from an external spec file.
///
/// This function resolves external instrument references by:
/// 1. Loading the referenced spec file (relative to spec_dir)
/// 2. Parsing it as an audio_v1 recipe
/// 3. Converting the synthesis to an InstrumentSynthesis
///
/// Supported external spec formats:
/// - `audio_v1` recipes with oscillator-based synthesis are converted directly
/// - Complex synthesis types (FM, Karplus-Strong, etc.) return an error suggesting
///   to inline the instrument or use a pre-generated WAV sample
fn load_instrument_from_ref(
    ref_path: &str,
    spec_dir: &Path,
) -> Result<InstrumentSynthesis, GenerateError> {
    use speccade_spec::Spec;

    // Resolve the path relative to spec_dir
    let full_path = spec_dir.join(ref_path);

    // Read the file
    let json_content = std::fs::read_to_string(&full_path).map_err(|e| {
        GenerateError::InstrumentError(format!(
            "Failed to read external instrument spec '{}': {}",
            ref_path, e
        ))
    })?;

    // Parse as a Spec
    let spec: Spec = serde_json::from_str(&json_content).map_err(|e| {
        GenerateError::InstrumentError(format!(
            "Failed to parse external instrument spec '{}': {}",
            ref_path, e
        ))
    })?;

    // Get the recipe
    let recipe = spec.recipe.as_ref().ok_or_else(|| {
        GenerateError::InstrumentError(format!(
            "External instrument spec '{}' has no recipe",
            ref_path
        ))
    })?;

    // Check recipe kind - must be audio_v1
    if recipe.kind != "audio_v1" && recipe.kind != "audio.v1" {
        return Err(GenerateError::InstrumentError(format!(
            "External instrument spec '{}' has unsupported recipe kind '{}', expected 'audio_v1'",
            ref_path, recipe.kind
        )));
    }

    // Parse as AudioV1Params
    let audio_params: speccade_spec::recipe::audio::AudioV1Params =
        serde_json::from_value(recipe.params.clone()).map_err(|e| {
            GenerateError::InstrumentError(format!(
                "Failed to parse audio params from '{}': {}",
                ref_path, e
            ))
        })?;

    // Get the first layer's synthesis (instruments typically have one layer)
    let layer = audio_params.layers.first().ok_or_else(|| {
        GenerateError::InstrumentError(format!(
            "External instrument spec '{}' has no synthesis layers",
            ref_path
        ))
    })?;

    // Convert audio::Synthesis to music::InstrumentSynthesis
    convert_audio_synthesis_to_instrument(&layer.synthesis, ref_path)
}

/// Convert an audio::Synthesis to a music::InstrumentSynthesis.
///
/// This handles the mapping between the more expressive audio synthesis types
/// and the simpler tracker instrument synthesis types.
fn convert_audio_synthesis_to_instrument(
    synthesis: &speccade_spec::recipe::audio::Synthesis,
    ref_path: &str,
) -> Result<InstrumentSynthesis, GenerateError> {
    use speccade_spec::recipe::audio::{Synthesis, Waveform};

    match synthesis {
        // Simple oscillator types map directly
        Synthesis::Oscillator {
            waveform,
            duty,
            ..
        } => {
            match waveform {
                Waveform::Sine => Ok(InstrumentSynthesis::Sine),
                Waveform::Square => Ok(InstrumentSynthesis::Square),
                Waveform::Pulse => {
                    Ok(InstrumentSynthesis::Pulse {
                        duty_cycle: duty.unwrap_or(0.5),
                    })
                }
                Waveform::Sawtooth => Ok(InstrumentSynthesis::Sawtooth),
                Waveform::Triangle => Ok(InstrumentSynthesis::Triangle),
            }
        }

        // Noise maps to periodic or non-periodic based on filter presence
        Synthesis::NoiseBurst { filter, .. } => {
            // If there's a filter, it's more "tonal", so use periodic
            Ok(InstrumentSynthesis::Noise {
                periodic: filter.is_some(),
            })
        }

        // Complex synthesis types are not directly supported in tracker instruments.
        // These should be pre-rendered to WAV or inlined with simple synthesis.
        Synthesis::FmSynth { .. } => Err(GenerateError::InstrumentError(format!(
            "External instrument '{}' uses FM synthesis which cannot be directly converted. \
             Please either: 1) Inline a simple synthesis (pulse/saw/sine/etc.) in the music spec, \
             2) Pre-render the instrument to a WAV file and use 'wav' instead of 'ref', or \
             3) Use the audio backend to generate a sample first.",
            ref_path
        ))),

        Synthesis::KarplusStrong { .. } => Err(GenerateError::InstrumentError(format!(
            "External instrument '{}' uses Karplus-Strong synthesis which cannot be directly converted. \
             Please either: 1) Inline a simple synthesis (pulse/saw/sine/etc.) in the music spec, \
             2) Pre-render the instrument to a WAV file and use 'wav' instead of 'ref', or \
             3) Use the audio backend to generate a sample first.",
            ref_path
        ))),

        Synthesis::Additive { .. } => Err(GenerateError::InstrumentError(format!(
            "External instrument '{}' uses additive synthesis which cannot be directly converted. \
             Please either: 1) Inline a simple synthesis in the music spec, or \
             2) Pre-render the instrument to a WAV file.",
            ref_path
        ))),

        Synthesis::MultiOscillator { .. } => Err(GenerateError::InstrumentError(format!(
            "External instrument '{}' uses multi-oscillator synthesis which cannot be directly converted. \
             Consider using a single oscillator or pre-rendering to WAV.",
            ref_path
        ))),

        Synthesis::PitchedBody { .. } => Err(GenerateError::InstrumentError(format!(
            "External instrument '{}' uses pitched body synthesis which cannot be directly converted. \
             Please pre-render to a WAV file.",
            ref_path
        ))),

        Synthesis::Metallic { .. } => Err(GenerateError::InstrumentError(format!(
            "External instrument '{}' uses metallic synthesis which cannot be directly converted. \
             Please pre-render to a WAV file.",
            ref_path
        ))),
    }
}

/// Extract the base_note from an InstrumentSynthesis::Sample, if present.
///
/// Note: This is kept for backwards compatibility with Sample synthesis type
/// which has base_note inside the enum variant. For other synthesis types,
/// base_note is now at the TrackerInstrument level.
///
/// # Returns
/// - `Some(note_name)` if Sample synthesis has base_note set
/// - `None` otherwise
#[allow(dead_code)]
pub(crate) fn get_synthesis_base_note(synthesis: &InstrumentSynthesis) -> Option<&str> {
    // TODO: Once the spec schema adds base_note to all synthesis types, implement this:
    //
    // match synthesis {
    //     InstrumentSynthesis::Pulse { base_note, .. } => base_note.as_deref(),
    //     InstrumentSynthesis::Triangle { base_note, .. } => base_note.as_deref(),
    //     InstrumentSynthesis::Sawtooth { base_note, .. } => base_note.as_deref(),
    //     InstrumentSynthesis::Sine { base_note, .. } => base_note.as_deref(),
    //     InstrumentSynthesis::Noise { base_note, .. } => base_note.as_deref(),
    //     InstrumentSynthesis::Sample { base_note, .. } => base_note.as_deref(),
    // }
    //
    // For now, only Sample type has base_note in the schema:
    match synthesis {
        InstrumentSynthesis::Sample { base_note, .. } => base_note.as_deref(),
        _ => None, // Other synthesis types don't have base_note in schema yet
    }
}

/// Extract effect code and parameter from PatternEffect, handling effect_xy nibbles.
///
/// This is used by both XM and IT pattern converters.
#[allow(dead_code)]
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
    use speccade_spec::recipe::audio::Envelope;
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
            synthesis: Some(InstrumentSynthesis::Pulse { duty_cycle: 0.5 }),
            envelope: envelope.clone(),
            default_volume: Some(64),
            ..Default::default()
        };

        let mut notes = HashMap::new();
        notes.insert("0".to_string(), vec![
            PatternNote {
                row: 0,
                note: "C4".to_string(),
                inst: 0,
                vol: Some(64),
                ..Default::default()
            },
            PatternNote {
                row: 4,
                note: "E4".to_string(),
                inst: 0,
                vol: Some(64),
                ..Default::default()
            },
            PatternNote {
                row: 8,
                note: "G4".to_string(),
                inst: 0,
                vol: Some(64),
                ..Default::default()
            },
            PatternNote {
                row: 12,
                note: "OFF".to_string(),
                inst: 0,
                ..Default::default()
            },
        ]);
        let pattern = TrackerPattern {
            rows: 16,
            notes: Some(notes),
            data: None,
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
            ..Default::default()
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

    // =========================================================================
    // External instrument reference tests
    // =========================================================================

    #[test]
    fn test_external_ref_file_not_found() {
        // Test that missing external reference returns a clear error
        let result = load_instrument_from_ref("nonexistent/file.json", Path::new("."));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to read"));
    }

    #[test]
    fn test_convert_oscillator_synthesis() {
        use speccade_spec::recipe::audio::{Synthesis, Waveform};

        // Test sine conversion
        let sine = Synthesis::Oscillator {
            waveform: Waveform::Sine,
            frequency: 440.0,
            freq_sweep: None,
            detune: None,
            duty: None,
        };
        let result = convert_audio_synthesis_to_instrument(&sine, "test.json");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), InstrumentSynthesis::Sine));

        // Test square conversion
        let square = Synthesis::Oscillator {
            waveform: Waveform::Square,
            frequency: 440.0,
            freq_sweep: None,
            detune: None,
            duty: None,
        };
        let result = convert_audio_synthesis_to_instrument(&square, "test.json");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), InstrumentSynthesis::Square));

        // Test sawtooth conversion
        let saw = Synthesis::Oscillator {
            waveform: Waveform::Sawtooth,
            frequency: 440.0,
            freq_sweep: None,
            detune: None,
            duty: None,
        };
        let result = convert_audio_synthesis_to_instrument(&saw, "test.json");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), InstrumentSynthesis::Sawtooth));

        // Test triangle conversion
        let tri = Synthesis::Oscillator {
            waveform: Waveform::Triangle,
            frequency: 440.0,
            freq_sweep: None,
            detune: None,
            duty: None,
        };
        let result = convert_audio_synthesis_to_instrument(&tri, "test.json");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), InstrumentSynthesis::Triangle));

        // Test pulse with duty cycle
        let pulse = Synthesis::Oscillator {
            waveform: Waveform::Pulse,
            frequency: 440.0,
            freq_sweep: None,
            detune: None,
            duty: Some(0.25),
        };
        let result = convert_audio_synthesis_to_instrument(&pulse, "test.json");
        assert!(result.is_ok());
        if let InstrumentSynthesis::Pulse { duty_cycle } = result.unwrap() {
            assert!((duty_cycle - 0.25).abs() < 0.001);
        } else {
            panic!("Expected Pulse synthesis");
        }
    }

    #[test]
    fn test_convert_noise_synthesis() {
        use speccade_spec::recipe::audio::{Filter, NoiseType, Synthesis};

        // Test noise without filter
        let noise = Synthesis::NoiseBurst {
            noise_type: NoiseType::White,
            filter: None,
        };
        let result = convert_audio_synthesis_to_instrument(&noise, "test.json");
        assert!(result.is_ok());
        if let InstrumentSynthesis::Noise { periodic } = result.unwrap() {
            assert!(!periodic);
        } else {
            panic!("Expected Noise synthesis");
        }

        // Test noise with filter (more tonal, so periodic)
        let noise_filtered = Synthesis::NoiseBurst {
            noise_type: NoiseType::Pink,
            filter: Some(Filter::Lowpass {
                cutoff: 1000.0,
                resonance: 0.5,
                cutoff_end: None,
            }),
        };
        let result = convert_audio_synthesis_to_instrument(&noise_filtered, "test.json");
        assert!(result.is_ok());
        if let InstrumentSynthesis::Noise { periodic } = result.unwrap() {
            assert!(periodic);
        } else {
            panic!("Expected Noise synthesis");
        }
    }

    #[test]
    fn test_complex_synthesis_returns_error() {
        use speccade_spec::recipe::audio::Synthesis;

        // FM synthesis should return helpful error
        let fm = Synthesis::FmSynth {
            carrier_freq: 440.0,
            modulator_freq: 880.0,
            modulation_index: 2.0,
            freq_sweep: None,
        };
        let result = convert_audio_synthesis_to_instrument(&fm, "test.json");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("FM synthesis"));

        // Karplus-Strong should return helpful error
        let karplus = Synthesis::KarplusStrong {
            frequency: 220.0,
            decay: 0.998,
            blend: 0.5,
        };
        let result = convert_audio_synthesis_to_instrument(&karplus, "test.json");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Karplus-Strong"));
    }

    #[test]
    fn test_load_external_oscillator_spec() {
        // Test loading a real external spec file with oscillator synthesis
        // The saw_lead.json uses oscillator synthesis which should convert successfully
        let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../golden/speccade/specs/audio");

        // Check if the file exists (may not exist in all CI environments)
        let ref_path = "saw_lead.json";
        let full_path = spec_dir.join(ref_path);
        if full_path.exists() {
            let result = load_instrument_from_ref(ref_path, &spec_dir);
            assert!(result.is_ok(), "Failed to load saw_lead.json: {:?}", result);
            // Should convert to Sawtooth synthesis
            assert!(matches!(result.unwrap(), InstrumentSynthesis::Sawtooth));
        }
    }

    #[test]
    fn test_load_external_karplus_spec_returns_error() {
        // Test loading a spec file with Karplus-Strong synthesis (should fail with helpful error)
        let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../golden/speccade/specs/audio");

        let ref_path = "bass_pluck.json";
        let full_path = spec_dir.join(ref_path);
        if full_path.exists() {
            let result = load_instrument_from_ref(ref_path, &spec_dir);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("Karplus-Strong"));
        }
    }
}
