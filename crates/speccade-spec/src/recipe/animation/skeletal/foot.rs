//! Foot system configuration for IK rigs.

use serde::{Deserialize, Serialize};

// =============================================================================
// Foot System Configuration
// =============================================================================

/// Configuration for an IK foot roll system.
/// Provides automatic heel-toe roll during foot plants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FootSystem {
    /// Name of this foot system (e.g., "foot_l", "foot_r").
    pub name: String,
    /// IK target bone name for the foot.
    pub ik_target: String,
    /// Heel pivot bone name.
    pub heel_bone: String,
    /// Toe pivot bone name.
    pub toe_bone: String,
    /// Ball (mid-foot) pivot bone name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ball_bone: Option<String>,
    /// Roll angle limits in degrees [min, max].
    #[serde(default = "default_foot_roll_limits")]
    pub roll_limits: [f64; 2],
}

fn default_foot_roll_limits() -> [f64; 2] {
    [-30.0, 60.0]
}

impl FootSystem {
    /// Creates a new foot system with minimal configuration.
    pub fn new(
        name: impl Into<String>,
        ik_target: impl Into<String>,
        heel_bone: impl Into<String>,
        toe_bone: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            ik_target: ik_target.into(),
            heel_bone: heel_bone.into(),
            toe_bone: toe_bone.into(),
            ball_bone: None,
            roll_limits: default_foot_roll_limits(),
        }
    }

    /// Adds a ball bone to this foot system.
    pub fn with_ball_bone(mut self, ball_bone: impl Into<String>) -> Self {
        self.ball_bone = Some(ball_bone.into());
        self
    }

    /// Sets the roll limits.
    pub fn with_roll_limits(mut self, min: f64, max: f64) -> Self {
        self.roll_limits = [min, max];
        self
    }
}
