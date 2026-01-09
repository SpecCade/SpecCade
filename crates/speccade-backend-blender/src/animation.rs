//! Skeletal animation generation handler.
//!
//! This module provides the interface for generating skeletal animations
//! using the `skeletal_animation.blender_clip_v1` recipe.

use std::path::Path;

use speccade_spec::recipe::animation::SkeletalAnimationBlenderClipV1Params;
use speccade_spec::Spec;

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::{BlenderMetrics, BlenderReport, MetricTolerances};
use crate::orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

/// Result of animation generation.
#[derive(Debug, Clone)]
pub struct AnimationResult {
    /// Path to the generated GLB file.
    pub output_path: std::path::PathBuf,
    /// Metrics from the generation.
    pub metrics: BlenderMetrics,
    /// The Blender report.
    pub report: BlenderReport,
}

/// Generates an animation from a spec.
///
/// # Arguments
///
/// * `spec` - The SpecCade spec with a `skeletal_animation.blender_clip_v1` recipe
/// * `out_root` - Root directory for output files
///
/// # Returns
///
/// An `AnimationResult` containing the path to the generated GLB and metrics.
pub fn generate(spec: &Spec, out_root: &Path) -> BlenderResult<AnimationResult> {
    generate_with_config(spec, out_root, OrchestratorConfig::default())
}

/// Generates an animation from a spec with custom orchestrator configuration.
pub fn generate_with_config(
    spec: &Spec,
    out_root: &Path,
    config: OrchestratorConfig,
) -> BlenderResult<AnimationResult> {
    // Validate recipe kind
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;
    if recipe.kind != "skeletal_animation.blender_clip_v1" {
        return Err(BlenderError::InvalidRecipeKind {
            kind: recipe.kind.clone(),
        });
    }

    // Parse and validate params
    let params: SkeletalAnimationBlenderClipV1Params =
        serde_json::from_value(recipe.params.clone())
            .map_err(BlenderError::DeserializeParamsFailed)?;

    // Serialize spec to JSON
    let spec_json = serde_json::to_string(spec).map_err(BlenderError::SerializeFailed)?;

    // Run orchestrator
    let orchestrator = Orchestrator::with_config(config);
    let report = orchestrator.run_with_spec_json(GenerationMode::Animation, &spec_json, out_root)?;

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

    // Validate animation metrics
    validate_animation_metrics(&metrics, &params)?;

    Ok(AnimationResult {
        output_path,
        metrics,
        report,
    })
}

/// Validates animation metrics against expected values from params.
fn validate_animation_metrics(
    metrics: &BlenderMetrics,
    params: &SkeletalAnimationBlenderClipV1Params,
) -> BlenderResult<()> {
    let tolerances = MetricTolerances::default();

    // Calculate expected frame count
    let expected_frame_count = (params.duration_seconds * params.fps as f64).ceil() as u32;

    // Validate frame count (exact match required)
    if let Some(actual_frame_count) = metrics.animation_frame_count {
        if actual_frame_count != expected_frame_count {
            return Err(BlenderError::metrics_validation_failed(format!(
                "Animation frame count {} does not match expected {} (duration: {}s, fps: {})",
                actual_frame_count, expected_frame_count, params.duration_seconds, params.fps
            )));
        }
    }

    // Validate duration (tolerance allowed)
    if let Some(actual_duration) = metrics.animation_duration_seconds {
        if (actual_duration - params.duration_seconds).abs() > tolerances.animation_duration {
            return Err(BlenderError::metrics_validation_failed(format!(
                "Animation duration {:.3}s does not match expected {:.3}s (tolerance: {:.3}s)",
                actual_duration, params.duration_seconds, tolerances.animation_duration
            )));
        }
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

    Ok(())
}

/// Generates an animation directly from params (without full spec).
///
/// This is useful for testing or when you want to bypass spec validation.
pub fn generate_from_params(
    params: &SkeletalAnimationBlenderClipV1Params,
    asset_id: &str,
    seed: u32,
    out_root: &Path,
) -> BlenderResult<AnimationResult> {
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
            "skeletal_animation.blender_clip_v1",
            serde_json::to_value(params).map_err(BlenderError::SerializeFailed)?,
        ))
        .build();

    generate(&spec, out_root)
}

/// Calculates expected metrics for an animation based on params.
///
/// Useful for validation and testing.
pub fn expected_metrics(params: &SkeletalAnimationBlenderClipV1Params) -> BlenderMetrics {
    let frame_count = (params.duration_seconds * params.fps as f64).ceil() as u32;
    let bone_count = params.skeleton_preset.bone_count() as u32;

    BlenderMetrics::for_animation(bone_count, frame_count, params.duration_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::animation::{AnimationKeyframe, BoneTransform, InterpolationMode};
    use speccade_spec::recipe::character::SkeletonPreset;
    use std::collections::HashMap;

    fn create_test_params() -> SkeletalAnimationBlenderClipV1Params {
        let mut bones = HashMap::new();
        bones.insert(
            "upper_leg_l".to_string(),
            BoneTransform::with_rotation([15.0, 0.0, 0.0]),
        );

        SkeletalAnimationBlenderClipV1Params {
            skeleton_preset: SkeletonPreset::HumanoidBasicV1,
            clip_name: "walk".to_string(),
            duration_seconds: 1.0,
            fps: 30,
            r#loop: true,
            keyframes: vec![
                AnimationKeyframe {
                    time: 0.0,
                    bones: bones.clone(),
                },
                AnimationKeyframe {
                    time: 1.0,
                    bones: bones,
                },
            ],
            interpolation: InterpolationMode::Linear,
            export: None,
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
    fn test_validate_animation_metrics_pass() {
        let params = create_test_params();
        let metrics = BlenderMetrics::for_animation(20, 30, 1.0);

        assert!(validate_animation_metrics(&metrics, &params).is_ok());
    }

    #[test]
    fn test_validate_animation_metrics_fail_frame_count() {
        let params = create_test_params();
        let metrics = BlenderMetrics::for_animation(20, 60, 1.0); // Wrong frame count

        let result = validate_animation_metrics(&metrics, &params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("frame count"));
    }

    #[test]
    fn test_validate_animation_metrics_fail_duration() {
        let params = create_test_params();
        let metrics = BlenderMetrics::for_animation(20, 30, 2.0); // Wrong duration

        let result = validate_animation_metrics(&metrics, &params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("duration"));
    }

    #[test]
    fn test_validate_animation_metrics_fail_bone_count() {
        let params = create_test_params();
        let metrics = BlenderMetrics::for_animation(10, 30, 1.0); // Wrong bone count

        let result = validate_animation_metrics(&metrics, &params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Bone count"));
    }

    #[test]
    fn test_params_serialization() {
        let params = create_test_params();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("humanoid_basic_v1"));
        assert!(json.contains("walk"));
        assert!(json.contains("linear"));

        let parsed: SkeletalAnimationBlenderClipV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.clip_name, "walk");
        assert_eq!(parsed.fps, 30);
        assert!(parsed.r#loop);
    }
}
