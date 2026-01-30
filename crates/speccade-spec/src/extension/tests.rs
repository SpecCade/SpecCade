//! Tests for the extension system types.

use super::*;

// ============================================================================
// Manifest Tests
// ============================================================================

#[test]
fn test_determinism_level_tier() {
    assert_eq!(DeterminismLevel::ByteIdentical.tier(), 1);
    assert_eq!(DeterminismLevel::SemanticEquivalent.tier(), 2);
    assert_eq!(DeterminismLevel::NonDeterministic.tier(), 3);
}

#[test]
fn test_determinism_level_requires_hash() {
    assert!(DeterminismLevel::ByteIdentical.requires_hash());
    assert!(!DeterminismLevel::SemanticEquivalent.requires_hash());
    assert!(!DeterminismLevel::NonDeterministic.requires_hash());
}

#[test]
fn test_determinism_level_requires_metrics() {
    assert!(!DeterminismLevel::ByteIdentical.requires_metrics());
    assert!(DeterminismLevel::SemanticEquivalent.requires_metrics());
    assert!(!DeterminismLevel::NonDeterministic.requires_metrics());
}

#[test]
fn test_extension_manifest_subprocess() {
    let manifest = ExtensionManifest::subprocess(
        "custom-texture-gen",
        "1.0.0",
        "custom-texture-gen",
        vec!["texture.custom_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );

    assert_eq!(manifest.name, "custom-texture-gen");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.tier, 1);
    assert_eq!(manifest.determinism, DeterminismLevel::ByteIdentical);
    assert!(manifest.handles_recipe("texture.custom_v1"));
    assert!(!manifest.handles_recipe("texture.other_v1"));
}

#[test]
fn test_validate_extension_manifest_valid() {
    let manifest = ExtensionManifest::subprocess(
        "custom-texture-gen",
        "1.0.0",
        "custom-texture-gen",
        vec!["texture.custom_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );

    assert!(validate_extension_manifest(&manifest).is_ok());
}

#[test]
fn test_validate_extension_manifest_invalid_name() {
    let manifest = ExtensionManifest::subprocess(
        "Invalid_Name",
        "1.0.0",
        "custom-texture-gen",
        vec!["texture.custom_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );

    let result = validate_extension_manifest(&manifest);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ManifestValidationError::InvalidName(_))));
}

#[test]
fn test_validate_extension_manifest_tier_mismatch() {
    let mut manifest = ExtensionManifest::subprocess(
        "custom-texture-gen",
        "1.0.0",
        "custom-texture-gen",
        vec!["texture.custom_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );
    manifest.tier = 2; // Wrong tier for ByteIdentical

    let result = validate_extension_manifest(&manifest);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ManifestValidationError::TierMismatch { .. })));
}

#[test]
fn test_validate_extension_manifest_no_recipe_kinds() {
    let manifest = ExtensionManifest::subprocess(
        "custom-texture-gen",
        "1.0.0",
        "custom-texture-gen",
        vec![], // Empty!
        DeterminismLevel::ByteIdentical,
    );

    let result = validate_extension_manifest(&manifest);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ManifestValidationError::NoRecipeKinds)));
}

#[test]
fn test_validate_extension_manifest_invalid_recipe_kind() {
    let manifest = ExtensionManifest::subprocess(
        "custom-texture-gen",
        "1.0.0",
        "custom-texture-gen",
        vec!["InvalidKind".to_string()], // Invalid format
        DeterminismLevel::ByteIdentical,
    );

    let result = validate_extension_manifest(&manifest);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ManifestValidationError::InvalidRecipeKind(_))));
}

// ============================================================================
// Contract Tests
// ============================================================================

#[test]
fn test_extension_output_manifest_success() {
    let manifest = ExtensionOutputManifest::success(
        vec![ExtensionOutputFile::primary(
            "output.png",
            "a".repeat(64),
            1024,
            "png",
        )],
        DeterminismReport::tier1("b".repeat(64), "c".repeat(64), 42),
    );

    assert!(manifest.success);
    assert_eq!(manifest.output_files.len(), 1);
    assert!(manifest.errors.is_empty());
}

#[test]
fn test_extension_output_manifest_failure() {
    let manifest = ExtensionOutputManifest::failure(
        vec![ExtensionErrorEntry::new("ERR001", "Something went wrong")],
        DeterminismReport::tier1("a".repeat(64), "b".repeat(64), 42),
    );

    assert!(!manifest.success);
    assert!(manifest.output_files.is_empty());
    assert_eq!(manifest.errors.len(), 1);
    assert_eq!(manifest.errors[0].code, "ERR001");
}

#[test]
fn test_determinism_report_tier1() {
    let report = DeterminismReport::tier1("input_hash", "output_hash", 42);

    assert_eq!(report.tier, 1);
    assert_eq!(report.determinism, DeterminismLevel::ByteIdentical);
    assert!(report.deterministic);
    assert!(report.output_hash.is_some());
}

#[test]
fn test_determinism_report_tier2() {
    let report = DeterminismReport::tier2("input_hash", 42);

    assert_eq!(report.tier, 2);
    assert_eq!(report.determinism, DeterminismLevel::SemanticEquivalent);
    assert!(report.deterministic);
    assert!(report.output_hash.is_none());
}

