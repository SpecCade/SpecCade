//! Generate command implementation
//!
//! Generates assets from a spec file using the appropriate backend.

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_spec::{
    canonical_recipe_hash, canonical_spec_hash, derive_variant_spec_seed, validate_for_generate,
    BackendError, ReportBuilder, ReportError, Spec,
};
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use super::reporting;
use crate::dispatch::dispatch_generate;

/// Run the generate command
///
/// # Arguments
/// * `spec_path` - Path to the spec JSON file
/// * `out_root` - Output root directory (default: current directory)
/// * `expand_variants` - Whether to expand `spec.variants[]` during generation
///
/// # Returns
/// Exit code: 0 success, 1 spec error, 2 generation error
pub fn run(spec_path: &str, out_root: Option<&str>, expand_variants: bool) -> Result<ExitCode> {
    let start = Instant::now();
    let out_root = out_root.unwrap_or(".");

    println!("{} {}", "Generating from:".cyan().bold(), spec_path);
    println!("{} {}", "Output root:".cyan().bold(), out_root);
    if expand_variants {
        println!("{} {}", "Expand variants:".cyan().bold(), "enabled".green());
    }

    // Read spec file
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;

    // Parse spec
    let spec = Spec::from_json(&spec_content)
        .with_context(|| format!("Failed to parse spec file: {}", spec_path))?;

    // Compute spec hash
    let spec_hash = canonical_spec_hash(&spec).unwrap_or_else(|_| "unknown".to_string());
    let recipe_hash = spec
        .recipe
        .as_ref()
        .and_then(|r| canonical_recipe_hash(r).ok());

    // Validate for generation (requires recipe)
    let validation_result = validate_for_generate(&spec);

    let backend_version = format!("speccade-cli v{}", env!("CARGO_PKG_VERSION"));
    let git_commit = option_env!("SPECCADE_GIT_SHA").map(|s| s.to_string());
    let git_dirty = matches!(option_env!("SPECCADE_GIT_DIRTY"), Some("1"));

    let with_git = |builder: ReportBuilder| -> ReportBuilder {
        if let Some(commit) = git_commit.as_ref() {
            builder.git_metadata(commit.clone(), git_dirty)
        } else {
            builder
        }
    };

    if !validation_result.is_ok() {
        let duration_ms = start.elapsed().as_millis() as u64;

        // Build error report
        let mut report_builder = with_git(ReportBuilder::new(spec_hash, backend_version))
            .spec_metadata(&spec)
            .duration_ms(duration_ms);
        if let Some(hash) = recipe_hash {
            report_builder = report_builder.recipe_hash(hash);
        }

        report_builder = reporting::apply_validation_messages(report_builder, &validation_result);

        let report = report_builder.ok(false).build();

        // Write report
        let report_path = reporting::report_path(spec_path, &spec.asset_id);
        reporting::write_report(&report, &report_path)?;

        // Print errors
        print_validation_errors(&validation_result);

        println!(
            "\n{} Spec validation failed with {} error(s)",
            "FAILED".red().bold(),
            validation_result.errors.len()
        );
        println!("{} {}", "Report written to:".dimmed(), report_path);

        return Ok(ExitCode::from(1));
    }

    // Print warnings if any
    if !validation_result.warnings.is_empty() {
        println!("\n{}", "Warnings:".yellow().bold());
        for warning in &validation_result.warnings {
            println!(
                "  {} [{}]: {}",
                "!".yellow(),
                warning.code.to_string().yellow(),
                warning.message
            );
        }
    }

    // Dispatch to backend
    println!("\n{}", "Dispatching to backend...".dimmed());

    let base_report_path = reporting::report_path(spec_path, &spec.asset_id);

    let base_gen_start = Instant::now();
    let base_result = dispatch_generate(&spec, out_root, Path::new(spec_path));
    let base_duration_ms = base_gen_start.elapsed().as_millis() as u64;

    let mut any_generation_failed = false;

    match base_result {
        Ok(outputs) => {
            let output_count = outputs.len();

            // Build success report
            let mut report_builder = with_git(ReportBuilder::new(
                spec_hash.clone(),
                backend_version.clone(),
            ))
            .spec_metadata(&spec)
            .duration_ms(base_duration_ms);
            if let Some(hash) = recipe_hash.clone() {
                report_builder = report_builder.recipe_hash(hash);
            }

            report_builder =
                reporting::apply_validation_messages(report_builder, &validation_result);

            for output in outputs {
                report_builder = report_builder.output(output);
            }

            let report = report_builder.ok(true).build();
            reporting::write_report(&report, &base_report_path)?;

            println!(
                "\n{} Generated {} output(s) in {}ms",
                "SUCCESS".green().bold(),
                output_count,
                base_duration_ms
            );
            println!("{} {}", "Report written to:".dimmed(), base_report_path);
        }
        Err(e) => {
            // Build error report
            let mut report_builder = with_git(ReportBuilder::new(
                spec_hash.clone(),
                backend_version.clone(),
            ))
            .spec_metadata(&spec)
            .duration_ms(base_duration_ms);
            if let Some(hash) = recipe_hash.clone() {
                report_builder = report_builder.recipe_hash(hash);
            }

            // Add generation error using BackendError trait
            report_builder = report_builder.error(ReportError::new(e.code(), e.message()));
            report_builder =
                reporting::apply_validation_messages(report_builder, &validation_result);

            let report = report_builder.ok(false).build();
            reporting::write_report(&report, &base_report_path)?;

            println!("\n{} {}", "GENERATION FAILED".red().bold(), e);
            println!("{} {}", "Report written to:".dimmed(), base_report_path);

            // If the base spec failed, don't attempt variant expansion.
            return Ok(ExitCode::from(2));
        }
    }

    // Optional variant expansion.
    if expand_variants {
        if let Some(variants) = spec.variants.as_ref() {
            if !variants.is_empty() {
                println!(
                    "\n{} {}",
                    "Expanding variants:".cyan().bold(),
                    variants.len()
                );
            }

            for variant in variants {
                let variant_id = variant.variant_id.as_str();
                let variant_out_root = Path::new(out_root)
                    .join("variants")
                    .join(variant_id)
                    .to_string_lossy()
                    .to_string();
                let variant_report_path =
                    reporting::report_path_variant(spec_path, &spec.asset_id, variant_id);

                let mut variant_spec = spec.clone();
                variant_spec.seed =
                    derive_variant_spec_seed(spec.seed, variant.seed_offset, variant_id);

                let variant_spec_hash =
                    canonical_spec_hash(&variant_spec).unwrap_or_else(|_| "unknown".to_string());

                let variant_gen_start = Instant::now();
                let variant_result =
                    dispatch_generate(&variant_spec, &variant_out_root, Path::new(spec_path));
                let variant_duration_ms = variant_gen_start.elapsed().as_millis() as u64;

                match variant_result {
                    Ok(outputs) => {
                        let output_count = outputs.len();
                        let mut report_builder = with_git(ReportBuilder::new(
                            variant_spec_hash,
                            backend_version.clone(),
                        ))
                        .spec_metadata(&variant_spec)
                        .variant(spec_hash.clone(), variant_id.to_string())
                        .duration_ms(variant_duration_ms);
                        if let Some(hash) = recipe_hash.clone() {
                            report_builder = report_builder.recipe_hash(hash);
                        }

                        report_builder = reporting::apply_validation_messages(
                            report_builder,
                            &validation_result,
                        );

                        for output in outputs {
                            report_builder = report_builder.output(output);
                        }

                        let report = report_builder.ok(true).build();
                        reporting::write_report(&report, &variant_report_path)?;

                        println!(
                            "  {} {} ({} output(s), {}ms)",
                            "VARIANT".green(),
                            variant_id,
                            output_count,
                            variant_duration_ms
                        );
                    }
                    Err(e) => {
                        any_generation_failed = true;

                        let mut report_builder = with_git(ReportBuilder::new(
                            variant_spec_hash,
                            backend_version.clone(),
                        ))
                        .spec_metadata(&variant_spec)
                        .variant(spec_hash.clone(), variant_id.to_string())
                        .duration_ms(variant_duration_ms);
                        if let Some(hash) = recipe_hash.clone() {
                            report_builder = report_builder.recipe_hash(hash);
                        }

                        report_builder =
                            report_builder.error(ReportError::new(e.code(), e.message()));
                        report_builder = reporting::apply_validation_messages(
                            report_builder,
                            &validation_result,
                        );

                        let report = report_builder.ok(false).build();
                        reporting::write_report(&report, &variant_report_path)?;

                        println!("  {} {}: {}", "VARIANT FAILED".red(), variant_id, e);
                    }
                }
            }
        }
    }

    if any_generation_failed {
        Ok(ExitCode::from(2))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Print validation errors to the console
fn print_validation_errors(result: &speccade_spec::ValidationResult) {
    if !result.errors.is_empty() {
        println!("\n{}", "Validation Errors:".red().bold());
        for error in &result.errors {
            let path_info = error
                .path
                .as_ref()
                .map(|p| format!(" at {}", p))
                .unwrap_or_default();
            println!(
                "  {} [{}]{}: {}",
                "x".red(),
                error.code.to_string().red(),
                path_info.dimmed(),
                error.message
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::audio::{AudioLayer, AudioV1Params, Envelope, Synthesis, Waveform};
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe, Spec, VariantSpec};

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

        // Invalid sample_rate is caught during validation (E012)
        let invalid_params = serde_json::json!({
            "duration_seconds": 0.1,
            "sample_rate": 12345,
            "layers": []
        });

        let spec = Spec::builder("test-asset-01", AssetType::Audio)
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
        )
        .unwrap();
        assert_eq!(code, ExitCode::from(1));

        let report_path = reporting::report_path(spec_path.to_str().unwrap(), &spec.asset_id);
        let json = std::fs::read_to_string(&report_path).unwrap();
        let report: speccade_spec::Report = serde_json::from_str(&json).unwrap();
        assert!(!report.ok);
        assert!(report.errors.iter().any(|e| e.code == "E012"));
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
            generate_loop_points: false,
            master_filter: None,
            effects: vec![],
            post_fx_lfos: vec![],
        };

        let spec = Spec::builder("test-asset-01", AssetType::Audio)
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
        let code = run(spec_path.to_str().unwrap(), Some(out_root), false).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        let output_path = tmp.path().join("test.wav");
        let out_bytes = std::fs::read(&output_path).unwrap();
        assert!(!out_bytes.is_empty());

        let report_path = reporting::report_path(spec_path.to_str().unwrap(), &spec.asset_id);
        let json = std::fs::read_to_string(&report_path).unwrap();
        let report: speccade_spec::Report = serde_json::from_str(&json).unwrap();
        assert!(report.ok);
        assert_eq!(report.outputs.len(), 1);
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
        let code = run(spec_path.to_str().unwrap(), Some(out_root), true).unwrap();
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
}
