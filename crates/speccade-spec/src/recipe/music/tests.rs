//! Tests for music/tracker recipe types - basic serialization.

use super::*;
use crate::recipe::audio::{AudioLayer, AudioV1Params, Envelope, Synthesis, Waveform};
use std::collections::HashMap;

// ==================== Top-Level Keys Tests ====================

#[test]
fn test_song_name_serialization() {
    let params = MusicTrackerSongV1Params {
        format: TrackerFormat::Xm,
        bpm: 125,
        speed: 6,
        channels: 8,
        r#loop: false,
        ..Default::default()
    };

    let json = serde_json::to_string(&params).unwrap();
    let parsed: MusicTrackerSongV1Params = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.format, TrackerFormat::Xm);
}

#[test]
fn test_format_xm_serialization() {
    let format = TrackerFormat::Xm;
    let json = serde_json::to_string(&format).unwrap();
    assert_eq!(json, r#""xm""#);
    let parsed: TrackerFormat = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, TrackerFormat::Xm);
}

#[test]
fn test_format_it_serialization() {
    let format = TrackerFormat::It;
    let json = serde_json::to_string(&format).unwrap();
    assert_eq!(json, r#""it""#);
    let parsed: TrackerFormat = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, TrackerFormat::It);
}

#[test]
fn test_tracker_format_extension() {
    assert_eq!(TrackerFormat::Xm.extension(), "xm");
    assert_eq!(TrackerFormat::It.extension(), "it");
}

#[test]
fn test_bpm_serialization() {
    let json = r#"{"format":"xm","bpm":140,"speed":6,"channels":8,"loop":false,"instruments":[],"patterns":{},"arrangement":[]}"#;
    let parsed: MusicTrackerSongV1Params = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.bpm, 140);
}

#[test]
fn test_bpm_default_value() {
    let json = r#"{"format":"xm","speed":6,"channels":8,"loop":false,"instruments":[],"patterns":{},"arrangement":[]}"#;
    let result: Result<MusicTrackerSongV1Params, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_speed_serialization() {
    let json = r#"{"format":"xm","bpm":125,"speed":8,"channels":8,"loop":false,"instruments":[],"patterns":{},"arrangement":[]}"#;
    let parsed: MusicTrackerSongV1Params = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.speed, 8);
}

#[test]
fn test_channels_serialization() {
    let json = r#"{"format":"xm","bpm":125,"speed":6,"channels":16,"loop":false,"instruments":[],"patterns":{},"arrangement":[]}"#;
    let parsed: MusicTrackerSongV1Params = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.channels, 16);
}

#[test]
fn test_loop_serialization() {
    let json = r#"{"format":"xm","bpm":125,"speed":6,"channels":8,"loop":true,"instruments":[],"patterns":{},"arrangement":[]}"#;
    let parsed: MusicTrackerSongV1Params = serde_json::from_str(json).unwrap();
    assert!(parsed.r#loop);
}

#[test]
fn test_loop_default_value() {
    let json = r#"{"format":"xm","bpm":125,"speed":6,"channels":8,"instruments":[],"patterns":{},"arrangement":[]}"#;
    let parsed: MusicTrackerSongV1Params = serde_json::from_str(json).unwrap();
    assert!(!parsed.r#loop);
}

#[test]
fn test_instruments_serialization() {
    let instr = TrackerInstrument {
        name: "Lead".to_string(),
        synthesis: Some(InstrumentSynthesis::Sine),
        envelope: Envelope {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
        },
        default_volume: Some(64),
        ..Default::default()
    };

    let json = serde_json::to_string(&instr).unwrap();
    let parsed: TrackerInstrument = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "Lead");
}

#[test]
fn test_patterns_serialization() {
    let mut patterns = HashMap::new();
    patterns.insert(
        "intro".to_string(),
        TrackerPattern {
            rows: 64,
            ..Default::default()
        },
    );

    let json = serde_json::to_string(&patterns).unwrap();
    let parsed: HashMap<String, TrackerPattern> = serde_json::from_str(&json).unwrap();
    assert!(parsed.contains_key("intro"));
}

#[test]
fn test_arrangement_serialization() {
    let arrangement = vec![
        ArrangementEntry {
            pattern: "intro".to_string(),
            repeat: 2,
        },
        ArrangementEntry {
            pattern: "verse".to_string(),
            repeat: 1,
        },
    ];

    let json = serde_json::to_string(&arrangement).unwrap();
    let parsed: Vec<ArrangementEntry> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), 2);
}

