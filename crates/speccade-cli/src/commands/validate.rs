//! Validate command implementation
//!
//! Validates a spec file and writes a report.

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_spec::{
    canonical_recipe_hash, canonical_spec_hash, validate_for_generate_with_budget,
    validate_spec_with_budget, BudgetProfile, ReportBuilder,
};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use super::json_output::{
    compile_warnings_to_json, input_error_to_json, validation_error_to_json,
    validation_warning_to_json, JsonError, JsonWarning, ValidateOutput, ValidateResult,
};
use super::reporting;
use crate::input::{load_spec, LoadResult};

/// Run the validate command
///
/// # Arguments
/// * `spec_path` - Path to the spec file (JSON or Starlark)
/// * `artifacts` - Whether to also validate artifact references
/// * `budget_name` - Optional budget profile name (default, strict, zx-8bit)
/// * `json_output` - Whether to output machine-readable JSON diagnostics
///
/// # Returns
/// Exit code: 0 if valid, 1 if invalid
pub fn run(
    spec_path: &str,
    artifacts: bool,
    budget_name: Option<&str>,
    json_output: bool,
) -> Result<ExitCode> {
    if json_output {
        run_json(spec_path, artifacts, budget_name)
    } else {
        run_human(spec_path, artifacts, budget_name)
    }
}

