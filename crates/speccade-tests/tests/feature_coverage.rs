//! Feature coverage enforcement tests
//!
//! Policy: 100% coverage required. No allowlist. No exceptions.
//! If a feature exists, it MUST have a golden example.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use speccade_tests::harness::TestHarness;
use tempfile::tempdir;

fn coverage_generate_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

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
    let _lock = coverage_generate_lock()
        .lock()
        .expect("coverage lock poisoned");

    let tmp = tempdir().expect("tempdir");
    let yaml_path = tmp.path().join("feature-coverage.yaml");
    let yaml_path = yaml_path
        .to_str()
        .expect("temp yaml path should be valid UTF-8");

    let output = harness.run_cli_in_dir(
        &root,
        &["coverage", "generate", "--strict", "--output", yaml_path],
    );

    if !output.success {
        panic!(
            "Coverage check failed!\n\nstdout:\n{}\n\nstderr:\n{}\n\n\
             Run `cargo run -p speccade-cli -- coverage report` to see uncovered features.\n\
             Create examples in specs/.",
            output.stdout, output.stderr
        );
    }
}

/// Verify the coverage YAML file exists and is valid after generation
#[test]
fn coverage_yaml_is_valid() {
    let root = workspace_root();
    let harness = TestHarness::new();
    let _lock = coverage_generate_lock()
        .lock()
        .expect("coverage lock poisoned");

    let tmp = tempdir().expect("tempdir");
    let yaml_path = tmp.path().join("feature-coverage.yaml");
    let yaml_path_str = yaml_path
        .to_str()
        .expect("temp yaml path should be valid UTF-8");

    // First generate the coverage report
    let gen_output =
        harness.run_cli_in_dir(&root, &["coverage", "generate", "--output", yaml_path_str]);

    if !gen_output.success {
        panic!("Coverage generate failed!\nstderr: {}", gen_output.stderr);
    }

    assert!(yaml_path.exists(), "Coverage YAML not generated");

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
