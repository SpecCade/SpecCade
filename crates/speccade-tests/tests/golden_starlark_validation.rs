//! Golden Starlark validation tests
//!
//! This test validates ALL .star files in golden/starlark/ directory
//! to ensure they compile and validate successfully.

use speccade_cli::input::load_spec;
use std::fs;
use std::path::PathBuf;

/// Path to the golden Starlark test fixtures
fn golden_starlark_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("golden")
        .join("starlark")
}

#[test]
fn validate_all_golden_starlark_files() {
    let golden_dir = golden_starlark_path();

    let mut failures = Vec::new();
    let mut successes = 0;
    let mut files_tested = Vec::new();

    // Collect all .star files
    let entries: Vec<_> = fs::read_dir(&golden_dir)
        .expect("Failed to read golden/starlark directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map_or(false, |e| e == "star")
        })
        .collect();

    println!("\n=== Validating {} .star files ===\n", entries.len());

    for entry in entries {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        files_tested.push(filename.clone());

        match load_spec(&path) {
            Ok(result) => {
                successes += 1;
                println!("✓ {} (asset_id: {}, type: {:?})",
                    filename,
                    result.spec.asset_id,
                    result.spec.asset_type
                );

                // Additional validation checks
                if result.spec.asset_id.is_empty() {
                    failures.push((filename.clone(), "asset_id is empty".to_string()));
                }
                if result.spec.outputs.is_empty() {
                    failures.push((filename.clone(), "no outputs defined".to_string()));
                }
            }
            Err(e) => {
                failures.push((filename.clone(), format!("{}", e)));
                println!("✗ {}: {}", filename, e);
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
        panic!("{} golden .star files failed validation", failures.len());
    }

    println!("\nAll golden .star files validated successfully!");
}
