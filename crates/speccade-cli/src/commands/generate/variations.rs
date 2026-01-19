//! Batch SFX variation generation logic.

use colored::Colorize;
use speccade_spec::{OutputFormat, Spec};
use std::path::Path;

use super::quality::QualityConstraints;
use crate::analysis::audio::analyze_wav;
use crate::commands::json_output::{VariationEntry, VariationsManifest};
use crate::dispatch::dispatch_generate;

/// Result of batch variation generation.
pub struct VariationBatchResult {
    /// The manifest containing all variation results
    pub manifest: VariationsManifest,
    /// Whether any variations failed
    pub any_failed: bool,
}

/// Generates batch variations for an audio spec (human-readable output).
pub fn generate_variations_human(
    spec: &Spec,
    out_root: &str,
    spec_path: &Path,
    num_variations: u32,
    preview_duration: Option<f64>,
    constraints: Option<&QualityConstraints>,
) -> VariationBatchResult {
    let manifest_constraints = constraints.map(|c| c.to_manifest_constraints());
    let mut manifest =
        VariationsManifest::new(spec.asset_id.clone(), spec.seed, manifest_constraints);

    let out_root_path = Path::new(out_root);

    for i in 0..num_variations {
        let var_seed = spec.seed.wrapping_add(i);
        let var_filename = format!("{}_var_{}.wav", spec.asset_id.replace('-', "_"), i);
        let var_path = out_root_path.join(&var_filename);

        // Create a modified spec with the new seed
        let mut var_spec = spec.clone();
        var_spec.seed = var_seed;

        let gen_result = dispatch_generate(&var_spec, out_root, spec_path, preview_duration);

        match gen_result {
            Ok(outputs) => {
                // Find WAV output and analyze it
                let wav_output = outputs.iter().find(|o| o.format == OutputFormat::Wav);

                if let Some(output) = wav_output {
                    // Read and analyze the generated WAV
                    let output_path = out_root_path.join(&output.path);
                    match std::fs::read(&output_path) {
                        Ok(wav_data) => {
                            match analyze_wav(&wav_data) {
                                Ok(metrics) => {
                                    // Check quality constraints
                                    let check_result =
                                        constraints.map(|c| c.check(&metrics)).unwrap_or(Ok(()));

                                    match check_result {
                                        Ok(()) => {
                                            // Rename to variation naming scheme
                                            if let Err(e) = std::fs::rename(&output_path, &var_path)
                                            {
                                                println!(
                                                    "  {} var_{}: rename failed: {}",
                                                    "!".yellow(),
                                                    i,
                                                    e
                                                );
                                                manifest.add_variation(VariationEntry {
                                                    index: i,
                                                    seed: var_seed,
                                                    path: None,
                                                    passed: false,
                                                    reason: Some(format!("rename failed: {}", e)),
                                                    hash: None,
                                                    peak_db: Some(metrics.quality.peak_db),
                                                    dc_offset: Some(metrics.quality.dc_offset),
                                                });
                                            } else {
                                                // Compute hash
                                                let hash = blake3::hash(&wav_data).to_hex();
                                                println!(
                                                    "  {} var_{} (seed={}, peak={:.1}dB)",
                                                    "PASS".green(),
                                                    i,
                                                    var_seed,
                                                    metrics.quality.peak_db
                                                );
                                                manifest.add_variation(VariationEntry {
                                                    index: i,
                                                    seed: var_seed,
                                                    path: Some(var_filename.clone()),
                                                    passed: true,
                                                    reason: None,
                                                    hash: Some(hash.to_string()),
                                                    peak_db: Some(metrics.quality.peak_db),
                                                    dc_offset: Some(metrics.quality.dc_offset),
                                                });
                                            }
                                        }
                                        Err(reason) => {
                                            // Remove the failed output
                                            let _ = std::fs::remove_file(&output_path);
                                            println!("  {} var_{}: {}", "FAIL".red(), i, reason);
                                            manifest.add_variation(VariationEntry {
                                                index: i,
                                                seed: var_seed,
                                                path: None,
                                                passed: false,
                                                reason: Some(reason),
                                                hash: None,
                                                peak_db: Some(metrics.quality.peak_db),
                                                dc_offset: Some(metrics.quality.dc_offset),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!(
                                        "  {} var_{}: analysis failed: {}",
                                        "!".yellow(),
                                        i,
                                        e
                                    );
                                    manifest.add_variation(VariationEntry {
                                        index: i,
                                        seed: var_seed,
                                        path: None,
                                        passed: false,
                                        reason: Some(format!("analysis failed: {}", e)),
                                        hash: None,
                                        peak_db: None,
                                        dc_offset: None,
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            println!("  {} var_{}: read failed: {}", "!".yellow(), i, e);
                            manifest.add_variation(VariationEntry {
                                index: i,
                                seed: var_seed,
                                path: None,
                                passed: false,
                                reason: Some(format!("read failed: {}", e)),
                                hash: None,
                                peak_db: None,
                                dc_offset: None,
                            });
                        }
                    }
                } else {
                    println!("  {} var_{}: no WAV output found", "!".yellow(), i);
                    manifest.add_variation(VariationEntry {
                        index: i,
                        seed: var_seed,
                        path: None,
                        passed: false,
                        reason: Some("no WAV output found".to_string()),
                        hash: None,
                        peak_db: None,
                        dc_offset: None,
                    });
                }
            }
            Err(e) => {
                println!("  {} var_{}: {}", "FAIL".red(), i, e);
                manifest.add_variation(VariationEntry {
                    index: i,
                    seed: var_seed,
                    path: None,
                    passed: false,
                    reason: Some(format!("generation failed: {}", e)),
                    hash: None,
                    peak_db: None,
                    dc_offset: None,
                });
            }
        }
    }

    let any_failed = manifest.failed > 0;
    VariationBatchResult {
        manifest,
        any_failed,
    }
}

/// Generates batch variations for an audio spec (JSON mode - silent).
pub fn generate_variations_json(
    spec: &Spec,
    out_root: &str,
    spec_path: &Path,
    num_variations: u32,
    preview_duration: Option<f64>,
    constraints: Option<&QualityConstraints>,
) -> VariationBatchResult {
    let manifest_constraints = constraints.map(|c| c.to_manifest_constraints());
    let mut manifest =
        VariationsManifest::new(spec.asset_id.clone(), spec.seed, manifest_constraints);

    let out_root_path = Path::new(out_root);

    for i in 0..num_variations {
        let var_seed = spec.seed.wrapping_add(i);
        let var_filename = format!("{}_var_{}.wav", spec.asset_id.replace('-', "_"), i);
        let var_path = out_root_path.join(&var_filename);

        let mut var_spec = spec.clone();
        var_spec.seed = var_seed;

        let gen_result = dispatch_generate(&var_spec, out_root, spec_path, preview_duration);

        match gen_result {
            Ok(var_outputs) => {
                let wav_output = var_outputs.iter().find(|o| o.format == OutputFormat::Wav);

                if let Some(output) = wav_output {
                    let output_path = out_root_path.join(&output.path);
                    match std::fs::read(&output_path) {
                        Ok(wav_data) => match analyze_wav(&wav_data) {
                            Ok(metrics) => {
                                let check_result =
                                    constraints.map(|c| c.check(&metrics)).unwrap_or(Ok(()));

                                match check_result {
                                    Ok(()) => {
                                        if std::fs::rename(&output_path, &var_path).is_ok() {
                                            let hash = blake3::hash(&wav_data).to_hex();
                                            manifest.add_variation(VariationEntry {
                                                index: i,
                                                seed: var_seed,
                                                path: Some(var_filename.clone()),
                                                passed: true,
                                                reason: None,
                                                hash: Some(hash.to_string()),
                                                peak_db: Some(metrics.quality.peak_db),
                                                dc_offset: Some(metrics.quality.dc_offset),
                                            });
                                        } else {
                                            manifest.add_variation(VariationEntry {
                                                index: i,
                                                seed: var_seed,
                                                path: None,
                                                passed: false,
                                                reason: Some("rename failed".to_string()),
                                                hash: None,
                                                peak_db: Some(metrics.quality.peak_db),
                                                dc_offset: Some(metrics.quality.dc_offset),
                                            });
                                        }
                                    }
                                    Err(reason) => {
                                        let _ = std::fs::remove_file(&output_path);
                                        manifest.add_variation(VariationEntry {
                                            index: i,
                                            seed: var_seed,
                                            path: None,
                                            passed: false,
                                            reason: Some(reason),
                                            hash: None,
                                            peak_db: Some(metrics.quality.peak_db),
                                            dc_offset: Some(metrics.quality.dc_offset),
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                manifest.add_variation(VariationEntry {
                                    index: i,
                                    seed: var_seed,
                                    path: None,
                                    passed: false,
                                    reason: Some(format!("analysis failed: {}", e)),
                                    hash: None,
                                    peak_db: None,
                                    dc_offset: None,
                                });
                            }
                        },
                        Err(e) => {
                            manifest.add_variation(VariationEntry {
                                index: i,
                                seed: var_seed,
                                path: None,
                                passed: false,
                                reason: Some(format!("read failed: {}", e)),
                                hash: None,
                                peak_db: None,
                                dc_offset: None,
                            });
                        }
                    }
                } else {
                    manifest.add_variation(VariationEntry {
                        index: i,
                        seed: var_seed,
                        path: None,
                        passed: false,
                        reason: Some("no WAV output found".to_string()),
                        hash: None,
                        peak_db: None,
                        dc_offset: None,
                    });
                }
            }
            Err(e) => {
                manifest.add_variation(VariationEntry {
                    index: i,
                    seed: var_seed,
                    path: None,
                    passed: false,
                    reason: Some(format!("generation failed: {}", e)),
                    hash: None,
                    peak_db: None,
                    dc_offset: None,
                });
            }
        }
    }

    let any_failed = manifest.failed > 0;
    VariationBatchResult {
        manifest,
        any_failed,
    }
}

/// Writes a variations manifest to disk.
pub fn write_manifest(manifest: &VariationsManifest, out_root: &Path) -> std::io::Result<()> {
    let manifest_path = out_root.join("variations.json");
    let manifest_json = serde_json::to_string_pretty(manifest)
        .expect("VariationsManifest serialization should not fail");
    std::fs::write(&manifest_path, &manifest_json)
}
