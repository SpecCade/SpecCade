//! Integration tests for Starlark input support.
//!
//! These tests verify that:
//! - Starlark specs can be loaded and parsed
//! - Starlark specs produce valid canonical Spec IR
//! - The eval command works correctly with Starlark files
//! - JSON and Starlark specs produce equivalent IR when content matches

use speccade_cli::input::{load_spec, SourceKind};
use speccade_spec::{AssetType, Spec};

/// Path to the golden Starlark test fixtures
fn golden_starlark_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("golden")
        .join("starlark")
}

#[test]
fn load_minimal_starlark_spec() {
    let path = golden_starlark_path().join("minimal.star");
    let result = load_spec(&path).expect("should load minimal.star");

    assert_eq!(result.source_kind, SourceKind::Starlark);
    assert_eq!(result.spec.asset_id, "starlark-minimal-01");
    assert_eq!(result.spec.seed, 42);
    assert_eq!(result.spec.license, "CC0-1.0");
    assert!(result.warnings.is_empty());
    assert!(!result.source_hash.is_empty());
}

#[test]
fn load_starlark_with_functions() {
    let path = golden_starlark_path().join("with_functions.star");
    let result = load_spec(&path).expect("should load with_functions.star");

    assert_eq!(result.source_kind, SourceKind::Starlark);
    assert_eq!(result.spec.asset_id, "starlark-functions-01");
    assert_eq!(result.spec.seed, 12345);
    assert_eq!(result.spec.outputs.len(), 2);

    // Check that outputs were generated correctly
    let paths: Vec<_> = result.spec.outputs.iter().map(|o| o.path.as_str()).collect();
    assert!(paths.contains(&"sounds/laser_blast.wav"));
    assert!(paths.contains(&"sounds/laser_charge.wav"));

    // Check that tags were combined correctly
    let tags = result.spec.style_tags.as_ref().expect("should have tags");
    assert!(tags.contains(&"retro".to_string()));
    assert!(tags.contains(&"laser".to_string()));
}

#[test]
fn load_starlark_with_comprehensions() {
    let path = golden_starlark_path().join("with_comprehensions.star");
    let result = load_spec(&path).expect("should load with_comprehensions.star");

    assert_eq!(result.source_kind, SourceKind::Starlark);
    assert_eq!(result.spec.asset_id, "starlark-comprehension-01");
    assert_eq!(result.spec.outputs.len(), 4);

    // Check that outputs were generated via comprehension
    let paths: Vec<_> = result.spec.outputs.iter().map(|o| o.path.as_str()).collect();
    assert!(paths.contains(&"drums/kick.wav"));
    assert!(paths.contains(&"drums/snare.wav"));
    assert!(paths.contains(&"drums/hihat.wav"));
    assert!(paths.contains(&"drums/tom.wav"));

    // Check that tags were uppercased
    let tags = result.spec.style_tags.as_ref().expect("should have tags");
    assert!(tags.contains(&"DRUM".to_string()));
    assert!(tags.contains(&"PERCUSSION".to_string()));
}

#[test]
fn starlark_spec_produces_valid_json() {
    let path = golden_starlark_path().join("minimal.star");
    let result = load_spec(&path).expect("should load minimal.star");

    // Should be able to serialize to JSON and back
    let json = result.spec.to_json_pretty().expect("should serialize to JSON");
    let parsed = Spec::from_json(&json).expect("should parse JSON");

    assert_eq!(parsed.asset_id, result.spec.asset_id);
    assert_eq!(parsed.seed, result.spec.seed);
}

#[test]
fn json_spec_source_kind_is_json() {
    // Create a temp JSON file
    let tmp = tempfile::tempdir().unwrap();
    let json_path = tmp.path().join("test.json");

    let json = r#"{
        "spec_version": 1,
        "asset_id": "json-test-01",
        "asset_type": "audio",
        "license": "CC0-1.0",
        "seed": 42,
        "outputs": [
            {
                "kind": "primary",
                "format": "wav",
                "path": "test.wav"
            }
        ]
    }"#;
    std::fs::write(&json_path, json).unwrap();

    let result = load_spec(&json_path).expect("should load json");
    assert_eq!(result.source_kind, SourceKind::Json);
}

#[test]
fn equivalent_json_and_starlark_produce_same_ir() {
    // Create equivalent JSON and Starlark specs
    let tmp = tempfile::tempdir().unwrap();

    let json_content = r#"{
        "spec_version": 1,
        "asset_id": "equivalence-test-01",
        "asset_type": "audio",
        "license": "CC0-1.0",
        "seed": 42,
        "outputs": [
            {
                "kind": "primary",
                "format": "wav",
                "path": "test.wav"
            }
        ]
    }"#;

    let starlark_content = r#"
{
    "spec_version": 1,
    "asset_id": "equivalence-test-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {
            "kind": "primary",
            "format": "wav",
            "path": "test.wav"
        }
    ]
}
"#;

    let json_path = tmp.path().join("spec.json");
    let star_path = tmp.path().join("spec.star");

    std::fs::write(&json_path, json_content).unwrap();
    std::fs::write(&star_path, starlark_content).unwrap();

    let json_result = load_spec(&json_path).expect("should load json");
    let star_result = load_spec(&star_path).expect("should load star");

    // The specs should be equivalent
    assert_eq!(json_result.spec.asset_id, star_result.spec.asset_id);
    assert_eq!(json_result.spec.seed, star_result.spec.seed);
    assert_eq!(json_result.spec.license, star_result.spec.license);
    assert_eq!(json_result.spec.outputs.len(), star_result.spec.outputs.len());

    // The canonical JSON should be identical
    let json_canonical = json_result.spec.to_json().unwrap();
    let star_canonical = star_result.spec.to_json().unwrap();
    assert_eq!(json_canonical, star_canonical);
}

