//! Sprite backend dispatch handlers.

use super::output_helpers::{
    get_metadata_outputs, get_primary_outputs, write_metadata_outputs, write_primary_png_outputs,
};
use super::{DispatchError, DispatchResult};
use speccade_spec::{OutputFormat, OutputResult, Spec, StageTiming};
use std::path::Path;
use std::time::Instant;

/// Generate spritesheet atlas outputs using the texture backend.
pub(super) fn generate_sprite_sheet(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_sprite_sheet()
        .map_err(|e| DispatchError::BackendError(format!("Invalid sprite sheet params: {}", e)))?;

    let result =
        speccade_backend_texture::generate_sprite_sheet(&params, spec.seed).map_err(|e| {
            DispatchError::BackendError(format!("Sprite sheet generation failed: {}", e))
        })?;

    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "sprite.sheet_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "sprite.sheet_v1")?;

    let mut outputs =
        write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    Ok(outputs)
}

/// Generate spritesheet atlas outputs with profiling instrumentation.
pub(super) fn generate_sprite_sheet_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_sprite_sheet()
        .map_err(|e| DispatchError::BackendError(format!("Invalid sprite sheet params: {}", e)))?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: pack_and_render
    let render_start = Instant::now();
    let result =
        speccade_backend_texture::generate_sprite_sheet(&params, spec.seed).map_err(|e| {
            DispatchError::BackendError(format!("Sprite sheet generation failed: {}", e))
        })?;
    stages.push(StageTiming::new(
        "pack_and_render",
        render_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "sprite.sheet_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "sprite.sheet_v1")?;

    let mut outputs =
        write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    stages.push(StageTiming::new(
        "write_outputs",
        write_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}

/// Generate sprite animation metadata.
pub(super) fn generate_sprite_animation(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_sprite_animation().map_err(|e| {
        DispatchError::BackendError(format!("Invalid sprite animation params: {}", e))
    })?;

    // Generate metadata
    let metadata = params.to_metadata();

    // Sprite animations output JSON as primary (not PNG)
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Json, "sprite.animation_v1")?;

    write_metadata_outputs(out_root, &primary_outputs, &metadata)
}

/// Generate sprite animation metadata with profiling instrumentation.
pub(super) fn generate_sprite_animation_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_sprite_animation().map_err(|e| {
        DispatchError::BackendError(format!("Invalid sprite animation params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: generate_metadata
    let gen_start = Instant::now();
    let metadata = params.to_metadata();
    stages.push(StageTiming::new(
        "generate_metadata",
        gen_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Json, "sprite.animation_v1")?;
    let outputs = write_metadata_outputs(out_root, &primary_outputs, &metadata)?;

    stages.push(StageTiming::new(
        "write_outputs",
        write_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}
