//! Integration tests for JSON canonicalization.
//!
//! These tests verify that:
//! 1. Canonicalization is idempotent: `canonicalize(canonicalize(x)) == canonicalize(x)`
//! 2. Same logical values produce identical canonical forms regardless of source ordering
//! 3. Edge cases (floats, unicode, control characters) are handled correctly
//!
//! These are acceptance criteria for Phase 3 of the Starlark migration.

use speccade_spec::hash::{canonicalize_json, canonical_spec_hash, canonical_value_hash};
use speccade_spec::{AssetType, OutputFormat, OutputSpec, Spec};

// ============================================================================
// Idempotence Tests
// ============================================================================

/// Test that canonicalization is idempotent with a realistic spec structure.
#[test]
fn test_canonicalization_idempotent_with_spec() {
    let spec = Spec::builder("canon-test-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .description("Test spec for canonicalization")
        .tag("test")
        .tag("canonicalization")
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .build();

    let value = spec.to_value().expect("spec should convert to value");

    let canon1 = canonicalize_json(&value).expect("first canonicalization should succeed");
    let reparsed: serde_json::Value =
        serde_json::from_str(&canon1).expect("canonical JSON should parse");
    let canon2 = canonicalize_json(&reparsed).expect("second canonicalization should succeed");

    assert_eq!(
        canon1, canon2,
        "canonicalization of a spec should be idempotent"
    );
}

/// Test that spec hash is stable across multiple calls.
#[test]
fn test_spec_hash_stability() {
    let spec = Spec::builder("hash-stability-01", AssetType::Texture)
        .license("MIT")
        .seed(12345)
        .description("A test texture")
        .output(OutputSpec::primary(
            OutputFormat::Png,
            "textures/test.png",
        ))
        .build();

    let hash1 = canonical_spec_hash(&spec).expect("first hash should succeed");
    let hash2 = canonical_spec_hash(&spec).expect("second hash should succeed");
    let hash3 = canonical_spec_hash(&spec).expect("third hash should succeed");

    assert_eq!(hash1, hash2);
    assert_eq!(hash2, hash3);
    assert_eq!(
        hash1.len(),
        64,
        "BLAKE3 hash should be 64 hex characters"
    );
}

// ============================================================================
// Order Independence Tests
// ============================================================================

/// Test that object key order doesn't affect the hash.
#[test]
fn test_key_order_independence() {
    // Create two JSON values with the same content but different key order
    let json1: serde_json::Value = serde_json::from_str(
        r#"{
            "zzz": "last",
            "aaa": "first",
            "mmm": "middle"
        }"#,
    )
    .unwrap();

    let json2: serde_json::Value = serde_json::from_str(
        r#"{
            "aaa": "first",
            "mmm": "middle",
            "zzz": "last"
        }"#,
    )
    .unwrap();

    let canon1 = canonicalize_json(&json1).unwrap();
    let canon2 = canonicalize_json(&json2).unwrap();

    assert_eq!(canon1, canon2, "key order should not affect canonical form");

    // Also verify the hash is identical
    let hash1 = canonical_value_hash(&json1).unwrap();
    let hash2 = canonical_value_hash(&json2).unwrap();

    assert_eq!(hash1, hash2, "key order should not affect hash");
}

/// Test that nested object key order doesn't affect the hash.
#[test]
fn test_nested_key_order_independence() {
    let json1: serde_json::Value = serde_json::from_str(
        r#"{
            "outer": {
                "zzz": {"c": 3, "a": 1, "b": 2},
                "aaa": {"y": 25, "z": 26, "x": 24}
            }
        }"#,
    )
    .unwrap();

    let json2: serde_json::Value = serde_json::from_str(
        r#"{
            "outer": {
                "aaa": {"x": 24, "y": 25, "z": 26},
                "zzz": {"a": 1, "b": 2, "c": 3}
            }
        }"#,
    )
    .unwrap();

    let hash1 = canonical_value_hash(&json1).unwrap();
    let hash2 = canonical_value_hash(&json2).unwrap();

    assert_eq!(
        hash1, hash2,
        "nested key order should not affect hash"
    );
}

// ============================================================================
// Round-trip Tests
// ============================================================================

/// Test that a complex spec survives a JSON round-trip with identical hash.
#[test]
fn test_spec_roundtrip_hash_preservation() {
    let original_spec = Spec::builder("roundtrip-01", AssetType::Music)
        .license("CC-BY-4.0")
        .seed(999)
        .description("A test music file with various metadata")
        .tag("electronic")
        .tag("ambient")
        .output(OutputSpec::primary(OutputFormat::Xm, "music/ambient.xm"))
        .build();

    // Get the original hash
    let original_hash = canonical_spec_hash(&original_spec).unwrap();

    // Serialize to JSON
    let json_string = original_spec.to_json_pretty().unwrap();

    // Parse back
    let restored_spec = Spec::from_json(&json_string).unwrap();

    // Get the restored hash
    let restored_hash = canonical_spec_hash(&restored_spec).unwrap();

    assert_eq!(
        original_hash, restored_hash,
        "spec hash should survive JSON round-trip"
    );
}

// ============================================================================
// Edge Case: Float Handling
// ============================================================================

