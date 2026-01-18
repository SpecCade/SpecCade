//! Preset Library QA Gates
//!
//! This test validates all audio presets in `packs/preset_library_v1/audio/`
//! by generating WAV files and analyzing them for quality issues.
//!
//! ## Quality Thresholds
//!
//! Each generated WAV must pass these checks:
//! - `clipping_detected` must be `false`
//! - `dc_offset` absolute value < 0.05
//! - `peak_amplitude` > 0.1 (not silent) and < 1.0 (no hard clipping)
//! - `rms_level` between 0.01 and 0.5
//!
//! ## Running
//!
//! ```bash
//! cargo test -p speccade-tests -- preset_qa --nocapture
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use speccade_cli::analysis::audio::{analyze_wav, AudioMetrics};
use speccade_spec::Spec;
use walkdir::WalkDir;

/// Quality thresholds for audio presets.
struct QualityThresholds {
    /// Maximum absolute DC offset allowed.
    max_dc_offset: f64,
    /// Minimum peak amplitude (to detect silent output).
    min_peak_amplitude: f64,
    /// Maximum peak amplitude (to detect hard clipping).
    max_peak_amplitude: f64,
    /// Minimum RMS level.
    min_rms_level: f64,
    /// Maximum RMS level.
    max_rms_level: f64,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            max_dc_offset: 0.05,
            min_peak_amplitude: 0.1,
            max_peak_amplitude: 1.0,
            min_rms_level: 0.01,
            max_rms_level: 0.5,
        }
    }
}

/// Result of QA check for a single preset.
#[derive(Debug)]
struct PresetQaResult {
    preset_path: PathBuf,
    asset_id: String,
    passed: bool,
    failures: Vec<String>,
    metrics: Option<QaMetrics>,
}

/// Extracted metrics for reporting.
#[derive(Debug)]
struct QaMetrics {
    peak_amplitude: f64,
    rms_level: f64,
    dc_offset: f64,
    clipping_detected: bool,
}

/// Convert dB to linear amplitude.
fn db_to_linear(db: f64) -> f64 {
    if db <= -100.0 {
        0.0
    } else {
        10_f64.powf(db / 20.0)
    }
}

/// Extract linear metrics from AudioMetrics.
fn extract_qa_metrics(metrics: &AudioMetrics) -> QaMetrics {
    QaMetrics {
        peak_amplitude: db_to_linear(metrics.quality.peak_db),
        rms_level: db_to_linear(metrics.quality.rms_db),
        dc_offset: metrics.quality.dc_offset,
        clipping_detected: metrics.quality.clipping_detected,
    }
}

/// Check a preset against quality thresholds.
fn check_quality(metrics: &QaMetrics, thresholds: &QualityThresholds) -> Vec<String> {
    let mut failures = Vec::new();

    if metrics.clipping_detected {
        failures.push("clipping_detected: true (expected false)".to_string());
    }

    if metrics.dc_offset.abs() >= thresholds.max_dc_offset {
        failures.push(format!(
            "dc_offset: {:.6} (must be < {:.2})",
            metrics.dc_offset, thresholds.max_dc_offset
        ));
    }

    if metrics.peak_amplitude <= thresholds.min_peak_amplitude {
        failures.push(format!(
            "peak_amplitude: {:.6} (must be > {:.2}, audio may be silent)",
            metrics.peak_amplitude, thresholds.min_peak_amplitude
        ));
    }

    if metrics.peak_amplitude >= thresholds.max_peak_amplitude {
        failures.push(format!(
            "peak_amplitude: {:.6} (must be < {:.2}, hard clipping)",
            metrics.peak_amplitude, thresholds.max_peak_amplitude
        ));
    }

    if metrics.rms_level < thresholds.min_rms_level {
        failures.push(format!(
            "rms_level: {:.6} (must be >= {:.2})",
            metrics.rms_level, thresholds.min_rms_level
        ));
    }

    if metrics.rms_level > thresholds.max_rms_level {
        failures.push(format!(
            "rms_level: {:.6} (must be <= {:.2})",
            metrics.rms_level, thresholds.max_rms_level
        ));
    }

    failures
}