#[test]
fn test_automation_serialization() {
    let automation = vec![
        AutomationEntry::VolumeFade {
            pattern: "intro".to_string(),
            channel: 0,
            start_row: 0,
            end_row: 63,
            start_vol: 0,
            end_vol: 64,
        },
        AutomationEntry::TempoChange {
            pattern: "chorus".to_string(),
            row: 0,
            bpm: 140,
        },
    ];

    let json = serde_json::to_string(&automation).unwrap();
    let parsed: Vec<AutomationEntry> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), 2);
}

#[test]
fn test_it_options_serialization() {
    let it_options = ItOptions {
        stereo: true,
        global_volume: 128,
        mix_volume: 48,
    };

    let json = serde_json::to_string(&it_options).unwrap();
    let parsed: ItOptions = serde_json::from_str(&json).unwrap();
    assert!(parsed.stereo);
    assert_eq!(parsed.global_volume, 128);
    assert_eq!(parsed.mix_volume, 48);
}

// ==================== Instrument Keys Tests ====================

#[test]
fn test_instrument_name_serialization() {
    let instr = TrackerInstrument {
        name: "Bass".to_string(),
        synthesis: Some(InstrumentSynthesis::Sawtooth),
        ..Default::default()
    };

    let json = serde_json::to_string(&instr).unwrap();
    assert!(json.contains("Bass"));
}

#[test]
fn test_instrument_ref_serialization() {
    let instr = TrackerInstrument {
        name: "External".to_string(),
        r#ref: Some("instruments/lead.spec.py".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&instr).unwrap();
    let parsed: TrackerInstrument = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.r#ref, Some("instruments/lead.spec.py".to_string()));
}

#[test]
fn test_instrument_synthesis_audio_v1_serialization() {
    let instr = TrackerInstrument {
        name: "Inline AudioV1".to_string(),
        synthesis_audio_v1: Some(AudioV1Params {
            base_note: Some(crate::recipe::audio::NoteSpec::NoteName("A4".to_string())),
            duration_seconds: 0.25,
            sample_rate: 44100,
            layers: vec![AudioLayer {
                synthesis: Synthesis::Oscillator {
                    waveform: Waveform::Sine,
                    frequency: 440.0,
                    freq_sweep: None,
                    detune: None,
                    duty: None,
                },
                envelope: Envelope::default(),
                volume: 1.0,
                pan: 0.0,
                delay: None,
            }],
            pitch_envelope: None,
            generate_loop_points: false,
            master_filter: None,
        }),
        ..Default::default()
    };

    let json = serde_json::to_string(&instr).unwrap();
    assert!(json.contains("synthesis_audio_v1"));

    let parsed: TrackerInstrument = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, instr.name);
    assert!(parsed.synthesis_audio_v1.is_some());
    assert!(parsed.synthesis.is_none());
}

#[test]
fn test_instrument_synthesis_pulse() {
    let pulse = InstrumentSynthesis::Pulse { duty_cycle: 0.5 };
    let json = serde_json::to_string(&pulse).unwrap();
    assert!(json.contains("pulse"));
    assert!(json.contains("duty_cycle"));

    let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, pulse);
}

#[test]
fn test_instrument_synthesis_triangle() {
    let triangle = InstrumentSynthesis::Triangle;
    let json = serde_json::to_string(&triangle).unwrap();
    assert!(json.contains("triangle"));

    let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, triangle);
}

#[test]
fn test_instrument_synthesis_sawtooth() {
    let sawtooth = InstrumentSynthesis::Sawtooth;
    let json = serde_json::to_string(&sawtooth).unwrap();
    assert!(json.contains("sawtooth"));

    let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, sawtooth);
}

#[test]
fn test_instrument_synthesis_sine() {
    let sine = InstrumentSynthesis::Sine;
    let json = serde_json::to_string(&sine).unwrap();
    assert!(json.contains("sine"));

    let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, sine);
}

#[test]
fn test_instrument_synthesis_noise() {
    let noise = InstrumentSynthesis::Noise { periodic: true };
    let json = serde_json::to_string(&noise).unwrap();
    assert!(json.contains("noise"));

    let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, noise);
}

