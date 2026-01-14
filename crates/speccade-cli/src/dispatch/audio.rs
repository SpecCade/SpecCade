//! Audio backend dispatch handler

use super::{get_primary_output, write_output_bytes, DispatchError};
use speccade_spec::{OutputFormat, OutputKind, OutputResult, Spec};
use std::path::{Path, PathBuf};

/// Generate audio using the unified audio backend
pub(super) fn generate_audio(spec: &Spec, out_root: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_audio::generate(spec)
        .map_err(|e| DispatchError::BackendError(format!("Audio generation failed: {}", e)))?;

    // Write WAV file to the output path from spec
    let primary_output = get_primary_output(spec)?;
    if primary_output.format != OutputFormat::Wav {
        return Err(DispatchError::BackendError(format!(
            "audio_v1 requires primary output format 'wav', got '{}'",
            primary_output.format
        )));
    }
    write_output_bytes(out_root, &primary_output.path, &result.wav.wav_data)?;

    Ok(vec![OutputResult::tier1(
        OutputKind::Primary,
        OutputFormat::Wav,
        PathBuf::from(&primary_output.path),
        result.wav.pcm_hash,
    )])
}
