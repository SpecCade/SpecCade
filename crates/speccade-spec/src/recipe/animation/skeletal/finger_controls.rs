//! Finger control types for hand animation.
//!
//! Provides curl, spread, and individual finger control with anatomically
//! correct conventions (positive curl = flexion/closing fist).

use serde::{Deserialize, Serialize};

// =============================================================================
// Finger Control Configuration
// =============================================================================

/// Finger control setup for a hand.
///
/// Creates simplified controls for animating fingers with curl (flexion)
/// and spread (abduction) parameters instead of individual bone rotations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FingerControls {
    /// Name of this control set (e.g., "hand_l_fingers").
    pub name: String,
    /// Side of the hand.
    pub side: HandSide,
    /// Bone name prefix (e.g., "finger" -> "finger_index_01_l").
    #[serde(default = "default_prefix")]
    pub bone_prefix: String,
    /// Which fingers to control.
    #[serde(default = "default_fingers")]
    pub fingers: Vec<FingerName>,
    /// Number of bones per finger (typically 3 for index-pinky, 2 for thumb).
    #[serde(default = "default_bones_per_finger")]
    pub bones_per_finger: u8,
    /// Maximum curl angle in degrees (positive = flexion).
    #[serde(default = "default_max_curl")]
    pub max_curl_degrees: f64,
    /// Maximum spread angle in degrees.
    #[serde(default = "default_max_spread")]
    pub max_spread_degrees: f64,
}

fn default_prefix() -> String {
    "finger".into()
}

fn default_fingers() -> Vec<FingerName> {
    vec![
        FingerName::Thumb,
        FingerName::Index,
        FingerName::Middle,
        FingerName::Ring,
        FingerName::Pinky,
    ]
}

fn default_bones_per_finger() -> u8 {
    3
}

fn default_max_curl() -> f64 {
    90.0
}

fn default_max_spread() -> f64 {
    15.0
}

/// Hand side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandSide {
    Left,
    Right,
}

impl HandSide {
    /// Returns the suffix used in bone names.
    pub fn suffix(&self) -> &'static str {
        match self {
            HandSide::Left => "_l",
            HandSide::Right => "_r",
        }
    }
}

/// Finger names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FingerName {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

impl FingerName {
    /// Returns the name used in bone names.
    pub fn bone_name(&self) -> &'static str {
        match self {
            FingerName::Thumb => "thumb",
            FingerName::Index => "index",
            FingerName::Middle => "middle",
            FingerName::Ring => "ring",
            FingerName::Pinky => "pinky",
        }
    }
}

/// Finger pose for animation keyframes.
///
/// All values are normalized (0.0 to 1.0) and will be scaled by the
/// max_curl_degrees and max_spread_degrees from FingerControls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FingerPose {
    /// Global curl for all fingers (0.0 = flat, 1.0 = full curl/fist).
    #[serde(default)]
    pub curl: f64,
    /// Global spread for all fingers (0.0 = together, 1.0 = max spread).
    #[serde(default)]
    pub spread: f64,
    /// Per-finger curl overrides (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumb_curl: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_curl: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub middle_curl: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ring_curl: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinky_curl: Option<f64>,
    /// Per-finger spread overrides (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumb_spread: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_spread: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub middle_spread: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ring_spread: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinky_spread: Option<f64>,
}

impl FingerPose {
    /// Creates a new finger pose with given curl and spread.
    pub fn new(curl: f64, spread: f64) -> Self {
        Self {
            curl,
            spread,
            ..Default::default()
        }
    }

    /// Creates a flat hand pose (no curl, no spread).
    pub fn flat() -> Self {
        Self::default()
    }

    /// Creates a fist pose (full curl, no spread).
    pub fn fist() -> Self {
        Self::new(1.0, 0.0)
    }

    /// Creates a relaxed pose (slight curl, slight spread).
    pub fn relaxed() -> Self {
        Self::new(0.3, 0.1)
    }

    /// Creates a pointing pose (index extended, others curled).
    pub fn pointing() -> Self {
        Self {
            curl: 1.0,
            spread: 0.0,
            index_curl: Some(0.0),
            thumb_curl: Some(0.5),
            ..Default::default()
        }
    }

