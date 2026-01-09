//! Skeletal animation recipe types.

use serde::{Deserialize, Serialize};

use super::character::SkeletonPreset;

/// Parameters for the `skeletal_animation.blender_clip_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletalAnimationBlenderClipV1Params {
    /// Skeleton rig to animate.
    pub skeleton_preset: SkeletonPreset,
    /// Name of the animation clip.
    pub clip_name: String,
    /// Duration of the animation in seconds.
    pub duration_seconds: f64,
    /// Frames per second.
    pub fps: u8,
    /// Whether the animation should loop.
    #[serde(default)]
    pub r#loop: bool,
    /// Keyframe definitions.
    pub keyframes: Vec<AnimationKeyframe>,
    /// Interpolation method between keyframes.
    #[serde(default)]
    pub interpolation: InterpolationMode,
    /// Export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<AnimationExportSettings>,
}

/// Keyframe definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationKeyframe {
    /// Time in seconds.
    pub time: f64,
    /// Bone transforms at this keyframe.
    pub bones: std::collections::HashMap<String, BoneTransform>,
}

/// Bone transform at a keyframe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoneTransform {
    /// Position offset [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<[f64; 3]>,
    /// Rotation in euler angles [X, Y, Z] degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,
    /// Scale [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f64; 3]>,
}

impl BoneTransform {
    /// Creates a new bone transform with only rotation.
    pub fn with_rotation(rotation: [f64; 3]) -> Self {
        Self {
            position: None,
            rotation: Some(rotation),
            scale: None,
        }
    }

    /// Creates a new bone transform with only position.
    pub fn with_position(position: [f64; 3]) -> Self {
        Self {
            position: Some(position),
            rotation: None,
            scale: None,
        }
    }

    /// Creates a new bone transform with position and rotation.
    pub fn with_position_rotation(position: [f64; 3], rotation: [f64; 3]) -> Self {
        Self {
            position: Some(position),
            rotation: Some(rotation),
            scale: None,
        }
    }

    /// Returns true if this transform has any data.
    pub fn is_empty(&self) -> bool {
        self.position.is_none() && self.rotation.is_none() && self.scale.is_none()
    }
}

/// Interpolation mode for animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpolationMode {
    /// Linear interpolation.
    #[default]
    Linear,
    /// Bezier curve interpolation.
    Bezier,
    /// No interpolation (step/constant).
    Constant,
}

/// Export settings for animations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimationExportSettings {
    /// Bake all transforms to keyframes.
    #[serde(default = "default_true")]
    pub bake_transforms: bool,
    /// Optimize keyframes (remove redundant keys).
    #[serde(default)]
    pub optimize_keyframes: bool,
    /// Export as separate file (vs embedded in mesh).
    #[serde(default)]
    pub separate_file: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AnimationExportSettings {
    fn default() -> Self {
        Self {
            bake_transforms: true,
            optimize_keyframes: false,
            separate_file: false,
        }
    }
}

/// Common animation presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationPreset {
    /// Idle breathing animation.
    Idle,
    /// Walk cycle.
    Walk,
    /// Run cycle.
    Run,
    /// Jump animation.
    Jump,
    /// Attack animation.
    Attack,
    /// Hit/damage reaction.
    Hit,
    /// Death animation.
    Death,
}

impl AnimationPreset {
    /// Returns a suggested duration for this animation type.
    pub fn suggested_duration(&self) -> f64 {
        match self {
            AnimationPreset::Idle => 2.0,
            AnimationPreset::Walk => 1.0,
            AnimationPreset::Run => 0.6,
            AnimationPreset::Jump => 0.8,
            AnimationPreset::Attack => 0.5,
            AnimationPreset::Hit => 0.3,
            AnimationPreset::Death => 1.5,
        }
    }

    /// Returns whether this animation typically loops.
    pub fn typically_loops(&self) -> bool {
        matches!(
            self,
            AnimationPreset::Idle | AnimationPreset::Walk | AnimationPreset::Run
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bone_transform() {
        let transform = BoneTransform::with_rotation([15.0, 0.0, 0.0]);
        assert!(transform.rotation.is_some());
        assert!(transform.position.is_none());
        assert!(!transform.is_empty());

        let empty = BoneTransform {
            position: None,
            rotation: None,
            scale: None,
        };
        assert!(empty.is_empty());
    }

    #[test]
    fn test_interpolation_mode_serde() {
        let mode = InterpolationMode::Bezier;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"bezier\"");

        let parsed: InterpolationMode = serde_json::from_str("\"linear\"").unwrap();
        assert_eq!(parsed, InterpolationMode::Linear);
    }

    #[test]
    fn test_animation_preset() {
        assert_eq!(AnimationPreset::Walk.suggested_duration(), 1.0);
        assert!(AnimationPreset::Walk.typically_loops());
        assert!(!AnimationPreset::Death.typically_loops());
    }

    #[test]
    fn test_keyframe_serde() {
        let mut bones = std::collections::HashMap::new();
        bones.insert(
            "upper_leg_l".to_string(),
            BoneTransform::with_rotation([15.0, 0.0, 0.0]),
        );

        let keyframe = AnimationKeyframe { time: 0.0, bones };

        let json = serde_json::to_string(&keyframe).unwrap();
        assert!(json.contains("upper_leg_l"));
        assert!(json.contains("rotation"));

        let parsed: AnimationKeyframe = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.time, 0.0);
    }
}
