//! Rigged animation generation handler.
//!
//! This module provides the interface for generating IK/rig-aware skeletal animations
//! using the `skeletal_animation.blender_rigged_v1` recipe.

use std::path::Path;

use speccade_spec::recipe::animation::SkeletalAnimationBlenderRiggedV1Params;
use speccade_spec::Spec;

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::{BlenderMetrics, BlenderReport, MetricTolerances};
use crate::orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

/// Result of rigged animation generation.
#[derive(Debug, Clone)]
pub struct RiggedAnimationResult {
    /// Path to the generated GLB file.
    pub output_path: std::path::PathBuf,
    /// Path to the generated .blend file (if save_blend was enabled).
    pub blend_path: Option<std::path::PathBuf>,
    /// Metrics from the generation.
    pub metrics: BlenderMetrics,
    /// The Blender report.
    pub report: BlenderReport,
}

/// Generates a rigged animation from a spec.
///
/// # Arguments
///
/// * `spec` - The SpecCade spec with a `skeletal_animation.blender_rigged_v1` recipe
/// * `out_root` - Root directory for output files
///
/// # Returns
///
/// A `RiggedAnimationResult` containing the path to the generated GLB and metrics.
pub fn generate(spec: &Spec, out_root: &Path) -> BlenderResult<RiggedAnimationResult> {
    generate_with_config(spec, out_root, OrchestratorConfig::default())
}

/// Generates a rigged animation from a spec with custom orchestrator configuration.
pub fn generate_with_config(
    spec: &Spec,
    out_root: &Path,
    config: OrchestratorConfig,
) -> BlenderResult<RiggedAnimationResult> {
    // Validate recipe kind
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;
    if recipe.kind != "skeletal_animation.blender_rigged_v1" {
        return Err(BlenderError::InvalidRecipeKind {
            kind: recipe.kind.clone(),
        });
    }

    // Parse and validate params
    let params: SkeletalAnimationBlenderRiggedV1Params =
        serde_json::from_value(recipe.params.clone())
            .map_err(BlenderError::DeserializeParamsFailed)?;

    // Serialize spec to JSON
    let spec_json = serde_json::to_string(spec).map_err(BlenderError::SerializeFailed)?;

    // Run orchestrator
    let orchestrator = Orchestrator::with_config(config);
    let report =
        orchestrator.run_with_spec_json(GenerationMode::RiggedAnimation, &spec_json, out_root)?;

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

    // Validate rigged animation metrics
    validate_rigged_animation_metrics(&metrics, &params)?;

    // Get blend path if it was generated
    let blend_path = report.blend_path.as_ref().map(|p| out_root.join(p));

    Ok(RiggedAnimationResult {
        output_path,
        blend_path,
        metrics,
        report,
    })
}