/// Validate a single preset file.
fn validate_preset(preset_path: &Path, thresholds: &QualityThresholds) -> PresetQaResult {
    let preset_path_buf = preset_path.to_path_buf();

    // Read and parse the spec
    let content = match fs::read_to_string(preset_path) {
        Ok(c) => c,
        Err(e) => {
            return PresetQaResult {
                preset_path: preset_path_buf,
                asset_id: "unknown".to_string(),
                passed: false,
                failures: vec![format!("Failed to read file: {}", e)],
                metrics: None,
            };
        }
    };

    let spec: Spec = match Spec::from_json(&content) {
        Ok(s) => s,
        Err(e) => {
            return PresetQaResult {
                preset_path: preset_path_buf,
                asset_id: "unknown".to_string(),
                passed: false,
                failures: vec![format!("Failed to parse spec: {}", e)],
                metrics: None,
            };
        }
    };

    let asset_id = spec.asset_id.clone();

    // Generate audio
    let gen_result = match speccade_backend_audio::generate(&spec) {
        Ok(r) => r,
        Err(e) => {
            return PresetQaResult {
                preset_path: preset_path_buf,
                asset_id,
                passed: false,
                failures: vec![format!("Generation failed: {}", e)],
                metrics: None,
            };
        }
    };

    // Analyze the generated WAV
    let analysis = match analyze_wav(&gen_result.wav.wav_data) {
        Ok(m) => m,
        Err(e) => {
            return PresetQaResult {
                preset_path: preset_path_buf,
                asset_id,
                passed: false,
                failures: vec![format!("Analysis failed: {}", e)],
                metrics: None,
            };
        }
    };

    // Extract metrics and check quality
    let qa_metrics = extract_qa_metrics(&analysis);
    let failures = check_quality(&qa_metrics, thresholds);

    PresetQaResult {
        preset_path: preset_path_buf,
        asset_id,
        passed: failures.is_empty(),
        failures,
        metrics: Some(qa_metrics),
    }
}

