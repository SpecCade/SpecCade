//! Inspect command implementation
//!
//! Generates intermediate build artifacts for debugging and inspection.
//! Supports texture.procedural_v1 (per-node PNGs) and music.tracker_song_compose_v1
//! (expanded params JSON).

mod compose;
mod texture;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_spec::{canonical_spec_hash, validate_for_generate};
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use super::json_output::{
    compile_warnings_to_json, error_codes, input_error_to_json, validation_error_to_json,
    validation_warning_to_json, InspectOutput, InspectResult, JsonError, JsonWarning,
};
use crate::input::{load_spec, LoadResult};

use compose::inspect_compose;
use texture::inspect_texture_procedural;

/// Run the inspect command
///
/// # Arguments
/// * `spec_path` - Path to the spec file (JSON or Starlark)
/// * `out_dir` - Output directory for intermediate artifacts
/// * `json_output` - Whether to output machine-readable JSON diagnostics
///
/// # Returns
/// Exit code: 0 success, 1 error
pub fn run(spec_path: &str, out_dir: &str, json_output: bool) -> Result<ExitCode> {
    if json_output {
        run_json(spec_path, out_dir)
    } else {
        run_human(spec_path, out_dir)
    }
}

/// Run inspect with human-readable (colored) output
fn run_human(spec_path: &str, out_dir: &str) -> Result<ExitCode> {
    let start = Instant::now();

    println!("{} {}", "Inspecting spec:".cyan().bold(), spec_path);
    println!("{} {}", "Output directory:".cyan().bold(), out_dir);

    // Load spec file
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

    // Validate spec
    let validation_result = validate_for_generate(&spec);
    if !validation_result.is_ok() {
        println!("\n{}", "Validation Errors:".red().bold());
        for error in &validation_result.errors {
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
        println!("\n{} Spec validation failed", "FAILED".red().bold());
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

    // Get recipe
    let recipe = spec
        .recipe
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Spec has no recipe defined"))?;

    println!("{} {}", "Recipe kind:".dimmed(), recipe.kind);

    // Create output directory
    fs::create_dir_all(out_dir)
        .with_context(|| format!("Failed to create output directory: {}", out_dir))?;

    let intermediates_dir = Path::new(out_dir).join("intermediates");
    fs::create_dir_all(&intermediates_dir)
        .with_context(|| "Failed to create intermediates directory")?;

    // Dispatch based on recipe kind
    let (intermediates, expanded_params_path, final_outputs) = match recipe.kind.as_str() {
        "texture.procedural_v1" => {
            let (ints, finals) = inspect_texture_procedural(&spec, &intermediates_dir, out_dir)?;
            (ints, None, finals)
        }
        "music.tracker_song_compose_v1" => {
            let (ints, expanded, finals) = inspect_compose(&spec, &intermediates_dir, out_dir)?;
            (ints, Some(expanded), finals)
        }
        _ => {
            println!(
                "\n{} inspect is not supported for recipe kind: {}",
                "SKIPPED".yellow().bold(),
                recipe.kind
            );
            println!(
                "{}",
                "Supported: texture.procedural_v1, music.tracker_song_compose_v1".dimmed()
            );
            return Ok(ExitCode::SUCCESS);
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    // Print results
    println!("\n{}", "Intermediate artifacts:".green().bold());
    for int_file in &intermediates {
        println!("  {} {}", "+".green(), int_file.path);
    }

    if let Some(ref expanded_path) = expanded_params_path {
        println!("\n{} {}", "Expanded params:".green().bold(), expanded_path);
    }

    if !final_outputs.is_empty() {
        println!("\n{}", "Final outputs:".green().bold());
        for out_file in &final_outputs {
            println!("  {} {}", "+".green(), out_file.path);
        }
    }

    println!(
        "\n{} Generated {} intermediate(s) in {}ms",
        "SUCCESS".green().bold(),
        intermediates.len(),
        duration_ms
    );
    println!("{} {}", "Spec hash:".dimmed(), &spec_hash[..16]);

    Ok(ExitCode::SUCCESS)
}

/// Run inspect with machine-readable JSON output
fn run_json(spec_path: &str, out_dir: &str) -> Result<ExitCode> {
    let start = Instant::now();

    // Load spec file
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
            let output = InspectOutput::failure(vec![error], vec![], None, None);
            println!("{}", serde_json::to_string_pretty(&output)?);
            return Ok(ExitCode::from(1));
        }
    };

    // Compute spec hash
    let spec_hash = canonical_spec_hash(&spec).unwrap_or_else(|_| "unknown".to_string());

    // Validate spec
    let validation_result = validate_for_generate(&spec);

    // Collect warnings
    let mut all_warnings: Vec<JsonWarning> = compile_warnings_to_json(&load_warnings);
    all_warnings.extend(
        validation_result
            .warnings
            .iter()
            .map(validation_warning_to_json),
    );

    if !validation_result.is_ok() {
        let errors: Vec<JsonError> = validation_result
            .errors
            .iter()
            .map(validation_error_to_json)
            .collect();
        let output =
            InspectOutput::failure(errors, all_warnings, Some(spec_hash), Some(source_hash));
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(ExitCode::from(1));
    }

    // Get recipe
    let recipe = match spec.recipe.as_ref() {
        Some(r) => r,
        None => {
            let error = JsonError::new(error_codes::INVALID_SPEC, "Spec has no recipe defined");
            let output = InspectOutput::failure(
                vec![error],
                all_warnings,
                Some(spec_hash),
                Some(source_hash),
            );
            println!("{}", serde_json::to_string_pretty(&output)?);
            return Ok(ExitCode::from(1));
        }
    };

    // Create output directory
    if let Err(e) = fs::create_dir_all(out_dir) {
        let error = JsonError::new(
            error_codes::GENERATION_ERROR,
            format!("Failed to create output directory: {}", e),
        );
        let output = InspectOutput::failure(
            vec![error],
            all_warnings,
            Some(spec_hash),
            Some(source_hash),
        );
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(ExitCode::from(1));
    }

    let intermediates_dir = Path::new(out_dir).join("intermediates");
    if let Err(e) = fs::create_dir_all(&intermediates_dir) {
        let error = JsonError::new(
            error_codes::GENERATION_ERROR,
            format!("Failed to create intermediates directory: {}", e),
        );
        let output = InspectOutput::failure(
            vec![error],
            all_warnings,
            Some(spec_hash),
            Some(source_hash),
        );
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(ExitCode::from(1));
    }

    // Dispatch based on recipe kind
    let inspect_result = match recipe.kind.as_str() {
        "texture.procedural_v1" => {
            match inspect_texture_procedural(&spec, &intermediates_dir, out_dir) {
                Ok((intermediates, final_outputs)) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    InspectResult {
                        asset_id: spec.asset_id.clone(),
                        asset_type: spec.asset_type.to_string(),
                        source_kind: source_kind.as_str().to_string(),
                        recipe_kind: recipe.kind.clone(),
                        out_dir: out_dir.to_string(),
                        intermediates,
                        final_outputs,
                        expanded_params_path: None,
                        duration_ms,
                    }
                }
                Err(e) => {
                    let error = JsonError::new(error_codes::GENERATION_ERROR, e.to_string());
                    let output = InspectOutput::failure(
                        vec![error],
                        all_warnings,
                        Some(spec_hash),
                        Some(source_hash),
                    );
                    println!("{}", serde_json::to_string_pretty(&output)?);
                    return Ok(ExitCode::from(1));
                }
            }
        }
        "music.tracker_song_compose_v1" => {
            match inspect_compose(&spec, &intermediates_dir, out_dir) {
                Ok((intermediates, expanded_path, final_outputs)) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    InspectResult {
                        asset_id: spec.asset_id.clone(),
                        asset_type: spec.asset_type.to_string(),
                        source_kind: source_kind.as_str().to_string(),
                        recipe_kind: recipe.kind.clone(),
                        out_dir: out_dir.to_string(),
                        intermediates,
                        final_outputs,
                        expanded_params_path: Some(expanded_path),
                        duration_ms,
                    }
                }
                Err(e) => {
                    let error = JsonError::new(error_codes::GENERATION_ERROR, e.to_string());
                    let output = InspectOutput::failure(
                        vec![error],
                        all_warnings,
                        Some(spec_hash),
                        Some(source_hash),
                    );
                    println!("{}", serde_json::to_string_pretty(&output)?);
                    return Ok(ExitCode::from(1));
                }
            }
        }
        _ => {
            // Unsupported recipe - return success with empty intermediates
            let duration_ms = start.elapsed().as_millis() as u64;
            InspectResult {
                asset_id: spec.asset_id.clone(),
                asset_type: spec.asset_type.to_string(),
                source_kind: source_kind.as_str().to_string(),
                recipe_kind: recipe.kind.clone(),
                out_dir: out_dir.to_string(),
                intermediates: vec![],
                final_outputs: vec![],
                expanded_params_path: None,
                duration_ms,
            }
        }
    };

    let output = InspectOutput::success(inspect_result, spec_hash, source_hash, all_warnings);
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(ExitCode::SUCCESS)
}
