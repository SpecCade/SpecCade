//! Sprite recipe output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::{OutputFormat, OutputKind};
use crate::recipe::Recipe;
use crate::spec::Spec;

use super::recipe_outputs::validate_primary_output_present;

/// Validates outputs for `sprite.sheet_v1` recipe.
pub(super) fn validate_sprite_sheet_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_sprite_sheet() {
        Ok(params) => {
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
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "sprite.sheet_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "sprite.sheet_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `sprite.animation_v1` recipe.
pub(super) fn validate_sprite_animation_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_sprite_animation() {
        Ok(_params) => {}
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "sprite.animation_v1 primary outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `sprite.render_from_mesh_v1` recipe.
pub(super) fn validate_sprite_render_from_mesh_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_sprite_render_from_mesh() {
        Ok(params) => {
            if params.frame_resolution[0] > 1024 || params.frame_resolution[1] > 1024 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "frame_resolution must be at most 1024x1024, got [{}, {}]",
                        params.frame_resolution[0], params.frame_resolution[1]
                    ),
                    "recipe.params.frame_resolution",
                ));
            }
            if params.frame_resolution[0] == 0 || params.frame_resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "frame_resolution must be positive, got [{}, {}]",
                        params.frame_resolution[0], params.frame_resolution[1]
                    ),
                    "recipe.params.frame_resolution",
                ));
            }
            if params.rotation_angles.len() > 16 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "rotation_angles count must be at most 16, got {}",
                        params.rotation_angles.len()
                    ),
                    "recipe.params.rotation_angles",
                ));
            }
            if params.rotation_angles.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "rotation_angles must not be empty",
                    "recipe.params.rotation_angles",
                ));
            }
            if params.camera_distance <= 0.0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "camera_distance must be positive, got {}",
                        params.camera_distance
                    ),
                    "recipe.params.camera_distance",
                ));
            }
            if !(-90.0..=90.0).contains(&params.camera_elevation) {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "camera_elevation must be in range [-90, 90] degrees, got {}",
                        params.camera_elevation
                    ),
                    "recipe.params.camera_elevation",
                ));
            }
            for (i, &c) in params.background_color.iter().enumerate() {
                if !(0.0..=1.0).contains(&c) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("background_color[{}] must be in [0, 1], got {}", i, c),
                        format!("recipe.params.background_color[{}]", i),
                    ));
                }
            }
            for (i, &dim) in params.mesh.dimensions.iter().enumerate() {
                if dim <= 0.0 {
                    let axis = ["X", "Y", "Z"][i];
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "mesh.dimensions[{}] ({}) must be positive, got {}",
                            i, axis, dim
                        ),
                        format!("recipe.params.mesh.dimensions[{}]", i),
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

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "sprite.render_from_mesh_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "sprite.render_from_mesh_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}
