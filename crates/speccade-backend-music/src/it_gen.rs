//! IT (Impulse Tracker) format generation.
//!
//! This module handles all IT-specific generation logic including:
//! - Module creation and configuration
//! - Instrument and sample generation (IT separates these)
//! - Pattern conversion and automation
//! - IT-specific options (stereo, global volume, etc.)

use std::collections::HashMap;
use std::path::Path;

use speccade_spec::recipe::music::{
    AutomationEntry, InstrumentSynthesis, ItOptions, MusicTrackerSongV1Params, TrackerInstrument,
    TrackerPattern,
};

use crate::envelope::convert_envelope_to_it;
use crate::generate::{
    extract_effect_code_param, get_instrument_synthesis, GenerateError, GenerateResult,
};
use crate::it::{
    effect_name_to_code as it_effect_name_to_code, ItInstrument, ItModule, ItNote, ItPattern,
    ItSample,
};
use crate::note::{note_name_to_it, DEFAULT_SAMPLE_RATE};
use crate::synthesis::{derive_instrument_seed, generate_loopable_sample, load_wav_sample};

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

    // Create module
    let mut module = ItModule::new(
        "SpecCade Song",
        params.channels,
        params.speed,
        params.bpm as u8,
    );

    // Apply IT-specific options
    if let Some(ref it_opts) = params.it_options {
        apply_it_options(&mut module, it_opts);
    }

    // Generate instruments and samples
    for (idx, instr) in params.instruments.iter().enumerate() {
        let (it_instrument, it_sample) =
            generate_it_instrument(instr, seed, idx as u32, spec_dir)?;
        module.add_instrument(it_instrument);
        module.add_sample(it_sample);
    }

    // Build pattern index map
    let mut pattern_index_map: HashMap<String, u8> = HashMap::new();

    // Convert patterns (with automation)
    for (pattern_idx, (name, pattern)) in params.patterns.iter().enumerate() {
        let mut it_pattern = convert_pattern_to_it(pattern, params.channels)?;

        // Apply automation to this pattern
        apply_automation_to_pattern(&mut it_pattern, name, &params.automation, params.channels)?;

        module.add_pattern(it_pattern);
        pattern_index_map.insert(name.clone(), pattern_idx as u8);
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
    let hash = blake3::hash(&data).to_hex().to_string();

    Ok(GenerateResult {
        data,
        hash,
        extension: "it",
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

/// Generate an IT instrument and sample from spec.
///
/// IT format separates instruments from samples, so this returns both.
/// The instrument maps all notes to the generated sample.
fn generate_it_instrument(
    instr: &TrackerInstrument,
    base_seed: u32,
    index: u32,
    spec_dir: &Path,
) -> Result<(ItInstrument, ItSample), GenerateError> {
    let instr_seed = derive_instrument_seed(base_seed, index);

    // Handle ref or inline synthesis
    let synthesis = get_instrument_synthesis(instr, spec_dir)?;

    // Generate sample data and determine sample rate based on synthesis type
    let (sample_data, sample_rate) = match &synthesis {
        InstrumentSynthesis::Sample { path, base_note } => {
            // Load WAV sample
            let sample_path = spec_dir.join(path);
            let data =
                load_wav_sample(&sample_path).map_err(|e| GenerateError::SampleLoadError(e))?;

            // Calculate C-5 speed (IT's pitch reference) based on base_note
            let c5_speed = if let Some(note_name) = base_note {
                // Parse the base note to get IT note number
                let it_note = note_name_to_it(note_name);
                if it_note > 119 {
                    return Err(GenerateError::SampleLoadError(format!(
                        "Invalid base_note '{}' for sample instrument '{}'",
                        note_name, instr.name
                    )));
                }

                // In IT, C-5 (note 60) should play at the C-5 speed (sample rate)
                // If the sample is recorded at a different note, we need to adjust the C-5 speed
                // C-5 speed = sample_rate * 2^((60 - base_note) / 12)
                let semitone_diff = 60 - it_note as i32;
                let speed_multiplier = 2.0_f64.powf(semitone_diff as f64 / 12.0);
                (DEFAULT_SAMPLE_RATE as f64 * speed_multiplier) as u32
            } else {
                // No base note specified, assume the sample is at C-5
                DEFAULT_SAMPLE_RATE
            };

            (data, c5_speed)
        }
        synth => {
            // Synthesize sample
            let (data, _loop_start, _loop_length) =
                generate_loopable_sample(synth, 60, DEFAULT_SAMPLE_RATE, 4, instr_seed);
            (data, DEFAULT_SAMPLE_RATE)
        }
    };

    // Create sample with the calculated sample rate (C-5 speed)
    let sample = ItSample::new(&instr.name, sample_data, sample_rate);
    let sample_length = sample.length_samples();

    // Set loop if needed
    let mut sample = if !matches!(synthesis, InstrumentSynthesis::Sample { .. }) {
        sample.with_loop(0, sample_length, false)
    } else {
        sample
    };

    // Set default volume
    sample.default_volume = instr.default_volume.unwrap_or(64).min(64);

    // Create instrument
    let mut it_instr = ItInstrument::new(&instr.name);

    // Map all notes to this sample (index + 1 since IT is 1-indexed)
    it_instr.map_all_to_sample((index + 1) as u8);

    // Convert envelope
    it_instr.volume_envelope = convert_envelope_to_it(&instr.envelope);

    Ok((it_instr, sample))
}

/// Convert a pattern from spec to IT format.
fn convert_pattern_to_it(
    pattern: &TrackerPattern,
    num_channels: u8,
) -> Result<ItPattern, GenerateError> {
    let mut it_pattern = ItPattern::empty(pattern.rows, num_channels);

    for note in &pattern.data {
        if note.channel >= num_channels {
            continue;
        }

        let it_note = if note.note == "---" || note.note == "..." {
            ItNote::empty()
        } else if note.note == "OFF" || note.note == "^^^" {
            ItNote::note_off()
        } else if note.note == "===" {
            ItNote::note_cut()
        } else {
            let mut n = ItNote::from_name(
                &note.note,
                note.instrument + 1, // IT instruments are 1-indexed
                note.volume.unwrap_or(64),
            );

            // Apply effect if present
            if let Some(ref effect) = note.effect {
                let (code, param) = extract_effect_code_param(effect, it_effect_name_to_code)?;
                if let Some(code) = code {
                    n = n.with_effect(code, param);
                }
            }

            n
        };

        it_pattern.set_note(note.row, note.channel, it_note);
    }

    Ok(it_pattern)
}

/// Apply IT-specific options to the module header.
fn apply_it_options(module: &mut ItModule, options: &ItOptions) {
    // Set stereo flag
    if options.stereo {
        module.header.flags |= crate::it::flags::STEREO;
    } else {
        module.header.flags &= !crate::it::flags::STEREO;
    }

    // Set global volume
    module.header.global_volume = options.global_volume.min(128);

    // Set mix volume
    module.header.mix_volume = options.mix_volume.min(128);
}

/// Apply automation entries to an IT pattern.
fn apply_automation_to_pattern(
    pattern: &mut ItPattern,
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
                    apply_volume_fade_it(
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
                    apply_tempo_change_it(pattern, *row, *bpm)?;
                }
            }
        }
    }
    Ok(())
}

