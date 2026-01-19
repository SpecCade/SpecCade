//! Result types for constraint evaluation.

use serde::{Deserialize, Serialize};

use super::Constraint;

/// The result of evaluating a single constraint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintResult {
    /// String representation of the constraint that was evaluated.
    pub constraint: String,
    /// Whether the constraint passed.
    pub passed: bool,
    /// The actual value from the report (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<serde_json::Value>,
    /// Human-readable message explaining the result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ConstraintResult {
    /// Creates a passing constraint result.
    pub fn pass(constraint: &Constraint, actual: Option<serde_json::Value>) -> Self {
        Self {
            constraint: constraint.to_string(),
            passed: true,
            actual,
            message: None,
        }
    }

    /// Creates a failing constraint result.
    pub fn fail(
        constraint: &Constraint,
        actual: Option<serde_json::Value>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            constraint: constraint.to_string(),
            passed: false,
            actual,
            message: Some(message.into()),
        }
    }

    /// Creates a skipped constraint result (when metric is not available).
    pub fn skipped(constraint: &Constraint, message: impl Into<String>) -> Self {
        Self {
            constraint: constraint.to_string(),
            passed: true, // Skipped constraints pass by default
            actual: None,
            message: Some(message.into()),
        }
    }
}

/// The result of verifying all constraints against a report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyResult {
    /// Asset ID from the report.
    pub asset_id: String,
    /// Whether all constraints passed.
    pub overall_pass: bool,
    /// Results for each constraint.
    pub results: Vec<ConstraintResult>,
}

impl VerifyResult {
    /// Creates a new verify result.
    pub fn new(asset_id: String, results: Vec<ConstraintResult>) -> Self {
        let overall_pass = results.iter().all(|r| r.passed);
        Self {
            asset_id,
            overall_pass,
            results,
        }
    }

    /// Serializes the verify result to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serializes the verify result to pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
