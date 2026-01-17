//! Shared validation utilities for stdlib functions.
//!
//! Provides common validation patterns for parameter checking, range validation,
//! and enum validation with consistent error messages.

use starlark::values::Value;

/// Validates that a float is positive (> 0).
pub fn validate_positive(value: f64, function: &str, param: &str) -> Result<(), String> {
    if value <= 0.0 {
        Err(format!(
            "S103: {}(): '{}' must be positive, got {}",
            function, param, value
        ))
    } else {
        Ok(())
    }
}

/// Validates that a float is in the range [0.0, 1.0].
pub fn validate_unit_range(value: f64, function: &str, param: &str) -> Result<(), String> {
    if !(0.0..=1.0).contains(&value) {
        Err(format!(
            "S103: {}(): '{}' must be in range 0.0 to 1.0, got {}",
            function, param, value
        ))
    } else {
        Ok(())
    }
}

/// Validates that a float is in the range [-1.0, 1.0].
pub fn validate_pan_range(value: f64, function: &str, param: &str) -> Result<(), String> {
    if !(-1.0..=1.0).contains(&value) {
        Err(format!(
            "S103: {}(): '{}' must be in range -1.0 to 1.0, got {}",
            function, param, value
        ))
    } else {
        Ok(())
    }
}

/// Validates that a string is one of the allowed enum values.
pub fn validate_enum(value: &str, allowed: &[&str], function: &str, param: &str) -> Result<(), String> {
    if !allowed.contains(&value) {
        let suggestion = find_similar(value, allowed);
        let mut msg = format!(
            "S104: {}(): '{}' must be one of: {}",
            function,
            param,
            allowed.join(", ")
        );
        if let Some(s) = suggestion {
            msg.push_str(&format!(". Did you mean '{}'?", s));
        }
        Err(msg)
    } else {
        Ok(())
    }
}

/// Validates that a string is non-empty.
pub fn validate_non_empty(value: &str, function: &str, param: &str) -> Result<(), String> {
    if value.is_empty() {
        Err(format!(
            "S101: {}(): '{}' must not be empty",
            function, param
        ))
    } else {
        Ok(())
    }
}

/// Validates that an integer is positive (> 0).
pub fn validate_positive_int(value: i64, function: &str, param: &str) -> Result<(), String> {
    if value <= 0 {
        Err(format!(
            "S103: {}(): '{}' must be positive, got {}",
            function, param, value
        ))
    } else {
        Ok(())
    }
}

/// Tries to find a similar string from the allowed values for error suggestions.
fn find_similar<'a>(value: &str, allowed: &[&'a str]) -> Option<&'a str> {
    let value_lower = value.to_lowercase();

    // First, try exact prefix match
    for &candidate in allowed {
        let candidate_lower = candidate.to_lowercase();
        if candidate_lower.starts_with(&value_lower) || value_lower.starts_with(&candidate_lower) {
            return Some(candidate);
        }
    }

    // Find the candidate with the smallest Levenshtein distance (threshold <= 3)
    let mut best_match: Option<&'a str> = None;
    let mut best_distance = usize::MAX;

    for &candidate in allowed {
        let distance = levenshtein_distance(&value_lower, &candidate.to_lowercase());
        if distance <= 3 && distance < best_distance {
            best_distance = distance;
            best_match = Some(candidate);
        }
    }

    best_match
}

/// Simple Levenshtein distance implementation for short strings.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 { return n; }
    if n == 0 { return m; }

    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[m][n]
}

/// Extracts an optional float from a Starlark Value.
///
/// Reserved for future use by stdlib functions that need optional float parameters.
#[allow(dead_code)]
pub fn extract_optional_float(value: Value) -> Option<f64> {
    if value.is_none() {
        return None;
    }
    if let Some(f) = value.unpack_i32() {
        return Some(f as f64);
    }
    if value.get_type() == "float" {
        if let Ok(f) = value.to_str().parse::<f64>() {
            return Some(f);
        }
    }
    None
}

/// Extracts a required float from a Starlark Value.
///
/// Reserved for future use by stdlib functions that need float parameters from dynamic values.
#[allow(dead_code)]
pub fn extract_float(value: Value, function: &str, param: &str) -> Result<f64, String> {
    if let Some(f) = value.unpack_i32() {
        return Ok(f as f64);
    }
    if value.get_type() == "float" {
        if let Ok(f) = value.to_str().parse::<f64>() {
            return Ok(f);
        }
    }
    Err(format!(
        "S102: {}(): '{}' expected float, got {}",
        function, param, value.get_type()
    ))
}

/// Extracts an optional string from a Starlark Value.
///
/// Reserved for future use by stdlib functions that need optional string parameters.
#[allow(dead_code)]
pub fn extract_optional_string(value: Value) -> Option<String> {
    if value.is_none() {
        return None;
    }
    value.unpack_str().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_positive() {
        assert!(validate_positive(440.0, "oscillator", "frequency").is_ok());
        assert!(validate_positive(-440.0, "oscillator", "frequency").is_err());
        assert!(validate_positive(0.0, "oscillator", "frequency").is_err());
    }

    #[test]
    fn test_validate_unit_range() {
        assert!(validate_unit_range(0.5, "envelope", "sustain").is_ok());
        assert!(validate_unit_range(0.0, "envelope", "sustain").is_ok());
        assert!(validate_unit_range(1.0, "envelope", "sustain").is_ok());
        assert!(validate_unit_range(1.5, "envelope", "sustain").is_err());
        assert!(validate_unit_range(-0.1, "envelope", "sustain").is_err());
    }

    #[test]
    fn test_validate_pan_range() {
        assert!(validate_pan_range(0.0, "audio_layer", "pan").is_ok());
        assert!(validate_pan_range(-1.0, "audio_layer", "pan").is_ok());
        assert!(validate_pan_range(1.0, "audio_layer", "pan").is_ok());
        assert!(validate_pan_range(2.0, "audio_layer", "pan").is_err());
        assert!(validate_pan_range(-1.5, "audio_layer", "pan").is_err());
    }

    #[test]
    fn test_validate_enum() {
        let waveforms = &["sine", "square", "sawtooth", "triangle"];
        assert!(validate_enum("sine", waveforms, "oscillator", "waveform").is_ok());

        let err = validate_enum("sinwave", waveforms, "oscillator", "waveform");
        assert!(err.is_err());
        let msg = err.unwrap_err();
        assert!(msg.contains("S104"));
        assert!(msg.contains("sine"));  // Should suggest "sine"
    }

    #[test]
    fn test_find_similar() {
        let waveforms = &["sine", "square", "sawtooth", "triangle"];
        assert_eq!(find_similar("sinwave", waveforms), Some("sine"));
        assert_eq!(find_similar("sin", waveforms), Some("sine"));
        assert_eq!(find_similar("sqare", waveforms), Some("square"));
        assert_eq!(find_similar("xyz", waveforms), None);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("sine", "sine"), 0);
        assert_eq!(levenshtein_distance("sine", "sin"), 1);
        assert_eq!(levenshtein_distance("sinwave", "sine"), 3);
        assert_eq!(levenshtein_distance("sqare", "square"), 1);
    }
}
