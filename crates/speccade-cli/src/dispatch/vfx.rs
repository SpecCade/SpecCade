//! VFX backend dispatch handlers.

use super::output_helpers::{
    get_metadata_outputs, get_primary_outputs, write_metadata_outputs, write_primary_png_outputs,
};
use super::{DispatchError, DispatchResult};
use speccade_spec::{OutputFormat, OutputResult, Spec, StageTiming};
use std::path::Path;
use std::time::Instant;

/// Generate VFX flipbook outputs using the texture backend.
pub(super) fn generate_vfx_flipbook(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_vfx_flipbook()
        .map_err(|e| DispatchError::BackendError(format!("Invalid VFX flipbook params: {}", e)))?;

    let result =
        speccade_backend_texture::generate_vfx_flipbook(&params, spec.seed).map_err(|e| {
            DispatchError::BackendError(format!("VFX flipbook generation failed: {}", e))
        })?;

    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "vfx.flipbook_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "vfx.flipbook_v1")?;

    let mut outputs =
        write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    Ok(outputs)
}

/// Generate VFX flipbook outputs with profiling instrumentation.
pub(super) fn generate_vfx_flipbook_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_vfx_flipbook()
        .map_err(|e| DispatchError::BackendError(format!("Invalid VFX flipbook params: {}", e)))?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: generate_frames
    let gen_start = Instant::now();
    let result =
        speccade_backend_texture::generate_vfx_flipbook(&params, spec.seed).map_err(|e| {
            DispatchError::BackendError(format!("VFX flipbook generation failed: {}", e))
        })?;
    stages.push(StageTiming::new(
        "generate_frames",
        gen_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "vfx.flipbook_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "vfx.flipbook_v1")?;

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

/// Generate VFX particle profile metadata output.
pub(super) fn generate_vfx_particle_profile(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_vfx_particle_profile().map_err(|e| {
        DispatchError::BackendError(format!("Invalid VFX particle profile params: {}", e))
    })?;

    let result =
        speccade_backend_texture::generate_particle_profile(&params, spec.seed).map_err(|e| {
            DispatchError::BackendError(format!("VFX particle profile generation failed: {}", e))
        })?;

    // Particle profile outputs are JSON (metadata-only)
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Json, "vfx.particle_profile_v1")?;

    write_metadata_outputs(out_root, &primary_outputs, &result.metadata)
}

/// Generate VFX particle profile metadata output with profiling instrumentation.
pub(super) fn generate_vfx_particle_profile_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_vfx_particle_profile().map_err(|e| {
        DispatchError::BackendError(format!("Invalid VFX particle profile params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: generate_metadata
    let gen_start = Instant::now();
    let result =
        speccade_backend_texture::generate_particle_profile(&params, spec.seed).map_err(|e| {
            DispatchError::BackendError(format!("VFX particle profile generation failed: {}", e))
        })?;
    stages.push(StageTiming::new(
        "generate_metadata",
        gen_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Json, "vfx.particle_profile_v1")?;
    let outputs = write_metadata_outputs(out_root, &primary_outputs, &result.metadata)?;

    stages.push(StageTiming::new(
        "write_outputs",
        write_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}
