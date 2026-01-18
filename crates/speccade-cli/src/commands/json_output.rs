//! JSON output types for machine-readable CLI output.
//!
//! This module provides structured output types for the `--json` flag on
//! `eval`, `validate`, and `generate` commands. These types enable LLM agents
//! and other tools to parse CLI output programmatically.

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

/// JSON output for the `eval` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalOutput {
    /// Whether the eval succeeded
    pub success: bool,
    /// Errors encountered during evaluation
    pub errors: Vec<JsonError>,
    /// Warnings from compilation
    pub warnings: Vec<JsonWarning>,
    /// The evaluated spec as JSON (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// BLAKE3 hash of the source file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
}

impl EvalOutput {
    /// Creates a successful eval output.
    pub fn success(
        spec_json: serde_json::Value,
        source_hash: String,
        warnings: Vec<JsonWarning>,
    ) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            warnings,
            result: Some(spec_json),
            source_hash: Some(source_hash),
        }
    }

    /// Creates a failed eval output.
    pub fn failure(errors: Vec<JsonError>, warnings: Vec<JsonWarning>) -> Self {
        Self {
            success: false,
            errors,
            warnings,
            result: None,
            source_hash: None,
        }
    }
}

/// JSON output for the `validate` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateOutput {
    /// Whether validation succeeded (no errors)
    pub success: bool,
    /// Validation errors
    pub errors: Vec<JsonError>,
    /// Validation warnings
    pub warnings: Vec<JsonWarning>,
    /// Validation result details (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ValidateResult>,
    /// Canonical spec hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_hash: Option<String>,
    /// BLAKE3 hash of the source file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
}

/// Validation result details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResult {
    /// Asset ID from the spec
    pub asset_id: String,
    /// Asset type
    pub asset_type: String,
    /// Source format (json/starlark)
    pub source_kind: String,
    /// Budget profile used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<String>,
    /// Recipe hash (if recipe present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_hash: Option<String>,
    /// Path to the generated report file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl ValidateOutput {
    /// Creates a successful validate output.
    pub fn success(
        result: ValidateResult,
        spec_hash: String,
        source_hash: String,
        warnings: Vec<JsonWarning>,
    ) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            warnings,
            result: Some(result),
            spec_hash: Some(spec_hash),
            source_hash: Some(source_hash),
        }
    }

    /// Creates a failed validate output.
    pub fn failure(
        errors: Vec<JsonError>,
        warnings: Vec<JsonWarning>,
        spec_hash: Option<String>,
        source_hash: Option<String>,
    ) -> Self {
        Self {
            success: false,
            errors,
            warnings,
            result: None,
            spec_hash,
            source_hash,
        }
    }
}

/// JSON output for the `generate` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOutput {
    /// Whether generation succeeded
    pub success: bool,
    /// Errors encountered during generation
    pub errors: Vec<JsonError>,
    /// Warnings from validation/generation
    pub warnings: Vec<JsonWarning>,
    /// Generation result details (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GenerateResult>,
    /// Canonical spec hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_hash: Option<String>,
    /// BLAKE3 hash of the source file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
}

/// A generated output file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// Output kind (primary, variant, etc.)
    pub kind: String,
    /// Output format (wav, png, etc.)
    pub format: String,
    /// Output path relative to out_root
    pub path: String,
    /// BLAKE3 hash of the output file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Whether this output was generated in preview mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
}

/// Generation result details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResult {
    /// Asset ID from the spec
    pub asset_id: String,
    /// Asset type
    pub asset_type: String,
    /// Source format (json/starlark)
    pub source_kind: String,
    /// Output root directory
    pub out_root: String,
    /// Budget profile used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<String>,
    /// Recipe hash (if recipe present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_hash: Option<String>,
    /// Whether this generation was served from cache
    pub cache_hit: bool,
    /// Generated output files
    pub outputs: Vec<GeneratedFile>,
    /// Path to the generated report file
    pub report_path: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Variant results (if expand_variants was enabled)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<VariantResult>,
}

/// Result for a single variant generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantResult {
    /// Variant ID
    pub variant_id: String,
    /// Whether this variant succeeded
    pub success: bool,
    /// Variant-specific spec hash
    pub spec_hash: String,
    /// Whether this generation was served from cache
    pub cache_hit: bool,
    /// Generated output files for this variant
    pub outputs: Vec<GeneratedFile>,
    /// Path to the variant report file
    pub report_path: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonError>,
}

