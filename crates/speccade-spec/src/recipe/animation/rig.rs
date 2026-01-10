//! Rigged animation recipe parameters.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::animator_rig::AnimatorRigConfig;
use super::clip::{AnimationKeyframe, IkKeyframe};
use super::common::{AnimationExportSettings, ConventionsConfig, InterpolationMode};
use super::pose::{AnimationPhase, PoseDefinition};
use super::procedural::ProceduralLayer;
use super::skeletal::RigSetup;
use crate::recipe::character::SkeletonPreset;

// =============================================================================
// Rigged Animation Recipe (v2 with IK support)
// =============================================================================

fn default_fps() -> u8 {
    30
}

/// Parameters for the `skeletal_animation.blender_rigged_v1` recipe.
/// This is the IK-enabled version of the animation recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletalAnimationBlenderRiggedV1Params {
    /// Skeleton rig to animate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton_preset: Option<SkeletonPreset>,
    /// Name of the animation clip.
    pub clip_name: String,
    /// Path to existing armature file (GLB/GLTF).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_armature: Option<String>,
    /// Character spec reference (alternative to input_armature).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    /// Duration of the animation in frames.
    pub duration_frames: u32,
    /// Duration of the animation in seconds (alternative to duration_frames).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    /// Frames per second.
    #[serde(default = "default_fps")]
    pub fps: u8,
    /// Whether the animation should loop.
    #[serde(default)]
    pub r#loop: bool,
    /// Root bone Z offset (for ground contact).
    #[serde(default)]
    pub ground_offset: f64,
    /// Rig setup configuration (IK chains, presets).
    #[serde(default)]
    pub rig_setup: RigSetup,
    /// Named pose definitions.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub poses: HashMap<String, PoseDefinition>,
    /// Animation phases with timing and IK targets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phases: Vec<AnimationPhase>,
    /// Procedural animation layers (breathing, sway, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub procedural_layers: Vec<ProceduralLayer>,
    /// Keyframe definitions (for FK animation or IK target animation).
    #[serde(default)]
    pub keyframes: Vec<AnimationKeyframe>,
    /// IK target keyframes (for animating IK targets).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ik_keyframes: Vec<IkKeyframe>,
    /// Interpolation method between keyframes.
    #[serde(default)]
    pub interpolation: InterpolationMode,
    /// Export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<AnimationExportSettings>,
    /// Animator rig configuration (visual aids for animators).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animator_rig: Option<AnimatorRigConfig>,
    /// Save .blend file alongside output.
    #[serde(default)]
    pub save_blend: bool,
    /// Validation conventions configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conventions: Option<ConventionsConfig>,
}
