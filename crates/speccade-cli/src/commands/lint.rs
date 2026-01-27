//! Lint command implementation
//!
//! Runs semantic quality lint rules on generated assets.

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use speccade_lint::{LintReport, RuleRegistry, Severity};
use speccade_spec::report::{LintIssueData, LintReportData};
use std::path::Path;
use std::process::ExitCode;

/// Output format for lint results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!(
                "unknown format '{}', expected 'text' or 'json'",
                s
            )),
        }
    }
}

/// JSON output for lint command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintOutput {
    /// Whether the lint passed (no errors).
    pub success: bool,
    /// Path to the linted asset.
    pub asset_path: String,
    /// Path to the original spec (if provided).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_path: Option<String>,
    /// The lint report with all issues.
    pub report: LintReportData,
}

/// Converts a speccade_lint::LintReport to LintReportData.
pub(crate) fn lint_report_to_data(report: &LintReport) -> LintReportData {
    LintReportData {
        ok: report.ok,
        error_count: report.summary.error_count,
        warning_count: report.summary.warning_count,
        info_count: report.summary.info_count,
        errors: report.errors.iter().map(lint_issue_to_data).collect(),
        warnings: report.warnings.iter().map(lint_issue_to_data).collect(),
        info: report.info.iter().map(lint_issue_to_data).collect(),
    }
}

/// Converts a speccade_lint::LintIssue to LintIssueData.
pub fn lint_issue_to_data(issue: &speccade_lint::LintIssue) -> LintIssueData {
    LintIssueData {
        rule_id: issue.rule_id.clone(),
        severity: match issue.severity {
            Severity::Info => "info".to_string(),
            Severity::Warning => "warning".to_string(),
            Severity::Error => "error".to_string(),
        },
        message: issue.message.clone(),
        asset_location: issue.asset_location.clone(),
        spec_path: issue.spec_path.clone(),
        actual_value: issue.actual_value.clone(),
        expected_range: issue.expected_range.clone(),
        suggestion: issue.suggestion.clone(),
        fix_template: issue.fix_template.clone(),
        fix_delta: issue.fix_delta,
        fix_param: issue.fix_param.clone(),
    }
}

