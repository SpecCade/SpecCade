//! Generate command implementation
//!
//! Generates assets from a spec file using the appropriate backend.

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_spec::{
    canonical_spec_hash, validate_for_generate, ReportBuilder, ReportError,
    Spec,
};
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use crate::dispatch::{dispatch_generate, DispatchError};
use super::reporting;

/// Run the generate command
///
/// # Arguments
/// * `spec_path` - Path to the spec JSON file
/// * `out_root` - Output root directory (default: current directory)
///
/// # Returns
/// Exit code: 0 success, 1 spec error, 2 generation error
pub fn run(spec_path: &str, out_root: Option<&str>) -> Result<ExitCode> {
    let start = Instant::now();
    let out_root = out_root.unwrap_or(".");

    println!(
        "{} {}",
        "Generating from:".cyan().bold(),
        spec_path
    );
    println!(
        "{} {}",
        "Output root:".cyan().bold(),
        out_root
    );

    // Read spec file
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;

    // Parse spec
    let spec = Spec::from_json(&spec_content)
        .with_context(|| format!("Failed to parse spec file: {}", spec_path))?;

    // Compute spec hash
    let spec_hash = canonical_spec_hash(&spec).unwrap_or_else(|_| "unknown".to_string());

    // Validate for generation (requires recipe)
    let validation_result = validate_for_generate(&spec);

    let backend_version = format!("speccade-cli v{}", env!("CARGO_PKG_VERSION"));

    if !validation_result.is_ok() {
        let duration_ms = start.elapsed().as_millis() as u64;

        // Build error report
        let mut report_builder = ReportBuilder::new(spec_hash, backend_version)
            .duration_ms(duration_ms);

        report_builder = reporting::apply_validation_messages(report_builder, &validation_result);

        let report = report_builder.ok(false).build();

        // Write report
        let report_path = reporting::report_path(spec_path, &spec.asset_id);
        reporting::write_report(&report, &report_path)?;

        // Print errors
        print_validation_errors(&validation_result);

        println!(
            "\n{} Spec validation failed with {} error(s)",
            "FAILED".red().bold(),
            validation_result.errors.len()
        );
        println!("{} {}", "Report written to:".dimmed(), report_path);

        return Ok(ExitCode::from(1));
    }

    // Print warnings if any
    if !validation_result.warnings.is_empty() {
        println!("\n{}", "Warnings:".yellow().bold());
        for warning in &validation_result.warnings {
            println!(
                "  {} [{}]: {}",
                "!".yellow(),
                warning.code.to_string().yellow(),
                warning.message
            );
        }
    }

    // Dispatch to backend
    println!("\n{}", "Dispatching to backend...".dimmed());

    match dispatch_generate(&spec, out_root, Path::new(spec_path)) {
        Ok(outputs) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let output_count = outputs.len();

            // Build success report
            let mut report_builder = ReportBuilder::new(spec_hash, backend_version)
                .duration_ms(duration_ms);

            report_builder = reporting::apply_validation_messages(report_builder, &validation_result);

            for output in outputs {
                report_builder = report_builder.output(output);
            }

            let report = report_builder.ok(true).build();

            // Write report
            let report_path = reporting::report_path(spec_path, &spec.asset_id);
            reporting::write_report(&report, &report_path)?;

            println!(
                "\n{} Generated {} output(s) in {}ms",
                "SUCCESS".green().bold(),
                output_count,
                duration_ms
            );
            println!("{} {}", "Report written to:".dimmed(), report_path);

            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;

            // Build error report
            let mut report_builder = ReportBuilder::new(spec_hash, backend_version)
                .duration_ms(duration_ms);

            // Add generation error
            let (code, message) = match &e {
                DispatchError::BackendNotImplemented(kind) => {
                    ("E013".to_string(), format!("Backend not implemented for recipe kind: {}", kind))
                }
                DispatchError::BackendError(msg) => {
                    ("E014".to_string(), format!("Backend execution failed: {}", msg))
                }
                DispatchError::NoRecipe => {
                    ("E010".to_string(), "Recipe required for generate command".to_string())
                }
            };

            report_builder = report_builder.error(ReportError::new(code, &message));

            report_builder = reporting::apply_validation_messages(report_builder, &validation_result);

            let report = report_builder.ok(false).build();

            // Write report
            let report_path = reporting::report_path(spec_path, &spec.asset_id);
            reporting::write_report(&report, &report_path)?;

            // Print error
            println!(
                "\n{} {}",
                "GENERATION FAILED".red().bold(),
                e
            );
            println!("{} {}", "Report written to:".dimmed(), report_path);

            Ok(ExitCode::from(2))
        }
    }
}

/// Print validation errors to the console
fn print_validation_errors(result: &speccade_spec::ValidationResult) {
    if !result.errors.is_empty() {
        println!("\n{}", "Validation Errors:".red().bold());
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
}
