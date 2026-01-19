//! Texture backend dispatch handler

use super::{write_output_bytes, DispatchError, DispatchResult};
use speccade_spec::{OutputFormat, OutputKind, OutputResult, Spec, StageTiming};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Generate trimsheet atlas outputs using the texture backend.
pub(super) fn generate_texture_trimsheet(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_trimsheet().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture trimsheet params: {}", e))
    })?;

    let result = speccade_backend_texture::generate_trimsheet(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Trimsheet generation failed: {}", e))
    })?;

    let mut outputs = Vec::new();

    // Find primary output for the atlas PNG
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.trimsheet_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.trimsheet_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        write_output_bytes(out_root, &output_spec.path, &result.png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            result.hash.clone(),
        ));
    }

    // Find metadata output (JSON sidecar)
    let metadata_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Metadata)
        .collect();

    for (output_index, output_spec) in &metadata_outputs {
        if output_spec.format != OutputFormat::Json {
            return Err(DispatchError::BackendError(format!(
                "texture.trimsheet_v1 metadata outputs must have format 'json' (outputs[{}].format)",
                output_index
            )));
        }

        let metadata_json = serde_json::to_string_pretty(&result.metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        // Calculate hash of the JSON metadata
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

/// Generate trimsheet atlas outputs with profiling instrumentation.
pub(super) fn generate_texture_trimsheet_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_trimsheet().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture trimsheet params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: pack_and_render
    let render_start = Instant::now();
    let result = speccade_backend_texture::generate_trimsheet(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Trimsheet generation failed: {}", e))
    })?;
    stages.push(StageTiming::new(
        "pack_and_render",
        render_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let mut outputs = Vec::new();

    // Find primary output for the atlas PNG
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.trimsheet_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.trimsheet_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        write_output_bytes(out_root, &output_spec.path, &result.png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            result.hash.clone(),
        ));
    }

    // Find metadata output (JSON sidecar)
    let metadata_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Metadata)
        .collect();

    for (output_index, output_spec) in &metadata_outputs {
        if output_spec.format != OutputFormat::Json {
            return Err(DispatchError::BackendError(format!(
                "texture.trimsheet_v1 metadata outputs must have format 'json' (outputs[{}].format)",
                output_index
            )));
        }

        let metadata_json = serde_json::to_string_pretty(&result.metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        // Calculate hash of the JSON metadata
        let metadata_hash = blake3::hash(metadata_json.as_bytes()).to_hex().to_string();

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Json,
            PathBuf::from(&output_spec.path),
            metadata_hash,
        ));
    }

    stages.push(StageTiming::new(
        "write_outputs",
        write_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}

/// Generate procedural texture outputs using the texture backend.
pub(super) fn generate_texture_procedural(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_procedural().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture procedural params: {}", e))
    })?;

    let nodes = speccade_backend_texture::generate_graph(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Procedural texture generation failed: {}", e))
    })?;

    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.procedural_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    let mut outputs = Vec::with_capacity(primary_outputs.len());

    for (output_index, output_spec) in primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        let source = output_spec.source.as_ref().ok_or_else(|| {
            DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must set 'source' (outputs[{}].source)",
                output_index
            ))
        })?;

        let value = nodes.get(source).ok_or_else(|| {
            DispatchError::BackendError(format!(
                "outputs[{}].source '{}' does not match any node id",
                output_index, source
            ))
        })?;

        let (png_data, hash) = speccade_backend_texture::encode_graph_value_png(value)
            .map_err(|e| DispatchError::BackendError(format!("PNG encoding failed: {}", e)))?;

        write_output_bytes(out_root, &output_spec.path, &png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            hash,
        ));
    }

    Ok(outputs)
}

/// Generate procedural texture outputs with profiling instrumentation.
pub(super) fn generate_texture_procedural_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_procedural().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture procedural params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: render_graph
    let render_start = Instant::now();
    let nodes = speccade_backend_texture::generate_graph(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Procedural texture generation failed: {}", e))
    })?;
    stages.push(StageTiming::new(
        "render_graph",
        render_start.elapsed().as_millis() as u64,
    ));

    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.procedural_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    // Stage: encode_outputs
    let encode_start = Instant::now();
    let mut outputs = Vec::with_capacity(primary_outputs.len());

    for (output_index, output_spec) in primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        let source = output_spec.source.as_ref().ok_or_else(|| {
            DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must set 'source' (outputs[{}].source)",
                output_index
            ))
        })?;

        let value = nodes.get(source).ok_or_else(|| {
            DispatchError::BackendError(format!(
                "outputs[{}].source '{}' does not match any node id",
                output_index, source
            ))
        })?;

        let (png_data, hash) = speccade_backend_texture::encode_graph_value_png(value)
            .map_err(|e| DispatchError::BackendError(format!("PNG encoding failed: {}", e)))?;

        write_output_bytes(out_root, &output_spec.path, &png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            hash,
        ));
    }
    stages.push(StageTiming::new(
        "encode_outputs",
        encode_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}
