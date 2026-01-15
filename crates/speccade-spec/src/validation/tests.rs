//! Validation tests.

use super::*;
use crate::output::{OutputFormat, OutputSpec};
use crate::recipe::Recipe;
use crate::spec::AssetType;

fn make_valid_spec() -> Spec {
    Spec::builder("test-asset-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .description("Test asset")
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .build()
}

#[test]
fn test_valid_spec() {
    let spec = make_valid_spec();
    let result = validate_spec(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_invalid_spec_version() {
    let mut spec = make_valid_spec();
    spec.spec_version = 2;
    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::UnsupportedSpecVersion));
}

#[test]
fn test_invalid_asset_id() {
    let test_cases = vec![
        ("1invalid", "starts with number"),
        ("ab", "too short"),
        ("UPPERCASE", "uppercase"),
        ("has spaces", "spaces"),
        ("a", "single char"),
    ];

    for (asset_id, desc) in test_cases {
        let mut spec = make_valid_spec();
        spec.asset_id = asset_id.to_string();
        let result = validate_spec(&spec);
        assert!(
            !result.is_ok(),
            "expected invalid for {}: {}",
            desc,
            asset_id
        );
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.code == ErrorCode::InvalidAssetId),
            "expected InvalidAssetId for {}: {}",
            desc,
            asset_id
        );
    }
}

#[test]
fn test_valid_asset_ids() {
    let valid_ids = vec![
        "abc",
        "test-asset-01",
        "laser_blast_01",
        "a23",
        "my-cool-asset-name-with-dashes",
    ];

    for asset_id in valid_ids {
        assert!(is_valid_asset_id(asset_id), "expected valid: {}", asset_id);
    }
}

#[test]
fn test_no_outputs() {
    let mut spec = make_valid_spec();
    spec.outputs.clear();
    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e.code == ErrorCode::NoOutputs));
}

#[test]
fn test_no_primary_output() {
    let mut spec = make_valid_spec();
    spec.outputs = vec![OutputSpec::metadata("meta.json")];
    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::NoPrimaryOutput));
}

#[test]
fn test_metadata_and_preview_outputs_are_reserved() {
    let mut spec = make_valid_spec();
    spec.outputs
        .push(OutputSpec::metadata("sounds/test.meta.json"));
    spec.outputs.push(OutputSpec::preview(
        OutputFormat::Wav,
        "sounds/test.preview.wav",
    ));

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::OutputValidationFailed));
}

#[test]
fn test_duplicate_output_path() {
    let mut spec = make_valid_spec();
    spec.outputs
        .push(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"));
    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::DuplicateOutputPath));
}

#[test]
fn test_unsafe_output_paths() {
    let unsafe_paths = vec![
        ("/absolute/path.wav", "absolute path"),
        ("C:/windows/path.wav", "drive letter"),
        ("path\\with\\backslash.wav", "backslash"),
        ("../parent/path.wav", "parent traversal"),
        ("sounds/../../../etc/passwd", "deep traversal"),
    ];

    for (path, desc) in unsafe_paths {
        let mut spec = make_valid_spec();
        spec.outputs = vec![OutputSpec::primary(OutputFormat::Wav, path)];
        let result = validate_spec(&spec);
        assert!(!result.is_ok(), "expected unsafe for {}: {}", desc, path);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.code == ErrorCode::UnsafeOutputPath),
            "expected UnsafeOutputPath for {}: {}",
            desc,
            path
        );
    }
}

#[test]
fn test_path_format_mismatch() {
    let mut spec = make_valid_spec();
    spec.outputs = vec![OutputSpec::primary(OutputFormat::Wav, "sounds/test.png")];
    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::PathFormatMismatch));
}

#[test]
fn test_recipe_asset_type_mismatch() {
    let mut spec = make_valid_spec();
    spec.recipe = Some(crate::recipe::Recipe::new(
        "music.tracker_song_v1",
        serde_json::json!({}),
    ));
    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::RecipeAssetTypeMismatch));
}

#[test]
fn test_missing_recipe_for_generate() {
    let spec = make_valid_spec();
    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::MissingRecipe));
}

#[test]
fn test_unsupported_recipe_kind_for_generate() {
    let mut spec = make_valid_spec();
    spec.recipe = Some(crate::recipe::Recipe::new(
        "audio_v999",
        serde_json::json!({
            "duration_seconds": 0.1,
            "layers": []
        }),
    ));

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::UnsupportedRecipeKind));
}

