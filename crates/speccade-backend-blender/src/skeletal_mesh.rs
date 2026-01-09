//! Skeletal mesh generation handler.
//!
//! This module provides the interface for generating skeletal (rigged) meshes
//! using the `skeletal_mesh.blender_rigged_mesh_v1` recipe.

use std::path::Path;

use speccade_spec::recipe::character::{SkeletalMeshBlenderRiggedMeshV1Params, SkeletalMeshConstraints};
use speccade_spec::Spec;

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::{BlenderMetrics, BlenderReport};
use crate::orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

/// Result of skeletal mesh generation.
#[derive(Debug, Clone)]
pub struct SkeletalMeshResult {
    /// Path to the generated GLB file.
    pub output_path: std::path::PathBuf,
    /// Metrics from the generation.
    pub metrics: BlenderMetrics,
    /// The Blender report.
    pub report: BlenderReport,
}

/// Generates a skeletal mesh from a spec.
///
/// # Arguments
///
/// * `spec` - The SpecCade spec with a `skeletal_mesh.blender_rigged_mesh_v1` recipe
/// * `out_root` - Root directory for output files
///
/// # Returns
///
/// A `SkeletalMeshResult` containing the path to the generated GLB and metrics.
pub fn generate(spec: &Spec, out_root: &Path) -> BlenderResult<SkeletalMeshResult> {
    generate_with_config(spec, out_root, OrchestratorConfig::default())
}

/// Generates a skeletal mesh from a spec with custom orchestrator configuration.
pub fn generate_with_config(
    spec: &Spec,
    out_root: &Path,
    config: OrchestratorConfig,
) -> BlenderResult<SkeletalMeshResult> {
    // Validate recipe kind
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;
    if recipe.kind != "skeletal_mesh.blender_rigged_mesh_v1" {
        return Err(BlenderError::InvalidRecipeKind {
            kind: recipe.kind.clone(),
        });
    }

    // Parse and validate params
    let params: SkeletalMeshBlenderRiggedMeshV1Params =
        serde_json::from_value(recipe.params.clone())
            .map_err(BlenderError::DeserializeParamsFailed)?;

    // Serialize spec to JSON
    let spec_json = serde_json::to_string(spec).map_err(BlenderError::SerializeFailed)?;

    // Run orchestrator
    let orchestrator = Orchestrator::with_config(config);
    let report = orchestrator.run_with_spec_json(GenerationMode::SkeletalMesh, &spec_json, out_root)?;

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

    // Validate bone count matches skeleton preset
    let expected_bone_count = params.skeleton_preset.bone_count() as u32;
    if let Some(actual_bone_count) = metrics.bone_count {
        if actual_bone_count != expected_bone_count {
            return Err(BlenderError::metrics_validation_failed(format!(
                "Bone count {} does not match skeleton preset {:?} (expected {})",
                actual_bone_count, params.skeleton_preset, expected_bone_count
            )));
        }
    }

    Ok(SkeletalMeshResult {
        output_path,
        metrics,
        report,
    })
}

/// Validates metrics against skeletal mesh constraints.
fn validate_constraints(
    metrics: &BlenderMetrics,
    constraints: &SkeletalMeshConstraints,
) -> BlenderResult<()> {
    if let (Some(max), Some(actual)) = (constraints.max_triangles, metrics.triangle_count) {
        if actual > max {
            return Err(BlenderError::constraint_violation(format!(
                "Triangle count {} exceeds maximum {}",
                actual, max
            )));
        }
    }

    if let (Some(max), Some(actual)) = (constraints.max_bones, metrics.bone_count) {
        if actual > max {
            return Err(BlenderError::constraint_violation(format!(
                "Bone count {} exceeds maximum {}",
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

    Ok(())
}

/// Generates a skeletal mesh directly from params (without full spec).
///
/// This is useful for testing or when you want to bypass spec validation.
pub fn generate_from_params(
    params: &SkeletalMeshBlenderRiggedMeshV1Params,
    asset_id: &str,
    seed: u32,
    out_root: &Path,
) -> BlenderResult<SkeletalMeshResult> {
    use speccade_spec::{AssetType, OutputFormat, OutputSpec};

    // Build a minimal spec
    let spec = Spec::builder(asset_id, AssetType::SkeletalMesh)
        .license("CC0-1.0")
        .seed(seed)
        .output(OutputSpec::primary(
            OutputFormat::Glb,
            format!("characters/{}.glb", asset_id),
        ))
        .recipe(speccade_spec::recipe::Recipe::new(
            "skeletal_mesh.blender_rigged_mesh_v1",
            serde_json::to_value(params).map_err(BlenderError::SerializeFailed)?,
        ))
        .build();

    generate(&spec, out_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::character::{BodyPart, BodyPartMesh, SkeletonPreset};
    use speccade_spec::recipe::mesh::MeshPrimitive;

    fn create_test_params() -> SkeletalMeshBlenderRiggedMeshV1Params {
        SkeletalMeshBlenderRiggedMeshV1Params {
            skeleton_preset: SkeletonPreset::HumanoidBasicV1,
            body_parts: vec![BodyPart {
                bone: "spine".to_string(),
                mesh: BodyPartMesh {
                    primitive: MeshPrimitive::Cylinder,
                    dimensions: [0.3, 0.5, 0.3],
                    segments: Some(8),
                    offset: None,
                    rotation: None,
                },
                material_index: Some(0),
            }],
            material_slots: vec![],
            skinning: None,
            export: None,
            constraints: Some(SkeletalMeshConstraints {
                max_triangles: Some(5000),
                max_bones: Some(64),
                max_materials: Some(4),
            }),
        }
    }

    #[test]
    fn test_validate_constraints_pass() {
        let metrics = BlenderMetrics::for_skeletal_mesh(
            1000,
            crate::metrics::BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 2.0, 1.0]),
            20,
            2,
            4,
        );

        let constraints = SkeletalMeshConstraints {
            max_triangles: Some(5000),
            max_bones: Some(64),
            max_materials: Some(4),
        };

        assert!(validate_constraints(&metrics, &constraints).is_ok());
    }

    #[test]
    fn test_validate_constraints_fail_bones() {
        let metrics = BlenderMetrics::for_skeletal_mesh(
            1000,
            crate::metrics::BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 2.0, 1.0]),
            100,
            2,
            4,
        );

        let constraints = SkeletalMeshConstraints {
            max_triangles: None,
            max_bones: Some(64),
            max_materials: None,
        };

        let result = validate_constraints(&metrics, &constraints);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Bone count"));
    }

    #[test]
    fn test_params_serialization() {
        let params = create_test_params();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("humanoid_basic_v1"));
        assert!(json.contains("spine"));
        assert!(json.contains("cylinder"));

        let parsed: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.skeleton_preset, SkeletonPreset::HumanoidBasicV1);
        assert_eq!(parsed.body_parts.len(), 1);
    }

    #[test]
    fn test_skeleton_preset_bone_count() {
        let preset = SkeletonPreset::HumanoidBasicV1;
        assert_eq!(preset.bone_count(), 20);
    }
}
