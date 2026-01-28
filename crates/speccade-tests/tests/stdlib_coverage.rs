//! Stdlib coverage tests
//!
//! Policy: 100% coverage required. No allowlist. No exceptions.
//! Every stdlib function must have at least one golden example.

use regex::Regex;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::PathBuf;

/// Path to the golden stdlib snapshot file.
fn snapshot_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("golden")
        .join("stdlib")
        .join("stdlib.snapshot.json")
}

/// Path to the golden Starlark test fixtures directory.
fn golden_starlark_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("golden")
        .join("starlark")
}

/// Load all function names from the stdlib snapshot.
fn load_stdlib_function_names() -> Vec<String> {
    let path = snapshot_path();
    let contents = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to read stdlib snapshot at {}: {}\n\n\
             Run: cargo run -p speccade-cli -- stdlib dump --format json > golden/stdlib/stdlib.snapshot.json",
            path.display(),
            e
        )
    });

    let parsed: serde_json::Value = serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("Failed to parse stdlib snapshot as JSON: {}", e));

    parsed["functions"]
        .as_array()
        .expect("stdlib snapshot should have 'functions' array")
        .iter()
        .filter_map(|f| f["name"].as_str().map(String::from))
        .collect()
}

/// Scan all golden Starlark files and return the set of stdlib function names
/// that appear to be called (matched via regex).
fn find_called_functions(function_names: &[String]) -> HashSet<String> {
    let dir = golden_starlark_dir();
    let mut called = HashSet::new();

    // Build regex patterns for each function name
    // Pattern: word boundary + function name + optional whitespace + opening paren
    let patterns: Vec<(String, Regex)> = function_names
        .iter()
        .map(|name| {
            let pattern = format!(r"\b{}\s*\(", regex::escape(name));
            (name.clone(), Regex::new(&pattern).unwrap())
        })
        .collect();

    // Read all .star files in the golden/starlark directory
    let entries = fs::read_dir(&dir).unwrap_or_else(|e| {
        panic!(
            "Failed to read golden starlark directory {}: {}",
            dir.display(),
            e
        )
    });

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "star") {
            let contents = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: could not read {}: {}", path.display(), e);
                    continue;
                }
            };

            // Check each function pattern against the file contents
            for (name, pattern) in &patterns {
                if pattern.is_match(&contents) {
                    called.insert(name.clone());
                }
            }
        }
    }

    called
}

#[test]
fn all_stdlib_functions_are_covered() {
    let all_functions = load_stdlib_function_names();
    let called_functions = find_called_functions(&all_functions);

    // Find uncovered functions
    let uncovered: BTreeSet<&str> = all_functions
        .iter()
        .filter(|name| !called_functions.contains(*name))
        .map(String::as_str)
        .collect();

    // Print coverage summary
    let total = all_functions.len();
    let covered = called_functions.len();

    println!();
    println!("=== Stdlib Coverage Summary ===");
    println!("Total functions:     {}", total);
    println!("Called in tests:     {}", covered);
    println!(
        "Coverage:            {:.1}%",
        (covered as f64 / total as f64) * 100.0
    );
    println!();

    if !uncovered.is_empty() {
        println!("Uncovered functions ({}):", uncovered.len());
        for name in &uncovered {
            println!("  - {}", name);
        }
        println!();
        println!("To fix this:");
        println!("  Add golden Starlark files that use these functions.");
        println!("  Policy: 100% coverage required. No exceptions.");
        println!();
        panic!(
            "Stdlib coverage check failed: {} uncovered function(s).\n\
             Uncovered: {:?}",
            uncovered.len(),
            uncovered
        );
    }

    println!("All stdlib functions are covered.");
}

#[test]
fn coverage_report() {
    // This test just prints a detailed coverage report without failing
    let all_functions = load_stdlib_function_names();
    let called_functions = find_called_functions(&all_functions);

    println!();
    println!("=== Detailed Stdlib Coverage Report ===");
    println!();

    // Group functions by category
    let path = snapshot_path();
    let contents = fs::read_to_string(&path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();

    let mut by_category: std::collections::BTreeMap<String, Vec<(String, bool)>> =
        std::collections::BTreeMap::new();

    for func in parsed["functions"].as_array().unwrap() {
        let name = func["name"].as_str().unwrap().to_string();
        let category = func["category"].as_str().unwrap_or("unknown").to_string();
        let is_covered = called_functions.contains(&name);
        by_category
            .entry(category)
            .or_default()
            .push((name, is_covered));
    }

    for (category, functions) in &by_category {
        let covered_count = functions.iter().filter(|(_, c)| *c).count();
        println!("{} ({}/{})", category, covered_count, functions.len());
        for (name, is_covered) in functions {
            let marker = if *is_covered { "[x]" } else { "[ ]" };
            println!("  {} {}", marker, name);
        }
        println!();
    }
}
