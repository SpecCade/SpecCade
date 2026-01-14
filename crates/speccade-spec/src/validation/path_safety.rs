//! Output path safety validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};

/// Checks if an output path is safe.
///
/// # Arguments
/// * `path` - The output path to validate
///
/// # Returns
/// * `true` if the path is safe, `false` otherwise.
pub fn is_safe_output_path(path: &str) -> bool {
    output_path_safety_errors(path).is_empty()
}

/// Validates an output path for safety.
pub(super) fn validate_output_path(
    output: &crate::output::OutputSpec,
    index: usize,
    result: &mut ValidationResult,
) {
    let path = &output.path;
    let path_field = format!("outputs[{}].path", index);

    for message in output_path_safety_errors(path) {
        result.add_error(ValidationError::with_path(
            ErrorCode::UnsafeOutputPath,
            message,
            &path_field,
        ));
    }

    // Check that extension matches format
    if !output.extension_matches() {
        result.add_error(ValidationError::with_path(
            ErrorCode::PathFormatMismatch,
            format!(
                "output path extension does not match format '{}': '{}'",
                output.format, path
            ),
            &path_field,
        ));
    }
}

pub(super) fn output_path_safety_errors(path: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // Check for empty path
    if path.is_empty() {
        errors.push("output path cannot be empty".to_string());
        return errors;
    }

    // Check for absolute paths (leading slash or drive letter)
    if path.starts_with('/') || path.starts_with('\\') {
        errors.push(format!(
            "output path must be relative, not absolute: '{}'",
            path
        ));
    }

    // Check for Windows drive letter
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        errors.push(format!(
            "output path must not contain drive letter: '{}'",
            path
        ));
    }

    // Check for backslashes
    if path.contains('\\') {
        errors.push(format!(
            "output path must use forward slashes only: '{}'",
            path
        ));
    }

    // Check for path traversal (..)
    for segment in path.split('/') {
        if segment == ".." {
            errors.push(format!("output path must not contain '..': '{}'", path));
            break;
        }
    }

    errors
}
