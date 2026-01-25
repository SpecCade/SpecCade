//! Simple Subprocess Extension
//!
//! This is a reference implementation of a SpecCade subprocess extension.
//! It demonstrates the I/O contract and determinism requirements.
//!
//! # Recipe Kind
//!
//! This extension handles `texture.simple_gradient_v1` specs.
//!
//! # Parameters
//!
//! - `width`: Texture width in pixels (default: 256)
//! - `height`: Texture height in pixels (default: 256)
//! - `start_color`: RGBA start color [r, g, b, a] (default: [0, 0, 0, 255])
//! - `end_color`: RGBA end color [r, g, b, a] (default: [255, 255, 255, 255])
//! - `direction`: Gradient direction "horizontal" or "vertical" (default: "horizontal")
//!
//! # Usage
//!
//! ```bash
//! simple-subprocess-extension --spec input.spec.json --out ./output --seed 42
//! ```

use clap::Parser;
use rand::prelude::*;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Parser, Debug)]
#[command(name = "simple-subprocess-extension")]
#[command(about = "Reference SpecCade subprocess extension")]
struct Args {
    /// Path to input spec JSON file
    #[arg(long)]
    spec: PathBuf,

    /// Output directory
    #[arg(long)]
    out: PathBuf,

    /// Seed for deterministic generation
    #[arg(long)]
    seed: u64,
}

// ============================================================================
// Spec Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields read from JSON but not all used in code
struct Spec {
    asset_id: String,
    seed: u32,
    outputs: Vec<OutputSpec>,
    recipe: Option<Recipe>,
}

#[derive(Debug, Deserialize)]
struct OutputSpec {
    kind: String,
    format: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct Recipe {
    kind: String,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct GradientParams {
    width: u32,
    height: u32,
    start_color: [u8; 4],
    end_color: [u8; 4],
    direction: String,
    noise_amount: f32,
}

impl Default for GradientParams {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            start_color: [0, 0, 0, 255],
            end_color: [255, 255, 255, 255],
            direction: "horizontal".to_string(),
            noise_amount: 0.0,
        }
    }
}

// ============================================================================
// Output Manifest Types
// ============================================================================

#[derive(Debug, Serialize)]
struct OutputManifest {
    manifest_version: u32,
    success: bool,
    output_files: Vec<OutputFile>,
    determinism_report: DeterminismReport,
    errors: Vec<ErrorEntry>,
    warnings: Vec<String>,
    duration_ms: u64,
    extension_version: String,
}

#[derive(Debug, Serialize)]
struct OutputFile {
    path: String,
    hash: String,
    size: u64,
    kind: String,
    format: String,
}

#[derive(Debug, Serialize)]
struct DeterminismReport {
    input_hash: String,
    output_hash: Option<String>,
    tier: u8,
    determinism: String,
    seed: u64,
    deterministic: bool,
}

#[derive(Debug, Serialize)]
struct ErrorEntry {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<serde_json::Value>,
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    let args = Args::parse();
    let start = Instant::now();

    // Create output directory
    if let Err(e) = fs::create_dir_all(&args.out) {
        write_error_manifest(
            &args.out,
            &args,
            "IO_ERROR",
            &format!("Failed to create output directory: {}", e),
            start.elapsed().as_millis() as u64,
        );
        std::process::exit(1);
    }

    // Read and parse spec
    let spec_content = match fs::read_to_string(&args.spec) {
        Ok(content) => content,
        Err(e) => {
            write_error_manifest(
                &args.out,
                &args,
                "IO_ERROR",
                &format!("Failed to read spec file: {}", e),
                start.elapsed().as_millis() as u64,
            );
            std::process::exit(2);
        }
    };

    let spec: Spec = match serde_json::from_str(&spec_content) {
        Ok(spec) => spec,
        Err(e) => {
            write_error_manifest(
                &args.out,
                &args,
                "INVALID_SPEC",
                &format!("Failed to parse spec: {}", e),
                start.elapsed().as_millis() as u64,
            );
            std::process::exit(3);
        }
    };

    // Compute input hash
    let input_hash = compute_canonical_hash(&spec_content);

    // Validate recipe
    let recipe = match &spec.recipe {
        Some(r) => r,
        None => {
            write_error_manifest(
                &args.out,
                &args,
                "MISSING_RECIPE",
                "Spec has no recipe",
                start.elapsed().as_millis() as u64,
            );
            std::process::exit(3);
        }
    };

    if recipe.kind != "texture.simple_gradient_v1" {
        write_error_manifest(
            &args.out,
            &args,
            "UNSUPPORTED_RECIPE",
            &format!("Unsupported recipe kind: {}", recipe.kind),
            start.elapsed().as_millis() as u64,
        );
        std::process::exit(3);
    }

