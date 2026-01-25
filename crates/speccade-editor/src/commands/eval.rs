//! Spec evaluation command for the editor.
//!
//! This command wraps the speccade-cli compiler to evaluate Starlark source
//! and return the compiled spec as JSON.

use serde::{Deserialize, Serialize};
use speccade_cli::compiler::{self, CompileError, CompilerConfig};

/// A structured error in eval output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalError {
    /// Error code (S-series from compiler)
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Source location (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// A warning from compilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalWarning {
    /// Warning code
    pub code: String,
    /// Human-readable warning message
    pub message: String,
    /// Source location (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// Output from the eval_spec command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalOutput {
    /// Whether the eval succeeded
    pub success: bool,
    /// Errors encountered during evaluation
    pub errors: Vec<EvalError>,
    /// Warnings from compilation
    pub warnings: Vec<EvalWarning>,
    /// The evaluated spec as JSON (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// BLAKE3 hash of the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
}

/// Evaluate Starlark source code and return the compiled spec.
///
/// This command is exposed to the Tauri frontend via IPC.
#[tauri::command]
pub fn eval_spec(source: String, filename: String) -> EvalOutput {
    // Compute source hash
    let source_hash = blake3::hash(source.as_bytes()).to_hex().to_string();

    // Configure compiler with default timeout
    let config = CompilerConfig::default();

    // Compile the Starlark source
    match compiler::compile(&filename, &source, &config) {
        Ok(result) => {
            // Convert spec to JSON value
            match serde_json::to_value(&result.spec) {
                Ok(spec_json) => {
                    let warnings = result
                        .warnings
                        .into_iter()
                        .map(|w| EvalWarning {
                            code: "STAR_W001".to_string(),
                            message: w.message,
                            location: w.location,
                        })
                        .collect();

                    EvalOutput {
                        success: true,
                        errors: Vec::new(),
                        warnings,
                        result: Some(spec_json),
                        source_hash: Some(source_hash),
                    }
                }
                Err(e) => EvalOutput {
                    success: false,
                    errors: vec![EvalError {
                        code: "JSON_SERIALIZE".to_string(),
                        message: format!("Failed to serialize spec: {}", e),
                        location: None,
                    }],
                    warnings: Vec::new(),
                    result: None,
                    source_hash: Some(source_hash),
                },
            }
        }
        Err(e) => {
            let (code, message, location) = match &e {
                CompileError::Syntax { location, message } => {
                    (e.code().to_string(), message.clone(), Some(location.clone()))
                }
                CompileError::Runtime { location, message } => {
                    (e.code().to_string(), message.clone(), Some(location.clone()))
                }
                CompileError::Timeout { seconds } => (
                    e.code().to_string(),
                    format!("Evaluation timed out after {}s", seconds),
                    None,
                ),
                CompileError::NotADict { type_name } => (
                    e.code().to_string(),
                    format!("Spec must return a dict, got {}", type_name),
                    None,
                ),
                CompileError::JsonConversion { message } => {
                    (e.code().to_string(), message.clone(), None)
                }
                CompileError::InvalidSpec { message } => {
                    (e.code().to_string(), message.clone(), None)
                }
                CompileError::StdlibArgument { function, param } => (
                    e.code().to_string(),
                    format!("{}(): missing required argument '{}'", function, param),
                    None,
                ),
                CompileError::StdlibType {
                    function,
                    param,
                    expected,
                    got,
                } => (
                    e.code().to_string(),
                    format!(
                        "{}(): '{}' expected {}, got {}",
                        function, param, expected, got
                    ),
                    None,
                ),
                CompileError::StdlibRange {
                    function,
                    param,
                    range,
                    got,
                } => (
                    e.code().to_string(),
                    format!("{}(): '{}' {}, got {}", function, param, range, got),
                    None,
                ),
                CompileError::StdlibEnum {
                    function,
                    param,
                    valid,
                } => (
                    e.code().to_string(),
                    format!("{}(): '{}' must be one of: {}", function, param, valid),
                    None,
                ),
            };

            EvalOutput {
                success: false,
                errors: vec![EvalError {
                    code,
                    message,
                    location,
                }],
                warnings: Vec::new(),
                result: None,
                source_hash: Some(source_hash),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_valid_spec() {
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
        let result = eval_spec(source.to_string(), "test.star".to_string());
        assert!(result.success);
        assert!(result.errors.is_empty());
        assert!(result.result.is_some());
        assert!(result.source_hash.is_some());
    }

    #[test]
    fn test_eval_syntax_error() {
        let source = r#"{ invalid json }"#;
        let result = eval_spec(source.to_string(), "test.star".to_string());
        assert!(!result.success);
        assert!(!result.errors.is_empty());
        assert!(result.result.is_none());
    }
}
