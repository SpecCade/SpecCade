//! Batch validation command
//!
//! Validates multiple assets in parallel

use anyhow::Result;
use glob::glob;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::commands::validate_asset;

/// Run batch validation
pub fn run(specs_pattern: &str, out_root: Option<&str>, _format: &str) -> Result<ExitCode> {
    let out_dir = if let Some(root) = out_root {
        PathBuf::from(root)
    } else {
        PathBuf::from("batch-validation")
    };

    std::fs::create_dir_all(&out_dir)?;

    // Find all matching spec files
    let specs: Vec<PathBuf> = glob(specs_pattern)?
        .filter_map(|e| e.ok())
        .filter(|p| {
            p.extension()
                .map(|e| e == "star" || e == "json")
                .unwrap_or(false)
        })
        .collect();

    if specs.is_empty() {
        println!("No specs found matching: {}", specs_pattern);
        return Ok(ExitCode::SUCCESS);
    }

    println!("Batch validating {} specs...", specs.len());
    println!("Output directory: {}", out_dir.display());

    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    for (i, spec_path) in specs.iter().enumerate() {
        println!("\n[{}/{}] {}", i + 1, specs.len(), spec_path.display());

        let spec_out_dir = out_dir.join(spec_path.file_stem().unwrap_or_default());
        std::fs::create_dir_all(&spec_out_dir)?;

        let result = validate_asset::run(
            spec_path.to_str().unwrap(),
            Some(spec_out_dir.to_str().unwrap()),
            true,
        );

        match result {
            Ok(ExitCode::SUCCESS) => {
                println!("  ✓ PASS");
                passed += 1;
                results.push(BatchResult {
                    spec: spec_path.to_string_lossy().to_string(),
                    success: true,
                    error: None,
                });
            }
            Ok(code) => {
                println!("  ✗ FAIL (exit code: {:?})", code);
                failed += 1;
                results.push(BatchResult {
                    spec: spec_path.to_string_lossy().to_string(),
                    success: false,
                    error: Some(format!("Exit code: {:?}", code)),
                });
            }
            Err(e) => {
                println!("  ✗ ERROR: {}", e);
                failed += 1;
                results.push(BatchResult {
                    spec: spec_path.to_string_lossy().to_string(),
                    success: false,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    // Generate batch report
    let batch_report = BatchReport {
        total: specs.len(),
        passed,
        failed,
        results,
    };

    let report_path = out_dir.join("batch-report.json");
    let report_json = serde_json::to_string_pretty(&batch_report)?;
    std::fs::write(&report_path, &report_json)?;

    // Print summary
    println!("\n{}", "=".repeat(60));
    println!("Batch Validation Summary:");
    println!("  Total:  {}", specs.len());
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("Report: {}", report_path.display());

    if failed > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

#[derive(Debug, serde::Serialize)]
struct BatchReport {
    total: usize,
    passed: usize,
    failed: usize,
    results: Vec<BatchResult>,
}

#[derive(Debug, serde::Serialize)]
struct BatchResult {
    spec: String,
    success: bool,
    error: Option<String>,
}
