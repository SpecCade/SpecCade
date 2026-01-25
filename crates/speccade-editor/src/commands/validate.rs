//! Spec validation command for the editor.
//!
//! This command validates compiled specs against the SpecCade schema
//! and optional budget profiles.

use serde::{Deserialize, Serialize};
use speccade_cli::compiler::{self, CompileError, CompilerConfig};
use speccade_spec::hash::canonical_spec_hash;
use speccade_spec::validation;

/// A structured error in validate output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateError {
    /// Error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// JSON path to the problematic field (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Source location (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// A warning from validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateWarning {
    /// Warning code
    pub code: String,
    /// Human-readable warning message
    pub message: String,
    /// JSON path (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Validation result details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResult {
    /// Asset ID from the spec
    pub asset_id: String,
    /// Asset type
    pub asset_type: String,
    /// Budget profile used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<String>,
    /// Canonical spec hash
    pub spec_hash: String,
}

/// Output from the validate_spec command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateOutput {
    /// Whether validation succeeded
    pub success: bool,
    /// Errors encountered during validation
    pub errors: Vec<ValidateError>,
    /// Warnings from validation
    pub warnings: Vec<ValidateWarning>,
    /// Validation result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ValidateResult>,
    /// BLAKE3 hash of the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
}

/// Validate Starlark source code and return validation results.
///
/// This command is exposed to the Tauri frontend via IPC.
#[tauri::command]
pub fn validate_spec(source: String, filename: String, budget: Option<String>) -> ValidateOutput {
    // Compute source hash
    let source_hash = blake3::hash(source.as_bytes()).to_hex().to_string();

    // Configure compiler with default timeout
    let config = CompilerConfig::default();

    // Compile the Starlark source
    let spec = match compiler::compile(&filename, &source, &config) {
        Ok(result) => result.spec,
        Err(e) => {
            let (code, message, location) = match &e {
                CompileError::Syntax { location, message } => (
                    e.code().to_string(),
                    message.clone(),
                    Some(location.clone()),
                ),
                CompileError::Runtime { location, message } => (
                    e.code().to_string(),
                    message.clone(),
                    Some(location.clone()),
                ),
                _ => (e.code().to_string(), e.to_string(), None),
            };

            return ValidateOutput {
                success: false,
                errors: vec![ValidateError {
                    code,
                    message,
                    path: None,
                    location,
                }],
                warnings: Vec::new(),
                result: None,
                source_hash: Some(source_hash),
            };
        }
    };

    // Validate the spec
    let validation_result = validation::validate_spec(&spec);

    let mut errors: Vec<ValidateError> = Vec::new();
    let mut warnings: Vec<ValidateWarning> = Vec::new();

    for err in &validation_result.errors {
        errors.push(ValidateError {
            code: err.code.code().to_string(),
            message: err.message.clone(),
            path: err.path.clone(),
            location: None,
        });
    }

    for warn in &validation_result.warnings {
        warnings.push(ValidateWarning {
            code: warn.code.code().to_string(),
            message: warn.message.clone(),
            path: warn.path.clone(),
        });
    }

    // Apply budget validation if specified
    if let Some(budget_name) = &budget {
        if let Some(profile) = validation::BudgetProfile::by_name(budget_name) {
            let budget_result = validation::validate_spec_with_budget(&spec, &profile);
            for err in budget_result.errors {
                errors.push(ValidateError {
                    code: err.code.code().to_string(),
                    message: err.message.clone(),
                    path: err.path.clone(),
                    location: None,
                });
            }
            for warn in budget_result.warnings {
                warnings.push(ValidateWarning {
                    code: warn.code.code().to_string(),
                    message: warn.message.clone(),
                    path: warn.path.clone(),
                });
            }
        } else {
            errors.push(ValidateError {
                code: "BUDGET_UNKNOWN".to_string(),
                message: format!("Unknown budget profile: {}", budget_name),
                path: None,
                location: None,
            });
        }
    }

    let success = errors.is_empty();
    let spec_hash = canonical_spec_hash(&spec).unwrap_or_else(|_| "unknown".to_string());

    ValidateOutput {
        success,
        errors,
        warnings,
        result: if success {
            Some(ValidateResult {
                asset_id: spec.asset_id.clone(),
                asset_type: spec.asset_type.to_string(),
                budget,
                spec_hash,
            })
        } else {
            None
        },
        source_hash: Some(source_hash),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_spec() {
        let source = r#"
{
    "spec_version": 1,
    "asset_id": "test-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {"kind": "primary", "format": "wav", "path": "test.wav"}
    ]
}
"#;
        let result = validate_spec(source.to_string(), "test.star".to_string(), None);
        assert!(result.success);
        assert!(result.errors.is_empty());
        assert!(result.result.is_some());
    }

    #[test]
    fn test_validate_syntax_error() {
        let source = r#"{ invalid }"#;
        let result = validate_spec(source.to_string(), "test.star".to_string(), None);
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }
}
