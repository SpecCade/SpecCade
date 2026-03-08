//! Validate-asset command implementation.
//!
//! Runs a full validation pipeline on a single 3D asset spec:
//! 1. Generate the asset
//! 2. Attempt preview-grid generation
//! 3. Read canonical metrics from the generation report
//! 4. Reuse generation lint results

use anyhow::{Context, Result};
use colored::{ColoredString, Colorize};
use serde::Serialize;
use speccade_spec::{report::LintReportData, OutputMetrics, Report, Spec};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use crate::commands::generate;
use crate::commands::preview_grid;
use crate::commands::reporting;
use crate::input::load_spec;

/// Comprehensive validation report.
#[derive(Debug, Serialize)]
struct ValidationReport {
    spec_path: String,
    asset_id: String,
    asset_type: String,
    timestamp: String,
    generation: GenerationResult,
    visual_evidence: VisualEvidence,
    metrics: serde_json::Value,
    lint_results: Vec<serde_json::Value>,
    quality_gates: QualityGates,
    validation_comments: Option<String>,
}

#[derive(Debug, Serialize)]
struct GenerationResult {
    success: bool,
    asset_path: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct VisualEvidence {
    grid_path: Option<String>,
    grid_generated: bool,
}

#[derive(Debug, Serialize, Clone, Copy)]
struct QualityGates {
    generation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_geometry: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manifold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_uvs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skeleton_valid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    animation_valid: Option<bool>,
}

/// Run the validate-asset command.
pub fn run(spec_path: &str, out_root: Option<&str>, _full_report: bool) -> Result<ExitCode> {
    let spec_path_pb = PathBuf::from(spec_path);

    let load_result =
        load_spec(&spec_path_pb).with_context(|| format!("Failed to load spec: {}", spec_path))?;
    let spec = load_result.spec;

    match spec.asset_type.as_str() {
        "static_mesh" | "skeletal_mesh" | "skeletal_animation" => {}
        other => {
            anyhow::bail!(
                "validate-asset only supports 3D assets (static_mesh, skeletal_mesh, skeletal_animation), got: {}",
                other
            );
        }
    }

    let validation_comments = if spec_path.ends_with(".star") {
        let source = std::fs::read_to_string(&spec_path_pb).ok();
        source
            .as_ref()
            .and_then(|s| preview_grid::extract_validation_comments(s))
    } else {
        None
    };

    let out_dir = out_root
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("validation-output"));
    fs::create_dir_all(&out_dir)?;

    println!("Validating: {} (type: {})", spec_path, spec.asset_type);
    println!("Output directory: {}", out_dir.display());
    let validation_start = Instant::now();

    println!("\n[1/4] Generating asset...");
    let step_start = Instant::now();
    let gen_result = generate::run(
        spec_path,
        Some(out_dir.to_str().unwrap()),
        false,
        None,
        false,
        None,
        false,
        false,
        None,
        None,
        None,
        false,
    );
    let gen_elapsed = step_start.elapsed();

    match gen_result {
        Ok(ExitCode::SUCCESS) => {
            println!(
                "  OK Asset generated successfully ({})",
                format!("{:.1}s", gen_elapsed.as_secs_f64()).dimmed()
            );
        }
        Ok(code) => {
            println!(
                "  FAIL Generation failed with exit code {:?} ({})",
                code,
                format!("{:.1}s", gen_elapsed.as_secs_f64()).dimmed()
            );
            return Ok(code);
        }
        Err(e) => {
            println!(
                "  FAIL Generation error after {}: {}",
                format!("{:.1}s", gen_elapsed.as_secs_f64()).dimmed(),
                e
            );
            return Err(e).context(format!("Asset generation failed for {}", spec_path));
        }
    }

    let asset_path = find_generated_glb(&out_dir, &spec).ok_or_else(|| {
        anyhow::anyhow!(
            "Generated GLB not found in {} after successful generation",
            out_dir.display()
        )
    })?;
    println!("  Generated: {}", asset_path.display());

    let generation_report = load_generation_report(spec_path, &spec.asset_id)?;
    let output_metrics = extract_output_metrics(&generation_report).ok_or_else(|| {
        anyhow::anyhow!(
            "Generation report {} did not contain output metrics",
            reporting::report_path(spec_path, &spec.asset_id)
        )
    })?;

