//! Bone constraint types for skeletal rigs.

use serde::{Deserialize, Serialize};

use super::common::ConstraintAxis;

// =============================================================================
// Bone Constraint Types
// =============================================================================

/// Bone constraint types for limiting bone rotations.
///
/// These constraints map to Blender's constraint system and are used to create
/// realistic joint limits for skeletal animations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum BoneConstraint {
    /// Hinge constraint - allows rotation around a single axis only.
    /// Maps to LIMIT_ROTATION with only one axis enabled.
    /// Ideal for elbows and knees.
    Hinge {
        /// Target bone name.
        bone: String,
        /// Rotation axis.
        #[serde(default)]
        axis: ConstraintAxis,
        /// Minimum rotation angle in degrees.
        #[serde(default)]
        min_angle: f64,
        /// Maximum rotation angle in degrees.
        #[serde(default = "default_hinge_max_angle")]
        max_angle: f64,
    },
    /// Ball constraint - allows rotation within a cone with optional twist limits.
    /// Maps to LIMIT_ROTATION with cone-like limits.
    /// Ideal for shoulders and hips.
    Ball {
        /// Target bone name.
        bone: String,
        /// Maximum cone angle in degrees (from the bone axis).
        #[serde(default = "default_cone_angle")]
        cone_angle: f64,
        /// Minimum twist rotation in degrees (around the bone axis).
        #[serde(default = "default_twist_min")]
        twist_min: f64,
        /// Maximum twist rotation in degrees (around the bone axis).
        #[serde(default = "default_twist_max")]
        twist_max: f64,
    },
    /// Planar constraint - allows rotation in a plane (one axis locked).
    /// Maps to LIMIT_ROTATION with one axis locked to zero.
    /// Ideal for wrists and ankles with limited lateral movement.
    Planar {
        /// Target bone name.
        bone: String,
        /// The normal axis of the plane (this axis is locked).
        #[serde(default)]
        plane_normal: ConstraintAxis,
        /// Minimum rotation angle in degrees (around allowed axes).
        #[serde(default = "default_planar_min")]
        min_angle: f64,
        /// Maximum rotation angle in degrees (around allowed axes).
        #[serde(default = "default_planar_max")]
        max_angle: f64,
    },
    /// Soft constraint - allows rotation with spring-like resistance.
    /// Maps to DAMPED_TRACK or custom driver for damping effect.
    /// Ideal for secondary motion like tails and hair.
    Soft {
        /// Target bone name.
        bone: String,
        /// Stiffness factor (0.0 = no resistance, 1.0 = maximum resistance).
        #[serde(default = "default_stiffness")]
        stiffness: f64,
        /// Damping factor (0.0 = no damping, 1.0 = critically damped).
        #[serde(default = "default_damping")]
        damping: f64,
    },
}

fn default_hinge_max_angle() -> f64 {
    160.0
}

fn default_cone_angle() -> f64 {
    45.0
}

fn default_twist_min() -> f64 {
    -45.0
}

fn default_twist_max() -> f64 {
    45.0
}

fn default_planar_min() -> f64 {
    -30.0
}

fn default_planar_max() -> f64 {
    30.0
}

fn default_stiffness() -> f64 {
    0.5
}

fn default_damping() -> f64 {
    0.5
}

impl BoneConstraint {
    /// Creates a new hinge constraint.
    pub fn hinge(bone: impl Into<String>, axis: ConstraintAxis, min_angle: f64, max_angle: f64) -> Self {
        BoneConstraint::Hinge {
            bone: bone.into(),
            axis,
            min_angle,
            max_angle,
        }
    }

    /// Creates a new ball constraint.
    pub fn ball(bone: impl Into<String>, cone_angle: f64, twist_min: f64, twist_max: f64) -> Self {
        BoneConstraint::Ball {
            bone: bone.into(),
            cone_angle,
            twist_min,
            twist_max,
        }
    }

    /// Creates a new planar constraint.
    pub fn planar(bone: impl Into<String>, plane_normal: ConstraintAxis, min_angle: f64, max_angle: f64) -> Self {
        BoneConstraint::Planar {
            bone: bone.into(),
            plane_normal,
            min_angle,
            max_angle,
        }
    }

