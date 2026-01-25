//! Modular kit mesh generation handler.
//!
//! This module provides the interface for generating modular kit meshes
//! (walls, pipes, doors) using the `static_mesh.modular_kit_v1` recipe.

use std::path::Path;

use speccade_spec::recipe::mesh::StaticMeshModularKitV1Params;
use speccade_spec::Spec;

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::{BlenderMetrics, BlenderReport};
use crate::orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

/// Result of modular kit mesh generation.
#[derive(Debug, Clone)]
pub struct ModularKitResult {
    /// Path to the generated GLB file.
    pub output_path: std::path::PathBuf,
    /// Metrics from the generation.
    pub metrics: BlenderMetrics,
    /// The Blender report.
    pub report: BlenderReport,
}

/// Generates a modular kit mesh from a spec.
///
/// # Arguments
///
/// * `spec` - The SpecCade spec with a `static_mesh.modular_kit_v1` recipe
/// * `out_root` - Root directory for output files
///
/// # Returns
///
/// A `ModularKitResult` containing the path to the generated GLB and metrics.
pub fn generate(spec: &Spec, out_root: &Path) -> BlenderResult<ModularKitResult> {
    generate_with_config(spec, out_root, OrchestratorConfig::default())
}

/// Generates a modular kit mesh from a spec with custom orchestrator configuration.
pub fn generate_with_config(
    spec: &Spec,
    out_root: &Path,
    config: OrchestratorConfig,
) -> BlenderResult<ModularKitResult> {
    // Validate recipe kind
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;
    if recipe.kind != "static_mesh.modular_kit_v1" {
        return Err(BlenderError::InvalidRecipeKind {
            kind: recipe.kind.clone(),
        });
    }

    // Parse and validate params (ensures params match expected schema)
    let _params: StaticMeshModularKitV1Params = serde_json::from_value(recipe.params.clone())
        .map_err(BlenderError::DeserializeParamsFailed)?;

    // Serialize spec to JSON
    let spec_json = serde_json::to_string(spec).map_err(BlenderError::SerializeFailed)?;

    // Run orchestrator
    let orchestrator = Orchestrator::with_config(config);
    let report =
        orchestrator.run_with_spec_json(GenerationMode::ModularKit, &spec_json, out_root)?;

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

    Ok(ModularKitResult {
        output_path,
        metrics,
        report,
    })
}

/// Generates a modular kit mesh directly from params (without full spec).
///
/// This is useful for testing or when you want to bypass spec validation.
pub fn generate_from_params(
    params: &StaticMeshModularKitV1Params,
    asset_id: &str,
    seed: u32,
    out_root: &Path,
) -> BlenderResult<ModularKitResult> {
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
            "static_mesh.modular_kit_v1",
            serde_json::to_value(params).map_err(BlenderError::SerializeFailed)?,
        ))
        .build();

    generate(&spec, out_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::mesh::{ModularKitType, PipeKitParams, PipeSegment, WallKitParams};

    fn create_test_wall_params() -> StaticMeshModularKitV1Params {
        StaticMeshModularKitV1Params {
            kit_type: ModularKitType::Wall(WallKitParams {
                width: 3.0,
                height: 2.5,
                thickness: 0.15,
                cutouts: vec![],
                has_baseboard: false,
                has_crown: false,
                baseboard_height: 0.1,
                crown_height: 0.08,
                bevel_width: 0.0,
            }),
            export: None,
        }
    }

    fn create_test_pipe_params() -> StaticMeshModularKitV1Params {
        StaticMeshModularKitV1Params {
            kit_type: ModularKitType::Pipe(PipeKitParams {
                diameter: 0.1,
                wall_thickness: 0.02,
                segments: vec![PipeSegment::Straight { length: 1.0 }],
                vertices: 16,
                bevel_width: 0.0,
            }),
            export: None,
        }
    }

    #[test]
    fn test_wall_params_serialization() {
        let params = create_test_wall_params();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("wall"));
        assert!(json.contains("width"));
        assert!(json.contains("height"));
        assert!(json.contains("thickness"));

        let parsed: StaticMeshModularKitV1Params = serde_json::from_str(&json).unwrap();
        match parsed.kit_type {
            ModularKitType::Wall(wall) => {
                assert!((wall.width - 3.0).abs() < f64::EPSILON);
                assert!((wall.height - 2.5).abs() < f64::EPSILON);
            }
            _ => panic!("Expected Wall kit type"),
        }
    }

    #[test]
    fn test_pipe_params_serialization() {
        let params = create_test_pipe_params();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("pipe"));
        assert!(json.contains("diameter"));
        assert!(json.contains("segments"));

        let parsed: StaticMeshModularKitV1Params = serde_json::from_str(&json).unwrap();
        match parsed.kit_type {
            ModularKitType::Pipe(pipe) => {
                assert!((pipe.diameter - 0.1).abs() < f64::EPSILON);
                assert_eq!(pipe.segments.len(), 1);
            }
            _ => panic!("Expected Pipe kit type"),
        }
    }
}