    /// Gets the effective curl for a specific finger.
    pub fn get_finger_curl(&self, finger: FingerName) -> f64 {
        match finger {
            FingerName::Thumb => self.thumb_curl.unwrap_or(self.curl),
            FingerName::Index => self.index_curl.unwrap_or(self.curl),
            FingerName::Middle => self.middle_curl.unwrap_or(self.curl),
            FingerName::Ring => self.ring_curl.unwrap_or(self.curl),
            FingerName::Pinky => self.pinky_curl.unwrap_or(self.curl),
        }
    }

    /// Gets the effective spread for a specific finger.
    pub fn get_finger_spread(&self, finger: FingerName) -> f64 {
        match finger {
            FingerName::Thumb => self.thumb_spread.unwrap_or(self.spread),
            FingerName::Index => self.index_spread.unwrap_or(self.spread),
            FingerName::Middle => self.middle_spread.unwrap_or(self.spread),
            FingerName::Ring => self.ring_spread.unwrap_or(self.spread),
            FingerName::Pinky => self.pinky_spread.unwrap_or(self.spread),
        }
    }
}

/// Keyframe for finger animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FingerKeyframe {
    /// Time in seconds.
    pub time: f64,
    /// The finger control set to animate.
    pub controls: String,
    /// The finger pose at this keyframe.
    pub pose: FingerPose,
}

impl FingerControls {
    /// Creates a new finger control set.
    pub fn new(name: impl Into<String>, side: HandSide) -> Self {
        Self {
            name: name.into(),
            side,
            bone_prefix: default_prefix(),
            fingers: default_fingers(),
            bones_per_finger: default_bones_per_finger(),
            max_curl_degrees: default_max_curl(),
            max_spread_degrees: default_max_spread(),
        }
    }

    /// Generates bone names for a finger.
    pub fn finger_bone_names(&self, finger: FingerName) -> Vec<String> {
        let finger_name = finger.bone_name();
        let suffix = self.side.suffix();
        let bones = if finger == FingerName::Thumb { 2 } else { self.bones_per_finger };

        (1..=bones)
            .map(|i| format!("{}_{}{:02}{}", self.bone_prefix, finger_name, i, suffix))
            .collect()
    }

    /// Validates the finger controls configuration.
    pub fn validate(&self) -> Result<(), FingerControlsError> {
        if self.name.is_empty() {
            return Err(FingerControlsError::EmptyName);
        }
        if self.fingers.is_empty() {
            return Err(FingerControlsError::NoFingers);
        }
        if self.bones_per_finger == 0 {
            return Err(FingerControlsError::NoBones);
        }
        Ok(())
    }
}

/// Errors for finger controls validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FingerControlsError {
    #[error("Finger controls name cannot be empty")]
    EmptyName,
    #[error("Must specify at least one finger")]
    NoFingers,
    #[error("Bones per finger must be at least 1")]
    NoBones,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finger_controls_serialization() {
        let controls = FingerControls::new("hand_l_fingers", HandSide::Left);

        let json = serde_json::to_string_pretty(&controls).unwrap();
        let parsed: FingerControls = serde_json::from_str(&json).unwrap();
        assert_eq!(controls, parsed);
    }

    #[test]
    fn test_finger_pose_serialization() {
        let pose = FingerPose::pointing();

        let json = serde_json::to_string_pretty(&pose).unwrap();
        let parsed: FingerPose = serde_json::from_str(&json).unwrap();
        assert_eq!(pose, parsed);
    }

    #[test]
    fn test_finger_bone_names() {
        let controls = FingerControls::new("hand_l_fingers", HandSide::Left);

        let index_bones = controls.finger_bone_names(FingerName::Index);
        assert_eq!(index_bones, vec![
            "finger_index01_l",
            "finger_index02_l",
            "finger_index03_l",
        ]);

        let thumb_bones = controls.finger_bone_names(FingerName::Thumb);
        assert_eq!(thumb_bones, vec![
            "finger_thumb01_l",
            "finger_thumb02_l",
        ]);
    }
}
