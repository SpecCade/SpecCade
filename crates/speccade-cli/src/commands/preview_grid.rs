//! Preview grid command implementation
//!
//! Generates a multi-angle validation grid PNG for 3D assets.

use anyhow::{Context, Result};
use colored::Colorize;
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use speccade_backend_blender::{GenerationMode, Orchestrator};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::input::load_spec;

/// View labels in grid order (2 rows x 3 columns)
const VIEW_LABELS: [&str; 6] = ["FRONT", "BACK", "TOP", "LEFT", "RIGHT", "ISO"];

/// Grid padding between panels
const GRID_PADDING: u32 = 4;

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

    // Check if Blender created individual frames (PIL not available) or a composited grid
    let frames_dir = out_root.join("validation_grid_frames");
    let final_out_path = if frames_dir.exists() && frames_dir.is_dir() {
        // Composite individual frames into grid on Rust side
        composite_frames_to_grid(&frames_dir, &out_path, panel_size)?;
        // Clean up individual frames
        fs::remove_dir_all(&frames_dir).ok();
        out_path.clone()
    } else if out_path.exists() {
        out_path.clone()
    } else {
        // Check if output was written elsewhere in out_root
        let alt_path = out_root.join(out_rel);
        if alt_path.exists() {
            if alt_path != out_path {
                fs::rename(&alt_path, &out_path)?;
            }
            out_path.clone()
        } else {
            anyhow::bail!("Blender completed but output not found at {} or {}", out_path.display(), frames_dir.display());
        }
    };

    println!(
        "{} Grid saved to: {}",
        "OK".green().bold(),
        final_out_path.display()
    );

    if let Some(duration_ms) = report.duration_ms {
        println!("  Rendered in {}ms", duration_ms);
    }

    Ok(ExitCode::SUCCESS)
}

/// Composite individual frame PNGs into a 3x2 grid
fn composite_frames_to_grid(frames_dir: &Path, out_path: &Path, panel_size: u32) -> Result<()> {
    // Load each view
    let mut frames: Vec<Option<DynamicImage>> = Vec::with_capacity(6);

    for label in VIEW_LABELS {
        let frame_path = frames_dir.join(format!("{}.png", label));
        if frame_path.exists() {
            let img = image::open(&frame_path)
                .with_context(|| format!("Failed to load frame: {}", frame_path.display()))?;
            frames.push(Some(img));
        } else {
            frames.push(None);
        }
    }

    // Calculate grid dimensions (3 cols x 2 rows)
    let grid_width = panel_size * 3 + GRID_PADDING * 4;
    let grid_height = panel_size * 2 + GRID_PADDING * 3;

    // Create output image with transparent background
    let mut grid: RgbaImage = ImageBuffer::from_pixel(grid_width, grid_height, Rgba([0, 0, 0, 0]));

    // Place each frame in the grid
    for (i, frame_opt) in frames.iter().enumerate() {
        let col = (i % 3) as u32;
        let row = (i / 3) as u32;
        let x = GRID_PADDING + col * (panel_size + GRID_PADDING);
        let y = GRID_PADDING + row * (panel_size + GRID_PADDING);

        if let Some(frame) = frame_opt {
            // Resize frame to panel_size if needed
            let resized = frame.resize_exact(panel_size, panel_size, image::imageops::FilterType::Lanczos3);
            let rgba = resized.to_rgba8();

            // Copy pixels to grid
            for (fx, fy, pixel) in rgba.enumerate_pixels() {
                let gx = x + fx;
                let gy = y + fy;
                if gx < grid_width && gy < grid_height {
                    grid.put_pixel(gx, gy, *pixel);
                }
            }

            // Draw label background and text
            draw_label(&mut grid, x + 4, y + 4, VIEW_LABELS[i]);
        }
    }

    // Save the composited grid
    grid.save(out_path)
        .with_context(|| format!("Failed to save grid: {}", out_path.display()))?;

    Ok(())
}

/// Draw a simple label on the image (basic implementation without font rendering)
fn draw_label(img: &mut RgbaImage, x: u32, y: u32, label: &str) {
    // Draw a semi-transparent black background rectangle
    let label_width = (label.len() * 8) as u32 + 4;
    let label_height = 16_u32;

    for ly in 0..label_height {
        for lx in 0..label_width {
            let px = x + lx;
            let py = y + ly;
            if px < img.width() && py < img.height() {
                img.put_pixel(px, py, Rgba([0, 0, 0, 180]));
            }
        }
    }

    // Note: Full text rendering would require a font library
    // For now, we just have the background - Blender's PIL fallback should handle this
    // or we could add the `imageproc` crate for text rendering in the future
    let _ = label; // Suppress unused warning
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
        // Create a minimal spec file in a temp directory
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.json");
        let out_path = tmp.path().join("cube.grid.png");

        // Write a minimal static mesh spec
        let spec_json = r#"{
            "spec_version": 1,
            "asset_id": "test-cube-grid",
            "asset_type": "static_mesh",
            "license": "CC0-1.0",
            "seed": 42,
            "recipe": {
                "kind": "static_mesh.blender_primitives_v1",
                "params": {
                    "base_primitive": "cube",
                    "dimensions": [1.0, 1.0, 1.0],
                    "panel_size": 128
                }
            },
            "outputs": [
                {
                    "kind": "primary",
                    "format": "png",
                    "path": "cube.grid.png"
                }
            ]
        }"#;

        std::fs::write(&spec_path, spec_json).unwrap();

        // This test requires Blender to be installed
        let result = run(
            spec_path.to_str().unwrap(),
            Some(out_path.to_str().unwrap()),
            128,
        );

        // If Blender is not available, gracefully skip the test
        if let Err(ref e) = result {
            let error_str = e.to_string();
            if error_str.contains("Blender") && error_str.contains("not found") {
                eprintln!("Skipping test: Blender not available");
                return;
            }
            if error_str.contains("BlenderNotFound") {
                eprintln!("Skipping test: Blender not available");
                return;
            }
            if error_str.contains("Failed to spawn Blender") {
                eprintln!("Skipping test: Blender not available");
                return;
            }
        }

        // Test passes if run() succeeded
        assert!(result.is_ok(), "Unexpected error: {:?}", result.err());

        // Verify output was created - check for either the final PNG or the frames directory
        let out_dir = tmp.path();
        let output_exists = out_path.exists() || out_dir.join("validation_grid_frames").exists();
        assert!(output_exists, "Output not found in {}", out_dir.display());

        // If the final PNG exists, verify it's a valid PNG
        if out_path.exists() {
            let data = std::fs::read(&out_path).unwrap();
            assert!(data.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG magic
        }
    }
}