/// Run the lint command.
///
/// # Arguments
/// * `input` - Path to the asset file to lint
/// * `spec_path` - Optional path to the original spec (for spec_path in output)
/// * `strict` - Whether to fail on warnings (in addition to errors)
/// * `disable_rules` - Rule IDs to disable
/// * `only_rules` - If provided, only run these rules (comma-separated)
/// * `format` - Output format (text or json)
///
/// # Returns
/// Exit code: 0 if passed, 1 if errors (or warnings in strict mode)
pub fn run(
    input: &str,
    spec_path: Option<&str>,
    strict: bool,
    disable_rules: &[String],
    only_rules: Option<&str>,
    format: OutputFormat,
) -> Result<ExitCode> {
    let asset_path = Path::new(input);

    // Validate input file exists
    if !asset_path.exists() {
        if format == OutputFormat::Json {
            let output = LintOutput {
                success: false,
                asset_path: input.to_string(),
                spec_path: spec_path.map(|s| s.to_string()),
                report: LintReportData {
                    ok: false,
                    error_count: 1,
                    warning_count: 0,
                    info_count: 0,
                    errors: vec![LintIssueData::new(
                        "lint/file-not-found",
                        "error",
                        format!("File not found: {}", input),
                        "Check the file path and try again.",
                    )],
                    warnings: vec![],
                    info: vec![],
                },
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .expect("LintOutput serialization should not fail")
            );
        } else {
            eprintln!("{}: File not found: {}", "error".red().bold(), input);
        }
        return Ok(ExitCode::from(1));
    }

    // Create and configure rule registry
    let mut registry = RuleRegistry::default_rules();

    // Disable specified rules
    for rule_id in disable_rules {
        registry.disable_rule(rule_id);
    }

    // Enable only specified rules if provided
    if let Some(only) = only_rules {
        let rules: Vec<&str> = only.split(',').map(|s| s.trim()).collect();
        registry.enable_only(&rules);
    }

    // Load spec if provided (for spec_path context in issues)
    let spec = if let Some(spec_file) = spec_path {
        match crate::input::load_spec(Path::new(spec_file)) {
            Ok(result) => Some(result.spec),
            Err(e) => {
                if format == OutputFormat::Text {
                    eprintln!(
                        "{}: Could not load spec file {}: {}",
                        "warning".yellow().bold(),
                        spec_file,
                        e
                    );
                }
                None
            }
        }
    } else {
        None
    };

    // Run lint
    let report = registry
        .lint(asset_path, spec.as_ref())
        .with_context(|| format!("Failed to lint file: {}", input))?;

    // Determine success based on strict mode
    let success = if strict {
        report.ok && !report.has_warnings()
    } else {
        report.ok
    };

    // Output results
    if format == OutputFormat::Json {
        let output = LintOutput {
            success,
            asset_path: input.to_string(),
            spec_path: spec_path.map(|s| s.to_string()),
            report: lint_report_to_data(&report),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .expect("LintOutput serialization should not fail")
        );
    } else {
        print_text_output(input, &report, strict);
    }

    if success {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// Print lint results in human-readable text format.
fn print_text_output(input: &str, report: &LintReport, strict: bool) {
    println!("{} {}", "Linting:".cyan().bold(), input);

    if report.total_issues() == 0 {
        println!("\n{} No issues found", "PASSED".green().bold());
        return;
    }

    // Print errors
    if !report.errors.is_empty() {
        println!("\n{}", "Errors:".red().bold());
        for issue in &report.errors {
            print_lint_issue(issue, "x".red());
        }
    }

    // Print warnings
    if !report.warnings.is_empty() {
        println!("\n{}", "Warnings:".yellow().bold());
        for issue in &report.warnings {
            print_lint_issue(issue, "!".yellow());
        }
    }

    // Print info
    if !report.info.is_empty() {
        println!("\n{}", "Info:".blue().bold());
        for issue in &report.info {
            print_lint_issue(issue, "i".blue());
        }
    }

    // Print summary
    let summary = format!(
        "{} error(s), {} warning(s), {} info",
        report.summary.error_count, report.summary.warning_count, report.summary.info_count
    );

    if report.ok && (!strict || !report.has_warnings()) {
        println!("\n{} {}", "PASSED".green().bold(), summary.dimmed());
    } else {
        println!("\n{} {}", "FAILED".red().bold(), summary.dimmed());
    }
}

/// Print a single lint issue (public for use by generate's lint integration).
pub(crate) fn print_lint_issue(issue: &speccade_lint::LintIssue, marker: colored::ColoredString) {
    let location = issue
        .asset_location
        .as_ref()
        .map(|l| format!(" at {}", l))
        .unwrap_or_default();

    println!(
        "  {} [{}]{}: {}",
        marker,
        issue.rule_id.cyan(),
        location.dimmed(),
        issue.message
    );

    // Print actual/expected values if present
    if let Some(actual) = &issue.actual_value {
        if let Some(expected) = &issue.expected_range {
            println!(
                "    {} actual={}, expected={}",
                "->".dimmed(),
                actual,
                expected
            );
        } else {
            println!("    {} actual={}", "->".dimmed(), actual);
        }
    }

    // Print suggestion
    println!("    {} {}", "suggestion:".dimmed(), issue.suggestion);

    // Print fix template if present
    if let Some(template) = &issue.fix_template {
        println!("    {} {}", "fix:".dimmed(), template);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_parsing() {
        assert_eq!("text".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("TEXT".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
        assert_eq!("JSON".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert!("invalid".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_lint_report_data_conversion() {
        let report = LintReport::new();
        let data = lint_report_to_data(&report);
        assert!(data.ok);
        assert_eq!(data.error_count, 0);
        assert_eq!(data.warning_count, 0);
        assert_eq!(data.info_count, 0);
    }

    #[test]
    fn test_lint_issue_data_conversion() {
        use speccade_lint::LintIssue;

        let issue = LintIssue::new(
            "audio/test",
            Severity::Warning,
            "Test message",
            "Test suggestion",
        )
        .with_actual_value("1.5")
        .with_expected_range("[-1.0, 1.0]");

        let data = lint_issue_to_data(&issue);
        assert_eq!(data.rule_id, "audio/test");
        assert_eq!(data.severity, "warning");
        assert_eq!(data.message, "Test message");
        assert_eq!(data.suggestion, "Test suggestion");
        assert_eq!(data.actual_value, Some("1.5".to_string()));
        assert_eq!(data.expected_range, Some("[-1.0, 1.0]".to_string()));
    }
}
