//! JSON output types for machine-readable CLI output.
//!
//! This module provides structured output types for the `--json` flag on
//! `eval`, `validate`, and `generate` commands. These types enable LLM agents
//! and other tools to parse CLI output programmatically.

mod analysis;
mod convert;
mod manifest;
mod records;

// Re-export all public types for backwards compatibility
pub use analysis::{
    AnalyzeOutput, AnalyzeResult, AudioCompareMetrics, BatchAnalyzeItem, BatchAnalyzeOutput,
    BatchAnalyzeSummary, CompareMetrics, CompareOutput, CompareResult, HistogramDiffMetrics,
    InspectOutput, InspectResult, IntermediateFile, MeshCompareMetrics, TextureCompareMetrics,
};
pub use convert::{
    compile_warnings_to_json, input_error_to_json, validation_error_to_json,
    validation_warning_to_json,
};
pub use manifest::{VariationConstraints, VariationEntry, VariationsManifest};
pub use records::{
    EvalOutput, ExpandOutput, GenerateOutput, GenerateResult, GeneratedFile, ValidateOutput,
    ValidateResult, VariantResult,
};

use serde::{Deserialize, Serialize};

/// Error codes for CLI operations.
///
/// These codes are stable and can be used for programmatic error handling.
/// Format: CLI_XXX for CLI-level errors, or passes through validation error codes.
pub mod error_codes {
    /// File could not be read
    pub const FILE_READ: &str = "CLI_001";
    /// Unknown file extension
    pub const UNKNOWN_EXTENSION: &str = "CLI_002";
    /// JSON parse error
    pub const JSON_PARSE: &str = "CLI_003";
    /// Starlark compilation error
    pub const STARLARK_COMPILE: &str = "CLI_004";
    /// Starlark evaluation timeout
    pub const STARLARK_TIMEOUT: &str = "CLI_005";
    /// Invalid spec (post-parse validation)
    pub const INVALID_SPEC: &str = "CLI_006";
    /// Starlark feature not enabled
    pub const STARLARK_NOT_ENABLED: &str = "CLI_007";
    /// Unknown budget profile
    pub const UNKNOWN_BUDGET: &str = "CLI_008";
    /// JSON serialization error
    pub const JSON_SERIALIZE: &str = "CLI_009";
    /// Generation error (wraps backend errors)
    pub const GENERATION_ERROR: &str = "CLI_010";
    /// Unsupported file format for analysis
    pub const UNSUPPORTED_FORMAT: &str = "CLI_011";
    /// Audio analysis error
    pub const AUDIO_ANALYSIS: &str = "CLI_012";
    /// Texture analysis error
    pub const TEXTURE_ANALYSIS: &str = "CLI_013";
    /// Mesh analysis error
    pub const MESH_ANALYSIS: &str = "CLI_014";
}

/// Warning codes for CLI operations.
pub mod warning_codes {
    /// Starlark compilation warning
    pub const STARLARK_WARNING: &str = "CLI_W001";
}

/// A structured error in JSON output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonError {
    /// Stable error code (e.g., "CLI_001", "E001")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// JSON path to the problematic field (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Source file path (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Line number (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Column number (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
    /// Suggestion for fixing the error (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl JsonError {
    /// Creates a new error with code and message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: None,
            file: None,
            line: None,
            col: None,
            suggestion: None,
        }
    }

    /// Sets the JSON path for this error.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Sets the file path for this error.
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Sets the line and column for this error.
    pub fn with_location(mut self, line: u32, col: u32) -> Self {
        self.line = Some(line);
        self.col = Some(col);
        self
    }

    /// Sets a suggestion for fixing the error.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// A structured warning in JSON output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonWarning {
    /// Stable warning code (e.g., "CLI_W001", "W001")
    pub code: String,
    /// Human-readable warning message
    pub message: String,
    /// JSON path to the problematic field (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Source file path (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Line number (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Column number (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
}

impl JsonWarning {
    /// Creates a new warning with code and message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: None,
            file: None,
            line: None,
            col: None,
        }
    }

    /// Sets the JSON path for this warning.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Sets the file path for this warning.
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Sets the line and column for this warning.
    pub fn with_location(mut self, line: u32, col: u32) -> Self {
        self.line = Some(line);
        self.col = Some(col);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_error_serialization() {
        let error = JsonError::new("E001", "test error")
            .with_path("spec.outputs[0]")
            .with_file("test.json");

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"code\":\"E001\""));
        assert!(json.contains("\"message\":\"test error\""));
        assert!(json.contains("\"path\":\"spec.outputs[0]\""));
        assert!(json.contains("\"file\":\"test.json\""));
    }

    #[test]
    fn test_json_error_optional_fields_skipped() {
        let error = JsonError::new("E001", "test error");
        let json = serde_json::to_string(&error).unwrap();

        assert!(!json.contains("\"path\""));
        assert!(!json.contains("\"file\""));
        assert!(!json.contains("\"line\""));
        assert!(!json.contains("\"col\""));
        assert!(!json.contains("\"suggestion\""));
    }
}
