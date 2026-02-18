//! Shared output helpers for dispatch modules.

use super::{write_output_bytes, DispatchError};
use speccade_spec::{OutputFormat, OutputKind, OutputResult};
use std::path::{Path, PathBuf};

/// Helper to validate and collect primary outputs with required format.
pub(super) fn get_primary_outputs<'a>(
    spec: &'a speccade_spec::Spec,
    required_format: OutputFormat,
    recipe_kind: &str,
) -> Result<Vec<(usize, &'a speccade_spec::OutputSpec)>, DispatchError> {
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(format!(
            "{} requires at least one output of kind 'primary'",
            recipe_kind
        )));
    }

    // Validate format for all primary outputs
    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != required_format {
            return Err(DispatchError::BackendError(format!(
                "{} primary outputs must have format '{}' (outputs[{}].format)",
                recipe_kind,
                match required_format {
                    OutputFormat::Png => "png",
                    OutputFormat::Json => "json",
                    _ => "unknown",
                },
                output_index
            )));
        }
    }

    Ok(primary_outputs)
}

/// Helper to validate and collect metadata outputs.
pub(super) fn get_metadata_outputs<'a>(
    spec: &'a speccade_spec::Spec,
    recipe_kind: &str,
) -> Result<Vec<(usize, &'a speccade_spec::OutputSpec)>, DispatchError> {
    let metadata_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Metadata)
        .collect();

    // Validate format for all metadata outputs
    for (output_index, output_spec) in &metadata_outputs {
        if output_spec.format != OutputFormat::Json {
            return Err(DispatchError::BackendError(format!(
                "{} metadata outputs must have format 'json' (outputs[{}].format)",
                recipe_kind, output_index
            )));
        }
    }

    Ok(metadata_outputs)
}

/// Helper to write PNG data for all primary outputs.
pub(super) fn write_primary_png_outputs(
    out_root: &Path,
    primary_outputs: &[(usize, &speccade_spec::OutputSpec)],
    png_data: &[u8],
    hash: &str,
) -> Result<Vec<OutputResult>, DispatchError> {
    let mut outputs = Vec::new();
    for (_, output_spec) in primary_outputs {
        write_output_bytes(out_root, &output_spec.path, png_data)?;
        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            hash.to_string(),
        ));
    }
    Ok(outputs)
}

/// Helper to serialize and write metadata JSON for all metadata outputs.
pub(super) fn write_metadata_outputs<T: serde::Serialize>(
    out_root: &Path,
    metadata_outputs: &[(usize, &speccade_spec::OutputSpec)],
    metadata: &T,
) -> Result<Vec<OutputResult>, DispatchError> {
    let mut outputs = Vec::new();
    for (_, output_spec) in metadata_outputs {
        let metadata_json = serde_json::to_string_pretty(metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        let metadata_hash = blake3::hash(metadata_json.as_bytes()).to_hex().to_string();
        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Json,
            PathBuf::from(&output_spec.path),
            metadata_hash,
        ));
    }
    Ok(outputs)
}
