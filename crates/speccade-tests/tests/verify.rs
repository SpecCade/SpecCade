//! Integration tests for the `speccade verify` command.
//!
//! Tests verify:
//! - Constraint evaluation against report metrics
//! - JSON output format
//! - Error handling for missing files
//!
//! ## Running Tests
//!
//! ```bash
//! cargo test -p speccade-tests --test verify
//! ```

use std::fs;
use std::process::Command;

use tempfile::tempdir;

/// Helper to get the speccade CLI binary path.
fn speccade_binary() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("speccade");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// Creates a test report JSON with the given metrics.
fn create_report_json(
    asset_id: &str,
    vertex_count: Option<u32>,
    face_count: Option<u32>,
    manifold: Option<bool>,
) -> String {
    let mut metrics = serde_json::Map::new();
    if let Some(v) = vertex_count {
        metrics.insert("vertex_count".to_string(), serde_json::json!(v));
    }
    if let Some(f) = face_count {
        metrics.insert("face_count".to_string(), serde_json::json!(f));
    }
    if let Some(m) = manifold {
        metrics.insert("manifold".to_string(), serde_json::json!(m));
    }

    let report = serde_json::json!({
        "report_version": 1,
        "spec_hash": "abc123",
        "asset_id": asset_id,
        "asset_type": "static_mesh",
        "ok": true,
        "errors": [],
        "warnings": [],
        "outputs": [
            {
                "kind": "primary",
                "format": "glb",
                "path": "test.glb",
                "metrics": metrics
            }
        ],
        "duration_ms": 100,
        "backend_version": "test v0.1.0",
        "target_triple": "x86_64-pc-windows-msvc"
    });

    serde_json::to_string_pretty(&report).unwrap()
}

/// Creates a constraints JSON with the given constraints.
fn create_constraints_json(constraints: Vec<serde_json::Value>) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "constraints": constraints
    }))
    .unwrap()
}

