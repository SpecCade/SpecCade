//! Batch generation commands.
//!
//! This module provides batch processing capabilities for generating
//! multiple specs at once.

use serde::{Deserialize, Serialize};
use speccade_cli::compiler::{self, CompileError, CompilerConfig};
use speccade_cli::dispatch::{dispatch_generate, DispatchError};
use std::path::Path;
use std::time::Instant;

use super::generate::GeneratedFile;

/// Output from the batch_generate command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGenerateOutput {
    /// Total number of specs processed.
    pub total: usize,
    /// Number of specs that succeeded.
    pub succeeded: usize,
    /// Number of specs that failed.
    pub failed: usize,
    /// Results for each spec in the batch.
    pub results: Vec<BatchItemResult>,
    /// Total elapsed time in milliseconds.
    pub elapsed_ms: u64,
}

/// Result for a single item in the batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemResult {
    /// Path to the source file.
    pub path: String,
    /// Whether generation succeeded.
    pub success: bool,
    /// List of generated output files.
    pub outputs: Vec<GeneratedFile>,
    /// Error message if generation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Generate assets from multiple spec files.
///
/// This command processes a batch of Starlark spec files, compiling and
/// generating outputs for each one. Results are collected and returned
/// together with timing information.
#[tauri::command]
pub fn batch_generate(paths: Vec<String>, output_dir: String) -> BatchGenerateOutput {
    let start = Instant::now();
    let config = CompilerConfig::default();
    let out_path = Path::new(&output_dir);

    let mut results = Vec::new();
    let mut succeeded = 0;
    let mut failed = 0;

    for path in &paths {
        // Read file
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                failed += 1;
                results.push(BatchItemResult {
                    path: path.clone(),
                    success: false,
                    outputs: vec![],
                    error: Some(format!("Read error: {}", e)),
                });
                continue;
            }
        };

        let spec_path = Path::new(path);
        let filename = spec_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.star".to_string());

        // Compile
        let spec = match compiler::compile(&filename, &source, &config) {
            Ok(result) => result.spec,
            Err(e) => {
                let error_msg = match &e {
                    CompileError::Syntax { location, message } => {
                        format!("Syntax error at {}: {}", location, message)
                    }
                    CompileError::Runtime { location, message } => {
                        format!("Runtime error at {}: {}", location, message)
                    }
                    _ => e.to_string(),
                };

                failed += 1;
                results.push(BatchItemResult {
                    path: path.clone(),
                    success: false,
                    outputs: vec![],
                    error: Some(format!("Compile error: {}", error_msg)),
                });
                continue;
            }
        };

        // Generate outputs
        let output_results = match dispatch_generate(&spec, &output_dir, spec_path, None) {
            Ok(results) => results,
            Err(e) => {
                let error_msg = match &e {
                    DispatchError::NoRecipe => "No recipe defined in the spec".to_string(),
                    DispatchError::BackendNotImplemented(kind) => {
                        format!("Backend not implemented for recipe kind: {}", kind)
                    }
                    DispatchError::BackendError(msg) => format!("Backend error: {}", msg),
                };

                failed += 1;
                results.push(BatchItemResult {
                    path: path.clone(),
                    success: false,
                    outputs: vec![],
                    error: Some(format!("Generate error: {}", error_msg)),
                });
                continue;
            }
        };

        // Map OutputResults to GeneratedFiles
        let outputs: Vec<GeneratedFile> = output_results
            .iter()
            .filter_map(|result| {
                let full_path = out_path.join(&result.path);
                let size_bytes = std::fs::metadata(&full_path).map(|m| m.len()).unwrap_or(0);

                Some(GeneratedFile {
                    path: result.path.to_string_lossy().to_string(),
                    size_bytes,
                    format: format!("{:?}", result.format).to_lowercase(),
                })
            })
            .collect();

        succeeded += 1;
        results.push(BatchItemResult {
            path: path.clone(),
            success: true,
            outputs,
            error: None,
        });
    }

    BatchGenerateOutput {
        total: paths.len(),
        succeeded,
        failed,
        results,
        elapsed_ms: start.elapsed().as_millis() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_generate_empty() {
        let result = batch_generate(vec![], "/tmp/output".to_string());
        assert_eq!(result.total, 0);
        assert_eq!(result.succeeded, 0);
        assert_eq!(result.failed, 0);
        assert!(result.results.is_empty());
    }

    #[test]
    fn test_batch_generate_nonexistent_file() {
        let result = batch_generate(
            vec!["/nonexistent/file.star".to_string()],
            "/tmp/output".to_string(),
        );
        assert_eq!(result.total, 1);
        assert_eq!(result.succeeded, 0);
        assert_eq!(result.failed, 1);
        assert!(!result.results[0].success);
        assert!(result.results[0]
            .error
            .as_ref()
            .unwrap()
            .contains("Read error"));
    }
}
