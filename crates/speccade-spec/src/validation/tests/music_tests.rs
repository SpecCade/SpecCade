//! Music validation tests.

use crate::output::{OutputFormat, OutputSpec};
use crate::spec::AssetType;
use crate::validation::*;

#[test]
fn test_music_requires_primary_format_matches_recipe_format() {
    let spec = crate::spec::Spec::builder("test-song-01", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "it",
                "bpm": 120,
                "speed": 6,
                "channels": 4
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::OutputValidationFailed));
}

#[test]
fn test_music_compose_requires_primary_format_matches_recipe_format() {
    let spec = crate::spec::Spec::builder("test-song-compose-01", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_compose_v1",
            serde_json::json!({
                "format": "it",
                "bpm": 120,
                "speed": 6,
                "channels": 4,
                "patterns": {}
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::OutputValidationFailed));
}

#[test]
fn test_music_allows_dual_xm_it_primary_outputs() {
    let spec = crate::spec::Spec::builder("test-song-02", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .output(OutputSpec::primary(OutputFormat::It, "songs/test.it"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 120,
                "speed": 6,
                "channels": 4,
                "patterns": {
                    "intro": { "rows": 4 }
                },
                "arrangement": [
                    { "pattern": "intro", "repeat": 1 }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_music_instrument_requires_exactly_one_source() {
    let missing_source = crate::spec::Spec::builder("test-song-04", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 120,
                "speed": 6,
                "channels": 4,
                "instruments": [
                    { "name": "bad" }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&missing_source);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams));

    let multiple_sources = crate::spec::Spec::builder("test-song-05", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 120,
                "speed": 6,
                "channels": 4,
                "instruments": [
                    { "name": "bad", "synthesis": { "type": "sine" }, "wav": "x.wav" }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&multiple_sources);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams));
}

#[test]
fn test_music_dual_outputs_rejects_duplicate_primary_format() {
    let spec = crate::spec::Spec::builder("test-song-03", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test1.xm"))
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test2.xm"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 120,
                "speed": 6,
                "channels": 4
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::OutputValidationFailed));
}

#[test]
fn test_music_semantics_reject_invalid_bpm_and_channels() {
    let spec = crate::spec::Spec::builder("test-song-06", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 300,
                "speed": 6,
                "channels": 40
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("bpm must be 32-255")));
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("channels must be 1-32")));
}

#[test]
fn test_music_semantics_reject_invalid_pattern_note_and_effect() {
    let spec = crate::spec::Spec::builder("test-song-07", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 120,
                "speed": 6,
                "channels": 1,
                "instruments": [{ "name": "lead", "synthesis": { "type": "sine" } }],
                "patterns": {
                    "intro": {
                        "rows": 4,
                        "data": [
                            { "row": 0, "channel": 3, "note": "C4", "inst": 0, "effect_name": "does_not_exist" }
                        ]
                    }
                }
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("exceeds configured channels")));
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("unknown effect_name")));
}

#[test]
fn test_music_semantics_reject_invalid_arrangement_and_automation() {
    let spec = crate::spec::Spec::builder("test-song-08", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 120,
                "speed": 6,
                "channels": 1,
                "instruments": [{ "name": "lead", "synthesis": { "type": "sine" } }],
                "patterns": {
                    "intro": { "rows": 4 }
                },
                "arrangement": [
                    { "pattern": "missing", "repeat": 1 }
                ],
                "automation": [
                    { "type": "tempo_change", "pattern": "intro", "row": 10, "bpm": 20 }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("arrangement references unknown pattern")));
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("tempo_change row")));
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("tempo_change bpm must be 32-255")));
}

#[test]
fn test_music_semantics_reject_it_loop_restart_position_above_255() {
    let spec = crate::spec::Spec::builder("test-song-09", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::It, "songs/test.it"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "it",
                "bpm": 120,
                "speed": 6,
                "channels": 4,
                "loop": true,
                "restart_position": 300,
                "patterns": {
                    "intro": { "rows": 4 }
                },
                "arrangement": [
                    { "pattern": "intro", "repeat": 1 }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("IT restart_position must be 0-255")));
}

#[test]
fn test_music_semantics_reject_empty_arrangement() {
    let spec = crate::spec::Spec::builder("test-song-10", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 120,
                "speed": 6,
                "channels": 4,
                "patterns": {
                    "intro": { "rows": 4 }
                }
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("arrangement must contain at least one entry")));
}

#[test]
fn test_music_compose_semantics_reject_empty_arrangement() {
    let spec = crate::spec::Spec::builder("test-song-compose-10", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .recipe(crate::recipe::Recipe::new(
            "music.tracker_song_compose_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 120,
                "speed": 6,
                "channels": 4,
                "patterns": {
                    "p0": {
                        "rows": 4,
                        "program": { "op": "stack", "parts": [] }
                    }
                }
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("arrangement must contain at least one entry")));
}
