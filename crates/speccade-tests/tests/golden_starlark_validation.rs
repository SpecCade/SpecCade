//! Starlark spec validation tests
//!
//! This test validates ALL .star files in the specs/ directory
//! to ensure they compile and validate successfully.

use speccade_cli::input::load_spec;
use speccade_spec::validation::{validate_spec_with_budget, BudgetProfile};
use std::fs;
use std::path::PathBuf;

/// Path to the Starlark specs directory
fn specs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("specs")
}

/// Recursively collect all .star files from a directory
fn collect_star_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return files,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_star_files(&path));
        } else if path.extension().map_or(false, |e| e == "star") {
            files.push(path);
        }
    }

    files
}

#[test]
fn validate_all_starlark_spec_files() {
    let specs_dir = specs_path();

    let mut failures = Vec::new();
    let mut successes = 0;
    let mut files_tested = Vec::new();

    // Collect all .star files recursively
    let star_files = collect_star_files(&specs_dir);

    println!("\n=== Validating {} .star files ===\n", star_files.len());

    for path in star_files {
        let relative_path = path.strip_prefix(&specs_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.file_name().unwrap().to_string_lossy().to_string());

        files_tested.push(relative_path.clone());

        match load_spec(&path) {
            Ok(result) => {
                successes += 1;
                println!(
                    "✓ {} (asset_id: {}, type: {:?})",
                    relative_path, result.spec.asset_id, result.spec.asset_type
                );

                // Additional validation checks
                if result.spec.asset_id.is_empty() {
                    failures.push((relative_path.clone(), "asset_id is empty".to_string()));
                }
                if result.spec.outputs.is_empty() {
                    failures.push((relative_path.clone(), "no outputs defined".to_string()));
                }
            }
            Err(e) => {
                failures.push((relative_path.clone(), format!("{}", e)));
                println!("✗ {}: {}", relative_path, e);
            }
        }
    }

    println!("\n=== Results ===");
    println!("Total files: {}", files_tested.len());
    println!("Passed: {}", successes);
    println!("Failed: {}", failures.len());

    if !failures.is_empty() {
        println!("\n=== Failures ===");
        for (file, error) in &failures {
            println!("\n--- {} ---", file);
            println!("{}", error);
        }
        panic!("{} .star spec files failed validation", failures.len());
    }

    println!("\nAll .star spec files validated successfully!");
}
