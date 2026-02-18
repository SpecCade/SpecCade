use super::helpers::note_to_midi;
use super::*;
use crate::report::{AssetType, Severity};
use crate::rules::{AssetData, LintRule};
use speccade_spec::recipe::music::{
    ArrangementEntry, MusicTrackerSongV1Params, PatternNote, TrackerPattern,
};
use speccade_spec::Spec;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Helper to create test asset data.
fn test_asset_data() -> AssetData<'static> {
    AssetData {
        path: Path::new("test.xm"),
        bytes: &[],
    }
}

/// Helper to create a basic spec with music params.
fn make_music_spec(params: MusicTrackerSongV1Params) -> Spec {
    use speccade_spec::recipe::Recipe;
    use speccade_spec::spec::AssetType;

    Spec {
        spec_version: 1,
        asset_id: "test-music".to_string(),
        asset_type: AssetType::Music,
        license: "CC0-1.0".to_string(),
        seed: 42,
        outputs: vec![],
        description: None,
        style_tags: None,
        engine_targets: None,
        migration_notes: None,
        variants: None,
        recipe: Some(Recipe::new(
            "music.tracker_song_v1",
            serde_json::to_value(params).unwrap(),
        )),
    }
}

/// Helper to create a pattern with notes.
fn make_pattern(rows: u16, notes: Vec<(u8, u16, &str)>) -> TrackerPattern {
    let mut notes_map: HashMap<String, Vec<PatternNote>> = HashMap::new();

    for (channel, row, note) in notes {
        notes_map
            .entry(channel.to_string())
            .or_default()
            .push(PatternNote {
                row,
                channel: None,
                note: note.to_string(),
                inst: 1,
                vol: None,
                effect: None,
                param: None,
                effect_name: None,
                effect_xy: None,
            });
    }

    TrackerPattern {
        rows,
        notes: Some(notes_map),
        data: None,
    }
}

// -------------------------------------------------------------------------
// note_to_midi tests
// -------------------------------------------------------------------------

#[test]
fn test_note_to_midi_basic() {
    assert_eq!(note_to_midi("C4"), Some(60));
    assert_eq!(note_to_midi("A4"), Some(69));
    assert_eq!(note_to_midi("C0"), Some(12));
    assert_eq!(note_to_midi("C-1"), Some(0));
}

#[test]
fn test_note_to_midi_sharps_flats() {
    assert_eq!(note_to_midi("C#4"), Some(61)); // C4=60 + 1 = 61
    assert_eq!(note_to_midi("Db4"), Some(61)); // D4=62 - 1 = 61 (enharmonic with C#4)
    assert_eq!(note_to_midi("F#3"), Some(54)); // F3=53 + 1 = 54
}

#[test]
fn test_note_to_midi_special() {
    assert_eq!(note_to_midi("---"), None);
    assert_eq!(note_to_midi("..."), None);
    assert_eq!(note_to_midi("==="), None);
    assert_eq!(note_to_midi(""), None);
}

#[test]
fn test_note_to_midi_numeric() {
    assert_eq!(note_to_midi("60"), Some(60));
    assert_eq!(note_to_midi("0"), Some(0));
    assert_eq!(note_to_midi("127"), Some(127));
}

// -------------------------------------------------------------------------
// EmptyPatternRule tests
// -------------------------------------------------------------------------

#[test]
fn test_empty_pattern_rule_detects_empty() {
    let rule = EmptyPatternRule;
    let asset = test_asset_data();

    let mut patterns = HashMap::new();
    patterns.insert(
        "intro".to_string(),
        TrackerPattern {
            rows: 64,
            notes: Some(HashMap::new()),
            data: None,
        },
    );

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns,
        arrangement: vec![ArrangementEntry {
            pattern: "intro".to_string(),
            repeat: 1,
        }],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "music/empty-pattern");
}

#[test]
fn test_empty_pattern_rule_passes_with_notes() {
    let rule = EmptyPatternRule;
    let asset = test_asset_data();

    let mut patterns = HashMap::new();
    patterns.insert(
        "intro".to_string(),
        make_pattern(64, vec![(0, 0, "C4"), (0, 16, "E4")]),
    );

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns,
        arrangement: vec![ArrangementEntry {
            pattern: "intro".to_string(),
            repeat: 1,
        }],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert!(issues.is_empty());
}

// -------------------------------------------------------------------------
// InvalidNoteRule tests
// -------------------------------------------------------------------------

#[test]
fn test_invalid_note_rule_detects_out_of_range() {
    let rule = InvalidNoteRule;
    let asset = test_asset_data();

    let mut patterns = HashMap::new();
    patterns.insert(
        "test".to_string(),
        make_pattern(
            64,
            vec![
                (0, 0, "C12"), // Invalid: too high (C12 = 156)
            ],
        ),
    );

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns,
        arrangement: vec![],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "music/invalid-note");
}