/// Run validate with human-readable (colored) output
fn run_human(spec_path: &str, artifacts: bool, budget_name: Option<&str>) -> Result<ExitCode> {
    let start = Instant::now();

    // Parse budget profile
    let budget = match budget_name {
        Some(name) => BudgetProfile::by_name(name).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown budget profile: {} (expected default, strict, or zx-8bit)",
                name
            )
        })?,
        None => BudgetProfile::default(),
    };

    println!("{} {}", "Validating:".cyan().bold(), spec_path);
    if budget_name.is_some() {
        println!("{} {}", "Budget:".dimmed(), budget.name);
    }

    // Load spec file (JSON or Starlark)
    let LoadResult {
        spec,
        source_kind,
        source_hash,
        warnings: load_warnings,
    } = load_spec(Path::new(spec_path))
        .with_context(|| format!("Failed to load spec file: {}", spec_path))?;

    // Print any load warnings
    for warning in &load_warnings {
        let location = warning
            .location
            .as_ref()
            .map(|l| format!(" at {}", l))
            .unwrap_or_default();
        println!(
            "  {} [load]{}: {}",
            "!".yellow(),
            location.dimmed(),
            warning.message
        );
    }

    println!(
        "{} {} ({})",
        "Source:".dimmed(),
        source_kind.as_str(),
        &source_hash[..16]
    );

    // Compute spec hash
    let spec_hash = canonical_spec_hash(&spec).unwrap_or_else(|_| "unknown".to_string());
    let recipe_hash = spec
        .recipe
        .as_ref()
        .and_then(|r| canonical_recipe_hash(r).ok());

    // Run validation with budget
    let validation_result = if artifacts {
        // Full validation including recipe requirements
        validate_for_generate_with_budget(&spec, &budget)
    } else {
        // Contract-only validation
        validate_spec_with_budget(&spec, &budget)
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    // Build report
    let backend_version = format!("speccade-cli v{}", env!("CARGO_PKG_VERSION"));
    let git_commit = option_env!("SPECCADE_GIT_SHA").map(|s| s.to_string());
    let git_dirty = matches!(option_env!("SPECCADE_GIT_DIRTY"), Some("1"));
    let mut report_builder = ReportBuilder::new(spec_hash.clone(), backend_version)
        .spec_metadata(&spec)
        .source_provenance(source_kind.as_str(), &source_hash)
        .duration_ms(duration_ms);

    // Add stdlib version for Starlark sources
    #[cfg(feature = "starlark")]
    if source_kind == crate::input::SourceKind::Starlark {
        report_builder = report_builder.stdlib_version(crate::compiler::STDLIB_VERSION);
    }

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

/// Run validate with machine-readable JSON output
fn run_json(spec_path: &str, artifacts: bool, budget_name: Option<&str>) -> Result<ExitCode> {
    let start = Instant::now();

    // Parse budget profile
    let budget = match budget_name {
        Some(name) => match BudgetProfile::by_name(name) {
            Some(b) => b,
            None => {
                let error = JsonError::new(
                    super::json_output::error_codes::UNKNOWN_BUDGET,
                    format!(
                        "Unknown budget profile: {} (expected default, strict, or zx-8bit)",
                        name
                    ),
                );
                let output = ValidateOutput::failure(vec![error], vec![], None, None);
                let json = serde_json::to_string_pretty(&output)
                    .expect("ValidateOutput serialization should not fail");
                println!("{}", json);
                return Ok(ExitCode::from(1));
            }
        },
        None => BudgetProfile::default(),
    };

    // Load spec file (JSON or Starlark)
    let load_result = load_spec(Path::new(spec_path));

    let (spec, source_kind, source_hash, load_warnings) = match load_result {
        Ok(LoadResult {
            spec,
            source_kind,
            source_hash,
            warnings,
        }) => (spec, source_kind, source_hash, warnings),
        Err(e) => {
            let error = input_error_to_json(&e, Some(spec_path));
            let output = ValidateOutput::failure(vec![error], vec![], None, None);
            let json = serde_json::to_string_pretty(&output)
                .expect("ValidateOutput serialization should not fail");
            println!("{}", json);
            return Ok(ExitCode::from(1));
        }
    };

    // Compute spec hash
    let spec_hash = canonical_spec_hash(&spec).unwrap_or_else(|_| "unknown".to_string());
    let recipe_hash = spec
        .recipe
        .as_ref()
        .and_then(|r| canonical_recipe_hash(r).ok());

    // Run validation with budget
    let validation_result = if artifacts {
        validate_for_generate_with_budget(&spec, &budget)
    } else {
        validate_spec_with_budget(&spec, &budget)
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    // Build report (still write it for consistency)
    let backend_version = format!("speccade-cli v{}", env!("CARGO_PKG_VERSION"));
    let git_commit = option_env!("SPECCADE_GIT_SHA").map(|s| s.to_string());
    let git_dirty = matches!(option_env!("SPECCADE_GIT_DIRTY"), Some("1"));
    let mut report_builder = ReportBuilder::new(spec_hash.clone(), backend_version)
        .spec_metadata(&spec)
        .source_provenance(source_kind.as_str(), &source_hash)
        .duration_ms(duration_ms);

    #[cfg(feature = "starlark")]
    if source_kind == crate::input::SourceKind::Starlark {
        report_builder = report_builder.stdlib_version(crate::compiler::STDLIB_VERSION);
    }

    if let Some(commit) = git_commit {
        report_builder = report_builder.git_metadata(commit, git_dirty);
    }
    if let Some(hash) = recipe_hash.clone() {
        report_builder = report_builder.recipe_hash(hash);
    }

    report_builder = reporting::apply_validation_messages(report_builder, &validation_result);

    let report = report_builder.ok(validation_result.is_ok()).build();

    // Write report
    let report_path = reporting::report_path(spec_path, &spec.asset_id);
    reporting::write_report(&report, &report_path)?;

    // Collect warnings (from both load and validation)
    let mut warnings: Vec<JsonWarning> = compile_warnings_to_json(&load_warnings);
    warnings.extend(
        validation_result
            .warnings
            .iter()
            .map(validation_warning_to_json),
    );

    // Build JSON output
    let output = if validation_result.is_ok() {
        let result = ValidateResult {
            asset_id: spec.asset_id.clone(),
            asset_type: spec.asset_type.to_string(),
            source_kind: source_kind.as_str().to_string(),
            budget: budget_name.map(|s| s.to_string()),
            recipe_hash,
            report_path: Some(report_path),
            duration_ms,
        };
        ValidateOutput::success(result, spec_hash, source_hash, warnings)
    } else {
        let errors: Vec<JsonError> = validation_result
            .errors
            .iter()
            .map(validation_error_to_json)
            .collect();
        ValidateOutput::failure(errors, warnings, Some(spec_hash), Some(source_hash))
    };

    let json = serde_json::to_string_pretty(&output)
        .expect("ValidateOutput serialization should not fail");
    println!("{}", json);

    if output.success {
        Ok(ExitCode::SUCCESS)
    } else {
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

        let code = run(spec_path.to_str().unwrap(), false, None, false).unwrap();
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

        let spec = Spec::builder("test-asset-02", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .description("test asset")
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        let code = run(spec_path.to_str().unwrap(), true, None, false).unwrap();
        assert_eq!(code, ExitCode::from(1));

        let report_path = reporting::report_path(spec_path.to_str().unwrap(), &spec.asset_id);
        let json = std::fs::read_to_string(&report_path).unwrap();
        let report: speccade_spec::Report = serde_json::from_str(&json).unwrap();
        assert!(!report.ok);
        assert!(report.errors.iter().any(|e| e.code == "E010"));
    }

    #[test]
    fn validate_json_output_success() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("validate-json-test-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .description("test asset")
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        // Run with json=true - should succeed
        let code = run(spec_path.to_str().unwrap(), false, None, true).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn validate_json_output_failure() {
        // Run with json=true on nonexistent file - should return exit code 1
        let code = run("/nonexistent/spec.json", false, None, true).unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn validate_json_output_validation_errors() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("validate-error-test-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .description("test asset")
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        // Run with artifacts=true which requires a recipe - should fail
        let code = run(spec_path.to_str().unwrap(), true, None, true).unwrap();
        assert_eq!(code, ExitCode::from(1));
    }
}