#[test]
fn test_legacy_texture_recipe_kind_rejected() {
    let spec = Spec::builder("legacy-texture-01", AssetType::Texture)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(
            OutputFormat::Png,
            "textures/legacy.png",
        ))
        .recipe(crate::recipe::Recipe::new(
            "texture.material_v1",
            serde_json::json!({
                "resolution": [16, 16],
                "tileable": true,
                "maps": ["albedo"]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::UnsupportedRecipeKind));
}

#[test]
fn test_audio_requires_wav_primary_output() {
    let mut spec = make_valid_spec();
    spec.outputs = vec![OutputSpec::primary(OutputFormat::Xm, "sounds/test.xm")];
    spec.recipe = Some(crate::recipe::Recipe::new(
        "audio_v1",
        serde_json::json!({
            "duration_seconds": 0.1,
            "layers": []
        }),
    ));

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::OutputValidationFailed));
}

#[test]
fn test_audio_lfo_rejects_depth_out_of_range() {
    let spec = Spec::builder("test-audio-lfo-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 2.0 },
                            "target": { "target": "volume", "amount": 1.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::InvalidRecipeParams));
}

#[test]
fn test_audio_lfo_allows_pitch_lfo_on_non_oscillator() {
    let spec = Spec::builder("test-audio-lfo-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "fm_synth", "carrier_freq": 440.0, "modulator_freq": 880.0, "modulation_index": 2.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 1.0 },
                            "target": { "target": "pitch", "semitones": 1.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_audio_lfo_rejects_filter_cutoff_lfo_without_filter() {
    let spec = Spec::builder("test-audio-lfo-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 1.0 },
                            "target": { "target": "filter_cutoff", "amount": 100.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("filter_cutoff LFO requires")));
}

#[test]
fn test_music_requires_primary_format_matches_recipe_format() {
    let spec = Spec::builder("test-song-01", AssetType::Music)
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
        .any(|e| e.code == ErrorCode::OutputValidationFailed));
}

#[test]
fn test_music_compose_requires_primary_format_matches_recipe_format() {
    let spec = Spec::builder("test-song-compose-01", AssetType::Music)
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
        .any(|e| e.code == ErrorCode::OutputValidationFailed));
}

#[test]
fn test_music_allows_dual_xm_it_primary_outputs() {
    let spec = Spec::builder("test-song-02", AssetType::Music)
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
    let missing_source = Spec::builder("test-song-04", AssetType::Music)
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
        .any(|e| e.code == ErrorCode::InvalidRecipeParams));

    let multiple_sources = Spec::builder("test-song-05", AssetType::Music)
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
        .any(|e| e.code == ErrorCode::InvalidRecipeParams));
}

#[test]
fn test_music_dual_outputs_rejects_duplicate_primary_format() {
    let spec = Spec::builder("test-song-03", AssetType::Music)
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
        .any(|e| e.code == ErrorCode::OutputValidationFailed));
}

#[test]
fn test_warnings() {
    let spec = Spec::builder("test-01", AssetType::Audio)
        .license("")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .build();

    let result = validate_spec(&spec);
    // Should pass but with warnings
    assert!(result.is_ok());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.code == WarningCode::MissingLicense));
    assert!(result
        .warnings
        .iter()
        .any(|w| w.code == WarningCode::MissingDescription));
}

#[test]
fn test_seed_near_overflow_warning() {
    let mut spec = make_valid_spec();
    spec.seed = u32::MAX;
    let result = validate_spec(&spec);
    assert!(result.is_ok());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.code == WarningCode::SeedNearOverflow));
}

#[test]
fn test_is_safe_output_path() {
    assert!(is_safe_output_path("sounds/laser.wav"));
    assert!(is_safe_output_path("textures/albedo.png"));
    assert!(is_safe_output_path("meshes/crate.glb"));

    assert!(!is_safe_output_path(""));
    assert!(!is_safe_output_path("/absolute/path"));
    assert!(!is_safe_output_path("C:/windows/path"));
    assert!(!is_safe_output_path("path\\backslash"));
    assert!(!is_safe_output_path("../traversal"));
}

