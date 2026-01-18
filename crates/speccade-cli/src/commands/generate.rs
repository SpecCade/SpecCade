//! Generate command implementation
//!
//! Generates assets from a spec file using the appropriate backend.

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_spec::{
    canonical_recipe_hash, canonical_spec_hash, derive_variant_spec_seed,
    validate_for_generate_with_budget, BackendError, BudgetProfile, ReportBuilder, ReportError,
};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use super::json_output::{
    compile_warnings_to_json, input_error_to_json, validation_error_to_json,
    validation_warning_to_json, GenerateOutput, GenerateResult, GeneratedFile, JsonError,
    JsonWarning, VariantResult,
};
use super::reporting;
use crate::cache::{CacheKey, CacheManager};
use crate::dispatch::{dispatch_generate, dispatch_generate_profiled};
use crate::input::{load_spec, LoadResult};

/// Run the generate command
///
/// # Arguments
/// * `spec_path` - Path to the spec file (JSON or Starlark)
/// * `out_root` - Output root directory (default: current directory)
/// * `expand_variants` - Whether to expand `spec.variants[]` during generation
/// * `budget_name` - Optional budget profile name (default, strict, zx-8bit)
/// * `json_output` - Whether to output machine-readable JSON diagnostics
/// * `preview_duration` - Optional preview duration in seconds (truncates audio generation)
/// * `no_cache` - Whether to bypass cache (default: false, cache enabled)
/// * `profile` - Whether to include per-stage timing in the report
///
/// # Returns
/// Exit code: 0 success, 1 spec error, 2 generation error
pub fn run(
    spec_path: &str,
    out_root: Option<&str>,
    expand_variants: bool,
    budget_name: Option<&str>,
    json_output: bool,
    preview_duration: Option<f64>,
    no_cache: bool,
    profile: bool,
) -> Result<ExitCode> {
    if json_output {
        run_json(
            spec_path,
            out_root,
            expand_variants,
            budget_name,
            preview_duration,
            no_cache,
            profile,
        )
    } else {
        run_human(
            spec_path,
            out_root,
            expand_variants,
            budget_name,
            preview_duration,
            no_cache,
            profile,
        )
    }
}