#[test]
fn test_determinism_report_non_deterministic() {
    let report = DeterminismReport::non_deterministic("input_hash", 42, "Uses system time");

    assert_eq!(report.tier, 3);
    assert_eq!(report.determinism, DeterminismLevel::NonDeterministic);
    assert!(!report.deterministic);
    assert_eq!(
        report.non_determinism_reason,
        Some("Uses system time".to_string())
    );
}

#[test]
fn test_validate_output_manifest_valid() {
    let manifest = ExtensionOutputManifest::success(
        vec![ExtensionOutputFile::primary(
            "output.png",
            "a".repeat(64),
            1024,
            "png",
        )],
        DeterminismReport::tier1("b".repeat(64), "c".repeat(64), 42),
    );

    assert!(validate_output_manifest(&manifest).is_ok());
}

#[test]
fn test_validate_output_manifest_invalid_path() {
    let manifest = ExtensionOutputManifest::success(
        vec![ExtensionOutputFile::primary(
            "../escape.png", // Path traversal attempt
            "a".repeat(64),
            1024,
            "png",
        )],
        DeterminismReport::tier1("b".repeat(64), "c".repeat(64), 42),
    );

    let result = validate_output_manifest(&manifest);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, OutputManifestValidationError::InvalidOutputPath(_))));
}

#[test]
fn test_validate_output_manifest_invalid_hash() {
    let manifest = ExtensionOutputManifest::success(
        vec![ExtensionOutputFile::primary(
            "output.png",
            "invalid_hash", // Too short
            1024,
            "png",
        )],
        DeterminismReport::tier1("b".repeat(64), "c".repeat(64), 42),
    );

    let result = validate_output_manifest(&manifest);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, OutputManifestValidationError::InvalidOutputHash(_))));
}

#[test]
fn test_validate_output_manifest_tier1_missing_output_hash() {
    let mut report = DeterminismReport::tier1("b".repeat(64), "c".repeat(64), 42);
    report.output_hash = None; // Remove required hash

    let manifest = ExtensionOutputManifest::success(
        vec![ExtensionOutputFile::primary(
            "output.png",
            "a".repeat(64),
            1024,
            "png",
        )],
        report,
    );

    let result = validate_output_manifest(&manifest);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, OutputManifestValidationError::MissingOutputHash)));
}

#[test]
fn test_validate_output_manifest_success_no_files() {
    let manifest = ExtensionOutputManifest::success(
        vec![], // No files for successful run!
        DeterminismReport::tier1("b".repeat(64), "c".repeat(64), 42),
    );

    let result = validate_output_manifest(&manifest);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, OutputManifestValidationError::NoOutputFiles)));
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_extension_manifest_serialization() {
    let manifest = ExtensionManifest::subprocess(
        "custom-texture-gen",
        "1.0.0",
        "custom-texture-gen",
        vec!["texture.custom_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );

    let json = serde_json::to_string_pretty(&manifest).unwrap();
    let parsed: ExtensionManifest = serde_json::from_str(&json).unwrap();

    assert_eq!(manifest, parsed);
}

#[test]
fn test_output_manifest_serialization() {
    let manifest = ExtensionOutputManifest::success(
        vec![ExtensionOutputFile::primary(
            "output.png",
            "a".repeat(64),
            1024,
            "png",
        )],
        DeterminismReport::tier1("b".repeat(64), "c".repeat(64), 42),
    )
    .with_duration(100)
    .with_extension_version("1.0.0")
    .with_warning("Low memory");

    let json = serde_json::to_string_pretty(&manifest).unwrap();
    let parsed: ExtensionOutputManifest = serde_json::from_str(&json).unwrap();

    assert_eq!(manifest, parsed);
}

#[test]
fn test_determinism_level_serialization() {
    // Test round-trip serialization
    for level in [
        DeterminismLevel::ByteIdentical,
        DeterminismLevel::SemanticEquivalent,
        DeterminismLevel::NonDeterministic,
    ] {
        let json = serde_json::to_string(&level).unwrap();
        let parsed: DeterminismLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, parsed);
    }
}

#[test]
fn test_extension_interface_subprocess_serialization() {
    use std::collections::HashMap;

    let interface = ExtensionInterface::Subprocess {
        executable: "my-tool".to_string(),
        args: vec!["--spec".to_string(), "{spec_path}".to_string()],
        env: HashMap::from([("FOO".to_string(), "bar".to_string())]),
        timeout_seconds: 60,
    };

    let json = serde_json::to_string_pretty(&interface).unwrap();
    let parsed: ExtensionInterface = serde_json::from_str(&json).unwrap();

    assert_eq!(interface, parsed);
}

#[test]
fn test_extension_interface_wasm_serialization() {
    let interface = ExtensionInterface::Wasm {
        module_path: "extensions/custom.wasm".to_string(),
        memory_limit: 128 * 1024 * 1024,
        timeout_seconds: 30,
    };

    let json = serde_json::to_string_pretty(&interface).unwrap();
    let parsed: ExtensionInterface = serde_json::from_str(&json).unwrap();

    assert_eq!(interface, parsed);
}
