//! Preview grid command implementation
//!
//! Generates a multi-angle validation grid PNG for 3D assets.

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_backend_blender::{GenerationMode, Orchestrator};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::input::load_spec;

/// Run the preview-grid command
///
/// # Arguments
/// * `spec_path` - Path to the spec file
/// * `out` - Output PNG path (optional)
/// * `panel_size` - Size of each panel in pixels
pub fn run(spec_path: &str, out: Option<&str>, panel_size: u32) -> Result<ExitCode> {
    println!("{} {}", "Preview Grid:".cyan().bold(), spec_path);

    let spec_path_pb = PathBuf::from(spec_path);

    // Read and parse spec (JSON or Starlark)
    let load_result = load_spec(&spec_path_pb)
        .with_context(|| format!("Failed to load spec: {}", spec_path))?;

    let mut spec = load_result.spec;

    // Verify it's a supported 3D asset type
    match spec.asset_type.as_str() {
        "static_mesh" | "skeletal_mesh" | "skeletal_animation" | "sprite" => {}
        other => {
            anyhow::bail!(
                "preview-grid only supports 3D assets (static_mesh, skeletal_mesh, skeletal_animation, sprite), got: {}",
                other
            );
        }
    }

    // Set up output path
    let spec_dir = spec_path_pb.parent().unwrap_or_else(|| Path::new("."));

    let out_path = if let Some(out) = out {
        PathBuf::from(out)
    } else {
        let stem = spec_path_pb
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("preview");
        spec_dir.join(format!("{}.grid.png", stem))
    };

    // Inject panel_size into recipe params for Blender
    if let Some(ref mut recipe) = spec.recipe {
        if let Some(params) = recipe.params.as_object_mut() {
            params.insert("panel_size".to_string(), serde_json::json!(panel_size));
        }
    }

    // Update spec outputs to point to our grid path
    let out_rel = out_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("validation_grid.png");

    spec.outputs = vec![speccade_spec::OutputSpec {
        kind: speccade_spec::OutputKind::Primary,
        format: speccade_spec::OutputFormat::Png,
        path: out_rel.to_string(),
        source: None,
    }];

    // Create output directory
    let out_root = out_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(out_root)
        .with_context(|| format!("Failed to create output directory: {}", out_root.display()))?;

    // Write spec to temp file for Blender
    let temp_dir = tempfile::tempdir()?;
    let temp_spec_path = temp_dir.path().join("spec.json");
    let temp_report_path = temp_dir.path().join("report.json");

    let spec_json = serde_json::to_string_pretty(&spec)?;
    fs::write(&temp_spec_path, &spec_json)?;

    // Run Blender
    let orchestrator = Orchestrator::new();
    let report = orchestrator.run(
        GenerationMode::ValidationGrid,
        &temp_spec_path,
        out_root,
        &temp_report_path,
    )?;

    if !report.ok {
        let error = report.error.unwrap_or_else(|| "Unknown error".to_string());
        anyhow::bail!("Blender validation grid generation failed: {}", error);
    }

    println!(
        "{} Grid saved to: {}",
        "OK".green().bold(),
        out_path.display()
    );

    if let Some(duration_ms) = report.duration_ms {
        println!("  Rendered in {}ms", duration_ms);
    }

    Ok(ExitCode::SUCCESS)
}

/// Extracts the [VALIDATION] comment block from a Starlark source file.
///
/// Looks for `# [VALIDATION]` and collects all subsequent `#` lines
/// until a blank line or non-comment line is encountered.
pub fn extract_validation_comments(source: &str) -> Option<String> {
    let mut in_validation_block = false;
    let mut lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed == "# [VALIDATION]" {
            in_validation_block = true;
            continue;
        }

        if in_validation_block {
            if trimmed.starts_with('#') {
                // Strip the leading `# ` and collect
                let content = trimmed
                    .strip_prefix("# ")
                    .unwrap_or(trimmed.strip_prefix('#').unwrap_or(trimmed));
                lines.push(content.to_string());
            } else if trimmed.is_empty() {
                // Empty line ends the block
                break;
            } else {
                // Non-comment line ends the block
                break;
            }
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_validation_comments() {
        let source = r#"
# Simple cube mesh
#
# [VALIDATION]
# SHAPE: A beveled cube with smooth subdivision
# PROPORTIONS: Equal 1.0 unit dimensions on all axes
# ORIENTATION: Cube centered at origin
# FRONT VIEW: Square face visible
# NOTES: Bevel creates smooth edges

spec(
    asset_id = "mesh-cube-01",
    ...
)
"#;
        let comments = extract_validation_comments(source);
        assert!(comments.is_some());
        let text = comments.unwrap();
        assert!(text.contains("SHAPE:"));
        assert!(text.contains("beveled cube"));
        assert!(text.contains("FRONT VIEW:"));
    }

    #[test]
    fn test_extract_validation_comments_none() {
        let source = r#"
# Simple cube mesh - no validation block
spec(
    asset_id = "mesh-cube-01",
)
"#;
        let comments = extract_validation_comments(source);
        assert!(comments.is_none());
    }

    #[test]
    fn test_unsupported_asset_type() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.json");

        let spec_json = r#"{
            "spec_version": 1,
            "asset_id": "test-asset-01",
            "asset_type": "audio",
            "license": "CC0-1.0",
            "seed": 42,
            "outputs": [
                {
                    "kind": "primary",
                    "format": "wav",
                    "path": "sounds/test.wav"
                }
            ]
        }"#;

        std::fs::write(&spec_path, spec_json).unwrap();

        let result = run(spec_path.to_str().unwrap(), None, 256);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("preview-grid only supports 3D assets"));
    }

    #[test]
    fn test_nonexistent_file() {
        let result = run("/nonexistent/spec.json", None, 256);
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "requires Blender"]
    fn test_preview_grid_end_to_end() {
        let tmp = tempfile::tempdir().unwrap();
        let out_path = tmp.path().join("cube.grid.png");

        // This test requires Blender to be installed
        let result = run(
            "golden/starlark/mesh_cube.star",
            Some(out_path.to_str().unwrap()),
            128,
        );

        // If Blender is not available, this will fail with a specific error
        if let Err(ref e) = result {
            if e.to_string().contains("Blender not found")
                || e.to_string().contains("BlenderNotFound")
            {
                eprintln!("Skipping test: Blender not available");
                return;
            }
        }

        assert!(result.is_ok());
        assert!(out_path.exists());

        // Verify it's a valid PNG
        let data = std::fs::read(&out_path).unwrap();
        assert!(data.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG magic
    }
}
