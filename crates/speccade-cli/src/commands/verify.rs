//! Verify command implementation
//!
//! Validates generated assets against user-defined constraints using metrics
//! from the generation report.

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_spec::{evaluate_constraints, ConstraintSet, OutputMetrics, Report, VerifyResult};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use super::json_output::{error_codes, JsonError};

/// JSON output for the verify command.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifyOutput {
    /// Whether verification succeeded (all constraints passed)
    pub success: bool,
    /// Errors encountered during verification
    pub errors: Vec<JsonError>,
    /// Verification result (on success or partial success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<VerifyResult>,
}

impl VerifyOutput {
    /// Creates a successful verify output.
    pub fn success(result: VerifyResult) -> Self {
        Self {
            success: result.overall_pass,
            errors: Vec::new(),
            result: Some(result),
        }
    }

    /// Creates a failed verify output (constraint failures).
    pub fn constraint_failure(result: VerifyResult) -> Self {
        Self {
            success: false,
            errors: Vec::new(),
            result: Some(result),
        }
    }

    /// Creates a failed verify output (errors before constraint evaluation).
    pub fn failure(errors: Vec<JsonError>) -> Self {
        Self {
            success: false,
            errors,
            result: None,
        }
    }
}

/// Run the verify command.
///
/// # Arguments
/// * `report_path` - Path to the *.report.json file
/// * `constraints_path` - Path to the *.constraints.json file
/// * `json_output` - Whether to output machine-readable JSON
///
/// # Returns
/// Exit code: 0 if all constraints pass, 1 if any fail or error occurs
pub fn run(report_path: &str, constraints_path: &str, json_output: bool) -> Result<ExitCode> {
    if json_output {
        run_json(report_path, constraints_path)
    } else {
        run_human(report_path, constraints_path)
    }
}

/// Run verify with human-readable (colored) output.
fn run_human(report_path: &str, constraints_path: &str) -> Result<ExitCode> {
    println!("{} {}", "Report:".cyan().bold(), report_path);
    println!("{} {}", "Constraints:".cyan().bold(), constraints_path);

    // Load report
    let report = load_report(Path::new(report_path))?;

    // Load constraints
    let constraints = load_constraints(Path::new(constraints_path))?;

    // Get asset ID from report
    let asset_id = report
        .asset_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    println!("{} {}", "Asset ID:".dimmed(), asset_id);
    println!("{} {}", "Constraints:".dimmed(), constraints.len());

    // Find metrics from report outputs
    let metrics = extract_metrics(&report);

    if metrics.is_none() {
        println!(
            "\n{} No output metrics found in report",
            "WARNING".yellow().bold()
        );
        println!("  The report may be from a Tier 1 backend that only produces hashes.");
        println!("  Constraint verification requires Tier 2 metrics.");

        // Create a result with all constraints skipped
        let empty_metrics = OutputMetrics::new();
        let result = evaluate_constraints(&asset_id, &empty_metrics, &constraints);

        print_results(&result);

        // All skipped = all pass (by default)
        if result.overall_pass {
            println!(
                "\n{} All constraints passed (skipped due to missing metrics)",
                "PASSED".green().bold()
            );
            return Ok(ExitCode::SUCCESS);
        }
    }

    let metrics = metrics.unwrap_or_default();

    // Evaluate constraints
    let result = evaluate_constraints(&asset_id, &metrics, &constraints);

    // Print results
    print_results(&result);

    if result.overall_pass {
        let passed_count = result.results.iter().filter(|r| r.passed).count();
        println!(
            "\n{} {}/{} constraints passed",
            "PASSED".green().bold(),
            passed_count,
            result.results.len()
        );
        Ok(ExitCode::SUCCESS)
    } else {
        let failed_count = result.results.iter().filter(|r| !r.passed).count();
        println!(
            "\n{} {}/{} constraints failed",
            "FAILED".red().bold(),
            failed_count,
            result.results.len()
        );
        Ok(ExitCode::from(1))
    }
}

