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
use crate::it::{ItModule, ItValidator};

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

    // Generate bytes
    let data = module.to_bytes()?;
    let report = ItValidator::validate(&data)
        .map_err(|e| GenerateError::FormatValidation(format!("generated IT parse failed: {}", e)))?;
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
    Ok(())
}
