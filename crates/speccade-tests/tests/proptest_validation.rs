//! Property-based validation tests for SpecCade using proptest.
//!
//! These tests verify that validation functions never panic and always
//! return correct results for arbitrary inputs, including boundary values.
//!
//! ## Running Tests
//!
//! ```bash
//! cargo test -p speccade-tests --test proptest_validation
//! ```

use proptest::prelude::*;

use speccade_spec::validation::common::{
    validate_non_negative, validate_positive, validate_range, validate_resolution,
    validate_unit_interval,
};
use speccade_spec::validation::is_valid_asset_id;
use speccade_spec::{AssetType, ErrorCode, OutputFormat, OutputSpec, Spec, SPEC_VERSION};

// ============================================================================
// 1. Asset ID Validation Boundaries
// ============================================================================

/// Strategy for generating random strings that may or may not be valid asset IDs.
fn arbitrary_string() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_\\-!@#$%^&*() ]{0,100}")
        .unwrap()
        .boxed()
}

proptest! {
    /// Random strings never panic when validated as asset IDs.
    #[test]
    fn asset_id_validation_never_panics(s in arbitrary_string()) {
        // Should never panic, only return true/false.
        let _ = is_valid_asset_id(&s);
    }

    /// Valid asset IDs match the documented regex pattern.
    #[test]
    fn valid_asset_ids_pass(
        prefix in "[a-z]",
        rest in "[a-z0-9_\\-]{2,63}"
    ) {
        let id = format!("{}{}", prefix, rest);
        // Truncate to 64 chars max (1 prefix + 63 rest)
        let id = &id[..id.len().min(64)];
        prop_assert!(
            is_valid_asset_id(id),
            "Expected valid asset_id: '{}'", id
        );
    }

    /// IDs starting with uppercase always fail.
    #[test]
    fn uppercase_start_asset_ids_fail(
        first in "[A-Z]",
        rest in "[a-z0-9_\\-]{2,20}"
    ) {
        let id = format!("{}{}", first, rest);
        prop_assert!(
            !is_valid_asset_id(&id),
            "Expected invalid asset_id (uppercase start): '{}'", id
        );
    }

    /// IDs that are too short (< 3 chars) always fail.
    #[test]
    fn short_asset_ids_fail(s in "[a-z][a-z0-9]{0,1}") {
        prop_assert!(
            !is_valid_asset_id(&s),
            "Expected invalid asset_id (too short): '{}'", s
        );
    }

    /// IDs that are too long (> 64 chars) always fail.
    #[test]
    fn long_asset_ids_fail(
        prefix in "[a-z]",
        rest in "[a-z0-9_\\-]{64,100}"
    ) {
        let id = format!("{}{}", prefix, rest);
        prop_assert!(
            !is_valid_asset_id(&id),
            "Expected invalid asset_id (too long, len={}): '{}'", id.len(), id
        );
    }

    /// Empty string always fails.
    #[test]
    fn empty_asset_id_fails(_dummy in 0u8..1u8) {
        prop_assert!(!is_valid_asset_id(""));
    }

    /// IDs with invalid characters that spec rejects always fail validation.
    #[test]
    fn invalid_asset_id_rejects_in_spec(bad_id in "[A-Z!@#$%^&*() ]{1,20}") {
        let spec = Spec::builder(bad_id.clone(), AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let result = speccade_spec::validate_spec(&spec);
        let has_asset_id_error = result.errors.iter().any(|e| e.code == ErrorCode::InvalidAssetId);
        prop_assert!(
            has_asset_id_error,
            "Expected InvalidAssetId error for '{}', got errors: {:?}",
            bad_id, result.errors
        );
    }
}

// ============================================================================
// 2. Audio Parameter Ranges
// ============================================================================

proptest! {
    /// validate_unit_interval never panics for any f64.
    #[test]
    fn unit_interval_never_panics(value in prop::num::f64::ANY) {
        let _ = validate_unit_interval("test", value);
    }

    /// validate_unit_interval accepts [0.0, 1.0].
    #[test]
    fn unit_interval_accepts_valid(value in 0.0f64..=1.0f64) {
        prop_assert!(validate_unit_interval("test", value).is_ok());
    }

    /// validate_unit_interval rejects values > 1.0.
    #[test]
    fn unit_interval_rejects_above(value in 1.001f64..1e10f64) {
        prop_assert!(validate_unit_interval("test", value).is_err());
    }

    /// validate_unit_interval rejects values < 0.0.
    #[test]
    fn unit_interval_rejects_below(value in -1e10f64..-0.001f64) {
        prop_assert!(validate_unit_interval("test", value).is_err());
    }

    /// validate_positive never panics for any f64.
    #[test]
    fn positive_never_panics(value in prop::num::f64::ANY) {
        let _ = validate_positive("test", value);
    }

    /// validate_positive accepts > 0.
    #[test]
    fn positive_accepts_valid(value in 0.001f64..1e10f64) {
        prop_assert!(validate_positive("test", value).is_ok());
    }

    /// validate_positive rejects <= 0.
    #[test]
    fn positive_rejects_non_positive(value in -1e10f64..=0.0f64) {
        prop_assert!(validate_positive("test", value).is_err());
    }

    /// validate_non_negative never panics for any f64.
    #[test]
    fn non_negative_never_panics(value in prop::num::f64::ANY) {
        let _ = validate_non_negative("test", value);
    }

    /// validate_non_negative accepts >= 0.
    #[test]
    fn non_negative_accepts_valid(value in 0.0f64..1e10f64) {
        prop_assert!(validate_non_negative("test", value).is_ok());
    }

    /// validate_non_negative rejects < 0.
    #[test]
    fn non_negative_rejects_negative(value in -1e10f64..-0.001f64) {
        prop_assert!(validate_non_negative("test", value).is_err());
    }

    /// validate_range never panics for any f64 inputs.
    #[test]
    fn range_never_panics(
        value in prop::num::f64::ANY,
        min in prop::num::f64::ANY,
        max in prop::num::f64::ANY,
    ) {
        let _ = validate_range("test", value, min, max);
    }

    /// validate_range accepts values within bounds.
    #[test]
    fn range_accepts_in_bounds(
        min in 0.0f64..100.0f64,
        span in 0.0f64..100.0f64,
        frac in 0.0f64..=1.0f64,
    ) {
        let max = min + span;
        let value = min + frac * span;
        prop_assert!(validate_range("test", value, min, max).is_ok());
    }

    /// NaN always rejected by all validators.
    #[test]
    fn nan_always_rejected(_dummy in 0u8..1u8) {
        prop_assert!(validate_unit_interval("test", f64::NAN).is_err());
        prop_assert!(validate_positive("test", f64::NAN).is_err());
        prop_assert!(validate_non_negative("test", f64::NAN).is_err());
        prop_assert!(validate_range("test", f64::NAN, 0.0, 1.0).is_err());
    }

    /// Infinity always rejected by all validators.
    #[test]
    fn infinity_always_rejected(_dummy in 0u8..1u8) {
        prop_assert!(validate_unit_interval("test", f64::INFINITY).is_err());
        prop_assert!(validate_unit_interval("test", f64::NEG_INFINITY).is_err());
        prop_assert!(validate_positive("test", f64::INFINITY).is_err());
        prop_assert!(validate_non_negative("test", f64::INFINITY).is_err());
        prop_assert!(validate_range("test", f64::INFINITY, 0.0, 100.0).is_err());
    }
}

// ============================================================================
// 3. Texture Resolution Bounds
// ============================================================================

proptest! {
    /// validate_resolution never panics for arbitrary dimensions.
    #[test]
    fn resolution_never_panics(width in 0u32..=16384u32, height in 0u32..=16384u32) {
        let _ = validate_resolution(width, height);
    }

    /// Zero width or height always rejected.
    #[test]
    fn resolution_rejects_zero_width(height in 1u32..=4096u32) {
        prop_assert!(validate_resolution(0, height).is_err());
    }

    #[test]
    fn resolution_rejects_zero_height(width in 1u32..=4096u32) {
        prop_assert!(validate_resolution(width, 0).is_err());
    }

    /// Valid resolutions within 4096x4096 accepted.
    #[test]
    fn resolution_accepts_valid(width in 1u32..=4096u32, height in 1u32..=4096u32) {
        prop_assert!(validate_resolution(width, height).is_ok());
    }

    /// Oversized resolutions (> 4096) rejected.
    #[test]
    fn resolution_rejects_oversized(width in 4097u32..=16384u32, height in 1u32..=4096u32) {
        prop_assert!(validate_resolution(width, height).is_err());
    }

    #[test]
    fn resolution_rejects_oversized_height(width in 1u32..=4096u32, height in 4097u32..=16384u32) {
        prop_assert!(validate_resolution(width, height).is_err());
    }
}

// ============================================================================
// 4. Spec Version Rejection
// ============================================================================

proptest! {
    /// All spec_version values != SPEC_VERSION (1) are rejected.
    #[test]
    fn invalid_spec_version_rejected(version in 2u32..=u32::MAX) {
        let json = format!(
            r#"{{
                "spec_version": {},
                "asset_id": "test-asset-01",
                "asset_type": "audio",
                "license": "CC0-1.0",
                "seed": 42,
                "outputs": [
                    {{
                        "kind": "primary",
                        "format": "wav",
                        "path": "sounds/test.wav"
                    }}
                ]
            }}"#,
            version
        );

        let spec = Spec::from_json(&json).unwrap();
        let result = speccade_spec::validate_spec(&spec);
        let has_version_error = result.errors.iter().any(|e| e.code == ErrorCode::UnsupportedSpecVersion);
        prop_assert!(
            has_version_error,
            "Expected UnsupportedSpecVersion for version={}, errors: {:?}",
            version, result.errors
        );
    }

    /// Spec version 0 is also rejected.
    #[test]
    fn spec_version_zero_rejected(_dummy in 0u8..1u8) {
        let spec = Spec {
            spec_version: 0,
            asset_id: "test-asset-01".to_string(),
            asset_type: AssetType::Audio,
            license: "CC0-1.0".to_string(),
            seed: 42,
            outputs: vec![OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav")],
            description: None,
            style_tags: None,
            engine_targets: None,
            migration_notes: None,
            variants: None,
            recipe: None,
        };

        let result = speccade_spec::validate_spec(&spec);
        let has_version_error = result.errors.iter().any(|e| e.code == ErrorCode::UnsupportedSpecVersion);
        prop_assert!(has_version_error);
    }
}

