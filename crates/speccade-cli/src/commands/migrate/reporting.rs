//! Progress reporting and output formatting
//!
//! Handles formatted output for migration and audit reports.

use colored::Colorize;
use std::collections::HashMap;

use super::audit::{category_to_parity_section, AuditEntry, KeyClassification, MigrationKeyStatus};
use super::conversion::MigrationEntry;
use super::legacy_parser::determine_category;

/// Print audit report and return overall completeness score
pub fn print_audit_report(entries: &[AuditEntry], threshold: f64) -> f64 {
    let total = entries.len();
    let success = entries.iter().filter(|e| e.success).count();
    let failed = entries.iter().filter(|e| !e.success).count();

    println!("{}", "Audit Report".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();
    println!("{:20} {}", "Total files:", total);
    println!("{:20} {}", "Parsed:", format!("{}", success).green());
    println!("{:20} {}", "Failed:", format!("{}", failed).red());
    println!();

    // Collect all keys across all specs for aggregate analysis
    let successful_entries: Vec<_> = entries.iter().filter(|e| e.success).collect();

    if successful_entries.is_empty() {
        println!("{}", "No specs parsed successfully.".red());
        return 0.0;
    }

    // Compute aggregate key classification
    let mut agg = KeyClassification::default();
    // Track missing keys with frequency: (section, key) -> count
    let mut missing_keys: HashMap<(String, String), usize> = HashMap::new();

    for entry in &successful_entries {
        let kc = &entry.key_classification;
        agg.implemented += kc.implemented;
        agg.partial += kc.partial;
        agg.not_implemented += kc.not_implemented;
        agg.deprecated += kc.deprecated;
        agg.unknown += kc.unknown;

        // Collect missing keys (not_implemented and unknown)
        for (key, status) in &kc.key_details {
            if matches!(
                status,
                MigrationKeyStatus::NotImplemented | MigrationKeyStatus::Unknown
            ) {
                // Get section from the entry path
                let section = determine_category(&entry.source_path)
                    .map(|c| category_to_parity_section(&c).to_string())
                    .unwrap_or_else(|_| "unknown".to_string());

                *missing_keys.entry((section, key.clone())).or_insert(0) += 1;
            }
        }
    }

    // Print aggregate classification
    println!("{}", "Aggregate Key Classification:".cyan().bold());
    println!("{}", "-".repeat(60).dimmed());
    println!(
        "{:20} {} ({})",
        "Implemented:",
        format!("{}", agg.implemented).green(),
        "fully supported".green()
    );
    println!(
        "{:20} {} ({})",
        "Partial:",
        format!("{}", agg.partial).yellow(),
        "some features missing".yellow()
    );
    println!(
        "{:20} {} ({})",
        "Not Implemented:",
        format!("{}", agg.not_implemented).red(),
        "not yet supported".red()
    );
    println!(
        "{:20} {} ({})",
        "Unknown:",
        format!("{}", agg.unknown).dimmed(),
        "not in parity matrix".dimmed()
    );
    if agg.deprecated > 0 {
        println!(
            "{:20} {} ({})",
            "Deprecated:",
            format!("{}", agg.deprecated).dimmed(),
            "legacy keys, ignored".dimmed()
        );
    }
    println!();

    // Compute overall completeness score (gap score)
    let completeness = agg.gap_score().unwrap_or(0.0);
    let completeness_percent = completeness * 100.0;
    let threshold_percent = threshold * 100.0;

    let completeness_str = format!("{:.1}%", completeness_percent);
    let completeness_colored = if completeness >= threshold {
        completeness_str.green()
    } else if completeness >= threshold * 0.8 {
        completeness_str.yellow()
    } else {
        completeness_str.red()
    };

    println!(
        "{:20} {} (threshold: {:.0}%)",
        "Completeness:", completeness_colored, threshold_percent
    );
    println!(
        "{:20} {}",
        "",
        "(implemented + 0.5*partial) / total_used".dimmed()
    );
    println!();

    // Print top missing keys sorted by frequency
    if !missing_keys.is_empty() {
        println!("{}", "Top Missing Keys:".cyan().bold());
        println!("{}", "-".repeat(60).dimmed());

        // Sort by frequency (descending), then by key name
        let mut sorted_missing: Vec<_> = missing_keys.iter().collect();
        sorted_missing.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

        // Show top 10 missing keys
        let display_count = sorted_missing.len().min(10);
        for ((section, key), count) in sorted_missing.iter().take(display_count) {
            let qualified = format!("{}::{}", section, key);
            println!("  {:>4}x  {}", count, qualified.red());
        }

        if sorted_missing.len() > display_count {
            println!(
                "  {} {} more missing keys...",
                "...".dimmed(),
                sorted_missing.len() - display_count
            );
        }
        println!();
    }

    // Print parse errors if any
    if failed > 0 {
        println!("{}", "Parse Errors:".red().bold());
        for entry in entries {
            if !entry.success {
                println!(
                    "  {} {}",
                    "!".red(),
                    entry.source_path.display().to_string().dimmed()
                );
                if let Some(ref error) = entry.error {
                    println!("    {}", error);
                }
            }
        }
        println!();
    }

    // Print result summary
    println!("{}", "Result:".cyan().bold());
    if completeness >= threshold {
        println!(
            "  {} Completeness {:.1}% meets threshold {:.0}%",
            "PASS".green().bold(),
            completeness_percent,
            threshold_percent
        );
    } else {
        println!(
            "  {} Completeness {:.1}% below threshold {:.0}%",
            "FAIL".red().bold(),
            completeness_percent,
            threshold_percent
        );
    }

    completeness
}

/// Print migration report
pub fn print_migration_report(entries: &[MigrationEntry]) {
    let total = entries.len();
    let success = entries.iter().filter(|e| e.success).count();
    let with_warnings = entries.iter().filter(|e| !e.warnings.is_empty()).count();
    let failed = entries.iter().filter(|e| !e.success).count();

    println!("{}", "Migration Report".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();
    println!("{:20} {}", "Total files:", total);
    println!("{:20} {}", "Converted:", format!("{}", success).green());
    println!(
        "{:20} {}",
        "With warnings:",
        format!("{}", with_warnings).yellow()
    );
    println!("{:20} {}", "Failed:", format!("{}", failed).red());
    println!();

    // Print per-file key classification
    let successful_entries: Vec<_> = entries.iter().filter(|e| e.success).collect();
    if !successful_entries.is_empty() {
        println!("{}", "Key Classification (per file):".cyan().bold());
        println!("{}", "-".repeat(60).dimmed());
        println!(
            "{:<40} {:>4} {:>4} {:>4} {:>4} {:>7}",
            "File", "Impl", "Part", "Miss", "Unkn", "Gap"
        );
        println!("{}", "-".repeat(60).dimmed());

        for entry in &successful_entries {
            let kc = &entry.key_classification;
            let filename = entry
                .source_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "?".to_string());
            let truncated = if filename.len() > 38 {
                format!("{}...", &filename[..35])
            } else {
                filename
            };

            let gap_str = match kc.gap_score() {
                Some(score) => format!("{:.0}%", score * 100.0),
                None => "-".to_string(),
            };

            // Color the gap score based on value
            let gap_colored = match kc.gap_score() {
                Some(score) if score >= 0.8 => gap_str.green().to_string(),
                Some(score) if score >= 0.5 => gap_str.yellow().to_string(),
                Some(_) => gap_str.red().to_string(),
                None => gap_str.dimmed().to_string(),
            };

            println!(
                "{:<40} {:>4} {:>4} {:>4} {:>4} {:>7}",
                truncated.dimmed(),
                format!("{}", kc.implemented).green(),
                format!("{}", kc.partial).yellow(),
                format!("{}", kc.not_implemented).red(),
                format!("{}", kc.unknown).dimmed(),
                gap_colored
            );
        }
        println!();

        // Compute and print overall aggregated stats
        let mut agg = KeyClassification::default();
        for entry in &successful_entries {
            let kc = &entry.key_classification;
            agg.implemented += kc.implemented;
            agg.partial += kc.partial;
            agg.not_implemented += kc.not_implemented;
            agg.deprecated += kc.deprecated;
            agg.unknown += kc.unknown;
        }

        println!("{}", "Overall Key Classification:".cyan().bold());
        println!("{}", "-".repeat(60).dimmed());
        println!(
            "{:20} {} ({})",
            "Implemented:",
            format!("{}", agg.implemented).green(),
            "fully supported".green()
        );
        println!(
            "{:20} {} ({})",
            "Partial:",
            format!("{}", agg.partial).yellow(),
            "some features missing".yellow()
        );
        println!(
            "{:20} {} ({})",
            "Not Implemented:",
            format!("{}", agg.not_implemented).red(),
            "not yet supported".red()
        );
        println!(
            "{:20} {} ({})",
            "Unknown:",
            format!("{}", agg.unknown).dimmed(),
            "not in parity matrix".dimmed()
        );
        if agg.deprecated > 0 {
            println!(
                "{:20} {} ({})",
                "Deprecated:",
                format!("{}", agg.deprecated).dimmed(),
                "legacy keys, ignored".dimmed()
            );
        }
        println!();

        // Overall gap score
        if let Some(gap) = agg.gap_score() {
            let gap_percent = gap * 100.0;
            let gap_str = format!("{:.1}%", gap_percent);
            let gap_colored = if gap_percent >= 80.0 {
                gap_str.green()
            } else if gap_percent >= 50.0 {
                gap_str.yellow()
            } else {
                gap_str.red()
            };
            println!(
                "{:20} {} ({} keys used, {} deprecated)",
                "Overall Gap Score:",
                gap_colored,
                agg.total_used(),
                agg.deprecated
            );
            println!(
                "{:20} {}",
                "",
                "(implemented + 0.5*partial) / total_used".dimmed()
            );
        } else {
            println!("{:20} {}", "Overall Gap Score:", "-".dimmed());
        }
        println!();
    }

    if with_warnings > 0 {
        println!("{}", "Warnings:".yellow().bold());
        for entry in entries {
            if !entry.warnings.is_empty() {
                println!(
                    "  {} {}",
                    "⚠".yellow(),
                    entry.source_path.display().to_string().dimmed()
                );
                if let Some(ref target) = entry.target_path {
                    println!("    -> {}", target.display().to_string().dimmed());
                }
                for warning in &entry.warnings {
                    println!("    - {}", warning);
                }
            }
        }
        println!();
    }

    if failed > 0 {
        println!("{}", "Errors:".red().bold());
        for entry in entries {
            if !entry.success {
                println!(
                    "  {} {}",
                    "✗".red(),
                    entry.source_path.display().to_string().dimmed()
                );
                if let Some(ref target) = entry.target_path {
                    println!("    -> {}", target.display().to_string().dimmed());
                }
                if let Some(ref error) = entry.error {
                    println!("    {}", error);
                }
            }
        }
        println!();
    }

    println!("{}", "Next Steps:".cyan().bold());
    println!("  1. Review migrated specs in specs/ directory");
    println!("  2. Update license fields from 'UNKNOWN'");
    println!("  3. Review and address any warnings");
    println!("  4. Test with: speccade validate --spec specs/<type>/<asset>.json");
}

#[cfg(test)]
mod tests {
    use super::{print_audit_report, print_migration_report, AuditEntry, KeyClassification};
    use super::{MigrationEntry, MigrationKeyStatus};
    use std::path::PathBuf;

    #[test]
    fn print_audit_report_returns_zero_when_no_specs_parsed() {
        let score = print_audit_report(&[], 0.8);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn print_audit_report_computes_expected_gap_score() {
        let key_classification = KeyClassification {
            implemented: 1,
            partial: 1,
            not_implemented: 1,
            deprecated: 0,
            unknown: 1,
            key_details: vec![
                ("ok".to_string(), MigrationKeyStatus::Implemented),
                ("partial".to_string(), MigrationKeyStatus::Partial),
                ("missing".to_string(), MigrationKeyStatus::NotImplemented),
                ("unknown".to_string(), MigrationKeyStatus::Unknown),
            ],
        };

        let entries = vec![AuditEntry {
            source_path: PathBuf::from("specs/sounds/laser.spec.py"),
            success: true,
            error: None,
            key_classification,
        }];

        let score = print_audit_report(&entries, 0.8);
        assert!((score - 0.375).abs() < 1e-9, "score={score}");
    }

    #[test]
    fn print_migration_report_smoke() {
        let ok = MigrationEntry {
            source_path: PathBuf::from("specs/sounds/laser.spec.py"),
            target_path: Some(PathBuf::from("specs/audio/laser.json")),
            success: true,
            warnings: vec!["manual review".to_string()],
            error: None,
            key_classification: KeyClassification::default(),
        };

        let ok_no_warnings = MigrationEntry {
            source_path: PathBuf::from("specs/textures/metal.spec.py"),
            target_path: Some(PathBuf::from("specs/texture/metal.json")),
            success: true,
            warnings: vec![],
            error: None,
            key_classification: KeyClassification::default(),
        };

        let failed = MigrationEntry {
            source_path: PathBuf::from("specs/music/broken.spec.py"),
            target_path: None,
            success: false,
            warnings: vec![],
            error: Some("parse error".to_string()),
            key_classification: KeyClassification::default(),
        };

        print_migration_report(&[ok, ok_no_warnings, failed]);
    }
}
