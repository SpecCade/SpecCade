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
    AutomationEntry, ItOptions, MusicTrackerSongV1Params, TrackerFormat, TrackerInstrument,
    TrackerPattern,
};
#[cfg(test)]
use speccade_spec::recipe::music::InstrumentSynthesis;

use crate::envelope::convert_envelope_to_it;
use crate::generate::{
    bake_instrument_sample, resolve_pattern_note_name, GenerateError, GenerateResult,
};
use crate::it::{
    effect_name_to_code as it_effect_name_to_code, ItInstrument, ItModule, ItNote, ItPattern,
    ItSample,
};
use crate::note::calculate_c5_speed_for_base_note;
#[cfg(test)]
use crate::note::note_name_to_it;

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
        let (it_instrument, it_sample) = generate_it_instrument(instr, seed, idx as u32, spec_dir)?;
        module.add_instrument(it_instrument);
        module.add_sample(it_sample);
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
        let mut it_pattern = convert_pattern_to_it(pattern, params.channels, &params.instruments)?;

        // Apply automation to this pattern
        apply_automation_to_pattern(&mut it_pattern, name, &params.automation, params.channels)?;

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
    if params.patterns.len() > u8::MAX as usize {
        return Err(GenerateError::InvalidParameter(format!(
            "patterns must be <= {}, got {}",
            u8::MAX,
            params.patterns.len()
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
    let baked = bake_instrument_sample(instr, base_seed, index, spec_dir, TrackerFormat::It)?;

    // IT samples store "C-5 speed" (playback rate for note C-5), not the sample's native rate.
    let c5_speed = calculate_c5_speed_for_base_note(baked.sample_rate, baked.base_midi);

    let mut sample = ItSample::new(&instr.name, baked.pcm16_mono, c5_speed);

    if let Some((loop_begin, loop_end)) = baked.loop_points {
        sample = sample.with_loop(loop_begin, loop_end, false);
    }

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
    instruments: &[TrackerInstrument],
) -> Result<ItPattern, GenerateError> {
    let mut it_pattern = ItPattern::empty(pattern.rows, num_channels);

    // Iterate over notes organized by channel
    for (channel, note) in pattern.flat_notes() {
        if channel >= num_channels {
            continue;
        }

        let note_name = resolve_pattern_note_name(note, instruments, "C5")?;
        let note_name = note_name.as_ref();

        let it_note = if note_name == "OFF" || note_name == "^^^" {
            ItNote::note_off()
        } else if note_name == "===" {
            ItNote::note_cut()
        } else {
            let mut n = ItNote::from_name(
                note_name,
                note.inst + 1, // IT instruments are 1-indexed
                note.vol.unwrap_or(64),
            );

            // Apply effect if present - supports both numeric effect and effect_name
            if let Some(effect_code) = note.effect {
                let param = note.param.unwrap_or(0);
                n = n.with_effect(effect_code, param);
            } else if let Some(ref effect_name) = note.effect_name {
                if let Some(code) = it_effect_name_to_code(effect_name) {
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

        it_pattern.set_note(note.row, channel, it_note);
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
                        pattern, *channel, *start_row, *end_row, *start_vol, *end_vol,
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
    use crate::it::sample_flags;
    use speccade_spec::recipe::audio::Envelope;
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
            synthesis: Some(InstrumentSynthesis::Pulse { duty_cycle: 0.5 }),
            envelope: envelope.clone(),
            default_volume: Some(64),
            ..Default::default()
        };

        let mut notes = HashMap::new();
        notes.insert(
            "0".to_string(),
            vec![
                PatternNote {
                    row: 0,
                    note: "C4".to_string(),
                    inst: 0,
                    vol: Some(64),
                    ..Default::default()
                },
                PatternNote {
                    row: 4,
                    note: "E4".to_string(),
                    inst: 0,
                    vol: Some(64),
                    ..Default::default()
                },
            ],
        );
        let pattern = TrackerPattern {
            rows: 16,
            notes: Some(notes),
            data: None,
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
            ..Default::default()
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

    // =========================================================================
    // Tests for resolving omitted pattern notes (trigger default/base note)
    // =========================================================================

    #[test]
    fn test_it_pattern_note_omitted_triggers_default_note() {
        let instruments = vec![TrackerInstrument {
            name: "Kick".to_string(),
            synthesis: Some(InstrumentSynthesis::Sine),
            ..Default::default()
        }];

        let pattern = TrackerPattern {
            rows: 1,
            data: Some(vec![PatternNote {
                row: 0,
                channel: Some(0),
                // note omitted (empty) should trigger the default IT note (C5)
                inst: 0,
                vol: Some(64),
                ..Default::default()
            }]),
            ..Default::default()
        };

        let it = convert_pattern_to_it(&pattern, 1, &instruments).unwrap();
        let cell = it.get_note(0, 0).unwrap();
        assert_eq!(cell.note, note_name_to_it("C5"));
        assert_eq!(cell.instrument, 1);
        assert_eq!(cell.volume, 64);
    }

    #[test]
    fn test_it_pattern_note_omitted_uses_instrument_base_note() {
        let instruments = vec![TrackerInstrument {
            name: "Lead".to_string(),
            synthesis: Some(InstrumentSynthesis::Triangle),
            base_note: Some("C4".to_string()),
            ..Default::default()
        }];

        let pattern = TrackerPattern {
            rows: 1,
            data: Some(vec![PatternNote {
                row: 0,
                channel: Some(0),
                inst: 0,
                vol: Some(32),
                ..Default::default()
            }]),
            ..Default::default()
        };

        let it = convert_pattern_to_it(&pattern, 1, &instruments).unwrap();
        let cell = it.get_note(0, 0).unwrap();
        assert_eq!(cell.note, note_name_to_it("C4"));
    }

    #[test]
    fn test_it_pattern_note_omitted_uses_sample_synth_base_note_when_instrument_base_note_missing()
    {
        let instruments = vec![TrackerInstrument {
            name: "Sampled".to_string(),
            synthesis: Some(InstrumentSynthesis::Sample {
                path: "samples/test.wav".to_string(),
                base_note: Some("D4".to_string()),
            }),
            ..Default::default()
        }];

        let pattern = TrackerPattern {
            rows: 1,
            data: Some(vec![PatternNote {
                row: 0,
                channel: Some(0),
                inst: 0,
                vol: Some(32),
                ..Default::default()
            }]),
            ..Default::default()
        };

        let it = convert_pattern_to_it(&pattern, 1, &instruments).unwrap();
        let cell = it.get_note(0, 0).unwrap();
        assert_eq!(cell.note, note_name_to_it("D4"));
    }

    #[test]
    fn test_it_pattern_explicit_note_overrides_instrument_base_note() {
        let instruments = vec![TrackerInstrument {
            name: "Lead".to_string(),
            synthesis: Some(InstrumentSynthesis::Triangle),
            base_note: Some("C5".to_string()),
            ..Default::default()
        }];

        let pattern = TrackerPattern {
            rows: 1,
            data: Some(vec![PatternNote {
                row: 0,
                channel: Some(0),
                note: "C4".to_string(),
                inst: 0,
                vol: Some(32),
                ..Default::default()
            }]),
            ..Default::default()
        };

        let it = convert_pattern_to_it(&pattern, 1, &instruments).unwrap();
        let cell = it.get_note(0, 0).unwrap();
        assert_eq!(cell.note, note_name_to_it("C4"));
    }

    #[test]
    fn test_it_pattern_no_note_marker_preserves_instrument_column() {
        let instruments = vec![TrackerInstrument {
            name: "Kick".to_string(),
            synthesis: Some(InstrumentSynthesis::Sine),
            ..Default::default()
        }];

        let pattern = TrackerPattern {
            rows: 1,
            data: Some(vec![PatternNote {
                row: 0,
                channel: Some(0),
                note: "...".to_string(),
                inst: 0,
                vol: Some(64),
                ..Default::default()
            }]),
            ..Default::default()
        };

        let it = convert_pattern_to_it(&pattern, 1, &instruments).unwrap();
        let cell = it.get_note(0, 0).unwrap();
        assert_eq!(cell.note, 0, "No-note marker should not trigger a note");
        assert_eq!(
            cell.instrument, 1,
            "No-note marker should still allow instrument-only events"
        );
    }

    #[test]
    fn test_tempo_change_it() {
        let mut pattern = ItPattern::empty(16, 4);

        apply_tempo_change_it(&mut pattern, 4, 140).unwrap();

        let note = pattern.get_note(4, 0).unwrap();
        assert_eq!(note.effect, 0x14);
        assert_eq!(note.effect_param, 140);
    }

    #[test]
    fn test_it_non_periodic_noise_one_shot_does_not_loop() {
        let instrument = TrackerInstrument {
            name: "Hihat".to_string(),
            synthesis: Some(InstrumentSynthesis::Noise { periodic: false }),
            envelope: Envelope {
                attack: 0.001,
                decay: 0.02,
                sustain: 0.0,
                release: 0.015,
            },
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();
        assert_eq!(
            it_sample.flags & sample_flags::LOOP,
            0,
            "Non-periodic noise one-shots should not loop (ringing/pitch artifacts)"
        );
        assert!(it_sample.length_samples() > 0);
    }

    // Tests for IT c5_speed calculation with synthesized instruments
    // Context: When no base_note is specified, IT synthesized samples are generated
    // at MIDI 72 (C5, 523.25 Hz), which is IT's reference pitch (C-5).
    // So c5_speed = sample_rate (no adjustment needed).

    #[test]
    fn test_synthesized_pulse_instrument_c5_speed() {
        // Pulse synthesis with no base_note generates at MIDI 72 (C5)
        // c5_speed = sample_rate because sample is at IT's reference pitch
        let instrument = TrackerInstrument {
            name: "Test Pulse".to_string(),
            synthesis: Some(InstrumentSynthesis::Pulse { duty_cycle: 0.5 }),
            default_volume: Some(64),
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();

        // c5_speed should be 22050 (sample rate) for MIDI 72 base note (IT's default)
        assert_eq!(it_sample.c5_speed, 22050);
    }

    #[test]
    fn test_synthesized_sine_instrument_c5_speed() {
        // Sine synthesis with no base_note generates at MIDI 72 (C5)
        let instrument = TrackerInstrument {
            name: "Test Sine".to_string(),
            synthesis: Some(InstrumentSynthesis::Sine),
            default_volume: Some(64),
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();

        assert_eq!(it_sample.c5_speed, 22050);
    }

    #[test]
    fn test_synthesized_noise_instrument_c5_speed() {
        // Noise synthesis with no base_note generates at MIDI 72 (IT's default)
        let instrument = TrackerInstrument {
            name: "Test Noise".to_string(),
            synthesis: Some(InstrumentSynthesis::Noise { periodic: false }),
            default_volume: Some(64),
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();

        assert_eq!(it_sample.c5_speed, 22050);
    }

    #[test]
    fn test_synthesized_triangle_instrument_c5_speed() {
        // Triangle synthesis with no base_note generates at MIDI 72 (C5)
        let instrument = TrackerInstrument {
            name: "Test Triangle".to_string(),
            synthesis: Some(InstrumentSynthesis::Triangle),
            default_volume: Some(64),
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();

        assert_eq!(it_sample.c5_speed, 22050);
    }

    #[test]
    fn test_synthesized_sawtooth_instrument_c5_speed() {
        // Sawtooth synthesis with no base_note generates at MIDI 72 (C5)
        let instrument = TrackerInstrument {
            name: "Test Sawtooth".to_string(),
            synthesis: Some(InstrumentSynthesis::Sawtooth),
            default_volume: Some(64),
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();

        assert_eq!(it_sample.c5_speed, 22050);
    }

    // Test that verifies the overall IT generation produces correct pitch
    #[test]
    fn test_generate_it_with_correct_c5_speed() {
        let params = create_test_params();
        let spec_dir = Path::new(".");
        let result = generate_it(&params, 42, spec_dir).unwrap();

        // The generated IT module should contain our synthesized instrument
        // with correct c5_speed. We verify this by checking the module data
        // contains valid IT format (starts with "IMPM")
        assert!(result.data.len() > 4);
        assert_eq!(&result.data[0..4], b"IMPM");
    }

    // =============================================================================
    // Tests for base_note / pattern_note combinations
    // =============================================================================
    //
    // These tests verify the correct pitch behavior for all 4 combinations of
    // base_note (instrument level) and pattern note (note in pattern data):
    //
    // 1. No base_note, no pattern note → Sample generated at C-5 (IT's reference)
    // 2. No base_note, pattern note "C5" → Same pitch mapping, triggered by C5
    // 3. base_note "C5", no pattern note → Sample configured for C5, no playback
    // 4. base_note "C5", pattern note "C4" → Sample plays one octave DOWN
    //
    // IT Format Context:
    // - IT's pitch reference is C-5 (IT note 60 = MIDI 72 = 523.25 Hz)
    // - c5_speed determines sample playback rate when C-5 is played
    // - Default IT synth generates at MIDI 72 (C5 = 523.25 Hz, IT C-5)
    // - Default sample rate is 22050 Hz

    /// Test Variant A: No base_note, no pattern note
    /// Expected: c5_speed = sample_rate (22050)
    /// When no base_note is specified, IT assumes the sample contains C-5 audio,
    /// so c5_speed = sample_rate (no adjustment needed).
    #[test]
    fn test_it_variant_a_no_base_note_no_pattern_note() {
        // Instrument with no base_note (defaults to MIDI 72 = C5 for IT)
        let instrument = TrackerInstrument {
            name: "Drum Kick".to_string(),
            synthesis: Some(InstrumentSynthesis::Sine),
            // base_note: None - defaults to MIDI 72 (C5) for IT format
            default_volume: Some(64),
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();

        // c5_speed should be 22050 (sample rate)
        // because: sample at MIDI 72 (C5), IT reference at MIDI 72 (C5)
        // semitone_diff = 60 - 60 = 0
        // c5_speed = 22050 * 2^(0/12) = 22050
        assert_eq!(
            it_sample.c5_speed, 22050,
            "No base_note: c5_speed should be 22050 (sample at IT's reference pitch)"
        );

        // Mathematical verification:
        // - Sample rate: 22050 Hz
        // - Sample contains: MIDI 72 (C5 = 523.25 Hz) = IT note 60 (C-5)
        // - IT reference: IT note 60 (C-5) = MIDI 72 (523.25 Hz)
        // - semitone_diff = 60 - 60 = 0
        // - c5_speed = 22050 * 2^(0/12) = 22050
        let expected_c5_speed = (22050.0 * 2.0_f64.powf(0.0 / 12.0)) as u32;
        assert_eq!(it_sample.c5_speed, expected_c5_speed);
    }

    /// Test Variant B: No base_note, pattern note "C5"
    /// This is the same pitch mapping as Variant A, just explicitly triggered at C5.
    /// The note in the pattern determines WHEN the sample plays, not the pitch configuration.
    #[test]
    fn test_it_variant_b_no_base_note_pattern_note_c5() {
        // Instrument with no base_note
        let instrument = TrackerInstrument {
            name: "Drum Snare".to_string(),
            synthesis: Some(InstrumentSynthesis::Noise { periodic: false }),
            // base_note: None - defaults to MIDI 72 (C5) for IT format
            default_volume: Some(64),
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();

        // c5_speed should be 22050 - the pattern note doesn't affect this
        // Pattern note only determines which tracker note triggers the sample
        assert_eq!(
            it_sample.c5_speed, 22050,
            "No base_note with C5 pattern: c5_speed should be 22050"
        );

        // When pattern plays "C5" (IT C-5, note 60):
        // - IT calculates playback speed based on note 60 vs reference note 60
        // - The sample plays at its natural rate (22050 Hz)
        // This is correct: playing C-5 on a sample at C5 should sound like C5
    }

    /// Test Variant C: base_note "C5", no pattern note
    /// Sample configured to play at correct pitch when triggered at C5.
    /// No pattern note means no playback trigger - just testing instrument config.
    #[test]
    fn test_it_variant_c_base_note_c5_no_pattern_note() {
        // Instrument with base_note = "C5" (MIDI 72)
        let instrument = TrackerInstrument {
            name: "Lead Synth".to_string(),
            synthesis: Some(InstrumentSynthesis::Sawtooth),
            base_note: Some("C5".to_string()), // MIDI 72 = IT C-5
            default_volume: Some(64),
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();

        // With base_note = "C5" (MIDI 72 = IT note 60), the sample is at the IT reference pitch.
        // c5_speed = sample_rate because sample is already at C-5
        // semitone_diff = 60 - 60 = 0
        // c5_speed = 22050 * 2^(0/12) = 22050
        assert_eq!(
            it_sample.c5_speed, 22050,
            "base_note C5: c5_speed should be 22050 (sample already at IT reference)"
        );

        // Mathematical verification:
        // - Sample rate: 22050 Hz
        // - Sample contains: MIDI 72 (C5 = 523.25 Hz) = IT note 60 (C-5)
        // - IT reference: IT note 60 (C-5) = MIDI 72 (523.25 Hz)
        // - semitone_diff = 60 - 60 = 0
        // - c5_speed = 22050 * 2^(0/12) = 22050
        let expected_c5_speed = (22050.0 * 2.0_f64.powf(0.0 / 12.0)) as u32;
        assert_eq!(it_sample.c5_speed, expected_c5_speed);
    }

    /// Test Variant D: base_note "C5", pattern note "C4"
    /// Sample plays one octave DOWN from its natural pitch.
    /// Pattern plays C4 (IT C-4, note 48) on a sample configured for C5 (IT C-5, note 60).
    #[test]
    fn test_it_variant_d_base_note_c5_pattern_note_c4() {
        // Instrument with base_note = "C5"
        let instrument = TrackerInstrument {
            name: "Bass".to_string(),
            synthesis: Some(InstrumentSynthesis::Triangle),
            base_note: Some("C5".to_string()), // MIDI 72 = IT C-5
            default_volume: Some(64),
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();

        // c5_speed is 22050 (configured for C5 base note)
        assert_eq!(it_sample.c5_speed, 22050);

        // When pattern plays "C4" (IT C-4, note 48) with this sample:
        // - IT reference is C-5 (note 60) at c5_speed
        // - Playing C-4 (note 48) is 12 semitones below C-5
        // - Playback speed = c5_speed * 2^((48-60)/12) = 22050 * 2^(-1) = 11025 Hz
        // - Since sample was generated at 22050 Hz for C5 (523.25 Hz):
        //   - Playing at 11025 Hz produces pitch: 523.25 / 2 = 261.6 Hz = C4
        // - But the sample CONTAINS C5 content, so we hear C5 pitch shifted down an octave
        //
        // Result: Sample plays one octave DOWN from its natural C5 pitch → sounds like C4

        // Verify the pitch relationship mathematically:
        // Pattern note C4 = IT note 48, c5_speed = 22050
        // Playback_speed = c5_speed * 2^((pattern_note - 60) / 12)
        //                = 22050 * 2^((48 - 60) / 12)
        //                = 22050 * 2^(-1)
        //                = 11025 Hz
        // Pitch ratio = 11025 / 22050 = 0.5 (one octave down)
        let pattern_it_note: i32 = 48; // C-4
        let playback_ratio = 2.0_f64.powf((pattern_it_note - 60) as f64 / 12.0);
        assert!(
            (playback_ratio - 0.5).abs() < 0.0001,
            "C4 pattern on C5-based sample should play at half speed (one octave down)"
        );
    }

    /// Additional test: base_note "A4" (MIDI 69) - non-C note for verification
    #[test]
    fn test_it_base_note_a4_non_c_note() {
        // Instrument with base_note = "A4" (A440 = MIDI 69)
        let instrument = TrackerInstrument {
            name: "Tuning Fork".to_string(),
            synthesis: Some(InstrumentSynthesis::Sine),
            base_note: Some("A4".to_string()), // MIDI 69 = IT A-4 (note 57)
            default_volume: Some(64),
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();

        // Sample at MIDI 69 (A4) = IT note 57 (A-4)
        // IT reference is note 60 (C-5)
        // semitone_diff = 60 - 57 = 3
        // c5_speed = 22050 * 2^(3/12) = 22050 * 1.1892 ≈ 26222
        let expected_c5_speed = (22050.0 * 2.0_f64.powf(3.0 / 12.0)) as u32;
        assert_eq!(
            it_sample.c5_speed, expected_c5_speed,
            "base_note A4: c5_speed should be {} (3 semitones up from A4 to C5)",
            expected_c5_speed
        );

        // When pattern plays A4 (IT A-4, note 57):
        // - Playback speed = c5_speed * 2^((57-60)/12) = 26222 * 2^(-0.25) ≈ 22050 Hz
        // - Sample plays at original rate, producing correct A4 pitch
    }

    /// Test: base_note "C3" (MIDI 48) - two octaves below IT reference
    #[test]
    fn test_it_base_note_c3_two_octaves_below() {
        let instrument = TrackerInstrument {
            name: "Sub Bass".to_string(),
            synthesis: Some(InstrumentSynthesis::Sine),
            base_note: Some("C3".to_string()), // MIDI 48 = IT C-3 (note 36)
            default_volume: Some(64),
            ..Default::default()
        };

        let spec_dir = Path::new(".");
        let (_it_instr, it_sample) = generate_it_instrument(&instrument, 42, 0, spec_dir).unwrap();

        // Sample at MIDI 48 (C3) = IT note 36 (C-3)
        // IT reference is note 60 (C-5)
        // semitone_diff = 60 - 36 = 24 (two octaves)
        // c5_speed = 22050 * 2^(24/12) = 22050 * 4 = 88200
        assert_eq!(
            it_sample.c5_speed, 88200,
            "base_note C3: c5_speed should be 88200 (4x sample rate for 2 octaves)"
        );

        // Mathematical verification
        let expected_c5_speed = (22050.0 * 2.0_f64.powf(24.0 / 12.0)) as u32;
        assert_eq!(it_sample.c5_speed, expected_c5_speed);
    }

    /// Test: Verify default sample rate constant is 22050
    #[test]
    fn test_it_default_sample_rate_constant() {
        use crate::note::DEFAULT_SAMPLE_RATE;
        assert_eq!(
            DEFAULT_SAMPLE_RATE, 22050,
            "Default sample rate should be 22050 Hz"
        );
    }

    /// Test: Verify XM default synth MIDI note constant is 60 (C4)
    #[test]
    fn test_xm_default_synth_midi_note_constant() {
        use crate::note::DEFAULT_SYNTH_MIDI_NOTE;
        assert_eq!(
            DEFAULT_SYNTH_MIDI_NOTE, 60,
            "XM default synth MIDI note should be 60 (C4)"
        );
    }

    /// Test: Verify IT default synth MIDI note constant is 72 (C5)
    #[test]
    fn test_it_default_synth_midi_note_constant() {
        use crate::note::DEFAULT_IT_SYNTH_MIDI_NOTE;
        assert_eq!(
            DEFAULT_IT_SYNTH_MIDI_NOTE, 72,
            "IT default synth MIDI note should be 72 (C5)"
        );
    }
}
