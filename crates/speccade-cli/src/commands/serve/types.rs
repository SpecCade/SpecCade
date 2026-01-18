//! Request and response types for the WebSocket analysis server.

use serde::{Deserialize, Serialize};

use crate::commands::json_output::JsonError;

/// Request types supported by the server.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnalyzeRequest {
    /// Analyze a file at the given absolute path.
    AnalyzePath {
        /// Absolute path to the file to analyze.
        path: String,
        /// Whether to include feature embeddings (optional, default false).
        #[serde(default)]
        embeddings: bool,
    },
    /// Analyze raw data provided as base64.
    AnalyzeData {
        /// Base64-encoded file data.
        data: String,
        /// Filename (used to detect asset type from extension).
        filename: String,
        /// Whether to include feature embeddings (optional, default false).
        #[serde(default)]
        embeddings: bool,
    },
}

/// Error response for invalid requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Whether the request succeeded (always false for errors).
    pub success: bool,
    /// Error details.
    pub errors: Vec<JsonError>,
}

impl ErrorResponse {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            success: false,
            errors: vec![JsonError::new(code, message)],
        }
    }
}
