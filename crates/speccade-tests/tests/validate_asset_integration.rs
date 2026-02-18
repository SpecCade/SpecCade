//! Integration tests for the validate-asset command
//!
//! Tests verify:
//! - validate-asset works with static mesh specs
//! - validate-asset rejects non-3D assets (audio)
//! - Validation report structure and quality gates
//!
//! ## Running Tests
//!
//! ```bash
//! cargo test -p speccade-tests --test validate_asset_integration
//! ```

use std::fs;

use speccade_tests::harness::TestHarness;

/// Test that validate-asset accepts static mesh specs
#[test]
fn test_validate_asset_static_mesh() {
    let harness = TestHarness::new();

    // Create a minimal static mesh spec
    let spec_content = r#"{
        "spec_version": 1,
        "asset_id": "test-mesh-validation",
        "asset_type": "static_mesh",
        "license": "CC0-1.0",
        "seed": 42,
        "recipe": {
            "kind": "static_mesh.blender_primitives_v1",
            "params": {
                "base_primitive": "cube",
                "dimensions": [1.0, 1.0, 1.0]
            }
        },
        "outputs": [
            {
                "kind": "primary",
                "format": "glb",
                "path": "test-mesh.glb"
            }
        ]
    }"#;

    let spec_path = harness.path().join("test_static_mesh.json");
    fs::write(&spec_path, spec_content).unwrap();

    // Run validate-asset
    let result = harness.run_cli(&[
        "validate-asset",
        "--spec",
        spec_path.to_str().unwrap(),
        "--out-root",
        harness.path().join("output").to_str().unwrap(),
    ]);

    // If Blender is not available, the test should be skipped
    if result.stderr.contains("Blender") || result.stderr.contains("not found") {
        println!("Skipping test: Blender not available");
        return;
    }

    // The command should either succeed or fail with a generation error (not a validation error)
    if !result.success {
        // Check that it's not a "wrong asset type" error
        assert!(
            !result.stderr.contains("only supports 3D assets"),
            "validate-asset should accept static_mesh: {}",
            result.stderr
        );
    }
}

/// Test that validate-asset rejects audio specs (only 3D assets supported)
#[test]
fn test_validate_asset_rejects_audio() {
    let harness = TestHarness::new();

    // Create an audio spec
    let spec_content = r#"{
        "spec_version": 1,
        "asset_id": "test-audio-validation",
        "asset_type": "audio",
        "license": "CC0-1.0",
        "seed": 42,
        "recipe": {
            "kind": "audio.sfx_preset",
            "params": {
                "preset": "sine_beep",
                "duration_ms": 500
            }
        },
        "outputs": [
            {
                "kind": "primary",
                "format": "wav",
                "path": "test-audio.wav"
            }
        ]
    }"#;

    let spec_path = harness.path().join("test_audio.json");
    fs::write(&spec_path, spec_content).unwrap();

    // Run validate-asset
    let result = harness.run_cli(&[
        "validate-asset",
        "--spec",
        spec_path.to_str().unwrap(),
        "--out-root",
        harness.path().join("output").to_str().unwrap(),
    ]);

    // Should fail with "only supports 3D assets" error
    assert!(!result.success, "validate-asset should reject audio assets");
    assert!(
        result.stderr.contains("only supports 3D assets")
            || result.stdout.contains("only supports 3D assets"),
        "Error should mention 3D assets: stderr={}, stdout={}",
        result.stderr,
        result.stdout
    );
}

/// Test that validate-asset report structure is valid
#[test]
fn test_validation_report_structure() {
    let harness = TestHarness::new();

    // Create a minimal static mesh spec
    let spec_content = r#"{
        "spec_version": 1,
        "asset_id": "test-report-structure",
        "asset_type": "static_mesh",
        "license": "CC0-1.0",
        "seed": 42,
        "recipe": {
            "kind": "static_mesh.blender_primitives_v1",
            "params": {
                "base_primitive": "cube",
                "dimensions": [1.0, 1.0, 1.0]
            }
        },
        "outputs": [
            {
                "kind": "primary",
                "format": "glb",
                "path": "test-mesh.glb"
            }
        ]
    }"#;

    let spec_path = harness.path().join("test_report.json");
    fs::write(&spec_path, spec_content).unwrap();

    let out_dir = harness.path().join("output");
    fs::create_dir_all(&out_dir).unwrap();

    // Run validate-asset
    let result = harness.run_cli(&[
        "validate-asset",
        "--spec",
        spec_path.to_str().unwrap(),
        "--out-root",
        out_dir.to_str().unwrap(),
    ]);

    // If Blender is not available, skip
    if result.stderr.contains("Blender") || result.stderr.contains("not found") {
        println!("Skipping test: Blender not available");
        return;
    }

    // Check if report was generated
    let report_path = out_dir.join("test-report-structure.validation-report.json");

    if report_path.exists() {
        let report_content = fs::read_to_string(&report_path).unwrap();
        let report: serde_json::Value = serde_json::from_str(&report_content).unwrap();

        // Verify report structure
        assert!(
            report.get("spec_path").is_some(),
            "Report should have spec_path"
        );
        assert!(
            report.get("asset_id").is_some(),
            "Report should have asset_id"
        );
        assert!(
            report.get("asset_type").is_some(),
            "Report should have asset_type"
        );
        assert!(
            report.get("timestamp").is_some(),
            "Report should have timestamp"
        );
        assert!(
            report.get("generation").is_some(),
            "Report should have generation"
        );
        assert!(
            report.get("visual_evidence").is_some(),
            "Report should have visual_evidence"
        );
        assert!(
            report.get("metrics").is_some(),
            "Report should have metrics"
        );
        assert!(
            report.get("lint_results").is_some(),
            "Report should have lint_results"
        );
        assert!(
            report.get("quality_gates").is_some(),
            "Report should have quality_gates"
        );
        assert!(
            report.get("validation_comments").is_some(),
            "Report should have validation_comments"
        );

        // Verify quality gates structure
        let gates = report.get("quality_gates").unwrap();
        assert!(
            gates.get("generation").is_some(),
            "Quality gates should have generation"
        );
        assert!(
            gates.get("has_geometry").is_some(),
            "Quality gates should have has_geometry"
        );
        assert!(
            gates.get("manifold").is_some(),
            "Quality gates should have manifold"
        );
        assert!(
            gates.get("has_uvs").is_some(),
            "Quality gates should have has_uvs"
        );
    }
}

