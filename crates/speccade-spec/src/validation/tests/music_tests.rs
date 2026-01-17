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
                "channels": 4
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
