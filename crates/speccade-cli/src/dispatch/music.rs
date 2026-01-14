//! Music backend dispatch handler

use super::{write_output_bytes, DispatchError};
use speccade_spec::recipe::music::{MusicTrackerSongV1Params, TrackerFormat};
use speccade_spec::{OutputFormat, OutputKind, OutputResult, Spec};
use std::path::{Path, PathBuf};

/// Generate music using the music backend
pub(super) fn generate_music(
    spec: &Spec,
    out_root: &Path,
    spec_dir: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_music_tracker_song()
        .map_err(|e| DispatchError::BackendError(format!("Invalid music params: {}", e)))?;

    generate_music_from_params(&params, &recipe.kind, spec, out_root, spec_dir)
}

pub(super) fn generate_music_compose(
    spec: &Spec,
    out_root: &Path,
    spec_dir: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_music_tracker_song_compose()
        .map_err(|e| DispatchError::BackendError(format!("Invalid music compose params: {}", e)))?;
    let expanded = speccade_backend_music::expand_compose(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Compose expansion failed: {}", e)))?;

    generate_music_from_params(&expanded, &recipe.kind, spec, out_root, spec_dir)
}

fn generate_music_from_params(
    params: &MusicTrackerSongV1Params,
    recipe_kind: &str,
    spec: &Spec,
    out_root: &Path,
    spec_dir: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let primary_outputs: Vec<&speccade_spec::OutputSpec> = spec
        .outputs
        .iter()
        .filter(|o| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "No primary output specified".to_string(),
        ));
    }

    // Keep a defensive check even though validate_for_generate enforces this.
    let expected = match params.format {
        TrackerFormat::Xm => OutputFormat::Xm,
        TrackerFormat::It => OutputFormat::It,
    };

    // Single-output mode (legacy behavior).
    if primary_outputs.len() == 1 {
        let primary_output = primary_outputs[0];
        if primary_output.format != expected {
            return Err(DispatchError::BackendError(format!(
                "{} requires primary output format '{}', got '{}'",
                recipe_kind, expected, primary_output.format
            )));
        }

        let result = speccade_backend_music::generate_music(params, spec.seed, spec_dir)
            .map_err(|e| DispatchError::BackendError(format!("Music generation failed: {}", e)))?;

        write_output_bytes(out_root, &primary_output.path, &result.data)?;

        let mut outputs = vec![OutputResult::tier1(
            OutputKind::Primary,
            primary_output.format,
            PathBuf::from(&primary_output.path),
            result.hash,
        )];

        if let Some(loop_report) = result.loop_report.as_ref() {
            let loop_path = format!("{}.loops.json", primary_output.path);
            let bytes = serde_json::to_vec_pretty(loop_report).map_err(|e| {
                DispatchError::BackendError(format!(
                    "Failed to serialize music loop report JSON: {}",
                    e
                ))
            })?;
            write_output_bytes(out_root, &loop_path, &bytes)?;
            let hash = blake3::hash(&bytes).to_hex().to_string();
            outputs.push(OutputResult::tier1(
                OutputKind::Metadata,
                OutputFormat::Json,
                PathBuf::from(&loop_path),
                hash,
            ));
        }

        return Ok(outputs);
    }

    // Multi-output mode: one XM and/or one IT primary output.
    let mut seen_xm = false;
    let mut seen_it = false;
    let mut results = Vec::new();

    for output in primary_outputs {
        let format = match output.format {
            OutputFormat::Xm => {
                if seen_xm {
                    return Err(DispatchError::BackendError(format!(
                        "Duplicate primary output format 'xm' for {}",
                        recipe_kind
                    )));
                }
                seen_xm = true;
                TrackerFormat::Xm
            }
            OutputFormat::It => {
                if seen_it {
                    return Err(DispatchError::BackendError(format!(
                        "Duplicate primary output format 'it' for {}",
                        recipe_kind
                    )));
                }
                seen_it = true;
                TrackerFormat::It
            }
            _ => {
                return Err(DispatchError::BackendError(format!(
                    "{} primary outputs must have format 'xm' or 'it', got '{}'",
                    recipe_kind, output.format
                )))
            }
        };

        let mut per_output_params = params.clone();
        per_output_params.format = format;

        let gen = speccade_backend_music::generate_music(&per_output_params, spec.seed, spec_dir)
            .map_err(|e| {
            DispatchError::BackendError(format!("Music generation failed: {}", e))
        })?;

        // Defensive: ensure backend output matches requested format.
        let actual_format = match gen.extension {
            "xm" => OutputFormat::Xm,
            "it" => OutputFormat::It,
            _ => {
                return Err(DispatchError::BackendError(format!(
                    "Unknown music format: {}",
                    gen.extension
                )))
            }
        };
        if actual_format != output.format {
            return Err(DispatchError::BackendError(format!(
                "Music backend returned '{}' but output was declared as '{}'",
                actual_format, output.format
            )));
        }

        write_output_bytes(out_root, &output.path, &gen.data)?;
        results.push(OutputResult::tier1(
            OutputKind::Primary,
            output.format,
            PathBuf::from(&output.path),
            gen.hash,
        ));

        if let Some(loop_report) = gen.loop_report.as_ref() {
            let loop_path = format!("{}.loops.json", output.path);
            let bytes = serde_json::to_vec_pretty(loop_report).map_err(|e| {
                DispatchError::BackendError(format!(
                    "Failed to serialize music loop report JSON: {}",
                    e
                ))
            })?;
            write_output_bytes(out_root, &loop_path, &bytes)?;
            let hash = blake3::hash(&bytes).to_hex().to_string();
            results.push(OutputResult::tier1(
                OutputKind::Metadata,
                OutputFormat::Json,
                PathBuf::from(&loop_path),
                hash,
            ));
        }
    }

    Ok(results)
}
