//! Unit tests for the inspect command.

use super::*;
use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe, Spec};

fn write_spec(dir: &tempfile::TempDir, filename: &str, spec: &Spec) -> std::path::PathBuf {
    let path = dir.path().join(filename);
    std::fs::write(&path, spec.to_json_pretty().unwrap()).unwrap();
    path
}

#[test]
fn inspect_texture_generates_intermediates() {
    let tmp = tempfile::tempdir().unwrap();

    let mut output = OutputSpec::primary(OutputFormat::Png, "mask.png");
    output.source = Some("mask".to_string());

    let recipe = Recipe::new(
        "texture.procedural_v1",
        serde_json::json!({
            "resolution": [16, 16],
            "tileable": true,
            "nodes": [
                { "id": "noise", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.1 } },
                { "id": "mask", "type": "threshold", "input": "noise", "threshold": 0.5 }
            ]
        }),
    );

    let spec = Spec::builder("inspect-tex-test-01", AssetType::Texture)
        .license("CC0-1.0")
        .seed(42)
        .output(output)
        .recipe(recipe)
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);
    let out_dir = tmp.path().join("out");

    let code = run(
        spec_path.to_str().unwrap(),
        out_dir.to_str().unwrap(),
        false,
    )
    .unwrap();

    assert_eq!(code, ExitCode::SUCCESS);

    // Check intermediates were created
    assert!(out_dir.join("intermediates/noise.png").exists());
    assert!(out_dir.join("intermediates/mask.png").exists());

    // Check final output was created
    assert!(out_dir.join("mask.png").exists());
}

#[test]
fn inspect_texture_json_output() {
    let tmp = tempfile::tempdir().unwrap();

    let mut output = OutputSpec::primary(OutputFormat::Png, "mask.png");
    output.source = Some("mask".to_string());

    let recipe = Recipe::new(
        "texture.procedural_v1",
        serde_json::json!({
            "resolution": [16, 16],
            "tileable": true,
            "nodes": [
                { "id": "noise", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.1 } },
                { "id": "mask", "type": "threshold", "input": "noise", "threshold": 0.5 }
            ]
        }),
    );

    let spec = Spec::builder("inspect-tex-test-02", AssetType::Texture)
        .license("CC0-1.0")
        .seed(42)
        .output(output)
        .recipe(recipe)
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);
    let out_dir = tmp.path().join("out");

    let code = run(spec_path.to_str().unwrap(), out_dir.to_str().unwrap(), true).unwrap();

    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn inspect_compose_generates_expanded_params() {
    let tmp = tempfile::tempdir().unwrap();

    let recipe = Recipe::new(
        "music.tracker_song_compose_v1",
        serde_json::json!({
            "format": "xm",
            "bpm": 120,
            "speed": 6,
            "channels": 4,
            "instruments": [{
                "name": "lead",
                "base_note": "C4",
                "synthesis": { "type": "sine" }
            }],
            "defs": {},
            "patterns": {
                "intro": {
                    "rows": 16,
                    "program": {
                        "op": "emit",
                        "at": { "op": "range", "start": 0, "step": 4, "count": 4 },
                        "cell": { "channel": 0, "note": "C4", "inst": 0, "vol": 64 }
                    }
                }
            },
            "arrangement": [{ "pattern": "intro", "repeat": 1 }]
        }),
    );

    let spec = Spec::builder("inspect-compose-test-01", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "test.xm"))
        .recipe(recipe)
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);
    let out_dir = tmp.path().join("out");

    let code = run(
        spec_path.to_str().unwrap(),
        out_dir.to_str().unwrap(),
        false,
    )
    .unwrap();

    assert_eq!(code, ExitCode::SUCCESS);

    // Check expanded params was created
    assert!(out_dir.join("intermediates/expanded_params.json").exists());

    // Verify it's valid JSON
    let content =
        std::fs::read_to_string(out_dir.join("intermediates/expanded_params.json")).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).unwrap();
}

#[test]
fn inspect_unsupported_recipe_returns_success() {
    let tmp = tempfile::tempdir().unwrap();

    let recipe = Recipe::new(
        "audio_v1",
        serde_json::json!({
            "duration_seconds": 0.1,
            "sample_rate": 22050,
            "layers": []
        }),
    );

    let spec = Spec::builder("inspect-audio-test-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .recipe(recipe)
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);
    let out_dir = tmp.path().join("out");

    let code = run(
        spec_path.to_str().unwrap(),
        out_dir.to_str().unwrap(),
        false,
    )
    .unwrap();

    // Should succeed (just skip with message)
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn inspect_missing_recipe_fails() {
    let tmp = tempfile::tempdir().unwrap();

    let spec = Spec::builder("inspect-no-recipe-01", AssetType::Texture)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Png, "test.png"))
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);
    let out_dir = tmp.path().join("out");

    let code = run(spec_path.to_str().unwrap(), out_dir.to_str().unwrap(), true).unwrap();

    assert_eq!(code, ExitCode::from(1));
}

#[test]
fn inspect_nonexistent_file_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let out_dir = tmp.path().join("out");

    let code = run("/nonexistent/spec.json", out_dir.to_str().unwrap(), true).unwrap();

    assert_eq!(code, ExitCode::from(1));
}
