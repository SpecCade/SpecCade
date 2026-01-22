//! Stdlib drift guard test using snapshot comparison.
//!
//! This test ensures that the Starlark stdlib does not change without
//! explicitly updating the golden snapshot. This helps prevent accidental
//! breaking changes to the stdlib API.

use speccade_cli::commands::stdlib::StdlibDump;

/// Path to the golden stdlib snapshot file.
fn snapshot_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("golden")
        .join("stdlib")
        .join("stdlib.snapshot.json")
}

#[test]
fn stdlib_matches_snapshot() {
    // Load the expected snapshot from disk
    let snapshot_file = snapshot_path();
    let snapshot_contents = std::fs::read_to_string(&snapshot_file).unwrap_or_else(|e| {
        panic!(
            "Failed to read stdlib snapshot at {}: {}\n\n\
             To create the snapshot, run:\n  \
             cargo run -p speccade-cli -- stdlib dump --format json > golden/stdlib/stdlib.snapshot.json",
            snapshot_file.display(),
            e
        )
    });

    let expected: serde_json::Value = serde_json::from_str(&snapshot_contents).unwrap_or_else(|e| {
        panic!(
            "Failed to parse stdlib snapshot as JSON: {}\n\n\
             The snapshot file may be corrupted. Regenerate it with:\n  \
             cargo run -p speccade-cli -- stdlib dump --format json > golden/stdlib/stdlib.snapshot.json",
            e
        )
    });

    // Generate the current stdlib dump
    let current_dump = StdlibDump::new();
    let current_json =
        serde_json::to_string_pretty(&current_dump).expect("Failed to serialize current stdlib");
    let current: serde_json::Value =
        serde_json::from_str(&current_json).expect("Failed to parse current stdlib as JSON");

    // Compare the two
    if expected != current {
        // Find specific differences for a helpful error message
        let expected_str = serde_json::to_string_pretty(&expected).unwrap();
        let current_str = serde_json::to_string_pretty(&current).unwrap();

        // Count function differences
        let expected_funcs = expected["functions"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);
        let current_funcs = current["functions"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);

        let expected_names: std::collections::HashSet<&str> = expected["functions"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|f| f["name"].as_str()).collect())
            .unwrap_or_default();

        let current_names: std::collections::HashSet<&str> = current["functions"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|f| f["name"].as_str()).collect())
            .unwrap_or_default();

        let added: Vec<_> = current_names.difference(&expected_names).collect();
        let removed: Vec<_> = expected_names.difference(&current_names).collect();

        let mut diff_summary = String::new();
        if expected_funcs != current_funcs {
            diff_summary.push_str(&format!(
                "Function count changed: {} -> {}\n",
                expected_funcs, current_funcs
            ));
        }
        if !added.is_empty() {
            diff_summary.push_str(&format!("Functions added: {:?}\n", added));
        }
        if !removed.is_empty() {
            diff_summary.push_str(&format!("Functions removed: {:?}\n", removed));
        }
        if diff_summary.is_empty() {
            diff_summary.push_str("Function signatures or metadata changed.\n");
        }

        // Show a unified diff of the first differences
        let expected_lines: Vec<&str> = expected_str.lines().collect();
        let current_lines: Vec<&str> = current_str.lines().collect();

        let mut diff_lines = Vec::new();
        let max_diff_lines = 20;
        let mut diff_count = 0;

        for (i, (exp, cur)) in expected_lines.iter().zip(current_lines.iter()).enumerate() {
            if exp != cur && diff_count < max_diff_lines {
                diff_lines.push(format!("Line {}: ", i + 1));
                diff_lines.push(format!("  - {}", exp));
                diff_lines.push(format!("  + {}", cur));
                diff_count += 1;
            }
        }

        // Handle length differences
        if expected_lines.len() != current_lines.len() && diff_count < max_diff_lines {
            diff_lines.push(format!(
                "Line count differs: {} vs {}",
                expected_lines.len(),
                current_lines.len()
            ));
        }

        panic!(
            "Stdlib has changed and does not match the snapshot.\n\n\
             {}\n\
             Diff preview:\n{}\n\n\
             If this change is intentional, update the snapshot by running:\n  \
             cargo run -p speccade-cli -- stdlib dump --format json > golden/stdlib/stdlib.snapshot.json",
            diff_summary,
            diff_lines.join("\n")
        );
    }
}

#[test]
fn snapshot_file_exists() {
    let path = snapshot_path();
    assert!(
        path.exists(),
        "Stdlib snapshot file does not exist at {}.\n\n\
         To create the snapshot, run:\n  \
         cargo run -p speccade-cli -- stdlib dump --format json > golden/stdlib/stdlib.snapshot.json",
        path.display()
    );
}

#[test]
fn snapshot_is_valid_json() {
    let path = snapshot_path();
    let contents = std::fs::read_to_string(&path).expect("Failed to read snapshot file");
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&contents);
    assert!(
        parsed.is_ok(),
        "Stdlib snapshot is not valid JSON: {}",
        parsed.unwrap_err()
    );
}

#[test]
fn snapshot_has_expected_structure() {
    let path = snapshot_path();
    let contents = std::fs::read_to_string(&path).expect("Failed to read snapshot file");
    let parsed: serde_json::Value =
        serde_json::from_str(&contents).expect("Failed to parse snapshot");

    assert!(
        parsed.get("stdlib_version").is_some(),
        "Snapshot missing 'stdlib_version' field"
    );
    assert!(
        parsed.get("functions").is_some(),
        "Snapshot missing 'functions' field"
    );
    assert!(
        parsed["functions"].is_array(),
        "'functions' field should be an array"
    );

    let functions = parsed["functions"].as_array().unwrap();
    assert!(
        !functions.is_empty(),
        "Snapshot should have at least one function"
    );

    // Check that each function has required fields
    for func in functions {
        assert!(
            func.get("name").is_some(),
            "Function missing 'name' field: {:?}",
            func
        );
        assert!(
            func.get("category").is_some(),
            "Function missing 'category' field: {:?}",
            func
        );
        assert!(
            func.get("description").is_some(),
            "Function missing 'description' field: {:?}",
            func
        );
        assert!(
            func.get("returns").is_some(),
            "Function missing 'returns' field: {:?}",
            func
        );
    }
}
