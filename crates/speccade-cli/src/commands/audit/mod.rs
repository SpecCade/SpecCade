//! Audio audit command implementation
//!
//! Provides audio quality regression detection by comparing current audio metrics
//! against baseline values. Supports tolerance configuration for pass/fail thresholds.

mod baseline;
mod types;

pub use baseline::AudioBaseline;
pub use types::{
    AuditFileResult, AuditMetrics, AuditOutput, AuditSummary, AuditTolerances, AuditViolation,
    ViolationKind,
};

use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use walkdir::WalkDir;

use crate::analysis::audio;

use super::json_output::{error_codes, JsonError};

/// Run the audit command.
///
/// # Arguments
/// * `input_dir` - Directory to scan for .wav files
/// * `tolerances_path` - Optional path to tolerances config file
/// * `update_baselines` - Whether to update/create baseline files
/// * `json_output` - Whether to output machine-readable JSON
///
/// # Returns
/// Exit code: 0 if all audits pass, 1 if any fail
pub fn run(
    input_dir: &str,
    tolerances_path: Option<&str>,
    update_baselines: bool,
    json_output: bool,
) -> Result<ExitCode> {
    // Load tolerances
    let tolerances = match tolerances_path {
        Some(path) => AuditTolerances::from_file(Path::new(path))?,
        None => AuditTolerances::default(),
    };

    let dir = Path::new(input_dir);
    if !dir.is_dir() {
        if json_output {
            let error = JsonError::new(
                error_codes::FILE_READ,
                format!("Input path is not a directory: {}", input_dir),
            );
            let output = AuditOutput::from_results(vec![], tolerances, vec![error]);
            println!("{}", serde_json::to_string_pretty(&output)?);
            return Ok(ExitCode::from(1));
        } else {
            anyhow::bail!("Input path is not a directory: {}", input_dir);
        }
    }

    // Find all .wav files
    let wav_files: Vec<_> = WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase() == "wav")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    // Sort for deterministic output
    let mut wav_files = wav_files;
    wav_files.sort();

    // Audit each file
    let mut results = Vec::new();
    for wav_path in &wav_files {
        let result = audit_file(wav_path, &tolerances, update_baselines);
        results.push(result);
    }

    // Output results
    if json_output {
        let output = AuditOutput::from_results(results, tolerances, vec![]);
        println!("{}", serde_json::to_string_pretty(&output)?);
        if output.success {
            Ok(ExitCode::SUCCESS)
        } else {
            Ok(ExitCode::from(1))
        }
    } else {
        print_results(&results, &tolerances, update_baselines);
        let failed = results.iter().filter(|r| !r.passed).count();
        if failed == 0 {
            Ok(ExitCode::SUCCESS)
        } else {
            Ok(ExitCode::from(1))
        }
    }
}

