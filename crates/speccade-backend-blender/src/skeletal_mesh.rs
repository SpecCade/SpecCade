//! Skeletal mesh generation handler.
//!
//! This module provides the interface for generating skeletal (rigged) meshes
//! using Blender via the `skeletal_mesh.*` recipes.

use std::path::Path;

use speccade_spec::recipe::character::{
    SkeletalMeshArmatureDrivenV1Params, SkeletalMeshConstraints, SkeletalMeshSkinnedMeshV1Params,
};
use speccade_spec::Spec;

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::{BlenderMetrics, BlenderReport};
use crate::orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

enum SkeletalMeshRecipeParams {
    ArmatureDriven(SkeletalMeshArmatureDrivenV1Params),
    SkinnedMesh(SkeletalMeshSkinnedMeshV1Params),
}

impl SkeletalMeshRecipeParams {
    fn constraints(&self) -> Option<&SkeletalMeshConstraints> {
        match self {
            SkeletalMeshRecipeParams::ArmatureDriven(p) => p.constraints.as_ref(),
            SkeletalMeshRecipeParams::SkinnedMesh(p) => p.constraints.as_ref(),
        }
    }

    fn skeleton_preset(&self) -> Option<&speccade_spec::recipe::character::SkeletonPreset> {
        match self {
            SkeletalMeshRecipeParams::ArmatureDriven(p) => p.skeleton_preset.as_ref(),
            SkeletalMeshRecipeParams::SkinnedMesh(p) => p.skeleton_preset.as_ref(),
        }
    }
}

/// Result of skeletal mesh generation.
#[derive(Debug, Clone)]
pub struct SkeletalMeshResult {
    /// Path to the generated GLB file.
    pub output_path: std::path::PathBuf,
    /// Path to the generated .blend file (if save_blend was enabled).
    pub blend_path: Option<std::path::PathBuf>,
    /// Metrics from the generation.
    pub metrics: BlenderMetrics,
    /// The Blender report.
    pub report: BlenderReport,
}

/// Generates a skeletal mesh from a spec.
///
/// # Arguments
///
/// * `spec` - The SpecCade spec with a skeletal mesh recipe
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
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;

    // Parse and validate params
    let params = match recipe.kind.as_str() {
        "skeletal_mesh.armature_driven_v1" => SkeletalMeshRecipeParams::ArmatureDriven(
            serde_json::from_value(recipe.params.clone())
                .map_err(BlenderError::DeserializeParamsFailed)?,
        ),
        "skeletal_mesh.skinned_mesh_v1" => SkeletalMeshRecipeParams::SkinnedMesh(
            serde_json::from_value(recipe.params.clone())
                .map_err(BlenderError::DeserializeParamsFailed)?,
        ),
        _ => {
            return Err(BlenderError::InvalidRecipeKind {
                kind: recipe.kind.clone(),
            });
        }
    };

    // Serialize spec to JSON
    let spec_json = serde_json::to_string(spec).map_err(BlenderError::SerializeFailed)?;

    // Run orchestrator
    let orchestrator = Orchestrator::with_config(config);
    let report =
        orchestrator.run_with_spec_json(GenerationMode::SkeletalMesh, &spec_json, out_root)?;

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
    if let Some(constraints) = params.constraints() {
        validate_constraints(&metrics, constraints)?;
    }

    // Validate bone count matches skeleton preset
    if let Some(skeleton_preset) = params.skeleton_preset() {
        let expected_bone_count = skeleton_preset.bone_count() as u32;
        if let Some(actual_bone_count) = metrics.bone_count {
            if actual_bone_count != expected_bone_count {
                return Err(BlenderError::metrics_validation_failed(format!(
                    "Bone count {} does not match skeleton preset {:?} (expected {})",
                    actual_bone_count, skeleton_preset, expected_bone_count
                )));
            }
        }
    }

    // Get blend path if it was generated
    let blend_path = report.blend_path.as_ref().map(|p| out_root.join(p));

    Ok(SkeletalMeshResult {
        output_path,
        blend_path,
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
    params: &SkeletalMeshArmatureDrivenV1Params,
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
            "skeletal_mesh.armature_driven_v1",
            serde_json::to_value(params).map_err(BlenderError::SerializeFailed)?,
        ))
        .build();

    generate(&spec, out_root)
}

pub fn generate_from_skinned_mesh_params(
    params: &SkeletalMeshSkinnedMeshV1Params,
    asset_id: &str,
    seed: u32,
    out_root: &Path,
) -> BlenderResult<SkeletalMeshResult> {
    use speccade_spec::{AssetType, OutputFormat, OutputSpec};

    let spec = Spec::builder(asset_id, AssetType::SkeletalMesh)
        .license("CC0-1.0")
        .seed(seed)
        .output(OutputSpec::primary(
            OutputFormat::Glb,
            format!("characters/{}.glb", asset_id),
        ))
        .recipe(speccade_spec::recipe::Recipe::new(
            "skeletal_mesh.skinned_mesh_v1",
            serde_json::to_value(params).map_err(BlenderError::SerializeFailed)?,
        ))
        .build();

    generate(&spec, out_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::character::SkeletonPreset;

    fn create_test_params() -> SkeletalMeshArmatureDrivenV1Params {
        SkeletalMeshArmatureDrivenV1Params {
            skeleton_preset: Some(SkeletonPreset::HumanoidBasicV1),
            skeleton: vec![],
            skinning_mode: None,
            bone_meshes: std::collections::HashMap::new(),
            bool_shapes: std::collections::HashMap::new(),
            material_slots: vec![],
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

        let parsed: SkeletalMeshArmatureDrivenV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.skeleton_preset,
            Some(SkeletonPreset::HumanoidBasicV1)
        );
        assert!(parsed.bone_meshes.is_empty());
    }

    #[test]
    fn test_skinned_mesh_params_serialization() {
        use speccade_spec::recipe::character::{SkinnedMeshBindingConfig, SkinnedMeshBindingMode};

        let params = SkeletalMeshSkinnedMeshV1Params {
            mesh_file: Some("./mesh.glb".to_string()),
            mesh_asset: None,
            skeleton_preset: Some(SkeletonPreset::HumanoidBasicV1),
            skeleton: vec![],
            binding: SkinnedMeshBindingConfig {
                mode: SkinnedMeshBindingMode::AutoWeights,
                vertex_group_map: std::collections::HashMap::new(),
                max_bone_influences: 4,
            },
            material_slots: vec![],
            export: None,
            constraints: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("mesh.glb"));
        assert!(json.contains("auto_weights"));

        let parsed: SkeletalMeshSkinnedMeshV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mesh_file.as_deref(), Some("./mesh.glb"));
        assert_eq!(parsed.binding.mode, SkinnedMeshBindingMode::AutoWeights);
    }

    #[test]
    fn test_skeleton_preset_bone_count() {
        let preset = SkeletonPreset::HumanoidBasicV1;
        assert_eq!(preset.bone_count(), 20);
    }
}
