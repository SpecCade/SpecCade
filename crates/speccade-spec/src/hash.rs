//! Canonical hashing and seed derivation.
//!
//! This module implements the determinism policy for SpecCade:
//! - Spec canonicalization using RFC 8785 (JCS)
//! - BLAKE3 hashing for spec hashes
//! - Seed derivation for layers and variants

use crate::error::SpecError;
use crate::recipe::Recipe;
use crate::spec::Spec;

/// Computes the canonical BLAKE3 hash of a spec.
///
/// The hash is computed as:
/// ```text
/// spec_hash = hex(BLAKE3(JCS(spec_json)))
/// ```
///
/// Where JCS is JSON Canonicalization Scheme per RFC 8785.
///
/// # Arguments
/// * `spec` - The spec to hash
///
/// # Returns
/// * A 64-character lowercase hexadecimal string
///
/// # Example
/// ```
/// use speccade_spec::{Spec, AssetType, OutputSpec, OutputFormat};
/// use speccade_spec::hash::canonical_spec_hash;
///
/// let spec = Spec::builder("test-01", AssetType::Audio)
///     .license("CC0-1.0")
///     .seed(42)
///     .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
///     .build();
///
/// let hash = canonical_spec_hash(&spec).unwrap();
/// assert_eq!(hash.len(), 64);
/// ```
pub fn canonical_spec_hash(spec: &Spec) -> Result<String, SpecError> {
    let value = spec.to_value()?;
    canonical_value_hash(&value)
}

/// Computes the canonical BLAKE3 hash of a recipe.
///
/// This is useful for caching and provenance: it fingerprints only the generator inputs
/// (recipe kind + params), independent of contract-only fields like description/tags.
pub fn canonical_recipe_hash(recipe: &Recipe) -> Result<String, SpecError> {
    let value = serde_json::to_value(recipe)?;
    canonical_value_hash(&value)
}

/// Computes the canonical BLAKE3 hash of a JSON value.
///
/// # Arguments
/// * `value` - The JSON value to hash
///
/// # Returns
/// * A 64-character lowercase hexadecimal string
pub fn canonical_value_hash(value: &serde_json::Value) -> Result<String, SpecError> {
    let canonical = canonicalize_json(value)?;
    let hash = blake3::hash(canonical.as_bytes());
    Ok(hash.to_hex().to_string())
}

/// Canonicalizes a JSON value according to RFC 8785 (JCS).
///
/// This produces a deterministic JSON string where:
/// - Object keys are sorted lexicographically
/// - No whitespace between tokens
/// - Numbers are formatted per IEEE 754
/// - Strings use minimal escaping
///
/// # Arguments
/// * `value` - The JSON value to canonicalize
///
/// # Returns
/// * A canonical JSON string
pub fn canonicalize_json(value: &serde_json::Value) -> Result<String, SpecError> {
    Ok(canonicalize_value(value))
}

/// Internal canonicalization function.
fn canonicalize_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => format_jcs_number(n),
        serde_json::Value::String(s) => format_jcs_string(s),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(canonicalize_value).collect();
            format!("[{}]", items.join(","))
        }
        serde_json::Value::Object(obj) => {
            // Sort keys lexicographically
            let mut sorted_keys: Vec<&String> = obj.keys().collect();
            sorted_keys.sort();

            let pairs: Vec<String> = sorted_keys
                .iter()
                .map(|k| {
                    let v = obj.get(*k).unwrap();
                    format!("{}:{}", format_jcs_string(k), canonicalize_value(v))
                })
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
    }
}

/// Formats a number according to JCS rules.
fn format_jcs_number(n: &serde_json::Number) -> String {
    if let Some(i) = n.as_i64() {
        return i.to_string();
    }
    if let Some(u) = n.as_u64() {
        return u.to_string();
    }
    if let Some(f) = n.as_f64() {
        // JCS number formatting rules:
        // - No leading zeros (except for 0.x)
        // - No trailing zeros after decimal
        // - No + sign for exponent
        // - Lowercase 'e' for exponent
        if f.is_nan() || f.is_infinite() {
            return "null".to_string(); // JCS treats these as null
        }
        if f == 0.0 {
            return "0".to_string();
        }
        if f.fract() == 0.0 && f.abs() < 1e15 {
            // Integer-like float
            return format!("{}", f as i64);
        }
        // Use general formatting
        let s = format!("{}", f);
        // Remove unnecessary trailing zeros and decimal points
        if s.contains('.') && !s.contains('e') && !s.contains('E') {
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            return trimmed.to_string();
        }
        s
    } else {
        "null".to_string()
    }
}

