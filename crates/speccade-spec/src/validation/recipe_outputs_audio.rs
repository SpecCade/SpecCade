//! Audio recipe output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::OutputFormat;
use crate::recipe::audio::{Effect, ModulationTarget, Synthesis, Waveform};
use crate::spec::Spec;
use crate::validation::{
    validate_non_negative, validate_positive, validate_range, validate_unit_interval, BudgetProfile,
};

use super::recipe_outputs::validate_single_primary_output_format;

/// Returns true if the target is a post-FX only target (not valid on layer LFOs).
fn is_post_fx_target(target: &ModulationTarget) -> bool {
    matches!(
        target,
        ModulationTarget::DelayTime { .. }
            | ModulationTarget::ReverbSize { .. }
            | ModulationTarget::DistortionDrive { .. }
    )
}

/// Returns true if the target is a layer-only target (not valid on post-FX LFOs).
fn is_layer_only_target(target: &ModulationTarget) -> bool {
    matches!(
        target,
        ModulationTarget::Pitch { .. }
            | ModulationTarget::Volume { .. }
            | ModulationTarget::FilterCutoff { .. }
            | ModulationTarget::Pan { .. }
            | ModulationTarget::PulseWidth { .. }
            | ModulationTarget::FmIndex { .. }
            | ModulationTarget::GrainSize { .. }
            | ModulationTarget::GrainDensity { .. }
    )
}

/// Returns a discriminant name for a modulation target (for duplicate checking).
fn target_discriminant(target: &ModulationTarget) -> &'static str {
    match target {
        ModulationTarget::Pitch { .. } => "pitch",
        ModulationTarget::Volume { .. } => "volume",
        ModulationTarget::FilterCutoff { .. } => "filter_cutoff",
        ModulationTarget::Pan { .. } => "pan",
        ModulationTarget::PulseWidth { .. } => "pulse_width",
        ModulationTarget::FmIndex { .. } => "fm_index",
        ModulationTarget::GrainSize { .. } => "grain_size",
        ModulationTarget::GrainDensity { .. } => "grain_density",
        ModulationTarget::DelayTime { .. } => "delay_time",
        ModulationTarget::ReverbSize { .. } => "reverb_size",
        ModulationTarget::DistortionDrive { .. } => "distortion_drive",
    }
}

/// Checks if there is at least one matching effect for a delay_time LFO target.
/// Delay, MultiTapDelay, Flanger, GranularDelay, and StereoWidener (Haas mode) support delay_time modulation.
fn has_matching_delay_effect(effects: &[Effect]) -> bool {
    use crate::recipe::audio::StereoWidenerMode;
    effects.iter().any(|e| {
        matches!(
            e,
            Effect::Delay { .. }
                | Effect::Flanger { .. }
                | Effect::MultiTapDelay { .. }
                | Effect::GranularDelay { .. }
        ) || matches!(
            e,
            Effect::StereoWidener {
                mode: StereoWidenerMode::Haas,
                ..
            }
        )
    })
}

/// Checks if there is at least one matching effect for a reverb_size LFO target.
fn has_matching_reverb_effect(effects: &[Effect]) -> bool {
    effects.iter().any(|e| matches!(e, Effect::Reverb { .. }))
}

/// Checks if there is at least one matching effect for a distortion_drive LFO target.
fn has_matching_distortion_effect(effects: &[Effect]) -> bool {
    effects
        .iter()
        .any(|e| matches!(e, Effect::Waveshaper { .. } | Effect::TapeSaturation { .. }))
}

/// Checks if the synthesis type supports pulse width modulation.
///
/// Returns true for:
/// - `Synthesis::Oscillator` with `waveform: Square` or `waveform: Pulse`
/// - `Synthesis::MultiOscillator` with at least one oscillator using `waveform: Square` or `waveform: Pulse`
fn synthesis_supports_pulse_width(synthesis: &Synthesis) -> bool {
    match synthesis {
        Synthesis::Oscillator { waveform, .. } => {
            matches!(waveform, Waveform::Square | Waveform::Pulse)
        }
        Synthesis::MultiOscillator { oscillators, .. } => oscillators
            .iter()
            .any(|osc| matches!(osc.waveform, Waveform::Square | Waveform::Pulse)),
        _ => false,
    }
}