/// Find all audio preset JSON files in the preset library.
/// Excludes `.report.json` files which are analysis reports, not presets.
fn find_audio_presets() -> Vec<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let preset_dir = manifest_dir
        .join("..")
        .join("..")
        .join("packs")
        .join("preset_library_v1")
        .join("audio");

    let preset_dir = match preset_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    WalkDir::new(&preset_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            let is_json = path.extension().map_or(false, |ext| ext == "json");
            let is_file = e.file_type().is_file();
            // Exclude .report.json files (analysis reports, not presets)
            let is_report = path
                .file_name()
                .map_or(false, |n| n.to_string_lossy().ends_with(".report.json"));
            is_json && is_file && !is_report
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Main QA gate test for all audio presets.
///
/// This test is marked `#[ignore]` because it processes 500+ presets
/// and takes significant time. Run explicitly for CI or pre-release QA:
///
/// ```bash
/// cargo test -p speccade-tests -- preset_qa --ignored --nocapture
/// ```
#[test]
#[ignore]
fn test_all_audio_presets_pass_qa() {
    let thresholds = QualityThresholds::default();
    let presets = find_audio_presets();

    if presets.is_empty() {
        panic!("No audio presets found in packs/preset_library_v1/audio/");
    }

    println!("Found {} audio presets to validate", presets.len());

    // Group results by category for better reporting
    let mut results_by_category: BTreeMap<String, Vec<PresetQaResult>> = BTreeMap::new();
    let mut total_passed = 0;
    let mut total_failed = 0;

    for preset_path in &presets {
        let result = validate_preset(preset_path, &thresholds);

        // Extract category from path (e.g., "bass", "drums/kicks")
        let category = preset_path
            .parent()
            .and_then(|p| {
                // Get relative path from audio directory
                let audio_dir = preset_path
                    .ancestors()
                    .find(|a| a.file_name().map_or(false, |n| n == "audio"))?;
                p.strip_prefix(audio_dir).ok()
            })
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|| "uncategorized".to_string());

        if result.passed {
            total_passed += 1;
        } else {
            total_failed += 1;
        }

        results_by_category
            .entry(category)
            .or_default()
            .push(result);
    }

    // Print summary by category
    println!("\n=== QA Results by Category ===\n");

    for (category, results) in &results_by_category {
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;

        if failed > 0 {
            println!(
                "[{}] {}/{} passed, {} failed:",
                category,
                passed,
                results.len(),
                failed
            );

            for result in results.iter().filter(|r| !r.passed) {
                println!("  FAIL: {}", result.asset_id);
                for failure in &result.failures {
                    println!("    - {}", failure);
                }
                if let Some(metrics) = &result.metrics {
                    println!(
                        "    metrics: peak={:.4}, rms={:.4}, dc={:.6}, clip={}",
                        metrics.peak_amplitude,
                        metrics.rms_level,
                        metrics.dc_offset,
                        metrics.clipping_detected
                    );
                }
            }
        } else {
            println!("[{}] {}/{} passed", category, passed, results.len());
        }
    }

    println!("\n=== Summary ===");
    println!("Total: {} passed, {} failed", total_passed, total_failed);

    // Collect all failures for the assertion message
    let all_failures: Vec<_> = results_by_category
        .values()
        .flatten()
        .filter(|r| !r.passed)
        .collect();

    assert!(
        all_failures.is_empty(),
        "\n{} preset(s) failed QA:\n{}",
        all_failures.len(),
        all_failures
            .iter()
            .map(|r| format!(
                "  {} ({}): {}",
                r.asset_id,
                r.preset_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                r.failures.join("; ")
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Quick sanity test that runs on a small subset of presets.
/// This runs as part of normal `cargo test` without `--ignored`.
#[test]
fn test_preset_qa_smoke() {
    let thresholds = QualityThresholds::default();
    let presets = find_audio_presets();

    if presets.is_empty() {
        println!("No audio presets found, skipping smoke test");
        return;
    }

    // Test first 5 presets as a smoke test
    let sample_size = presets.len().min(5);
    let mut failures = Vec::new();

    for preset_path in presets.iter().take(sample_size) {
        let result = validate_preset(preset_path, &thresholds);
        if !result.passed {
            failures.push(result);
        }
    }

    assert!(
        failures.is_empty(),
        "Smoke test failed for {} of {} presets:\n{}",
        failures.len(),
        sample_size,
        failures
            .iter()
            .map(|r| format!("  {}: {}", r.asset_id, r.failures.join("; ")))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_to_linear_conversions() {
        // 0 dB = 1.0 linear
        assert!((db_to_linear(0.0) - 1.0).abs() < 0.0001);

        // -6 dB ~= 0.5 linear
        assert!((db_to_linear(-6.0206) - 0.5).abs() < 0.001);

        // -20 dB = 0.1 linear
        assert!((db_to_linear(-20.0) - 0.1).abs() < 0.0001);

        // Very low dB returns 0
        assert_eq!(db_to_linear(-100.0), 0.0);
        assert_eq!(db_to_linear(-150.0), 0.0);
    }

    #[test]
    fn test_quality_thresholds() {
        let thresholds = QualityThresholds::default();

        // Good metrics should pass
        let good_metrics = QaMetrics {
            peak_amplitude: 0.5,
            rms_level: 0.2,
            dc_offset: 0.001,
            clipping_detected: false,
        };
        assert!(check_quality(&good_metrics, &thresholds).is_empty());

        // Clipping should fail
        let clipping_metrics = QaMetrics {
            peak_amplitude: 0.5,
            rms_level: 0.2,
            dc_offset: 0.001,
            clipping_detected: true,
        };
        let failures = check_quality(&clipping_metrics, &thresholds);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("clipping_detected"));

        // High DC offset should fail
        let dc_metrics = QaMetrics {
            peak_amplitude: 0.5,
            rms_level: 0.2,
            dc_offset: 0.1,
            clipping_detected: false,
        };
        let failures = check_quality(&dc_metrics, &thresholds);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("dc_offset"));

        // Silent audio should fail
        let silent_metrics = QaMetrics {
            peak_amplitude: 0.05,
            rms_level: 0.005,
            dc_offset: 0.0,
            clipping_detected: false,
        };
        let failures = check_quality(&silent_metrics, &thresholds);
        assert_eq!(failures.len(), 2); // Both peak and rms fail
    }

    #[test]
    fn test_find_audio_presets() {
        let presets = find_audio_presets();
        // Should find presets if the directory exists
        // This test just verifies the function doesn't panic
        println!("Found {} presets", presets.len());
    }
}