impl GenerateOutput {
    /// Creates a successful generate output.
    pub fn success(
        result: GenerateResult,
        spec_hash: String,
        source_hash: String,
        warnings: Vec<JsonWarning>,
    ) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            warnings,
            result: Some(result),
            spec_hash: Some(spec_hash),
            source_hash: Some(source_hash),
        }
    }

    /// Creates a failed generate output.
    pub fn failure(
        errors: Vec<JsonError>,
        warnings: Vec<JsonWarning>,
        spec_hash: Option<String>,
        source_hash: Option<String>,
    ) -> Self {
        Self {
            success: false,
            errors,
            warnings,
            result: None,
            spec_hash,
            source_hash,
        }
    }
}

/// Converts an InputError to a JsonError.
pub fn input_error_to_json(err: &crate::input::InputError, file: Option<&str>) -> JsonError {
    use crate::input::InputError;

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

/// JSON output for the `expand` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandOutput {
    /// Whether the expansion succeeded
    pub success: bool,
    /// Errors encountered during expansion
    pub errors: Vec<JsonError>,
    /// Warnings from compilation (Starlark only)
    pub warnings: Vec<JsonWarning>,
    /// The expanded tracker params JSON (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expanded: Option<serde_json::Value>,
    /// BLAKE3 hash of the source file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
    /// Source format (json/starlark)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<String>,
}

impl ExpandOutput {
    /// Creates a successful expand output.
    pub fn success(
        expanded: serde_json::Value,
        source_hash: String,
        source_kind: String,
        warnings: Vec<JsonWarning>,
    ) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            warnings,
            expanded: Some(expanded),
            source_hash: Some(source_hash),
            source_kind: Some(source_kind),
        }
    }

    /// Creates a failed expand output.
    pub fn failure(errors: Vec<JsonError>, warnings: Vec<JsonWarning>) -> Self {
        Self {
            success: false,
            errors,
            warnings,
            expanded: None,
            source_hash: None,
            source_kind: None,
        }
    }
}

/// JSON output for the `analyze` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeOutput {
    /// Whether analysis succeeded
    pub success: bool,
    /// Errors encountered during analysis
    pub errors: Vec<JsonError>,
    /// Analysis result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AnalyzeResult>,
}

/// Analysis result details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeResult {
    /// Input file path
    pub input: String,
    /// Asset type analyzed (audio/texture)
    pub asset_type: String,
    /// BLAKE3 hash of the input file
    pub input_hash: String,
    /// Extracted metrics (structure depends on asset type)
    pub metrics: std::collections::BTreeMap<String, serde_json::Value>,
    /// Fixed-dimension feature embedding for similarity search (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f64>>,
}

impl AnalyzeOutput {
    /// Creates a successful analyze output.
    pub fn success(result: AnalyzeResult) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            result: Some(result),
        }
    }

    /// Creates a failed analyze output.
    pub fn failure(errors: Vec<JsonError>) -> Self {
        Self {
            success: false,
            errors,
            result: None,
        }
    }
}

/// Result for a single file in batch mode (either success or error).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalyzeItem {
    /// Input file path
    pub input: String,
    /// Whether analysis succeeded
    pub success: bool,
    /// Asset type analyzed (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<String>,
    /// BLAKE3 hash of the input file (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_hash: Option<String>,
    /// Extracted metrics (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<std::collections::BTreeMap<String, serde_json::Value>>,
    /// Fixed-dimension feature embedding (if requested and successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f64>>,
    /// Error information (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonError>,
}

impl BatchAnalyzeItem {
    /// Creates a successful batch item from an AnalyzeResult.
    pub fn success(result: AnalyzeResult) -> Self {
        Self {
            input: result.input,
            success: true,
            asset_type: Some(result.asset_type),
            input_hash: Some(result.input_hash),
            metrics: Some(result.metrics),
            embedding: result.embedding,
            error: None,
        }
    }

    /// Creates a failed batch item.
    pub fn failure(input: String, error: JsonError) -> Self {
        Self {
            input,
            success: false,
            asset_type: None,
            input_hash: None,
            metrics: None,
            embedding: None,
            error: Some(error),
        }
    }
}

/// Summary statistics for batch analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalyzeSummary {
    /// Total number of files processed
    pub total: usize,
    /// Number of successfully analyzed files
    pub succeeded: usize,
    /// Number of failed files
    pub failed: usize,
}

/// JSON output for batch analyze command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalyzeOutput {
    /// Whether the overall batch operation completed (always true unless catastrophic failure)
    pub success: bool,
    /// Individual file results
    pub results: Vec<BatchAnalyzeItem>,
    /// Summary statistics
    pub summary: BatchAnalyzeSummary,
}