    println!("\n[2/4] Generating preview grid...");
    let step_start = Instant::now();
    let grid_filename = format!("{}.grid.png", spec.asset_id.replace("/", "_"));
    let grid_path = out_dir.join(&grid_filename);
    let grid_result = preview_grid::run(spec_path, Some(grid_path.to_str().unwrap()), 256);
    let grid_elapsed = step_start.elapsed();

    match grid_result {
        Ok(ExitCode::SUCCESS) => {
            println!(
                "  OK Preview grid generated ({})",
                format!("{:.1}s", grid_elapsed.as_secs_f64()).dimmed()
            );
        }
        Ok(code) => {
            println!(
                "  WARN Preview grid failed after {} (exit code: {:?}), continuing...",
                format!("{:.1}s", grid_elapsed.as_secs_f64()).dimmed(),
                code
            );
        }
        Err(e) => {
            println!(
                "  WARN Preview grid error after {}: {}, continuing...",
                format!("{:.1}s", grid_elapsed.as_secs_f64()).dimmed(),
                e
            );
        }
    }

    println!("\n[3/4] Reading generation metrics...");
    let step_start = Instant::now();
    let metrics_json = serde_json::to_value(&output_metrics)?;
    let metrics_elapsed = step_start.elapsed();
    println!(
        "  OK Metrics loaded from generation report ({})",
        format!("{:.1}s", metrics_elapsed.as_secs_f64()).dimmed()
    );
    print_metric_summary(&output_metrics);

    let metrics_path = out_dir.join(format!("{}.metrics.json", spec.asset_id.replace("/", "_")));
    std::fs::write(&metrics_path, serde_json::to_string_pretty(&metrics_json)?)?;
    println!("  Metrics saved: {}", metrics_path.display());

    println!("\n[4/4] Reading lint results...");
    let step_start = Instant::now();
    let lint_results = lint_results_to_json_values(generation_report.lint.as_ref())?;
    let lint_elapsed = step_start.elapsed();
    let lint_path = out_dir.join(format!("{}.lint.json", spec.asset_id.replace("/", "_")));
    std::fs::write(&lint_path, serde_json::to_string_pretty(&lint_results)?)?;
    println!(
        "  OK Lint loaded ({}) : {} issues found",
        format!("{:.1}s", lint_elapsed.as_secs_f64()).dimmed(),
        lint_results.len()
    );
    print_lint_summary(generation_report.lint.as_ref());
    println!("  Lint report saved: {}", lint_path.display());

    let quality_gates = build_quality_gates(spec.asset_type.as_str(), &output_metrics);

    let report = ValidationReport {
        spec_path: spec_path.to_string(),
        asset_id: spec.asset_id.clone(),
        asset_type: spec.asset_type.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        generation: GenerationResult {
            success: true,
            asset_path: Some(asset_path.to_string_lossy().to_string()),
            error: None,
        },
        visual_evidence: VisualEvidence {
            grid_path: if grid_path.exists() {
                Some(grid_filename)
            } else {
                None
            },
            grid_generated: grid_path.exists(),
        },
        metrics: metrics_json,
        lint_results,
        quality_gates,
        validation_comments,
    };

    let report_path = out_dir.join(format!(
        "{}.validation-report.json",
        spec.asset_id.replace("/", "_")
    ));
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;

    let total_elapsed = validation_start.elapsed();
    println!("\n{}", "=".repeat(60));
    println!(
        "{} {}",
        "VALIDATION COMPLETE".bold().green(),
        format!("({:.1}s)", total_elapsed.as_secs_f64()).dimmed()
    );
    println!("{}", "=".repeat(60));
    println!("Report: {}", report_path.display());
    println!("\nQuality Gates:");
    print_quality_gate("Generation", Some(quality_gates.generation), false);
    print_quality_gate("Has Geometry", quality_gates.has_geometry, false);
    print_quality_gate("Manifold", quality_gates.manifold, true);
    print_quality_gate("Has UVs", quality_gates.has_uvs, true);
    print_quality_gate("Skeleton", quality_gates.skeleton_valid, false);
    print_quality_gate("Animation", quality_gates.animation_valid, false);

    Ok(ExitCode::SUCCESS)
}

