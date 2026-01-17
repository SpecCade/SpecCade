//! General validation tests.

use crate::output::{OutputFormat, OutputSpec};
use crate::spec::AssetType;
use crate::validation::*;

fn make_valid_spec() -> crate::spec::Spec {
    crate::spec::Spec::builder("test-asset-01", AssetType::Audio)
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
        .any(|e| e.code == crate::error::ErrorCode::UnsupportedSpecVersion));
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
                .any(|e| e.code == crate::error::ErrorCode::InvalidAssetId),
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
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::NoOutputs));
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
        .any(|e| e.code == crate::error::ErrorCode::NoPrimaryOutput));
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
        .any(|e| e.code == crate::error::ErrorCode::OutputValidationFailed));
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
        .any(|e| e.code == crate::error::ErrorCode::DuplicateOutputPath));
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
                .any(|e| e.code == crate::error::ErrorCode::UnsafeOutputPath),
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
        .any(|e| e.code == crate::error::ErrorCode::PathFormatMismatch));
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
        .any(|e| e.code == crate::error::ErrorCode::RecipeAssetTypeMismatch));
}

#[test]
fn test_missing_recipe_for_generate() {
    let spec = make_valid_spec();
    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::MissingRecipe));
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
        .any(|e| e.code == crate::error::ErrorCode::UnsupportedRecipeKind));
}

#[test]
fn test_legacy_texture_recipe_kind_rejected() {
    let spec = crate::spec::Spec::builder("legacy-texture-01", AssetType::Texture)
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
        .any(|e| e.code == crate::error::ErrorCode::UnsupportedRecipeKind));
}

#[test]
fn test_warnings() {
    let spec = crate::spec::Spec::builder("test-01", AssetType::Audio)
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
        .any(|w| w.code == crate::error::WarningCode::MissingLicense));
    assert!(result
        .warnings
        .iter()
        .any(|w| w.code == crate::error::WarningCode::MissingDescription));
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
        .any(|w| w.code == crate::error::WarningCode::SeedNearOverflow));
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
