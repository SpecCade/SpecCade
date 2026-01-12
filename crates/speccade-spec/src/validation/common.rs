//! Common validation utilities shared across backends.
//!
//! This module provides reusable validation functions for common parameter types
//! like resolutions, unit intervals, and positive values.

use std::fmt;

/// Error type for common validation failures.
#[derive(Debug, Clone, PartialEq)]
pub struct CommonValidationError {
    /// Human-readable error message.
    pub message: String,
}

impl CommonValidationError {
    /// Creates a new validation error with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CommonValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CommonValidationError {}

/// Validate that resolution is positive and doesn't overflow.
///
/// # Arguments
/// * `width` - Width in pixels
/// * `height` - Height in pixels
///
/// # Returns
/// * `Ok(())` if resolution is valid
/// * `Err(CommonValidationError)` if resolution is zero or would overflow
///
/// # Example
/// ```
/// use speccade_spec::validation::common::validate_resolution;
///
/// assert!(validate_resolution(1024, 1024).is_ok());
/// assert!(validate_resolution(0, 100).is_err());
/// ```
pub fn validate_resolution(width: u32, height: u32) -> Result<(), CommonValidationError> {
    // Practical cap: these generators allocate multiple `width * height` buffers in memory.
    // Keeping this bounded prevents accidental OOMs from malformed specs.
    const MAX_DIMENSION: u32 = 4096;
    const MAX_PIXELS: u64 = (MAX_DIMENSION as u64) * (MAX_DIMENSION as u64);

    if width == 0 || height == 0 {
        return Err(CommonValidationError::new(format!(
            "resolution must be at least 1x1, got [{}, {}]",
            width, height
        )));
    }

    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(CommonValidationError::new(format!(
            "resolution is too large: max is {}x{}, got [{}, {}]",
            MAX_DIMENSION, MAX_DIMENSION, width, height
        )));
    }

    let pixels = (width as u64) * (height as u64);
    if pixels > MAX_PIXELS {
        return Err(CommonValidationError::new(format!(
            "resolution is too large: max is {} pixels, got {}",
            MAX_PIXELS, pixels
        )));
    }

    Ok(())
}

/// Validate that a value is in [0, 1] (the unit interval).
///
/// # Arguments
/// * `name` - Name of the parameter (for error messages)
/// * `value` - Value to validate
///
/// # Returns
/// * `Ok(())` if value is in [0, 1]
/// * `Err(CommonValidationError)` if value is outside range or not finite
///
/// # Example
/// ```
/// use speccade_spec::validation::common::validate_unit_interval;
///
/// assert!(validate_unit_interval("opacity", 0.5).is_ok());
/// assert!(validate_unit_interval("opacity", 1.5).is_err());
/// ```
pub fn validate_unit_interval(name: &str, value: f64) -> Result<(), CommonValidationError> {
    if !value.is_finite() {
        return Err(CommonValidationError::new(format!(
            "{} must be finite, got {}",
            name, value
        )));
    }
    if !(0.0..=1.0).contains(&value) {
        return Err(CommonValidationError::new(format!(
            "{} must be in [0, 1], got {}",
            name, value
        )));
    }
    Ok(())
}

/// Validate that a value is positive (> 0).
///
/// # Arguments
/// * `name` - Name of the parameter (for error messages)
/// * `value` - Value to validate
///
/// # Returns
/// * `Ok(())` if value is positive
/// * `Err(CommonValidationError)` if value is <= 0 or not finite
///
/// # Example
/// ```
/// use speccade_spec::validation::common::validate_positive;
///
/// assert!(validate_positive("strength", 1.0).is_ok());
/// assert!(validate_positive("strength", 0.0).is_err());
/// assert!(validate_positive("strength", -1.0).is_err());
/// ```
pub fn validate_positive(name: &str, value: f64) -> Result<(), CommonValidationError> {
    if !value.is_finite() {
        return Err(CommonValidationError::new(format!(
            "{} must be finite, got {}",
            name, value
        )));
    }
    if value <= 0.0 {
        return Err(CommonValidationError::new(format!(
            "{} must be positive, got {}",
            name, value
        )));
    }
    Ok(())
}

/// Validate that a value is non-negative (>= 0).
///
/// # Arguments
/// * `name` - Name of the parameter (for error messages)
/// * `value` - Value to validate
///
/// # Returns
/// * `Ok(())` if value is non-negative
/// * `Err(CommonValidationError)` if value is < 0 or not finite
///
/// # Example
/// ```
/// use speccade_spec::validation::common::validate_non_negative;
///
/// assert!(validate_non_negative("bump_strength", 0.0).is_ok());
/// assert!(validate_non_negative("bump_strength", 1.0).is_ok());
/// assert!(validate_non_negative("bump_strength", -1.0).is_err());
/// ```
pub fn validate_non_negative(name: &str, value: f64) -> Result<(), CommonValidationError> {
    if !value.is_finite() {
        return Err(CommonValidationError::new(format!(
            "{} must be finite, got {}",
            name, value
        )));
    }
    if value < 0.0 {
        return Err(CommonValidationError::new(format!(
            "{} must be non-negative, got {}",
            name, value
        )));
    }
    Ok(())
}