#[test]
fn test_verify_all_pass_human_output() {
    let tmp = tempdir().unwrap();

    // Create report with metrics that pass all constraints
    let report_json = create_report_json("test-asset", Some(500), Some(250), Some(true));
    let report_path = tmp.path().join("test.report.json");
    fs::write(&report_path, &report_json).unwrap();

    // Create constraints
    let constraints_json = create_constraints_json(vec![
        serde_json::json!({"type": "max_vertex_count", "value": 1000}),
        serde_json::json!({"type": "max_face_count", "value": 500}),
        serde_json::json!({"type": "require_manifold"}),
    ]);
    let constraints_path = tmp.path().join("test.constraints.json");
    fs::write(&constraints_path, &constraints_json).unwrap();

    // Run verify command
    let output = Command::new(speccade_binary())
        .args([
            "verify",
            "--report",
            report_path.to_str().unwrap(),
            "--constraints",
            constraints_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute speccade");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Expected success, got: {}", stdout);
    assert!(
        stdout.contains("PASSED") || stdout.contains("PASS"),
        "Expected PASS in output: {}",
        stdout
    );
}

#[test]
fn test_verify_some_fail_human_output() {
    let tmp = tempdir().unwrap();

    // Create report with metrics that fail some constraints
    let report_json = create_report_json("test-asset", Some(1500), Some(250), Some(false));
    let report_path = tmp.path().join("test.report.json");
    fs::write(&report_path, &report_json).unwrap();

    // Create constraints
    let constraints_json = create_constraints_json(vec![
        serde_json::json!({"type": "max_vertex_count", "value": 1000}),
        serde_json::json!({"type": "max_face_count", "value": 500}),
        serde_json::json!({"type": "require_manifold"}),
    ]);
    let constraints_path = tmp.path().join("test.constraints.json");
    fs::write(&constraints_path, &constraints_json).unwrap();

    // Run verify command
    let output = Command::new(speccade_binary())
        .args([
            "verify",
            "--report",
            report_path.to_str().unwrap(),
            "--constraints",
            constraints_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute speccade");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!output.status.success(), "Expected failure, got: {}", stdout);
    assert!(
        stdout.contains("FAIL") || stdout.contains("FAILED"),
        "Expected FAIL in output: {}",
        stdout
    );
}

#[test]
fn test_verify_json_output_success() {
    let tmp = tempdir().unwrap();

    // Create report
    let report_json = create_report_json("json-test-asset", Some(500), None, None);
    let report_path = tmp.path().join("test.report.json");
    fs::write(&report_path, &report_json).unwrap();

    // Create constraints
    let constraints_json = create_constraints_json(vec![
        serde_json::json!({"type": "max_vertex_count", "value": 1000}),
    ]);
    let constraints_path = tmp.path().join("test.constraints.json");
    fs::write(&constraints_path, &constraints_json).unwrap();

    // Run verify command with --json
    let output = Command::new(speccade_binary())
        .args([
            "verify",
            "--report",
            report_path.to_str().unwrap(),
            "--constraints",
            constraints_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute speccade");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Expected success, got: {}", stdout);

    // Parse JSON output
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Failed to parse JSON output: {}\n{}", e, stdout));

    assert_eq!(json["success"], true);
    assert_eq!(json["result"]["asset_id"], "json-test-asset");
    assert_eq!(json["result"]["overall_pass"], true);
    assert!(json["result"]["results"].is_array());
}

#[test]
fn test_verify_json_output_failure() {
    let tmp = tempdir().unwrap();

    // Create report with vertex count exceeding limit
    let report_json = create_report_json("json-fail-asset", Some(1500), None, None);
    let report_path = tmp.path().join("test.report.json");
    fs::write(&report_path, &report_json).unwrap();

    // Create constraints
    let constraints_json = create_constraints_json(vec![
        serde_json::json!({"type": "max_vertex_count", "value": 1000}),
    ]);
    let constraints_path = tmp.path().join("test.constraints.json");
    fs::write(&constraints_path, &constraints_json).unwrap();

    // Run verify command with --json
    let output = Command::new(speccade_binary())
        .args([
            "verify",
            "--report",
            report_path.to_str().unwrap(),
            "--constraints",
            constraints_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute speccade");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!output.status.success(), "Expected failure, got: {}", stdout);

    // Parse JSON output
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Failed to parse JSON output: {}\n{}", e, stdout));

    assert_eq!(json["success"], false);
    assert_eq!(json["result"]["overall_pass"], false);

    // Check that the result contains the failed constraint
    let results = json["result"]["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["passed"], false);
    assert_eq!(results[0]["actual"], 1500);
    assert!(results[0]["message"].as_str().unwrap().contains("exceeds"));
}

#[test]
fn test_verify_missing_report_file() {
    let tmp = tempdir().unwrap();

    // Create constraints only
    let constraints_json = create_constraints_json(vec![
        serde_json::json!({"type": "max_vertex_count", "value": 1000}),
    ]);
    let constraints_path = tmp.path().join("test.constraints.json");
    fs::write(&constraints_path, &constraints_json).unwrap();

    // Run verify command with nonexistent report
    let output = Command::new(speccade_binary())
        .args([
            "verify",
            "--report",
            "/nonexistent/report.json",
            "--constraints",
            constraints_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute speccade");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!output.status.success(), "Expected failure");

    // Parse JSON output
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().unwrap().is_empty());
}

#[test]
fn test_verify_missing_constraints_file() {
    let tmp = tempdir().unwrap();

    // Create report only
    let report_json = create_report_json("test-asset", Some(500), None, None);
    let report_path = tmp.path().join("test.report.json");
    fs::write(&report_path, &report_json).unwrap();

    // Run verify command with nonexistent constraints
    let output = Command::new(speccade_binary())
        .args([
            "verify",
            "--report",
            report_path.to_str().unwrap(),
            "--constraints",
            "/nonexistent/constraints.json",
            "--json",
        ])
        .output()
        .expect("Failed to execute speccade");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!output.status.success(), "Expected failure");

    // Parse JSON output
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().unwrap().is_empty());
}

#[test]
fn test_verify_skipped_constraints() {
    let tmp = tempdir().unwrap();

    // Create report without vertex_count metric
    let report_json = create_report_json("skip-test-asset", None, Some(250), None);
    let report_path = tmp.path().join("test.report.json");
    fs::write(&report_path, &report_json).unwrap();

    // Create constraints that require vertex_count
    let constraints_json = create_constraints_json(vec![
        serde_json::json!({"type": "max_vertex_count", "value": 1000}),
        serde_json::json!({"type": "max_face_count", "value": 500}),
    ]);
    let constraints_path = tmp.path().join("test.constraints.json");
    fs::write(&constraints_path, &constraints_json).unwrap();

    // Run verify command with --json
    let output = Command::new(speccade_binary())
        .args([
            "verify",
            "--report",
            report_path.to_str().unwrap(),
            "--constraints",
            constraints_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute speccade");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should pass because skipped constraints pass by default
    assert!(output.status.success(), "Expected success, got: {}", stdout);

    // Parse JSON output
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["result"]["overall_pass"], true);

    // Check that the vertex_count constraint was skipped
    let results = json["result"]["results"].as_array().unwrap();
    let vertex_result = results
        .iter()
        .find(|r| r["constraint"].as_str().unwrap().contains("max_vertex_count"))
        .unwrap();
    assert_eq!(vertex_result["passed"], true);
    assert!(vertex_result["message"]
        .as_str()
        .unwrap()
        .contains("not available"));
}

#[test]
fn test_verify_all_constraint_types() {
    let tmp = tempdir().unwrap();

    // Create report with all metrics
    let report = serde_json::json!({
        "report_version": 1,
        "spec_hash": "abc123",
        "asset_id": "full-test-asset",
        "asset_type": "static_mesh",
        "ok": true,
        "errors": [],
        "warnings": [],
        "outputs": [
            {
                "kind": "primary",
                "format": "glb",
                "path": "test.glb",
                "metrics": {
                    "vertex_count": 500,
                    "face_count": 250,
                    "triangle_count": 500,
                    "quad_count": 200,
                    "quad_percentage": 80.0,
                    "manifold": true,
                    "non_manifold_edge_count": 0,
                    "degenerate_face_count": 0,
                    "uv_coverage": 0.75,
                    "uv_overlap_percentage": 2.0,
                    "bone_count": 32
                }
            }
        ],
        "duration_ms": 100,
        "backend_version": "test v0.1.0",
        "target_triple": "x86_64-pc-windows-msvc"
    });

    let report_path = tmp.path().join("test.report.json");
    fs::write(&report_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();

    // Create constraints for all types
    let constraints_json = create_constraints_json(vec![
        serde_json::json!({"type": "max_vertex_count", "value": 1000}),
        serde_json::json!({"type": "max_face_count", "value": 500}),
        serde_json::json!({"type": "max_triangle_count", "value": 1000}),
        serde_json::json!({"type": "min_quad_percentage", "value": 50.0}),
        serde_json::json!({"type": "require_manifold"}),
        serde_json::json!({"type": "max_non_manifold_edges", "value": 0}),
        serde_json::json!({"type": "max_degenerate_faces", "value": 0}),
        serde_json::json!({"type": "uv_coverage_min", "value": 0.5}),
        serde_json::json!({"type": "uv_overlap_max", "value": 5.0}),
        serde_json::json!({"type": "max_bone_count", "value": 64}),
    ]);
    let constraints_path = tmp.path().join("test.constraints.json");
    fs::write(&constraints_path, &constraints_json).unwrap();

    // Run verify command with --json
    let output = Command::new(speccade_binary())
        .args([
            "verify",
            "--report",
            report_path.to_str().unwrap(),
            "--constraints",
            constraints_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute speccade");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Expected success, got: {}", stdout);

    // Parse JSON output
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["result"]["overall_pass"], true);

    let results = json["result"]["results"].as_array().unwrap();
    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|r| r["passed"] == true));
}

#[test]
fn test_verify_invalid_constraints_json() {
    let tmp = tempdir().unwrap();

    // Create valid report
    let report_json = create_report_json("test-asset", Some(500), None, None);
    let report_path = tmp.path().join("test.report.json");
    fs::write(&report_path, &report_json).unwrap();

    // Create invalid constraints JSON
    let constraints_path = tmp.path().join("test.constraints.json");
    fs::write(&constraints_path, "not valid json").unwrap();

    // Run verify command with --json
    let output = Command::new(speccade_binary())
        .args([
            "verify",
            "--report",
            report_path.to_str().unwrap(),
            "--constraints",
            constraints_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute speccade");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!output.status.success(), "Expected failure");

    // Parse JSON output
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().unwrap().is_empty());
}

#[test]
fn test_verify_empty_constraints() {
    let tmp = tempdir().unwrap();

    // Create report
    let report_json = create_report_json("test-asset", Some(500), None, None);
    let report_path = tmp.path().join("test.report.json");
    fs::write(&report_path, &report_json).unwrap();

    // Create empty constraints
    let constraints_json = create_constraints_json(vec![]);
    let constraints_path = tmp.path().join("test.constraints.json");
    fs::write(&constraints_path, &constraints_json).unwrap();

    // Run verify command with --json
    let output = Command::new(speccade_binary())
        .args([
            "verify",
            "--report",
            report_path.to_str().unwrap(),
            "--constraints",
            constraints_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute speccade");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should pass with empty constraints
    assert!(output.status.success(), "Expected success, got: {}", stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["result"]["overall_pass"], true);
    assert!(json["result"]["results"].as_array().unwrap().is_empty());
}
