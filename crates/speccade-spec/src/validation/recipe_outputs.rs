//! Recipe-specific output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::{OutputFormat, OutputKind};
use crate::recipe::Recipe;
use crate::spec::Spec;
use crate::validation::BudgetProfile;

use super::recipe_outputs_audio::validate_audio_outputs_with_budget;
use super::recipe_outputs_music::{
    validate_music_compose_outputs_with_budget, validate_music_outputs_with_budget,
};
use super::recipe_outputs_texture::validate_texture_procedural_outputs_with_budget;

/// Validates outputs for a recipe using the default budget profile.
pub(super) fn validate_outputs_for_recipe(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
    validate_outputs_for_recipe_with_budget(spec, recipe, &BudgetProfile::default(), result)
}

/// Validates outputs for a recipe with a specific budget profile.
pub(super) fn validate_outputs_for_recipe_with_budget(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    budget: &BudgetProfile,
    result: &mut ValidationResult,
) {
    match recipe.kind.as_str() {
        "audio_v1" => validate_audio_outputs_with_budget(spec, recipe, budget, result),
        "music.tracker_song_v1" => validate_music_outputs_with_budget(spec, recipe, budget, result),
        "music.tracker_song_compose_v1" => {
            validate_music_compose_outputs_with_budget(spec, recipe, budget, result)
        }
        "texture.procedural_v1" => {
            validate_texture_procedural_outputs_with_budget(spec, recipe, budget, result)
        }
        "texture.trimsheet_v1" => {
            validate_texture_trimsheet_outputs(spec, recipe, result)
        }
        "static_mesh.blender_primitives_v1" => {
            validate_static_mesh_blender_primitives(recipe, result);
            validate_single_primary_output_format(spec, OutputFormat::Glb, result);
        }
        "skeletal_mesh.blender_rigged_mesh_v1" => {
            validate_skeletal_mesh_blender_rigged(recipe, result);
            validate_single_primary_output_format(spec, OutputFormat::Glb, result);
        }
        "skeletal_animation.blender_clip_v1" => {
            validate_skeletal_animation_blender_clip(recipe, result);
            validate_single_primary_output_format(spec, OutputFormat::Glb, result);
        }
        "skeletal_animation.blender_rigged_v1" => {
            validate_skeletal_animation_blender_rigged(recipe, result);
            validate_single_primary_output_format(spec, OutputFormat::Glb, result);
        }
        _ if recipe.kind.starts_with("texture.") => {
            result.add_error(ValidationError::with_path(
                ErrorCode::UnsupportedRecipeKind,
                format!(
                    "unsupported texture recipe kind '{}'; use 'texture.procedural_v1' or 'texture.trimsheet_v1'",
                    recipe.kind
                ),
                "recipe.kind",
            ));
            validate_primary_output_present(spec, result);
        }
        _ => validate_primary_output_present(spec, result),
    }
}

pub(crate) fn validate_primary_output_present(spec: &Spec, result: &mut ValidationResult) {
    let has_primary = spec.outputs.iter().any(|o| o.kind == OutputKind::Primary);
    if !has_primary {
        result.add_error(ValidationError::with_path(
            ErrorCode::NoPrimaryOutput,
            "at least one output must have kind 'primary'",
            "outputs",
        ));
    }
}