fn make_valid_texture_procedural_spec() -> Spec {
    let mut output = OutputSpec::primary(OutputFormat::Png, "textures/mask.png");
    output.source = Some("mask".to_string());

    Spec::builder("procedural-test-01", AssetType::Texture)
        .license("CC0-1.0")
        .seed(123)
        .output(output)
        .recipe(Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({
                "resolution": [16, 16],
                "tileable": true,
                "nodes": [
                    { "id": "n", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.1 } },
                    { "id": "mask", "type": "threshold", "input": "n", "threshold": 0.5 }
                ]
            }),
        ))
        .build()
}

#[test]
fn test_texture_procedural_valid_spec() {
    let spec = make_valid_texture_procedural_spec();
    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_texture_procedural_rejects_missing_output_source() {
    let mut spec = make_valid_texture_procedural_spec();
    spec.outputs[0].source = None;

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("must set 'source'")));
}

#[test]
fn test_texture_procedural_rejects_unknown_output_source() {
    let mut spec = make_valid_texture_procedural_spec();
    spec.outputs[0].source = Some("missing".to_string());

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("does not match any recipe.params.nodes")));
}

#[test]
fn test_texture_procedural_rejects_cycles() {
    let mut output = OutputSpec::primary(OutputFormat::Png, "textures/a.png");
    output.source = Some("a".to_string());

    let spec = Spec::builder("procedural-cycle-01", AssetType::Texture)
        .license("CC0-1.0")
        .seed(1)
        .output(output)
        .recipe(Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({
                "resolution": [8, 8],
                "tileable": true,
                "nodes": [
                    { "id": "a", "type": "invert", "input": "b" },
                    { "id": "b", "type": "invert", "input": "a" }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("cycle detected")));
}

#[test]
fn test_texture_procedural_rejects_obvious_type_mismatch() {
    let mut output = OutputSpec::primary(OutputFormat::Png, "textures/bad.png");
    output.source = Some("bad".to_string());

    let spec = Spec::builder("procedural-types-01", AssetType::Texture)
        .license("CC0-1.0")
        .seed(1)
        .output(output)
        .recipe(Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({
                "resolution": [8, 8],
                "tileable": true,
                "nodes": [
                    { "id": "n", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.1 } },
                    { "id": "bad", "type": "palette", "input": "n", "palette": ["#000000", "#ffffff"] }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("type mismatch")));
}

#[test]
fn test_audio_lfo_allows_fm_index_on_fm_synth() {
    let spec = Spec::builder("test-audio-fm-index-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "fm_synth", "carrier_freq": 440.0, "modulator_freq": 880.0, "modulation_index": 4.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 1.0 },
                            "target": { "target": "fm_index", "amount": 2.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_audio_lfo_rejects_fm_index_on_non_fm_synth() {
    let spec = Spec::builder("test-audio-fm-index-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 1.0 },
                            "target": { "target": "fm_index", "amount": 2.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("fm_index LFO target is only valid for FmSynth")));
}

#[test]
fn test_audio_lfo_rejects_fm_index_with_zero_amount() {
    let spec = Spec::builder("test-audio-fm-index-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "fm_synth", "carrier_freq": 440.0, "modulator_freq": 880.0, "modulation_index": 4.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 1.0 },
                            "target": { "target": "fm_index", "amount": 0.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::InvalidRecipeParams));
}

#[test]
fn test_audio_lfo_allows_grain_size_on_granular() {
    let spec = Spec::builder("test-audio-grain-size-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": {
                            "type": "granular",
                            "source": { "type": "tone", "waveform": "sine", "frequency": 440.0 },
                            "grain_size_ms": 50.0,
                            "grain_density": 20.0
                        },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                            "target": { "target": "grain_size", "amount_ms": 30.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_audio_lfo_rejects_grain_size_on_non_granular() {
    let spec = Spec::builder("test-audio-grain-size-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                            "target": { "target": "grain_size", "amount_ms": 30.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("grain_size LFO target is only valid for Granular")));
}

#[test]
fn test_audio_lfo_rejects_grain_size_with_zero_amount() {
    let spec = Spec::builder("test-audio-grain-size-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": {
                            "type": "granular",
                            "source": { "type": "tone", "waveform": "sine", "frequency": 440.0 },
                            "grain_size_ms": 50.0,
                            "grain_density": 20.0
                        },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                            "target": { "target": "grain_size", "amount_ms": 0.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::InvalidRecipeParams));
}

#[test]
fn test_audio_lfo_allows_grain_density_on_granular() {
    let spec = Spec::builder("test-audio-grain-dens-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": {
                            "type": "granular",
                            "source": { "type": "noise", "noise_type": "white" },
                            "grain_size_ms": 50.0,
                            "grain_density": 20.0
                        },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "triangle", "rate": 1.0, "depth": 0.8 },
                            "target": { "target": "grain_density", "amount": 15.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_audio_lfo_rejects_grain_density_on_non_granular() {
    let spec = Spec::builder("test-audio-grain-dens-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "fm_synth", "carrier_freq": 440.0, "modulator_freq": 880.0, "modulation_index": 2.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                            "target": { "target": "grain_density", "amount": 15.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("grain_density LFO target is only valid for Granular")));
}

#[test]
fn test_audio_lfo_rejects_grain_density_with_zero_amount() {
    let spec = Spec::builder("test-audio-grain-dens-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": {
                            "type": "granular",
                            "source": { "type": "noise", "noise_type": "white" },
                            "grain_size_ms": 50.0,
                            "grain_density": 20.0
                        },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                            "target": { "target": "grain_density", "amount": 0.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == ErrorCode::InvalidRecipeParams));
}

// ============================================================================
// Post-FX LFO Tests
// ============================================================================

#[test]
fn test_post_fx_lfo_delay_time_valid() {
    let spec = Spec::builder("test-post-fx-lfo-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.5,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.1 },
                        "volume": 0.8,
                        "pan": 0.0
                    }
                ],
                "effects": [
                    { "type": "delay", "time_ms": 250.0, "feedback": 0.4, "wet": 0.3 }
                ],
                "post_fx_lfos": [
                    {
                        "config": { "waveform": "sine", "rate": 0.5, "depth": 1.0 },
                        "target": { "target": "delay_time", "amount_ms": 25.0 }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_post_fx_lfo_rejects_delay_time_on_layer() {
    let spec = Spec::builder("test-post-fx-lfo-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.5,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.1 },
                        "volume": 0.8,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 0.5, "depth": 1.0 },
                            "target": { "target": "delay_time", "amount_ms": 25.0 }
                        }
                    }
                ],
                "effects": [
                    { "type": "delay", "time_ms": 250.0, "feedback": 0.4, "wet": 0.3 }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("delay_time LFO target is only valid in post_fx_lfos")));
}

#[test]
fn test_post_fx_lfo_rejects_layer_target_on_post_fx() {
    let spec = Spec::builder("test-post-fx-lfo-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.5,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.1 },
                        "volume": 0.8,
                        "pan": 0.0
                    }
                ],
                "effects": [],
                "post_fx_lfos": [
                    {
                        "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                        "target": { "target": "pitch", "semitones": 2.0 }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("pitch LFO target is only valid in layer LFOs")));
}

#[test]
fn test_post_fx_lfo_rejects_duplicate_target() {
    let spec = Spec::builder("test-post-fx-lfo-04", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.5,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.1 },
                        "volume": 0.8,
                        "pan": 0.0
                    }
                ],
                "effects": [
                    { "type": "delay", "time_ms": 250.0, "feedback": 0.4, "wet": 0.3 }
                ],
                "post_fx_lfos": [
                    {
                        "config": { "waveform": "sine", "rate": 0.5, "depth": 1.0 },
                        "target": { "target": "delay_time", "amount_ms": 25.0 }
                    },
                    {
                        "config": { "waveform": "triangle", "rate": 1.0, "depth": 0.5 },
                        "target": { "target": "delay_time", "amount_ms": 10.0 }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("duplicate delay_time target in post_fx_lfos")));
}

#[test]
fn test_post_fx_lfo_rejects_delay_time_without_delay_effect() {
    let spec = Spec::builder("test-post-fx-lfo-05", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.5,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.1 },
                        "volume": 0.8,
                        "pan": 0.0
                    }
                ],
                "effects": [
                    { "type": "reverb", "room_size": 0.7, "damping": 0.5, "wet": 0.3 }
                ],
                "post_fx_lfos": [
                    {
                        "config": { "waveform": "sine", "rate": 0.5, "depth": 1.0 },
                        "target": { "target": "delay_time", "amount_ms": 25.0 }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("delay_time LFO requires at least one delay effect")));
}