/// Run verify with machine-readable JSON output.
fn run_json(report_path: &str, constraints_path: &str) -> Result<ExitCode> {
    // Load report
    let report = match load_report(Path::new(report_path)) {
        Ok(r) => r,
        Err(e) => {
            let error = JsonError::new(
                error_codes::FILE_READ,
                format!("Failed to load report: {}", e),
            )
            .with_file(report_path);
            let output = VerifyOutput::failure(vec![error]);
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .expect("VerifyOutput serialization should not fail")
            );
            return Ok(ExitCode::from(1));
        }
    };

    // Load constraints
    let constraints = match load_constraints(Path::new(constraints_path)) {
        Ok(c) => c,
        Err(e) => {
            let error = JsonError::new(
                error_codes::FILE_READ,
                format!("Failed to load constraints: {}", e),
            )
            .with_file(constraints_path);
            let output = VerifyOutput::failure(vec![error]);
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .expect("VerifyOutput serialization should not fail")
            );
            return Ok(ExitCode::from(1));
        }
    };

    // Get asset ID from report
    let asset_id = report
        .asset_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    // Find metrics from report outputs (or use empty if none)
    let metrics = extract_metrics(&report).unwrap_or_default();

    // Evaluate constraints
    let result = evaluate_constraints(&asset_id, &metrics, &constraints);

    // Build output
    let output = if result.overall_pass {
        VerifyOutput::success(result)
    } else {
        VerifyOutput::constraint_failure(result)
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("VerifyOutput serialization should not fail")
    );

    if output.success {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// Load a report from a JSON file.
fn load_report(path: &Path) -> Result<Report> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read report file: {}", path.display()))?;

    Report::from_json(&content)
        .with_context(|| format!("Failed to parse report JSON: {}", path.display()))
}

/// Load constraints from a JSON file.
fn load_constraints(path: &Path) -> Result<ConstraintSet> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read constraints file: {}", path.display()))?;

    ConstraintSet::from_json(&content)
        .with_context(|| format!("Failed to parse constraints JSON: {}", path.display()))
}

/// Extract metrics from report outputs.
/// Returns the first output with metrics, or None if no metrics are available.
fn extract_metrics(report: &Report) -> Option<OutputMetrics> {
    report
        .outputs
        .iter()
        .find_map(|output| output.metrics.clone())
}

