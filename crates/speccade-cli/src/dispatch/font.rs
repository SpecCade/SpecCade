//! Font backend dispatch handlers.

use super::output_helpers::{
    get_metadata_outputs, get_primary_outputs, write_metadata_outputs, write_primary_png_outputs,
};
use super::{DispatchError, DispatchResult};
use speccade_spec::{OutputFormat, OutputResult, Spec, StageTiming};
use std::path::Path;
use std::time::Instant;

/// Generate bitmap font outputs using the texture backend.
pub(super) fn generate_font_bitmap(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_font_bitmap()
        .map_err(|e| DispatchError::BackendError(format!("Invalid bitmap font params: {}", e)))?;

    let result =
        speccade_backend_texture::generate_bitmap_font(&params, spec.seed).map_err(|e| {
            DispatchError::BackendError(format!("Bitmap font generation failed: {}", e))
        })?;

    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "font.bitmap_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "font.bitmap_v1")?;

    let mut outputs =
        write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    Ok(outputs)
}

/// Generate bitmap font outputs with profiling instrumentation.
pub(super) fn generate_font_bitmap_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_font_bitmap()
        .map_err(|e| DispatchError::BackendError(format!("Invalid bitmap font params: {}", e)))?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    let render_start = Instant::now();
    let result =
        speccade_backend_texture::generate_bitmap_font(&params, spec.seed).map_err(|e| {
            DispatchError::BackendError(format!("Bitmap font generation failed: {}", e))
        })?;
    stages.push(StageTiming::new(
        "pack_and_render",
        render_start.elapsed().as_millis() as u64,
    ));

    let write_start = Instant::now();
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "font.bitmap_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "font.bitmap_v1")?;

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