    /// Creates a new soft constraint.
    pub fn soft(bone: impl Into<String>, stiffness: f64, damping: f64) -> Self {
        BoneConstraint::Soft {
            bone: bone.into(),
            stiffness: stiffness.clamp(0.0, 1.0),
            damping: damping.clamp(0.0, 1.0),
        }
    }

    /// Returns the target bone name for this constraint.
    pub fn bone_name(&self) -> &str {
        match self {
            BoneConstraint::Hinge { bone, .. } => bone,
            BoneConstraint::Ball { bone, .. } => bone,
            BoneConstraint::Planar { bone, .. } => bone,
            BoneConstraint::Soft { bone, .. } => bone,
        }
    }

    /// Validates the constraint configuration.
    pub fn validate(&self) -> Result<(), BoneConstraintError> {
        match self {
            BoneConstraint::Hinge { bone, min_angle, max_angle, .. } => {
                if bone.is_empty() {
                    return Err(BoneConstraintError::EmptyBoneName);
                }
                if min_angle > max_angle {
                    return Err(BoneConstraintError::InvalidAngleRange {
                        min: *min_angle,
                        max: *max_angle,
                    });
                }
            }
            BoneConstraint::Ball { bone, cone_angle, twist_min, twist_max, .. } => {
                if bone.is_empty() {
                    return Err(BoneConstraintError::EmptyBoneName);
                }
                if *cone_angle < 0.0 || *cone_angle > 180.0 {
                    return Err(BoneConstraintError::InvalidConeAngle(*cone_angle));
                }
                if twist_min > twist_max {
                    return Err(BoneConstraintError::InvalidAngleRange {
                        min: *twist_min,
                        max: *twist_max,
                    });
                }
            }
            BoneConstraint::Planar { bone, min_angle, max_angle, .. } => {
                if bone.is_empty() {
                    return Err(BoneConstraintError::EmptyBoneName);
                }
                if min_angle > max_angle {
                    return Err(BoneConstraintError::InvalidAngleRange {
                        min: *min_angle,
                        max: *max_angle,
                    });
                }
            }
            BoneConstraint::Soft { bone, stiffness, damping } => {
                if bone.is_empty() {
                    return Err(BoneConstraintError::EmptyBoneName);
                }
                if *stiffness < 0.0 || *stiffness > 1.0 {
                    return Err(BoneConstraintError::InvalidStiffness(*stiffness));
                }
                if *damping < 0.0 || *damping > 1.0 {
                    return Err(BoneConstraintError::InvalidDamping(*damping));
                }
            }
        }
        Ok(())
    }
}

/// Errors that can occur when validating a bone constraint.
#[derive(Debug, Clone, PartialEq)]
pub enum BoneConstraintError {
    /// Bone name is empty.
    EmptyBoneName,
    /// Angle range is invalid (min > max).
    InvalidAngleRange { min: f64, max: f64 },
    /// Cone angle is out of valid range (0-180).
    InvalidConeAngle(f64),
    /// Stiffness value is out of range (0-1).
    InvalidStiffness(f64),
    /// Damping value is out of range (0-1).
    InvalidDamping(f64),
}

impl std::fmt::Display for BoneConstraintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoneConstraintError::EmptyBoneName => write!(f, "Bone name cannot be empty"),
            BoneConstraintError::InvalidAngleRange { min, max } => {
                write!(f, "Invalid angle range: min ({}) > max ({})", min, max)
            }
            BoneConstraintError::InvalidConeAngle(angle) => {
                write!(f, "Cone angle must be between 0 and 180 degrees, got {}", angle)
            }
            BoneConstraintError::InvalidStiffness(val) => {
                write!(f, "Stiffness must be between 0 and 1, got {}", val)
            }
            BoneConstraintError::InvalidDamping(val) => {
                write!(f, "Damping must be between 0 and 1, got {}", val)
            }
        }
    }
}

impl std::error::Error for BoneConstraintError {}

/// Configuration for bone constraints on a rig.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ConstraintConfig {
    /// List of bone constraints to apply.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<BoneConstraint>,
}

impl ConstraintConfig {
    /// Creates a new empty constraint config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a constraint to the config.
    pub fn with_constraint(mut self, constraint: BoneConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Validates all constraints in the config.
    pub fn validate(&self) -> Result<(), BoneConstraintError> {
        for constraint in &self.constraints {
            constraint.validate()?;
        }
        Ok(())
    }

    /// Returns true if there are no constraints.
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }
}
