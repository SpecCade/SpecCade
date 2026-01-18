//! Audio backend dispatch handler

use super::{get_primary_output, write_output_bytes, DispatchError};
use speccade_spec::{OutputFormat, OutputKind, OutputResult, Spec};
use std::path::{Path, PathBuf};

/// Generate audio using the unified audio backend
pub(super) fn generate_audio(
    spec: &Spec,
    out_root: &Path,
    preview_duration: Option<f64>,
) -> Result<Vec<OutputResult>, DispatchError> {
    let result = if let Some(duration) = preview_duration {
        speccade_backend_audio::generate_preview(spec, duration)
            .map_err(|e| DispatchError::BackendError(format!("Audio generation failed: {}", e)))?
    } else {
        speccade_backend_audio::generate(spec)
            .map_err(|e| DispatchError::BackendError(format!("Audio generation failed: {}", e)))?
    };

    // Write WAV file to the output path from spec
    let primary_output = get_primary_output(spec)?;
    if primary_output.format != OutputFormat::Wav {
        return Err(DispatchError::BackendError(format!(
            "audio_v1 requires primary output format 'wav', got '{}'",
            primary_output.format
        )));
    }
    write_output_bytes(out_root, &primary_output.path, &result.wav.wav_data)?;

    let mut output = OutputResult::tier1(
        OutputKind::Primary,
        OutputFormat::Wav,
        PathBuf::from(&primary_output.path),
        result.wav.pcm_hash,
    );

    // Mark as preview if generated with preview duration
    if preview_duration.is_some() {
        output.preview = Some(true);
    }

    Ok(vec![output])
}
