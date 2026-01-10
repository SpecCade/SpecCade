//! Tests for music/tracker recipe types - automation, IT options, and integration.

use super::*;
use crate::recipe::audio_sfx::Envelope;
use std::collections::HashMap;

// ==================== Automation Keys Tests ====================

#[test]
fn test_automation_volume_fade_type() {
    let auto = AutomationEntry::VolumeFade {
        pattern: "intro".to_string(),
        channel: 0,
        start_row: 0,
        end_row: 63,
        start_vol: 0,
        end_vol: 64,
    };

    let json = serde_json::to_string(&auto).unwrap();
    assert!(json.contains("volume_fade"));
}

#[test]
fn test_automation_tempo_change_type() {
    let auto = AutomationEntry::TempoChange {
        pattern: "chorus".to_string(),
        row: 0,
        bpm: 140,
    };

    let json = serde_json::to_string(&auto).unwrap();
    assert!(json.contains("tempo_change"));
}

#[test]
fn test_automation_pattern_serialization() {
    let auto = AutomationEntry::VolumeFade {
        pattern: "verse".to_string(),
        channel: 0,
        start_row: 0,
        end_row: 63,
        start_vol: 64,
        end_vol: 0,
    };

    let json = serde_json::to_string(&auto).unwrap();
    assert!(json.contains("verse"));
}

#[test]
fn test_automation_channel_serialization() {
    let auto = AutomationEntry::VolumeFade {
        pattern: "intro".to_string(),
        channel: 2,
        start_row: 0,
        end_row: 63,
        start_vol: 0,
        end_vol: 64,
    };

    let json = serde_json::to_string(&auto).unwrap();
    let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
    match parsed {
        AutomationEntry::VolumeFade { channel, .. } => assert_eq!(channel, 2),
        _ => panic!("Wrong automation type"),
    }
}

#[test]
fn test_automation_start_row_serialization() {
    let auto = AutomationEntry::VolumeFade {
        pattern: "intro".to_string(),
        channel: 0,
        start_row: 16,
        end_row: 63,
        start_vol: 0,
        end_vol: 64,
    };

    let json = serde_json::to_string(&auto).unwrap();
    let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
    match parsed {
        AutomationEntry::VolumeFade { start_row, .. } => assert_eq!(start_row, 16),
        _ => panic!("Wrong automation type"),
    }
}

#[test]
fn test_automation_end_row_serialization() {
    let auto = AutomationEntry::VolumeFade {
        pattern: "intro".to_string(),
        channel: 0,
        start_row: 0,
        end_row: 48,
        start_vol: 0,
        end_vol: 64,
    };

    let json = serde_json::to_string(&auto).unwrap();
    let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
    match parsed {
        AutomationEntry::VolumeFade { end_row, .. } => assert_eq!(end_row, 48),
        _ => panic!("Wrong automation type"),
    }
}

#[test]
fn test_automation_start_vol_serialization() {
    let auto = AutomationEntry::VolumeFade {
        pattern: "intro".to_string(),
        channel: 0,
        start_row: 0,
        end_row: 63,
        start_vol: 32,
        end_vol: 64,
    };

    let json = serde_json::to_string(&auto).unwrap();
    let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
    match parsed {
        AutomationEntry::VolumeFade { start_vol, .. } => assert_eq!(start_vol, 32),
        _ => panic!("Wrong automation type"),
    }
}

#[test]
fn test_automation_end_vol_serialization() {
    let auto = AutomationEntry::VolumeFade {
        pattern: "intro".to_string(),
        channel: 0,
        start_row: 0,
        end_row: 63,
        start_vol: 0,
        end_vol: 48,
    };

    let json = serde_json::to_string(&auto).unwrap();
    let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
    match parsed {
        AutomationEntry::VolumeFade { end_vol, .. } => assert_eq!(end_vol, 48),
        _ => panic!("Wrong automation type"),
    }
}

#[test]
fn test_automation_tempo_row_serialization() {
    let auto = AutomationEntry::TempoChange {
        pattern: "bridge".to_string(),
        row: 32,
        bpm: 150,
    };

    let json = serde_json::to_string(&auto).unwrap();
    let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
    match parsed {
        AutomationEntry::TempoChange { row, .. } => assert_eq!(row, 32),
        _ => panic!("Wrong automation type"),
    }
}

#[test]
fn test_automation_tempo_bpm_serialization() {
    let auto = AutomationEntry::TempoChange {
        pattern: "outro".to_string(),
        row: 0,
        bpm: 180,
    };

    let json = serde_json::to_string(&auto).unwrap();
    let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
    match parsed {
        AutomationEntry::TempoChange { bpm, .. } => assert_eq!(bpm, 180),
        _ => panic!("Wrong automation type"),
    }
}

// ==================== IT Options Keys Tests ====================

#[test]
fn test_it_options_stereo_serialization() {
    let opts = ItOptions {
        stereo: false,
        global_volume: 128,
        mix_volume: 48,
    };

    let json = serde_json::to_string(&opts).unwrap();
    let parsed: ItOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.stereo, false);
}

