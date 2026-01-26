//! Lint command for the editor.
//!
//! This command runs the lint system on generated preview data.

use serde::{Deserialize, Serialize};
use speccade_lint::{LintIssue, LintReport, RuleRegistry, Severity};
use std::path::Path;

/// A lint issue formatted for the editor frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorLintIssue {
    /// Rule identifier (e.g., "audio/clipping").
    pub rule_id: String,
    /// Severity level: "error", "warning", or "info".
    pub severity: String,
    /// Human-readable message.
    pub message: String,
    /// Suggestion for fixing the issue.
    pub suggestion: String,
    /// Location in source spec (for navigation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_path: Option<String>,
    /// Actual measured value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_value: Option<String>,
    /// Expected/acceptable range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_range: Option<String>,
}

impl From<LintIssue> for EditorLintIssue {
    fn from(issue: LintIssue) -> Self {
        let severity = match issue.severity {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        Self {
            rule_id: issue.rule_id,
            severity: severity.to_string(),
            message: issue.message,
            suggestion: issue.suggestion,
            spec_path: issue.spec_path,
            actual_value: issue.actual_value,
            expected_range: issue.expected_range,
        }
    }
}

/// Output from the lint command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintOutput {
    /// Whether lint passed (no errors).
    pub ok: bool,
    /// All lint issues found.
    pub issues: Vec<EditorLintIssue>,
    /// Error count.
    pub error_count: usize,
    /// Warning count.
    pub warning_count: usize,
    /// Info count.
    pub info_count: usize,
}

impl From<LintReport> for LintOutput {
    fn from(report: LintReport) -> Self {
        let mut issues = Vec::new();

        for issue in report.errors {
            issues.push(EditorLintIssue::from(issue));
        }
        for issue in report.warnings {
            issues.push(EditorLintIssue::from(issue));
        }
        for issue in report.info {
            issues.push(EditorLintIssue::from(issue));
        }

        Self {
            ok: report.ok,
            issues,
            error_count: report.summary.error_count,
            warning_count: report.summary.warning_count,
            info_count: report.summary.info_count,
        }
    }
}

impl Default for LintOutput {
    fn default() -> Self {
        Self {
            ok: true,
            issues: Vec::new(),
            error_count: 0,
            warning_count: 0,
            info_count: 0,
        }
    }
}

/// Run lint rules on asset data in memory.
///
/// # Arguments
/// * `asset_path` - Path hint for determining asset type (e.g., "sound.wav")
/// * `data` - Raw asset bytes
/// * `spec` - Optional spec for context-aware linting
///
/// # Returns
/// LintOutput with all detected issues.
pub fn lint_asset_bytes(
    asset_path: &Path,
    data: &[u8],
    spec: Option<&speccade_spec::Spec>,
) -> LintOutput {
    let registry = RuleRegistry::default_rules();
    match registry.lint_bytes(asset_path, data, spec) {
        Ok(report) => LintOutput::from(report),
        Err(e) => {
            // IO error during lint - return as single error
            let mut output = LintOutput::default();
            output.ok = false;
            output.error_count = 1;
            output.issues.push(EditorLintIssue {
                rule_id: "lint/io-error".to_string(),
                severity: "error".to_string(),
                message: format!("Failed to run lint: {}", e),
                suggestion: "Check that the asset was generated correctly.".to_string(),
                spec_path: None,
                actual_value: None,
                expected_range: None,
            });
            output
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_lint::{LintIssue, LintReport, Severity};

    #[test]
    fn test_lint_output_from_report() {
        let mut report = LintReport::new();
        report.add_issue(LintIssue::new(
            "audio/clipping",
            Severity::Error,
            "Clipping detected",
            "Reduce amplitude",
        ));
        report.add_issue(LintIssue::new(
            "audio/too-quiet",
            Severity::Warning,
            "Volume too low",
            "Increase gain",
        ));

        let output = LintOutput::from(report);
        assert!(!output.ok);
        assert_eq!(output.error_count, 1);
        assert_eq!(output.warning_count, 1);
        assert_eq!(output.issues.len(), 2);
    }

    #[test]
    fn test_empty_lint_output() {
        let output = LintOutput::default();
        assert!(output.ok);
        assert_eq!(output.error_count, 0);
        assert!(output.issues.is_empty());
    }
}
