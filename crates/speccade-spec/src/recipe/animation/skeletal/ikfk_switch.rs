//! IK/FK switching types for seamless animation workflow.
//!
//! Allows instant pose transfer between IK and FK control without popping.

use serde::{Deserialize, Serialize};

// =============================================================================
// IK/FK Switch Configuration
// =============================================================================

/// IK/FK switch configuration for a limb.
///
/// Enables animators to switch between IK and FK control modes with
/// automatic pose snapping to prevent visual popping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IkFkSwitch {
    /// Name of this switch (e.g., "arm_l_ikfk").
    pub name: String,
    /// The IK chain this switch controls.
    pub ik_chain: String,
    /// FK bones in order (root to tip, e.g., ["upper_arm_l", "lower_arm_l", "hand_l"]).
    pub fk_bones: Vec<String>,
    /// Default mode at frame 0.
    #[serde(default)]
    pub default_mode: IkFkMode,
}

/// IK/FK control mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IkFkMode {
    /// Inverse Kinematics mode (target-driven).
    #[default]
    Ik,
    /// Forward Kinematics mode (rotation-driven).
    Fk,
}

/// Keyframe for IK/FK switch with optional snapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IkFkKeyframe {
    /// Frame number for this switch.
    pub frame: u32,
    /// The switch to control.
    pub switch: String,
    /// Mode to switch to.
    pub mode: IkFkMode,
    /// If true, snap pose before switching to prevent visual popping.
    /// - When switching IK->FK: Copy FK bone rotations to match current IK pose.
    /// - When switching FK->IK: Move IK target/pole to match current FK pose.
    #[serde(default = "default_true")]
    pub snap: bool,
}

fn default_true() -> bool {
    true
}

impl IkFkSwitch {
    /// Creates a new IK/FK switch.
    pub fn new(name: impl Into<String>, ik_chain: impl Into<String>, fk_bones: Vec<String>) -> Self {
        Self {
            name: name.into(),
            ik_chain: ik_chain.into(),
            fk_bones,
            default_mode: IkFkMode::default(),
        }
    }

    /// Sets the default mode.
    pub fn with_default_mode(mut self, mode: IkFkMode) -> Self {
        self.default_mode = mode;
        self
    }

    /// Validates the switch configuration.
    pub fn validate(&self) -> Result<(), IkFkSwitchError> {
        if self.name.is_empty() {
            return Err(IkFkSwitchError::EmptyName);
        }
        if self.ik_chain.is_empty() {
            return Err(IkFkSwitchError::EmptyIkChain);
        }
        if self.fk_bones.is_empty() {
            return Err(IkFkSwitchError::NoFkBones);
        }
        Ok(())
    }
}

/// Errors for IK/FK switch validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IkFkSwitchError {
    #[error("IK/FK switch name cannot be empty")]
    EmptyName,
    #[error("IK chain name cannot be empty")]
    EmptyIkChain,
    #[error("FK bones list cannot be empty")]
    NoFkBones,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ikfk_switch_serialization() {
        let switch = IkFkSwitch::new(
            "arm_l_ikfk",
            "arm_l_ik",
            vec!["upper_arm_l".into(), "lower_arm_l".into(), "hand_l".into()],
        );

        let json = serde_json::to_string_pretty(&switch).unwrap();
        let parsed: IkFkSwitch = serde_json::from_str(&json).unwrap();
        assert_eq!(switch, parsed);
    }

    #[test]
    fn test_ikfk_keyframe_serialization() {
        let kf = IkFkKeyframe {
            frame: 30,
            switch: "arm_l_ikfk".into(),
            mode: IkFkMode::Fk,
            snap: true,
        };

        let json = serde_json::to_string_pretty(&kf).unwrap();
        let parsed: IkFkKeyframe = serde_json::from_str(&json).unwrap();
        assert_eq!(kf, parsed);
    }
}
