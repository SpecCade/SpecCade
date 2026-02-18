//! IT (Impulse Tracker) format generation.
//!
//! This module handles all IT-specific generation logic including:
//! - Module creation and configuration
//! - Instrument and sample generation (IT separates these)
//! - Pattern conversion and automation
//! - IT-specific options (stereo, global volume, etc.)

use std::collections::HashMap;
use std::path::Path;

use speccade_spec::recipe::music::MusicTrackerSongV1Params;

use crate::generate::{GenerateError, GenerateResult, MusicLoopReport};
use crate::it::{effects as it_effects, ItModule, ItNote, ItValidator};

mod automation;
mod instrument;
mod pattern;

#[cfg(test)]
mod tests;

pub use automation::apply_automation_to_pattern;
pub use instrument::generate_it_instrument;
pub use pattern::convert_pattern_to_it;

/// Generate an IT module from params.
///
/// Creates a complete IT tracker module including instruments, samples,
/// patterns, and the order table from the arrangement.
///
/// # Arguments
/// * `params` - Music tracker song parameters
/// * `seed` - Base seed for deterministic synthesis
/// * `spec_dir` - Directory for resolving relative sample paths
///
/// # Returns
/// Generated IT module bytes with hash
pub fn generate_it(
    params: &MusicTrackerSongV1Params,
    seed: u32,
    spec_dir: &Path,
) -> Result<GenerateResult, GenerateError> {
    // Validate parameters
    validate_it_params(params)?;
    let bpm = u8::try_from(params.bpm).map_err(|_| {
        GenerateError::InvalidParameter(format!(
            "bpm {} exceeds IT header range (must be 32-255)",
            params.bpm
        ))
    })?;

    // Create module
    let mut module = ItModule::new("SpecCade Song", params.channels, params.speed, bpm);

    // Apply IT-specific options
    if let Some(ref it_opts) = params.it_options {
        automation::apply_it_options(&mut module, it_opts);
    }

    // Generate instruments and samples
    let mut instrument_loop_reports = Vec::with_capacity(params.instruments.len());
    for (idx, instr) in params.instruments.iter().enumerate() {
        let (it_instrument, it_sample, loop_report) =
            instrument::generate_it_instrument(instr, seed, idx as u32, spec_dir)?;
        module.add_instrument(it_instrument);
        module.add_sample(it_sample);
        instrument_loop_reports.push(loop_report);
    }

    // Build pattern index map
    let mut pattern_index_map: HashMap<String, u8> = HashMap::new();

    // Convert patterns (with automation).
    //
    // Determinism: `patterns` is a HashMap, so we must iterate in a stable order.
    let mut pattern_names: Vec<String> = params.patterns.keys().cloned().collect();
    pattern_names.sort();

    for (pattern_idx, name) in pattern_names.iter().enumerate() {
        let pattern = params
            .patterns
            .get(name)
            .ok_or_else(|| GenerateError::PatternNotFound(name.clone()))?;
        let mut it_pattern =
            pattern::convert_pattern_to_it(pattern, params.channels, &params.instruments)?;

        // Apply automation to this pattern
        automation::apply_automation_to_pattern(
            &mut it_pattern,
            name,
            &params.automation,
            params.channels,
        )?;

        let pattern_idx_u8 = u8::try_from(pattern_idx).map_err(|_| {
            GenerateError::InvalidParameter(format!(
                "patterns must be <= {}, got {}",
                u8::MAX,
                pattern_names.len()
            ))
        })?;
        module.add_pattern(it_pattern);
        pattern_index_map.insert(name.clone(), pattern_idx_u8);
    }

    // Build order table from arrangement
    let mut order_table = Vec::new();
    for entry in &params.arrangement {
        let idx = pattern_index_map
            .get(&entry.pattern)
            .ok_or_else(|| GenerateError::PatternNotFound(entry.pattern.clone()))?;
        for _ in 0..entry.repeat {
            order_table.push(*idx);
        }
    }

    // Set order table
    if order_table.is_empty() {
        order_table.push(0);
    }
    module.set_orders(&order_table);

    // IT loop behavior: insert a terminal position jump in the last order entry.
    //
    // This makes IT honor `loop` semantics instead of silently ignoring them.
    if params.r#loop {
        let restart = params.restart_position.unwrap_or(0);
        apply_it_loop_jump(&mut module, &order_table, restart, params.channels)?;
    }

    // Generate bytes
    let data = module.to_bytes()?;
    let report = ItValidator::validate(&data).map_err(|e| {
        GenerateError::FormatValidation(format!("generated IT parse failed: {}", e))
    })?;
    if !report.is_valid {
        let details = report
            .errors
            .iter()
            .take(3)
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        return Err(GenerateError::FormatValidation(format!(
            "generated IT failed validation with {} error(s): {}",
            report.errors.len(),
            details
        )));
    }
    let hash = blake3::hash(&data).to_hex().to_string();

    Ok(GenerateResult {
        data,
        hash,
        extension: "it",
        loop_report: Some(MusicLoopReport {
            extension: "it".to_string(),
            instruments: instrument_loop_reports,
        }),
    })
}

