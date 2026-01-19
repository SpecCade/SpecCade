//! Output record types for core CLI commands (eval, validate, generate, expand).

use super::{JsonError, JsonWarning};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

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
