//! XM (FastTracker II) format generation.
//!
//! This module handles all XM-specific generation logic including:
//! - Module creation and configuration
//! - Instrument synthesis and sample generation
//! - Pattern conversion and automation

use std::collections::HashMap;
use std::path::Path;

use speccade_spec::recipe::music::{
    AutomationEntry, InstrumentSynthesis, MusicTrackerSongV1Params, TrackerInstrument,
    TrackerPattern,
};

use crate::envelope::convert_envelope_to_xm;
use crate::generate::{
    extract_effect_code_param, get_instrument_synthesis, GenerateError, GenerateResult,
};
use crate::note::{calculate_pitch_correction, note_name_to_xm, DEFAULT_SAMPLE_RATE};
use crate::synthesis::{derive_instrument_seed, generate_loopable_sample, load_wav_sample};
use crate::xm::{effect_name_to_code as xm_effect_name_to_code, XmInstrument, XmModule, XmNote, XmPattern, XmSample};

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
    for (idx, instr) in params.instruments.iter().enumerate() {
        let xm_instrument = generate_xm_instrument(instr, seed, idx as u32, spec_dir)?;
        module.add_instrument(xm_instrument);
    }

    // Build pattern index map
    let mut pattern_index_map: HashMap<String, u8> = HashMap::new();

    // Convert patterns (with automation)
    for (pattern_idx, (name, pattern)) in params.patterns.iter().enumerate() {
        let mut xm_pattern = convert_pattern_to_xm(pattern, params.channels)?;

        // Apply automation to this pattern
        apply_automation_to_xm_pattern(&mut xm_pattern, name, &params.automation, params.channels)?;

        module.add_pattern(xm_pattern);
        pattern_index_map.insert(name.clone(), pattern_idx as u8);
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

    // Set restart position for looping
    if params.r#loop {
        module.set_restart_position(0);
    }

    // Generate bytes
    let data = module.to_bytes()?;
    let hash = blake3::hash(&data).to_hex().to_string();

    Ok(GenerateResult {
        data,
        hash,
        extension: "xm",
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
    Ok(())
}

/// Generate an XM instrument from spec.
fn generate_xm_instrument(
    instr: &TrackerInstrument,
    base_seed: u32,
    index: u32,
    spec_dir: &Path,
) -> Result<XmInstrument, GenerateError> {
    let instr_seed = derive_instrument_seed(base_seed, index);

    // Handle ref or inline synthesis
    let synthesis = get_instrument_synthesis(instr, spec_dir)?;

    // Generate sample data and pitch correction based on synthesis type
    let (sample_data, finetune, relative_note) = match &synthesis {
        InstrumentSynthesis::Sample { path, base_note } => {
            // Load WAV sample
            let sample_path = spec_dir.join(path);
            let data =
                load_wav_sample(&sample_path).map_err(|e| GenerateError::SampleLoadError(e))?;

            // Calculate pitch correction based on base_note
            let (ft, rn) = if let Some(note_name) = base_note {
                // Parse the base note to get MIDI number
                let xm_note = note_name_to_xm(note_name);
                if xm_note == 0 || xm_note == 97 {
                    return Err(GenerateError::SampleLoadError(format!(
                        "Invalid base_note '{}' for sample instrument '{}'",
                        note_name, instr.name
                    )));
                }
                // XM notes are 1-indexed, so xm_note=1 is C-0 (MIDI 0)
                // The relative_note offset should be set so that when the tracker plays
                // the base note, it plays at the sample's natural pitch
                let midi_note = (xm_note - 1) as i8;
                // The sample is recorded at this MIDI note, so we need to adjust
                // the playback so that playing this note uses the natural sample rate
                let (ft, rn) = calculate_pitch_correction(DEFAULT_SAMPLE_RATE);
                // Adjust relative note to account for the base note
                // If base_note is C-4 (MIDI 48, XM 49), we want playing C-4 to use natural pitch
                // relative_note shifts all playback, so we subtract the base note offset
                (ft, rn - midi_note)
            } else {
                // No base note specified, assume C-4 (MIDI 60)
                calculate_pitch_correction(DEFAULT_SAMPLE_RATE)
            };

            (data, ft, rn)
        }
        synth => {
            // Synthesize sample
            let (data, _loop_start, _loop_length) =
                generate_loopable_sample(synth, 60, DEFAULT_SAMPLE_RATE, 4, instr_seed);

            // Get pitch correction for sample rate
            let (ft, rn) = calculate_pitch_correction(DEFAULT_SAMPLE_RATE);
            (data, ft, rn)
        }
    };

    // Create sample
    let mut sample = XmSample::new(&instr.name, sample_data, true);
    sample.finetune = finetune;
    sample.relative_note = relative_note;

    // Set loop if needed (for sustained instruments)
    if !matches!(synthesis, InstrumentSynthesis::Sample { .. }) {
        sample.loop_type = 1; // Forward loop
        sample.loop_start = 0;
        sample.loop_length = sample.length_samples();
    }

    // Set default volume
    sample.volume = instr.default_volume.unwrap_or(64).min(64);

    // Create instrument
    let mut xm_instr = XmInstrument::new(&instr.name, sample);

    // Convert envelope to XM envelope
    xm_instr.volume_envelope = convert_envelope_to_xm(&instr.envelope);

    Ok(xm_instr)
}

/// Convert a pattern from spec to XM format.
fn convert_pattern_to_xm(
    pattern: &TrackerPattern,
    num_channels: u8,
) -> Result<XmPattern, GenerateError> {
    let mut xm_pattern = XmPattern::empty(pattern.rows, num_channels);

    for note in &pattern.data {
        if note.channel >= num_channels {
            continue;
        }

        let xm_note = if note.note == "---" || note.note == "..." {
            XmNote::empty()
        } else if note.note == "OFF" || note.note == "===" {
            XmNote::note_off()
        } else {
            let mut n = XmNote::from_name(
                &note.note,
                note.instrument + 1, // XM instruments are 1-indexed
                note.volume,
            );

            // Apply effect if present
            if let Some(ref effect) = note.effect {
                let (code, param) = extract_effect_code_param(effect, xm_effect_name_to_code)?;
                if let Some(code) = code {
                    n = n.with_effect(code, param);
                }
            }

            n
        };

        xm_pattern.set_note(note.row, note.channel, xm_note);
    }

    Ok(xm_pattern)
}

/// Apply automation entries to an XM pattern.
fn apply_automation_to_xm_pattern(
    pattern: &mut XmPattern,
    pattern_name: &str,
    automation: &[AutomationEntry],
    _num_channels: u8,
) -> Result<(), GenerateError> {
    for auto in automation {
        match auto {
            AutomationEntry::VolumeFade {
                pattern: target,
                channel,
                start_row,
                end_row,
                start_vol,
                end_vol,
            } => {
                if target == pattern_name {
                    apply_volume_fade_xm(
                        pattern,
                        *channel,
                        *start_row,
                        *end_row,
                        *start_vol,
                        *end_vol,
                    )?;
                }
            }
            AutomationEntry::TempoChange {
                pattern: target,
                row,
                bpm,
            } => {
                if target == pattern_name {
                    apply_tempo_change_xm(pattern, *row, *bpm)?;
                }
            }
        }
    }
    Ok(())
}

/// Apply volume fade automation to an XM pattern.
fn apply_volume_fade_xm(
    pattern: &mut XmPattern,
    channel: u8,
    start_row: u16,
    end_row: u16,
    start_vol: u8,
    end_vol: u8,
) -> Result<(), GenerateError> {
    if start_row >= end_row {
        return Err(GenerateError::AutomationError(
            "start_row must be less than end_row".to_string(),
        ));
    }

    let num_steps = (end_row - start_row) as f64;
    let vol_diff = end_vol as f64 - start_vol as f64;

    for row in start_row..=end_row {
        let progress = (row - start_row) as f64 / num_steps;
        let volume = (start_vol as f64 + vol_diff * progress).round() as u8;
        let volume = volume.min(64);

        // Get existing note or create volume command
        let mut note = pattern
            .get_note(row, channel)
            .copied()
            .unwrap_or_else(XmNote::empty);
        // XM volume column: 0x10-0x50 for volume 0-64
        note.volume = 0x10 + volume;
        pattern.set_note(row, channel, note);
    }

    Ok(())
}

/// Apply tempo change automation to an XM pattern.
fn apply_tempo_change_xm(pattern: &mut XmPattern, row: u16, bpm: u8) -> Result<(), GenerateError> {
    if bpm < 32 {
        return Err(GenerateError::AutomationError(format!(
            "BPM {} is too low (min 32)",
            bpm
        )));
    }

    // XM effect F is tempo/BPM set
    let effect_code = 0x0F;
    let mut note = pattern
        .get_note(row, 0)
        .copied()
        .unwrap_or_else(XmNote::empty);
    note = note.with_effect(effect_code, bpm);
    pattern.set_note(row, 0, note);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::audio_sfx::Envelope;
    use speccade_spec::recipe::music::{ArrangementEntry, PatternNote, TrackerFormat};

    fn create_test_params() -> MusicTrackerSongV1Params {
        let envelope = Envelope {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
        };

        let instrument = TrackerInstrument {
            name: "Test Lead".to_string(),
            r#ref: None,
            synthesis: Some(InstrumentSynthesis::Pulse { duty_cycle: 0.5 }),
            envelope: envelope.clone(),
            default_volume: Some(64),
        };

        let pattern = TrackerPattern {
            rows: 16,
            data: vec![
                PatternNote {
                    row: 0,
                    channel: 0,
                    note: "C4".to_string(),
                    instrument: 0,
                    volume: Some(64),
                    effect: None,
                },
                PatternNote {
                    row: 4,
                    channel: 0,
                    note: "E4".to_string(),
                    instrument: 0,
                    volume: Some(64),
                    effect: None,
                },
            ],
        };

        let mut patterns = HashMap::new();
        patterns.insert("intro".to_string(), pattern);

        MusicTrackerSongV1Params {
            format: TrackerFormat::Xm,
            bpm: 120,
            speed: 6,
            channels: 4,
            r#loop: true,
            instruments: vec![instrument],
            patterns,
            arrangement: vec![ArrangementEntry {
                pattern: "intro".to_string(),
                repeat: 1,
            }],
            automation: vec![],
            it_options: None,
        }
    }

    #[test]
    fn test_generate_xm() {
        let params = create_test_params();
        let spec_dir = Path::new(".");
        let result = generate_xm(&params, 42, spec_dir).unwrap();

        assert_eq!(result.extension, "xm");
        assert!(!result.data.is_empty());
        assert_eq!(result.hash.len(), 64);
    }

    #[test]
    fn test_xm_param_validation() {
        let mut params = create_test_params();
        params.channels = 100; // Invalid for XM

        assert!(validate_xm_params(&params).is_err());
    }

    #[test]
    fn test_volume_fade_xm() {
        let mut pattern = XmPattern::empty(16, 4);

        apply_volume_fade_xm(&mut pattern, 0, 0, 8, 64, 0).unwrap();

        // Check interpolation
        let note_start = pattern.get_note(0, 0).unwrap();
        assert_eq!(note_start.volume, 0x10 + 64);

        let note_end = pattern.get_note(8, 0).unwrap();
        assert_eq!(note_end.volume, 0x10 + 0);
    }
}
