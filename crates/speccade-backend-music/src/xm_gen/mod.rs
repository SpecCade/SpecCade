//! XM (FastTracker II) format generation.
//!
//! This module handles all XM-specific generation logic including:
//! - Module creation and configuration
//! - Instrument synthesis and sample generation
//! - Pattern conversion and automation

mod automation;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::Path;

use speccade_spec::recipe::music::{
    MusicTrackerSongV1Params, TrackerFormat, TrackerInstrument, TrackerPattern,
};

use crate::envelope::convert_envelope_to_xm;
use crate::generate::{
    bake_instrument_sample, resolve_pattern_note_name, GenerateError, GenerateResult,
    MusicInstrumentLoopReport, MusicLoopReport,
};
use crate::note::calculate_xm_pitch_correction;
use crate::xm::{
    effect_name_to_code as xm_effect_name_to_code, XmInstrument, XmModule, XmNote, XmPattern,
    XmSample,
};

pub use automation::{apply_automation_to_xm_pattern, apply_tempo_change_xm, apply_volume_fade_xm};

/// Generate an XM module from params.
///
/// Creates a complete XM tracker module including instruments, patterns,
/// and the order table from the arrangement.
///
/// # Arguments
/// * `params` - Music tracker song parameters
/// * `seed` - Base seed for deterministic synthesis
/// * `spec_dir` - Directory for resolving relative sample paths
///
/// # Returns
/// Generated XM module bytes with hash
pub fn generate_xm(
    params: &MusicTrackerSongV1Params,
    seed: u32,
    spec_dir: &Path,
) -> Result<GenerateResult, GenerateError> {
    // Validate parameters
    validate_xm_params(params)?;

    // Create module
    let mut module = XmModule::new("SpecCade Song", params.channels, params.speed, params.bpm);

    // Generate instruments
    let mut instrument_loop_reports = Vec::with_capacity(params.instruments.len());
    for (idx, instr) in params.instruments.iter().enumerate() {
        let (xm_instrument, loop_report) =
            generate_xm_instrument(instr, seed, idx as u32, spec_dir)?;
        module.add_instrument(xm_instrument);
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
        let mut xm_pattern = convert_pattern_to_xm(pattern, params.channels, &params.instruments)?;

        // Apply automation to this pattern
        apply_automation_to_xm_pattern(&mut xm_pattern, name, &params.automation, params.channels)?;

        let pattern_idx_u8 = u8::try_from(pattern_idx).map_err(|_| {
            GenerateError::InvalidParameter(format!(
                "patterns must be <= {}, got {}",
                u8::MAX,
                pattern_names.len()
            ))
        })?;
        module.add_pattern(xm_pattern);
        pattern_index_map.insert(name.clone(), pattern_idx_u8);
    }

    // Build order table from arrangement
    let mut order_table = Vec::new();
    for entry in &params.arrangement {
        let pattern_idx = pattern_index_map
            .get(&entry.pattern)
            .ok_or_else(|| GenerateError::PatternNotFound(entry.pattern.clone()))?;
        for _ in 0..entry.repeat {
            order_table.push(*pattern_idx);
        }
    }

    // Set order table
    if order_table.is_empty() {
        order_table.push(0);
    }
    module.set_order_table(&order_table);

    // Set restart position for looping.
    //
    // Note: XM uses an order-table index for restart position.
    if params.r#loop {
        let restart = params.restart_position.unwrap_or(0);
        if restart as usize >= order_table.len() {
            return Err(GenerateError::InvalidParameter(format!(
                "restart_position {} is out of range for arrangement length {}",
                restart,
                order_table.len()
            )));
        }
        module.set_restart_position(restart);
    }

    // Generate bytes
    let data = module.to_bytes()?;
    let hash = blake3::hash(&data).to_hex().to_string();

    Ok(GenerateResult {
        data,
        hash,
        extension: "xm",
        loop_report: Some(MusicLoopReport {
            extension: "xm".to_string(),
            instruments: instrument_loop_reports,
        }),
    })
}

/// Validate XM-specific parameters.
fn validate_xm_params(params: &MusicTrackerSongV1Params) -> Result<(), GenerateError> {
    if params.channels < 1 || params.channels > 32 {
        return Err(GenerateError::InvalidParameter(format!(
            "channels must be 1-32, got {}",
            params.channels
        )));
    }
    if params.bpm < 30 || params.bpm > 300 {
        return Err(GenerateError::InvalidParameter(format!(
            "bpm must be 30-300, got {}",
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

/// Generate an XM instrument from spec.
pub(crate) fn generate_xm_instrument(
    instr: &TrackerInstrument,
    base_seed: u32,
    index: u32,
    spec_dir: &Path,
) -> Result<(XmInstrument, MusicInstrumentLoopReport), GenerateError> {
    let (baked, loop_report) =
        bake_instrument_sample(instr, base_seed, index, spec_dir, TrackerFormat::Xm)?;

    let (finetune, relative_note) =
        calculate_xm_pitch_correction(baked.sample_rate, baked.base_midi);

    // Create sample
    let mut sample = XmSample::new(&instr.name, baked.pcm16_mono, true);
    sample.finetune = finetune;
    sample.relative_note = relative_note;

    if let Some(loop_region) = baked.loop_region {
        sample.loop_type = match loop_region.mode {
            crate::generate::LoopMode::Forward => 1,
            crate::generate::LoopMode::PingPong => 2,
        };
        sample.loop_start = loop_region.start;
        sample.loop_length = loop_region.end.saturating_sub(loop_region.start);
    }

    // Set default volume
    sample.volume = instr.default_volume.unwrap_or(64).min(64);

    // Create instrument
    let mut xm_instr = XmInstrument::new(&instr.name, sample);

    // Convert envelope to XM envelope
    xm_instr.volume_envelope = convert_envelope_to_xm(&instr.envelope);

    Ok((xm_instr, loop_report))
}

/// Convert a pattern from spec to XM format.
pub(crate) fn convert_pattern_to_xm(
    pattern: &TrackerPattern,
    num_channels: u8,
    instruments: &[TrackerInstrument],
) -> Result<XmPattern, GenerateError> {
    let mut xm_pattern = XmPattern::empty(pattern.rows, num_channels);

    // Iterate over notes organized by channel
    for (channel, note) in pattern.flat_notes() {
        if channel >= num_channels {
            continue;
        }

        let note_name = resolve_pattern_note_name(note, instruments, "C4")?;
        let note_name = note_name.as_ref();

        let xm_note = if note_name == "OFF" || note_name == "===" {
            XmNote::note_off()
        } else {
            let mut n = XmNote::from_name(
                note_name,
                note.inst + 1, // XM instruments are 1-indexed
                note.vol,
            );

            // Apply effect if present - supports both numeric effect and effect_name
            if let Some(effect_code) = note.effect {
                let param = note.param.unwrap_or(0);
                n = n.with_effect(effect_code, param);
            } else if let Some(ref effect_name) = note.effect_name {
                if let Some(code) = xm_effect_name_to_code(effect_name) {
                    let param = if let Some([x, y]) = note.effect_xy {
                        (x << 4) | (y & 0x0F)
                    } else {
                        note.param.unwrap_or(0)
                    };
                    n = n.with_effect(code, param);
                }
            }

            n
        };

        xm_pattern.set_note(note.row, channel, xm_note);
    }

    Ok(xm_pattern)
}
