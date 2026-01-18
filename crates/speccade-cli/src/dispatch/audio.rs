//! Audio backend dispatch handler

use super::waveform::{generate_waveform_png, preview_path_from_primary};
use super::{get_primary_output, write_output_bytes, DispatchError, DispatchResult};
use speccade_backend_audio::wav::extract_pcm_data;
use speccade_spec::{OutputFormat, OutputKind, OutputResult, Spec, StageTiming};
use std::path::{Path, PathBuf};
use std::time::Instant;

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

    let mut outputs = Vec::new();

    // Primary WAV output
    let mut wav_output = OutputResult::tier1(
        OutputKind::Primary,
        OutputFormat::Wav,
        PathBuf::from(&primary_output.path),
        result.wav.pcm_hash.clone(),
    );

    // Mark as preview if generated with preview duration
    if preview_duration.is_some() {
        wav_output.preview = Some(true);
    }
    outputs.push(wav_output);

    // Generate waveform preview PNG
    if let Some(pcm_data) = extract_pcm_data(&result.wav.wav_data) {
        let waveform = generate_waveform_png(pcm_data, result.wav.is_stereo);
        let preview_path = preview_path_from_primary(&primary_output.path);

        write_output_bytes(out_root, &preview_path, &waveform.png_data)?;

        let preview_output = OutputResult::tier1(
            OutputKind::Preview,
            OutputFormat::Png,
            PathBuf::from(&preview_path),
            waveform.hash,
        );
        outputs.push(preview_output);
    }

    Ok(outputs)
}

/// Generate audio with profiling instrumentation.
pub(super) fn generate_audio_profiled(
    spec: &Spec,
    out_root: &Path,
    preview_duration: Option<f64>,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: render_audio
    let render_start = Instant::now();
    let result = if let Some(duration) = preview_duration {
        speccade_backend_audio::generate_preview(spec, duration)
            .map_err(|e| DispatchError::BackendError(format!("Audio generation failed: {}", e)))?
    } else {
        speccade_backend_audio::generate(spec)
            .map_err(|e| DispatchError::BackendError(format!("Audio generation failed: {}", e)))?
    };
    stages.push(StageTiming::new(
        "render_audio",
        render_start.elapsed().as_millis() as u64,
    ));

    // Stage: encode_output (write WAV)
    let encode_start = Instant::now();
    let primary_output = get_primary_output(spec)?;
    if primary_output.format != OutputFormat::Wav {
        return Err(DispatchError::BackendError(format!(
            "audio_v1 requires primary output format 'wav', got '{}'",
            primary_output.format
        )));
    }
    write_output_bytes(out_root, &primary_output.path, &result.wav.wav_data)?;
    stages.push(StageTiming::new(
        "encode_output",
        encode_start.elapsed().as_millis() as u64,
    ));

    let mut outputs = Vec::new();

    let mut wav_output = OutputResult::tier1(
        OutputKind::Primary,
        OutputFormat::Wav,
        PathBuf::from(&primary_output.path),
        result.wav.pcm_hash.clone(),
    );

    if preview_duration.is_some() {
        wav_output.preview = Some(true);
    }
    outputs.push(wav_output);

    // Stage: generate_waveform (preview PNG)
    let waveform_start = Instant::now();
    if let Some(pcm_data) = extract_pcm_data(&result.wav.wav_data) {
        let waveform = generate_waveform_png(pcm_data, result.wav.is_stereo);
        let preview_path = preview_path_from_primary(&primary_output.path);

        write_output_bytes(out_root, &preview_path, &waveform.png_data)?;

        let preview_output = OutputResult::tier1(
            OutputKind::Preview,
            OutputFormat::Png,
            PathBuf::from(&preview_path),
            waveform.hash,
        );
        outputs.push(preview_output);
    }
    stages.push(StageTiming::new(
        "generate_waveform",
        waveform_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}
