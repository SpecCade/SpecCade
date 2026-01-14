//! Audio recipe output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::OutputFormat;
use crate::spec::Spec;
use crate::validation::{
    validate_non_negative, validate_positive, validate_range, validate_unit_interval,
};

use super::recipe_outputs::validate_single_primary_output_format;

pub(super) fn validate_audio_outputs(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
    const MAX_AUDIO_DURATION_SECONDS: f64 = 30.0;
    const MAX_AUDIO_LAYERS: usize = 32;

    let params = match recipe.as_audio() {
        Ok(params) => params,
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    };

    if let Err(e) = validate_positive("duration_seconds", params.duration_seconds) {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidRecipeParams,
            e.to_string(),
            "recipe.params.duration_seconds",
        ));
    } else if params.duration_seconds > MAX_AUDIO_DURATION_SECONDS {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidRecipeParams,
            format!(
                "duration_seconds must be <= {}, got {}",
                MAX_AUDIO_DURATION_SECONDS, params.duration_seconds
            ),
            "recipe.params.duration_seconds",
        ));
    }

    match params.sample_rate {
        22050 | 44100 | 48000 => {}
        other => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("sample_rate must be 22050, 44100, or 48000, got {}", other),
                "recipe.params.sample_rate",
            ));
        }
    }

    if params.layers.len() > MAX_AUDIO_LAYERS {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidRecipeParams,
            format!(
                "layers must have at most {} entries, got {}",
                MAX_AUDIO_LAYERS,
                params.layers.len()
            ),
            "recipe.params.layers",
        ));
    }

    for (i, layer) in params.layers.iter().enumerate() {
        if let Err(e) = validate_unit_interval("volume", layer.volume) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                e.to_string(),
                format!("recipe.params.layers[{}].volume", i),
            ));
        }
        if let Err(e) = validate_range("pan", layer.pan, -1.0, 1.0) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                e.to_string(),
                format!("recipe.params.layers[{}].pan", i),
            ));
        }
        if let Some(delay) = layer.delay {
            if let Err(e) = validate_non_negative("delay", delay) {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    e.to_string(),
                    format!("recipe.params.layers[{}].delay", i),
                ));
            } else if delay > params.duration_seconds {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "delay must be <= duration_seconds ({}), got {}",
                        params.duration_seconds, delay
                    ),
                    format!("recipe.params.layers[{}].delay", i),
                ));
            }
        }

        if let Err(e) = validate_non_negative("attack", layer.envelope.attack) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                e.to_string(),
                format!("recipe.params.layers[{}].envelope.attack", i),
            ));
        }
        if let Err(e) = validate_non_negative("decay", layer.envelope.decay) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                e.to_string(),
                format!("recipe.params.layers[{}].envelope.decay", i),
            ));
        }
        if let Err(e) = validate_unit_interval("sustain", layer.envelope.sustain) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                e.to_string(),
                format!("recipe.params.layers[{}].envelope.sustain", i),
            ));
        }
        if let Err(e) = validate_non_negative("release", layer.envelope.release) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                e.to_string(),
                format!("recipe.params.layers[{}].envelope.release", i),
            ));
        }
    }

    validate_single_primary_output_format(spec, OutputFormat::Wav, result);
}
