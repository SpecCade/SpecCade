//! Music recipe output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::{OutputFormat, OutputKind};
use crate::spec::Spec;

use super::recipe_outputs::validate_primary_output_present;

pub(super) fn validate_music_outputs(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
    let params = match recipe.as_music_tracker_song() {
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

    validate_music_outputs_common(
        spec,
        &recipe.kind,
        params.format,
        &params.instruments,
        result,
    );
}

pub(super) fn validate_music_compose_outputs(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
    let params = match recipe.as_music_tracker_song_compose() {
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

    validate_music_outputs_common(
        spec,
        &recipe.kind,
        params.format,
        &params.instruments,
        result,
    );
}

fn validate_music_outputs_common(
    spec: &Spec,
    recipe_kind: &str,
    format: crate::recipe::music::TrackerFormat,
    instruments: &[crate::recipe::music::TrackerInstrument],
    result: &mut ValidationResult,
) {
    validate_primary_output_present(spec, result);

    // Validate instrument sources are well-formed (matches backend behavior).
    for (idx, instrument) in instruments.iter().enumerate() {
        let mut sources = Vec::new();
        if instrument.r#ref.is_some() {
            sources.push("ref");
        }
        if instrument.wav.is_some() {
            sources.push("wav");
        }
        if instrument.synthesis_audio_v1.is_some() {
            sources.push("synthesis_audio_v1");
        }
        if instrument.synthesis.is_some() {
            sources.push("synthesis");
        }

        if sources.len() != 1 {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!(
                    "music instrument must set exactly one of: ref, wav, synthesis_audio_v1, synthesis (got: {})",
                    if sources.is_empty() {
                        "none".to_string()
                    } else {
                        sources.join(", ")
                    }
                ),
                format!("recipe.params.instruments[{}]", idx),
            ));
        }
    }

    let expected_format = match format {
        crate::recipe::music::TrackerFormat::Xm => OutputFormat::Xm,
        crate::recipe::music::TrackerFormat::It => OutputFormat::It,
    };

    let primary_outputs: Vec<(usize, &crate::output::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return;
    }

    if primary_outputs.len() == 1 {
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
        return;
    }

    // Multi-output mode: allow at most one XM and one IT primary output.
    let mut seen_xm = false;
    let mut seen_it = false;

    for (index, output) in &primary_outputs {
        match output.format {
            OutputFormat::Xm => {
                if seen_xm {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        format!("duplicate primary output format 'xm' for {}", recipe_kind),
                        format!("outputs[{}].format", index),
                    ));
                }
                seen_xm = true;
            }
            OutputFormat::It => {
                if seen_it {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        format!("duplicate primary output format 'it' for {}", recipe_kind),
                        format!("outputs[{}].format", index),
                    ));
                }
                seen_it = true;
            }
            _ => {
                result.add_error(ValidationError::with_path(
                    ErrorCode::OutputValidationFailed,
                    format!(
                        "{} primary outputs must have format 'xm' or 'it'",
                        recipe_kind
                    ),
                    format!("outputs[{}].format", index),
                ));
            }
        }
    }

    if primary_outputs.len() > 2 {
        result.add_error(ValidationError::with_path(
            ErrorCode::OutputValidationFailed,
            format!(
                "{} supports at most 2 primary outputs (xm + it), got {}",
                recipe_kind,
                primary_outputs.len()
            ),
            "outputs",
        ));
    }

    // Defensive: ensure the recipe's declared format is among the requested outputs.
    if !primary_outputs
        .iter()
        .any(|(_, o)| o.format == expected_format)
    {
        result.add_error(ValidationError::with_path(
            ErrorCode::OutputValidationFailed,
            format!(
                "recipe.params.format '{}' must match one of the primary output formats",
                expected_format
            ),
            "recipe.params.format",
        ));
    }
}