impl BatchAnalyzeOutput {
    /// Creates a new batch output from results.
    pub fn new(results: Vec<BatchAnalyzeItem>) -> Self {
        let total = results.len();
        let succeeded = results.iter().filter(|r| r.success).count();
        let failed = total - succeeded;

        Self {
            success: true,
            results,
            summary: BatchAnalyzeSummary {
                total,
                succeeded,
                failed,
            },
        }
    }
}

/// JSON output for the `inspect` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectOutput {
    /// Whether inspection succeeded
    pub success: bool,
    /// Errors encountered during inspection
    pub errors: Vec<JsonError>,
    /// Warnings from validation
    pub warnings: Vec<JsonWarning>,
    /// Inspection result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<InspectResult>,
    /// Canonical spec hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_hash: Option<String>,
    /// BLAKE3 hash of the source file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
}

/// Inspection result details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectResult {
    /// Asset ID from the spec
    pub asset_id: String,
    /// Asset type
    pub asset_type: String,
    /// Source format (json/starlark)
    pub source_kind: String,
    /// Recipe kind
    pub recipe_kind: String,
    /// Output directory for intermediates
    pub out_dir: String,
    /// Intermediate artifact paths
    pub intermediates: Vec<IntermediateFile>,
    /// Final output paths (from spec outputs)
    pub final_outputs: Vec<IntermediateFile>,
    /// Expanded params JSON path (for compose specs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expanded_params_path: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// An intermediate file produced during inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediateFile {
    /// Identifier for this intermediate (node id, layer name, etc.)
    pub id: String,
    /// File format (png, json, etc.)
    pub format: String,
    /// Path to the file (relative to out_dir)
    pub path: String,
    /// BLAKE3 hash of the file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl InspectOutput {
    /// Creates a successful inspect output.
    pub fn success(
        result: InspectResult,
        spec_hash: String,
        source_hash: String,
        warnings: Vec<JsonWarning>,
    ) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            warnings,
            result: Some(result),
            spec_hash: Some(spec_hash),
            source_hash: Some(source_hash),
        }
    }

    /// Creates a failed inspect output.
    pub fn failure(
        errors: Vec<JsonError>,
        warnings: Vec<JsonWarning>,
        spec_hash: Option<String>,
        source_hash: Option<String>,
    ) -> Self {
        Self {
            success: false,
            errors,
            warnings,
            result: None,
            spec_hash,
            source_hash,
        }
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

    #[test]
    fn test_eval_output_success() {
        let spec_json = serde_json::json!({"asset_id": "test"});
        let output = EvalOutput::success(spec_json, "abc123".to_string(), vec![]);

        assert!(output.success);
        assert!(output.errors.is_empty());
        assert!(output.result.is_some());
        assert_eq!(output.source_hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_eval_output_failure() {
        let errors = vec![JsonError::new("CLI_001", "file not found")];
        let output = EvalOutput::failure(errors, vec![]);

        assert!(!output.success);
        assert_eq!(output.errors.len(), 1);
        assert!(output.result.is_none());
    }

    #[test]
    fn test_validate_output_serialization() {
        let result = ValidateResult {
            asset_id: "test-asset".to_string(),
            asset_type: "audio".to_string(),
            source_kind: "json".to_string(),
            budget: None,
            recipe_hash: None,
            report_path: Some("test.report.json".to_string()),
            duration_ms: 100,
        };

        let output = ValidateOutput::success(
            result,
            "spechash".to_string(),
            "sourcehash".to_string(),
            vec![],
        );

        let json = serde_json::to_string_pretty(&output).unwrap();
        // Pretty-printed JSON uses `: ` (colon followed by space)
        assert!(json.contains("\"success\": true"));
        assert!(json.contains("\"asset_id\": \"test-asset\""));
    }

    #[test]
    fn test_generate_output_with_variants() {
        let result = GenerateResult {
            asset_id: "test-asset".to_string(),
            asset_type: "audio".to_string(),
            source_kind: "starlark".to_string(),
            out_root: "./out".to_string(),
            budget: Some("strict".to_string()),
            recipe_hash: Some("recipehash".to_string()),
            cache_hit: false,
            outputs: vec![GeneratedFile {
                kind: "primary".to_string(),
                format: "wav".to_string(),
                path: "test.wav".to_string(),
                hash: Some("filehash".to_string()),
                preview: None,
            }],
            report_path: "test.report.json".to_string(),
            duration_ms: 250,
            variants: vec![],
        };

        let output = GenerateOutput::success(
            result,
            "spechash".to_string(),
            "sourcehash".to_string(),
            vec![],
        );

        let json = serde_json::to_string_pretty(&output).unwrap();
        // Pretty-printed JSON uses `: ` (colon followed by space)
        assert!(json.contains("\"success\": true"));
        assert!(json.contains("\"out_root\": \"./out\""));
    }
}
