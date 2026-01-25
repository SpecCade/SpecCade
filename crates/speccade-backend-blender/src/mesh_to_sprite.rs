//! Mesh-to-sprite generation backend.
//!
//! This module renders a 3D mesh from multiple rotation angles and packs
//! the resulting frames into a sprite atlas with metadata.

use std::path::{Path, PathBuf};

use speccade_spec::{OutputKind, Spec};

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::BlenderReport;
use crate::orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

/// Result of mesh-to-sprite generation.
#[derive(Debug)]
pub struct MeshToSpriteResult {
    /// Path to the generated sprite atlas PNG.
    pub output_path: PathBuf,
    /// Path to the generated metadata JSON (if requested).
    pub metadata_path: Option<PathBuf>,
    /// Generation metrics.
    pub metrics: MeshToSpriteMetrics,
    /// Raw Blender report.
    pub report: BlenderReport,
}

/// Metrics specific to mesh-to-sprite generation.
#[derive(Debug, Clone, Default)]
pub struct MeshToSpriteMetrics {
    /// Atlas dimensions [width, height] in pixels.
    pub atlas_dimensions: Option<[u32; 2]>,
    /// Number of frames rendered.
    pub frame_count: Option<u32>,
    /// Frame resolution [width, height] in pixels.
    pub frame_resolution: Option<[u32; 2]>,
    /// Camera preset used.
    pub camera: Option<String>,
    /// Lighting preset used.
    pub lighting: Option<String>,
    /// Generation duration in milliseconds.
    pub duration_ms: Option<u64>,
}

/// Generate a sprite atlas from a 3D mesh.
///
/// # Arguments
///
/// * `spec` - The spec containing `sprite.render_from_mesh_v1` recipe
/// * `out_root` - Root directory for output files
///
/// # Returns
///
/// A `MeshToSpriteResult` on success, or a `BlenderError` on failure.
pub fn generate(spec: &Spec, out_root: &Path) -> BlenderResult<MeshToSpriteResult> {
    // Validate recipe kind
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;
    if recipe.kind != "sprite.render_from_mesh_v1" {
        return Err(BlenderError::InvalidRecipeKind {
            kind: recipe.kind.clone(),
        });
    }

    // Create orchestrator with default config
    let orchestrator = Orchestrator::new();

    // Create temp directory for spec and report files
    let temp_dir = tempfile::tempdir().map_err(BlenderError::Io)?;
    let spec_path = temp_dir.path().join("spec.json");
    let report_path = temp_dir.path().join("report.json");

    // Write spec to temp file
    let spec_json = serde_json::to_string(spec).map_err(|e| BlenderError::GenerationFailed {
        message: format!("Failed to serialize spec: {}", e),
    })?;
    std::fs::write(&spec_path, &spec_json).map_err(BlenderError::WriteSpecFailed)?;

    // Run Blender
    let report = orchestrator.run(
        GenerationMode::MeshToSprite,
        &spec_path,
        out_root,
        &report_path,
    )?;

    // Read the report as raw JSON to extract sprite-specific metrics
    let report_content =
        std::fs::read_to_string(&report_path).map_err(|e| BlenderError::ReadReportFailed {
            path: report_path.clone(),
            source: e,
        })?;
    let raw_report: serde_json::Value =
        serde_json::from_str(&report_content).map_err(BlenderError::ParseReportFailed)?;

    // Extract metrics from raw JSON
    let metrics = extract_metrics_from_json(&raw_report, report.duration_ms);

    // Get primary output path
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| BlenderError::GenerationFailed {
            message: "No primary output specified".to_string(),
        })?;

    let output_path = out_root.join(&primary_output.path);

    // Check if metadata output was requested
    let metadata_path = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Metadata)
        .map(|o| out_root.join(&o.path));

    Ok(MeshToSpriteResult {
        output_path,
        metadata_path,
        metrics,
        report,
    })
}