/// Validate that a value is within a specified range [min, max].
///
/// # Arguments
/// * `name` - Name of the parameter (for error messages)
/// * `value` - Value to validate
/// * `min` - Minimum allowed value (inclusive)
/// * `max` - Maximum allowed value (inclusive)
///
/// # Returns
/// * `Ok(())` if value is within range
/// * `Err(CommonValidationError)` if value is outside range or not finite
///
/// # Example
/// ```
/// use speccade_spec::validation::common::validate_range;
///
/// assert!(validate_range("bpm", 120.0, 30.0, 300.0).is_ok());
/// assert!(validate_range("bpm", 500.0, 30.0, 300.0).is_err());
/// ```
pub fn validate_range(
    name: &str,
    value: f64,
    min: f64,
    max: f64,
) -> Result<(), CommonValidationError> {
    if !value.is_finite() {
        return Err(CommonValidationError::new(format!(
            "{} must be finite, got {}",
            name, value
        )));
    }
    if value < min || value > max {
        return Err(CommonValidationError::new(format!(
            "{} must be in [{}, {}], got {}",
            name, min, max, value
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_resolution_valid() {
        assert!(validate_resolution(1, 1).is_ok());
        assert!(validate_resolution(1024, 1024).is_ok());
        assert!(validate_resolution(4096, 4096).is_ok());
    }

    #[test]
    fn test_validate_resolution_zero() {
        let err = validate_resolution(0, 100).unwrap_err();
        assert!(err.message.contains("resolution"));
        assert!(err.message.contains("[0, 100]"));

        let err = validate_resolution(100, 0).unwrap_err();
        assert!(err.message.contains("resolution"));

        let err = validate_resolution(0, 0).unwrap_err();
        assert!(err.message.contains("resolution"));
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn test_validate_resolution_overflow() {
        // Reject absurd resolutions even if they fit in `usize`.
        let err = validate_resolution(u32::MAX, u32::MAX).unwrap_err();
        assert!(err.message.contains("too large"));
    }

    #[test]
    fn test_validate_resolution_max_dimension() {
        assert!(validate_resolution(4096, 4096).is_ok());
        assert!(validate_resolution(4096, 1).is_ok());

        let err = validate_resolution(4097, 4096).unwrap_err();
        assert!(err.message.contains("max is"));
    }

    #[test]
    fn test_validate_unit_interval_valid() {
        assert!(validate_unit_interval("test", 0.0).is_ok());
        assert!(validate_unit_interval("test", 0.5).is_ok());
        assert!(validate_unit_interval("test", 1.0).is_ok());
    }

    #[test]
    fn test_validate_unit_interval_out_of_range() {
        let err = validate_unit_interval("opacity", -0.1).unwrap_err();
        assert!(err.message.contains("opacity"));
        assert!(err.message.contains("[0, 1]"));

        let err = validate_unit_interval("opacity", 1.1).unwrap_err();
        assert!(err.message.contains("opacity"));
    }

    #[test]
    fn test_validate_unit_interval_non_finite() {
        let err = validate_unit_interval("test", f64::NAN).unwrap_err();
        assert!(err.message.contains("finite"));

        let err = validate_unit_interval("test", f64::INFINITY).unwrap_err();
        assert!(err.message.contains("finite"));
    }

    #[test]
    fn test_validate_positive_valid() {
        assert!(validate_positive("test", 0.001).is_ok());
        assert!(validate_positive("test", 1.0).is_ok());
        assert!(validate_positive("test", 1000.0).is_ok());
    }

    #[test]
    fn test_validate_positive_invalid() {
        let err = validate_positive("strength", 0.0).unwrap_err();
        assert!(err.message.contains("strength"));
        assert!(err.message.contains("positive"));

        let err = validate_positive("strength", -1.0).unwrap_err();
        assert!(err.message.contains("positive"));
    }

    #[test]
    fn test_validate_non_negative_valid() {
        assert!(validate_non_negative("test", 0.0).is_ok());
        assert!(validate_non_negative("test", 0.001).is_ok());
        assert!(validate_non_negative("test", 1000.0).is_ok());
    }

    #[test]
    fn test_validate_non_negative_invalid() {
        let err = validate_non_negative("bump", -0.001).unwrap_err();
        assert!(err.message.contains("bump"));
        assert!(err.message.contains("non-negative"));
    }

    #[test]
    fn test_validate_range_valid() {
        assert!(validate_range("bpm", 120.0, 30.0, 300.0).is_ok());
        assert!(validate_range("bpm", 30.0, 30.0, 300.0).is_ok());
        assert!(validate_range("bpm", 300.0, 30.0, 300.0).is_ok());
    }

    #[test]
    fn test_validate_range_invalid() {
        let err = validate_range("bpm", 20.0, 30.0, 300.0).unwrap_err();
        assert!(err.message.contains("bpm"));
        assert!(err.message.contains("[30, 300]"));

        let err = validate_range("bpm", 500.0, 30.0, 300.0).unwrap_err();
        assert!(err.message.contains("bpm"));
    }

    #[test]
    fn test_error_display() {
        let err = CommonValidationError::new("test error message");
        assert_eq!(err.to_string(), "test error message");
    }
}
