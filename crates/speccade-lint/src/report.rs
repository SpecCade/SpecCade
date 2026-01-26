//! Lint report types for structured output.

use serde::{Deserialize, Serialize};

/// Severity level for lint issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Suggestions, stylistic preferences.
    Info,
    /// Likely problems, worth investigating.
    Warning,
    /// Definitely broken, should fail strict mode.
    Error,
}

/// Asset type for rule applicability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Audio,
    Texture,
    Mesh,
    Music,
}

/// A single lint issue detected in an asset.
///
/// Designed to be LLM-actionable with spec paths and fix templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    /// Unique identifier for this rule (e.g., "audio/clipping", "mesh/non-manifold").
    pub rule_id: String,

    /// Severity level.
    pub severity: Severity,

    /// Human-readable description of the issue.
    pub message: String,

    /// Location in generated asset (for humans/debugging).
    /// Examples: "layer[0]", "bone:upper_arm_l", "frequency_spectrum[8000..22050]"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_location: Option<String>,

    /// Location in source spec (for LLM edits).
    /// Examples: "recipe.params.layers[0].synthesis", "mesh.armature.bones[3]"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_path: Option<String>,

    /// Measured value that triggered the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_value: Option<String>,

    /// Expected or acceptable range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_range: Option<String>,

    /// Human-readable explanation of how to fix.
    pub suggestion: String,

    /// Starlark snippet to insert/replace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_template: Option<String>,

    /// Multiplier for numeric fixes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_delta: Option<f64>,

    /// Specific parameter to adjust.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_param: Option<String>,
}

impl LintIssue {
    /// Creates a new lint issue with required fields.
    pub fn new(rule_id: impl Into<String>, severity: Severity, message: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity,
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

    /// Builder method to set asset location.
    pub fn with_asset_location(mut self, location: impl Into<String>) -> Self {
        self.asset_location = Some(location.into());
        self
    }

    /// Builder method to set spec path.
    pub fn with_spec_path(mut self, path: impl Into<String>) -> Self {
        self.spec_path = Some(path.into());
        self
    }

    /// Builder method to set actual value.
    pub fn with_actual_value(mut self, value: impl Into<String>) -> Self {
        self.actual_value = Some(value.into());
        self
    }

    /// Builder method to set expected range.
    pub fn with_expected_range(mut self, range: impl Into<String>) -> Self {
        self.expected_range = Some(range.into());
        self
    }

    /// Builder method to set fix template.
    pub fn with_fix_template(mut self, template: impl Into<String>) -> Self {
        self.fix_template = Some(template.into());
        self
    }

    /// Builder method to set fix delta.
    pub fn with_fix_delta(mut self, delta: f64) -> Self {
        self.fix_delta = Some(delta);
        self
    }

    /// Builder method to set fix param.
    pub fn with_fix_param(mut self, param: impl Into<String>) -> Self {
        self.fix_param = Some(param.into());
        self
    }
}

/// Summary statistics for a lint run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LintSummary {
    /// Number of error-level issues.
    pub error_count: usize,
    /// Number of warning-level issues.
    pub warning_count: usize,
    /// Number of info-level issues.
    pub info_count: usize,
}

/// Complete lint report for a generation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintReport {
    /// True if no errors were found.
    pub ok: bool,
    /// Error-level issues (definitely broken).
    pub errors: Vec<LintIssue>,
    /// Warning-level issues (likely problems).
    pub warnings: Vec<LintIssue>,
    /// Info-level issues (suggestions).
    pub info: Vec<LintIssue>,
    /// Summary statistics.
    pub summary: LintSummary,
}

impl LintReport {
    /// Creates a new empty lint report.
    pub fn new() -> Self {
        Self {
            ok: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            summary: LintSummary::default(),
        }
    }

    /// Adds an issue to the report and updates the summary.
    pub fn add_issue(&mut self, issue: LintIssue) {
        match issue.severity {
            Severity::Info => {
                self.summary.info_count += 1;
                self.info.push(issue);
            }
            Severity::Warning => {
                self.summary.warning_count += 1;
                self.warnings.push(issue);
            }
            Severity::Error => {
                self.summary.error_count += 1;
                self.ok = false;
                self.errors.push(issue);
            }
        }
    }

    /// Merges another report into this one.
    pub fn merge(&mut self, other: LintReport) {
        for issue in other.errors {
            self.add_issue(issue);
        }
        for issue in other.warnings {
            self.add_issue(issue);
        }
        for issue in other.info {
            self.add_issue(issue);
        }
    }

    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.ok
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        self.summary.warning_count > 0
    }

    /// Returns the total issue count.
    pub fn total_issues(&self) -> usize {
        self.summary.error_count + self.summary.warning_count + self.summary.info_count
    }
}

impl Default for LintReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_issue_builder() {
        let issue = LintIssue::new(
            "audio/clipping",
            Severity::Error,
            "Sample values exceed ±1.0",
            "Reduce amplitude",
        )
        .with_actual_value("1.25")
        .with_expected_range("[-1.0, 1.0]")
        .with_fix_delta(0.8);

        assert_eq!(issue.rule_id, "audio/clipping");
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.actual_value, Some("1.25".to_string()));
        assert_eq!(issue.fix_delta, Some(0.8));
    }

    #[test]
    fn test_lint_report_add_issue() {
        let mut report = LintReport::new();
        assert!(report.ok);

        report.add_issue(LintIssue::new(
            "audio/too-quiet",
            Severity::Warning,
            "Volume too low",
            "Increase gain",
        ));
        assert!(report.ok);
        assert_eq!(report.summary.warning_count, 1);

        report.add_issue(LintIssue::new(
            "audio/clipping",
            Severity::Error,
            "Clipping detected",
            "Reduce amplitude",
        ));
        assert!(!report.ok);
        assert_eq!(report.summary.error_count, 1);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }
}
