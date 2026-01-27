//! JSON output mode for the generate command.

use anyhow::Result;
use speccade_spec::{
    canonical_recipe_hash, canonical_spec_hash, derive_variant_spec_seed,
    validate_for_generate_with_budget, BackendError, BudgetProfile, OutputFormat, ReportBuilder,
    ReportError,
};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use super::quality::QualityConstraints;
use super::variations::{generate_variations_json, write_manifest};
use crate::cache::{CacheKey, CacheManager};
use crate::commands::json_output::{
    compile_warnings_to_json, error_codes, input_error_to_json, validation_error_to_json,
    validation_warning_to_json, GenerateOutput, GenerateResult, GeneratedFile, JsonError,
    JsonWarning, VariantResult,
};
use crate::commands::reporting;
use crate::dispatch::{dispatch_generate, dispatch_generate_profiled};
use crate::input::{load_spec, LoadResult};

/// Run generate with machine-readable JSON output.
#[allow(clippy::too_many_arguments)]
pub fn run_json(
    spec_path: &str,
    out_root: Option<&str>,
    expand_variants: bool,
    budget_name: Option<&str>,
    preview_duration: Option<f64>,
    no_cache: bool,
    profile: bool,
    variations: Option<u32>,
    constraints: Option<QualityConstraints>,
    save_blend: bool,
) -> Result<ExitCode> {
    let start = Instant::now();
    let out_root_str = out_root.unwrap_or(".");

    // Parse budget profile
    let budget = match budget_name {
        Some(name) => match BudgetProfile::by_name(name) {
            Some(b) => b,
            None => {
                let error = JsonError::new(
                    error_codes::UNKNOWN_BUDGET,
                    format!(
                        "Unknown budget profile: {} (expected default, strict, zx-8bit, or nethercore)",
                        name
                    ),
                );
                let output = GenerateOutput::failure(vec![error], vec![], None, None);
                let json = serde_json::to_string_pretty(&output)
                    .expect("GenerateOutput serialization should not fail");
                println!("{}", json);
                return Ok(ExitCode::from(1));
            }
        },
        None => BudgetProfile::default(),
    };

    // Load spec file (JSON or Starlark)
    let load_result = load_spec(Path::new(spec_path));

    let (mut spec, source_kind, source_hash, load_warnings) = match load_result {
        Ok(LoadResult {
            spec,
            source_kind,
            source_hash,
            warnings,
        }) => (spec, source_kind, source_hash, warnings),
        Err(e) => {
            let error = input_error_to_json(&e, Some(spec_path));
            let output = GenerateOutput::failure(vec![error], vec![], None, None);
            let json = serde_json::to_string_pretty(&output)
                .expect("GenerateOutput serialization should not fail");
            println!("{}", json);
            return Ok(ExitCode::from(1));
        }
    };

    // Inject save_blend into recipe params if --save-blend flag is set
    if save_blend {
        if let Some(ref mut recipe) = spec.recipe {
            if let Some(params) = recipe.params.as_object_mut() {
                if let Some(export) = params.get_mut("export") {
                    if let Some(export_obj) = export.as_object_mut() {
                        export_obj.insert(
                            "save_blend".to_string(),
                            serde_json::Value::Bool(true),
                        );
                    }
                } else {
                    params.insert(
                        "export".to_string(),
                        serde_json::json!({"save_blend": true}),
                    );
                }
                params.insert(
                    "save_blend".to_string(),
                    serde_json::Value::Bool(true),
                );
            }
        }
    }

    // Compute spec hash
    let spec_hash = canonical_spec_hash(&spec).unwrap_or_else(|_| "unknown".to_string());
    let recipe_hash = spec
        .recipe
        .as_ref()
        .and_then(|r| canonical_recipe_hash(r).ok());

    // Validate for generation with budget
    let validation_result = validate_for_generate_with_budget(&spec, &budget);

    let backend_version = format!("speccade-cli v{}", env!("CARGO_PKG_VERSION"));
    let git_commit = option_env!("SPECCADE_GIT_SHA").map(|s| s.to_string());
    let git_dirty = matches!(option_env!("SPECCADE_GIT_DIRTY"), Some("1"));

    // Helper to add git and source provenance to report builders
    let with_provenance = |builder: ReportBuilder| -> ReportBuilder {
        let mut builder = builder.source_provenance(source_kind.as_str(), &source_hash);

        #[cfg(feature = "starlark")]
        if source_kind == crate::input::SourceKind::Starlark {
            builder = builder.stdlib_version(crate::compiler::STDLIB_VERSION);
        }

        if let Some(commit) = git_commit.as_ref() {
            builder = builder.git_metadata(commit.clone(), git_dirty);
        }
        builder
    };

    // Collect warnings from load
    let mut all_warnings: Vec<JsonWarning> = compile_warnings_to_json(&load_warnings);
    all_warnings.extend(
        validation_result
            .warnings
            .iter()
            .map(validation_warning_to_json),
    );

    if !validation_result.is_ok() {
        let duration_ms = start.elapsed().as_millis() as u64;

        // Build error report (still write for consistency)
        let mut report_builder =
            with_provenance(ReportBuilder::new(spec_hash.clone(), backend_version))
                .spec_metadata(&spec)
                .duration_ms(duration_ms);
        if let Some(hash) = recipe_hash {
            report_builder = report_builder.recipe_hash(hash);
        }

        report_builder = reporting::apply_validation_messages(report_builder, &validation_result);
        let report = report_builder.ok(false).build();

        let report_path = reporting::report_path(spec_path, &spec.asset_id);
        reporting::write_report(&report, &report_path)?;

        // Build JSON output with validation errors
        let errors: Vec<JsonError> = validation_result
            .errors
            .iter()
            .map(validation_error_to_json)
            .collect();

        let output =
            GenerateOutput::failure(errors, all_warnings, Some(spec_hash), Some(source_hash));

        let json = serde_json::to_string_pretty(&output)
            .expect("GenerateOutput serialization should not fail");
        println!("{}", json);

        return Ok(ExitCode::from(1));
    }

    // Initialize cache manager (if caching is enabled)
    let cache_mgr = if !no_cache {
        CacheManager::new().ok()
    } else {
        None
    };

    // Compute cache key
    let cache_key = CacheKey::new(&spec, backend_version.clone(), preview_duration.is_some()).ok();

    // Check cache
    let base_report_path = reporting::report_path(spec_path, &spec.asset_id);
    let base_gen_start = Instant::now();
    let mut cache_hit = false;

    // Dispatch with optional profiling
    let base_result = if profile {
        // Profiling mode: use profiled dispatch
        dispatch_generate_profiled(
            &spec,
            out_root_str,
            Path::new(spec_path),
            preview_duration,
            true,
        )
    } else if let (Some(mgr), Some(key)) = (cache_mgr.as_ref(), cache_key.as_ref()) {
        if let Ok(Some(outputs)) = mgr.get(key, Path::new(out_root_str)) {
            cache_hit = true;
            Ok(crate::dispatch::DispatchResult::new(outputs))
        } else {
            dispatch_generate(&spec, out_root_str, Path::new(spec_path), preview_duration)
                .map(crate::dispatch::DispatchResult::new)
        }
    } else {
        dispatch_generate(&spec, out_root_str, Path::new(spec_path), preview_duration)
            .map(crate::dispatch::DispatchResult::new)
    };

    let base_duration_ms = base_gen_start.elapsed().as_millis() as u64;

    match base_result {
        Ok(dispatch_result) => {
            let outputs = dispatch_result.outputs;
            let stages = dispatch_result.stages;

            // Store in cache (if generation happened and caching is enabled, and not profiling)
            if !cache_hit && !profile {
                if let (Some(mgr), Some(key)) = (cache_mgr.as_ref(), cache_key.as_ref()) {
                    let _ = mgr.put(key, &outputs, Path::new(out_root_str));
                }
            }

            // Build success report
            let mut report_builder = with_provenance(ReportBuilder::new(
                spec_hash.clone(),
                backend_version.clone(),
            ))
            .spec_metadata(&spec)
            .duration_ms(base_duration_ms);
            if let Some(hash) = recipe_hash.clone() {
                report_builder = report_builder.recipe_hash(hash);
            }

            // Add stage timings if profiling was enabled
            if let Some(stage_timings) = stages {
                report_builder = report_builder.stages(stage_timings);
            }

            report_builder =
                reporting::apply_validation_messages(report_builder, &validation_result);

            for output in &outputs {
                report_builder = report_builder.output(output.clone());
            }

            // Run lint on generated outputs (no text printing in JSON mode)
            if let Some(lint_data) =
                reporting::run_lint_on_outputs(&outputs, &spec, out_root_str, false)
            {
                report_builder = report_builder.lint(lint_data);
            }

            let report = report_builder.ok(true).build();
            reporting::write_report(&report, &base_report_path)?;

            // Convert outputs to GeneratedFile
            let generated_files: Vec<GeneratedFile> = outputs
                .iter()
                .map(|o| GeneratedFile {
                    kind: o.kind.to_string(),
                    format: o.format.to_string(),
                    path: o.path.to_string_lossy().to_string(),
                    hash: o.hash.clone(),
                    preview: o.preview,
                })
                .collect();

            // Handle variant expansion
            let (variant_results, any_variant_failed) = generate_variants_json(
                &spec,
                out_root_str,
                spec_path,
                expand_variants,
                preview_duration,
                &spec_hash,
                recipe_hash.clone(),
                &backend_version,
                &validation_result,
                &with_provenance,
            )?;

            // Batch variation generation (JSON mode)
            let mut any_variation_failed = false;
            if let Some(num_variations) = variations {
                let is_audio = spec.outputs.iter().any(|o| o.format == OutputFormat::Wav);

                if is_audio && num_variations > 0 {
                    let result = generate_variations_json(
                        &spec,
                        out_root_str,
                        Path::new(spec_path),
                        num_variations,
                        preview_duration,
                        constraints.as_ref(),
                    );

                    // Write manifest file
                    let _ = write_manifest(&result.manifest, Path::new(out_root_str));

                    if result.any_failed {
                        any_variation_failed = true;
                    }
                }
            }

            let total_duration_ms = start.elapsed().as_millis() as u64;

            let result = GenerateResult {
                asset_id: spec.asset_id.clone(),
                asset_type: spec.asset_type.to_string(),
                source_kind: source_kind.as_str().to_string(),
                out_root: out_root_str.to_string(),
                budget: budget_name.map(|s| s.to_string()),
                recipe_hash,
                cache_hit,
                outputs: generated_files,
                report_path: base_report_path,
                duration_ms: total_duration_ms,
                variants: variant_results,
            };

            let output = GenerateOutput::success(result, spec_hash, source_hash, all_warnings);

            let json = serde_json::to_string_pretty(&output)
                .expect("GenerateOutput serialization should not fail");
            println!("{}", json);

            if any_variant_failed || any_variation_failed {
                Ok(ExitCode::from(2))
            } else {
                Ok(ExitCode::SUCCESS)
            }
        }
        Err(e) => {
            // Build error report
            let mut report_builder =
                with_provenance(ReportBuilder::new(spec_hash.clone(), backend_version))
                    .spec_metadata(&spec)
                    .duration_ms(base_duration_ms);
            if let Some(hash) = recipe_hash {
                report_builder = report_builder.recipe_hash(hash);
            }

            report_builder = report_builder.error(ReportError::new(e.code(), e.message()));
            report_builder =
                reporting::apply_validation_messages(report_builder, &validation_result);

            let report = report_builder.ok(false).build();
            reporting::write_report(&report, &base_report_path)?;

            // Build JSON error output
            let error = JsonError::new(error_codes::GENERATION_ERROR, e.message());
            let output = GenerateOutput::failure(
                vec![error],
                all_warnings,
                Some(spec_hash),
                Some(source_hash),
            );

            let json = serde_json::to_string_pretty(&output)
                .expect("GenerateOutput serialization should not fail");
            println!("{}", json);

            Ok(ExitCode::from(2))
        }
    }
}

