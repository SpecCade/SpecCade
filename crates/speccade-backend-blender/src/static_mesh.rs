//! Static mesh generation handler.
//!
//! This module provides the interface for generating static meshes using
//! the `static_mesh.blender_primitives_v1` recipe.

use std::path::Path;

use speccade_spec::recipe::mesh::{MeshConstraints, StaticMeshBlenderPrimitivesV1Params};
use speccade_spec::Spec;

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::{BlenderMetrics, BlenderReport};
use crate::orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

/// Result of static mesh generation.
#[derive(Debug, Clone)]
pub struct StaticMeshResult {
    /// Path to the generated GLB file.
    pub output_path: std::path::PathBuf,
    /// Metrics from the generation.
    pub metrics: BlenderMetrics,
    /// The Blender report.
    pub report: BlenderReport,
}

/// Generates a static mesh from a spec.
///
/// # Arguments
///
/// * `spec` - The SpecCade spec with a `static_mesh.blender_primitives_v1` recipe
/// * `out_root` - Root directory for output files
///
/// # Returns
///
/// A `StaticMeshResult` containing the path to the generated GLB and metrics.
pub fn generate(spec: &Spec, out_root: &Path) -> BlenderResult<StaticMeshResult> {
    generate_with_config(spec, out_root, OrchestratorConfig::default())
}

/// Generates a static mesh from a spec with custom orchestrator configuration.
pub fn generate_with_config(
    spec: &Spec,
    out_root: &Path,
    config: OrchestratorConfig,
) -> BlenderResult<StaticMeshResult> {
    // Validate recipe kind
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;
    if recipe.kind != "static_mesh.blender_primitives_v1" {
        return Err(BlenderError::InvalidRecipeKind {
            kind: recipe.kind.clone(),
        });
    }

    // Parse and validate params
    let params: StaticMeshBlenderPrimitivesV1Params = serde_json::from_value(recipe.params.clone())
        .map_err(BlenderError::DeserializeParamsFailed)?;

    // Serialize spec to JSON
    let spec_json = serde_json::to_string(spec).map_err(BlenderError::SerializeFailed)?;

    // Run orchestrator
    let orchestrator = Orchestrator::with_config(config);
    let report =
        orchestrator.run_with_spec_json(GenerationMode::StaticMesh, &spec_json, out_root)?;

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

    // Validate constraints if specified
    if let Some(ref constraints) = params.constraints {
        validate_constraints(&metrics, constraints)?;
    }

    Ok(StaticMeshResult {
        output_path,
        metrics,
        report,
    })
}

/// Validates metrics against mesh constraints.
fn validate_constraints(
    metrics: &BlenderMetrics,
    constraints: &MeshConstraints,
) -> BlenderResult<()> {
    if let (Some(max), Some(actual)) = (constraints.max_triangles, metrics.triangle_count) {
        if actual > max {
            return Err(BlenderError::constraint_violation(format!(
                "Triangle count {} exceeds maximum {}",
                actual, max
            )));
        }
    }

    if let (Some(max), Some(actual)) = (constraints.max_materials, metrics.material_slot_count) {
        if actual > max {
            return Err(BlenderError::constraint_violation(format!(
                "Material count {} exceeds maximum {}",
                actual, max
            )));
        }
    }

    if let (Some(max), Some(actual)) = (constraints.max_vertices, metrics.vertex_count) {
        if actual > max {
            return Err(BlenderError::constraint_violation(format!(
                "Vertex count {} exceeds maximum {}",
                actual, max
            )));
        }
    }

    Ok(())
}

/// Generates a static mesh directly from params (without full spec).
///
/// This is useful for testing or when you want to bypass spec validation.
pub fn generate_from_params(
    params: &StaticMeshBlenderPrimitivesV1Params,
    asset_id: &str,
    seed: u32,
    out_root: &Path,
) -> BlenderResult<StaticMeshResult> {
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
            "static_mesh.blender_primitives_v1",
            serde_json::to_value(params).map_err(BlenderError::SerializeFailed)?,
        ))
        .build();

    generate(&spec, out_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::mesh::{MeshPrimitive, UvProjection, UvProjectionMethod};

    fn create_test_params() -> StaticMeshBlenderPrimitivesV1Params {
        StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Cube,
            dimensions: [1.0, 1.0, 1.0],
            modifiers: vec![],
            uv_projection: Some(UvProjection::Simple(UvProjectionMethod::Box)),
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: Some(MeshConstraints {
                max_triangles: Some(1000),
                max_materials: Some(4),
                max_vertices: None,
            }),
            lod_chain: None,
            collision_mesh: None,
            navmesh: None,
        }
    }

    #[test]
    fn test_validate_constraints_pass() {
        let metrics = BlenderMetrics::for_static_mesh(
            100,
            crate::metrics::BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 1.0, 1.0]),
            4,
            2,
        );

        let constraints = MeshConstraints {
            max_triangles: Some(500),
            max_materials: Some(4),
            max_vertices: None,
        };

        assert!(validate_constraints(&metrics, &constraints).is_ok());
    }

    #[test]
    fn test_validate_constraints_fail_triangles() {
        let metrics = BlenderMetrics::for_static_mesh(
            1000,
            crate::metrics::BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 1.0, 1.0]),
            4,
            2,
        );

        let constraints = MeshConstraints {
            max_triangles: Some(500),
            max_materials: None,
            max_vertices: None,
        };

        let result = validate_constraints(&metrics, &constraints);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Triangle count"));
    }

    #[test]
    fn test_validate_constraints_fail_materials() {
        let metrics = BlenderMetrics::for_static_mesh(
            100,
            crate::metrics::BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 1.0, 1.0]),
            4,
            10,
        );

        let constraints = MeshConstraints {
            max_triangles: None,
            max_materials: Some(4),
            max_vertices: None,
        };

        let result = validate_constraints(&metrics, &constraints);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Material count"));
    }

    #[test]
    fn test_params_serialization() {
        let params = create_test_params();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("cube"));
        assert!(json.contains("dimensions"));

        let parsed: StaticMeshBlenderPrimitivesV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.base_primitive, MeshPrimitive::Cube);
    }
}
