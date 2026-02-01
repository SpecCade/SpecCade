//! Batch validation command
//!
//! Validates multiple assets in parallel

use anyhow::Result;
use colored::Colorize;
use glob::glob;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

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
    let batch_start = Instant::now();

    for (i, spec_path) in specs.iter().enumerate() {
        let item_start = Instant::now();
        let progress = format!("[{}/{}]", i + 1, specs.len()).cyan().bold();
        println!("\n{} {}", progress, spec_path.display());

        let spec_out_dir = out_dir.join(spec_path.file_stem().unwrap_or_default());
        std::fs::create_dir_all(&spec_out_dir)?;

        // Print progress bar
        let progress_pct = ((i + 1) as f64 / specs.len() as f64 * 100.0) as u32;
        let bar_width = 30;
        let filled = (progress_pct as usize * bar_width) / 100;
        let bar = format!(
            "[{}{}] {}%",
            "=".repeat(filled),
            " ".repeat(bar_width - filled),
            progress_pct
        );
        eprintln!("  Progress: {}", bar.dimmed());

        let result = validate_asset::run(
            spec_path.to_str().unwrap(),
            Some(spec_out_dir.to_str().unwrap()),
            true,
        );

        let elapsed = item_start.elapsed();
        let time_str = format!("{:.1}s", elapsed.as_secs_f64()).dimmed();

        match result {
            Ok(ExitCode::SUCCESS) => {
                println!(
                    "  {} {} {}",
                    "✓ PASS".green().bold(),
                    "•".dimmed(),
                    time_str
                );
                passed += 1;
                results.push(BatchResult {
                    spec: spec_path.to_string_lossy().to_string(),
                    success: true,
                    error: None,
                });
            }
            Ok(code) => {
                println!(
                    "  {} {} {} (exit code: {:?})",
                    "✗ FAIL".red().bold(),
                    "•".dimmed(),
                    time_str,
                    code
                );
                failed += 1;
                results.push(BatchResult {
                    spec: spec_path.to_string_lossy().to_string(),
                    success: false,
                    error: Some(format!("Exit code: {:?}", code)),
                });
            }
            Err(e) => {
                println!(
                    "  {} {} {}: {}",
                    "✗ ERROR".red().bold(),
                    "•".dimmed(),
                    time_str,
                    e
                );
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
    let total_elapsed = batch_start.elapsed();
    let avg_time = if !specs.is_empty() {
        total_elapsed.as_secs_f64() / specs.len() as f64
    } else {
        0.0
    };

    println!("\n{}", "=".repeat(60));
    println!(
        "{} {} {}",
        "Batch Validation Summary".bold(),
        "•".dimmed(),
        format!(
            "{:.1}s total, {:.1}s avg",
            total_elapsed.as_secs_f64(),
            avg_time
        )
        .dimmed()
    );
    println!("  Total:  {}", specs.len());
    println!(
        "  Passed: {}",
        if failed == 0 {
            passed.to_string().green()
        } else {
            passed.to_string().normal()
        }
    );
    println!(
        "  Failed: {}",
        if failed > 0 {
            failed.to_string().red()
        } else {
            failed.to_string().normal()
        }
    );
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