#[test]
fn test_it_options_stereo_default() {
    let opts = ItOptions::default();
    assert_eq!(opts.stereo, true);
}

#[test]
fn test_it_options_global_volume_serialization() {
    let opts = ItOptions {
        stereo: true,
        global_volume: 96,
        mix_volume: 48,
    };

    let json = serde_json::to_string(&opts).unwrap();
    let parsed: ItOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.global_volume, 96);
}

#[test]
fn test_it_options_global_volume_default() {
    let opts = ItOptions::default();
    assert_eq!(opts.global_volume, 128);
}

#[test]
fn test_it_options_mix_volume_serialization() {
    let opts = ItOptions {
        stereo: true,
        global_volume: 128,
        mix_volume: 64,
    };

    let json = serde_json::to_string(&opts).unwrap();
    let parsed: ItOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.mix_volume, 64);
}

#[test]
fn test_it_options_mix_volume_default() {
    let opts = ItOptions::default();
    assert_eq!(opts.mix_volume, 48);
}

// ==================== Arrangement Entry Tests ====================

#[test]
fn test_arrangement_pattern_serialization() {
    let entry = ArrangementEntry {
        pattern: "chorus".to_string(),
        repeat: 4,
    };

    let json = serde_json::to_string(&entry).unwrap();
    let parsed: ArrangementEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.pattern, "chorus");
}

#[test]
fn test_arrangement_repeat_serialization() {
    let entry = ArrangementEntry {
        pattern: "verse".to_string(),
        repeat: 3,
    };

    let json = serde_json::to_string(&entry).unwrap();
    let parsed: ArrangementEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.repeat, 3);
}

#[test]
fn test_arrangement_default_repeat() {
    let json = r#"{"pattern": "intro"}"#;
    let entry: ArrangementEntry = serde_json::from_str(json).unwrap();
    assert_eq!(entry.pattern, "intro");
    assert_eq!(entry.repeat, 1);
}

// ==================== Full Integration Tests ====================

#[test]
fn test_full_song_serialization_roundtrip() {
    let envelope = Envelope {
        attack: 0.01,
        decay: 0.1,
        sustain: 0.7,
        release: 0.2,
    };

    let instrument = TrackerInstrument {
        name: "Lead".to_string(),
        r#ref: None,
        synthesis: Some(InstrumentSynthesis::Pulse { duty_cycle: 0.5 }),
        envelope: envelope.clone(),
        default_volume: Some(64),
    };

    let pattern = TrackerPattern {
        rows: 64,
        data: vec![PatternNote {
            row: 0,
            channel: 0,
            note: "C4".to_string(),
            instrument: 0,
            volume: Some(64),
            effect: Some(PatternEffect {
                r#type: Some("vibrato".to_string()),
                param: Some(0x44),
                effect_xy: None,
            }),
        }],
    };

    let mut patterns = HashMap::new();
    patterns.insert("intro".to_string(), pattern);

    let params = MusicTrackerSongV1Params {
        format: TrackerFormat::Xm,
        bpm: 125,
        speed: 6,
        channels: 8,
        r#loop: true,
        instruments: vec![instrument],
        patterns,
        arrangement: vec![ArrangementEntry {
            pattern: "intro".to_string(),
            repeat: 2,
        }],
        automation: vec![AutomationEntry::VolumeFade {
            pattern: "intro".to_string(),
            channel: 0,
            start_row: 0,
            end_row: 63,
            start_vol: 0,
            end_vol: 64,
        }],
        it_options: None,
    };

    let json = serde_json::to_string_pretty(&params).unwrap();
    let parsed: MusicTrackerSongV1Params = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.format, TrackerFormat::Xm);
    assert_eq!(parsed.bpm, 125);
    assert_eq!(parsed.speed, 6);
    assert_eq!(parsed.channels, 8);
    assert_eq!(parsed.r#loop, true);
    assert_eq!(parsed.instruments.len(), 1);
    assert_eq!(parsed.patterns.len(), 1);
    assert_eq!(parsed.arrangement.len(), 1);
    assert_eq!(parsed.automation.len(), 1);
}

#[test]
fn test_full_it_song_with_it_options() {
    let params = MusicTrackerSongV1Params {
        format: TrackerFormat::It,
        bpm: 140,
        speed: 8,
        channels: 16,
        r#loop: false,
        instruments: vec![],
        patterns: HashMap::new(),
        arrangement: vec![],
        automation: vec![],
        it_options: Some(ItOptions {
            stereo: false,
            global_volume: 96,
            mix_volume: 64,
        }),
    };

    let json = serde_json::to_string(&params).unwrap();
    let parsed: MusicTrackerSongV1Params = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.format, TrackerFormat::It);
    assert!(parsed.it_options.is_some());
    let opts = parsed.it_options.unwrap();
    assert_eq!(opts.stereo, false);
    assert_eq!(opts.global_volume, 96);
    assert_eq!(opts.mix_volume, 64);
}
