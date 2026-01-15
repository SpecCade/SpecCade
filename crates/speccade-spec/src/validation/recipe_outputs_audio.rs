//! Audio recipe output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::OutputFormat;
use crate::spec::Spec;
use crate::validation::{
    validate_non_negative, validate_positive, validate_range, validate_unit_interval,
};
use crate::recipe::audio::ModulationTarget;

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

        if let Some(lfo) = &layer.lfo {
            // ----------------
            // LFO config
            // ----------------
            if let Err(e) = validate_positive("rate", lfo.config.rate) {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    e.to_string(),
                    format!("recipe.params.layers[{}].lfo.config.rate", i),
                ));
            }

            if let Err(e) = validate_unit_interval("depth", lfo.config.depth) {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    e.to_string(),
                    format!("recipe.params.layers[{}].lfo.config.depth", i),
                ));
            } else if lfo.config.depth == 0.0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "depth must be > 0.0 (otherwise this LFO is a no-op)",
                    format!("recipe.params.layers[{}].lfo.config.depth", i),
                ));
            }

            if let Some(phase) = lfo.config.phase {
                if let Err(e) = validate_unit_interval("phase", phase) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        e.to_string(),
                        format!("recipe.params.layers[{}].lfo.config.phase", i),
                    ));
                }
            }

            // ----------------
            // LFO target
            // ----------------
            match &lfo.target {
                ModulationTarget::Pitch { semitones } => {
                    if let Err(e) = validate_positive("semitones", *semitones) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            e.to_string(),
                            format!("recipe.params.layers[{}].lfo.target.semitones", i),
                        ));
                    }
                }
                ModulationTarget::Volume { amount } => {
                    if let Err(e) = validate_unit_interval("amount", *amount) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            e.to_string(),
                            format!("recipe.params.layers[{}].lfo.target.amount", i),
                        ));
                    } else if *amount == 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            "amount must be > 0.0 (otherwise this LFO is a no-op)",
                            format!("recipe.params.layers[{}].lfo.target.amount", i),
                        ));
                    }
                }
                ModulationTarget::FilterCutoff { amount } => {
                    if let Err(e) = validate_positive("amount", *amount) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            e.to_string(),
                            format!("recipe.params.layers[{}].lfo.target.amount", i),
                        ));
                    }

                    if layer.filter.is_none() {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            "filter_cutoff LFO requires layers[].filter to be present (otherwise this LFO is a no-op)",
                            format!("recipe.params.layers[{}].lfo.target", i),
                        ));
                    }
                }
                ModulationTarget::Pan { amount } => {
                    if let Err(e) = validate_unit_interval("amount", *amount) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            e.to_string(),
                            format!("recipe.params.layers[{}].lfo.target.amount", i),
                        ));
                    } else if *amount == 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            "amount must be > 0.0 (otherwise this LFO is a no-op)",
                            format!("recipe.params.layers[{}].lfo.target.amount", i),
                        ));
                    }
                }
            }
        }
    }

    validate_single_primary_output_format(spec, OutputFormat::Wav, result);
}
