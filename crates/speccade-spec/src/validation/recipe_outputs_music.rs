//! Music recipe output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::{OutputFormat, OutputKind};
use crate::recipe::music::{parse_effect_name, TrackerFormat};
use crate::spec::Spec;
use crate::validation::BudgetProfile;

use super::recipe_outputs::validate_primary_output_present;

/// Validates music outputs with the default budget profile.
#[allow(dead_code)]
pub(super) fn validate_music_outputs(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
    validate_music_outputs_with_budget(spec, recipe, &BudgetProfile::default(), result)
}

/// Validates music outputs with a specific budget profile.
pub(super) fn validate_music_outputs_with_budget(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    budget: &BudgetProfile,
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

    validate_music_outputs_common_with_budget(
        spec,
        &recipe.kind,
        params.format,
        &params.instruments,
        budget,
        result,
    );
    validate_tracker_song_semantics(&params, result);
}

/// Validates music compose outputs with the default budget profile.
#[allow(dead_code)]
pub(super) fn validate_music_compose_outputs(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
    validate_music_compose_outputs_with_budget(spec, recipe, &BudgetProfile::default(), result)
}

/// Validates music compose outputs with a specific budget profile.
pub(super) fn validate_music_compose_outputs_with_budget(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    budget: &BudgetProfile,
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

    validate_music_outputs_common_with_budget(
        spec,
        &recipe.kind,
        params.format,
        &params.instruments,
        budget,
        result,
    );
    validate_tracker_song_compose_semantics(&params, result);
}

/// Validates common music outputs with the default budget profile.
#[allow(dead_code)]
fn validate_music_outputs_common(
    spec: &Spec,
    recipe_kind: &str,
    format: crate::recipe::music::TrackerFormat,
    instruments: &[crate::recipe::music::TrackerInstrument],
    result: &mut ValidationResult,
) {
    validate_music_outputs_common_with_budget(
        spec,
        recipe_kind,
        format,
        instruments,
        &BudgetProfile::default(),
        result,
    )
}

/// Validates common music outputs with a specific budget profile.
fn validate_music_outputs_common_with_budget(
    spec: &Spec,
    recipe_kind: &str,
    format: crate::recipe::music::TrackerFormat,
    instruments: &[crate::recipe::music::TrackerInstrument],
    budget: &BudgetProfile,
    result: &mut ValidationResult,
) {
    validate_primary_output_present(spec, result);

    // Check instrument count against budget
    let max_instruments = match format {
        crate::recipe::music::TrackerFormat::Xm => budget.music.xm_max_instruments as usize,
        crate::recipe::music::TrackerFormat::It => budget.music.it_max_instruments as usize,
    };

    if instruments.len() > max_instruments {
        result.add_error(ValidationError::with_path(
            ErrorCode::BudgetExceeded,
            format!(
                "instruments count {} exceeds budget limit of {} for {:?} format (profile: {})",
                instruments.len(),
                max_instruments,
                format,
                budget.name
            ),
            "recipe.params.instruments",
        ));
    }

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

fn validate_common_tracker_params(
    format: TrackerFormat,
    bpm: u16,
    speed: u8,
    channels: u8,
    path_prefix: &str,
    result: &mut ValidationResult,
) {
    if !(32..=255).contains(&bpm) {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidRecipeParams,
            format!("bpm must be 32-255, got {}", bpm),
            format!("{}.bpm", path_prefix),
        ));
    }
    if !(1..=31).contains(&speed) {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidRecipeParams,
            format!("speed must be 1-31, got {}", speed),
            format!("{}.speed", path_prefix),
        ));
    }

    let max_channels = match format {
        TrackerFormat::Xm => 32,
        TrackerFormat::It => 64,
    };
    if channels == 0 || channels > max_channels {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidRecipeParams,
            format!(
                "channels must be 1-{} for {:?} format, got {}",
                max_channels, format, channels
            ),
            format!("{}.channels", path_prefix),
        ));
    }
}

