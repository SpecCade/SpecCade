//! Tests for the backends module.

use super::*;
use speccade_spec::extension::{DeterminismLevel, ExtensionManifest};

// ============================================================================
// Registry Tests
// ============================================================================

#[test]
fn test_registry_new() {
    let registry = ExtensionRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_registry_register() {
    let mut registry = ExtensionRegistry::new();

    let manifest = ExtensionManifest::subprocess(
        "custom-texture-gen",
        "1.0.0",
        "custom-texture-gen",
        vec!["texture.custom_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );

    assert!(registry.register(manifest).is_ok());
    assert_eq!(registry.len(), 1);
    assert!(registry.get("custom-texture-gen").is_some());
}

#[test]
fn test_registry_recipe_lookup() {
    let mut registry = ExtensionRegistry::new();

    let manifest = ExtensionManifest::subprocess(
        "custom-texture-gen",
        "1.0.0",
        "custom-texture-gen",
        vec!["texture.custom_v1".to_string(), "texture.fancy_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );

    registry.register(manifest).unwrap();

    // Should find extension for both recipe kinds
    assert!(registry.has_extension_for("texture.custom_v1"));
    assert!(registry.has_extension_for("texture.fancy_v1"));
    assert!(!registry.has_extension_for("texture.other_v1"));

    let ext = registry.get_for_recipe("texture.custom_v1").unwrap();
    assert_eq!(ext.name, "custom-texture-gen");
}

#[test]
fn test_registry_duplicate_name() {
    let mut registry = ExtensionRegistry::new();

    let manifest1 = ExtensionManifest::subprocess(
        "my-extension",
        "1.0.0",
        "my-extension",
        vec!["texture.custom_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );

    let manifest2 = ExtensionManifest::subprocess(
        "my-extension", // Same name!
        "2.0.0",
        "my-extension-v2",
        vec!["texture.other_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );

    assert!(registry.register(manifest1).is_ok());

    let result = registry.register(manifest2);
    assert!(matches!(result, Err(RegistryError::AlreadyRegistered(_))));
}

#[test]
fn test_registry_recipe_conflict() {
    let mut registry = ExtensionRegistry::new();

    let manifest1 = ExtensionManifest::subprocess(
        "extension-a",
        "1.0.0",
        "extension-a",
        vec!["texture.custom_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );

    let manifest2 = ExtensionManifest::subprocess(
        "extension-b",
        "1.0.0",
        "extension-b",
        vec!["texture.custom_v1".to_string()], // Same recipe kind!
        DeterminismLevel::ByteIdentical,
    );

    assert!(registry.register(manifest1).is_ok());

    let result = registry.register(manifest2);
    assert!(matches!(result, Err(RegistryError::RecipeKindConflict { .. })));
}

#[test]
fn test_registry_unregister() {
    let mut registry = ExtensionRegistry::new();

    let manifest = ExtensionManifest::subprocess(
        "custom-texture-gen",
        "1.0.0",
        "custom-texture-gen",
        vec!["texture.custom_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );

    registry.register(manifest).unwrap();
    assert!(registry.has_extension_for("texture.custom_v1"));

    registry.unregister("custom-texture-gen").unwrap();
    assert!(!registry.has_extension_for("texture.custom_v1"));
    assert!(registry.get("custom-texture-gen").is_none());
}

#[test]
fn test_registry_list() {
    let mut registry = ExtensionRegistry::new();

    let manifest1 = ExtensionManifest::subprocess(
        "extension-a",
        "1.0.0",
        "extension-a",
        vec!["texture.custom_v1".to_string()],
        DeterminismLevel::ByteIdentical,
    );

    let manifest2 = ExtensionManifest::subprocess(
        "extension-b",
        "1.0.0",
        "extension-b",
        vec!["texture.other_v1".to_string()],
        DeterminismLevel::SemanticEquivalent,
    );

    registry.register(manifest1).unwrap();
    registry.register(manifest2).unwrap();

    let names: Vec<_> = registry.list().map(|m| m.name.as_str()).collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"extension-a"));
    assert!(names.contains(&"extension-b"));
}

// ============================================================================
// Subprocess Config Tests
// ============================================================================

#[test]
fn test_subprocess_config_default() {
    let config = SubprocessConfig::default();
    assert_eq!(config.default_timeout, 300);
    assert!(config.capture_stderr);
    assert!(config.verify_hashes);
    assert!(config.working_dir.is_none());
}

#[test]
fn test_subprocess_runner_new() {
    let runner = SubprocessRunner::new();
    assert_eq!(runner.config().default_timeout, 300);
}

// ============================================================================
// Hash Combination Tests
// ============================================================================

#[test]
fn test_combine_output_hashes_deterministic() {
    let hashes = vec![
        "a".repeat(64),
        "b".repeat(64),
        "c".repeat(64),
    ];

    let combined1 = subprocess::combine_output_hashes(&hashes);
    let combined2 = subprocess::combine_output_hashes(&hashes);

    assert_eq!(combined1, combined2, "hash combination should be deterministic");
}

#[test]
fn test_combine_output_hashes_order_independent() {
    let hashes1 = vec![
        "a".repeat(64),
        "b".repeat(64),
        "c".repeat(64),
    ];

    let hashes2 = vec![
        "c".repeat(64),
        "a".repeat(64),
        "b".repeat(64),
    ];

    let combined1 = subprocess::combine_output_hashes(&hashes1);
    let combined2 = subprocess::combine_output_hashes(&hashes2);

    assert_eq!(combined1, combined2, "hash combination should be order-independent");
}

#[test]
fn test_combine_output_hashes_different_inputs() {
    let hashes1 = vec!["a".repeat(64)];
    let hashes2 = vec!["b".repeat(64)];

    let combined1 = subprocess::combine_output_hashes(&hashes1);
    let combined2 = subprocess::combine_output_hashes(&hashes2);

    assert_ne!(combined1, combined2, "different inputs should produce different hashes");
}
