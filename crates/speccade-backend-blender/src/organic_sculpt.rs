//! Organic sculpt mesh generation handler.
//!
//! This module provides the interface for generating organic meshes using
//! metaballs, voxel remeshing, smoothing, and displacement noise via
//! the `static_mesh.organic_sculpt_v1` recipe.

use std::path::Path;

use speccade_spec::recipe::mesh::StaticMeshOrganicSculptV1Params;
use speccade_spec::Spec;

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::{BlenderMetrics, BlenderReport};
use crate::orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

/// Result of organic sculpt mesh generation.
#[derive(Debug, Clone)]
pub struct OrganicSculptResult {
    /// Path to the generated GLB file.
    pub output_path: std::path::PathBuf,
    /// Metrics from the generation.
    pub metrics: BlenderMetrics,
    /// The Blender report.
    pub report: BlenderReport,
}

/// Generates an organic sculpt mesh from a spec.
///
/// # Arguments
///
/// * `spec` - The SpecCade spec with a `static_mesh.organic_sculpt_v1` recipe
/// * `out_root` - Root directory for output files
///
/// # Returns
///
/// An `OrganicSculptResult` containing the path to the generated GLB and metrics.
pub fn generate(spec: &Spec, out_root: &Path) -> BlenderResult<OrganicSculptResult> {
    generate_with_config(spec, out_root, OrchestratorConfig::default())
}

/// Generates an organic sculpt mesh from a spec with custom orchestrator configuration.
pub fn generate_with_config(
    spec: &Spec,
    out_root: &Path,
    config: OrchestratorConfig,
) -> BlenderResult<OrganicSculptResult> {
    // Validate recipe kind
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;
    if recipe.kind != "static_mesh.organic_sculpt_v1" {
        return Err(BlenderError::InvalidRecipeKind {
            kind: recipe.kind.clone(),
        });
    }

    // Parse and validate params (ensures params match expected schema)
    let _params: StaticMeshOrganicSculptV1Params = serde_json::from_value(recipe.params.clone())
        .map_err(BlenderError::DeserializeParamsFailed)?;

    // Serialize spec to JSON
    let spec_json = serde_json::to_string(spec).map_err(BlenderError::SerializeFailed)?;

    // Run orchestrator
    let orchestrator = Orchestrator::with_config(config);
    let report =
        orchestrator.run_with_spec_json(GenerationMode::OrganicSculpt, &spec_json, out_root)?;

    // Get output path from report
    let output_path_str = report
        .output_path
        .as_ref()
        .ok_or_else(|| BlenderError::generation_failed("No output path in report"))?;
    let output_path = out_root.join(output_path_str);

    // Verify output exists
    if !output_path.exists() {
        return Err(BlenderError::OutputNotFound {
            path: output_path.clone(),
        });
    }

    // Get metrics
    let metrics = report
        .metrics
        .clone()
        .ok_or_else(|| BlenderError::generation_failed("No metrics in report"))?;

    Ok(OrganicSculptResult {
        output_path,
        metrics,
        report,
    })
}

/// Generates an organic sculpt mesh directly from params (without full spec).
///
/// This is useful for testing or when you want to bypass spec validation.
pub fn generate_from_params(
    params: &StaticMeshOrganicSculptV1Params,
    asset_id: &str,
    seed: u32,
    out_root: &Path,
) -> BlenderResult<OrganicSculptResult> {
    use speccade_spec::{AssetType, OutputFormat, OutputSpec};

    // Build a minimal spec
    let spec = Spec::builder(asset_id, AssetType::StaticMesh)
        .license("CC0-1.0")
        .seed(seed)
        .output(OutputSpec::primary(
            OutputFormat::Glb,
            format!("meshes/{}.glb", asset_id),
        ))
        .recipe(speccade_spec::recipe::Recipe::new(
            "static_mesh.organic_sculpt_v1",
            serde_json::to_value(params).map_err(BlenderError::SerializeFailed)?,
        ))
        .build();

    generate(&spec, out_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::mesh::{DisplacementNoise, MetaballSource};

    fn create_test_params() -> StaticMeshOrganicSculptV1Params {
        StaticMeshOrganicSculptV1Params {
            metaballs: vec![
                MetaballSource {
                    position: [0.0, 0.0, 0.0],
                    radius: 1.0,
                    stiffness: 2.0,
                },
                MetaballSource {
                    position: [0.5, 0.0, 0.0],
                    radius: 0.8,
                    stiffness: 2.0,
                },
            ],
            remesh_voxel_size: 0.1,
            smooth_iterations: 2,
            displacement: None,
            export: None,
        }
    }

    fn create_test_params_with_displacement() -> StaticMeshOrganicSculptV1Params {
        StaticMeshOrganicSculptV1Params {
            metaballs: vec![MetaballSource {
                position: [0.0, 0.0, 0.0],
                radius: 1.0,
                stiffness: 2.0,
            }],
            remesh_voxel_size: 0.08,
            smooth_iterations: 3,
            displacement: Some(DisplacementNoise {
                strength: 0.1,
                scale: 2.0,
                octaves: 4,
                seed: Some(42),
            }),
            export: None,
        }
    }

    #[test]
    fn test_params_serialization() {
        let params = create_test_params();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("metaballs"));
        assert!(json.contains("remesh_voxel_size"));
        assert!(json.contains("smooth_iterations"));

        let parsed: StaticMeshOrganicSculptV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.metaballs.len(), 2);
        assert!((parsed.remesh_voxel_size - 0.1).abs() < f64::EPSILON);
        assert_eq!(parsed.smooth_iterations, 2);
    }

    #[test]
    fn test_params_with_displacement_serialization() {
        let params = create_test_params_with_displacement();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("displacement"));
        assert!(json.contains("strength"));
        assert!(json.contains("scale"));
        assert!(json.contains("seed"));

        let parsed: StaticMeshOrganicSculptV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.displacement.is_some());
        let disp = parsed.displacement.unwrap();
        assert!((disp.strength - 0.1).abs() < f64::EPSILON);
        assert!((disp.scale - 2.0).abs() < f64::EPSILON);
        assert_eq!(disp.seed, Some(42));
    }
}
