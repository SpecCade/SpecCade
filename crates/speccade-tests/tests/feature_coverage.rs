//! Feature coverage enforcement tests
//!
//! Policy: 100% coverage required. No allowlist. No exceptions.
//! If a feature exists, it MUST have a golden example.

use std::path::PathBuf;

use speccade_tests::harness::TestHarness;

/// Get the workspace root directory (speccade project root)
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Run `speccade coverage generate --strict` and expect success
#[test]
fn coverage_is_complete() {
    let root = workspace_root();
    let harness = TestHarness::new();

    let output = harness.run_cli_in_dir(&root, &["coverage", "generate", "--strict"]);

    if !output.success {
        panic!(
            "Coverage check failed!\n\nstdout:\n{}\n\nstderr:\n{}\n\n\
             Run `cargo run -p speccade-cli -- coverage report` to see uncovered features.\n\
             Create examples in golden/starlark/ or golden/speccade/specs/.",
            output.stdout, output.stderr
        );
    }
}

/// Verify the coverage YAML file exists and is valid after generation
#[test]
fn coverage_yaml_is_valid() {
    let root = workspace_root();
    let harness = TestHarness::new();

    // First generate the coverage report
    let gen_output = harness.run_cli_in_dir(&root, &["coverage", "generate"]);

    if !gen_output.success {
        panic!("Coverage generate failed!\nstderr: {}", gen_output.stderr);
    }

    let yaml_path = root.join("docs/coverage/feature-coverage.yaml");

    assert!(
        yaml_path.exists(),
        "Coverage YAML not found at {}",
        yaml_path.display()
    );

    let content = std::fs::read_to_string(&yaml_path).expect("Failed to read coverage YAML");

    // Verify it's valid YAML by parsing it
    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Coverage YAML is not valid YAML");

    // Verify required fields exist
    assert!(
        parsed.get("schema_version").is_some(),
        "Missing schema_version"
    );
    assert!(parsed.get("summary").is_some(), "Missing summary");

    let summary = parsed.get("summary").unwrap();
    assert!(
        summary.get("total_features").is_some(),
        "Missing total_features in summary"
    );
    assert!(
        summary.get("covered").is_some(),
        "Missing covered in summary"
    );
    assert!(
        summary.get("coverage_percent").is_some(),
        "Missing coverage_percent in summary"
    );
}
