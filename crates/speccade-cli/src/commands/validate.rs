//! Validate command implementation
//!
//! Validates a spec file and writes a report.

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_spec::{
    canonical_spec_hash, validate_for_generate, validate_spec, ReportBuilder, ReportError,
    ReportWarning, Spec,
};
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

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

    println!(
        "{} {}",
        "Validating:".cyan().bold(),
        spec_path
    );

    // Read spec file
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;

    // Parse spec
    let spec = Spec::from_json(&spec_content)
        .with_context(|| format!("Failed to parse spec file: {}", spec_path))?;

    // Compute spec hash
    let spec_hash = canonical_spec_hash(&spec).unwrap_or_else(|_| "unknown".to_string());

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
    let mut report_builder = ReportBuilder::new(spec_hash.clone(), backend_version)
        .duration_ms(duration_ms)
        .target_triple(get_target_triple());

    // Add errors
    for error in &validation_result.errors {
        let report_error = if let Some(ref path) = error.path {
            ReportError::with_path(error.code.to_string(), &error.message, path)
        } else {
            ReportError::new(error.code.to_string(), &error.message)
        };
        report_builder = report_builder.error(report_error);
    }

    // Add warnings
    for warning in &validation_result.warnings {
        let report_warning = if let Some(ref path) = warning.path {
            ReportWarning::with_path(warning.code.to_string(), &warning.message, path)
        } else {
            ReportWarning::new(warning.code.to_string(), &warning.message)
        };
        report_builder = report_builder.warning(report_warning);
    }

    let report = report_builder.ok(validation_result.is_ok()).build();

    // Write report
    let report_path = get_report_path(spec_path, &spec.asset_id);
    write_report(&report, &report_path)?;

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
fn print_validation_results(
    result: &speccade_spec::ValidationResult,
    report_path: &str,
) {
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

/// Get the report file path based on spec path and asset_id
fn get_report_path(spec_path: &str, asset_id: &str) -> String {
    let spec_dir = Path::new(spec_path)
        .parent()
        .unwrap_or(Path::new("."));
    spec_dir
        .join(format!("{}.report.json", asset_id))
        .to_string_lossy()
        .to_string()
}

/// Write report to a JSON file
fn write_report(report: &speccade_spec::Report, path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(report)
        .context("Failed to serialize report")?;
    fs::write(path, json)
        .with_context(|| format!("Failed to write report to: {}", path))?;
    Ok(())
}

/// Get the current target triple
fn get_target_triple() -> String {
    // Build a target triple from compile-time information
    #[cfg(target_arch = "x86_64")]
    const ARCH: &str = "x86_64";
    #[cfg(target_arch = "x86")]
    const ARCH: &str = "i686";
    #[cfg(target_arch = "aarch64")]
    const ARCH: &str = "aarch64";
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
    const ARCH: &str = "unknown";

    #[cfg(target_os = "windows")]
    const OS: &str = "windows";
    #[cfg(target_os = "linux")]
    const OS: &str = "linux";
    #[cfg(target_os = "macos")]
    const OS: &str = "darwin";
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    const OS: &str = "unknown";

    #[cfg(target_env = "msvc")]
    const ENV: &str = "msvc";
    #[cfg(target_env = "gnu")]
    const ENV: &str = "gnu";
    #[cfg(not(any(target_env = "msvc", target_env = "gnu")))]
    const ENV: &str = "";

    if ENV.is_empty() {
        format!("{}-unknown-{}", ARCH, OS)
    } else {
        format!("{}-pc-{}-{}", ARCH, OS, ENV)
    }
}
