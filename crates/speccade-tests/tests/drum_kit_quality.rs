//! Drum Kit Quality Gate Tests
//!
//! This test validates the drum kit examples (kick, snare, hi-hat) by:
//! 1. Generating each drum sample from its Starlark spec
//! 2. Analyzing the output WAV file
//! 3. Validating quality metrics:
//!    - peak_db < 0 (no clipping)
//!    - dc_offset < 0.01 (minimal DC offset)
//!    - rms_db in range -24 to -6 dBFS (appropriate loudness)
//!
//! ## Running
//!
//! ```bash
//! cargo test -p speccade-tests --test drum_kit_quality
//! ```

use std::path::PathBuf;

use speccade_cli::analysis::audio::{analyze_wav, AudioMetrics};
use speccade_cli::input::load_spec;

/// Path to the drum kit examples
fn drum_examples_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs")
        .join("examples")
        .join("music")
        .join("drums")
}

/// Quality thresholds for drum sounds
struct DrumQualityThresholds {
    /// Maximum peak in dB (must be < 0 for no clipping)
    max_peak_db: f64,
    /// Maximum absolute DC offset
    max_dc_offset: f64,
    /// Minimum RMS in dB
    min_rms_db: f64,
    /// Maximum RMS in dB
    max_rms_db: f64,
}

impl Default for DrumQualityThresholds {
    fn default() -> Self {
        Self {
            max_peak_db: 0.0,     // No clipping
            max_dc_offset: 0.01,  // Very low DC offset
            min_rms_db: -24.0,    // Not too quiet
            max_rms_db: -6.0,     // Not too loud
        }
    }
}

/// Result of quality validation
#[derive(Debug)]
struct QualityCheckResult {
    passed: bool,
    failures: Vec<String>,
    peak_db: f64,
    rms_db: f64,
    dc_offset: f64,
    clipping_detected: bool,
}

/// Check audio metrics against quality thresholds
fn check_drum_quality(metrics: &AudioMetrics, thresholds: &DrumQualityThresholds) -> QualityCheckResult {
    let mut failures = Vec::new();

    // Check peak (no clipping)
    if metrics.quality.peak_db >= thresholds.max_peak_db {
        failures.push(format!(
            "peak_db: {:.2} dB (must be < {:.2} dB for no clipping)",
            metrics.quality.peak_db, thresholds.max_peak_db
        ));
    }

    // Check clipping flag
    if metrics.quality.clipping_detected {
        failures.push("clipping_detected: true (expected false)".to_string());
    }

    // Check DC offset
    if metrics.quality.dc_offset.abs() >= thresholds.max_dc_offset {
        failures.push(format!(
            "dc_offset: {:.6} (must be < {:.4})",
            metrics.quality.dc_offset, thresholds.max_dc_offset
        ));
    }

    // Check RMS range
    if metrics.quality.rms_db < thresholds.min_rms_db {
        failures.push(format!(
            "rms_db: {:.2} dB (must be >= {:.2} dB, audio may be too quiet)",
            metrics.quality.rms_db, thresholds.min_rms_db
        ));
    }

    if metrics.quality.rms_db > thresholds.max_rms_db {
        failures.push(format!(
            "rms_db: {:.2} dB (must be <= {:.2} dB, audio may be too loud)",
            metrics.quality.rms_db, thresholds.max_rms_db
        ));
    }

    QualityCheckResult {
        passed: failures.is_empty(),
        failures,
        peak_db: metrics.quality.peak_db,
        rms_db: metrics.quality.rms_db,
        dc_offset: metrics.quality.dc_offset,
        clipping_detected: metrics.quality.clipping_detected,
    }
}

/// Generate and analyze a drum sample, returning the analysis result
fn generate_and_analyze_drum(spec_name: &str) -> Result<AudioMetrics, String> {
    let spec_path = drum_examples_path().join(format!("{}.star", spec_name));

    // Load and parse the Starlark spec
    let load_result = load_spec(&spec_path)
        .map_err(|e| format!("Failed to load {}.star: {}", spec_name, e))?;

    // Generate the audio
    let gen_result = speccade_backend_audio::generate(&load_result.spec)
        .map_err(|e| format!("Failed to generate {}: {}", spec_name, e))?;

    // Analyze the generated WAV
    let metrics = analyze_wav(&gen_result.wav.wav_data)
        .map_err(|e| format!("Failed to analyze {}: {}", spec_name, e))?;

    Ok(metrics)
}