/// Test that integer-like floats are normalized consistently.
#[test]
fn test_float_normalization() {
    // 1.0 and 1 should produce the same canonical form
    let json_float: serde_json::Value = serde_json::from_str(r#"{"x": 1.0}"#).unwrap();
    let json_int: serde_json::Value = serde_json::from_str(r#"{"x": 1}"#).unwrap();

    let canon_float = canonicalize_json(&json_float).unwrap();
    let canon_int = canonicalize_json(&json_int).unwrap();

    // Both should canonicalize to the same form
    assert_eq!(canon_float, canon_int, "1.0 and 1 should have same canonical form");
}

/// Test that zero is handled consistently.
#[test]
fn test_zero_normalization() {
    let json_zero: serde_json::Value = serde_json::from_str(r#"{"x": 0}"#).unwrap();
    let json_zero_float: serde_json::Value = serde_json::from_str(r#"{"x": 0.0}"#).unwrap();

    let canon1 = canonicalize_json(&json_zero).unwrap();
    let canon2 = canonicalize_json(&json_zero_float).unwrap();

    assert_eq!(canon1, canon2, "0 and 0.0 should have same canonical form");
    assert_eq!(canon1, r#"{"x":0}"#);
}

// ============================================================================
// Edge Case: Unicode Handling
// ============================================================================

/// Test that unicode strings are preserved correctly.
#[test]
fn test_unicode_preservation() {
    let json = serde_json::json!({
        "chinese": "\u{4e2d}\u{6587}",  // Chinese characters
        "emoji": "\u{1F600}",             // Emoji (grinning face)
        "mixed": "Hello \u{4e16}\u{754c}" // "Hello World" with Chinese characters
    });

    let canon = canonicalize_json(&json).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();

    assert_eq!(
        reparsed["chinese"].as_str().unwrap(),
        "\u{4e2d}\u{6587}",
        "Chinese characters should be preserved"
    );
    assert_eq!(
        reparsed["emoji"].as_str().unwrap(),
        "\u{1F600}",
        "Emoji should be preserved"
    );
}

// ============================================================================
// Edge Case: Empty Structures
// ============================================================================

/// Test that empty objects and arrays are handled correctly.
#[test]
fn test_empty_structures() {
    let json = serde_json::json!({
        "empty_object": {},
        "empty_array": [],
        "nested_empty": {
            "obj": {},
            "arr": []
        }
    });

    let canon = canonicalize_json(&json).unwrap();

    // Verify the structure
    assert!(canon.contains(r#""empty_array":[]"#));
    assert!(canon.contains(r#""empty_object":{}"#));

    // Verify idempotence
    let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
    let canon2 = canonicalize_json(&reparsed).unwrap();
    assert_eq!(canon, canon2);
}

// ============================================================================
// Edge Case: Special String Values
// ============================================================================

/// Test that special string values are escaped correctly.
#[test]
fn test_string_escaping() {
    let json = serde_json::json!({
        "newline": "line1\nline2",
        "tab": "col1\tcol2",
        "quote": "she said \"hello\"",
        "backslash": "path\\to\\file"
    });

    let canon = canonicalize_json(&json).unwrap();

    // Verify the escaping
    assert!(canon.contains(r#"\n"#), "newline should be escaped");
    assert!(canon.contains(r#"\t"#), "tab should be escaped");
    assert!(canon.contains(r#"\""#), "quote should be escaped");
    assert!(canon.contains(r#"\\"#), "backslash should be escaped");

    // Verify round-trip
    let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
    assert_eq!(
        reparsed["newline"].as_str().unwrap(),
        "line1\nline2"
    );
    assert_eq!(
        reparsed["quote"].as_str().unwrap(),
        "she said \"hello\""
    );
}

// ============================================================================
// Determinism Tests
// ============================================================================

/// Test that the same spec always produces the same hash.
#[test]
fn test_deterministic_hashing() {
    // Run 100 iterations to ensure determinism
    let mut hashes = Vec::new();

    for _ in 0..100 {
        let spec = Spec::builder("determinism-test-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .build();

        let hash = canonical_spec_hash(&spec).unwrap();
        hashes.push(hash);
    }

    // All hashes should be identical
    let first = &hashes[0];
    for (i, hash) in hashes.iter().enumerate() {
        assert_eq!(
            hash, first,
            "hash at iteration {} should match first hash",
            i
        );
    }
}

/// Test that changing any field changes the hash.
#[test]
fn test_hash_sensitivity() {
    let base_spec = Spec::builder("sensitivity-test-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .build();

    let base_hash = canonical_spec_hash(&base_spec).unwrap();

    // Change seed
    let spec_with_different_seed = Spec::builder("sensitivity-test-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(43) // Different seed
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .build();

    let different_seed_hash = canonical_spec_hash(&spec_with_different_seed).unwrap();
    assert_ne!(
        base_hash, different_seed_hash,
        "different seed should produce different hash"
    );

    // Change asset_id
    let spec_with_different_id = Spec::builder("sensitivity-test-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .build();

    let different_id_hash = canonical_spec_hash(&spec_with_different_id).unwrap();
    assert_ne!(
        base_hash, different_id_hash,
        "different asset_id should produce different hash"
    );

    // Change license
    let spec_with_different_license = Spec::builder("sensitivity-test-01", AssetType::Audio)
        .license("MIT") // Different license
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .build();

    let different_license_hash = canonical_spec_hash(&spec_with_different_license).unwrap();
    assert_ne!(
        base_hash, different_license_hash,
        "different license should produce different hash"
    );
}
