//! Conversion helpers for transforming errors and warnings to JSON format.

use super::{error_codes, warning_codes, JsonError, JsonWarning};
use crate::input::InputError;

/// Converts an InputError to a JsonError.
pub fn input_error_to_json(err: &InputError, file: Option<&str>) -> JsonError {
    let (code, message) = match err {
        InputError::FileRead { path, source } => (
            error_codes::FILE_READ,
            format!("Failed to read file '{}': {}", path.display(), source),
        ),
        InputError::UnknownExtension { extension } => (
            error_codes::UNKNOWN_EXTENSION,
            match extension {
                Some(ext) => format!(
                    "Unknown file extension '.{}' (expected .json or .star)",
                    ext
                ),
                None => "File has no extension (expected .json or .star)".to_string(),
            },
        ),
        InputError::JsonParse { message } => (
            error_codes::JSON_PARSE,
            format!("JSON parse error: {}", message),
        ),
        #[cfg(feature = "starlark")]
        InputError::StarlarkCompile { message } => (
            error_codes::STARLARK_COMPILE,
            format!("Starlark compilation error: {}", message),
        ),
        #[cfg(feature = "starlark")]
        InputError::Timeout { seconds } => (
            error_codes::STARLARK_TIMEOUT,
            format!("Starlark evaluation timed out after {}s", seconds),
        ),
        #[cfg(not(feature = "starlark"))]
        InputError::StarlarkNotEnabled => (
            error_codes::STARLARK_NOT_ENABLED,
            "Starlark support is not enabled. Rebuild with --features starlark".to_string(),
        ),
        InputError::InvalidSpec { message } => (
            error_codes::INVALID_SPEC,
            format!("Invalid spec: {}", message),
        ),
    };

    let mut error = JsonError::new(code, message);
    if let Some(f) = file {
        error = error.with_file(f);
    }
    error
}

/// Converts a ValidationError to a JsonError.
pub fn validation_error_to_json(err: &speccade_spec::ValidationError) -> JsonError {
    let mut error = JsonError::new(err.code.to_string(), &err.message);
    if let Some(ref path) = err.path {
        error = error.with_path(path);
    }
    error
}

/// Converts a ValidationWarning to a JsonWarning.
pub fn validation_warning_to_json(warn: &speccade_spec::ValidationWarning) -> JsonWarning {
    let mut warning = JsonWarning::new(warn.code.to_string(), &warn.message);
    if let Some(ref path) = warn.path {
        warning = warning.with_path(path);
    }
    warning
}

/// Converts compile warnings to JsonWarnings.
pub fn compile_warnings_to_json(warnings: &[crate::input::CompileWarning]) -> Vec<JsonWarning> {
    warnings
        .iter()
        .map(|w| {
            let mut warning = JsonWarning::new(warning_codes::STARLARK_WARNING, &w.message);
            if let Some(ref loc) = w.location {
                warning = warning.with_path(loc);
            }
            warning
        })
        .collect()
}