/// Test that validates a single drum and prints detailed results
fn validate_drum(name: &str, thresholds: &DrumQualityThresholds) -> bool {
    println!("\n=== Testing {} ===", name);

    match generate_and_analyze_drum(name) {
        Ok(metrics) => {
            let result = check_drum_quality(&metrics, thresholds);

            println!("  Format: {}Hz, {} channels, {} bits",
                metrics.format.sample_rate,
                metrics.format.channels,
                metrics.format.bits_per_sample
            );
            println!("  Duration: {:.1}ms", metrics.format.duration_ms);
            println!("  Quality:");
            println!("    peak_db: {:.2} dB", result.peak_db);
            println!("    rms_db: {:.2} dB", result.rms_db);
            println!("    dc_offset: {:.6}", result.dc_offset);
            println!("    clipping_detected: {}", result.clipping_detected);

            if result.passed {
                println!("  Result: PASSED");
                true
            } else {
                println!("  Result: FAILED");
                for failure in &result.failures {
                    println!("    - {}", failure);
                }
                false
            }
        }
        Err(e) => {
            println!("  Error: {}", e);
            println!("  Result: FAILED");
            false
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_kick_drum_quality() {
    let thresholds = DrumQualityThresholds::default();
    assert!(
        validate_drum("kick", &thresholds),
        "Kick drum failed quality checks"
    );
}

#[test]
fn test_snare_drum_quality() {
    let thresholds = DrumQualityThresholds::default();
    assert!(
        validate_drum("snare", &thresholds),
        "Snare drum failed quality checks"
    );
}

#[test]
fn test_hihat_quality() {
    let thresholds = DrumQualityThresholds::default();
    assert!(
        validate_drum("hihat", &thresholds),
        "Hi-hat failed quality checks"
    );
}

/// Combined test that validates all drums and provides a summary
#[test]
fn test_all_drums_pass_quality_gates() {
    let thresholds = DrumQualityThresholds::default();
    let drums = ["kick", "snare", "hihat"];

    println!("\n========================================");
    println!("  Drum Kit Quality Gate Validation");
    println!("========================================");
    println!("\nThresholds:");
    println!("  max_peak_db: < {:.1} dB (no clipping)", thresholds.max_peak_db);
    println!("  max_dc_offset: < {:.4}", thresholds.max_dc_offset);
    println!("  rms_db range: {:.1} to {:.1} dB", thresholds.min_rms_db, thresholds.max_rms_db);

    let mut passed = 0;
    let mut failed = 0;
    let mut failed_names = Vec::new();

    for drum in &drums {
        if validate_drum(drum, &thresholds) {
            passed += 1;
        } else {
            failed += 1;
            failed_names.push(*drum);
        }
    }

    println!("\n========================================");
    println!("  Summary: {} passed, {} failed", passed, failed);
    println!("========================================");

    assert!(
        failed == 0,
        "\nThe following drums failed quality checks: {}",
        failed_names.join(", ")
    );
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    /// Test that the drum examples directory exists and contains the expected files
    #[test]
    fn test_drum_examples_exist() {
        let path = drum_examples_path();
        assert!(path.exists(), "Drum examples directory does not exist: {:?}", path);

        let kick_path = path.join("kick.star");
        let snare_path = path.join("snare.star");
        let hihat_path = path.join("hihat.star");

        assert!(kick_path.exists(), "kick.star does not exist: {:?}", kick_path);
        assert!(snare_path.exists(), "snare.star does not exist: {:?}", snare_path);
        assert!(hihat_path.exists(), "hihat.star does not exist: {:?}", hihat_path);
    }

    /// Test that all drum specs can be loaded without errors
    #[test]
    fn test_drum_specs_load_successfully() {
        let drums = ["kick", "snare", "hihat"];

        for drum in &drums {
            let spec_path = drum_examples_path().join(format!("{}.star", drum));
            let result = load_spec(&spec_path);
            assert!(
                result.is_ok(),
                "Failed to load {}.star: {:?}",
                drum,
                result.err()
            );
        }
    }

    /// Test that all drums have correct asset IDs
    #[test]
    fn test_drum_asset_ids() {
        let drums = [
            ("kick", "drum-kit-kick-01"),
            ("snare", "drum-kit-snare-01"),
            ("hihat", "drum-kit-hihat-01"),
        ];

        for (name, expected_id) in &drums {
            let spec_path = drum_examples_path().join(format!("{}.star", name));
            let result = load_spec(&spec_path).expect(&format!("Failed to load {}.star", name));
            assert_eq!(
                result.spec.asset_id, *expected_id,
                "Asset ID mismatch for {}",
                name
            );
        }
    }

    /// Test that all drums generate audio with reasonable duration
    #[test]
    fn test_drum_durations_reasonable() {
        let drums = ["kick", "snare", "hihat"];

        for drum in &drums {
            let metrics = generate_and_analyze_drum(drum)
                .expect(&format!("Failed to generate {}", drum));

            // Drums should be short samples (less than 1 second)
            assert!(
                metrics.format.duration_ms < 1000.0,
                "{} duration too long: {:.1}ms",
                drum,
                metrics.format.duration_ms
            );

            // But not too short (at least 50ms)
            assert!(
                metrics.format.duration_ms >= 50.0,
                "{} duration too short: {:.1}ms",
                drum,
                metrics.format.duration_ms
            );
        }
    }
}
