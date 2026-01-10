//! Migrate command implementation
//!
//! Migrates legacy .spec.py files to canonical JSON format.

mod audit;
mod conversion;
mod legacy_parser;
mod reporting;

use anyhow::{bail, Result};
use colored::Colorize;
use std::path::Path;
use std::process::ExitCode;

// Re-export types needed by other modules
pub use audit::{AuditEntry, KeyClassification};
pub use conversion::MigrationEntry;

use audit::audit_spec;
use conversion::migrate_spec;
use legacy_parser::find_legacy_specs;
use reporting::{print_audit_report, print_migration_report};

/// Run the migrate command
///
/// # Arguments
/// * `project_path` - Path to the project directory containing legacy specs
/// * `allow_exec_specs` - Whether to allow Python execution for parsing
///
/// # Returns
/// Exit code: 0 success, 1 error
pub fn run(project_path: &str, allow_exec_specs: bool) -> Result<ExitCode> {
    println!("{} {}", "Migrate project:".cyan().bold(), project_path);

    if allow_exec_specs {
        println!(
            "{} {}",
            "WARNING:".yellow().bold(),
            "Python execution enabled. Only use with trusted files!".yellow()
        );
    }

    // Check if project path exists
    let path = Path::new(project_path);
    if !path.exists() {
        bail!("Project directory does not exist: {}", project_path);
    }

    if !path.is_dir() {
        bail!("Path is not a directory: {}", project_path);
    }

    // Find legacy specs
    let specs_dir = path.join(".studio").join("specs");
    if !specs_dir.exists() {
        bail!(
            "No .studio/specs directory found in project: {}",
            project_path
        );
    }

    println!("\n{} {}", "Scanning:".cyan(), specs_dir.display());

    let spec_files = find_legacy_specs(&specs_dir)?;

    if spec_files.is_empty() {
        println!("{} No legacy .spec.py files found.", "INFO".yellow().bold());
        return Ok(ExitCode::SUCCESS);
    }

    println!(
        "{} {} legacy spec files found\n",
        "Found:".cyan(),
        spec_files.len()
    );

    // Migrate each spec
    let mut entries = Vec::new();

    for spec_file in &spec_files {
        let entry = match migrate_spec(spec_file, path, allow_exec_specs) {
            Ok(entry) => entry,
            Err(e) => MigrationEntry {
                source_path: spec_file.to_path_buf(),
                target_path: None,
                success: false,
                warnings: Vec::new(),
                error: Some(e.to_string()),
                key_classification: KeyClassification::default(),
            },
        };

        // Print progress
        if entry.success && !entry.warnings.is_empty() {
            print!("{} ", "⚠".yellow());
        } else if entry.success {
            print!("{} ", "✓".green());
        } else {
            print!("{} ", "✗".red());
        }

        entries.push(entry);
    }

    println!("\n");

    // Generate report
    print_migration_report(&entries);

    // Return success if any files were converted
    let success_count = entries.iter().filter(|e| e.success).count();
    if success_count > 0 {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// Run the audit command (--audit mode)
///
/// Scans specs, parses them, collects legacy keys, and reports aggregate completeness.
///
/// # Arguments
/// * `project_path` - Path to the project directory containing legacy specs
/// * `allow_exec_specs` - Whether to allow Python execution for parsing
/// * `threshold` - Minimum completeness threshold (0.0-1.0)
///
/// # Returns
/// Exit code: 0 if completeness >= threshold, 1 otherwise
pub fn run_audit(project_path: &str, allow_exec_specs: bool, threshold: f64) -> Result<ExitCode> {
    println!("{} {}", "Audit project:".cyan().bold(), project_path);
    println!("{} {:.0}%", "Threshold:".cyan(), threshold * 100.0);

    if allow_exec_specs {
        println!(
            "{} {}",
            "WARNING:".yellow().bold(),
            "Python execution enabled. Only use with trusted files!".yellow()
        );
    }

    // Check if project path exists
    let path = Path::new(project_path);
    if !path.exists() {
        bail!("Project directory does not exist: {}", project_path);
    }

    if !path.is_dir() {
        bail!("Path is not a directory: {}", project_path);
    }

    // Find legacy specs
    let specs_dir = path.join(".studio").join("specs");
    if !specs_dir.exists() {
        bail!(
            "No .studio/specs directory found in project: {}",
            project_path
        );
    }

    println!("\n{} {}", "Scanning:".cyan(), specs_dir.display());

    let spec_files = find_legacy_specs(&specs_dir)?;

    if spec_files.is_empty() {
        println!("{} No legacy .spec.py files found.", "INFO".yellow().bold());
        return Ok(ExitCode::SUCCESS);
    }

    println!(
        "{} {} legacy spec files found\n",
        "Found:".cyan(),
        spec_files.len()
    );

    // Audit each spec (parse and classify keys, but don't migrate)
    let mut entries = Vec::new();
    let mut parse_errors = 0;

    for spec_file in &spec_files {
        let entry = match audit_spec(spec_file, allow_exec_specs) {
            Ok(entry) => entry,
            Err(e) => {
                parse_errors += 1;
                AuditEntry {
                    source_path: spec_file.to_path_buf(),
                    success: false,
                    error: Some(e.to_string()),
                    key_classification: KeyClassification::default(),
                }
            }
        };

        // Print progress
        if entry.success {
            print!("{} ", ".".dimmed());
        } else {
            print!("{} ", "!".red());
        }

        entries.push(entry);
    }

    println!("\n");

    // Generate audit report
    let completeness = print_audit_report(&entries, threshold);

    // Return appropriate exit code
    if parse_errors > 0 {
        // Hard failure: I/O or parse errors
        Ok(ExitCode::from(1))
    } else if completeness >= threshold {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests for the module structure
    // Individual function tests are in their respective submodules

    #[test]
    fn test_re_exports_available() {
        // Verify that re-exported types are accessible
        let _kc = KeyClassification::default();
        assert_eq!(_kc.total_used(), 0);
    }
}
