//! Boolean kit mesh generation handler.
//!
//! This module provides the interface for generating hard-surface meshes using
//! sequential boolean operations via the `static_mesh.boolean_kit_v1` recipe.

use std::path::Path;

use speccade_spec::recipe::mesh::StaticMeshBooleanKitV1Params;
use speccade_spec::Spec;

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::{BlenderMetrics, BlenderReport};
use crate::orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

/// Result of boolean kit mesh generation.
#[derive(Debug, Clone)]
pub struct BooleanKitResult {
    /// Path to the generated GLB file.
    pub output_path: std::path::PathBuf,
    /// Metrics from the generation.
    pub metrics: BlenderMetrics,
    /// The Blender report.
    pub report: BlenderReport,
}

/// Generates a boolean kit mesh from a spec.
pub fn generate(spec: &Spec, out_root: &Path) -> BlenderResult<BooleanKitResult> {
    generate_with_config(spec, out_root, OrchestratorConfig::default())
}

/// Generates a boolean kit mesh from a spec with custom orchestrator configuration.
pub fn generate_with_config(
    spec: &Spec,
    out_root: &Path,
    config: OrchestratorConfig,
) -> BlenderResult<BooleanKitResult> {
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;
    if recipe.kind != "static_mesh.boolean_kit_v1" {
        return Err(BlenderError::InvalidRecipeKind {
            kind: recipe.kind.clone(),
        });
    }

    let _params: StaticMeshBooleanKitV1Params = serde_json::from_value(recipe.params.clone())
        .map_err(BlenderError::DeserializeParamsFailed)?;

    let spec_json = serde_json::to_string(spec).map_err(BlenderError::SerializeFailed)?;

    let orchestrator = Orchestrator::with_config(config);
    let report =
        orchestrator.run_with_spec_json(GenerationMode::BooleanKit, &spec_json, out_root)?;

    let output_path_str = report
        .output_path
        .as_ref()
        .ok_or_else(|| BlenderError::generation_failed("No output path in report"))?;
    let output_path = out_root.join(output_path_str);

    if !output_path.exists() {
        return Err(BlenderError::OutputNotFound {
            path: output_path.clone(),
        });
    }

    let metrics = report
        .metrics
        .clone()
        .ok_or_else(|| BlenderError::generation_failed("No metrics in report"))?;

    Ok(BooleanKitResult {
        output_path,
        metrics,
        report,
    })
}
