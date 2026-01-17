//! Recipe-specific output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::{OutputFormat, OutputKind};
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
        "music.tracker_song_v1" => {
            validate_music_outputs_with_budget(spec, recipe, budget, result)
        }
        "music.tracker_song_compose_v1" => {
            validate_music_compose_outputs_with_budget(spec, recipe, budget, result)
        }
        "texture.procedural_v1" => {
            validate_texture_procedural_outputs_with_budget(spec, recipe, budget, result)
        }
        "static_mesh.blender_primitives_v1" => {
            validate_single_primary_output_format(spec, OutputFormat::Glb, result)
        }
        "skeletal_mesh.blender_rigged_mesh_v1" => {
            validate_single_primary_output_format(spec, OutputFormat::Glb, result)
        }
        "skeletal_animation.blender_clip_v1" => {
            validate_single_primary_output_format(spec, OutputFormat::Glb, result)
        }
        _ if recipe.kind.starts_with("texture.") => {
            result.add_error(ValidationError::with_path(
                ErrorCode::UnsupportedRecipeKind,
                format!(
                    "unsupported texture recipe kind '{}'; use 'texture.procedural_v1'",
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