/// Validates rigged animation metrics against expected values from params.
fn validate_rigged_animation_metrics(
    metrics: &BlenderMetrics,
    params: &SkeletalAnimationBlenderRiggedV1Params,
) -> BlenderResult<()> {
    let tolerances = MetricTolerances::default();

    // Calculate expected duration in seconds
    let expected_duration_seconds = params.duration_seconds.unwrap_or_else(|| {
        params.duration_frames as f64 / params.fps as f64
    });

    // Calculate expected frame count
    let expected_frame_count = (expected_duration_seconds * params.fps as f64).ceil() as u32;

    // Validate frame count (exact match required)
    if let Some(actual_frame_count) = metrics.animation_frame_count {
        if actual_frame_count != expected_frame_count {
            return Err(BlenderError::metrics_validation_failed(format!(
                "Animation frame count {} does not match expected {} (duration: {}s, fps: {})",
                actual_frame_count, expected_frame_count, expected_duration_seconds, params.fps
            )));
        }
    }

    // Validate duration (tolerance allowed)
    if let Some(actual_duration) = metrics.animation_duration_seconds {
        if (actual_duration - expected_duration_seconds).abs() > tolerances.animation_duration {
            return Err(BlenderError::metrics_validation_failed(format!(
                "Animation duration {:.3}s does not match expected {:.3}s (tolerance: {:.3}s)",
                actual_duration, expected_duration_seconds, tolerances.animation_duration
            )));
        }
    }

    // Validate bone count matches skeleton preset (if provided)
    if let Some(skeleton_preset) = &params.skeleton_preset {
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

    Ok(())
}

/// Generates a rigged animation directly from params (without full spec).
///
/// This is useful for testing or when you want to bypass spec validation.
pub fn generate_from_params(
    params: &SkeletalAnimationBlenderRiggedV1Params,
    asset_id: &str,
    seed: u32,
    out_root: &Path,
) -> BlenderResult<RiggedAnimationResult> {
    use speccade_spec::{AssetType, OutputFormat, OutputSpec};

    // Build a minimal spec
    let spec = Spec::builder(asset_id, AssetType::SkeletalAnimation)
        .license("CC0-1.0")
        .seed(seed)
        .output(OutputSpec::primary(
            OutputFormat::Glb,
            format!("animations/{}.glb", asset_id),
        ))
        .recipe(speccade_spec::recipe::Recipe::new(
            "skeletal_animation.blender_rigged_v1",
            serde_json::to_value(params).map_err(BlenderError::SerializeFailed)?,
        ))
        .build();

    generate(&spec, out_root)
}

/// Calculates expected metrics for a rigged animation based on params.
///
/// Useful for validation and testing.
pub fn expected_metrics(params: &SkeletalAnimationBlenderRiggedV1Params) -> BlenderMetrics {
    let duration_seconds = params.duration_seconds.unwrap_or_else(|| {
        params.duration_frames as f64 / params.fps as f64
    });
    let frame_count = (duration_seconds * params.fps as f64).ceil() as u32;
    let bone_count = params.skeleton_preset.as_ref().map(|p| p.bone_count() as u32);

    BlenderMetrics::for_animation(
        bone_count.unwrap_or(0),
        frame_count,
        duration_seconds,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::character::SkeletonPreset;

    fn create_test_params() -> SkeletalAnimationBlenderRiggedV1Params {
        SkeletalAnimationBlenderRiggedV1Params {
            skeleton_preset: Some(SkeletonPreset::HumanoidBasicV1),
            clip_name: "walk".to_string(),
            input_armature: None,
            character: None,
            duration_frames: 30,
            duration_seconds: Some(1.0),
            fps: 30,
            r#loop: true,
            ground_offset: 0.0,
            rig_setup: Default::default(),
            poses: Default::default(),
            phases: vec![],
            procedural_layers: vec![],
            keyframes: vec![],
            ik_keyframes: vec![],
            interpolation: Default::default(),
            export: None,
            animator_rig: None,
            save_blend: false,
            conventions: None,
        }
    }

    #[test]
    fn test_expected_metrics() {
        let params = create_test_params();
        let metrics = expected_metrics(&params);

        assert_eq!(metrics.bone_count, Some(20)); // HumanoidBasicV1 has 20 bones
        assert_eq!(metrics.animation_frame_count, Some(30)); // 1.0s * 30fps = 30 frames
        assert_eq!(metrics.animation_duration_seconds, Some(1.0));
    }

    #[test]
    fn test_expected_metrics_from_frames() {
        let mut params = create_test_params();
        params.duration_seconds = None;
        params.duration_frames = 60;
        params.fps = 30;

        let metrics = expected_metrics(&params);

        assert_eq!(metrics.animation_frame_count, Some(60)); // 60 frames
        assert_eq!(metrics.animation_duration_seconds, Some(2.0)); // 60 frames / 30 fps = 2.0s
    }

    #[test]
    fn test_validate_rigged_animation_metrics_pass() {
        let params = create_test_params();
        let metrics = BlenderMetrics::for_animation(20, 30, 1.0);

        assert!(validate_rigged_animation_metrics(&metrics, &params).is_ok());
    }

    #[test]
    fn test_validate_rigged_animation_metrics_fail_frame_count() {
        let params = create_test_params();
        let metrics = BlenderMetrics::for_animation(20, 60, 1.0); // Wrong frame count

        let result = validate_rigged_animation_metrics(&metrics, &params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("frame count"));
    }

    #[test]
    fn test_validate_rigged_animation_metrics_fail_duration() {
        let params = create_test_params();
        let metrics = BlenderMetrics::for_animation(20, 30, 2.0); // Wrong duration

        let result = validate_rigged_animation_metrics(&metrics, &params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("duration"));
    }

    #[test]
    fn test_validate_rigged_animation_metrics_fail_bone_count() {
        let params = create_test_params();
        let metrics = BlenderMetrics::for_animation(10, 30, 1.0); // Wrong bone count

        let result = validate_rigged_animation_metrics(&metrics, &params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Bone count"));
    }

    #[test]
    fn test_validate_rigged_animation_metrics_no_skeleton_preset() {
        let mut params = create_test_params();
        params.skeleton_preset = None;
        let metrics = BlenderMetrics::for_animation(10, 30, 1.0); // Any bone count should pass

        // Should pass because no skeleton preset means no bone count validation
        assert!(validate_rigged_animation_metrics(&metrics, &params).is_ok());
    }

    #[test]
    fn test_params_serialization() {
        let params = create_test_params();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("humanoid_basic_v1"));
        assert!(json.contains("walk"));

        let parsed: SkeletalAnimationBlenderRiggedV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.clip_name, "walk");
        assert_eq!(parsed.fps, 30);
        assert!(parsed.r#loop);
    }
}
