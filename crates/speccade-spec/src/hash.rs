//! Canonical hashing and seed derivation.
//!
//! This module implements the determinism policy for SpecCade:
//! - Spec canonicalization using RFC 8785 (JCS)
//! - BLAKE3 hashing for spec hashes
//! - Seed derivation for layers and variants

use crate::error::SpecError;
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
}