/// Generate variants in JSON mode.
#[allow(clippy::too_many_arguments)]
fn generate_variants_json<F>(
    spec: &speccade_spec::Spec,
    out_root_str: &str,
    spec_path: &str,
    expand_variants: bool,
    preview_duration: Option<f64>,
    spec_hash: &str,
    recipe_hash: Option<String>,
    backend_version: &str,
    validation_result: &speccade_spec::ValidationResult,
    with_provenance: &F,
) -> Result<(Vec<VariantResult>, bool)>
where
    F: Fn(ReportBuilder) -> ReportBuilder,
{
    let mut variant_results = Vec::new();
    let mut any_variant_failed = false;

    if !expand_variants {
        return Ok((variant_results, any_variant_failed));
    }

    if let Some(variants) = spec.variants.as_ref() {
        for variant in variants {
            let variant_id = variant.variant_id.as_str();
            let variant_out_root = Path::new(out_root_str)
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
            let variant_result = dispatch_generate(
                &variant_spec,
                &variant_out_root,
                Path::new(spec_path),
                preview_duration,
            );
            let variant_duration_ms = variant_gen_start.elapsed().as_millis() as u64;

            match variant_result {
                Ok(variant_outputs) => {
                    let mut report_builder = with_provenance(ReportBuilder::new(
                        variant_spec_hash.clone(),
                        backend_version.to_string(),
                    ))
                    .spec_metadata(&variant_spec)
                    .variant(spec_hash.to_string(), variant_id.to_string())
                    .duration_ms(variant_duration_ms);
                    if let Some(hash) = recipe_hash.clone() {
                        report_builder = report_builder.recipe_hash(hash);
                    }

                    report_builder =
                        reporting::apply_validation_messages(report_builder, validation_result);

                    for output in &variant_outputs {
                        report_builder = report_builder.output(output.clone());
                    }

                    let report = report_builder.ok(true).build();
                    reporting::write_report(&report, &variant_report_path)?;

                    let variant_files: Vec<GeneratedFile> = variant_outputs
                        .iter()
                        .map(|o| GeneratedFile {
                            kind: o.kind.to_string(),
                            format: o.format.to_string(),
                            path: o.path.to_string_lossy().to_string(),
                            hash: o.hash.clone(),
                            preview: o.preview,
                        })
                        .collect();

                    variant_results.push(VariantResult {
                        variant_id: variant_id.to_string(),
                        success: true,
                        spec_hash: variant_spec_hash,
                        cache_hit: false,
                        outputs: variant_files,
                        report_path: variant_report_path,
                        duration_ms: variant_duration_ms,
                        error: None,
                    });
                }
                Err(e) => {
                    any_variant_failed = true;

                    let mut report_builder = with_provenance(ReportBuilder::new(
                        variant_spec_hash.clone(),
                        backend_version.to_string(),
                    ))
                    .spec_metadata(&variant_spec)
                    .variant(spec_hash.to_string(), variant_id.to_string())
                    .duration_ms(variant_duration_ms);
                    if let Some(hash) = recipe_hash.clone() {
                        report_builder = report_builder.recipe_hash(hash);
                    }

                    report_builder = report_builder.error(ReportError::new(e.code(), e.message()));
                    report_builder =
                        reporting::apply_validation_messages(report_builder, validation_result);

                    let report = report_builder.ok(false).build();
                    reporting::write_report(&report, &variant_report_path)?;

                    let error = JsonError::new(error_codes::GENERATION_ERROR, e.message());

                    variant_results.push(VariantResult {
                        variant_id: variant_id.to_string(),
                        success: false,
                        spec_hash: variant_spec_hash,
                        cache_hit: false,
                        outputs: vec![],
                        report_path: variant_report_path,
                        duration_ms: variant_duration_ms,
                        error: Some(error),
                    });
                }
            }
        }
    }

    Ok((variant_results, any_variant_failed))
}
