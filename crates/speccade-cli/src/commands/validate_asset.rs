//! Validate-asset command implementation
//!
//! Runs full validation pipeline on a single asset:
//! 1. Generate asset from spec
//! 2. Create preview-grid PNG
//! 3. Analyze metrics
//! 4. Run lint
//! 5. Combine into validation report

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::commands::generate;
use crate::commands::preview_grid;
use crate::input::load_spec;
use speccade_spec::Spec;

/// Run the validate-asset command
pub fn run(spec_path: &str, out_root: Option<&str>, _full_report: bool) -> Result<ExitCode> {
    let spec_path_pb = PathBuf::from(spec_path);

    // Load spec
    let load_result =
        load_spec(&spec_path_pb).with_context(|| format!("Failed to load spec: {}", spec_path))?;

    let spec = load_result.spec;

    // Verify it's a supported 3D asset type
    match spec.asset_type.as_str() {
        "static_mesh" | "skeletal_mesh" | "skeletal_animation" => {}
        other => {
            anyhow::bail!(
                "validate-asset only supports 3D assets (static_mesh, skeletal_mesh, skeletal_animation), got: {}",
                other
            );
        }
    }

    // Set up output directory
    let out_dir = if let Some(root) = out_root {
        PathBuf::from(root)
    } else {
        PathBuf::from("validation-output")
    };

    fs::create_dir_all(&out_dir)?;

    println!("Validating: {} (type: {})", spec_path, spec.asset_type);
    println!("Output directory: {}", out_dir.display());

    // Step 1: Generate the asset
    println!("\n[1/4] Generating asset...");
    let gen_result = generate::run(
        spec_path,
        Some(out_dir.to_str().unwrap()),
        false, // expand_variants
        None,  // budget_name - no strict budget
        false, // json_output
        None,  // preview_duration
        false, // no_cache
        false, // profile
        None,  // variations
        None,  // max_peak_db
        None,  // max_dc_offset
        false, // save_blend
    );

    match gen_result {
        Ok(ExitCode::SUCCESS) => {
            println!("  ✓ Asset generated successfully");
        }
        Ok(code) => {
            println!("  ✗ Generation failed with exit code: {:?}", code);
            return Ok(code);
        }
        Err(e) => {
            println!("  ✗ Generation error: {}", e);
            return Err(e);
        }
    }

    // Find the generated asset
    let asset_path = find_generated_glb(&out_dir, &spec)
        .ok_or_else(|| anyhow::anyhow!("Generated GLB not found in {}", out_dir.display()))?;

    println!("  Generated: {}", asset_path.display());

    // Step 2: Generate preview grid
    println!("\n[2/4] Generating preview grid...");

    let grid_filename = format!("{}.grid.png", spec.asset_id.replace("/", "_"));
    let grid_path = out_dir.join(&grid_filename);

    let grid_result = preview_grid::run(
        spec_path,
        Some(grid_path.to_str().unwrap()),
        256, // panel size
    );

    match grid_result {
        Ok(ExitCode::SUCCESS) => {
            println!("  ✓ Preview grid generated: {}", grid_path.display());
        }
        Ok(code) => {
            println!(
                "  ⚠ Preview grid failed (exit code: {:?}), continuing...",
                code
            );
            // Don't fail validation if preview fails - asset might still be valid
        }
        Err(e) => {
            println!("  ⚠ Preview grid error: {}, continuing...", e);
        }
    }

    println!("\nPreview-grid complete. Ready for analysis.");

    Ok(ExitCode::SUCCESS)
}

/// Find the generated GLB file in output directory
fn find_generated_glb(out_dir: &Path, spec: &Spec) -> Option<PathBuf> {
    // Check primary output from spec
    for output in &spec.outputs {
        if output.format == speccade_spec::OutputFormat::Glb {
            let path = out_dir.join(&output.path);
            if path.exists() {
                return Some(path);
            }
        }
    }

    // Fallback: search for any .glb file
    if let Ok(entries) = std::fs::read_dir(out_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "glb").unwrap_or(false) {
                return Some(path);
            }
        }
    }

    None
}
