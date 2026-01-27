use anyhow::{Context, Result};
use colored::Colorize;
use speccade_lint::RuleRegistry;
use speccade_spec::report::LintReportData;
use speccade_spec::{OutputResult, Spec};
use std::fs;
use std::path::Path;

use crate::commands::lint::lint_report_to_data;

/// Run lint on generated output files and return the combined lint report data.
///
/// This is non-blocking: lint failures do not cause generation to fail.
/// Returns `None` if no lintable outputs exist or lint cannot run.
pub(crate) fn run_lint_on_outputs(
    outputs: &[OutputResult],
    spec: &Spec,
    out_root: &str,
    print_text: bool,
) -> Option<LintReportData> {
    let registry = RuleRegistry::default_rules();
    let mut combined_report: Option<speccade_lint::LintReport> = None;

    for output in outputs {
        let asset_path = Path::new(out_root).join(&output.path);
        if !asset_path.exists() {
            continue;
        }

        match registry.lint(&asset_path, Some(spec)) {
            Ok(report) => {
                if print_text && report.total_issues() > 0 {
                    let display_path = asset_path.to_string_lossy();
                    print_lint_issues(&display_path, &report);
                }
                match &mut combined_report {
                    Some(existing) => existing.merge(report),
                    None => combined_report = Some(report),
                }
            }
            Err(e) => {
                if print_text {
                    eprintln!(
                        "  {} Could not lint {}: {}",
                        "!".yellow(),
                        asset_path.display(),
                        e
                    );
                }
            }
        }
    }

    if let Some(ref report) = combined_report {
        if print_text {
            print_lint_text_summary(report);
        }
    }

    combined_report.map(|r| lint_report_to_data(&r))
}

/// Print lint issues for a single file (matches standalone lint command style).
fn print_lint_issues(path: &str, report: &speccade_lint::LintReport) {
    use crate::commands::lint::print_lint_issue;

    if !report.errors.is_empty() {
        println!("\n{} {}", "Lint errors:".red().bold(), path);
        for issue in &report.errors {
            print_lint_issue(issue, "x".red());
        }
    }
    if !report.warnings.is_empty() {
        println!("\n{} {}", "Lint warnings:".yellow().bold(), path);
        for issue in &report.warnings {
            print_lint_issue(issue, "!".yellow());
        }
    }
    if !report.info.is_empty() {
        println!("\n{} {}", "Lint info:".blue().bold(), path);
        for issue in &report.info {
            print_lint_issue(issue, "i".blue());
        }
    }
}

/// Print a lint summary line.
fn print_lint_text_summary(report: &speccade_lint::LintReport) {
    let summary = format!(
        "{} error(s), {} warning(s), {} info",
        report.summary.error_count, report.summary.warning_count, report.summary.info_count
    );
    if report.ok {
        println!("\n{} {}", "Lint PASSED".green().bold(), summary.dimmed());
    } else {
        println!("\n{} {}", "Lint FAILED".red().bold(), summary.dimmed());
    }
}

pub(crate) fn apply_validation_messages(
    mut builder: speccade_spec::ReportBuilder,
    validation: &speccade_spec::ValidationResult,
) -> speccade_spec::ReportBuilder {
    for err in &validation.errors {
        builder = builder.error(speccade_spec::ReportError::from_validation_error(err));
    }
    for warn in &validation.warnings {
        builder = builder.warning(speccade_spec::ReportWarning::from_validation_warning(warn));
    }
    builder
}

pub(crate) fn report_path(spec_path: &str, asset_id: &str) -> String {
    let spec_dir = Path::new(spec_path).parent().unwrap_or(Path::new("."));
    spec_dir
        .join(format!("{}.report.json", asset_id))
        .to_string_lossy()
        .to_string()
}

pub(crate) fn report_path_variant(spec_path: &str, asset_id: &str, variant_id: &str) -> String {
    let spec_dir = Path::new(spec_path).parent().unwrap_or(Path::new("."));
    spec_dir
        .join(format!("{}__{}.report.json", asset_id, variant_id))
        .to_string_lossy()
        .to_string()
}

pub(crate) fn write_report(report: &speccade_spec::Report, path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(report).context("Failed to serialize report")?;
    fs::write(path, json).with_context(|| format!("Failed to write report to: {}", path))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{
        ErrorCode, ValidationError, ValidationResult, ValidationWarning, WarningCode,
    };
    use speccade_spec::{OutputFormat, OutputKind, OutputResult, ReportBuilder};

    #[test]
    fn test_report_path_sibling_file() {
        let path = report_path("specs/audio/test.json", "laser-blast-01");
        // Use Path for platform-independent comparison
        let path = Path::new(&path);
        let expected = Path::new("specs")
            .join("audio")
            .join("laser-blast-01.report.json");
        assert_eq!(path, expected);
    }

    #[test]
    fn test_report_path_variant_sibling_file() {
        let path = report_path_variant("specs/audio/test.json", "laser-blast-01", "soft");
        let path = Path::new(&path);
        let expected = Path::new("specs")
            .join("audio")
            .join("laser-blast-01__soft.report.json");
        assert_eq!(path, expected);
    }

    #[test]
    fn test_write_report_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let report_path = tmp.path().join("report.json");

        let report = ReportBuilder::new("hash".to_string(), "backend v1.0".to_string())
            .ok(true)
            .output(OutputResult::tier1(
                OutputKind::Primary,
                OutputFormat::Wav,
                "sounds/test.wav".into(),
                "outhash".to_string(),
            ))
            .build();

        write_report(&report, report_path.to_str().unwrap()).unwrap();

        let json = fs::read_to_string(&report_path).unwrap();
        let parsed: speccade_spec::Report = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.report_version, report.report_version);
        assert_eq!(parsed.spec_hash, report.spec_hash);
        assert_eq!(parsed.outputs.len(), 1);
    }

    #[test]
    fn test_apply_validation_messages() {
        let validation = ValidationResult::failure_with_warnings(
            vec![ValidationError::new(ErrorCode::NoOutputs, "no outputs")],
            vec![ValidationWarning::new(
                WarningCode::MissingLicense,
                "missing",
            )],
        );

        let report = apply_validation_messages(
            ReportBuilder::new("hash".to_string(), "backend v1.0".to_string()),
            &validation,
        )
        .build();

        assert!(!report.ok);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.warnings.len(), 1);
    }
}
