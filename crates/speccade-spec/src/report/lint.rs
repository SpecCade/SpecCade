//! Lint report data types for generation reports.
//!
//! These types mirror the lint report types from `speccade_lint` without creating
//! a circular dependency. The CLI handles conversion between the two.

use serde::{Deserialize, Serialize};

/// Lint report data included in generation reports.
///
/// This mirrors `speccade_lint::LintReport` without the dependency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LintReportData {
    /// True if no errors were found.
    pub ok: bool,
    /// Number of errors.
    pub error_count: usize,
    /// Number of warnings.
    pub warning_count: usize,
    /// Number of info-level issues.
    pub info_count: usize,
    /// Error-level issues.
    pub errors: Vec<LintIssueData>,
    /// Warning-level issues.
    pub warnings: Vec<LintIssueData>,
    /// Info-level issues.
    pub info: Vec<LintIssueData>,
}

impl LintReportData {
    /// Creates an empty lint report (no issues).
    pub fn empty() -> Self {
        Self {
            ok: true,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    /// Returns true if there are any issues.
    pub fn has_issues(&self) -> bool {
        self.error_count > 0 || self.warning_count > 0 || self.info_count > 0
    }
}

/// Lint issue data included in generation reports.
///
/// This mirrors `speccade_lint::LintIssue` without the dependency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LintIssueData {
    /// Rule identifier (e.g., "audio/clipping").
    pub rule_id: String,
    /// Severity level ("error", "warning", "info").
    pub severity: String,
    /// Human-readable description.
    pub message: String,
    /// Location in generated asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_location: Option<String>,
    /// Location in source spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_path: Option<String>,
    /// Measured value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_value: Option<String>,
    /// Expected or acceptable range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_range: Option<String>,
    /// Human-readable fix suggestion.
    pub suggestion: String,
    /// Starlark snippet for fix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_template: Option<String>,
    /// Multiplier for numeric fixes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_delta: Option<f64>,
    /// Parameter to adjust.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_param: Option<String>,
}

impl LintIssueData {
    /// Creates a new lint issue.
    pub fn new(
        rule_id: impl Into<String>,
        severity: impl Into<String>,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity: severity.into(),
            message: message.into(),
            asset_location: None,
            spec_path: None,
            actual_value: None,
            expected_range: None,
            suggestion: suggestion.into(),
            fix_template: None,
            fix_delta: None,
            fix_param: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_report_data_empty() {
        let report = LintReportData::empty();
        assert!(report.ok);
        assert_eq!(report.error_count, 0);
        assert!(!report.has_issues());
    }

    #[test]
    fn test_lint_issue_data_serialization() {
        let issue = LintIssueData::new(
            "audio/clipping",
            "error",
            "Sample values exceed threshold",
            "Reduce amplitude",
        );

        let json = serde_json::to_string(&issue).unwrap();
        assert!(json.contains("\"rule_id\":\"audio/clipping\""));
        assert!(json.contains("\"severity\":\"error\""));
    }
}
