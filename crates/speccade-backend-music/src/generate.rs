//! Main entry point for music generation from SpecCade specs.
//!
//! This module converts SpecCade `MusicTrackerSongV1Params` specs into XM or IT
//! tracker module files with full determinism guarantees.

use std::collections::HashMap;

use speccade_spec::recipe::audio_sfx::Envelope;
use speccade_spec::recipe::music::{
    InstrumentSynthesis, MusicTrackerSongV1Params, TrackerFormat, TrackerInstrument, TrackerPattern,
};
use thiserror::Error;

use crate::it::{
    effect_name_to_code as it_effect_name_to_code, env_flags, ItEnvelope, ItEnvelopePoint,
    ItInstrument, ItModule, ItNote, ItPattern, ItSample,
};
use crate::note::{calculate_pitch_correction, DEFAULT_SAMPLE_RATE};
use crate::synthesis::{derive_instrument_seed, generate_loopable_sample};
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
///
/// # Returns
/// `GenerateResult` containing the module bytes, hash, and extension
///
/// # Example
/// ```ignore
/// use speccade_backend_music::generate::generate_music;
/// use speccade_spec::recipe::music::MusicTrackerSongV1Params;
///
/// let params = MusicTrackerSongV1Params { ... };
/// let result = generate_music(&params, 42)?;
/// std::fs::write(format!("song.{}", result.extension), &result.data)?;
/// ```
pub fn generate_music(params: &MusicTrackerSongV1Params, seed: u32) -> Result<GenerateResult, GenerateError> {
    match params.format {
        TrackerFormat::Xm => generate_xm(params, seed),
        TrackerFormat::It => generate_it(params, seed),
    }
}

/// Generate an XM module from params.
fn generate_xm(params: &MusicTrackerSongV1Params, seed: u32) -> Result<GenerateResult, GenerateError> {
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
        let xm_instrument = generate_xm_instrument(instr, seed, idx as u32)?;
        module.add_instrument(xm_instrument);
    }

    // Build pattern index map
    let mut pattern_index_map: HashMap<String, u8> = HashMap::new();

    // Convert patterns
    for (pattern_idx, (name, pattern)) in params.patterns.iter().enumerate() {
        let xm_pattern = convert_pattern_to_xm(pattern, params.channels)?;
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
fn generate_it(params: &MusicTrackerSongV1Params, seed: u32) -> Result<GenerateResult, GenerateError> {
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

    // Generate instruments and samples
    for (idx, instr) in params.instruments.iter().enumerate() {
        let (it_instrument, it_sample) = generate_it_instrument(instr, seed, idx as u32)?;
        module.add_instrument(it_instrument);
        module.add_sample(it_sample);
    }

    // Build pattern index map
    let mut pattern_index_map: HashMap<String, u8> = HashMap::new();

    // Convert patterns
    for (pattern_idx, (name, pattern)) in params.patterns.iter().enumerate() {
        let it_pattern = convert_pattern_to_it(pattern, params.channels)?;
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
) -> Result<XmInstrument, GenerateError> {
    let instr_seed = derive_instrument_seed(base_seed, index);

    // Generate sample data based on synthesis type
    let sample_data = match &instr.synthesis {
        InstrumentSynthesis::Sample { .. } => {
            // Sample-based: return empty data (would load from file in full impl)
            vec![]
        }
        synth => {
            // Synthesize sample
            let (data, _loop_start, _loop_length) =
                generate_loopable_sample(synth, 60, DEFAULT_SAMPLE_RATE, 4, instr_seed);
            data
        }
    };

    // Get pitch correction for sample rate
    let (finetune, relative_note) = calculate_pitch_correction(DEFAULT_SAMPLE_RATE);

    // Create sample
    let mut sample = XmSample::new(&instr.name, sample_data, true);
    sample.finetune = finetune;
    sample.relative_note = relative_note;

    // Set loop if needed (for sustained instruments)
    if !matches!(instr.synthesis, InstrumentSynthesis::Sample { .. }) {
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

/// Generate an IT instrument and sample from spec.
fn generate_it_instrument(
    instr: &TrackerInstrument,
    base_seed: u32,
    index: u32,
) -> Result<(ItInstrument, ItSample), GenerateError> {
    let instr_seed = derive_instrument_seed(base_seed, index);

    // Generate sample data based on synthesis type
    let sample_data = match &instr.synthesis {
        InstrumentSynthesis::Sample { .. } => {
            // Sample-based: return empty data (would load from file in full impl)
            vec![]
        }
        synth => {
            // Synthesize sample
            let (data, _loop_start, _loop_length) =
                generate_loopable_sample(synth, 60, DEFAULT_SAMPLE_RATE, 4, instr_seed);
            data
        }
    };

    // Create sample
    let sample = ItSample::new(&instr.name, sample_data, DEFAULT_SAMPLE_RATE);
    let sample_length = sample.length_samples();

    // Set loop if needed
    let mut sample = if !matches!(instr.synthesis, InstrumentSynthesis::Sample { .. }) {
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
                if let Some(code) = xm_effect_name_to_code(&effect.r#type) {
                    n = n.with_effect(code, effect.param);
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
                if let Some(code) = it_effect_name_to_code(&effect.r#type) {
                    n = n.with_effect(code, effect.param);
                }
            }

            n
        };

        it_pattern.set_note(note.row, note.channel, it_note);
    }

    Ok(it_pattern)
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
            synthesis: InstrumentSynthesis::Pulse { duty_cycle: 0.5 },
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
        }
    }

    #[test]
    fn test_generate_xm() {
        let params = create_test_params();
        let result = generate_music(&params, 42).unwrap();

        assert_eq!(result.extension, "xm");
        assert!(!result.data.is_empty());
        assert_eq!(result.hash.len(), 64);
    }

    #[test]
    fn test_generate_it() {
        let mut params = create_test_params();
        params.format = TrackerFormat::It;

        let result = generate_music(&params, 42).unwrap();

        assert_eq!(result.extension, "it");
        assert!(!result.data.is_empty());
        assert_eq!(result.hash.len(), 64);
    }

    #[test]
    fn test_determinism() {
        let params = create_test_params();

        let result1 = generate_music(&params, 42).unwrap();
        let result2 = generate_music(&params, 42).unwrap();

        assert_eq!(result1.hash, result2.hash);
        assert_eq!(result1.data, result2.data);
    }

    #[test]
    fn test_different_seeds_different_output() {
        // Use noise synthesis which uses the seed
        let mut params = create_test_params();
        params.instruments[0].synthesis = InstrumentSynthesis::Noise { periodic: false };

        let result1 = generate_music(&params, 42).unwrap();
        let result2 = generate_music(&params, 43).unwrap();

        // Different seeds should produce different hashes (due to noise synthesis)
        assert_ne!(result1.hash, result2.hash);
    }

    #[test]
    fn test_invalid_channels() {
        let mut params = create_test_params();
        params.channels = 100; // Invalid for XM (max 32)

        let result = generate_music(&params, 42);
        assert!(result.is_err());
    }
}