#[test]
fn test_instrument_synthesis_sample() {
    let sample = InstrumentSynthesis::Sample {
        path: "samples/kick.wav".to_string(),
        base_note: Some("C4".to_string()),
    };
    let json = serde_json::to_string(&sample).unwrap();
    assert!(json.contains("sample"));
    assert!(json.contains("samples/kick.wav"));

    let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, sample);
}

#[test]
fn test_instrument_envelope_serialization() {
    let envelope = Envelope {
        attack: 0.05,
        decay: 0.15,
        sustain: 0.6,
        release: 0.3,
    };

    let json = serde_json::to_string(&envelope).unwrap();
    let parsed: Envelope = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.attack, 0.05);
    assert_eq!(parsed.decay, 0.15);
    assert_eq!(parsed.sustain, 0.6);
    assert_eq!(parsed.release, 0.3);
}

#[test]
fn test_instrument_envelope_default() {
    let envelope = instrument::default_envelope();
    assert_eq!(envelope.attack, 0.01);
    assert_eq!(envelope.decay, 0.1);
    assert_eq!(envelope.sustain, 0.7);
    assert_eq!(envelope.release, 0.2);
}

// ==================== Pattern Keys Tests ====================

#[test]
fn test_pattern_rows_serialization() {
    let pattern = TrackerPattern {
        rows: 128,
        ..Default::default()
    };

    let json = serde_json::to_string(&pattern).unwrap();
    let parsed: TrackerPattern = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.rows, 128);
}

#[test]
fn test_pattern_notes_serialization() {
    let note = PatternNote {
        row: 0,
        channel: Some(0),
        note: "C4".to_string(),
        inst: 1,
        vol: Some(64),
        ..Default::default()
    };

    let pattern = TrackerPattern {
        rows: 64,
        data: Some(vec![note]),
        notes: None,
    };

    let json = serde_json::to_string(&pattern).unwrap();
    let parsed: TrackerPattern = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.data.as_ref().map(|d| d.len()).unwrap_or(0), 1);
}

// ==================== Note Keys Tests ====================

#[test]
fn test_note_row_serialization() {
    let note = PatternNote {
        row: 16,
        note: "C4".to_string(),
        ..Default::default()
    };

    let json = serde_json::to_string(&note).unwrap();
    let parsed: PatternNote = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.row, 16);
}

#[test]
fn test_note_channel_serialization() {
    let note = PatternNote {
        channel: Some(3),
        note: "E4".to_string(),
        ..Default::default()
    };

    let json = serde_json::to_string(&note).unwrap();
    let parsed: PatternNote = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.channel, Some(3));
}

#[test]
fn test_note_note_serialization() {
    let note = PatternNote {
        note: "A#5".to_string(),
        ..Default::default()
    };

    let json = serde_json::to_string(&note).unwrap();
    let parsed: PatternNote = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.note, "A#5");
}

#[test]
fn test_note_instrument_serialization() {
    let note = PatternNote {
        note: "C4".to_string(),
        inst: 5,
        ..Default::default()
    };

    let json = serde_json::to_string(&note).unwrap();
    let parsed: PatternNote = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.inst, 5);
}

#[test]
fn test_note_volume_serialization() {
    let note = PatternNote {
        note: "C4".to_string(),
        vol: Some(48),
        ..Default::default()
    };

    let json = serde_json::to_string(&note).unwrap();
    let parsed: PatternNote = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.vol, Some(48));
}

#[test]
fn test_note_effect_serialization() {
    let note = PatternNote {
        note: "C4".to_string(),
        effect_name: Some("vibrato".to_string()),
        param: Some(0x44),
        ..Default::default()
    };

    let json = serde_json::to_string(&note).unwrap();
    let parsed: PatternNote = serde_json::from_str(&json).unwrap();
    assert!(parsed.effect_name.is_some());
}

#[test]
fn test_note_effect_param_serialization() {
    let note = PatternNote {
        note: "C4".to_string(),
        effect_name: Some("arpeggio".to_string()),
        param: Some(0x37),
        ..Default::default()
    };

    let json = serde_json::to_string(&note).unwrap();
    let parsed: PatternNote = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.param, Some(0x37));
}

#[test]
fn test_note_effect_xy_serialization() {
    let note = PatternNote {
        note: "C4".to_string(),
        effect_name: Some("portamento".to_string()),
        effect_xy: Some([0x3, 0x7]),
        ..Default::default()
    };

    let json = serde_json::to_string(&note).unwrap();
    let parsed: PatternNote = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.effect_xy, Some([0x3, 0x7]));
}