fn apply_it_loop_jump(
    module: &mut ItModule,
    order_table: &[u8],
    restart_position: u16,
    num_channels: u8,
) -> Result<(), GenerateError> {
    if restart_position as usize >= order_table.len() {
        return Err(GenerateError::InvalidParameter(format!(
            "restart_position {} is out of range for arrangement length {}",
            restart_position,
            order_table.len()
        )));
    }
    let restart_param = u8::try_from(restart_position).map_err(|_| {
        GenerateError::InvalidParameter(format!(
            "IT loop restart_position {} exceeds effect parameter range (0-255)",
            restart_position
        ))
    })?;

    let last_pattern_idx = *order_table.last().ok_or_else(|| {
        GenerateError::InvalidParameter("cannot apply IT loop: arrangement is empty".to_string())
    })? as usize;

    let last_pattern = module.patterns.get_mut(last_pattern_idx).ok_or_else(|| {
        GenerateError::InvalidParameter(format!(
            "cannot apply IT loop: order table references missing pattern index {}",
            last_pattern_idx
        ))
    })?;

    if last_pattern.num_rows == 0 {
        return Err(GenerateError::InvalidParameter(format!(
            "cannot apply IT loop: last pattern {} has zero rows",
            last_pattern_idx
        )));
    }
    let last_row = last_pattern.num_rows - 1;

    let mut selected_channel = None;
    for channel in 0..num_channels {
        let note = last_pattern
            .get_note(last_row, channel)
            .copied()
            .unwrap_or_else(ItNote::empty);
        if note.effect == 0
            || (note.effect == it_effects::POSITION_JUMP && note.effect_param == restart_param)
        {
            selected_channel = Some((channel, note));
            break;
        }
    }

    let (channel, mut note) = selected_channel.ok_or_else(|| {
        GenerateError::InvalidParameter(format!(
            "cannot apply IT loop: no channel available on last row {} (all {} channel(s) already use effects)",
            last_row, num_channels
        ))
    })?;

    note.effect = it_effects::POSITION_JUMP;
    note.effect_param = restart_param;
    last_pattern.set_note(last_row, channel, note);

    Ok(())
}

/// Validate IT-specific parameters.
fn validate_it_params(params: &MusicTrackerSongV1Params) -> Result<(), GenerateError> {
    if params.channels < 1 || params.channels > 64 {
        return Err(GenerateError::InvalidParameter(format!(
            "channels must be 1-64, got {}",
            params.channels
        )));
    }
    if params.bpm < 32 || params.bpm > 255 {
        return Err(GenerateError::InvalidParameter(format!(
            "bpm must be 32-255, got {}",
            params.bpm
        )));
    }
    if params.speed < 1 || params.speed > 31 {
        return Err(GenerateError::InvalidParameter(format!(
            "speed must be 1-31, got {}",
            params.speed
        )));
    }
    if params.patterns.len() > u8::MAX as usize {
        return Err(GenerateError::InvalidParameter(format!(
            "patterns must be <= {}, got {}",
            u8::MAX,
            params.patterns.len()
        )));
    }
    if let Some(restart_position) = params.restart_position {
        if restart_position > u8::MAX as u16 {
            return Err(GenerateError::InvalidParameter(format!(
                "IT restart_position must be 0-255, got {}",
                restart_position
            )));
        }
    }
    Ok(())
}