pub(crate) fn validate_single_primary_output_format(
    spec: &Spec,
    expected_format: OutputFormat,
    result: &mut ValidationResult,
) {
    validate_primary_output_present(spec, result);

    let primary_outputs: Vec<(usize, &crate::output::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.len() != 1 {
        result.add_error(ValidationError::with_path(
            ErrorCode::OutputValidationFailed,
            format!(
                "expected exactly 1 primary output, got {}",
                primary_outputs.len()
            ),
            "outputs",
        ));
        return;
    }

    let (index, output) = primary_outputs[0];
    if output.format != expected_format {
        result.add_error(ValidationError::with_path(
            ErrorCode::OutputValidationFailed,
            format!(
                "primary output format must be '{}' for this recipe, got '{}'",
                expected_format, output.format
            ),
            format!("outputs[{}].format", index),
        ));
    }
}

// =============================================================================
// Tier-2 Recipe Params Validation
// =============================================================================

/// Validates params for `static_mesh.blender_primitives_v1` recipe.
///
/// This validates that the params match the expected schema and rejects
/// unknown fields.
fn validate_static_mesh_blender_primitives(recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_static_mesh_blender_primitives() {
        Ok(params) => {
            // Validate dimensions are positive
            for (i, &dim) in params.dimensions.iter().enumerate() {
                if dim <= 0.0 {
                    let axis = ["X", "Y", "Z"][i];
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("dimensions[{}] ({}) must be positive, got {}", i, axis, dim),
                        format!("recipe.params.dimensions[{}]", i),
                    ));
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }
}

/// Validates params for `skeletal_mesh.blender_rigged_mesh_v1` recipe.
///
/// This validates that the params match the expected schema and rejects
/// unknown fields.
fn validate_skeletal_mesh_blender_rigged(recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_skeletal_mesh_blender_rigged_mesh() {
        Ok(params) => {
            // Validate that either skeleton_preset or skeleton is provided
            if params.skeleton_preset.is_none() && params.skeleton.is_empty() {
                // Only warn if both body_parts and parts are also empty
                // (legacy specs may rely on external armature)
                if params.body_parts.is_empty() && params.parts.is_empty() {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        "either 'skeleton_preset', 'skeleton', 'body_parts', or 'parts' must be provided",
                        "recipe.params",
                    ));
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }
}

/// Validates params for `skeletal_animation.blender_clip_v1` recipe.
///
/// This validates that the params match the expected schema and rejects
/// unknown fields.
fn validate_skeletal_animation_blender_clip(recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_skeletal_animation_blender_clip() {
        Ok(params) => {
            // Validate duration is positive
            if params.duration_seconds <= 0.0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "duration_seconds must be positive, got {}",
                        params.duration_seconds
                    ),
                    "recipe.params.duration_seconds",
                ));
            }

            // Validate fps is reasonable
            if params.fps == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "fps must be greater than 0",
                    "recipe.params.fps",
                ));
            }

            // Validate clip_name is not empty
            if params.clip_name.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "clip_name must not be empty",
                    "recipe.params.clip_name",
                ));
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }
}

/// Validates params for `skeletal_animation.blender_rigged_v1` recipe.
///
/// This validates that the params match the expected schema and rejects
/// unknown fields.
fn validate_skeletal_animation_blender_rigged(recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_skeletal_animation_blender_rigged() {
        Ok(params) => {
            // Validate duration_frames is positive
            if params.duration_frames == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "duration_frames must be greater than 0",
                    "recipe.params.duration_frames",
                ));
            }

            // Validate fps is reasonable
            if params.fps == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "fps must be greater than 0",
                    "recipe.params.fps",
                ));
            }

            // Validate clip_name is not empty
            if params.clip_name.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "clip_name must not be empty",
                    "recipe.params.clip_name",
                ));
            }

            // If duration_seconds is provided, it should be positive
            if let Some(duration_seconds) = params.duration_seconds {
                if duration_seconds <= 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "duration_seconds must be positive, got {}",
                            duration_seconds
                        ),
                        "recipe.params.duration_seconds",
                    ));
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }
}

/// Validates outputs for `texture.trimsheet_v1` recipe.
///
/// Trimsheet specs require:
/// - Exactly one primary output with PNG format
/// - Optional metadata output(s) with JSON format
fn validate_texture_trimsheet_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    // Validate params parse correctly
    match recipe.as_texture_trimsheet() {
        Ok(params) => {
            // Validate resolution is positive
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }

            // Validate tiles have unique ids
            let mut seen_ids = std::collections::HashSet::new();
            for (i, tile) in params.tiles.iter().enumerate() {
                if !seen_ids.insert(&tile.id) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("duplicate tile id: '{}'", tile.id),
                        format!("recipe.params.tiles[{}].id", i),
                    ));
                }

                // Validate tile dimensions
                if tile.width == 0 || tile.height == 0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "tile '{}' dimensions must be positive, got {}x{}",
                            tile.id, tile.width, tile.height
                        ),
                        format!("recipe.params.tiles[{}]", i),
                    ));
                }

                // Validate tile fits in atlas with padding
                let padded_width = tile.width + params.padding * 2;
                let padded_height = tile.height + params.padding * 2;
                if padded_width > params.resolution[0] || padded_height > params.resolution[1] {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "tile '{}' ({}x{}) with padding {} is too large for atlas ({}x{})",
                            tile.id, tile.width, tile.height, params.padding,
                            params.resolution[0], params.resolution[1]
                        ),
                        format!("recipe.params.tiles[{}]", i),
                    ));
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    // Validate outputs
    validate_primary_output_present(spec, result);

    // Check primary outputs are PNG
    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "texture.trimsheet_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }

        // Check metadata outputs are JSON
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "texture.trimsheet_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}
