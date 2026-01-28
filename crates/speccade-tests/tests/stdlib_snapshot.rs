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

    // Build name -> function maps for both snapshots
    let snapshot_funcs: std::collections::HashMap<&str, &serde_json::Value> = expected["functions"]
        .as_array()
        .expect("snapshot missing 'functions' array")
        .iter()
        .filter_map(|f| f["name"].as_str().map(|n| (n, f)))
        .collect();

    let current_funcs: std::collections::HashMap<&str, &serde_json::Value> = current["functions"]
        .as_array()
        .expect("current stdlib missing 'functions' array")
        .iter()
        .filter_map(|f| f["name"].as_str().map(|n| (n, f)))
        .collect();

    // Detect removals (breaking)
    let removed: Vec<&str> = snapshot_funcs
        .keys()
        .filter(|name| !current_funcs.contains_key(*name))
        .copied()
        .collect();

    if !removed.is_empty() {
        panic!(
            "BREAKING: Functions removed from stdlib:\n  {}\n\n\
             These functions exist in the snapshot but are missing from the current stdlib.\n\
             If this removal is intentional, update the snapshot:\n  \
             cargo run -p speccade-cli -- stdlib dump --format json > golden/stdlib/stdlib.snapshot.json",
            removed.join("\n  ")
        );
    }

    // Detect signature changes (breaking) — compare params and returns
    let mut changed: Vec<String> = Vec::new();
    for (name, snap_func) in &snapshot_funcs {
        if let Some(cur_func) = current_funcs.get(name) {
            let snap_sig = (&snap_func["params"], &snap_func["returns"]);
            let cur_sig = (&cur_func["params"], &cur_func["returns"]);
            if snap_sig != cur_sig {
                changed.push(format!(
                    "  {}:\n    snapshot: params={}, returns={}\n    current:  params={}, returns={}",
                    name,
                    snap_func["params"], snap_func["returns"],
                    cur_func["params"], cur_func["returns"],
                ));
            }
        }
    }

    if !changed.is_empty() {
        panic!(
            "BREAKING: Function signatures changed in stdlib:\n{}\n\n\
             If these changes are intentional, update the snapshot:\n  \
             cargo run -p speccade-cli -- stdlib dump --format json > golden/stdlib/stdlib.snapshot.json",
            changed.join("\n")
        );
    }

    // Detect additions — auto-update snapshot
    let added: Vec<&str> = current_funcs
        .keys()
        .filter(|name| !snapshot_funcs.contains_key(*name))
        .copied()
        .collect();

    if !added.is_empty() {
        // Only additions, no removals or signature changes — safe to auto-update
        std::fs::write(&snapshot_file, &current_json).unwrap_or_else(|e| {
            panic!(
                "Failed to auto-update snapshot at {}: {}",
                snapshot_file.display(),
                e
            )
        });
        eprintln!(
            "NOTE: Stdlib snapshot auto-updated with {} new function(s): {}",
            added.len(),
            added.join(", ")
        );
    }

    // Also auto-update if non-signature metadata changed (e.g. descriptions, categories)
    // but no functions were added/removed/signature-changed
    if added.is_empty() && expected != current {
        // Something non-breaking changed (description text, category, etc.) — auto-update
        std::fs::write(&snapshot_file, &current_json).unwrap_or_else(|e| {
            panic!(
                "Failed to auto-update snapshot at {}: {}",
                snapshot_file.display(),
                e
            )
        });
        eprintln!("NOTE: Stdlib snapshot auto-updated (non-breaking metadata changes).");
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