/// Test that validate-asset accepts skeletal mesh specs
#[test]
fn test_validate_asset_skeletal_mesh() {
    let harness = TestHarness::new();

    // Create a minimal skeletal mesh spec
    let spec_content = r#"{
        "spec_version": 1,
        "asset_id": "test-skeletal-validation",
        "asset_type": "skeletal_mesh",
        "license": "CC0-1.0",
        "seed": 42,
        "recipe": {
            "kind": "skeletal_mesh.simple",
            "params": {
                "base_mesh": "humanoid",
                "bone_count": 4
            }
        },
        "outputs": [
            {
                "kind": "primary",
                "format": "glb",
                "path": "test-skeletal.glb"
            }
        ]
    }"#;

    let spec_path = harness.path().join("test_skeletal.json");
    fs::write(&spec_path, spec_content).unwrap();

    // Run validate-asset
    let result = harness.run_cli(&[
        "validate-asset",
        "--spec",
        spec_path.to_str().unwrap(),
        "--out-root",
        harness.path().join("output").to_str().unwrap(),
    ]);

    // If Blender is not available, the test should be skipped
    if result.stderr.contains("Blender") || result.stderr.contains("not found") {
        println!("Skipping test: Blender not available");
        return;
    }

    // Should not fail with "wrong asset type" error
    if !result.success {
        assert!(
            !result.stderr.contains("only supports 3D assets"),
            "validate-asset should accept skeletal_mesh: {}",
            result.stderr
        );
    }
}

/// Test that validate-asset accepts animation specs
#[test]
fn test_validate_asset_skeletal_animation() {
    let harness = TestHarness::new();

    // Create a minimal animation spec
    let spec_content = r#"{
        "spec_version": 1,
        "asset_id": "test-animation-validation",
        "asset_type": "skeletal_animation",
        "license": "CC0-1.0",
        "seed": 42,
        "recipe": {
            "kind": "skeletal_animation.idle",
            "params": {
                "duration_seconds": 2.0,
                "loop": true
            }
        },
        "outputs": [
            {
                "kind": "primary",
                "format": "glb",
                "path": "test-animation.glb"
            }
        ]
    }"#;

    let spec_path = harness.path().join("test_animation.json");
    fs::write(&spec_path, spec_content).unwrap();

    // Run validate-asset
    let result = harness.run_cli(&[
        "validate-asset",
        "--spec",
        spec_path.to_str().unwrap(),
        "--out-root",
        harness.path().join("output").to_str().unwrap(),
    ]);

    // If Blender is not available, the test should be skipped
    if result.stderr.contains("Blender") || result.stderr.contains("not found") {
        println!("Skipping test: Blender not available");
        return;
    }

    // Should not fail with "wrong asset type" error
    if !result.success {
        assert!(
            !result.stderr.contains("only supports 3D assets"),
            "validate-asset should accept skeletal_animation: {}",
            result.stderr
        );
    }
}

/// Test that validate-asset rejects texture specs
#[test]
fn test_validate_asset_rejects_texture() {
    let harness = TestHarness::new();

    // Create a texture spec
    let spec_content = r#"{
        "spec_version": 1,
        "asset_id": "test-texture-validation",
        "asset_type": "texture",
        "license": "CC0-1.0",
        "seed": 42,
        "recipe": {
            "kind": "texture.procedural",
            "params": {
                "pattern": "checkerboard",
                "size": 256
            }
        },
        "outputs": [
            {
                "kind": "primary",
                "format": "png",
                "path": "test-texture.png"
            }
        ]
    }"#;

    let spec_path = harness.path().join("test_texture.json");
    fs::write(&spec_path, spec_content).unwrap();

    // Run validate-asset
    let result = harness.run_cli(&[
        "validate-asset",
        "--spec",
        spec_path.to_str().unwrap(),
        "--out-root",
        harness.path().join("output").to_str().unwrap(),
    ]);

    // Should fail with "only supports 3D assets" error
    assert!(
        !result.success,
        "validate-asset should reject texture assets"
    );
    assert!(
        result.stderr.contains("only supports 3D assets")
            || result.stdout.contains("only supports 3D assets"),
        "Error should mention 3D assets: stderr={}, stdout={}",
        result.stderr,
        result.stdout
    );
}
