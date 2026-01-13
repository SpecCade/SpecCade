//! Validate command implementation
//!
//! Validates a spec file and writes a report.

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_spec::{
    canonical_recipe_hash, canonical_spec_hash, validate_for_generate, validate_spec,
    ReportBuilder, Spec,
};
use std::fs;
use std::process::ExitCode;
use std::time::Instant;

use super::reporting;

/// Run the validate command
///
/// # Arguments
/// * `spec_path` - Path to the spec JSON file
/// * `artifacts` - Whether to also validate artifact references
///
/// # Returns
/// Exit code: 0 if valid, 1 if invalid
pub fn run(spec_path: &str, artifacts: bool) -> Result<ExitCode> {
    let start = Instant::now();

    println!("{} {}", "Validating:".cyan().bold(), spec_path);

    // Read spec file
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;

    // Parse spec
    let spec = Spec::from_json(&spec_content)
        .with_context(|| format!("Failed to parse spec file: {}", spec_path))?;

    // Compute spec hash
    let spec_hash = canonical_spec_hash(&spec).unwrap_or_else(|_| "unknown".to_string());
    let recipe_hash = spec
        .recipe
        .as_ref()
        .and_then(|r| canonical_recipe_hash(r).ok());

    // Run validation
    let validation_result = if artifacts {
        // Full validation including recipe requirements
        validate_for_generate(&spec)
    } else {
        // Contract-only validation
        validate_spec(&spec)
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    // Build report
    let backend_version = format!("speccade-cli v{}", env!("CARGO_PKG_VERSION"));
    let git_commit = option_env!("SPECCADE_GIT_SHA").map(|s| s.to_string());
    let git_dirty = matches!(option_env!("SPECCADE_GIT_DIRTY"), Some("1"));
    let mut report_builder = ReportBuilder::new(spec_hash.clone(), backend_version)
        .spec_metadata(&spec)
        .duration_ms(duration_ms);
    if let Some(commit) = git_commit {
        report_builder = report_builder.git_metadata(commit, git_dirty);
    }
    if let Some(hash) = recipe_hash {
        report_builder = report_builder.recipe_hash(hash);
    }

    report_builder = reporting::apply_validation_messages(report_builder, &validation_result);

    let report = report_builder.ok(validation_result.is_ok()).build();

    // Write report
    let report_path = reporting::report_path(spec_path, &spec.asset_id);
    reporting::write_report(&report, &report_path)?;

    // Print results
    print_validation_results(&validation_result, &report_path);

    if validation_result.is_ok() {
        println!(
            "\n{} Spec is valid ({}ms)",
            "SUCCESS".green().bold(),
            duration_ms
        );
        Ok(ExitCode::SUCCESS)
    } else {
        println!(
            "\n{} Spec has {} error(s) ({}ms)",
            "FAILED".red().bold(),
            validation_result.errors.len(),
            duration_ms
        );
        Ok(ExitCode::from(1))
    }
}

/// Print validation results to the console
fn print_validation_results(result: &speccade_spec::ValidationResult, report_path: &str) {
    if !result.errors.is_empty() {
        println!("\n{}", "Errors:".red().bold());
        for error in &result.errors {
            let path_info = error
                .path
                .as_ref()
                .map(|p| format!(" at {}", p))
                .unwrap_or_default();
            println!(
                "  {} [{}]{}: {}",
                "x".red(),
                error.code.to_string().red(),
                path_info.dimmed(),
                error.message
            );
        }
    }

    if !result.warnings.is_empty() {
        println!("\n{}", "Warnings:".yellow().bold());
        for warning in &result.warnings {
            let path_info = warning
                .path
                .as_ref()
                .map(|p| format!(" at {}", p))
                .unwrap_or_default();
            println!(
                "  {} [{}]{}: {}",
                "!".yellow(),
                warning.code.to_string().yellow(),
                path_info.dimmed(),
                warning.message
            );
        }
    }

    println!("\n{} {}", "Report written to:".dimmed(), report_path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Spec};

    fn write_spec(dir: &tempfile::TempDir, filename: &str, spec: &Spec) -> std::path::PathBuf {
        let path = dir.path().join(filename);
        std::fs::write(&path, spec.to_json_pretty().unwrap()).unwrap();
        path
    }

    #[test]
    fn validate_contract_only_allows_missing_recipe_and_writes_report() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("test-asset-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .description("test asset")
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        let code = run(spec_path.to_str().unwrap(), false).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        let report_path = reporting::report_path(spec_path.to_str().unwrap(), &spec.asset_id);
        let json = std::fs::read_to_string(&report_path).unwrap();
        let report: speccade_spec::Report = serde_json::from_str(&json).unwrap();
        assert!(report.ok);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn validate_artifacts_requires_recipe_and_reports_missing_recipe() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("test-asset-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .description("test asset")
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        let code = run(spec_path.to_str().unwrap(), true).unwrap();
        assert_eq!(code, ExitCode::from(1));

        let report_path = reporting::report_path(spec_path.to_str().unwrap(), &spec.asset_id);
        let json = std::fs::read_to_string(&report_path).unwrap();
        let report: speccade_spec::Report = serde_json::from_str(&json).unwrap();
        assert!(!report.ok);
        assert!(report.errors.iter().any(|e| e.code == "E010"));
    }
}
