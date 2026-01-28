//! Skeleton types and presets.

use serde::{Deserialize, Serialize};

/// Predefined skeleton rigs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkeletonPreset {
    /// Basic humanoid skeleton with 20 bones (no fingers).
    HumanoidBasicV1,
    /// Detailed humanoid skeleton with 52 bones (full 3-bone fingers, toes).
    HumanoidDetailedV1,
    /// Game-optimized humanoid with 40 bones (twist bones, 1-bone fingers).
    HumanoidGameV1,
}

impl SkeletonPreset {
    /// Returns the bone names for this skeleton preset.
    pub fn bone_names(&self) -> &'static [&'static str] {
        match self {
            SkeletonPreset::HumanoidBasicV1 => &[
                "root",
                "hips",
                "spine",
                "chest",
                "neck",
                "head",
                "shoulder_l",
                "upper_arm_l",
                "lower_arm_l",
                "hand_l",
                "shoulder_r",
                "upper_arm_r",
                "lower_arm_r",
                "hand_r",
                "upper_leg_l",
                "lower_leg_l",
                "foot_l",
                "upper_leg_r",
                "lower_leg_r",
                "foot_r",
            ],
            SkeletonPreset::HumanoidDetailedV1 => &[
                // Core
                "root",
                "hips",
                "spine",
                "chest",
                "neck",
                "head",
                // Left arm
                "shoulder_l",
                "upper_arm_l",
                "lower_arm_l",
                "hand_l",
                // Left hand fingers
                "thumb_01_l",
                "thumb_02_l",
                "thumb_03_l",
                "index_01_l",
                "index_02_l",
                "index_03_l",
                "middle_01_l",
                "middle_02_l",
                "middle_03_l",
                "ring_01_l",
                "ring_02_l",
                "ring_03_l",
                "pinky_01_l",
                "pinky_02_l",
                "pinky_03_l",
                // Right arm
                "shoulder_r",
                "upper_arm_r",
                "lower_arm_r",
                "hand_r",
                // Right hand fingers
                "thumb_01_r",
                "thumb_02_r",
                "thumb_03_r",
                "index_01_r",
                "index_02_r",
                "index_03_r",
                "middle_01_r",
                "middle_02_r",
                "middle_03_r",
                "ring_01_r",
                "ring_02_r",
                "ring_03_r",
                "pinky_01_r",
                "pinky_02_r",
                "pinky_03_r",
                // Left leg
                "upper_leg_l",
                "lower_leg_l",
                "foot_l",
                "toe_l",
                // Right leg
                "upper_leg_r",
                "lower_leg_r",
                "foot_r",
                "toe_r",
            ],
            SkeletonPreset::HumanoidGameV1 => &[
                // Core
                "root",
                "hips",
                "spine",
                "chest",
                "neck",
                "head",
                // Left arm with twist bones
                "shoulder_l",
                "upper_arm_l",
                "upper_arm_twist_l",
                "lower_arm_l",
                "lower_arm_twist_l",
                "hand_l",
                // Left hand (simplified 1-bone fingers)
                "thumb_l",
                "index_l",
                "middle_l",
                "ring_l",
                "pinky_l",
                // Right arm with twist bones
                "shoulder_r",
                "upper_arm_r",
                "upper_arm_twist_r",
                "lower_arm_r",
                "lower_arm_twist_r",
                "hand_r",
                // Right hand (simplified 1-bone fingers)
                "thumb_r",
                "index_r",
                "middle_r",
                "ring_r",
                "pinky_r",
                // Left leg with twist bones
                "upper_leg_l",
                "upper_leg_twist_l",
                "lower_leg_l",
                "lower_leg_twist_l",
                "foot_l",
                "toe_l",
                // Right leg with twist bones
                "upper_leg_r",
                "upper_leg_twist_r",
                "lower_leg_r",
                "lower_leg_twist_r",
                "foot_r",
                "toe_r",
            ],
        }
    }

    /// Returns the number of bones in this skeleton.
    pub fn bone_count(&self) -> usize {
        self.bone_names().len()
    }
}

/// A bone in a custom skeleton definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletonBone {
    /// Unique bone name.
    pub bone: String,
    /// Bone head position [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<[f64; 3]>,
    /// Bone tail position [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tail: Option<[f64; 3]>,
    /// Parent bone name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Mirror from another bone (L->R reflection across X=0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror: Option<String>,
    /// Bone roll in degrees (rotation around bone's Y axis).
    /// Controls the orientation of the bone's local X/Z axes.
    /// Important for correct flexion direction on finger bones.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roll: Option<f64>,
}