fn load_generation_report(spec_path: &str, asset_id: &str) -> Result<Report> {
    let report_path = reporting::report_path(spec_path, asset_id);
    let report_json = fs::read_to_string(&report_path)
        .with_context(|| format!("Failed to read generation report: {}", report_path))?;
    Report::from_json(&report_json)
        .with_context(|| format!("Failed to parse generation report: {}", report_path))
}

fn extract_output_metrics(report: &Report) -> Option<OutputMetrics> {
    report
        .outputs
        .iter()
        .find_map(|output| output.metrics.clone())
}

fn lint_results_to_json_values(lint: Option<&LintReportData>) -> Result<Vec<serde_json::Value>> {
    let Some(lint) = lint else {
        return Ok(Vec::new());
    };

    lint.errors
        .iter()
        .chain(lint.warnings.iter())
        .chain(lint.info.iter())
        .map(serde_json::to_value)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Failed to serialize lint results")
}

fn build_quality_gates(asset_type: &str, metrics: &OutputMetrics) -> QualityGates {
    let mut gates = QualityGates {
        generation: true,
        has_geometry: metrics
            .vertex_count
            .zip(metrics.triangle_count)
            .map(|(vertices, triangles)| vertices > 0 && triangles > 0),
        manifold: metrics.manifold,
        has_uvs: metrics
            .has_uv_map
            .or_else(|| metrics.uv_layer_count.map(|count| count > 0)),
        skeleton_valid: metrics.bone_count.map(|count| count > 0),
        animation_valid: metrics.animation_frame_count.map(|count| count > 0),
    };

    match asset_type {
        "static_mesh" => {
            gates.skeleton_valid = None;
            gates.animation_valid = None;
        }
        "skeletal_mesh" => {
            gates.animation_valid = None;
        }
        "skeletal_animation" => {
            gates.has_geometry = None;
            gates.manifold = None;
            gates.has_uvs = None;
        }
        _ => {}
    }

    gates
}

fn print_metric_summary(metrics: &OutputMetrics) {
    if let (Some(vertices), Some(triangles)) = (metrics.vertex_count, metrics.triangle_count) {
        println!(
            "    - Topology: {} vertices, {} triangles",
            vertices, triangles
        );
    }
    if let Some(bones) = metrics.bone_count {
        println!("    - Skeleton: {} bones", bones);
    }
    if let (Some(frames), Some(duration)) = (
        metrics.animation_frame_count,
        metrics.animation_duration_seconds,
    ) {
        println!(
            "    - Animation: {} frames, {:.2}s duration",
            frames, duration
        );
    }
    if let Some(violations) = metrics.hinge_axis_violations {
        println!("    - Hinge axis violations: {}", violations);
    }
    if let Some(violations) = metrics.range_violations {
        println!("    - Range violations: {}", violations);
    }
    if let Some(spikes) = metrics.velocity_spikes {
        println!("    - Velocity spikes: {}", spikes);
    }
}

fn print_lint_summary(lint: Option<&LintReportData>) {
    let Some(lint) = lint else {
        println!("    - No lint data recorded during generation");
        return;
    };

    for issue in lint
        .errors
        .iter()
        .chain(lint.warnings.iter())
        .chain(lint.info.iter())
    {
        let icon = match issue.severity.as_str() {
            "error" => "FAIL".red(),
            "warning" => "WARN".yellow(),
            _ => "INFO".blue(),
        };
        println!("    - {} [{}] {}", icon, issue.rule_id, issue.message);
    }
}

fn print_quality_gate(label: &str, value: Option<bool>, false_is_warning: bool) {
    println!(
        "  {:<12} {}",
        format!("{}:", label),
        format_gate_status(value, false_is_warning)
    );
}

fn format_gate_status(value: Option<bool>, false_is_warning: bool) -> ColoredString {
    match value {
        Some(true) => "PASS".green(),
        Some(false) if false_is_warning => "WARN".yellow(),
        Some(false) => "FAIL".red(),
        None => "N/A".dimmed(),
    }
}

/// Find the generated GLB file in the output directory.
fn find_generated_glb(out_dir: &Path, spec: &Spec) -> Option<PathBuf> {
    for output in &spec.outputs {
        if output.format == speccade_spec::OutputFormat::Glb {
            let path = out_dir.join(&output.path);
            if path.exists() {
                return Some(path);
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir(out_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "glb").unwrap_or(false) {
                return Some(path);
            }
        }
    }

    None
}