fn validate_tracker_song_semantics(
    params: &crate::recipe::music::MusicTrackerSongV1Params,
    result: &mut ValidationResult,
) {
    validate_common_tracker_params(
        params.format,
        params.bpm,
        params.speed,
        params.channels,
        "recipe.params",
        result,
    );

    match params.format {
        TrackerFormat::Xm => {
            if params.instruments.len() > 128 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "XM supports at most 128 instruments, got {}",
                        params.instruments.len()
                    ),
                    "recipe.params.instruments",
                ));
            }
            if params.patterns.len() > 256 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!("XM supports at most 256 patterns, got {}", params.patterns.len()),
                    "recipe.params.patterns",
                ));
            }
        }
        TrackerFormat::It => {
            if params.instruments.len() > 99 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "IT supports at most 99 instruments/samples, got {}",
                        params.instruments.len()
                    ),
                    "recipe.params.instruments",
                ));
            }
            if params.patterns.len() > 200 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!("IT supports at most 200 patterns, got {}", params.patterns.len()),
                    "recipe.params.patterns",
                ));
            }
        }
    }

    for (name, pattern) in &params.patterns {
        if pattern.rows == 0 {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("pattern '{}' must have rows > 0", name),
                format!("recipe.params.patterns.{}.rows", name),
            ));
            continue;
        }
        if params.format == TrackerFormat::Xm && pattern.rows > 256 {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!(
                    "pattern '{}' has {} rows; XM supports at most 256",
                    name, pattern.rows
                ),
                format!("recipe.params.patterns.{}.rows", name),
            ));
        }
        if params.format == TrackerFormat::It && pattern.rows > 200 {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!(
                    "pattern '{}' has {} rows; IT supports at most 200",
                    name, pattern.rows
                ),
                format!("recipe.params.patterns.{}.rows", name),
            ));
        }

        for (channel, note) in pattern.flat_notes() {
            if channel >= params.channels {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "pattern '{}' note channel {} exceeds configured channels {}",
                        name, channel, params.channels
                    ),
                    format!("recipe.params.patterns.{}", name),
                ));
            }
            if note.row >= pattern.rows {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "pattern '{}' note row {} is out of range for {} rows",
                        name, note.row, pattern.rows
                    ),
                    format!("recipe.params.patterns.{}", name),
                ));
            }
            if let Some(vol) = note.vol {
                if vol > 64 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "pattern '{}' note volume {} is out of range (0-64)",
                            name, vol
                        ),
                        format!("recipe.params.patterns.{}", name),
                    ));
                }
            }

            let trimmed_note = note.note.trim();
            let is_note_off = matches!(trimmed_note, "OFF" | "===" | "^^^");
            if !is_note_off && note.inst as usize >= params.instruments.len() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "pattern '{}' references instrument {} but only {} instrument(s) are defined",
                        name,
                        note.inst,
                        params.instruments.len()
                    ),
                    format!("recipe.params.patterns.{}", name),
                ));
            }

            if note.effect.is_some() && note.effect_name.is_some() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "pattern '{}' note at row {} channel {} cannot set both effect and effect_name",
                        name, note.row, channel
                    ),
                    format!("recipe.params.patterns.{}", name),
                ));
            }

            if let Some([x, y]) = note.effect_xy {
                if x > 0x0F || y > 0x0F {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "pattern '{}' note at row {} channel {} has invalid effect_xy [{}, {}] (nibbles must be 0-15)",
                            name, note.row, channel, x, y
                        ),
                        format!("recipe.params.patterns.{}", name),
                    ));
                }
            }

            if let Some(effect_name) = &note.effect_name {
                match parse_effect_name(effect_name, note.param, note.effect_xy) {
                    Some(effect) => {
                        let validation = match params.format {
                            TrackerFormat::Xm => effect.validate_xm(),
                            TrackerFormat::It => effect.validate_it(),
                        };
                        if let Err(err) = validation {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::InvalidRecipeParams,
                                format!(
                                    "pattern '{}' note at row {} channel {} has invalid effect '{}': {}",
                                    name, note.row, channel, effect_name, err
                                ),
                                format!("recipe.params.patterns.{}", name),
                            ));
                        }
                    }
                    None => {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!(
                                "pattern '{}' note at row {} channel {} has unknown effect_name '{}'",
                                name, note.row, channel, effect_name
                            ),
                            format!("recipe.params.patterns.{}", name),
                        ));
                    }
                }
            }
        }
    }

    let mut order_count: usize = 0;
    for (idx, entry) in params.arrangement.iter().enumerate() {
        if entry.repeat == 0 {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                "arrangement repeat must be >= 1",
                format!("recipe.params.arrangement[{}].repeat", idx),
            ));
            continue;
        }
        if !params.patterns.contains_key(&entry.pattern) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!(
                    "arrangement references unknown pattern '{}'",
                    entry.pattern
                ),
                format!("recipe.params.arrangement[{}].pattern", idx),
            ));
        }
        order_count = order_count.saturating_add(entry.repeat as usize);
    }

    if params.format == TrackerFormat::Xm && order_count > 256 {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidRecipeParams,
            format!(
                "XM order table supports at most 256 entries, arrangement expands to {}",
                order_count
            ),
            "recipe.params.arrangement",
        ));
    }

    if params.r#loop {
        if let Some(restart) = params.restart_position {
            if restart as usize >= order_count.max(1) {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "restart_position {} is out of range for arrangement length {}",
                        restart,
                        order_count.max(1)
                    ),
                    "recipe.params.restart_position",
                ));
            }
        }
    }

    for (idx, auto) in params.automation.iter().enumerate() {
        match auto {
            crate::recipe::music::AutomationEntry::VolumeFade {
                pattern,
                channel,
                start_row,
                end_row,
                start_vol,
                end_vol,
            } => {
                if *channel >= params.channels {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "volume_fade channel {} exceeds configured channels {}",
                            channel, params.channels
                        ),
                        format!("recipe.params.automation[{}].channel", idx),
                    ));
                }
                if start_row >= end_row {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        "volume_fade start_row must be less than end_row",
                        format!("recipe.params.automation[{}]", idx),
                    ));
                }
                if *start_vol > 64 || *end_vol > 64 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "volume_fade volumes must be 0-64 (got start={}, end={})",
                            start_vol, end_vol
                        ),
                        format!("recipe.params.automation[{}]", idx),
                    ));
                }
                match params.patterns.get(pattern) {
                    Some(pat) => {
                        if *end_row >= pat.rows {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::InvalidRecipeParams,
                                format!(
                                    "volume_fade row range {}..={} exceeds pattern '{}' rows {}",
                                    start_row, end_row, pattern, pat.rows
                                ),
                                format!("recipe.params.automation[{}]", idx),
                            ));
                        }
                    }
                    None => {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("volume_fade references unknown pattern '{}'", pattern),
                            format!("recipe.params.automation[{}].pattern", idx),
                        ));
                    }
                }
            }
            crate::recipe::music::AutomationEntry::TempoChange { pattern, row, bpm } => {
                if !params.patterns.contains_key(pattern) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("tempo_change references unknown pattern '{}'", pattern),
                        format!("recipe.params.automation[{}].pattern", idx),
                    ));
                } else if let Some(pat) = params.patterns.get(pattern) {
                    if *row >= pat.rows {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!(
                                "tempo_change row {} exceeds pattern '{}' rows {}",
                                row, pattern, pat.rows
                            ),
                            format!("recipe.params.automation[{}].row", idx),
                        ));
                    }
                }
                if *bpm < 32 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("tempo_change bpm must be 32-255, got {}", bpm),
                        format!("recipe.params.automation[{}].bpm", idx),
                    ));
                }
            }
        }
    }
}