    // Parse params
    let params: GradientParams = match serde_json::from_value(recipe.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            write_error_manifest(
                &args.out,
                &args,
                "INVALID_PARAM",
                &format!("Failed to parse params: {}", e),
                start.elapsed().as_millis() as u64,
            );
            std::process::exit(3);
        }
    };

    // Validate params
    if params.width == 0 || params.height == 0 {
        write_error_manifest(
            &args.out,
            &args,
            "INVALID_PARAM",
            "Width and height must be greater than 0",
            start.elapsed().as_millis() as u64,
        );
        std::process::exit(3);
    }

    if params.direction != "horizontal" && params.direction != "vertical" {
        write_error_manifest(
            &args.out,
            &args,
            "INVALID_PARAM",
            &format!("Invalid direction '{}', must be 'horizontal' or 'vertical'", params.direction),
            start.elapsed().as_millis() as u64,
        );
        std::process::exit(3);
    }

    // Get primary output
    let primary_output = match spec.outputs.iter().find(|o| o.kind == "primary") {
        Some(o) => o,
        None => {
            write_error_manifest(
                &args.out,
                &args,
                "INVALID_SPEC",
                "No primary output specified",
                start.elapsed().as_millis() as u64,
            );
            std::process::exit(3);
        }
    };

    if primary_output.format != "png" {
        write_error_manifest(
            &args.out,
            &args,
            "UNSUPPORTED_FORMAT",
            &format!("Unsupported output format '{}', only 'png' is supported", primary_output.format),
            start.elapsed().as_millis() as u64,
        );
        std::process::exit(3);
    }

    // Generate texture
    let png_data = generate_gradient(&params, args.seed);

    // Write output file
    let output_path = args.out.join(&primary_output.path);
    if let Some(parent) = output_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            write_error_manifest(
                &args.out,
                &args,
                "IO_ERROR",
                &format!("Failed to create output directory: {}", e),
                start.elapsed().as_millis() as u64,
            );
            std::process::exit(4);
        }
    }

    if let Err(e) = fs::write(&output_path, &png_data) {
        write_error_manifest(
            &args.out,
            &args,
            "IO_ERROR",
            &format!("Failed to write output file: {}", e),
            start.elapsed().as_millis() as u64,
        );
        std::process::exit(4);
    }

    // Compute output hash
    let output_hash = blake3::hash(&png_data).to_hex().to_string();

    // Create success manifest
    let manifest = OutputManifest {
        manifest_version: 1,
        success: true,
        output_files: vec![OutputFile {
            path: primary_output.path.clone(),
            hash: output_hash.clone(),
            size: png_data.len() as u64,
            kind: "primary".to_string(),
            format: "png".to_string(),
        }],
        determinism_report: DeterminismReport {
            input_hash,
            output_hash: Some(output_hash),
            tier: 1,
            determinism: "byte_identical".to_string(),
            seed: args.seed,
            deterministic: true,
        },
        errors: vec![],
        warnings: vec![],
        duration_ms: start.elapsed().as_millis() as u64,
        extension_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    // Write manifest
    let manifest_path = args.out.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
    if let Err(e) = fs::write(&manifest_path, &manifest_json) {
        eprintln!("Failed to write manifest: {}", e);
        std::process::exit(4);
    }

    eprintln!(
        "[simple-subprocess-extension] Generated {} ({}x{}) in {}ms",
        primary_output.path,
        params.width,
        params.height,
        manifest.duration_ms
    );
}

// ============================================================================
// Generation
// ============================================================================

fn generate_gradient(params: &GradientParams, seed: u64) -> Vec<u8> {
    let mut rng = Pcg64::seed_from_u64(seed);

    // Create RGBA pixel data
    let mut pixels = Vec::with_capacity((params.width * params.height * 4) as usize);

    for y in 0..params.height {
        for x in 0..params.width {
            // Calculate gradient factor (0.0 to 1.0)
            let t = match params.direction.as_str() {
                "horizontal" => x as f32 / (params.width - 1).max(1) as f32,
                "vertical" => y as f32 / (params.height - 1).max(1) as f32,
                _ => 0.5,
            };

            // Add noise if specified
            let noise = if params.noise_amount > 0.0 {
                (rng.gen::<f32>() - 0.5) * params.noise_amount
            } else {
                0.0
            };

            let t = (t + noise).clamp(0.0, 1.0);

            // Interpolate colors
            let r = lerp(params.start_color[0], params.end_color[0], t);
            let g = lerp(params.start_color[1], params.end_color[1], t);
            let b = lerp(params.start_color[2], params.end_color[2], t);
            let a = lerp(params.start_color[3], params.end_color[3], t);

            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
            pixels.push(a);
        }
    }

    // Encode as PNG
    encode_png(params.width, params.height, &pixels)
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    let a = a as f32;
    let b = b as f32;
    (a + (b - a) * t).round() as u8
}

fn encode_png(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        // Use deterministic compression settings
        encoder.set_compression(png::Compression::Default);
        encoder.set_filter(png::FilterType::NoFilter);

        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(pixels).unwrap();
    }
    output
}

// ============================================================================
// Helpers
// ============================================================================

/// Computes a canonical hash of JSON content.
///
/// This is a simplified version - in production, use proper JCS canonicalization.
fn compute_canonical_hash(json: &str) -> String {
    // Parse and re-serialize to normalize whitespace
    // Note: This doesn't do full JCS canonicalization, but is sufficient for demo
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    let canonical = serde_json::to_string(&value).unwrap();
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}

fn write_error_manifest(
    out_dir: &Path,
    args: &Args,
    code: &str,
    message: &str,
    duration_ms: u64,
) {
    // Try to compute input hash if spec file exists
    let input_hash = fs::read_to_string(&args.spec)
        .map(|s| compute_canonical_hash(&s))
        .unwrap_or_else(|_| "0".repeat(64));

    let manifest = OutputManifest {
        manifest_version: 1,
        success: false,
        output_files: vec![],
        determinism_report: DeterminismReport {
            input_hash,
            output_hash: None,
            tier: 1,
            determinism: "byte_identical".to_string(),
            seed: args.seed,
            deterministic: true,
        },
        errors: vec![ErrorEntry {
            code: code.to_string(),
            message: message.to_string(),
            context: None,
        }],
        warnings: vec![],
        duration_ms,
        extension_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let manifest_path = out_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
    let _ = fs::write(manifest_path, manifest_json);

    eprintln!("[simple-subprocess-extension] Error: {} - {}", code, message);
}