/// Formats a string according to JCS rules.
fn format_jcs_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c < '\x20' => {
                // Control characters
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

/// Derives a seed for a specific layer from the base seed.
///
/// This uses BLAKE3 to derive a deterministic seed for each layer,
/// ensuring independent random streams.
///
/// ```text
/// layer_seed = truncate_u32(BLAKE3(base_seed || layer_index))
/// ```
///
/// # Arguments
/// * `base_seed` - The spec's base seed (u32)
/// * `layer_index` - The 0-indexed layer number
///
/// # Returns
/// * A derived u32 seed for the layer
///
/// # Example
/// ```
/// use speccade_spec::hash::derive_layer_seed;
///
/// let base = 42u32;
/// let layer0_seed = derive_layer_seed(base, 0);
/// let layer1_seed = derive_layer_seed(base, 1);
/// assert_ne!(layer0_seed, layer1_seed);
/// ```
pub fn derive_layer_seed(base_seed: u32, layer_index: u32) -> u32 {
    // Concatenate base_seed and layer_index as little-endian bytes
    let mut input = Vec::with_capacity(8);
    input.extend_from_slice(&base_seed.to_le_bytes());
    input.extend_from_slice(&layer_index.to_le_bytes());

    // Hash with BLAKE3
    let hash = blake3::hash(&input);

    // Truncate to u32 (first 4 bytes, little-endian)
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    u32::from_le_bytes(bytes)
}

/// Derives a seed for a specific variant from the base seed.
///
/// This uses BLAKE3 to derive a deterministic seed for each variant,
/// ensuring independent random streams based on the variant identifier.
///
/// ```text
/// variant_seed = truncate_u32(BLAKE3(base_seed || variant_id))
/// ```
///
/// # Arguments
/// * `base_seed` - The spec's base seed (u32)
/// * `variant_id` - The variant identifier string
///
/// # Returns
/// * A derived u32 seed for the variant
///
/// # Example
/// ```
/// use speccade_spec::hash::derive_variant_seed;
///
/// let base = 42u32;
/// let soft_seed = derive_variant_seed(base, "soft");
/// let hard_seed = derive_variant_seed(base, "hard");
/// assert_ne!(soft_seed, hard_seed);
/// ```
pub fn derive_variant_seed(base_seed: u32, variant_id: &str) -> u32 {
    // Concatenate base_seed (as little-endian bytes) and variant_id (as UTF-8)
    let mut input = Vec::with_capacity(4 + variant_id.len());
    input.extend_from_slice(&base_seed.to_le_bytes());
    input.extend_from_slice(variant_id.as_bytes());

    // Hash with BLAKE3
    let hash = blake3::hash(&input);

    // Truncate to u32 (first 4 bytes, little-endian)
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    u32::from_le_bytes(bytes)
}

/// Derives a seed for a specific *spec variant* from the base seed, variant id, and seed offset.
///
/// This is intended for expanding `Spec.variants[]` in the CLI and other tooling.
/// It incorporates both the human-readable `variant_id` and the numeric `seed_offset`
/// into the derived seed to support both named variants and deterministic seed sweeps.
///
/// ```text
/// variant_seed = truncate_u32(BLAKE3(base_seed || seed_offset || variant_id))
/// ```
///
/// Where:
/// - `base_seed` is encoded as 4 little-endian bytes
/// - `seed_offset` is encoded as 4 little-endian bytes
/// - `variant_id` is the UTF-8 encoded variant identifier string
pub fn derive_variant_spec_seed(base_seed: u32, seed_offset: u32, variant_id: &str) -> u32 {
    let mut input = Vec::with_capacity(8 + variant_id.len());
    input.extend_from_slice(&base_seed.to_le_bytes());
    input.extend_from_slice(&seed_offset.to_le_bytes());
    input.extend_from_slice(variant_id.as_bytes());

    let hash = blake3::hash(&input);
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    u32::from_le_bytes(bytes)
}

/// Computes a BLAKE3 hash of arbitrary data.
///
/// # Arguments
/// * `data` - The data to hash
///
/// # Returns
/// * A 64-character lowercase hexadecimal string
pub fn blake3_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Computes a BLAKE3 hash of a string.
///
/// # Arguments
/// * `s` - The string to hash
///
/// # Returns
/// * A 64-character lowercase hexadecimal string
pub fn blake3_hash_str(s: &str) -> String {
    blake3_hash(s.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{OutputFormat, OutputSpec};
    use crate::spec::AssetType;

    #[test]
    fn test_canonical_spec_hash() {
        let spec = Spec::builder("test-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let hash = canonical_spec_hash(&spec).unwrap();
        assert_eq!(hash.len(), 64);

        // Same spec should produce same hash
        let hash2 = canonical_spec_hash(&spec).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_stability() {
        // This test ensures the hash doesn't change between runs
        let spec = Spec::builder("laser-blast-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .description("Test laser blast")
            .output(OutputSpec::primary(
                OutputFormat::Wav,
                "sounds/laser_blast.wav",
            ))
            .build();

        let hash1 = canonical_spec_hash(&spec).unwrap();
        let hash2 = canonical_spec_hash(&spec).unwrap();
        assert_eq!(hash1, hash2, "hash should be stable across calls");
    }

    #[test]
    fn test_different_specs_different_hashes() {
        let spec1 = Spec::builder("test-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let spec2 = Spec::builder("test-02", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let hash1 = canonical_spec_hash(&spec1).unwrap();
        let hash2 = canonical_spec_hash(&spec2).unwrap();
        assert_ne!(hash1, hash2, "different specs should have different hashes");
    }

    #[test]
    fn test_seed_change_changes_hash() {
        let spec1 = Spec::builder("test-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let spec2 = Spec::builder("test-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(43)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let hash1 = canonical_spec_hash(&spec1).unwrap();
        let hash2 = canonical_spec_hash(&spec2).unwrap();
        assert_ne!(
            hash1, hash2,
            "different seeds should produce different hashes"
        );
    }

    #[test]
    fn test_canonicalize_json_object_ordering() {
        let json1: serde_json::Value = serde_json::from_str(r#"{"b": 1, "a": 2}"#).unwrap();
        let json2: serde_json::Value = serde_json::from_str(r#"{"a": 2, "b": 1}"#).unwrap();

        let canonical1 = canonicalize_json(&json1).unwrap();
        let canonical2 = canonicalize_json(&json2).unwrap();

        assert_eq!(canonical1, canonical2);
        assert_eq!(canonical1, r#"{"a":2,"b":1}"#);
    }

    #[test]
    fn test_canonicalize_json_nested() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"z": [1, 2, 3], "a": {"c": true, "b": false}}"#).unwrap();

        let canonical = canonicalize_json(&json).unwrap();
        assert_eq!(canonical, r#"{"a":{"b":false,"c":true},"z":[1,2,3]}"#);
    }

    #[test]
    fn test_canonicalize_json_strings() {
        let json: serde_json::Value = serde_json::from_str(r#"{"text": "hello\nworld"}"#).unwrap();

        let canonical = canonicalize_json(&json).unwrap();
        assert_eq!(canonical, r#"{"text":"hello\nworld"}"#);
    }

    #[test]
    fn test_derive_layer_seed() {
        let base_seed = 42u32;

        let seed0 = derive_layer_seed(base_seed, 0);
        let seed1 = derive_layer_seed(base_seed, 1);
        let seed2 = derive_layer_seed(base_seed, 2);

        // All seeds should be different
        assert_ne!(seed0, seed1);
        assert_ne!(seed1, seed2);
        assert_ne!(seed0, seed2);

        // Same inputs should produce same outputs (determinism)
        assert_eq!(derive_layer_seed(base_seed, 0), seed0);
        assert_eq!(derive_layer_seed(base_seed, 1), seed1);
    }

    #[test]
    fn test_derive_variant_seed() {
        let base_seed = 42u32;

        let soft_seed = derive_variant_seed(base_seed, "soft");
        let hard_seed = derive_variant_seed(base_seed, "hard");

        // Different variants should produce different seeds
        assert_ne!(soft_seed, hard_seed);

        // Same inputs should produce same outputs (determinism)
        assert_eq!(derive_variant_seed(base_seed, "soft"), soft_seed);
        assert_eq!(derive_variant_seed(base_seed, "hard"), hard_seed);
    }

    #[test]
    fn test_derive_layer_seed_different_base() {
        let seed_42_0 = derive_layer_seed(42, 0);
        let seed_43_0 = derive_layer_seed(43, 0);

        // Different base seeds should produce different layer seeds
        assert_ne!(seed_42_0, seed_43_0);
    }

    #[test]
    fn test_derive_variant_seed_different_base() {
        let seed_42_soft = derive_variant_seed(42, "soft");
        let seed_43_soft = derive_variant_seed(43, "soft");

        // Different base seeds should produce different variant seeds
        assert_ne!(seed_42_soft, seed_43_soft);
    }

    #[test]
    fn test_derive_variant_spec_seed() {
        let base_seed = 42u32;

        let a0 = derive_variant_spec_seed(base_seed, 0, "a");
        let a1 = derive_variant_spec_seed(base_seed, 1, "a");
        let b0 = derive_variant_spec_seed(base_seed, 0, "b");

        assert_ne!(a0, a1, "different offsets should produce different seeds");
        assert_ne!(a0, b0, "different ids should produce different seeds");

        // Determinism
        assert_eq!(a0, derive_variant_spec_seed(base_seed, 0, "a"));
    }

    #[test]
    fn test_blake3_hash() {
        let data = b"hello world";
        let hash = blake3_hash(data);
        assert_eq!(hash.len(), 64);

        // Known BLAKE3 hash for "hello world"
        // Verified with: echo -n "hello world" | b3sum
        assert_eq!(
            hash,
            "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
    }

    #[test]
    fn test_blake3_hash_str() {
        let hash = blake3_hash_str("hello world");
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
    }

    #[test]
    fn test_format_jcs_number() {
        assert_eq!(format_jcs_number(&serde_json::Number::from(42)), "42");
        assert_eq!(format_jcs_number(&serde_json::Number::from(0)), "0");
        assert_eq!(format_jcs_number(&serde_json::Number::from(-1)), "-1");
    }

    #[test]
    fn test_format_jcs_string() {
        assert_eq!(format_jcs_string("hello"), "\"hello\"");
        assert_eq!(format_jcs_string("hello\nworld"), "\"hello\\nworld\"");
        assert_eq!(format_jcs_string("quote\"here"), "\"quote\\\"here\"");
        assert_eq!(format_jcs_string("back\\slash"), "\"back\\\\slash\"");
    }

    // ========================================================================
    // Canonicalization Idempotence Tests
    // ========================================================================

    #[test]
    fn test_canonicalization_idempotent_simple() {
        let value = serde_json::json!({"b": 1, "a": 2});
        let canon1 = canonicalize_json(&value).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&canon1).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();

        assert_eq!(canon1, canon2, "canonicalization should be idempotent");
    }

    #[test]
    fn test_canonicalization_idempotent_nested() {
        let value = serde_json::json!({
            "z": {"b": 1, "a": 2},
            "y": [3, 2, 1],
            "x": {"nested": {"deep": true}}
        });

        let canon1 = canonicalize_json(&value).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&canon1).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();

        assert_eq!(
            canon1, canon2,
            "nested canonicalization should be idempotent"
        );
    }

    #[test]
    fn test_canonicalization_idempotent_with_arrays() {
        let value = serde_json::json!({
            "items": [
                {"id": "b", "value": 2},
                {"id": "a", "value": 1}
            ],
            "nested_arrays": [[1, 2], [3, 4]]
        });

        let canon1 = canonicalize_json(&value).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&canon1).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();

        assert_eq!(
            canon1, canon2,
            "array canonicalization should be idempotent"
        );
    }

    #[test]
    fn test_canonicalization_idempotent_empty_structures() {
        let value = serde_json::json!({
            "empty_object": {},
            "empty_array": [],
            "nested_empty": {"a": {}, "b": []}
        });

        let canon1 = canonicalize_json(&value).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&canon1).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();

        assert_eq!(
            canon1, canon2,
            "empty structure canonicalization should be idempotent"
        );
    }

    // ========================================================================
    // Float Edge Case Tests
    // ========================================================================

    #[test]
    fn test_canonicalization_float_zero() {
        let value = serde_json::json!({"x": 0.0});
        let canon = canonicalize_json(&value).unwrap();
        assert_eq!(canon, r#"{"x":0}"#);

        // Verify idempotence
        let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();
        assert_eq!(canon, canon2);
    }

    #[test]
    fn test_canonicalization_float_integer_like() {
        let test_cases = vec![
            (1.0, "1"),
            (42.0, "42"),
            (-1.0, "-1"),
            (1000000.0, "1000000"),
        ];

        for (input, expected) in test_cases {
            let value = serde_json::json!({"x": input});
            let canon = canonicalize_json(&value).unwrap();
            assert_eq!(
                canon,
                format!(r#"{{"x":{}}}"#, expected),
                "Failed for {}",
                input
            );

            // Verify idempotence
            let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
            let canon2 = canonicalize_json(&reparsed).unwrap();
            assert_eq!(canon, canon2);
        }
    }

    #[test]
    fn test_canonicalization_float_decimals() {
        let value = serde_json::json!({"x": 0.5, "y": 0.125});
        let canon = canonicalize_json(&value).unwrap();

        // Verify it parses back correctly
        let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
        let x = reparsed["x"].as_f64().unwrap();
        let y = reparsed["y"].as_f64().unwrap();

        assert!((x - 0.5).abs() < f64::EPSILON);
        assert!((y - 0.125).abs() < f64::EPSILON);

        // Verify idempotence
        let canon2 = canonicalize_json(&reparsed).unwrap();
        assert_eq!(canon, canon2);
    }

    #[test]
    fn test_canonicalization_float_large_values() {
        let value = serde_json::json!({"x": 1e10, "y": 1e15});
        let canon = canonicalize_json(&value).unwrap();

        // Verify idempotence
        let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();
        assert_eq!(canon, canon2);
    }

    #[test]
    fn test_canonicalization_float_nan_infinity() {
        // NaN and Infinity should fail to create serde_json::Number
        // because serde_json doesn't allow NaN/Infinity
        let n = serde_json::Number::from_f64(f64::NAN);
        let inf = serde_json::Number::from_f64(f64::INFINITY);
        let neg_inf = serde_json::Number::from_f64(f64::NEG_INFINITY);

        // These become None because serde_json doesn't allow NaN/Infinity
        assert!(n.is_none(), "NaN should not be representable");
        assert!(inf.is_none(), "Infinity should not be representable");
        assert!(
            neg_inf.is_none(),
            "Negative infinity should not be representable"
        );
    }

    // ========================================================================
    // String Edge Case Tests
    // ========================================================================

    #[test]
    fn test_canonicalization_string_empty() {
        let value = serde_json::json!({"x": ""});
        let canon = canonicalize_json(&value).unwrap();
        assert_eq!(canon, r#"{"x":""}"#);

        // Verify idempotence
        let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();
        assert_eq!(canon, canon2);
    }

    #[test]
    fn test_canonicalization_string_escape_sequences() {
        let value = serde_json::json!({
            "newline": "a\nb",
            "tab": "a\tb",
            "quote": "a\"b",
            "backslash": "a\\b"
        });

        let canon = canonicalize_json(&value).unwrap();

        // Verify it contains proper escapes (keys are sorted)
        assert!(canon.contains(r#""backslash":"a\\b""#));
        assert!(canon.contains(r#""newline":"a\nb""#));
        assert!(canon.contains(r#""quote":"a\"b""#));
        assert!(canon.contains(r#""tab":"a\tb""#));

        // Verify round-trip
        let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();
        assert_eq!(canon, canon2);
    }

    #[test]
    fn test_canonicalization_string_unicode() {
        // Test with various unicode characters
        let value = serde_json::json!({
            "chinese": "\u{4e2d}\u{6587}",
            "japanese": "\u{3042}\u{3044}\u{3046}"
        });

        let canon = canonicalize_json(&value).unwrap();

        // Verify round-trip
        let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
        assert_eq!(reparsed["chinese"].as_str().unwrap(), "\u{4e2d}\u{6587}");

        // Verify idempotence
        let canon2 = canonicalize_json(&reparsed).unwrap();
        assert_eq!(canon, canon2);
    }

    #[test]
    fn test_canonicalization_string_control_characters() {
        let value = serde_json::json!({"x": "\x00\x01\x1f"});
        let canon = canonicalize_json(&value).unwrap();

        // Should escape control characters as \uXXXX
        assert!(canon.contains("\\u0000"), "should escape null character");
        assert!(canon.contains("\\u0001"), "should escape SOH character");
        assert!(canon.contains("\\u001f"), "should escape unit separator");

        // Verify round-trip
        let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();
        assert_eq!(canon, canon2);
    }

    // ========================================================================
    // Object Key Ordering Tests
    // ========================================================================

    #[test]
    fn test_canonicalization_key_ordering_simple() {
        let json1: serde_json::Value = serde_json::from_str(r#"{"z": 1, "a": 2, "m": 3}"#).unwrap();
        let json2: serde_json::Value = serde_json::from_str(r#"{"a": 2, "z": 1, "m": 3}"#).unwrap();
        let json3: serde_json::Value = serde_json::from_str(r#"{"m": 3, "z": 1, "a": 2}"#).unwrap();

        let canon1 = canonicalize_json(&json1).unwrap();
        let canon2 = canonicalize_json(&json2).unwrap();
        let canon3 = canonicalize_json(&json3).unwrap();

        // All should produce the same canonical form
        assert_eq!(canon1, canon2);
        assert_eq!(canon2, canon3);
        assert_eq!(canon1, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn test_canonicalization_key_ordering_nested() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
            "z": {"b": 1, "a": 2},
            "a": {"d": 3, "c": 4}
        }"#,
        )
        .unwrap();

        let canon = canonicalize_json(&json).unwrap();

        // Keys should be sorted at all levels
        assert_eq!(canon, r#"{"a":{"c":4,"d":3},"z":{"a":2,"b":1}}"#);
    }

    // ========================================================================
    // Array Preservation Tests
    // ========================================================================

    #[test]
    fn test_canonicalization_preserves_array_order() {
        let value = serde_json::json!({"items": [3, 1, 2]});
        let canon = canonicalize_json(&value).unwrap();

        // Array order should be preserved
        assert_eq!(canon, r#"{"items":[3,1,2]}"#);
    }

    #[test]
    fn test_canonicalization_preserves_object_array_order() {
        let value = serde_json::json!({
            "items": [
                {"id": "third", "value": 3},
                {"id": "first", "value": 1},
                {"id": "second", "value": 2}
            ]
        });

        let canon = canonicalize_json(&value).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();

        // Array order preserved (object keys sorted within each object)
        assert_eq!(reparsed["items"][0]["id"].as_str().unwrap(), "third");
        assert_eq!(reparsed["items"][1]["id"].as_str().unwrap(), "first");
        assert_eq!(reparsed["items"][2]["id"].as_str().unwrap(), "second");
    }

    // ========================================================================
    // Deep Nesting Tests
    // ========================================================================

    #[test]
    fn test_canonicalization_deep_nesting() {
        // Test 5+ levels of nesting
        let value = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "value": 42
                            }
                        }
                    }
                }
            }
        });

        let canon1 = canonicalize_json(&value).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&canon1).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();

        assert_eq!(
            canon1, canon2,
            "deep nesting canonicalization should be idempotent"
        );

        // Verify the value is accessible
        assert_eq!(
            reparsed["level1"]["level2"]["level3"]["level4"]["level5"]["value"]
                .as_i64()
                .unwrap(),
            42
        );
    }

    #[test]
    fn test_canonicalization_mixed_nesting() {
        // Mix of arrays and objects
        let value = serde_json::json!({
            "data": [
                {
                    "nested": [1, 2, {"inner": true}]
                },
                [
                    {"key": "value"},
                    [3, 4, 5]
                ]
            ]
        });

        let canon1 = canonicalize_json(&value).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&canon1).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();

        assert_eq!(
            canon1, canon2,
            "mixed nesting canonicalization should be idempotent"
        );
    }
}