/// Print verification results to the console.
fn print_results(result: &VerifyResult) {
    println!("\n{}", "Results:".bold());

    for r in &result.results {
        let status = if r.passed {
            "PASS".green()
        } else {
            "FAIL".red()
        };

        let actual_str = r
            .actual
            .as_ref()
            .map(|v| format!(" (actual: {})", v))
            .unwrap_or_default();

        println!("  {} {}{}", status, r.constraint, actual_str.dimmed());

        if let Some(msg) = &r.message {
            if !r.passed {
                println!("       {}", msg.yellow());
            } else if msg.contains("not available") {
                println!("       {}", msg.dimmed());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{Constraint, OutputMetrics, ReportBuilder};
    use tempfile::tempdir;

    fn create_test_report(metrics: Option<OutputMetrics>) -> Report {
        let mut builder =
            ReportBuilder::new("test-hash".to_string(), "test-backend v0.1.0".to_string());

        if let Some(m) = metrics {
            use speccade_spec::{OutputFormat, OutputKind, OutputResult};
            let output =
                OutputResult::tier2(OutputKind::Primary, OutputFormat::Glb, "test.glb".into(), m);
            builder = builder.output(output);
        }

        builder.ok(true).duration_ms(100).build()
    }

    fn write_report(
        dir: &tempfile::TempDir,
        filename: &str,
        report: &Report,
    ) -> std::path::PathBuf {
        let path = dir.path().join(filename);
        fs::write(&path, report.to_json_pretty().unwrap()).unwrap();
        path
    }

    fn write_constraints(
        dir: &tempfile::TempDir,
        filename: &str,
        constraints: &ConstraintSet,
    ) -> std::path::PathBuf {
        let path = dir.path().join(filename);
        fs::write(&path, constraints.to_json_pretty().unwrap()).unwrap();
        path
    }

    #[test]
    fn test_verify_all_pass() {
        let tmp = tempdir().unwrap();

        let metrics = OutputMetrics::new()
            .with_vertex_count(500)
            .with_face_count(250)
            .with_manifold(true);

        let report = create_test_report(Some(metrics));
        let report_path = write_report(&tmp, "test.report.json", &report);

        let constraints = ConstraintSet::from_constraints(vec![
            Constraint::MaxVertexCount { value: 1000 },
            Constraint::MaxFaceCount { value: 500 },
            Constraint::RequireManifold,
        ]);
        let constraints_path = write_constraints(&tmp, "test.constraints.json", &constraints);

        let code = run(
            report_path.to_str().unwrap(),
            constraints_path.to_str().unwrap(),
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn test_verify_some_fail() {
        let tmp = tempdir().unwrap();

        let metrics = OutputMetrics::new()
            .with_vertex_count(1500) // Exceeds limit
            .with_face_count(250)
            .with_manifold(false); // Fails manifold check

        let report = create_test_report(Some(metrics));
        let report_path = write_report(&tmp, "test.report.json", &report);

        let constraints = ConstraintSet::from_constraints(vec![
            Constraint::MaxVertexCount { value: 1000 },
            Constraint::MaxFaceCount { value: 500 },
            Constraint::RequireManifold,
        ]);
        let constraints_path = write_constraints(&tmp, "test.constraints.json", &constraints);

        let code = run(
            report_path.to_str().unwrap(),
            constraints_path.to_str().unwrap(),
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn test_verify_json_output_success() {
        let tmp = tempdir().unwrap();

        let metrics = OutputMetrics::new().with_vertex_count(500);

        let report = create_test_report(Some(metrics));
        let report_path = write_report(&tmp, "test.report.json", &report);

        let constraints =
            ConstraintSet::from_constraints(vec![Constraint::MaxVertexCount { value: 1000 }]);
        let constraints_path = write_constraints(&tmp, "test.constraints.json", &constraints);

        let code = run(
            report_path.to_str().unwrap(),
            constraints_path.to_str().unwrap(),
            true,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn test_verify_json_output_failure() {
        let tmp = tempdir().unwrap();

        let metrics = OutputMetrics::new().with_vertex_count(1500);

        let report = create_test_report(Some(metrics));
        let report_path = write_report(&tmp, "test.report.json", &report);

        let constraints =
            ConstraintSet::from_constraints(vec![Constraint::MaxVertexCount { value: 1000 }]);
        let constraints_path = write_constraints(&tmp, "test.constraints.json", &constraints);

        let code = run(
            report_path.to_str().unwrap(),
            constraints_path.to_str().unwrap(),
            true,
        )
        .unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn test_verify_missing_report() {
        let tmp = tempdir().unwrap();

        let constraints =
            ConstraintSet::from_constraints(vec![Constraint::MaxVertexCount { value: 1000 }]);
        let constraints_path = write_constraints(&tmp, "test.constraints.json", &constraints);

        let code = run(
            "/nonexistent/report.json",
            constraints_path.to_str().unwrap(),
            true,
        )
        .unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn test_verify_missing_constraints() {
        let tmp = tempdir().unwrap();

        let report = create_test_report(None);
        let report_path = write_report(&tmp, "test.report.json", &report);

        let code = run(
            report_path.to_str().unwrap(),
            "/nonexistent/constraints.json",
            true,
        )
        .unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn test_verify_no_metrics_all_skipped() {
        let tmp = tempdir().unwrap();

        // Report without metrics (Tier 1)
        let report = create_test_report(None);
        let report_path = write_report(&tmp, "test.report.json", &report);

        let constraints = ConstraintSet::from_constraints(vec![
            Constraint::MaxVertexCount { value: 1000 },
            Constraint::RequireManifold,
        ]);
        let constraints_path = write_constraints(&tmp, "test.constraints.json", &constraints);

        // Should pass because skipped constraints pass by default
        let code = run(
            report_path.to_str().unwrap(),
            constraints_path.to_str().unwrap(),
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn test_load_constraints_invalid_json() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("bad.constraints.json");
        fs::write(&path, "not valid json").unwrap();

        let result = load_constraints(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_report_invalid_json() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("bad.report.json");
        fs::write(&path, "not valid json").unwrap();

        let result = load_report(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_output_serialization() {
        let result = VerifyResult::new(
            "test-asset".to_string(),
            vec![speccade_spec::ConstraintResult::pass(
                &Constraint::MaxVertexCount { value: 1000 },
                Some(serde_json::json!(500)),
            )],
        );

        let output = VerifyOutput::success(result);
        let json = serde_json::to_string_pretty(&output).unwrap();

        assert!(json.contains("\"success\": true"));
        assert!(json.contains("\"asset_id\": \"test-asset\""));
        assert!(json.contains("\"overall_pass\": true"));
    }
}
