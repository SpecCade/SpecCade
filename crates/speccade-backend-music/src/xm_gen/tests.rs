//! Tests for XM generation module.

use super::*;
use std::collections::HashMap;
use std::path::Path;

use speccade_spec::recipe::audio::Envelope;
use speccade_spec::recipe::music::{
    ArrangementEntry, InstrumentSynthesis, PatternNote, TrackerFormat,
};

use crate::note::note_name_to_xm;

fn create_test_params() -> MusicTrackerSongV1Params {
    let envelope = Envelope {
        attack: 0.01,
        decay: 0.1,
        sustain: 0.5,
        release: 0.2,
    };

    let instrument = TrackerInstrument {
        name: "Test Lead".to_string(),
        synthesis: Some(InstrumentSynthesis::Pulse {
            duty_cycle: 0.5,
            base_note: None,
        }),
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
        ..Default::default()
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
fn test_xm_param_validation_rejects_bpm_above_tracker_limit() {
    let mut params = create_test_params();
    params.bpm = 300;

    let err = validate_xm_params(&params).unwrap_err();
    assert!(err.to_string().contains("bpm must be 32-255"));
}

#[test]
fn test_volume_fade_xm() {
    let mut pattern = XmPattern::empty(16, 4);

    apply_volume_fade_xm(&mut pattern, 0, 0, 8, 64, 0).unwrap();

    // Check interpolation
    let note_start = pattern.get_note(0, 0).unwrap();
    assert_eq!(note_start.volume, 0x10 + 64);

    let note_end = pattern.get_note(8, 0).unwrap();
    assert_eq!(note_end.volume, 0x10);
}

#[test]
fn test_volume_fade_xm_rejects_out_of_range_channel() {
    let mut pattern = XmPattern::empty(16, 1);
    let err = apply_volume_fade_xm(&mut pattern, 3, 0, 8, 64, 0).unwrap_err();
    assert!(err.to_string().contains("channel 3 out of range"));
}

#[test]
fn test_tempo_change_xm_rejects_out_of_range_row() {
    let mut pattern = XmPattern::empty(4, 1);
    let err = apply_tempo_change_xm(&mut pattern, 8, 140).unwrap_err();
    assert!(err.to_string().contains("tempo change row 8 out of range"));
}

// =========================================================================
// Tests for resolving omitted pattern notes (trigger default/base note)
// =========================================================================

#[test]
fn test_xm_pattern_note_omitted_triggers_default_note() {
    let instruments = vec![TrackerInstrument {
        name: "Kick".to_string(),
        synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
        ..Default::default()
    }];

    let pattern = TrackerPattern {
        rows: 1,
        data: Some(vec![PatternNote {
            row: 0,
            channel: Some(0),
            // note omitted (empty) should trigger the default XM note (C4)
            inst: 0,
            vol: Some(64),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let xm = convert_pattern_to_xm(&pattern, 1, &instruments).unwrap();
    let cell = xm.get_note(0, 0).unwrap();
    assert_eq!(cell.note, note_name_to_xm("C4"));
    assert_eq!(cell.instrument, 1);
    assert_eq!(cell.volume, 0x10 + 64);
}

#[test]
fn test_xm_pattern_note_omitted_uses_instrument_base_note() {
    let instruments = vec![TrackerInstrument {
        name: "Lead".to_string(),
        synthesis: Some(InstrumentSynthesis::Triangle { base_note: None }),
        base_note: Some("C5".to_string()),
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

    let xm = convert_pattern_to_xm(&pattern, 1, &instruments).unwrap();
    let cell = xm.get_note(0, 0).unwrap();
    assert_eq!(cell.note, note_name_to_xm("C5"));
}

#[test]
fn test_xm_pattern_note_omitted_uses_sample_synth_base_note_when_instrument_base_note_missing() {
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

    let xm = convert_pattern_to_xm(&pattern, 1, &instruments).unwrap();
    let cell = xm.get_note(0, 0).unwrap();
    assert_eq!(cell.note, note_name_to_xm("D4"));
}

#[test]
fn test_xm_pattern_explicit_note_overrides_instrument_base_note() {
    let instruments = vec![TrackerInstrument {
        name: "Lead".to_string(),
        synthesis: Some(InstrumentSynthesis::Triangle { base_note: None }),
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

    let xm = convert_pattern_to_xm(&pattern, 1, &instruments).unwrap();
    let cell = xm.get_note(0, 0).unwrap();
    assert_eq!(cell.note, note_name_to_xm("C4"));
}

#[test]
fn test_xm_pattern_no_note_marker_preserves_instrument_column() {
    let instruments = vec![TrackerInstrument {
        name: "Kick".to_string(),
        synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
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

    let xm = convert_pattern_to_xm(&pattern, 1, &instruments).unwrap();
    let cell = xm.get_note(0, 0).unwrap();
    assert_eq!(cell.note, 0, "No-note marker should not trigger a note");
    assert_eq!(
        cell.instrument, 1,
        "No-note marker should still allow instrument-only events"
    );
}

#[test]
fn test_xm_pattern_rejects_out_of_range_channel() {
    let instruments = vec![TrackerInstrument {
        name: "Lead".to_string(),
        synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
        ..Default::default()
    }];

    let pattern = TrackerPattern {
        rows: 1,
        data: Some(vec![PatternNote {
            row: 0,
            channel: Some(2),
            note: "C4".to_string(),
            inst: 0,
            ..Default::default()
        }]),
        ..Default::default()
    };

    let err = convert_pattern_to_xm(&pattern, 1, &instruments).unwrap_err();
    assert!(
        err.to_string().contains("exceeds configured channel count"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn test_xm_pattern_rejects_out_of_range_row() {
    let instruments = vec![TrackerInstrument {
        name: "Lead".to_string(),
        synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
        ..Default::default()
    }];

    let pattern = TrackerPattern {
        rows: 1,
        data: Some(vec![PatternNote {
            row: 4,
            channel: Some(0),
            note: "C4".to_string(),
            inst: 0,
            ..Default::default()
        }]),
        ..Default::default()
    };

    let err = convert_pattern_to_xm(&pattern, 1, &instruments).unwrap_err();
    assert!(err.to_string().contains("row 4 is out of range"));
}

#[test]
fn test_xm_pattern_rejects_unknown_effect_name() {
    let instruments = vec![TrackerInstrument {
        name: "Lead".to_string(),
        synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
        ..Default::default()
    }];

    let pattern = TrackerPattern {
        rows: 1,
        data: Some(vec![PatternNote {
            row: 0,
            channel: Some(0),
            note: "C4".to_string(),
            inst: 0,
            effect_name: Some("does_not_exist".to_string()),
            param: Some(1),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let err = convert_pattern_to_xm(&pattern, 1, &instruments).unwrap_err();
    assert!(err.to_string().contains("unknown effect_name"));
}

#[test]
fn test_xm_pattern_rejects_it_only_effect_name() {
    let instruments = vec![TrackerInstrument {
        name: "Lead".to_string(),
        synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
        ..Default::default()
    }];

    let pattern = TrackerPattern {
        rows: 1,
        data: Some(vec![PatternNote {
            row: 0,
            channel: Some(0),
            note: "C4".to_string(),
            inst: 0,
            effect_name: Some("set_channel_volume".to_string()),
            param: Some(32),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let err = convert_pattern_to_xm(&pattern, 1, &instruments).unwrap_err();
    assert!(err.to_string().contains("not supported in XM"));
}

#[test]
fn test_xm_pattern_rejects_explicit_note_with_missing_instrument() {
    let pattern = TrackerPattern {
        rows: 1,
        data: Some(vec![PatternNote {
            row: 0,
            channel: Some(0),
            note: "C4".to_string(),
            inst: 0,
            ..Default::default()
        }]),
        ..Default::default()
    };

    let err = convert_pattern_to_xm(&pattern, 1, &[]).unwrap_err();
    assert!(err.to_string().contains("pattern references instrument 0"));
}

#[test]
fn test_xm_non_periodic_noise_one_shot_does_not_loop() {
    let instrument = TrackerInstrument {
        name: "Hihat".to_string(),
        synthesis: Some(InstrumentSynthesis::Noise {
            periodic: false,
            base_note: None,
        }),
        envelope: Envelope {
            attack: 0.001,
            decay: 0.02,
            sustain: 0.0,
            release: 0.015,
        },
        ..Default::default()
    };

    let spec_dir = Path::new(".");
    let (xm_instr, _) = generate_xm_instrument(&instrument, 42, 0, spec_dir).unwrap();
    assert_eq!(
        xm_instr.sample.loop_type, 0,
        "Non-periodic noise one-shots should not loop (ringing/pitch artifacts)"
    );
    assert!(
        xm_instr.sample.length_samples() > 0,
        "One-shot sample should have non-zero length"
    );
}

// =========================================================================
// Tests for XM instrument pitch correction
// =========================================================================
// Context: Synthesized instruments generate at MIDI 60 (C4) at 22050 Hz.
// The relative_note should be 16 to compensate for both sample rate and base note.

/// Helper to create a minimal instrument for testing pitch correction
fn create_test_instrument(synthesis: InstrumentSynthesis) -> TrackerInstrument {
    TrackerInstrument {
        name: "Test".to_string(),
        synthesis: Some(synthesis),
        default_volume: Some(64),
        ..Default::default()
    }
}

#[test]
fn test_synthesized_pulse_instrument_pitch_correction() {
    // Synthesized Pulse instrument generates at MIDI 60 (C4), 22050 Hz
    // Should get relative_note = 16
    let instr = create_test_instrument(InstrumentSynthesis::Pulse {
        duty_cycle: 0.5,
        base_note: None,
    });
    let spec_dir = Path::new(".");

    let (xm_instr, _) = generate_xm_instrument(&instr, 42, 0, spec_dir).unwrap();

    // Access the sample's relative_note (XmInstrument has a single sample)
    assert_eq!(
        xm_instr.sample.relative_note, 16,
        "Pulse instrument should have relative_note = 16"
    );
}

#[test]
fn test_synthesized_sine_instrument_pitch_correction() {
    // Synthesized Sine instrument generates at MIDI 60 (C4), 22050 Hz
    // Should get relative_note = 16
    let instr = create_test_instrument(InstrumentSynthesis::Sine { base_note: None });
    let spec_dir = Path::new(".");

    let (xm_instr, _) = generate_xm_instrument(&instr, 42, 0, spec_dir).unwrap();

    assert_eq!(
        xm_instr.sample.relative_note, 16,
        "Sine instrument should have relative_note = 16"
    );
}

#[test]
fn test_synthesized_noise_instrument_pitch_correction() {
    // Synthesized Noise instrument generates at MIDI 60 (C4), 22050 Hz
    // Should get relative_note = 16
    let instr = create_test_instrument(InstrumentSynthesis::Noise {
        periodic: false,
        base_note: None,
    });
    let spec_dir = Path::new(".");

    let (xm_instr, _) = generate_xm_instrument(&instr, 42, 0, spec_dir).unwrap();

    assert_eq!(
        xm_instr.sample.relative_note, 16,
        "Noise instrument should have relative_note = 16"
    );
}

#[test]
fn test_synthesized_triangle_instrument_pitch_correction() {
    // Synthesized Triangle instrument generates at MIDI 60 (C4), 22050 Hz
    // Should get relative_note = 16
    let instr = create_test_instrument(InstrumentSynthesis::Triangle { base_note: None });
    let spec_dir = Path::new(".");

    let (xm_instr, _) = generate_xm_instrument(&instr, 42, 0, spec_dir).unwrap();

    assert_eq!(
        xm_instr.sample.relative_note, 16,
        "Triangle instrument should have relative_note = 16"
    );
}

#[test]
fn test_synthesized_sawtooth_instrument_pitch_correction() {
    // Synthesized Sawtooth instrument generates at MIDI 60 (C4), 22050 Hz
    // Should get relative_note = 16
    let instr = create_test_instrument(InstrumentSynthesis::Sawtooth { base_note: None });
    let spec_dir = Path::new(".");

    let (xm_instr, _) = generate_xm_instrument(&instr, 42, 0, spec_dir).unwrap();

    assert_eq!(
        xm_instr.sample.relative_note, 16,
        "Sawtooth instrument should have relative_note = 16"
    );
}

#[test]
fn test_finetune_value_for_synthesized_instruments() {
    // All synthesized instruments at 22050 Hz should have finetune ~101
    // (from the fractional part of 16.79 semitones)
    let instr = create_test_instrument(InstrumentSynthesis::Pulse {
        duty_cycle: 0.5,
        base_note: None,
    });
    let spec_dir = Path::new(".");

    let (xm_instr, _) = generate_xm_instrument(&instr, 42, 0, spec_dir).unwrap();

    let finetune = xm_instr.sample.finetune;
    assert!(
        (100..=102).contains(&finetune),
        "Finetune should be ~101, got {}",
        finetune
    );
}

// =============================================================================
// Tests for base_note / pattern_note combinations
// =============================================================================
//
// These tests verify the correct pitch behavior for all 4 combinations of
// base_note (instrument level) and pattern note (note in pattern data):
//
// 1. No base_note, no pattern note → Plays at tracker's default (XM: C-4)
// 2. No base_note, pattern note "C4" → Same pitch mapping, triggered by C4
// 3. base_note "C5", no pattern note → Sample configured for C5, no playback
// 4. base_note "C5", pattern note "C4" → Sample plays one octave DOWN
//
// XM Format Context:
// - XM's pitch reference is C-4 (XM note 49 = MIDI 60 = 261.6 Hz) at 8363 Hz
// - relative_note and finetune adjust pitch from the reference
// - Default synth generates at MIDI 60 (C4) at 22050 Hz sample rate
// - relative_note formula: 48 + rate_correction - base_note_0indexed
//   where base_note_0indexed = MIDI - 12

/// Test Variant A: No base_note, no pattern note
/// Expected: relative_note = 16, finetune ~101
/// The sample rate compensation (16 semitones) is needed because 22050/8363 ratio.
#[test]
fn test_xm_variant_a_no_base_note_no_pattern_note() {
    // Instrument with no base_note (defaults to MIDI 60 = C4)
    let instrument = TrackerInstrument {
        name: "Drum Kick".to_string(),
        synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
        // base_note: None - defaults to MIDI 60 (C4)
        default_volume: Some(64),
        ..Default::default()
    };

    let spec_dir = Path::new(".");
    let (xm_instr, _) = generate_xm_instrument(&instrument, 42, 0, spec_dir).unwrap();

    // relative_note should be 16 (sample rate compensation only, since base is at reference)
    // rate_correction: 12 * log2(22050/8363) = 12 * 1.399 = 16.79, floor = 16
    // base_note_0indexed = MIDI 60 - 12 = 48 (XM C-4 is note 48, 0-indexed)
    // relative_note = 48 + 16 - 48 = 16
    assert_eq!(
        xm_instr.sample.relative_note, 16,
        "No base_note: relative_note should be 16 (sample rate compensation)"
    );

    // Finetune should be ~101 (fractional semitone: 0.79 * 128 ≈ 101)
    assert!(
        xm_instr.sample.finetune >= 100 && xm_instr.sample.finetune <= 102,
        "No base_note: finetune should be ~101, got {}",
        xm_instr.sample.finetune
    );

    // Mathematical verification:
    // Sample rate ratio: 22050 / 8363 = 2.637
    // Semitones: 12 * log2(2.637) = 16.79
    // relative_note = floor(16.79) = 16 (for MIDI 60 = XM note 48, 0-indexed)
    // Since 48 + 16 - 48 = 16, the sample at C4 will play correctly at XM C-4
    let rate_semitones = 12.0 * (22050.0_f64 / 8363.0).log2();
    let expected_relative_note = rate_semitones.floor() as i8 + 48 - 48; // base at MIDI 60
    assert_eq!(xm_instr.sample.relative_note, expected_relative_note);
}

/// Test Variant B: No base_note, pattern note "C4"
/// Same pitch configuration as Variant A; pattern note determines trigger.
#[test]
fn test_xm_variant_b_no_base_note_pattern_note_c4() {
    // Instrument with no base_note
    let instrument = TrackerInstrument {
        name: "Drum Snare".to_string(),
        synthesis: Some(InstrumentSynthesis::Noise {
            periodic: false,
            base_note: None,
        }),
        // base_note: None - defaults to MIDI 60 (C4)
        default_volume: Some(64),
        ..Default::default()
    };

    let spec_dir = Path::new(".");
    let (xm_instr, _) = generate_xm_instrument(&instrument, 42, 0, spec_dir).unwrap();

    // relative_note should still be 16 - pattern note doesn't affect this
    assert_eq!(
        xm_instr.sample.relative_note, 16,
        "No base_note with C4 pattern: relative_note should still be 16"
    );

    // When pattern plays "C4" (XM C-4, note 49 = index 48):
    // - XM adds relative_note to the pattern note for pitch calculation
    // - Effective note = 48 + 16 = 64
    // - This correctly plays the sample at the rate needed for C4 pitch
}

/// Test Variant C: base_note "C5", no pattern note
/// Sample configured to play at correct pitch when triggered at C5.
#[test]
fn test_xm_variant_c_base_note_c5_no_pattern_note() {
    // Instrument with base_note = "C5" (MIDI 72)
    let instrument = TrackerInstrument {
        name: "Lead Synth".to_string(),
        synthesis: Some(InstrumentSynthesis::Sawtooth { base_note: None }),
        base_note: Some("C5".to_string()), // MIDI 72 = XM C-5 (note 61 = index 60)
        default_volume: Some(64),
        ..Default::default()
    };

    let spec_dir = Path::new(".");
    let (xm_instr, _) = generate_xm_instrument(&instrument, 42, 0, spec_dir).unwrap();

    // With base_note = "C5" (MIDI 72 = XM note 60, 0-indexed):
    // rate_correction = 16 (same as before, from 22050/8363 ratio)
    // base_note_0indexed = MIDI 72 - 12 = 60
    // relative_note = 48 + 16 - 60 = 4
    assert_eq!(
        xm_instr.sample.relative_note, 4,
        "base_note C5: relative_note should be 4"
    );

    // Mathematical verification:
    // The sample contains C5 (one octave above XM reference C-4)
    // So we need LESS pitch adjustment (smaller relative_note)
    // relative_note = 48 + 16 - 60 = 4
    // When playing C-5 (note 61, index 60):
    //   effective_note = 60 + 4 = 64
    //   This matches the expected playback for C5 content
}

/// Test Variant D: base_note "C5", pattern note "C4"
/// Sample plays one octave DOWN from its natural pitch.
#[test]
fn test_xm_variant_d_base_note_c5_pattern_note_c4() {
    // Instrument with base_note = "C5"
    let instrument = TrackerInstrument {
        name: "Bass".to_string(),
        synthesis: Some(InstrumentSynthesis::Triangle { base_note: None }),
        base_note: Some("C5".to_string()), // MIDI 72 = XM C-5
        default_volume: Some(64),
        ..Default::default()
    };

    let spec_dir = Path::new(".");
    let (xm_instr, _) = generate_xm_instrument(&instrument, 42, 0, spec_dir).unwrap();

    // relative_note is 4 (configured for C5 base note)
    assert_eq!(xm_instr.sample.relative_note, 4);

    // When pattern plays "C4" (XM C-4, note 49 = index 48) with this sample:
    // - Effective note = 48 + 4 = 52
    // - But the sample was recorded at C5 (effective note 64 for natural playback)
    // - Difference: 52 - 64 = -12 semitones (one octave down)
    //
    // Result: Sample plays one octave DOWN from its natural C5 pitch → sounds like C4

    // Verify the pitch relationship:
    // Pattern C4 (index 48) + relative_note (4) = effective 52
    // Natural C5 playback would be index 60 + relative_note (4) = effective 64
    // Difference: 52 - 64 = -12 semitones = one octave down
    let pattern_note_index: i32 = 48; // C-4, 0-indexed
    let natural_note_index: i32 = 60; // C-5, 0-indexed
    let relative_note = xm_instr.sample.relative_note as i32;

    let pattern_effective = pattern_note_index + relative_note;
    let natural_effective = natural_note_index + relative_note;
    let semitone_diff = pattern_effective - natural_effective;

    assert_eq!(
        semitone_diff, -12,
        "C4 pattern on C5-based sample should be 12 semitones (one octave) down"
    );
}

/// Additional test: base_note "A4" (MIDI 69) - non-C note for verification
#[test]
fn test_xm_base_note_a4_non_c_note() {
    // Instrument with base_note = "A4" (A440 = MIDI 69)
    let instrument = TrackerInstrument {
        name: "Tuning Fork".to_string(),
        synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
        base_note: Some("A4".to_string()), // MIDI 69 = XM A-4 (note 58, index 57)
        default_volume: Some(64),
        ..Default::default()
    };

    let spec_dir = Path::new(".");
    let (xm_instr, _) = generate_xm_instrument(&instrument, 42, 0, spec_dir).unwrap();

    // Sample at MIDI 69 (A4) = XM note 57 (0-indexed)
    // rate_correction = 16
    // base_note_0indexed = MIDI 69 - 12 = 57
    // relative_note = 48 + 16 - 57 = 7
    let expected_relative_note = 48 + 16 - 57;
    assert_eq!(
        xm_instr.sample.relative_note, expected_relative_note as i8,
        "base_note A4: relative_note should be {} (7 semitones adjustment)",
        expected_relative_note
    );

    // When pattern plays A4 (XM A-4, note 58, index 57):
    // - effective_note = 57 + 7 = 64
    // - This produces the correct playback rate for A4 pitch
}

/// Test: base_note "C3" (MIDI 48) - two octaves below XM reference
#[test]
fn test_xm_base_note_c3_two_octaves_below() {
    let instrument = TrackerInstrument {
        name: "Sub Bass".to_string(),
        synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
        base_note: Some("C3".to_string()), // MIDI 48 = XM C-3 (note 37, index 36)
        default_volume: Some(64),
        ..Default::default()
    };

    let spec_dir = Path::new(".");
    let (xm_instr, _) = generate_xm_instrument(&instrument, 42, 0, spec_dir).unwrap();

    // Sample at MIDI 48 (C3) = XM note 36 (0-indexed)
    // rate_correction = 16
    // base_note_0indexed = MIDI 48 - 12 = 36
    // relative_note = 48 + 16 - 36 = 28
    assert_eq!(
        xm_instr.sample.relative_note, 28,
        "base_note C3: relative_note should be 28 (2 octaves + rate compensation)"
    );

    // Mathematical verification:
    // The sample is 2 octaves below XM reference (C-4)
    // We need 24 semitones MORE relative_note compared to C4 base
    // C4 base: relative_note = 16
    // C3 base: relative_note = 16 + 12 = 28
    // But using formula: 48 + 16 - 36 = 28 ✓
}

/// Test: base_note "C6" (MIDI 84) - two octaves above XM reference
#[test]
fn test_xm_base_note_c6_two_octaves_above() {
    let instrument = TrackerInstrument {
        name: "High Lead".to_string(),
        synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
        base_note: Some("C6".to_string()), // MIDI 84 = XM C-6 (note 73, index 72)
        default_volume: Some(64),
        ..Default::default()
    };

    let spec_dir = Path::new(".");
    let (xm_instr, _) = generate_xm_instrument(&instrument, 42, 0, spec_dir).unwrap();

    // Sample at MIDI 84 (C6) = XM note 72 (0-indexed)
    // rate_correction = 16
    // base_note_0indexed = MIDI 84 - 12 = 72
    // relative_note = 48 + 16 - 72 = -8
    assert_eq!(
        xm_instr.sample.relative_note, -8,
        "base_note C6: relative_note should be -8 (sample above reference)"
    );

    // When pattern plays C-6 (index 72):
    // - effective_note = 72 + (-8) = 64
    // - This produces the correct playback for C6 content
}

/// Test: Verify XM relative_note relationship between octaves
#[test]
fn test_xm_relative_note_octave_relationships() {
    let spec_dir = Path::new(".");

    // Create instruments at C3, C4, C5, C6 (each one octave apart)
    let notes = [("C3", 48), ("C4", 60), ("C5", 72), ("C6", 84)];
    let mut relative_notes = Vec::new();

    for (note_name, _midi) in &notes {
        let instrument = TrackerInstrument {
            name: format!("{} Test", note_name),
            synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
            base_note: Some(note_name.to_string()),
            default_volume: Some(64),
            ..Default::default()
        };

        let (xm_instr, _) = generate_xm_instrument(&instrument, 42, 0, spec_dir).unwrap();
        relative_notes.push(xm_instr.sample.relative_note);
    }

    // Each octave difference should be 12 semitones in relative_note
    // Higher base_note = lower relative_note (formula subtracts base_note)
    assert_eq!(
        relative_notes[0] - relative_notes[1],
        12,
        "C3 to C4: relative_note should differ by 12"
    );
    assert_eq!(
        relative_notes[1] - relative_notes[2],
        12,
        "C4 to C5: relative_note should differ by 12"
    );
    assert_eq!(
        relative_notes[2] - relative_notes[3],
        12,
        "C5 to C6: relative_note should differ by 12"
    );
}

/// Test: Verify finetune stays consistent across different base_notes
#[test]
fn test_xm_finetune_consistent_across_base_notes() {
    let spec_dir = Path::new(".");

    // Finetune should be the same regardless of base_note
    // (it only depends on sample rate)
    let notes = ["C3", "C4", "C5", "A4", "F#5"];
    let mut finetunes = Vec::new();

    for note_name in &notes {
        let instrument = TrackerInstrument {
            name: format!("{} Test", note_name),
            synthesis: Some(InstrumentSynthesis::Sine { base_note: None }),
            base_note: Some(note_name.to_string()),
            default_volume: Some(64),
            ..Default::default()
        };

        let (xm_instr, _) = generate_xm_instrument(&instrument, 42, 0, spec_dir).unwrap();
        finetunes.push(xm_instr.sample.finetune);
    }

    // All finetunes should be the same (from 22050 Hz sample rate)
    for (i, &ft) in finetunes.iter().enumerate() {
        assert_eq!(
            ft, finetunes[0],
            "Finetune for {} should match baseline (all at same sample rate)",
            notes[i]
        );
    }
}

/// Test: Verify default sample rate constant is 22050
#[test]
fn test_xm_default_sample_rate_constant() {
    use crate::note::DEFAULT_SAMPLE_RATE;
    assert_eq!(
        DEFAULT_SAMPLE_RATE, 22050,
        "Default sample rate should be 22050 Hz"
    );
}

/// Test: Verify default synth MIDI note constant is 60 (C4)
#[test]
fn test_xm_default_synth_midi_note_constant() {
    use crate::note::DEFAULT_SYNTH_MIDI_NOTE;
    assert_eq!(
        DEFAULT_SYNTH_MIDI_NOTE, 60,
        "Default synth MIDI note should be 60 (C4)"
    );
}

/// Test: Verify XM reference frequency constant is 8363
#[test]
fn test_xm_reference_frequency_constant() {
    use crate::note::XM_BASE_FREQ;
    assert_eq!(
        XM_BASE_FREQ, 8363.0,
        "XM reference frequency should be 8363 Hz"
    );
}
