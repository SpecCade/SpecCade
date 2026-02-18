//! UI backend dispatch handlers.

use super::output_helpers::{
    get_metadata_outputs, get_primary_outputs, write_metadata_outputs, write_primary_png_outputs,
};
use super::{DispatchError, DispatchResult};
use speccade_spec::{OutputFormat, OutputResult, Spec, StageTiming};
use std::path::Path;
use std::time::Instant;

/// Generate nine-slice panel outputs using the texture backend.
pub(super) fn generate_ui_nine_slice(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_ui_nine_slice()
        .map_err(|e| DispatchError::BackendError(format!("Invalid UI nine-slice params: {}", e)))?;

    let result = speccade_backend_texture::generate_nine_slice(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Nine-slice generation failed: {}", e)))?;

    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "ui.nine_slice_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "ui.nine_slice_v1")?;

    let mut outputs =
        write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    Ok(outputs)
}

/// Generate nine-slice panel outputs with profiling instrumentation.
pub(super) fn generate_ui_nine_slice_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_ui_nine_slice()
        .map_err(|e| DispatchError::BackendError(format!("Invalid UI nine-slice params: {}", e)))?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    let render_start = Instant::now();
    let result = speccade_backend_texture::generate_nine_slice(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Nine-slice generation failed: {}", e)))?;
    stages.push(StageTiming::new(
        "generate_nine_slice",
        render_start.elapsed().as_millis() as u64,
    ));

    let write_start = Instant::now();
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "ui.nine_slice_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "ui.nine_slice_v1")?;

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

/// Generate icon set outputs using the texture backend.
pub(super) fn generate_ui_icon_set(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_ui_icon_set()
        .map_err(|e| DispatchError::BackendError(format!("Invalid UI icon set params: {}", e)))?;

    let result = speccade_backend_texture::generate_icon_set(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Icon set generation failed: {}", e)))?;

    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "ui.icon_set_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "ui.icon_set_v1")?;

    let mut outputs =
        write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    Ok(outputs)
}

/// Generate icon set outputs with profiling instrumentation.
pub(super) fn generate_ui_icon_set_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_ui_icon_set()
        .map_err(|e| DispatchError::BackendError(format!("Invalid UI icon set params: {}", e)))?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    let render_start = Instant::now();
    let result = speccade_backend_texture::generate_icon_set(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Icon set generation failed: {}", e)))?;
    stages.push(StageTiming::new(
        "pack_and_render",
        render_start.elapsed().as_millis() as u64,
    ));

    let write_start = Instant::now();
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "ui.icon_set_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "ui.icon_set_v1")?;

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

/// Generate item card outputs using the texture backend.
pub(super) fn generate_ui_item_card(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_ui_item_card()
        .map_err(|e| DispatchError::BackendError(format!("Invalid UI item card params: {}", e)))?;

    let result = speccade_backend_texture::generate_item_card(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Item card generation failed: {}", e)))?;

    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "ui.item_card_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "ui.item_card_v1")?;

    let mut outputs =
        write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    Ok(outputs)
}

/// Generate item card outputs with profiling instrumentation.
pub(super) fn generate_ui_item_card_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_ui_item_card()
        .map_err(|e| DispatchError::BackendError(format!("Invalid UI item card params: {}", e)))?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    let render_start = Instant::now();
    let result = speccade_backend_texture::generate_item_card(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Item card generation failed: {}", e)))?;
    stages.push(StageTiming::new(
        "generate_item_card",
        render_start.elapsed().as_millis() as u64,
    ));

    let write_start = Instant::now();
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "ui.item_card_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "ui.item_card_v1")?;

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

/// Generate damage number sprite outputs using the texture backend.
pub(super) fn generate_ui_damage_number(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_ui_damage_number().map_err(|e| {
        DispatchError::BackendError(format!("Invalid UI damage number params: {}", e))
    })?;

    let result =
        speccade_backend_texture::generate_damage_number(&params, spec.seed).map_err(|e| {
            DispatchError::BackendError(format!("Damage number generation failed: {}", e))
        })?;

    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "ui.damage_number_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "ui.damage_number_v1")?;

    let mut outputs =
        write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    Ok(outputs)
}

/// Generate damage number sprite outputs with profiling instrumentation.
pub(super) fn generate_ui_damage_number_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_ui_damage_number().map_err(|e| {
        DispatchError::BackendError(format!("Invalid UI damage number params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    let render_start = Instant::now();
    let result =
        speccade_backend_texture::generate_damage_number(&params, spec.seed).map_err(|e| {
            DispatchError::BackendError(format!("Damage number generation failed: {}", e))
        })?;
    stages.push(StageTiming::new(
        "generate_damage_number",
        render_start.elapsed().as_millis() as u64,
    ));

    let write_start = Instant::now();
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "ui.damage_number_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "ui.damage_number_v1")?;

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