#[test]
fn test_invalid_note_rule_passes_valid_notes() {
    let rule = InvalidNoteRule;
    let asset = test_asset_data();

    let mut patterns = HashMap::new();
    patterns.insert(
        "test".to_string(),
        make_pattern(
            64,
            vec![
                (0, 0, "C4"),
                (1, 0, "G9"),  // G9 = 127 (highest valid)
                (2, 0, "C-1"), // C-1 = 0 (lowest valid)
            ],
        ),
    );

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns,
        arrangement: vec![],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert!(issues.is_empty());
}

// -------------------------------------------------------------------------
// EmptyArrangementRule tests
// -------------------------------------------------------------------------

#[test]
fn test_empty_arrangement_rule_detects_empty() {
    let rule = EmptyArrangementRule;
    let asset = test_asset_data();

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns: HashMap::new(),
        arrangement: vec![],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "music/empty-arrangement");
}

#[test]
fn test_empty_arrangement_rule_passes_with_entries() {
    let rule = EmptyArrangementRule;
    let asset = test_asset_data();

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns: HashMap::new(),
        arrangement: vec![ArrangementEntry {
            pattern: "intro".to_string(),
            repeat: 1,
        }],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert!(issues.is_empty());
}

// -------------------------------------------------------------------------
// ExtremeTempoRule tests
// -------------------------------------------------------------------------

#[test]
fn test_extreme_tempo_rule_detects_too_slow() {
    let rule = ExtremeTempoRule;
    let asset = test_asset_data();

    let params = MusicTrackerSongV1Params {
        bpm: 30, // Too slow
        speed: 6,
        channels: 4,
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "music/extreme-tempo");
}

#[test]
fn test_extreme_tempo_rule_detects_too_fast() {
    let rule = ExtremeTempoRule;
    let asset = test_asset_data();

    let params = MusicTrackerSongV1Params {
        bpm: 350, // Too fast
        speed: 6,
        channels: 4,
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "music/extreme-tempo");
}

#[test]
fn test_extreme_tempo_rule_passes_normal() {
    let rule = ExtremeTempoRule;
    let asset = test_asset_data();

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert!(issues.is_empty());
}

// -------------------------------------------------------------------------
// DensePatternRule tests
// -------------------------------------------------------------------------

#[test]
fn test_dense_pattern_rule_detects_too_many_notes() {
    let rule = DensePatternRule;
    let asset = test_asset_data();

    let mut patterns = HashMap::new();
    // 10 simultaneous notes on row 0
    patterns.insert(
        "dense".to_string(),
        make_pattern(
            64,
            vec![
                (0, 0, "C4"),
                (1, 0, "D4"),
                (2, 0, "E4"),
                (3, 0, "F4"),
                (4, 0, "G4"),
                (5, 0, "A4"),
                (6, 0, "B4"),
                (7, 0, "C5"),
                (8, 0, "D5"),
                (9, 0, "E5"),
            ],
        ),
    );

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 16,
        patterns,
        arrangement: vec![],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "music/dense-pattern");
}

#[test]
fn test_dense_pattern_rule_passes_normal() {
    let rule = DensePatternRule;
    let asset = test_asset_data();

    let mut patterns = HashMap::new();
    patterns.insert(
        "normal".to_string(),
        make_pattern(
            64,
            vec![(0, 0, "C4"), (1, 0, "E4"), (2, 0, "G4"), (3, 0, "C5")],
        ),
    );

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns,
        arrangement: vec![],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert!(issues.is_empty());
}

// -------------------------------------------------------------------------
// SparsePatternRule tests
// -------------------------------------------------------------------------

#[test]
fn test_sparse_pattern_rule_detects_sparse() {
    let rule = SparsePatternRule;
    let asset = test_asset_data();

    let mut patterns = HashMap::new();
    // 64 rows * 8 channels = 512 cells, only 2 notes = 0.4%
    patterns.insert(
        "sparse".to_string(),
        make_pattern(64, vec![(0, 0, "C4"), (0, 32, "E4")]),
    );

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 8,
        patterns,
        arrangement: vec![],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "music/sparse-pattern");
}

// -------------------------------------------------------------------------
// NoVariationRule tests
// -------------------------------------------------------------------------

#[test]
fn test_no_variation_rule_detects_repetition() {
    let rule = NoVariationRule;
    let asset = test_asset_data();

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns: HashMap::new(),
        arrangement: vec![ArrangementEntry {
            pattern: "A".to_string(),
            repeat: 8,
        }],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "music/no-variation");
}

