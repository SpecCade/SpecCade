//! Main entry point for music generation from SpecCade specs.
//!
//! This module converts SpecCade `MusicTrackerSongV1Params` specs into XM or IT
//! tracker module files with full determinism guarantees.

use std::collections::HashMap;
use std::path::Path;

use speccade_spec::recipe::audio_sfx::Envelope;
use speccade_spec::recipe::music::{
    AutomationEntry, InstrumentSynthesis, ItOptions, MusicTrackerSongV1Params, TrackerFormat,
    TrackerInstrument, TrackerPattern,
};
use thiserror::Error;

use crate::it::{
    effect_name_to_code as it_effect_name_to_code, env_flags, ItEnvelope, ItEnvelopePoint,
    ItInstrument, ItModule, ItNote, ItPattern, ItSample,
};
use crate::note::{calculate_pitch_correction, note_name_to_xm, note_name_to_it, DEFAULT_SAMPLE_RATE};
use crate::synthesis::{derive_instrument_seed, generate_loopable_sample, load_wav_sample};
use crate::xm::{
    effect_name_to_code as xm_effect_name_to_code, XmEnvelope, XmEnvelopePoint, XmInstrument,
    XmModule, XmNote, XmPattern, XmSample,
};

/// Error type for music generation.
#[derive(Debug, Error)]
pub enum GenerateError {
    /// Invalid parameter value.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Pattern not found in spec.
    #[error("Pattern not found: {0}")]
    PatternNotFound(String),

    /// IO error during writing.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Unsupported synthesis type.
    #[error("Unsupported synthesis type: {0}")]
    UnsupportedSynthesis(String),

    /// Sample loading error.
    #[error("Sample loading error: {0}")]
    SampleLoadError(String),

    /// Instrument definition error.
    #[error("Instrument error: {0}")]
    InstrumentError(String),

    /// Automation error.
    #[error("Automation error: {0}")]
    AutomationError(String),
}

/// Result of music generation.
pub struct GenerateResult {
    /// Generated tracker module bytes.
    pub data: Vec<u8>,
    /// BLAKE3 hash of the generated data.
    pub hash: String,
    /// File extension ("xm" or "it").
    pub extension: &'static str,
}

/// Generate a tracker module from a SpecCade music spec.
///
/// # Arguments
/// * `params` - Music tracker song parameters from the spec
/// * `seed` - Base seed for deterministic generation
/// * `spec_dir` - Directory containing the spec file (for resolving relative sample paths)
///
/// # Returns
/// `GenerateResult` containing the module bytes, hash, and extension
///
/// # Example
/// ```ignore
/// use speccade_backend_music::generate::generate_music;
/// use speccade_spec::recipe::music::MusicTrackerSongV1Params;
/// use std::path::Path;
///
/// let params = MusicTrackerSongV1Params { ... };
/// let spec_dir = Path::new("path/to/spec/dir");
/// let result = generate_music(&params, 42, spec_dir)?;
/// std::fs::write(format!("song.{}", result.extension), &result.data)?;
/// ```
pub fn generate_music(
    params: &MusicTrackerSongV1Params,
    seed: u32,
    spec_dir: &Path,
) -> Result<GenerateResult, GenerateError> {
    match params.format {
        TrackerFormat::Xm => generate_xm(params, seed, spec_dir),
        TrackerFormat::It => generate_it(params, seed, spec_dir),
    }
}

