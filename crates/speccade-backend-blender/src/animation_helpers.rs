//! Animation helpers generation handler.
//!
//! This module provides the interface for generating procedural locomotion animations
//! using the `skeletal_animation.helpers_v1` recipe.
//!
//! Animation helpers use preset-based configurations to generate common animation types:
//! - Walk cycles with foot plants and arm swing
//! - Run cycles with faster timing and more dynamic motion
//! - Idle sway with subtle breathing and weight shifting

use std::path::Path;

use speccade_spec::recipe::animation::AnimationHelpersV1Params;
use speccade_spec::Spec;

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::{BlenderMetrics, BlenderReport};
use crate::orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

/// Result of animation helpers generation.
#[derive(Debug, Clone)]
pub struct AnimationHelpersResult {
    /// Path to the generated GLB file.
    pub output_path: std::path::PathBuf,
    /// Path to the generated .blend file (if save_blend was enabled).
    pub blend_path: Option<std::path::PathBuf>,
    /// Metrics from the generation.
    pub metrics: BlenderMetrics,
    /// The Blender report.
    pub report: BlenderReport,
}

/// Generates an animation using helpers from a spec.
///
/// # Arguments
///
/// * `spec` - The SpecCade spec with a `skeletal_animation.helpers_v1` recipe
/// * `out_root` - Root directory for output files
///
/// # Returns
///
/// An `AnimationHelpersResult` containing the path to the generated GLB and metrics.
pub fn generate(spec: &Spec, out_root: &Path) -> BlenderResult<AnimationHelpersResult> {
    generate_with_config(spec, out_root, OrchestratorConfig::default())
}

/// Generates an animation using helpers from a spec with custom orchestrator configuration.
pub fn generate_with_config(
    spec: &Spec,
    out_root: &Path,
    config: OrchestratorConfig,
) -> BlenderResult<AnimationHelpersResult> {
    // Validate recipe kind
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;
    if recipe.kind != "skeletal_animation.helpers_v1" {
        return Err(BlenderError::InvalidRecipeKind {
            kind: recipe.kind.clone(),
        });
    }

    // Parse and validate params
    let _params: AnimationHelpersV1Params = serde_json::from_value(recipe.params.clone())
        .map_err(BlenderError::DeserializeParamsFailed)?;

    // Serialize spec to JSON
    let spec_json = serde_json::to_string(spec).map_err(BlenderError::SerializeFailed)?;

    // Run orchestrator with animation_helpers mode
    let orchestrator = Orchestrator::with_config(config);
    let report =
        orchestrator.run_with_spec_json(GenerationMode::AnimationHelpers, &spec_json, out_root)?;

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

    // Get blend path if it was generated
    let blend_path = report.blend_path.as_ref().map(|p| out_root.join(p));

    Ok(AnimationHelpersResult {
        output_path,
        blend_path,
        metrics,
        report,
    })
}

/// Generates an animation helpers animation directly from params (without full spec).
///
/// This is useful for testing or when you want to bypass spec validation.
pub fn generate_from_params(
    params: &AnimationHelpersV1Params,
    asset_id: &str,
    seed: u32,
    out_root: &Path,
) -> BlenderResult<AnimationHelpersResult> {
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
            "skeletal_animation.helpers_v1",
            serde_json::to_value(params).map_err(BlenderError::SerializeFailed)?,
        ))
        .build();

    generate(&spec, out_root)
}

/// Calculates expected metrics for an animation helpers animation based on params.
///
/// Useful for validation and testing.
pub fn expected_metrics(params: &AnimationHelpersV1Params) -> BlenderMetrics {
    let frame_count = params.settings.cycle_frames;
    let duration_seconds = frame_count as f64 / params.fps as f64;

    // Skeleton type determines bone count (humanoid = ~20-25 bones typically)
    let bone_count = match params.skeleton {
        speccade_spec::recipe::animation::SkeletonType::Humanoid => 20,
        speccade_spec::recipe::animation::SkeletonType::Quadruped => 24,
    };

    BlenderMetrics::for_animation(bone_count, frame_count, duration_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::animation::{AnimationHelperPreset, CycleSettings, SkeletonType};

    fn create_test_params() -> AnimationHelpersV1Params {
        AnimationHelpersV1Params::new(AnimationHelperPreset::WalkCycle)
    }

    #[test]
    fn test_expected_metrics_walk_cycle() {
        let params = create_test_params();
        let metrics = expected_metrics(&params);

        assert_eq!(metrics.bone_count, Some(20)); // Humanoid
        assert_eq!(metrics.animation_frame_count, Some(60)); // Default walk cycle frames
        assert!((metrics.animation_duration_seconds.unwrap() - 2.0).abs() < 0.01); // 60 frames / 30 fps
    }

    #[test]
    fn test_expected_metrics_run_cycle() {
        let params = AnimationHelpersV1Params::run_cycle();
        let metrics = expected_metrics(&params);

        assert_eq!(metrics.bone_count, Some(20)); // Humanoid
        assert_eq!(metrics.animation_frame_count, Some(30)); // Run cycle frames
        assert!((metrics.animation_duration_seconds.unwrap() - 1.0).abs() < 0.01); // 30 frames / 30 fps
    }

    #[test]
    fn test_expected_metrics_quadruped() {
        let mut params = create_test_params();
        params.skeleton = SkeletonType::Quadruped;
        let metrics = expected_metrics(&params);

        assert_eq!(metrics.bone_count, Some(24)); // Quadruped has more bones
    }

    #[test]
    fn test_params_serialization() {
        let params = create_test_params();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("walk_cycle"));
        assert!(json.contains("humanoid"));

        let parsed: AnimationHelpersV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.preset, AnimationHelperPreset::WalkCycle);
        assert_eq!(parsed.skeleton, SkeletonType::Humanoid);
    }

    #[test]
    fn test_cycle_settings_customization() {
        let params = AnimationHelpersV1Params::walk_cycle()
            .with_settings(CycleSettings::new().with_stride_length(1.0).with_cycle_frames(48))
            .with_clip_name("custom_walk")
            .with_fps(24);

        assert_eq!(params.settings.stride_length, 1.0);
        assert_eq!(params.settings.cycle_frames, 48);
        assert_eq!(params.clip_name, "custom_walk");
        assert_eq!(params.fps, 24);
    }
}
