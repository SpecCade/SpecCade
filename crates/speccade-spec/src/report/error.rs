//! Error and warning types for reports.

use crate::error::{ValidationError, ValidationWarning};
use serde::{Deserialize, Serialize};

/// Error entry in a report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportError {
    /// Error code (e.g., "E001").
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// JSON path to the problematic field (e.g., "outputs\[0\].path").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ReportError {
    /// Creates a new report error.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: None,
        }
    }

    /// Creates a new report error with a JSON path.
    pub fn with_path(
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: Some(path.into()),
        }
    }

    /// Converts a ValidationError to a ReportError.
    pub fn from_validation_error(err: &ValidationError) -> Self {
        Self {
            code: err.code.code().to_string(),
            message: err.message.clone(),
            path: err.path.clone(),
        }
    }
}

/// Warning entry in a report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportWarning {
    /// Warning code (e.g., "W001").
    pub code: String,
    /// Human-readable warning message.
    pub message: String,
    /// JSON path to the problematic field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ReportWarning {
    /// Creates a new report warning.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: None,
        }
    }

    /// Creates a new report warning with a JSON path.
    pub fn with_path(
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: Some(path.into()),
        }
    }

    /// Converts a ValidationWarning to a ReportWarning.
    pub fn from_validation_warning(warn: &ValidationWarning) -> Self {
        Self {
            code: warn.code.code().to_string(),
            message: warn.message.clone(),
            path: warn.path.clone(),
        }
    }
}