/// Generate an XM module from params.
fn generate_xm(params: &MusicTrackerSongV1Params, seed: u32, spec_dir: &Path) -> Result<GenerateResult, GenerateError> {
    // Validate parameters
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

/// Generate an IT module from params.
fn generate_it(params: &MusicTrackerSongV1Params, seed: u32, spec_dir: &Path) -> Result<GenerateResult, GenerateError> {
    // Validate parameters
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
        let (it_instrument, it_sample) = generate_it_instrument(instr, seed, idx as u32, spec_dir)?;
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
            let data = load_wav_sample(&sample_path)
                .map_err(|e| GenerateError::SampleLoadError(e))?;

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

/// Get the synthesis configuration for an instrument, handling ref if present.
fn get_instrument_synthesis(
    instr: &TrackerInstrument,
    spec_dir: &Path,
) -> Result<InstrumentSynthesis, GenerateError> {
    if let Some(ref ref_path) = instr.r#ref {
        // Load external spec file
        load_instrument_from_ref(ref_path, spec_dir)
    } else if let Some(ref synthesis) = instr.synthesis {
        // Use inline synthesis
        Ok(synthesis.clone())
    } else {
        // Default to a basic synthesis if neither is provided
        Err(GenerateError::InstrumentError(format!(
            "Instrument '{}' must have either 'ref' or 'synthesis' defined",
            instr.name
        )))
    }
}

/// Load instrument synthesis from an external spec file.
fn load_instrument_from_ref(
    ref_path: &str,
    spec_dir: &Path,
) -> Result<InstrumentSynthesis, GenerateError> {
    // TODO: Implement full spec file loading
    // For now, return an error as this requires the full spec parser
    Err(GenerateError::InstrumentError(format!(
        "External instrument reference loading not yet implemented: {}",
        ref_path
    )))
}

/// Generate an IT instrument and sample from spec.
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
            let data = load_wav_sample(&sample_path)
                .map_err(|e| GenerateError::SampleLoadError(e))?;

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

/// Convert ADSR envelope to XM envelope.
fn convert_envelope_to_xm(envelope: &Envelope) -> XmEnvelope {
    // Convert seconds to ticks (assuming ~50 ticks per second at default tempo)
    let ticks_per_sec = 50.0;

    let attack_ticks = (envelope.attack * ticks_per_sec) as u16;
    let decay_ticks = (envelope.decay * ticks_per_sec) as u16;
    let release_ticks = (envelope.release * ticks_per_sec) as u16;
    let sustain_value = (envelope.sustain * 64.0) as u16;

    let mut points = Vec::new();

    // Attack: 0 -> 64
    points.push(XmEnvelopePoint {
        frame: 0,
        value: 0,
    });
    points.push(XmEnvelopePoint {
        frame: attack_ticks.max(1),
        value: 64,
    });

    // Decay: 64 -> sustain
    let decay_end = attack_ticks + decay_ticks;
    points.push(XmEnvelopePoint {
        frame: decay_end.max(attack_ticks + 1),
        value: sustain_value,
    });

    // Sustain hold point
    let sustain_end = decay_end + 100;
    points.push(XmEnvelopePoint {
        frame: sustain_end,
        value: sustain_value,
    });

    // Release: sustain -> 0
    points.push(XmEnvelopePoint {
        frame: sustain_end + release_ticks,
        value: 0,
    });

    XmEnvelope {
        points,
        sustain_point: 2,
        loop_start: 0,
        loop_end: 0,
        enabled: true,
        sustain_enabled: true,
        loop_enabled: false,
    }
}

/// Convert ADSR envelope to IT envelope.
fn convert_envelope_to_it(envelope: &Envelope) -> ItEnvelope {
    let ticks_per_sec = 50.0;

    let attack_ticks = (envelope.attack * ticks_per_sec) as u16;
    let decay_ticks = (envelope.decay * ticks_per_sec) as u16;
    let release_ticks = (envelope.release * ticks_per_sec) as u16;
    let sustain_value = (envelope.sustain * 64.0) as i8;

    let mut points = Vec::new();

    // Attack
    points.push(ItEnvelopePoint { tick: 0, value: 0 });
    points.push(ItEnvelopePoint {
        tick: attack_ticks.max(1),
        value: 64,
    });

    // Decay
    let decay_end = attack_ticks + decay_ticks;
    points.push(ItEnvelopePoint {
        tick: decay_end.max(attack_ticks + 1),
        value: sustain_value,
    });

    // Sustain hold
    let sustain_end = decay_end + 100;
    points.push(ItEnvelopePoint {
        tick: sustain_end,
        value: sustain_value,
    });

    // Release
    points.push(ItEnvelopePoint {
        tick: sustain_end + release_ticks,
        value: 0,
    });

    ItEnvelope {
        flags: env_flags::ENABLED | env_flags::SUSTAIN_LOOP,
        points,
        loop_begin: 0,
        loop_end: 0,
        sustain_begin: 2,
        sustain_end: 3,
    }
}

/// Convert a pattern from spec to XM format.
fn convert_pattern_to_xm(pattern: &TrackerPattern, num_channels: u8) -> Result<XmPattern, GenerateError> {
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

/// Convert a pattern from spec to IT format.
fn convert_pattern_to_it(pattern: &TrackerPattern, num_channels: u8) -> Result<ItPattern, GenerateError> {
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

/// Extract effect code and parameter from PatternEffect, handling effect_xy nibbles.
fn extract_effect_code_param<F>(
    effect: &speccade_spec::recipe::music::PatternEffect,
    name_to_code: F,
) -> Result<(Option<u8>, u8), GenerateError>
where
    F: Fn(&str) -> Option<u8>,
{
    // Calculate parameter from effect_xy nibbles if present
    let param = if let Some((x, y)) = effect.effect_xy {
        // Convert nibbles to byte: param = (x << 4) | y
        (x << 4) | y
    } else if let Some(p) = effect.param {
        p
    } else {
        0
    };

    // Get effect code from type name
    let code = if let Some(ref type_name) = effect.r#type {
        name_to_code(type_name)
    } else {
        None
    };

    Ok((code, param))
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
        let mut note = pattern.get_note(row, channel).copied().unwrap_or_else(ItNote::empty);
        note.volume = volume;
        pattern.set_note(row, channel, note);
    }

    Ok(())
}

/// Apply tempo change automation to an IT pattern.
fn apply_tempo_change_it(
    pattern: &mut ItPattern,
    row: u16,
    bpm: u8,
) -> Result<(), GenerateError> {
    if bpm < 32 {
        return Err(GenerateError::AutomationError(format!(
            "BPM {} is too low (min 32)",
            bpm
        )));
    }

    // IT effect T is tempo set (0x14)
    let effect_code = 0x14; // 'T' command
    let mut note = pattern.get_note(row, 0).copied().unwrap_or_else(ItNote::empty);
    note.effect = effect_code;
    note.effect_param = bpm;
    pattern.set_note(row, 0, note);

    Ok(())
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
        let mut note = pattern.get_note(row, channel).copied().unwrap_or_else(XmNote::empty);
        // XM volume column: 0x10-0x50 for volume 0-64
        note.volume = 0x10 + volume;
        pattern.set_note(row, channel, note);
    }

    Ok(())
}