#[test]
fn starlark_syntax_error_returns_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("invalid.star");

    // Invalid Starlark syntax
    std::fs::write(&path, "{ invalid syntax }").unwrap();

    let result = load_spec(&path);
    assert!(result.is_err());
}

#[test]
fn starlark_runtime_error_returns_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("runtime_error.star");

    // Reference to undefined variable
    std::fs::write(&path, "undefined_variable").unwrap();

    let result = load_spec(&path);
    assert!(result.is_err());
}

#[test]
fn starlark_non_dict_result_returns_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("not_dict.star");

    // Returns a list instead of a dict
    std::fs::write(&path, "[1, 2, 3]").unwrap();

    let result = load_spec(&path);
    assert!(result.is_err());
}

#[test]
fn starlark_invalid_spec_returns_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("invalid_spec.star");

    // Valid dict but missing required spec fields
    std::fs::write(&path, "{\"only_field\": \"value\"}").unwrap();

    let result = load_spec(&path);
    assert!(result.is_err());
}

#[test]
fn unknown_extension_returns_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("test.yaml");
    std::fs::write(&path, "key: value").unwrap();

    let result = load_spec(&path);
    assert!(result.is_err());
}

#[test]
fn source_hash_differs_for_different_content() {
    let tmp = tempfile::tempdir().unwrap();

    let path1 = tmp.path().join("spec1.star");
    let path2 = tmp.path().join("spec2.star");

    let content1 = r#"
{
    "spec_version": 1,
    "asset_id": "hash-test-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 1,
    "outputs": [{"kind": "primary", "format": "wav", "path": "a.wav"}]
}
"#;

    let content2 = r#"
{
    "spec_version": 1,
    "asset_id": "hash-test-02",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 2,
    "outputs": [{"kind": "primary", "format": "wav", "path": "b.wav"}]
}
"#;

    std::fs::write(&path1, content1).unwrap();
    std::fs::write(&path2, content2).unwrap();

    let result1 = load_spec(&path1).expect("should load spec1");
    let result2 = load_spec(&path2).expect("should load spec2");

    // Source hashes should be different
    assert_ne!(result1.source_hash, result2.source_hash);
}

// ========================================================================
// Stdlib examples tests
// ========================================================================

#[test]
fn load_stdlib_audio_oscillator() {
    let path = golden_starlark_path().join("audio_synth_oscillator.star");
    let result = load_spec(&path).expect("should load audio_synth_oscillator.star");

    assert_eq!(result.source_kind, SourceKind::Starlark);
    assert_eq!(result.spec.asset_id, "stdlib-audio-osc-01");
    assert_eq!(result.spec.asset_type, AssetType::Audio);
    assert!(result.spec.recipe.is_some());
}

#[test]
fn load_stdlib_audio_fm() {
    let path = golden_starlark_path().join("audio_synth_fm.star");
    let result = load_spec(&path).expect("should load audio_synth_fm.star");

    assert_eq!(result.spec.asset_id, "stdlib-audio-fm-01");
    assert!(result.spec.recipe.is_some());
}

#[test]
fn load_stdlib_audio_layered() {
    let path = golden_starlark_path().join("audio_synth_layered.star");
    let result = load_spec(&path).expect("should load audio_synth_layered.star");

    assert_eq!(result.spec.asset_id, "stdlib-audio-layered-01");
    assert!(result.spec.recipe.is_some());
}

#[test]
fn load_stdlib_texture_noise() {
    let path = golden_starlark_path().join("texture_noise.star");
    let result = load_spec(&path).expect("should load texture_noise.star");

    assert_eq!(result.spec.asset_id, "stdlib-texture-noise-01");
    assert_eq!(result.spec.asset_type, AssetType::Texture);
    assert!(result.spec.recipe.is_some());
}

#[test]
fn load_stdlib_texture_colored() {
    let path = golden_starlark_path().join("texture_colored.star");
    let result = load_spec(&path).expect("should load texture_colored.star");

    assert_eq!(result.spec.asset_id, "stdlib-texture-colored-01");
    assert!(result.spec.recipe.is_some());
}

#[test]
fn load_stdlib_mesh_cube() {
    let path = golden_starlark_path().join("mesh_cube.star");
    let result = load_spec(&path).expect("should load mesh_cube.star");

    assert_eq!(result.spec.asset_id, "stdlib-mesh-cube-01");
    assert_eq!(result.spec.asset_type, AssetType::StaticMesh);
    assert!(result.spec.recipe.is_some());
}

#[test]
fn load_stdlib_mesh_sphere() {
    let path = golden_starlark_path().join("mesh_sphere.star");
    let result = load_spec(&path).expect("should load mesh_sphere.star");

    assert_eq!(result.spec.asset_id, "stdlib-mesh-sphere-01");
    assert!(result.spec.recipe.is_some());
}