fn validate_tracker_song_compose_semantics(
    params: &crate::recipe::music::MusicTrackerSongComposeV1Params,
    result: &mut ValidationResult,
) {
    validate_common_tracker_params(
        params.format,
        params.bpm,
        params.speed,
        params.channels,
        "recipe.params",
        result,
    );

    let mut order_count: usize = 0;
    for (idx, entry) in params.arrangement.iter().enumerate() {
        if entry.repeat == 0 {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                "arrangement repeat must be >= 1",
                format!("recipe.params.arrangement[{}].repeat", idx),
            ));
            continue;
        }
        if !params.patterns.contains_key(&entry.pattern) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!(
                    "arrangement references unknown compose pattern '{}'",
                    entry.pattern
                ),
                format!("recipe.params.arrangement[{}].pattern", idx),
            ));
        }
        order_count = order_count.saturating_add(entry.repeat as usize);
    }

    if params.format == TrackerFormat::Xm && order_count > 256 {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidRecipeParams,
            format!(
                "XM order table supports at most 256 entries, arrangement expands to {}",
                order_count
            ),
            "recipe.params.arrangement",
        ));
    }

    if params.r#loop {
        if let Some(restart) = params.restart_position {
            if restart as usize >= order_count.max(1) {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "restart_position {} is out of range for arrangement length {}",
                        restart,
                        order_count.max(1)
                    ),
                    "recipe.params.restart_position",
                ));
            }
        }
    }
}