/// Run generate with human-readable (colored) output
fn run_human(
    spec_path: &str,
    out_root: Option<&str>,
    expand_variants: bool,
    budget_name: Option<&str>,
    preview_duration: Option<f64>,
    no_cache: bool,
    profile: bool,
) -> Result<ExitCode> {
    let start = Instant::now();
    let out_root = out_root.unwrap_or(".");

    // Parse budget profile
    let budget = match budget_name {
        Some(name) => BudgetProfile::by_name(name).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown budget profile: {} (expected default, strict, zx-8bit, or nethercore)",
                name
            )
        })?,
        None => BudgetProfile::default(),
    };

    println!("{} {}", "Generating from:".cyan().bold(), spec_path);
    println!("{} {}", "Output root:".cyan().bold(), out_root);
    if budget_name.is_some() {
        println!("{} {}", "Budget:".dimmed(), budget.name);
    }
    if expand_variants {
        println!("{} {}", "Expand variants:".cyan().bold(), "enabled".green());
    }
    if let Some(duration) = preview_duration {
        println!("{} {:.2}s", "Preview mode:".yellow().bold(), duration);
    }
    if profile {
        println!("{} enabled", "Profile:".cyan().bold());
    }

    // Load spec file (JSON or Starlark)
    let LoadResult {
        spec,
        source_kind,
        source_hash,
        warnings: load_warnings,
    } = load_spec(Path::new(spec_path))
        .with_context(|| format!("Failed to load spec file: {}", spec_path))?;

    // Print any load warnings
    for warning in &load_warnings {
        let location = warning
            .location
            .as_ref()
            .map(|l| format!(" at {}", l))
            .unwrap_or_default();
        println!(
            "  {} [load]{}: {}",
            "!".yellow(),
            location.dimmed(),
            warning.message
        );
    }

    println!(
        "{} {} ({})",
        "Source:".dimmed(),
        source_kind.as_str(),
        &source_hash[..16]
    );

    // Compute spec hash
    let spec_hash = canonical_spec_hash(&spec).unwrap_or_else(|_| "unknown".to_string());
    let recipe_hash = spec
        .recipe
        .as_ref()
        .and_then(|r| canonical_recipe_hash(r).ok());

    // Validate for generation (requires recipe) with budget
    let validation_result = validate_for_generate_with_budget(&spec, &budget);

    let backend_version = format!("speccade-cli v{}", env!("CARGO_PKG_VERSION"));
    let git_commit = option_env!("SPECCADE_GIT_SHA").map(|s| s.to_string());
    let git_dirty = matches!(option_env!("SPECCADE_GIT_DIRTY"), Some("1"));

    // Helper to add git and source provenance to report builders
    let with_provenance = |builder: ReportBuilder| -> ReportBuilder {
        let mut builder = builder.source_provenance(source_kind.as_str(), &source_hash);

        // Add stdlib version for Starlark sources
        #[cfg(feature = "starlark")]
        if source_kind == crate::input::SourceKind::Starlark {
            builder = builder.stdlib_version(crate::compiler::STDLIB_VERSION);
        }

        if let Some(commit) = git_commit.as_ref() {
            builder = builder.git_metadata(commit.clone(), git_dirty);
        }
        builder
    };

    if !validation_result.is_ok() {
        let duration_ms = start.elapsed().as_millis() as u64;

        // Build error report
        let mut report_builder = with_provenance(ReportBuilder::new(spec_hash, backend_version))
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

    // Initialize cache manager (if caching is enabled)
    let cache_mgr = if no_cache {
        None
    } else {
        match CacheManager::new() {
            Ok(mgr) => Some(mgr),
            Err(e) => {
                println!("  {} Failed to initialize cache: {}", "!".yellow(), e);
                None
            }
        }
    };

    // Compute cache key
    let cache_key = if let Some(recipe) = spec.recipe.as_ref() {
        CacheKey::new(
            recipe,
            spec.seed,
            backend_version.clone(),
            preview_duration.is_some(),
        )
        .ok()
    } else {
        None
    };

    // Check cache
    let base_report_path = reporting::report_path(spec_path, &spec.asset_id);
    let base_gen_start = Instant::now();
    let mut cache_hit = false;

    // Dispatch with optional profiling
    let base_result = if profile {
        // Profiling mode: use profiled dispatch, no cache (profiling implies fresh run)
        println!("\n{}", "Dispatching to backend (profiled)...".dimmed());
        dispatch_generate_profiled(
            &spec,
            out_root,
            Path::new(spec_path),
            preview_duration,
            true,
        )
    } else if let (Some(mgr), Some(key)) = (cache_mgr.as_ref(), cache_key.as_ref()) {
        if mgr.has_entry(key) {
            println!("\n{}", "Cache hit, restoring outputs...".green());
            cache_hit = true;
            match mgr.get(key, Path::new(out_root)) {
                Ok(Some(outputs)) => Ok(crate::dispatch::DispatchResult::new(outputs)),
                Ok(None) => {
                    println!("  {} Cache entry missing, regenerating...", "!".yellow());
                    cache_hit = false;
                    dispatch_generate(&spec, out_root, Path::new(spec_path), preview_duration)
                        .map(crate::dispatch::DispatchResult::new)
                }
                Err(e) => {
                    println!(
                        "  {} Cache retrieval failed: {}, regenerating...",
                        "!".yellow(),
                        e
                    );
                    cache_hit = false;
                    dispatch_generate(&spec, out_root, Path::new(spec_path), preview_duration)
                        .map(crate::dispatch::DispatchResult::new)
                }
            }
        } else {
            println!("\n{}", "Dispatching to backend...".dimmed());
            dispatch_generate(&spec, out_root, Path::new(spec_path), preview_duration)
                .map(crate::dispatch::DispatchResult::new)
        }
    } else {
        println!("\n{}", "Dispatching to backend...".dimmed());
        dispatch_generate(&spec, out_root, Path::new(spec_path), preview_duration)
            .map(crate::dispatch::DispatchResult::new)
    };

    let base_duration_ms = base_gen_start.elapsed().as_millis() as u64;

    let mut any_generation_failed = false;

    match base_result {
        Ok(dispatch_result) => {
            let outputs = dispatch_result.outputs;
            let stages = dispatch_result.stages;
            let output_count = outputs.len();

            // Store in cache (if generation happened and caching is enabled, and not profiling)
            if !cache_hit && !profile {
                if let (Some(mgr), Some(key)) = (cache_mgr.as_ref(), cache_key.as_ref()) {
                    if let Err(e) = mgr.put(key, &outputs, Path::new(out_root)) {
                        println!("  {} Failed to cache outputs: {}", "!".yellow(), e);
                    }
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

            let report = report_builder.ok(true).build();
            reporting::write_report(&report, &base_report_path)?;

            let status = if cache_hit {
                format!("{} (cache)", "SUCCESS".green().bold())
            } else {
                format!("{}", "SUCCESS".green().bold())
            };

            println!(
                "\n{} Generated {} output(s) in {}ms",
                status, output_count, base_duration_ms
            );
            println!("{} {}", "Report written to:".dimmed(), base_report_path);
        }
        Err(e) => {
            // Build error report
            let mut report_builder = with_provenance(ReportBuilder::new(
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
                let variant_result = dispatch_generate(
                    &variant_spec,
                    &variant_out_root,
                    Path::new(spec_path),
                    preview_duration,
                );
                let variant_duration_ms = variant_gen_start.elapsed().as_millis() as u64;

                match variant_result {
                    Ok(outputs) => {
                        let output_count = outputs.len();
                        let mut report_builder = with_provenance(ReportBuilder::new(
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

                        let mut report_builder = with_provenance(ReportBuilder::new(
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

/// Run generate with machine-readable JSON output
fn run_json(
    spec_path: &str,
    out_root: Option<&str>,
    expand_variants: bool,
    budget_name: Option<&str>,
    preview_duration: Option<f64>,
    no_cache: bool,
    profile: bool,
) -> Result<ExitCode> {
    let start = Instant::now();
    let out_root_str = out_root.unwrap_or(".");

    // Parse budget profile
    let budget = match budget_name {
        Some(name) => match BudgetProfile::by_name(name) {
            Some(b) => b,
            None => {
                let error = JsonError::new(
                    super::json_output::error_codes::UNKNOWN_BUDGET,
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

    let (spec, source_kind, source_hash, load_warnings) = match load_result {
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
    let cache_key = if let Some(recipe) = spec.recipe.as_ref() {
        CacheKey::new(
            recipe,
            spec.seed,
            backend_version.clone(),
            preview_duration.is_some(),
        )
        .ok()
    } else {
        None
    };

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
            let mut variant_results = Vec::new();
            let mut any_variant_failed = false;

            if expand_variants {
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

                        let variant_spec_hash = canonical_spec_hash(&variant_spec)
                            .unwrap_or_else(|_| "unknown".to_string());

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

                                let error = JsonError::new(
                                    super::json_output::error_codes::GENERATION_ERROR,
                                    e.message(),
                                );

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

            if any_variant_failed {
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
            let error = JsonError::new(
                super::json_output::error_codes::GENERATION_ERROR,
                e.message(),
            );
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
            None,
            false,
            None,
            false,
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
            false,
            false,
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
        let code = run(
            spec_path.to_str().unwrap(),
            Some(out_root),
            true,
            None,
            false,
            None,
            false,
            false,
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
        )
        .unwrap();
        assert_eq!(code, ExitCode::from(1));
    }
}
