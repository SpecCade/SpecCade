//! Animation clip types including keyframes and bone transforms.

use serde::{Deserialize, Serialize};

use super::common::{AnimationExportSettings, InterpolationMode};
use crate::recipe::character::SkeletonPreset;

// =============================================================================
// Bone Transforms
// =============================================================================

/// Bone transform at a keyframe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

// =============================================================================
// Keyframes
// =============================================================================

/// Keyframe definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationKeyframe {
    /// Time in seconds.
    pub time: f64,
    /// Bone transforms at this keyframe.
    pub bones: std::collections::HashMap<String, BoneTransform>,
}

/// Keyframe for IK target animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IkKeyframe {
    /// Time in seconds.
    pub time: f64,
    /// IK target transforms at this keyframe.
    /// Keys are IK chain names (e.g., "ik_leg_l"), values are world positions.
    pub targets: std::collections::HashMap<String, IkTargetTransform>,
}

/// Transform for an IK target at a keyframe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IkTargetTransform {
    /// World position [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<[f64; 3]>,
    /// World rotation in euler angles [X, Y, Z] degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,
    /// IK/FK blend value (0.0 = pure FK, 1.0 = pure IK).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ik_fk_blend: Option<f64>,
}

impl IkTargetTransform {
    /// Creates a new IK target transform with just position.
    pub fn at_position(position: [f64; 3]) -> Self {
        Self {
            position: Some(position),
            rotation: None,
            ik_fk_blend: None,
        }
    }

    /// Creates a new IK target transform with position and rotation.
    pub fn with_position_rotation(position: [f64; 3], rotation: [f64; 3]) -> Self {
        Self {
            position: Some(position),
            rotation: Some(rotation),
            ik_fk_blend: None,
        }
    }

    /// Sets the IK/FK blend value.
    pub fn with_ik_fk_blend(mut self, blend: f64) -> Self {
        self.ik_fk_blend = Some(blend.clamp(0.0, 1.0));
        self
    }
}

// =============================================================================
// Animation Clip Params
// =============================================================================

/// Parameters for the `skeletal_animation.blender_clip_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

    #[test]
    fn test_ik_keyframe_serde() {
        let mut targets = std::collections::HashMap::new();
        targets.insert(
            "ik_leg_l".to_string(),
            IkTargetTransform::at_position([0.1, 0.0, 0.0]),
        );
        targets.insert(
            "ik_leg_r".to_string(),
            IkTargetTransform::at_position([-0.1, 0.0, 0.0]).with_ik_fk_blend(0.5),
        );

        let keyframe = IkKeyframe {
            time: 0.5,
            targets,
        };

        let json = serde_json::to_string(&keyframe).unwrap();
        assert!(json.contains("ik_leg_l"));
        assert!(json.contains("ik_leg_r"));
        assert!(json.contains("ik_fk_blend"));

        let parsed: IkKeyframe = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.time, 0.5);
        assert_eq!(parsed.targets.len(), 2);
    }
}
