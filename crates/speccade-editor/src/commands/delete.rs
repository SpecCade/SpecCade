//! File deletion command.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Output from batch delete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteOutput {
    pub total: usize,
    pub deleted: usize,
    pub failed: usize,
    pub errors: Vec<DeleteError>,
}

/// Error deleting a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteError {
    pub path: String,
    pub message: String,
}

/// Delete multiple files.
#[tauri::command]
pub fn batch_delete(paths: Vec<String>) -> BatchDeleteOutput {
    let mut deleted = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for path in &paths {
        let file_path = Path::new(path);

        // Safety: only delete .star files
        if !path.ends_with(".star") {
            failed += 1;
            errors.push(DeleteError {
                path: path.clone(),
                message: "Can only delete .star spec files".to_string(),
            });
            continue;
        }

        match std::fs::remove_file(file_path) {
            Ok(()) => deleted += 1,
            Err(e) => {
                failed += 1;
                errors.push(DeleteError {
                    path: path.clone(),
                    message: e.to_string(),
                });
            }
        }
    }

    BatchDeleteOutput {
        total: paths.len(),
        deleted,
        failed,
        errors,
    }
}
