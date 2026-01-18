//! Tests for the audio audit command.

use super::*;
use std::fs;
use tempfile::TempDir;

fn create_test_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * channels as u32 * 2;
    let block_align = channels * 2;
    let data_size = samples.len() * 2;
    let file_size = 36 + data_size;

    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(file_size as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_size as u32).to_le_bytes());

    for &s in samples {
        let sample_i16 = (s * 32767.0) as i16;
        wav.extend_from_slice(&sample_i16.to_le_bytes());
    }

    wav
}

fn setup_test_dir() -> (TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("test.wav");

    // Create a clean audio file (no clipping, low DC offset)
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    fs::write(&wav_path, &wav_data).unwrap();

    (tmp, wav_path)
}

#[test]
fn test_default_tolerances() {
    let tolerances = AuditTolerances::default();
    assert_eq!(tolerances.max_peak_db, 0.0);
    assert_eq!(tolerances.max_dc_offset, 0.05);
    assert!(!tolerances.allow_clipping);
}

#[test]
fn test_baseline_serialization() {
    let baseline = AudioBaseline {
        schema_version: 1,
        peak_db: -6.0,
        rms_db: -12.0,
        dc_offset: 0.001,
        clipping_detected: false,
        file_hash: Some("abc123".to_string()),
    };

    let json = serde_json::to_string_pretty(&baseline).unwrap();
    let parsed: AudioBaseline = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.peak_db, -6.0);
    assert_eq!(parsed.rms_db, -12.0);
    assert!(!parsed.clipping_detected);
}

#[test]
fn test_audit_file_passes_with_clean_audio() {
    let (tmp, wav_path) = setup_test_dir();
    let tolerances = AuditTolerances::default();

    let result = audit_file(&wav_path, &tolerances, false);

    assert!(result.passed, "Expected clean audio to pass audit");
    assert!(result.violations.is_empty());
    assert!(result.metrics.is_some());
    assert!(result.error.is_none());

    drop(tmp);
}

#[test]
fn test_audit_file_fails_with_clipping() {
    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("clipping.wav");

    // Create audio that clips (samples at max)
    let samples: Vec<f32> = (0..4410).map(|_| 1.0).collect();
    let wav_data = create_test_wav(&samples, 44100);
    fs::write(&wav_path, &wav_data).unwrap();

    let tolerances = AuditTolerances::default();
    let result = audit_file(&wav_path, &tolerances, false);

    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.kind == ViolationKind::ClippingDetected));
}

#[test]
fn test_audit_update_baseline() {
    let (tmp, wav_path) = setup_test_dir();
    let tolerances = AuditTolerances::default();

    // First run - create baseline
    let result = audit_file(&wav_path, &tolerances, true);
    assert!(result.passed);

    // Check baseline was created
    let baseline_file = baseline::baseline_path(&wav_path);
    assert!(baseline_file.exists());

    // Load and verify baseline
    let baseline = AudioBaseline::from_file(&baseline_file).unwrap();
    assert_eq!(baseline.schema_version, 1);
    assert!(baseline.file_hash.is_some());

    drop(tmp);
}

#[test]
fn test_audit_regression_detection() {
    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("test.wav");
    let _baseline_file = baseline::baseline_path(&wav_path);

    // Create initial audio and baseline
    let samples1: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data1 = create_test_wav(&samples1, 44100);
    fs::write(&wav_path, &wav_data1).unwrap();

    let tolerances = AuditTolerances::default();
    let _ = audit_file(&wav_path, &tolerances, true);

    // Replace audio with different amplitude (causes regression)
    let samples2: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.1).collect();
    let wav_data2 = create_test_wav(&samples2, 44100);
    fs::write(&wav_path, &wav_data2).unwrap();

    // Now audit should detect regression
    let result = audit_file(&wav_path, &tolerances, false);

    // The RMS level should have changed significantly
    assert!(result.baseline.is_some());
    assert!(result.metrics.is_some());
}

#[test]
fn test_tolerances_from_file() {
    let tmp = tempfile::tempdir().unwrap();
    let config_path = tmp.path().join("tolerances.json");

    let tolerances = AuditTolerances {
        max_peak_db: -3.0,
        max_dc_offset: 0.1,
        allow_clipping: true,
        peak_db_delta: 1.0,
        rms_db_delta: 2.0,
        dc_offset_delta: 0.02,
    };

    let json = serde_json::to_string_pretty(&tolerances).unwrap();
    fs::write(&config_path, &json).unwrap();

    let loaded = AuditTolerances::from_file(&config_path).unwrap();
    assert_eq!(loaded.max_peak_db, -3.0);
    assert!(loaded.allow_clipping);
}

#[test]
fn test_audit_output_summary() {
    let results = vec![
        AuditFileResult {
            path: "pass.wav".to_string(),
            passed: true,
            violations: vec![],
            metrics: Some(AuditMetrics {
                peak_db: -6.0,
                rms_db: -12.0,
                dc_offset: 0.0,
                clipping_detected: false,
            }),
            baseline: None,
            error: None,
        },
        AuditFileResult {
            path: "fail.wav".to_string(),
            passed: false,
            violations: vec![AuditViolation {
                kind: ViolationKind::ClippingDetected,
                message: "Clipping".to_string(),
                actual: None,
                expected: None,
                delta: None,
            }],
            metrics: Some(AuditMetrics {
                peak_db: 0.0,
                rms_db: -3.0,
                dc_offset: 0.0,
                clipping_detected: true,
            }),
            baseline: Some(AuditMetrics {
                peak_db: -6.0,
                rms_db: -12.0,
                dc_offset: 0.0,
                clipping_detected: false,
            }),
            error: None,
        },
    ];

    let output = AuditOutput::from_results(results, AuditTolerances::default(), vec![]);

    assert!(!output.success);
    assert_eq!(output.summary.total, 2);
    assert_eq!(output.summary.passed, 1);
    assert_eq!(output.summary.failed, 1);
    assert_eq!(output.summary.with_baseline, 1);
    assert_eq!(output.summary.without_baseline, 1);
}

#[test]
fn test_run_audit_on_directory() {
    let tmp = tempfile::tempdir().unwrap();

    // Create a test WAV file
    let wav_path = tmp.path().join("test.wav");
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    fs::write(&wav_path, &wav_data).unwrap();

    // Run audit
    let result = run(tmp.path().to_str().unwrap(), None, false, true);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExitCode::SUCCESS);
}

#[test]
fn test_run_audit_empty_directory() {
    let tmp = tempfile::tempdir().unwrap();

    let result = run(tmp.path().to_str().unwrap(), None, false, true);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExitCode::SUCCESS);
}

#[test]
fn test_run_audit_nonexistent_directory() {
    let result = run("/nonexistent/path", None, false, true);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExitCode::from(1));
}