/// Audit a single audio file.
fn audit_file(
    wav_path: &Path,
    tolerances: &AuditTolerances,
    update_baseline: bool,
) -> AuditFileResult {
    let path_str = wav_path.display().to_string();

    // Read and analyze the WAV file
    let data = match fs::read(wav_path) {
        Ok(d) => d,
        Err(e) => {
            return AuditFileResult {
                path: path_str,
                passed: false,
                violations: vec![],
                metrics: None,
                baseline: None,
                error: Some(format!("Failed to read file: {}", e)),
            };
        }
    };

    let file_hash = blake3::hash(&data).to_hex().to_string();

    let metrics = match audio::analyze_wav(&data) {
        Ok(m) => m,
        Err(e) => {
            return AuditFileResult {
                path: path_str,
                passed: false,
                violations: vec![],
                metrics: None,
                baseline: None,
                error: Some(format!("Failed to analyze file: {}", e)),
            };
        }
    };

    let audit_metrics = AuditMetrics::from(&metrics);

    // Load or create baseline
    let baseline_file = baseline::baseline_path(wav_path);
    let loaded_baseline = if baseline_file.exists() {
        AudioBaseline::from_file(&baseline_file).ok()
    } else {
        None
    };

    // Update baseline if requested
    if update_baseline {
        let new_baseline = AudioBaseline::from_metrics(&metrics, Some(file_hash));
        if let Err(e) = new_baseline.save(&baseline_file) {
            return AuditFileResult {
                path: path_str,
                passed: false,
                violations: vec![],
                metrics: Some(audit_metrics),
                baseline: loaded_baseline.map(|b| AuditMetrics {
                    peak_db: b.peak_db,
                    rms_db: b.rms_db,
                    dc_offset: b.dc_offset,
                    clipping_detected: b.clipping_detected,
                }),
                error: Some(format!("Failed to write baseline: {}", e)),
            };
        }
    }

    // Check violations
    let mut violations = Vec::new();

    // Absolute threshold checks
    if metrics.quality.peak_db > tolerances.max_peak_db {
        violations.push(AuditViolation {
            kind: ViolationKind::PeakExceeded,
            message: format!(
                "Peak level {:.2} dB exceeds threshold {:.2} dB",
                metrics.quality.peak_db, tolerances.max_peak_db
            ),
            actual: Some(metrics.quality.peak_db),
            expected: Some(tolerances.max_peak_db),
            delta: None,
        });
    }

    if metrics.quality.dc_offset.abs() > tolerances.max_dc_offset {
        violations.push(AuditViolation {
            kind: ViolationKind::DcOffsetExceeded,
            message: format!(
                "DC offset {:.4} exceeds threshold {:.4}",
                metrics.quality.dc_offset, tolerances.max_dc_offset
            ),
            actual: Some(metrics.quality.dc_offset),
            expected: Some(tolerances.max_dc_offset),
            delta: None,
        });
    }

    if metrics.quality.clipping_detected && !tolerances.allow_clipping {
        violations.push(AuditViolation {
            kind: ViolationKind::ClippingDetected,
            message: "Clipping detected".to_string(),
            actual: None,
            expected: None,
            delta: None,
        });
    }

    // Regression checks against baseline
    if let Some(ref baseline) = loaded_baseline {
        let peak_delta = (metrics.quality.peak_db - baseline.peak_db).abs();
        if peak_delta > tolerances.peak_db_delta {
            violations.push(AuditViolation {
                kind: ViolationKind::PeakRegression,
                message: format!(
                    "Peak level changed by {:.2} dB (threshold: {:.2} dB)",
                    peak_delta, tolerances.peak_db_delta
                ),
                actual: Some(metrics.quality.peak_db),
                expected: Some(baseline.peak_db),
                delta: Some(peak_delta),
            });
        }

        let rms_delta = (metrics.quality.rms_db - baseline.rms_db).abs();
        if rms_delta > tolerances.rms_db_delta {
            violations.push(AuditViolation {
                kind: ViolationKind::RmsRegression,
                message: format!(
                    "RMS level changed by {:.2} dB (threshold: {:.2} dB)",
                    rms_delta, tolerances.rms_db_delta
                ),
                actual: Some(metrics.quality.rms_db),
                expected: Some(baseline.rms_db),
                delta: Some(rms_delta),
            });
        }

        let dc_delta = (metrics.quality.dc_offset - baseline.dc_offset).abs();
        if dc_delta > tolerances.dc_offset_delta {
            violations.push(AuditViolation {
                kind: ViolationKind::DcOffsetRegression,
                message: format!(
                    "DC offset changed by {:.4} (threshold: {:.4})",
                    dc_delta, tolerances.dc_offset_delta
                ),
                actual: Some(metrics.quality.dc_offset),
                expected: Some(baseline.dc_offset),
                delta: Some(dc_delta),
            });
        }

        if metrics.quality.clipping_detected != baseline.clipping_detected {
            violations.push(AuditViolation {
                kind: ViolationKind::ClippingRegression,
                message: format!(
                    "Clipping status changed from {} to {}",
                    baseline.clipping_detected, metrics.quality.clipping_detected
                ),
                actual: None,
                expected: None,
                delta: None,
            });
        }
    }

    let passed = violations.is_empty();

    AuditFileResult {
        path: path_str,
        passed,
        violations,
        metrics: Some(audit_metrics),
        baseline: loaded_baseline.map(|b| AuditMetrics {
            peak_db: b.peak_db,
            rms_db: b.rms_db,
            dc_offset: b.dc_offset,
            clipping_detected: b.clipping_detected,
        }),
        error: None,
    }
}

/// Print human-readable results.
fn print_results(results: &[AuditFileResult], tolerances: &AuditTolerances, updated: bool) {
    println!("{}", "Audio Audit Report".cyan().bold());
    println!("{}", "==================".dimmed());

    if updated {
        println!("{} Baselines updated\n", "NOTE:".yellow());
    }

    println!(
        "{} max_peak_db={:.1}, max_dc_offset={:.3}, allow_clipping={}",
        "Tolerances:".dimmed(),
        tolerances.max_peak_db,
        tolerances.max_dc_offset,
        tolerances.allow_clipping
    );
    println!(
        "{} peak_db_delta={:.1}, rms_db_delta={:.1}, dc_offset_delta={:.3}\n",
        "Deltas:".dimmed(),
        tolerances.peak_db_delta,
        tolerances.rms_db_delta,
        tolerances.dc_offset_delta
    );

    let mut passed = 0;
    let mut failed = 0;
    let mut errors = 0;

    for result in results {
        if result.error.is_some() {
            errors += 1;
            println!(
                "{} {} - {}",
                "ERROR".red(),
                result.path,
                result.error.as_ref().unwrap()
            );
        } else if result.passed {
            passed += 1;
            let baseline_info = if result.baseline.is_some() {
                "(baseline)"
            } else {
                "(no baseline)"
            };
            println!(
                "{} {} {}",
                "PASS".green(),
                result.path,
                baseline_info.dimmed()
            );
        } else {
            failed += 1;
            println!("{} {}", "FAIL".red().bold(), result.path);
            for violation in &result.violations {
                println!("  {} {}", "-".red(), violation.message);
            }
        }
    }

    println!("\n{}", "Summary".cyan().bold());
    println!("{}", "-------".dimmed());
    println!("Total:  {}", results.len());
    println!("Passed: {}", format!("{}", passed).green());
    if failed > 0 {
        println!("Failed: {}", format!("{}", failed).red());
    } else {
        println!("Failed: 0");
    }
    if errors > 0 {
        println!("Errors: {}", format!("{}", errors).red());
    } else {
        println!("Errors: 0");
    }

    if failed == 0 && errors == 0 {
        println!("\n{}", "All audits passed!".green().bold());
    } else {
        println!("\n{}", "Some audits failed.".red().bold());
    }
}

#[cfg(test)]
mod tests;