#[test]
fn test_no_variation_rule_passes_with_variation() {
    let rule = NoVariationRule;
    let asset = test_asset_data();

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns: HashMap::new(),
        arrangement: vec![
            ArrangementEntry {
                pattern: "A".to_string(),
                repeat: 2,
            },
            ArrangementEntry {
                pattern: "B".to_string(),
                repeat: 2,
            },
            ArrangementEntry {
                pattern: "A".to_string(),
                repeat: 2,
            },
        ],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert!(issues.is_empty());
}

// -------------------------------------------------------------------------
// UnusedChannelRule tests
// -------------------------------------------------------------------------

#[test]
fn test_unused_channel_rule_detects_unused() {
    let rule = UnusedChannelRule;
    let asset = test_asset_data();

    let mut patterns = HashMap::new();
    // Only channels 0 and 1 used, but 4 channels declared
    patterns.insert(
        "test".to_string(),
        make_pattern(64, vec![(0, 0, "C4"), (1, 0, "E4")]),
    );

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns,
        arrangement: vec![],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    // Channels 2 and 3 are unused
    assert_eq!(issues.len(), 2);
    assert!(issues.iter().all(|i| i.rule_id == "music/unused-channel"));
}

// -------------------------------------------------------------------------
// UnresolvedTensionRule tests
// -------------------------------------------------------------------------

#[test]
fn test_unresolved_tension_rule_detects_tritone() {
    let rule = UnresolvedTensionRule;
    let asset = test_asset_data();

    let mut patterns = HashMap::new();
    // Ends with C4 and F#4 (tritone = 6 semitones)
    patterns.insert(
        "ending".to_string(),
        make_pattern(64, vec![(0, 63, "C4"), (1, 63, "F#4")]),
    );

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns,
        arrangement: vec![ArrangementEntry {
            pattern: "ending".to_string(),
            repeat: 1,
        }],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "music/unresolved-tension");
    assert!(issues[0].message.contains("tritone"));
}

#[test]
fn test_unresolved_tension_rule_passes_consonant() {
    let rule = UnresolvedTensionRule;
    let asset = test_asset_data();

    let mut patterns = HashMap::new();
    // Ends with C major chord (consonant)
    patterns.insert(
        "ending".to_string(),
        make_pattern(64, vec![(0, 63, "C4"), (1, 63, "E4"), (2, 63, "G4")]),
    );

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns,
        arrangement: vec![ArrangementEntry {
            pattern: "ending".to_string(),
            repeat: 1,
        }],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert!(issues.is_empty());
}

// -------------------------------------------------------------------------
// VoiceCrossingRule tests
// -------------------------------------------------------------------------

#[test]
fn test_voice_crossing_rule_detects_crossing() {
    let rule = VoiceCrossingRule;
    let asset = test_asset_data();

    let mut patterns = HashMap::new();
    // Channel 0 (soprano) at C3, Channel 1 (alto) at C5 - crossing!
    patterns.insert(
        "test".to_string(),
        make_pattern(
            64,
            vec![
                (0, 0, "C3"), // Soprano too low
                (1, 0, "C5"), // Alto too high
            ],
        ),
    );

    let params = MusicTrackerSongV1Params {
        bpm: 120,
        speed: 6,
        channels: 4,
        patterns,
        arrangement: vec![],
        ..Default::default()
    };

    let spec = make_music_spec(params);
    let issues = rule.check(&asset, Some(&spec));

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "music/voice-crossing");
}

// -------------------------------------------------------------------------
// all_rules test
// -------------------------------------------------------------------------

#[test]
fn test_all_rules_returns_12_rules() {
    let rules = all_rules();
    assert_eq!(rules.len(), 12);

    // Verify each rule has the correct prefix
    for rule in &rules {
        assert!(rule.id().starts_with("music/"));
        assert!(rule.applies_to().contains(&AssetType::Music));
    }
}

#[test]
fn test_all_rules_unique_ids() {
    let rules = all_rules();
    let ids: HashSet<_> = rules.iter().map(|r| r.id()).collect();
    assert_eq!(ids.len(), rules.len(), "All rule IDs should be unique");
}

#[test]
fn test_severity_distribution() {
    let rules = all_rules();

    let errors = rules
        .iter()
        .filter(|r| r.default_severity() == Severity::Error)
        .count();
    let warnings = rules
        .iter()
        .filter(|r| r.default_severity() == Severity::Warning)
        .count();
    let infos = rules
        .iter()
        .filter(|r| r.default_severity() == Severity::Info)
        .count();

    assert_eq!(errors, 3, "Expected 3 error-level rules");
    assert_eq!(warnings, 6, "Expected 6 warning-level rules");
    assert_eq!(infos, 3, "Expected 3 info-level rules");
}