/// Apply tempo change automation to an XM pattern.
fn apply_tempo_change_xm(
    pattern: &mut XmPattern,
    row: u16,
    bpm: u8,
) -> Result<(), GenerateError> {
    if bpm < 32 {
        return Err(GenerateError::AutomationError(format!(
            "BPM {} is too low (min 32)",
            bpm
        )));
    }

    // XM effect F is tempo/BPM set
    let effect_code = 0x0F;
    let mut note = pattern.get_note(row, 0).copied().unwrap_or_else(XmNote::empty);
    note = note.with_effect(effect_code, bpm);
    pattern.set_note(row, 0, note);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::music::{ArrangementEntry, PatternNote};
    use std::collections::HashMap;

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
                PatternNote {
                    row: 8,
                    channel: 0,
                    note: "G4".to_string(),
                    instrument: 0,
                    volume: Some(64),
                    effect: None,
                },
                PatternNote {
                    row: 12,
                    channel: 0,
                    note: "OFF".to_string(),
                    instrument: 0,
                    volume: None,
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
        let result = generate_music(&params, 42, spec_dir).unwrap();

        assert_eq!(result.extension, "xm");
        assert!(!result.data.is_empty());
        assert_eq!(result.hash.len(), 64);
    }

    #[test]
    fn test_generate_it() {
        let mut params = create_test_params();
        params.format = TrackerFormat::It;

        let spec_dir = Path::new(".");
        let result = generate_music(&params, 42, spec_dir).unwrap();

        assert_eq!(result.extension, "it");
        assert!(!result.data.is_empty());
        assert_eq!(result.hash.len(), 64);
    }

    #[test]
    fn test_determinism() {
        let params = create_test_params();
        let spec_dir = Path::new(".");

        let result1 = generate_music(&params, 42, spec_dir).unwrap();
        let result2 = generate_music(&params, 42, spec_dir).unwrap();

        assert_eq!(result1.hash, result2.hash);
        assert_eq!(result1.data, result2.data);
    }

    #[test]
    fn test_different_seeds_different_output() {
        // Use noise synthesis which uses the seed
        let mut params = create_test_params();
        params.instruments[0].synthesis = Some(InstrumentSynthesis::Noise { periodic: false });

        let spec_dir = Path::new(".");
        let result1 = generate_music(&params, 42, spec_dir).unwrap();
        let result2 = generate_music(&params, 43, spec_dir).unwrap();

        // Different seeds should produce different hashes (due to noise synthesis)
        assert_ne!(result1.hash, result2.hash);
    }

    #[test]
    fn test_invalid_channels() {
        let mut params = create_test_params();
        params.channels = 100; // Invalid for XM (max 32)

        let spec_dir = Path::new(".");
        let result = generate_music(&params, 42, spec_dir);
        assert!(result.is_err());
    }
}
