//! Human-readable (colored) output mode for the generate command.

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_spec::{
    canonical_recipe_hash, canonical_spec_hash, derive_variant_spec_seed,
    validate_for_generate_with_budget, BackendError, BudgetProfile, OutputFormat, ReportBuilder,
    ReportError,
};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use super::quality::QualityConstraints;
use super::variations::{generate_variations_human, write_manifest};
use crate::cache::{CacheKey, CacheManager};
use crate::commands::reporting;
use crate::dispatch::{dispatch_generate, dispatch_generate_profiled};
use crate::input::{load_spec, LoadResult};

/// Run generate with human-readable (colored) output.
#[allow(clippy::too_many_arguments)]
pub fn run_human(
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
    if let Some(n) = variations {
        println!("{} {}", "Variations:".cyan().bold(), n);
        if let Some(ref c) = constraints {
            if let Some(max_peak) = c.max_peak_db {
                println!("  {} {:.1} dB", "Max peak:".dimmed(), max_peak);
            }
            if let Some(max_dc) = c.max_dc_offset {
                println!("  {} {:.6}", "Max DC offset:".dimmed(), max_dc);
            }
        }
    }

    // Load spec file (JSON or Starlark)
    let LoadResult {
        mut spec,
        source_kind,
        source_hash,
        warnings: load_warnings,
    } = load_spec(Path::new(spec_path))
        .with_context(|| format!("Failed to load spec file: {}", spec_path))?;

    // Inject save_blend into recipe params if --save-blend flag is set
    if save_blend {
        if let Some(ref mut recipe) = spec.recipe {
            if let Some(params) = recipe.params.as_object_mut() {
                // For mesh pipelines, inject into nested export settings
                if let Some(export) = params.get_mut("export") {
                    if let Some(export_obj) = export.as_object_mut() {
                        export_obj.insert("save_blend".to_string(), serde_json::Value::Bool(true));
                    }
                } else {
                    // No export block yet - create one with save_blend
                    params.insert(
                        "export".to_string(),
                        serde_json::json!({"save_blend": true}),
                    );
                }
                // Also set top-level save_blend for handlers that read it directly
                params.insert("save_blend".to_string(), serde_json::Value::Bool(true));
            }
        }
    }

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
    let cache_key = CacheKey::new(&spec, backend_version.clone(), preview_duration.is_some()).ok();

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

            // Run lint on generated outputs
            if let Some(lint_data) = reporting::run_lint_on_outputs(&outputs, &spec, out_root, true)
            {
                report_builder = report_builder.lint(lint_data);
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

    // Batch variation generation
    if let Some(num_variations) = variations {
        // Variations only make sense for audio specs
        let is_audio = spec.outputs.iter().any(|o| o.format == OutputFormat::Wav);

        if !is_audio {
            println!(
                "\n{} --variations only applies to audio specs (WAV output)",
                "WARNING".yellow().bold()
            );
        } else if num_variations > 0 {
            println!(
                "\n{} {}",
                "Generating variations:".cyan().bold(),
                num_variations
            );

            let result = generate_variations_human(
                &spec,
                out_root,
                Path::new(spec_path),
                num_variations,
                preview_duration,
                constraints.as_ref(),
            );

            // Write manifest
            if let Err(e) = write_manifest(&result.manifest, Path::new(out_root)) {
                println!(
                    "\n{} Failed to write variations manifest: {}",
                    "WARNING".yellow().bold(),
                    e
                );
            } else {
                println!(
                    "\n{} {}/{} variations passed",
                    "Variations:".cyan().bold(),
                    result.manifest.passed,
                    result.manifest.total
                );
                println!(
                    "{} {}",
                    "Manifest written to:".dimmed(),
                    Path::new(out_root).join("variations.json").display()
                );
            }

            if result.any_failed {
                any_generation_failed = true;
            }
        }
    }

    if any_generation_failed {
        Ok(ExitCode::from(2))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Print validation errors to the console.
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
