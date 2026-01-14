//! Aim constraints and twist bone configuration.

use serde::{Deserialize, Serialize};

use super::super::common::{AimAxis, ConstraintAxis};

// =============================================================================
// Aim Constraint Configuration
// =============================================================================

/// Configuration for an aim (look-at) constraint.
/// Makes a bone always point toward a target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AimConstraint {
    /// Name of this aim constraint.
    pub name: String,
    /// Bone to apply the aim constraint to.
    pub bone: String,
    /// Target to aim at (bone name or empty object name).
    pub target: String,
    /// Axis of the bone that should point at the target.
    #[serde(default)]
    pub track_axis: AimAxis,
    /// Up axis for the bone (perpendicular to track axis).
    #[serde(default = "default_up_axis")]
    pub up_axis: ConstraintAxis,
    /// Constraint influence (0.0-1.0).
    #[serde(default = "default_influence")]
    pub influence: f64,
}

fn default_up_axis() -> ConstraintAxis {
    ConstraintAxis::Z
}

fn default_influence() -> f64 {
    1.0
}

impl AimConstraint {
    /// Creates a new aim constraint.
    pub fn new(
        name: impl Into<String>,
        bone: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            bone: bone.into(),
            target: target.into(),
            track_axis: AimAxis::default(),
            up_axis: default_up_axis(),
            influence: 1.0,
        }
    }

    /// Sets the track axis.
    pub fn with_track_axis(mut self, axis: AimAxis) -> Self {
        self.track_axis = axis;
        self
    }

    /// Sets the up axis.
    pub fn with_up_axis(mut self, axis: ConstraintAxis) -> Self {
        self.up_axis = axis;
        self
    }

    /// Sets the influence.
    pub fn with_influence(mut self, influence: f64) -> Self {
        self.influence = influence.clamp(0.0, 1.0);
        self
    }
}

// =============================================================================
// Twist Bone Configuration
// =============================================================================

/// Configuration for twist bone distribution.
/// Distributes rotation from a source bone across twist bones.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TwistBone {
    /// Name of this twist setup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Source bone to copy rotation from.
    pub source: String,
    /// Target twist bone.
    pub target: String,
    /// Axis to copy rotation on.
    #[serde(default = "default_twist_axis")]
    pub axis: ConstraintAxis,
    /// Influence factor (0.0-1.0).
    #[serde(default = "default_twist_influence")]
    pub influence: f64,
}

fn default_twist_axis() -> ConstraintAxis {
    ConstraintAxis::Y
}

fn default_twist_influence() -> f64 {
    0.5
}

impl TwistBone {
    /// Creates a new twist bone configuration.
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            name: None,
            source: source.into(),
            target: target.into(),
            axis: default_twist_axis(),
            influence: default_twist_influence(),
        }
    }

    /// Sets the name of this twist setup.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the twist axis.
    pub fn with_axis(mut self, axis: ConstraintAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Sets the influence factor.
    pub fn with_influence(mut self, influence: f64) -> Self {
        self.influence = influence.clamp(0.0, 1.0);
        self
    }
}