/// Apply volume fade automation to an IT pattern.
fn apply_volume_fade_it(
    pattern: &mut ItPattern,
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
            .unwrap_or_else(ItNote::empty);
        note.volume = volume;
        pattern.set_note(row, channel, note);
    }

    Ok(())
}

/// Apply tempo change automation to an IT pattern.
fn apply_tempo_change_it(pattern: &mut ItPattern, row: u16, bpm: u8) -> Result<(), GenerateError> {
    if bpm < 32 {
        return Err(GenerateError::AutomationError(format!(
            "BPM {} is too low (min 32)",
            bpm
        )));
    }

    // IT effect T is tempo set (0x14)
    let effect_code = 0x14; // 'T' command
    let mut note = pattern
        .get_note(row, 0)
        .copied()
        .unwrap_or_else(ItNote::empty);
    note.effect = effect_code;
    note.effect_param = bpm;
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
            format: TrackerFormat::It,
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
    fn test_generate_it() {
        let params = create_test_params();
        let spec_dir = Path::new(".");
        let result = generate_it(&params, 42, spec_dir).unwrap();

        assert_eq!(result.extension, "it");
        assert!(!result.data.is_empty());
        assert_eq!(result.hash.len(), 64);
    }

    #[test]
    fn test_it_param_validation() {
        let mut params = create_test_params();
        params.channels = 100; // Invalid for IT (max 64)

        assert!(validate_it_params(&params).is_err());
    }

    #[test]
    fn test_volume_fade_it() {
        let mut pattern = ItPattern::empty(16, 4);

        apply_volume_fade_it(&mut pattern, 0, 0, 8, 64, 0).unwrap();

        // Check interpolation
        let note_start = pattern.get_note(0, 0).unwrap();
        assert_eq!(note_start.volume, 64);

        let note_end = pattern.get_note(8, 0).unwrap();
        assert_eq!(note_end.volume, 0);
    }

    #[test]
    fn test_tempo_change_it() {
        let mut pattern = ItPattern::empty(16, 4);

        apply_tempo_change_it(&mut pattern, 4, 140).unwrap();

        let note = pattern.get_note(4, 0).unwrap();
        assert_eq!(note.effect, 0x14);
        assert_eq!(note.effect_param, 140);
    }
}
