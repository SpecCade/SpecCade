//! Skeleton types and presets.

use serde::{Deserialize, Serialize};

/// Predefined skeleton rigs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkeletonPreset {
    /// Basic humanoid skeleton with 22 bones.
    HumanoidBasicV1,
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
        }
    }

    /// Returns the number of bones in this skeleton.
    pub fn bone_count(&self) -> usize {
        self.bone_names().len()
    }
}

/// A bone in a custom skeleton definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
}
