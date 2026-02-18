//! Adversarial spec tests.
//!
//! Walks `specs/adversarial/` and verifies that each `.star` spec:
//! 1. Can be compiled from Starlark without panicking
//! 2. Fails validation (either at parse or validate_spec/validate_for_generate stage)
//! 3. Produces clear error messages (no panics)
//!
//! ## Running Tests
//!
//! ```bash
//! cargo test -p speccade-tests --test adversarial_specs
//! ```

use std::path::PathBuf;

use speccade_cli::input::load_spec;

fn adversarial_specs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("specs")
        .join("adversarial")
}

/// Collect all .star files from the adversarial specs directory.
fn collect_adversarial_specs() -> Vec<PathBuf> {
    let dir = adversarial_specs_dir();
    if !dir.exists() {
        panic!(
            "Adversarial specs directory does not exist: {}",
            dir.display()
        );
    }

    let mut specs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("Failed to read adversarial specs directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("star") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    specs.sort();
    assert!(
        !specs.is_empty(),
        "No .star files found in {}",
        dir.display()
    );
    specs
}

/// Test that all adversarial specs are present.
#[test]
fn adversarial_specs_exist() {
    let specs = collect_adversarial_specs();
    let names: Vec<String> = specs
        .iter()
        .map(|p| p.file_stem().unwrap().to_string_lossy().to_string())
        .collect();

    let expected = [
        "circular_graph_texture",
        "duplicate_bone_names",
        "empty_pattern_music",
        "empty_skeleton_character",
        "max_channels_music",
        "missing_graph_node_texture",
        "zero_duration_audio",
        "zero_resolution_texture",
    ];

    for name in &expected {
        assert!(
            names.contains(&name.to_string()),
            "Missing adversarial spec: {}.star",
            name
        );
    }
}

/// Test each adversarial spec: it must either fail to parse/compile
/// or fail validation. It must never panic.
#[test]
fn adversarial_specs_fail_validation() {
    let specs = collect_adversarial_specs();

    for spec_path in &specs {
        let spec_name = spec_path.file_stem().unwrap().to_string_lossy();
        eprintln!("Testing adversarial spec: {}", spec_name);

        // Step 1: Try to load the spec (Starlark compilation + JSON parse).
        // Some specs may fail at this stage (e.g., if they produce invalid JSON structure).
        let load_result = load_spec(spec_path);

        match load_result {
            Err(e) => {
                // Failed to parse/compile — this is an acceptable outcome for adversarial specs
                // as long as we got a clear error (no panic).
                eprintln!("  -> Parse/compile error (OK): {}", e);
            }
            Ok(result) => {
                // Step 2: Run validate_spec.
                let validation = speccade_spec::validate_spec(&result.spec);

                if !validation.is_ok() {
                    eprintln!(
                        "  -> validate_spec failed (OK): {:?}",
                        validation
                            .errors
                            .iter()
                            .map(|e| format!("{}: {}", e.code, e.message))
                            .collect::<Vec<_>>()
                    );
                    continue;
                }

                // Step 3: Run validate_for_generate (stricter).
                let gen_validation = speccade_spec::validate_for_generate(&result.spec);

                if !gen_validation.is_ok() {
                    eprintln!(
                        "  -> validate_for_generate failed (OK): {:?}",
                        gen_validation
                            .errors
                            .iter()
                            .map(|e| format!("{}: {}", e.code, e.message))
                            .collect::<Vec<_>>()
                    );
                    continue;
                }

                // If we get here, the spec passed both validation stages.
                // This is unexpected for adversarial specs — flag it.
                panic!(
                    "Adversarial spec '{}' unexpectedly passed all validation stages!\n\
                     Spec: {:?}",
                    spec_name, result.spec.asset_id
                );
            }
        }
    }
}

/// Test that adversarial specs never cause panics during load + validate cycle.
/// This test is separate to make the intent clear.
#[test]
fn adversarial_specs_no_panics() {
    let specs = collect_adversarial_specs();

    for spec_path in &specs {
        // Catch panics during load.
        let result = std::panic::catch_unwind(|| {
            let load_result = load_spec(spec_path);
            if let Ok(result) = load_result {
                let _ = speccade_spec::validate_spec(&result.spec);
                let _ = speccade_spec::validate_for_generate(&result.spec);
            }
        });

        assert!(
            result.is_ok(),
            "Adversarial spec {:?} caused a panic!",
            spec_path.file_stem().unwrap()
        );
    }
}
