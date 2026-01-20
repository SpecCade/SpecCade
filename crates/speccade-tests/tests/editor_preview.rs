//! Tests for CLI preview artifacts (EDITOR-002).

use std::fs;

use speccade_spec::{OutputFormat, OutputKind, Report};
use speccade_tests::harness::TestHarness;

#[test]
fn generate_emits_waveform_preview_and_reports_it() {
    let harness = TestHarness::new();

    let spec_path = harness.path().join("audio_preview.json");
    fs::write(
        &spec_path,
        r#"{
  "spec_version": 1,
  "asset_id": "editor-preview-audio-01",
  "asset_type": "audio",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    { "kind": "primary", "format": "wav", "path": "audio/editor_preview.wav" }
  ],
  "recipe": {
    "kind": "audio_v1",
    "params": {
      "duration_seconds": 0.2,
      "sample_rate": 22050,
      "layers": [
        {
          "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
          "envelope": { "attack": 0.01, "decay": 0.03, "sustain": 0.5, "release": 0.05 },
          "volume": 0.8,
          "pan": 0.0
        }
      ]
    }
  }
}"#,
    )
    .unwrap();

    let cli = harness.run_cli(&[
        "generate",
        "--spec",
        spec_path.to_str().unwrap(),
        "--out-root",
        harness.path().to_str().unwrap(),
        "--json",
    ]);
    cli.assert_success();

    let output: serde_json::Value =
        serde_json::from_str(&cli.stdout).expect("generate --json should be valid JSON");
    assert_eq!(output["success"].as_bool(), Some(true));

    let outputs = output["result"]["outputs"]
        .as_array()
        .expect("result.outputs array");

    let preview = outputs
        .iter()
        .find(|o| o["kind"] == "preview" && o["format"] == "png")
        .expect("expected a preview png output");

    let preview_rel = preview["path"]
        .as_str()
        .expect("preview output should include path");
    assert!(
        harness.path().join(preview_rel).exists(),
        "preview file missing: {}",
        preview_rel
    );

    let report_rel = output["result"]["report_path"]
        .as_str()
        .expect("result.report_path");
    let report_json = fs::read_to_string(harness.path().join(report_rel)).expect("read report");
    let report: Report = serde_json::from_str(&report_json).expect("parse report");

    assert!(
        report
            .outputs
            .iter()
            .any(|o| o.kind == OutputKind::Preview && o.format == OutputFormat::Png),
        "report.outputs should include preview png"
    );
}

