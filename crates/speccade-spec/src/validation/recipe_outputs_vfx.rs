//! VFX recipe output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::{OutputFormat, OutputKind};
use crate::recipe::Recipe;
use crate::spec::Spec;

use super::recipe_outputs::validate_primary_output_present;

/// Validates outputs for `vfx.flipbook_v1` recipe.
pub(super) fn validate_vfx_flipbook_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_vfx_flipbook() {
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
                "vfx.flipbook_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "vfx.flipbook_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `vfx.particle_profile_v1` recipe.
pub(super) fn validate_vfx_particle_profile_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_vfx_particle_profile() {
        Ok(params) => {
            if let Some(intensity) = params.intensity {
                if intensity < 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("intensity must be non-negative, got {}", intensity),
                        "recipe.params.intensity",
                    ));
                }
            }
            if let Some(strength) = params.distortion_strength {
                if !(0.0..=1.0).contains(&strength) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "distortion_strength must be in [0.0, 1.0], got {}",
                            strength
                        ),
                        "recipe.params.distortion_strength",
                    ));
                }
            }
            if let Some(tint) = params.color_tint {
                for (i, &c) in tint.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("color_tint[{}] must be in [0.0, 1.0], got {}", i, c),
                            format!("recipe.params.color_tint[{}]", i),
                        ));
                    }
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
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "vfx.particle_profile_v1 primary outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}