/// Generate with custom orchestrator configuration.
pub fn generate_with_config(
    spec: &Spec,
    out_root: &Path,
    config: OrchestratorConfig,
) -> BlenderResult<MeshToSpriteResult> {
    // Validate recipe kind
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;
    if recipe.kind != "sprite.render_from_mesh_v1" {
        return Err(BlenderError::InvalidRecipeKind {
            kind: recipe.kind.clone(),
        });
    }

    let orchestrator = Orchestrator::with_config(config);

    // Create temp directory for spec and report files
    let temp_dir = tempfile::tempdir().map_err(BlenderError::Io)?;
    let spec_path = temp_dir.path().join("spec.json");
    let report_path = temp_dir.path().join("report.json");

    // Write spec to temp file
    let spec_json = serde_json::to_string(spec).map_err(|e| BlenderError::GenerationFailed {
        message: format!("Failed to serialize spec: {}", e),
    })?;
    std::fs::write(&spec_path, &spec_json).map_err(BlenderError::WriteSpecFailed)?;

    // Run Blender
    let report = orchestrator.run(
        GenerationMode::MeshToSprite,
        &spec_path,
        out_root,
        &report_path,
    )?;

    // Read the report as raw JSON to extract sprite-specific metrics
    let report_content =
        std::fs::read_to_string(&report_path).map_err(|e| BlenderError::ReadReportFailed {
            path: report_path.clone(),
            source: e,
        })?;
    let raw_report: serde_json::Value =
        serde_json::from_str(&report_content).map_err(BlenderError::ParseReportFailed)?;

    // Extract metrics from raw JSON
    let metrics = extract_metrics_from_json(&raw_report, report.duration_ms);

    // Get primary output path
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| BlenderError::GenerationFailed {
            message: "No primary output specified".to_string(),
        })?;

    let output_path = out_root.join(&primary_output.path);

    // Check if metadata output was requested
    let metadata_path = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Metadata)
        .map(|o| out_root.join(&o.path));

    Ok(MeshToSpriteResult {
        output_path,
        metadata_path,
        metrics,
        report,
    })
}

/// Extract mesh-to-sprite metrics from a raw JSON report.
fn extract_metrics_from_json(
    raw_report: &serde_json::Value,
    duration_ms: Option<u64>,
) -> MeshToSpriteMetrics {
    let mut metrics = MeshToSpriteMetrics::default();

    // Extract from report metrics if available
    if let Some(raw_metrics) = raw_report.get("metrics") {
        // Atlas dimensions
        if let Some(dims) = raw_metrics.get("atlas_dimensions") {
            if let Some(arr) = dims.as_array() {
                if arr.len() >= 2 {
                    metrics.atlas_dimensions = Some([
                        arr[0].as_u64().unwrap_or(0) as u32,
                        arr[1].as_u64().unwrap_or(0) as u32,
                    ]);
                }
            }
        }

        // Frame count
        if let Some(count) = raw_metrics.get("frame_count") {
            metrics.frame_count = count.as_u64().map(|c| c as u32);
        }

        // Frame resolution
        if let Some(res) = raw_metrics.get("frame_resolution") {
            if let Some(arr) = res.as_array() {
                if arr.len() >= 2 {
                    metrics.frame_resolution = Some([
                        arr[0].as_u64().unwrap_or(0) as u32,
                        arr[1].as_u64().unwrap_or(0) as u32,
                    ]);
                }
            }
        }

        // Camera preset
        if let Some(camera) = raw_metrics.get("camera") {
            metrics.camera = camera.as_str().map(String::from);
        }

        // Lighting preset
        if let Some(lighting) = raw_metrics.get("lighting") {
            metrics.lighting = lighting.as_str().map(String::from);
        }
    }

    // Duration from report
    metrics.duration_ms = duration_ms;

    metrics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_metrics_from_empty_json() {
        let raw_report = serde_json::json!({
            "ok": true
        });

        let metrics = extract_metrics_from_json(&raw_report, Some(100));
        assert!(metrics.atlas_dimensions.is_none());
        assert!(metrics.frame_count.is_none());
        assert_eq!(metrics.duration_ms, Some(100));
    }

    #[test]
    fn test_extract_metrics_with_data() {
        let raw_report = serde_json::json!({
            "ok": true,
            "metrics": {
                "atlas_dimensions": [256, 256],
                "frame_count": 8,
                "frame_resolution": [64, 64],
                "camera": "orthographic",
                "lighting": "three_point"
            },
            "duration_ms": 500
        });

        let metrics = extract_metrics_from_json(&raw_report, Some(500));
        assert_eq!(metrics.atlas_dimensions, Some([256, 256]));
        assert_eq!(metrics.frame_count, Some(8));
        assert_eq!(metrics.frame_resolution, Some([64, 64]));
        assert_eq!(metrics.camera, Some("orthographic".to_string()));
        assert_eq!(metrics.lighting, Some("three_point".to_string()));
        assert_eq!(metrics.duration_ms, Some(500));
    }

    #[test]
    fn test_metrics_default() {
        let metrics = MeshToSpriteMetrics::default();
        assert!(metrics.atlas_dimensions.is_none());
        assert!(metrics.frame_count.is_none());
        assert!(metrics.frame_resolution.is_none());
        assert!(metrics.camera.is_none());
        assert!(metrics.lighting.is_none());
        assert!(metrics.duration_ms.is_none());
    }
}
