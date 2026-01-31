//! Batch validation command.

use super::validate::{validate_spec, ValidateOutput};
use serde::{Deserialize, Serialize};

/// Output from batch validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchValidateOutput {
    pub total: usize,
    pub valid: usize,
    pub invalid: usize,
    pub results: Vec<BatchValidateItem>,
}

/// Result for a single validated file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchValidateItem {
    pub path: String,
    pub result: ValidateOutput,
}

/// Validate multiple spec files.
#[tauri::command]
pub fn batch_validate(paths: Vec<String>, budget: Option<String>) -> BatchValidateOutput {
    let mut results = Vec::new();
    let mut valid = 0;
    let mut invalid = 0;

    for path in &paths {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                invalid += 1;
                results.push(BatchValidateItem {
                    path: path.clone(),
                    result: ValidateOutput {
                        success: false,
                        errors: vec![super::validate::ValidateError {
                            code: "READ_ERROR".to_string(),
                            message: e.to_string(),
                            path: None,
                            location: None,
                        }],
                        warnings: vec![],
                        result: None,
                        source_hash: None,
                    },
                });
                continue;
            }
        };

        let filename = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.star".to_string());

        let result = validate_spec(source, filename, budget.clone());
        if result.success {
            valid += 1;
        } else {
            invalid += 1;
        }
        results.push(BatchValidateItem {
            path: path.clone(),
            result,
        });
    }

    BatchValidateOutput {
        total: paths.len(),
        valid,
        invalid,
        results,
    }
}