// ============================================================================
// 5. Serde Roundtrip Invariant
// ============================================================================

proptest! {
    /// Spec roundtrip: serialize -> deserialize -> serialize produces identical JSON.
    #[test]
    fn spec_serde_roundtrip(
        seed in 0u32..=u32::MAX,
        asset_type_idx in 0usize..3usize,
    ) {
        let (asset_type, format, ext) = match asset_type_idx {
            0 => (AssetType::Audio, OutputFormat::Wav, "sounds/test.wav"),
            1 => (AssetType::Texture, OutputFormat::Png, "textures/test.png"),
            2 => (AssetType::Music, OutputFormat::Xm, "music/test.xm"),
            _ => unreachable!(),
        };

        let spec = Spec::builder("roundtrip-test-01", asset_type)
            .license("CC0-1.0")
            .seed(seed)
            .output(OutputSpec::primary(format, ext))
            .build();

        // First serialize
        let json1 = spec.to_json().unwrap();
        // Deserialize
        let parsed = Spec::from_json(&json1).unwrap();
        // Second serialize
        let json2 = parsed.to_json().unwrap();

        prop_assert_eq!(
            json1, json2,
            "Serde roundtrip produced different JSON"
        );
    }

    /// Spec with description roundtrips correctly.
    #[test]
    fn spec_with_description_roundtrip(
        desc in "[a-zA-Z0-9 ]{0,50}",
        seed in 0u32..100000u32,
    ) {
        let spec = Spec::builder("desc-roundtrip-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(seed)
            .description(desc)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let json1 = spec.to_json().unwrap();
        let parsed = Spec::from_json(&json1).unwrap();
        let json2 = parsed.to_json().unwrap();

        prop_assert_eq!(json1, json2);
    }

    /// Spec with style_tags roundtrips correctly.
    #[test]
    fn spec_with_tags_roundtrip(
        tag1 in "[a-z]{3,10}",
        tag2 in "[a-z]{3,10}",
    ) {
        let spec = Spec::builder("tags-roundtrip-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .tag(tag1)
            .tag(tag2)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let json1 = spec.to_json().unwrap();
        let parsed = Spec::from_json(&json1).unwrap();
        let json2 = parsed.to_json().unwrap();

        prop_assert_eq!(json1, json2);
    }
}

// ============================================================================
// 6. Spec Validation Invariants
// ============================================================================

proptest! {
    /// validate_spec never panics regardless of seed value.
    #[test]
    fn validate_spec_never_panics(seed in 0u32..=u32::MAX) {
        let spec = Spec::builder("test-seed-range-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(seed)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        // Should never panic.
        let result = speccade_spec::validate_spec(&spec);

        // Valid spec should always pass (spec_version is correct).
        prop_assert!(
            result.is_ok(),
            "Expected valid spec with seed={}, errors: {:?}",
            seed, result.errors
        );
    }

    /// validate_for_generate rejects specs without recipes.
    #[test]
    fn validate_for_generate_rejects_no_recipe(seed in 0u32..=u32::MAX) {
        let spec = Spec::builder("no-recipe-proptest-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(seed)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let result = speccade_spec::validate_for_generate(&spec);
        let has_missing_recipe = result.errors.iter().any(|e| e.code == ErrorCode::MissingRecipe);
        prop_assert!(
            has_missing_recipe,
            "Expected MissingRecipe error for seed={}", seed
        );
    }

    /// Specs with empty outputs always fail.
    #[test]
    fn empty_outputs_always_fail(seed in 0u32..1000u32) {
        let spec = Spec {
            spec_version: SPEC_VERSION,
            asset_id: "empty-outputs-01".to_string(),
            asset_type: AssetType::Audio,
            license: "CC0-1.0".to_string(),
            seed,
            outputs: vec![],
            description: None,
            style_tags: None,
            engine_targets: None,
            migration_notes: None,
            variants: None,
            recipe: None,
        };

        let result = speccade_spec::validate_spec(&spec);
        let has_no_outputs = result.errors.iter().any(|e| e.code == ErrorCode::NoOutputs);
        prop_assert!(
            has_no_outputs,
            "Expected NoOutputs error, got: {:?}", result.errors
        );
    }

    /// Duplicate output paths always detected.
    #[test]
    fn duplicate_paths_detected(seed in 0u32..1000u32) {
        let spec = Spec::builder("dup-path-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(seed)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/same.wav"))
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/same.wav"))
            .build();

        let result = speccade_spec::validate_spec(&spec);
        let has_dup = result.errors.iter().any(|e| e.code == ErrorCode::DuplicateOutputPath);
        prop_assert!(
            has_dup,
            "Expected DuplicateOutputPath error, got: {:?}", result.errors
        );
    }

    /// Seeds near u32::MAX produce overflow warnings.
    #[test]
    fn high_seeds_produce_warnings(offset in 0u32..=1000u32) {
        let seed = u32::MAX - offset;
        let spec = Spec::builder("high-seed-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(seed)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let result = speccade_spec::validate_spec(&spec);
        let has_overflow_warning = result.warnings.iter().any(|w| {
            w.code == speccade_spec::WarningCode::SeedNearOverflow
        });
        prop_assert!(
            has_overflow_warning,
            "Expected SeedNearOverflow warning for seed={}", seed
        );
    }
}

// ============================================================================
// 7. Output Path Safety
// ============================================================================

proptest! {
    /// Path traversal attempts always detected.
    #[test]
    fn path_traversal_rejected(depth in 1usize..5usize) {
        let traversal = "../".repeat(depth);
        let path = format!("{}etc/passwd", traversal);

        prop_assert!(
            !speccade_spec::is_safe_output_path(&path),
            "Expected unsafe path: '{}'", path
        );
    }

    /// Normal relative paths are safe.
    #[test]
    fn normal_paths_safe(
        dir in "[a-z]{3,10}",
        file in "[a-z]{3,10}",
    ) {
        let path = format!("{}/{}.wav", dir, file);
        prop_assert!(
            speccade_spec::is_safe_output_path(&path),
            "Expected safe path: '{}'", path
        );
    }
}
