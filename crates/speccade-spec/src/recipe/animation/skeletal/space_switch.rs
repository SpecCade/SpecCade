//! Space switching types for dynamic parent changes.
//!
//! Allows bones to dynamically change what their transforms are relative to.
//! Essential for picking up objects, planting hands, and character interactions.

use serde::{Deserialize, Serialize};

// =============================================================================
// Space Switch Configuration
// =============================================================================

/// Defines available parent spaces for a bone.
///
/// Space switching allows a bone to change its parent space mid-animation,
/// which is essential for:
/// - Picking up objects (hand switches to object space)
/// - Planting hands on walls (hand switches to world space)
/// - Two-character interactions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpaceSwitch {
    /// Name of this switch (e.g., "hand_l_space").
    pub name: String,
    /// The bone being controlled.
    pub bone: String,
    /// Available parent spaces.
    pub spaces: Vec<ParentSpace>,
    /// Default space index (0-based).
    #[serde(default)]
    pub default_space: usize,
}

/// A parent space option.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParentSpace {
    /// Display name (e.g., "World", "Root", "Head").
    pub name: String,
    /// Space type.
    #[serde(flatten)]
    pub kind: SpaceKind,
}

/// Type of parent space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpaceKind {
    /// World space (no parent).
    World,
    /// Root bone space.
    Root,
    /// Relative to another bone.
    Bone {
        /// The bone to use as parent.
        bone: String,
    },
    /// Relative to an external object/empty.
    Object {
        /// The object name.
        object: String,
    },
}

/// Keyframe for space switch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpaceSwitchKeyframe {
    /// Frame number for this switch.
    pub frame: u32,
    /// The switch to control.
    pub switch: String,
    /// Space index to switch to (0-based into spaces array).
    pub space: usize,
    /// If true, maintain visual pose when switching spaces (compensate transform).
    #[serde(default = "default_true")]
    pub maintain_pose: bool,
}

fn default_true() -> bool {
    true
}

impl SpaceSwitch {
    /// Creates a new space switch with world and root spaces.
    pub fn new(name: impl Into<String>, bone: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bone: bone.into(),
            spaces: vec![
                ParentSpace {
                    name: "World".into(),
                    kind: SpaceKind::World,
                },
                ParentSpace {
                    name: "Root".into(),
                    kind: SpaceKind::Root,
                },
            ],
            default_space: 0,
        }
    }

    /// Adds a bone space option.
    pub fn with_bone_space(mut self, name: impl Into<String>, bone: impl Into<String>) -> Self {
        self.spaces.push(ParentSpace {
            name: name.into(),
            kind: SpaceKind::Bone { bone: bone.into() },
        });
        self
    }

    /// Adds an object space option.
    pub fn with_object_space(mut self, name: impl Into<String>, object: impl Into<String>) -> Self {
        self.spaces.push(ParentSpace {
            name: name.into(),
            kind: SpaceKind::Object {
                object: object.into(),
            },
        });
        self
    }

    /// Sets the default space index.
    pub fn with_default_space(mut self, index: usize) -> Self {
        self.default_space = index;
        self
    }

    /// Validates the space switch configuration.
    pub fn validate(&self) -> Result<(), SpaceSwitchError> {
        if self.name.is_empty() {
            return Err(SpaceSwitchError::EmptyName);
        }
        if self.bone.is_empty() {
            return Err(SpaceSwitchError::EmptyBone);
        }
        if self.spaces.is_empty() {
            return Err(SpaceSwitchError::NoSpaces);
        }
        if self.default_space >= self.spaces.len() {
            return Err(SpaceSwitchError::InvalidDefaultSpace {
                index: self.default_space,
                max: self.spaces.len() - 1,
            });
        }
        Ok(())
    }
}

/// Errors for space switch validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpaceSwitchError {
    #[error("Space switch name cannot be empty")]
    EmptyName,
    #[error("Bone name cannot be empty")]
    EmptyBone,
    #[error("Space switch must have at least one space")]
    NoSpaces,
    #[error("Invalid default space index {index}, max is {max}")]
    InvalidDefaultSpace { index: usize, max: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_switch_serialization() {
        let switch = SpaceSwitch::new("hand_l_space", "hand_l")
            .with_bone_space("Chest", "chest")
            .with_bone_space("Head", "head");

        let json = serde_json::to_string_pretty(&switch).unwrap();
        let parsed: SpaceSwitch = serde_json::from_str(&json).unwrap();
        assert_eq!(switch, parsed);
    }

    #[test]
    fn test_space_switch_keyframe_serialization() {
        let kf = SpaceSwitchKeyframe {
            frame: 60,
            switch: "hand_l_space".into(),
            space: 2,
            maintain_pose: true,
        };

        let json = serde_json::to_string_pretty(&kf).unwrap();
        let parsed: SpaceSwitchKeyframe = serde_json::from_str(&json).unwrap();
        assert_eq!(kf, parsed);
    }
}
