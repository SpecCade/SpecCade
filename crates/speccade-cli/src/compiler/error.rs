//! Error types for Starlark compilation.
//!
//! ## Error Code Ranges
//!
//! | Range | Category | Description |
//! |-------|----------|-------------|
//! | S001-S009 | Compiler | Syntax, runtime, timeout errors |
//! | S101-S199 | Stdlib | Function argument validation |
//! | S201-S299 | Reserved | Future stdlib categories |

use thiserror::Error;

/// Errors from Starlark compilation.
///
/// Error codes use a stable S-series format:
/// - S001-S009: Compiler errors (syntax, runtime, timeout)
/// - S101-S199: Stdlib argument validation errors
#[derive(Debug, Error)]
pub enum CompileError {
    /// S001: Syntax error in Starlark source.
    #[error("S001: syntax error at {location}: {message}")]
    Syntax { location: String, message: String },

    /// S002: Runtime error during evaluation.
    #[error("S002: runtime error at {location}: {message}")]
    Runtime { location: String, message: String },

    /// S003: Evaluation timed out.
    #[error("S003: evaluation timed out after {seconds}s")]
    Timeout { seconds: u64 },

    /// S004: Output value is not a dict.
    #[error("S004: spec must return a dict, got {type_name}")]
    NotADict { type_name: String },

    /// S005: Output cannot be converted to JSON.
    #[error("S005: cannot convert to JSON: {message}")]
    JsonConversion { message: String },

    /// S006: Resulting JSON is not a valid Spec.
    #[error("S006: invalid spec: {message}")]
    InvalidSpec { message: String },

    /// S101: Invalid stdlib function argument.
    #[error("S101: {function}(): missing required argument '{param}'")]
    StdlibArgument { function: String, param: String },

    /// S102: Type mismatch in stdlib function.
    #[error("S102: {function}(): '{param}' expected {expected}, got {got}")]
    StdlibType {
        function: String,
        param: String,
        expected: String,
        got: String,
    },

    /// S103: Value out of range in stdlib function.
    #[error("S103: {function}(): '{param}' {range}, got {got}")]
    StdlibRange {
        function: String,
        param: String,
        range: String,
        got: String,
    },

    /// S104: Invalid enum value in stdlib function.
    #[error("S104: {function}(): '{param}' must be one of: {valid}")]
    StdlibEnum {
        function: String,
        param: String,
        valid: String,
    },
}

impl CompileError {
    /// Returns the error code (e.g., "S001", "S103").
    pub fn code(&self) -> &'static str {
        match self {
            CompileError::Syntax { .. } => "S001",
            CompileError::Runtime { .. } => "S002",
            CompileError::Timeout { .. } => "S003",
            CompileError::NotADict { .. } => "S004",
            CompileError::JsonConversion { .. } => "S005",
            CompileError::InvalidSpec { .. } => "S006",
            CompileError::StdlibArgument { .. } => "S101",
            CompileError::StdlibType { .. } => "S102",
            CompileError::StdlibRange { .. } => "S103",
            CompileError::StdlibEnum { .. } => "S104",
        }
    }

    /// Returns the error category.
    pub fn category(&self) -> &'static str {
        match self {
            CompileError::Syntax { .. }
            | CompileError::Runtime { .. }
            | CompileError::Timeout { .. }
            | CompileError::NotADict { .. }
            | CompileError::JsonConversion { .. }
            | CompileError::InvalidSpec { .. } => "compiler",
            CompileError::StdlibArgument { .. }
            | CompileError::StdlibType { .. }
            | CompileError::StdlibRange { .. }
            | CompileError::StdlibEnum { .. } => "stdlib",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = CompileError::Syntax {
            location: "test.star".to_string(),
            message: "unexpected token".to_string(),
        };
        assert_eq!(err.code(), "S001");
        assert_eq!(err.category(), "compiler");
        assert!(err.to_string().contains("S001"));

        let err = CompileError::StdlibRange {
            function: "oscillator".to_string(),
            param: "frequency".to_string(),
            range: "must be positive".to_string(),
            got: "-440".to_string(),
        };
        assert_eq!(err.code(), "S103");
        assert_eq!(err.category(), "stdlib");
        assert!(err.to_string().contains("S103"));
    }

    #[test]
    fn test_stdlib_error_display() {
        let err = CompileError::StdlibEnum {
            function: "oscillator".to_string(),
            param: "waveform".to_string(),
            valid: "sine, square, sawtooth, triangle".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("S104"));
        assert!(msg.contains("oscillator"));
        assert!(msg.contains("waveform"));
        assert!(msg.contains("sine"));
    }
}
