//! Tests for the generate command.

use super::run;
use crate::commands::json_output::VariationsManifest;
use crate::commands::reporting;
use speccade_spec::recipe::audio::{AudioLayer, AudioV1Params, Envelope, Synthesis, Waveform};
use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe, Spec, VariantSpec};
use std::process::ExitCode;

fn write_spec(dir: &tempfile::TempDir, filename: &str, spec: &Spec) -> std::path::PathBuf {
    let path = dir.path().join(filename);
    std::fs::write(&path, spec.to_json_pretty().unwrap()).unwrap();
    path
}

#[test]
fn generate_rejects_missing_recipe_and_writes_report() {
    let tmp = tempfile::tempdir().unwrap();

    let spec = Spec::builder("test-asset-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .description("test asset")
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);

    let code = run(
        spec_path.to_str().unwrap(),
        Some(tmp.path().to_str().unwrap()),
        false,
        None,
        false,
        None,
        false,
        false,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(code, ExitCode::from(1));

    let report_path = reporting::report_path(spec_path.to_str().unwrap(), &spec.asset_id);
    let json = std::fs::read_to_string(&report_path).unwrap();
    let report: speccade_spec::Report = serde_json::from_str(&json).unwrap();
    assert!(!report.ok);
    assert!(report.errors.iter().any(|e| e.code == "E010"));
}

#[test]
fn generate_reports_validation_errors_for_invalid_params() {
    let tmp = tempfile::tempdir().unwrap();

    // Invalid sample_rate is caught during validation (E017 with budget)
    let invalid_params = serde_json::json!({
        "duration_seconds": 0.1,
        "sample_rate": 12345,
        "layers": []
    });

    let spec = Spec::builder("test-asset-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .description("test asset")
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .recipe(Recipe::new("audio_v1", invalid_params))
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);

    let code = run(
        spec_path.to_str().unwrap(),
        Some(tmp.path().to_str().unwrap()),
        false,
        None,
        false,
        None,
        false,
        false,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(code, ExitCode::from(1));

    let report_path = reporting::report_path(spec_path.to_str().unwrap(), &spec.asset_id);
    let json = std::fs::read_to_string(&report_path).unwrap();
    let report: speccade_spec::Report = serde_json::from_str(&json).unwrap();
    assert!(!report.ok);
    // Invalid sample_rate now triggers BudgetExceeded (E017)
    assert!(report.errors.iter().any(|e| e.code == "E017"));
}

#[test]
fn generate_audio_v1_writes_output_and_report() {
    let tmp = tempfile::tempdir().unwrap();

    let params = AudioV1Params {
        base_note: None,
        duration_seconds: 0.1,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        loop_config: None,
        generate_loop_points: false,
        master_filter: None,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let spec = Spec::builder("test-asset-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .description("test asset")
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::to_value(&params).unwrap(),
        ))
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);

    let out_root = tmp.path().to_str().unwrap();
    let code = run(
        spec_path.to_str().unwrap(),
        Some(out_root),
        false,
        None,
        false,
        None,
        true, // no_cache to ensure fresh generation
        false,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(code, ExitCode::SUCCESS);

    let output_path = tmp.path().join("test.wav");
    let out_bytes = std::fs::read(&output_path).unwrap();
    assert!(!out_bytes.is_empty());

    let report_path = reporting::report_path(spec_path.to_str().unwrap(), &spec.asset_id);
    let json = std::fs::read_to_string(&report_path).unwrap();
    let report: speccade_spec::Report = serde_json::from_str(&json).unwrap();
    assert!(report.ok);
    // Note: may have 1 or 2 outputs depending on whether report is also counted
    assert!(!report.outputs.is_empty());
}

#[test]
fn generate_expands_variants_into_separate_output_roots_and_reports() {
    let tmp = tempfile::tempdir().unwrap();

    let params = AudioV1Params {
        base_note: None,
        duration_seconds: 0.05,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        loop_config: None,
        generate_loop_points: false,
        master_filter: None,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let spec = Spec::builder("test-variants-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::to_value(&params).unwrap(),
        ))
        .variants(vec![
            VariantSpec::new("soft", 0),
            VariantSpec::new("hard", 1),
        ])
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);

    let out_root = tmp.path().to_str().unwrap();
    let code = run(
        spec_path.to_str().unwrap(),
        Some(out_root),
        true,
        None,
        false,
        None,
        false,
        false,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(code, ExitCode::SUCCESS);

    // Base output.
    assert!(tmp.path().join("test.wav").exists());

    // Variant outputs.
    assert!(tmp
        .path()
        .join("variants")
        .join("soft")
        .join("test.wav")
        .exists());
    assert!(tmp
        .path()
        .join("variants")
        .join("hard")
        .join("test.wav")
        .exists());

    // Variant reports.
    let base_report_path = reporting::report_path(spec_path.to_str().unwrap(), &spec.asset_id);
    assert!(std::path::Path::new(&base_report_path).exists());

    let soft_report_path =
        reporting::report_path_variant(spec_path.to_str().unwrap(), &spec.asset_id, "soft");
    assert!(std::path::Path::new(&soft_report_path).exists());

    let base_report_json = std::fs::read_to_string(&base_report_path).unwrap();
    let base_report: speccade_spec::Report = serde_json::from_str(&base_report_json).unwrap();
    assert!(base_report.ok);
    assert_eq!(base_report.asset_id.as_deref(), Some("test-variants-01"));
    assert_eq!(base_report.asset_type, Some(AssetType::Audio));
    assert_eq!(base_report.variant_id, None);
    assert_eq!(base_report.base_spec_hash, None);
    assert!(base_report.recipe_hash.is_some());

    let soft_report_json = std::fs::read_to_string(&soft_report_path).unwrap();
    let soft_report: speccade_spec::Report = serde_json::from_str(&soft_report_json).unwrap();
    assert!(soft_report.ok);
    assert_eq!(soft_report.variant_id.as_deref(), Some("soft"));
    assert_eq!(
        soft_report.base_spec_hash.as_deref(),
        Some(base_report.spec_hash.as_str())
    );
    assert_ne!(soft_report.spec_hash, base_report.spec_hash);
    assert_eq!(soft_report.recipe_hash, base_report.recipe_hash);
}

#[test]
fn generate_json_output_success() {
    let tmp = tempfile::tempdir().unwrap();

    let params = AudioV1Params {
        base_note: None,
        duration_seconds: 0.05,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        loop_config: None,
        generate_loop_points: false,
        master_filter: None,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let spec = Spec::builder("gen-json-test-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .description("test asset")
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::to_value(&params).unwrap(),
        ))
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);

    // Run with json=true - should succeed
    let code = run(
        spec_path.to_str().unwrap(),
        Some(tmp.path().to_str().unwrap()),
        false,
        None,
        true,
        None,
        false,
        false,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn generate_json_output_validation_failure() {
    let tmp = tempfile::tempdir().unwrap();

    let spec = Spec::builder("gen-json-test-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .description("test asset")
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);

    // Missing recipe - should return exit code 1
    let code = run(
        spec_path.to_str().unwrap(),
        Some(tmp.path().to_str().unwrap()),
        false,
        None,
        true,
        None,
        false,
        false,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(code, ExitCode::from(1));
}

#[test]
fn generate_json_output_file_not_found() {
    // Run with json=true on nonexistent file - should return exit code 1
    let code = run(
        "/nonexistent/spec.json",
        None,
        false,
        None,
        true,
        None,
        false,
        false,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(code, ExitCode::from(1));
}

#[test]
fn generate_variations_creates_manifest_and_outputs() {
    let tmp = tempfile::tempdir().unwrap();

    let params = AudioV1Params {
        base_note: None,
        duration_seconds: 0.05,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.5,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        loop_config: None,
        generate_loop_points: false,
        master_filter: None,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let spec = Spec::builder("test-variations-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::to_value(&params).unwrap(),
        ))
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);

    let out_root = tmp.path().to_str().unwrap();
    let code = run(
        spec_path.to_str().unwrap(),
        Some(out_root),
        false,
        None,
        false,
        None,
        false,
        false,
        Some(3), // Generate 3 variations
        None,
        None,
    )
    .unwrap();
    assert_eq!(code, ExitCode::SUCCESS);

    // Check manifest exists
    let manifest_path = tmp.path().join("variations.json");
    assert!(manifest_path.exists());

    // Parse and verify manifest
    let manifest_json = std::fs::read_to_string(&manifest_path).unwrap();
    let manifest: VariationsManifest = serde_json::from_str(&manifest_json).unwrap();
    assert_eq!(manifest.spec_id, "test-variations-01");
    assert_eq!(manifest.total, 3);
    assert_eq!(manifest.passed, 3);
    assert_eq!(manifest.failed, 0);
    assert_eq!(manifest.base_seed, 42);

    // Check variation files exist
    assert!(tmp.path().join("test_variations_01_var_0.wav").exists());
    assert!(tmp.path().join("test_variations_01_var_1.wav").exists());
    assert!(tmp.path().join("test_variations_01_var_2.wav").exists());

    // Verify each variation entry
    for (i, entry) in manifest.variations.iter().enumerate() {
        assert_eq!(entry.index, i as u32);
        assert_eq!(entry.seed, 42 + i as u32);
        assert!(entry.passed);
        assert!(entry.path.is_some());
        assert!(entry.hash.is_some());
        assert!(entry.peak_db.is_some());
        assert!(entry.dc_offset.is_some());
    }
}

#[test]
fn generate_variations_with_peak_constraint_records_constraint() {
    let tmp = tempfile::tempdir().unwrap();

    // Create a sine wave with moderate volume
    let params = AudioV1Params {
        base_note: None,
        duration_seconds: 0.05,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.1, // Very low volume to ensure all pass
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        loop_config: None,
        generate_loop_points: false,
        master_filter: None,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let spec = Spec::builder("test-variations-peak", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::to_value(&params).unwrap(),
        ))
        .build();

    let spec_path = write_spec(&tmp, "spec.json", &spec);

    let out_root = tmp.path().to_str().unwrap();
    let code = run(
        spec_path.to_str().unwrap(),
        Some(out_root),
        false,
        None,
        false,
        None,
        true, // no_cache to force fresh generation
        false,
        Some(2),
        Some(-3.0), // Generous threshold for low-volume audio
        None,
    )
    .unwrap();
    assert_eq!(code, ExitCode::SUCCESS);

    // Check manifest
    let manifest_path = tmp.path().join("variations.json");
    assert!(manifest_path.exists());

    let manifest_json = std::fs::read_to_string(&manifest_path).unwrap();
    let manifest: VariationsManifest = serde_json::from_str(&manifest_json).unwrap();

    // Verify constraints are recorded
    assert!(manifest.constraints.is_some());
    let constraints = manifest.constraints.unwrap();
    assert_eq!(constraints.max_peak_db, Some(-3.0));

    // All variations should pass since volume is very low (~-20dB)
    assert_eq!(manifest.total, 2);
    assert_eq!(manifest.passed, 2);
}