/// Validates audio outputs with the default budget profile.
#[allow(dead_code)]
pub(super) fn validate_audio_outputs(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
    validate_audio_outputs_with_budget(spec, recipe, &BudgetProfile::default(), result)
}

/// Validates audio outputs with a specific budget profile.
pub(super) fn validate_audio_outputs_with_budget(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    budget: &BudgetProfile,
    result: &mut ValidationResult,
) {
    let max_audio_duration_seconds = budget.audio.max_duration_seconds;
    let max_audio_layers = budget.audio.max_layers;
    let allowed_sample_rates = &budget.audio.allowed_sample_rates;

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
    } else if params.duration_seconds > max_audio_duration_seconds {
        result.add_error(ValidationError::with_path(
            ErrorCode::BudgetExceeded,
            format!(
                "duration_seconds {} exceeds budget limit of {} (profile: {})",
                params.duration_seconds, max_audio_duration_seconds, budget.name
            ),
            "recipe.params.duration_seconds",
        ));
    }

    if !allowed_sample_rates.contains(&params.sample_rate) {
        result.add_error(ValidationError::with_path(
            ErrorCode::BudgetExceeded,
            format!(
                "sample_rate {} is not in allowed rates {:?} (profile: {})",
                params.sample_rate, allowed_sample_rates, budget.name
            ),
            "recipe.params.sample_rate",
        ));
    }

    if params.layers.len() > max_audio_layers {
        result.add_error(ValidationError::with_path(
            ErrorCode::BudgetExceeded,
            format!(
                "layers count {} exceeds budget limit of {} (profile: {})",
                params.layers.len(),
                max_audio_layers,
                budget.name
            ),
            "recipe.params.layers",
        ));
    }

    // Count expanded layers (supersaw_unison voices expand to multiple virtual layers)
    let expanded_layer_count: usize = params
        .layers
        .iter()
        .map(|layer| match &layer.synthesis {
            Synthesis::SupersawUnison { voices, .. } => (*voices as usize).max(1),
            _ => 1,
        })
        .sum();

    if expanded_layer_count > max_audio_layers {
        result.add_error(ValidationError::with_path(
            ErrorCode::BudgetExceeded,
            format!(
                "total expanded layers (including supersaw_unison voices) {} exceeds budget limit of {} (profile: {})",
                expanded_layer_count, max_audio_layers, budget.name
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
            // Post-FX target check (not valid on layer LFOs)
            // ----------------
            if is_post_fx_target(&lfo.target) {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "{} LFO target is only valid in post_fx_lfos, not in layer LFOs",
                        target_discriminant(&lfo.target)
                    ),
                    format!("recipe.params.layers[{}].lfo.target", i),
                ));
            }

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
                ModulationTarget::PulseWidth { amount } => {
                    // Validate amount range [0.0, 0.49]
                    if let Err(e) = validate_range("amount", *amount, 0.0, 0.49) {
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

                    // Validate synthesis compatibility
                    if !synthesis_supports_pulse_width(&layer.synthesis) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            "pulse_width LFO target is only valid for Oscillator/MultiOscillator with square or pulse waveform",
                            format!("recipe.params.layers[{}].lfo.target", i),
                        ));
                    }
                }
                ModulationTarget::FmIndex { amount } => {
                    // Validate amount is positive
                    if let Err(e) = validate_positive("amount", *amount) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            e.to_string(),
                            format!("recipe.params.layers[{}].lfo.target.amount", i),
                        ));
                    }

                    // Validate synthesis compatibility - only FmSynth is valid
                    if !matches!(layer.synthesis, Synthesis::FmSynth { .. }) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            "fm_index LFO target is only valid for FmSynth synthesis",
                            format!("recipe.params.layers[{}].lfo.target", i),
                        ));
                    }
                }
                ModulationTarget::GrainSize { amount_ms } => {
                    // Validate amount_ms is positive
                    if let Err(e) = validate_positive("amount_ms", *amount_ms) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            e.to_string(),
                            format!("recipe.params.layers[{}].lfo.target.amount_ms", i),
                        ));
                    }

                    // Validate synthesis compatibility - only Granular is valid
                    if !matches!(layer.synthesis, Synthesis::Granular { .. }) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            "grain_size LFO target is only valid for Granular synthesis",
                            format!("recipe.params.layers[{}].lfo.target", i),
                        ));
                    }
                }
                ModulationTarget::GrainDensity { amount } => {
                    // Validate amount is positive
                    if let Err(e) = validate_positive("amount", *amount) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            e.to_string(),
                            format!("recipe.params.layers[{}].lfo.target.amount", i),
                        ));
                    }

                    // Validate synthesis compatibility - only Granular is valid
                    if !matches!(layer.synthesis, Synthesis::Granular { .. }) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            "grain_density LFO target is only valid for Granular synthesis",
                            format!("recipe.params.layers[{}].lfo.target", i),
                        ));
                    }
                }
                ModulationTarget::DelayTime { .. }
                | ModulationTarget::ReverbSize { .. }
                | ModulationTarget::DistortionDrive { .. } => {
                    // Already handled above (post-FX only target error)
                    // No additional validation needed here
                }
            }
        }
    }

    // ========================================================================
    // Post-FX LFO validation
    // ========================================================================
    let mut seen_targets = std::collections::HashSet::new();
    for (i, lfo) in params.post_fx_lfos.iter().enumerate() {
        let path_prefix = format!("recipe.params.post_fx_lfos[{}]", i);

        // ----------------
        // Layer-only target check (not valid on post-FX LFOs)
        // ----------------
        if is_layer_only_target(&lfo.target) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!(
                    "{} LFO target is only valid in layer LFOs, not in post_fx_lfos",
                    target_discriminant(&lfo.target)
                ),
                format!("{}.target", path_prefix),
            ));
        }

        // ----------------
        // Duplicate target check
        // ----------------
        let discriminant = target_discriminant(&lfo.target);
        if !seen_targets.insert(discriminant) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!(
                    "duplicate {} target in post_fx_lfos (max 1 per target)",
                    discriminant
                ),
                format!("{}.target", path_prefix),
            ));
        }

        // ----------------
        // LFO config validation
        // ----------------
        if let Err(e) = validate_positive("rate", lfo.config.rate) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                e.to_string(),
                format!("{}.config.rate", path_prefix),
            ));
        }

        if let Err(e) = validate_unit_interval("depth", lfo.config.depth) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                e.to_string(),
                format!("{}.config.depth", path_prefix),
            ));
        } else if lfo.config.depth == 0.0 {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                "depth must be > 0.0 (otherwise this LFO is a no-op)",
                format!("{}.config.depth", path_prefix),
            ));
        }

        if let Some(phase) = lfo.config.phase {
            if let Err(e) = validate_unit_interval("phase", phase) {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    e.to_string(),
                    format!("{}.config.phase", path_prefix),
                ));
            }
        }

        // ----------------
        // Target-specific validation
        // ----------------
        match &lfo.target {
            ModulationTarget::DelayTime { amount_ms } => {
                // Validate amount_ms is positive
                if let Err(e) = validate_positive("amount_ms", *amount_ms) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        e.to_string(),
                        format!("{}.target.amount_ms", path_prefix),
                    ));
                }

                // Validate that there is at least one matching effect
                if !has_matching_delay_effect(&params.effects) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        "delay_time LFO requires at least one delay effect in effects[]",
                        format!("{}.target", path_prefix),
                    ));
                }
            }
            ModulationTarget::ReverbSize { amount } => {
                // Validate amount is positive
                if let Err(e) = validate_positive("amount", *amount) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        e.to_string(),
                        format!("{}.target.amount", path_prefix),
                    ));
                }

                // Validate that there is at least one matching effect
                if !has_matching_reverb_effect(&params.effects) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        "reverb_size LFO requires at least one reverb effect in effects[]",
                        format!("{}.target", path_prefix),
                    ));
                }
            }
            ModulationTarget::DistortionDrive { amount } => {
                // Validate amount is positive
                if let Err(e) = validate_positive("amount", *amount) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        e.to_string(),
                        format!("{}.target.amount", path_prefix),
                    ));
                }

                // Validate that there is at least one matching effect
                if !has_matching_distortion_effect(&params.effects) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        "distortion_drive LFO requires at least one waveshaper or tape_saturation effect in effects[]",
                        format!("{}.target", path_prefix),
                    ));
                }
            }
            // Layer-only targets are handled above with the is_layer_only_target check
            _ => {}
        }
    }

    validate_single_primary_output_format(spec, OutputFormat::Wav, result);
}
