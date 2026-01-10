//! Skeletal animation recipe types.

use serde::{Deserialize, Serialize};

use super::character::SkeletonPreset;

// =============================================================================
// IK Chain Types
// =============================================================================

/// IK preset types for common rig configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IkPreset {
    /// Humanoid leg IK (hip -> knee -> foot).
    /// Creates IK targets at feet with pole targets at knees.
    HumanoidLegs,
    /// Humanoid arm IK (shoulder -> elbow -> hand).
    /// Creates IK targets at hands with pole targets at elbows.
    HumanoidArms,
    /// Quadruped foreleg IK (shoulder -> elbow -> paw).
    QuadrupedForelegs,
    /// Quadruped hindleg IK (hip -> knee -> paw).
    QuadrupedHindlegs,
    /// Tentacle IK (multi-bone chain with spline-like behavior).
    Tentacle,
    /// Tail IK (multi-bone chain for tail animation).
    Tail,
}

impl IkPreset {
    /// Returns the default chain length for this preset.
    pub fn default_chain_length(&self) -> u8 {
        match self {
            IkPreset::HumanoidLegs => 2,
            IkPreset::HumanoidArms => 2,
            IkPreset::QuadrupedForelegs => 2,
            IkPreset::QuadrupedHindlegs => 2,
            IkPreset::Tentacle => 4,
            IkPreset::Tail => 4,
        }
    }

    /// Returns whether this preset typically uses a pole target.
    pub fn uses_pole_target(&self) -> bool {
        match self {
            IkPreset::HumanoidLegs => true,
            IkPreset::HumanoidArms => true,
            IkPreset::QuadrupedForelegs => true,
            IkPreset::QuadrupedHindlegs => true,
            IkPreset::Tentacle => false,
            IkPreset::Tail => false,
        }
    }

    /// Returns the required bone suffixes for this preset.
    /// Used for validation and auto-detection.
    pub fn required_bone_patterns(&self) -> &'static [&'static str] {
        match self {
            IkPreset::HumanoidLegs => &["upper_leg", "lower_leg", "foot"],
            IkPreset::HumanoidArms => &["upper_arm", "lower_arm", "hand"],
            IkPreset::QuadrupedForelegs => &["front_upper", "front_lower", "front_paw"],
            IkPreset::QuadrupedHindlegs => &["back_upper", "back_lower", "back_paw"],
            IkPreset::Tentacle => &["tentacle"],
            IkPreset::Tail => &["tail"],
        }
    }

    /// Returns the default pole offset direction for this preset.
    /// Represented as [X, Y, Z] offset from the mid-bone.
    pub fn default_pole_offset(&self) -> Option<[f64; 3]> {
        match self {
            IkPreset::HumanoidLegs => Some([0.0, 0.3, 0.0]),     // Forward of knee
            IkPreset::HumanoidArms => Some([0.0, -0.3, 0.0]),    // Behind elbow
            IkPreset::QuadrupedForelegs => Some([0.0, 0.3, 0.0]),
            IkPreset::QuadrupedHindlegs => Some([0.0, -0.3, 0.0]),
            IkPreset::Tentacle => None,
            IkPreset::Tail => None,
        }
    }
}

/// Configuration for an IK target (the end effector target).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IkTargetConfig {
    /// Name of the target control bone/empty.
    pub name: String,
    /// Initial world position [X, Y, Z] (optional, defaults to tip bone position).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<[f64; 3]>,
    /// Name of an existing bone to use as target (optional, mutually exclusive with position).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone: Option<String>,
}

impl IkTargetConfig {
    /// Creates a new IK target config with just a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            position: None,
            bone: None,
        }
    }

    /// Creates a target config positioned at specific coordinates.
    pub fn at_position(name: impl Into<String>, position: [f64; 3]) -> Self {
        Self {
            name: name.into(),
            position: Some(position),
            bone: None,
        }
    }

    /// Creates a target config attached to an existing bone.
    pub fn from_bone(name: impl Into<String>, bone: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            position: None,
            bone: Some(bone.into()),
        }
    }
}

/// Configuration for an IK pole target (controls bend direction).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoleConfig {
    /// Name of the pole target control bone/empty.
    pub name: String,
    /// Initial world position [X, Y, Z] (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<[f64; 3]>,
    /// Name of an existing bone to use as pole target (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone: Option<String>,
    /// Pole angle in degrees (rotation around the chain axis).
    #[serde(default)]
    pub angle: f64,
}

impl PoleConfig {
    /// Creates a new pole config with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            position: None,
            bone: None,
            angle: 0.0,
        }
    }

    /// Creates a pole config with a specific position.
    pub fn at_position(name: impl Into<String>, position: [f64; 3]) -> Self {
        Self {
            name: name.into(),
            position: Some(position),
            bone: None,
            angle: 0.0,
        }
    }

    /// Sets the pole angle.
    pub fn with_angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }
}

/// IK chain definition for a single limb or bone chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IkChain {
    /// Unique name for this IK chain.
    pub name: String,
    /// Number of bones in the chain (from tip bone going up the hierarchy).
    pub chain_length: u8,
    /// IK target configuration (end effector target).
    pub target: IkTargetConfig,
    /// Pole target configuration (optional, for controlling bend direction).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pole: Option<PoleConfig>,
    /// IK constraint influence (0.0 = pure FK, 1.0 = pure IK).
    #[serde(default = "default_influence")]
    pub influence: f64,
}

fn default_influence() -> f64 {
    1.0
}

impl IkChain {
    /// Creates a new IK chain with minimal configuration.
    pub fn new(name: impl Into<String>, chain_length: u8, target: IkTargetConfig) -> Self {
        Self {
            name: name.into(),
            chain_length,
            target,
            pole: None,
            influence: 1.0,
        }
    }

    /// Adds a pole target to this chain.
    pub fn with_pole(mut self, pole: PoleConfig) -> Self {
        self.pole = Some(pole);
        self
    }

    /// Sets the IK influence.
    pub fn with_influence(mut self, influence: f64) -> Self {
        self.influence = influence.clamp(0.0, 1.0);
        self
    }

    /// Validates the IK chain configuration.
    pub fn validate(&self) -> Result<(), IkChainError> {
        if self.name.is_empty() {
            return Err(IkChainError::EmptyName);
        }
        if self.chain_length == 0 {
            return Err(IkChainError::InvalidChainLength);
        }
        if self.target.name.is_empty() {
            return Err(IkChainError::EmptyTargetName);
        }
        if self.target.position.is_some() && self.target.bone.is_some() {
            return Err(IkChainError::ConflictingTargetConfig);
        }
        if let Some(pole) = &self.pole {
            if pole.name.is_empty() {
                return Err(IkChainError::EmptyPoleName);
            }
            if pole.position.is_some() && pole.bone.is_some() {
                return Err(IkChainError::ConflictingPoleConfig);
            }
        }
        Ok(())
    }
}

/// Errors that can occur when validating an IK chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IkChainError {
    /// Chain name is empty.
    EmptyName,
    /// Chain length must be at least 1.
    InvalidChainLength,
    /// Target name is empty.
    EmptyTargetName,
    /// Target has both position and bone specified.
    ConflictingTargetConfig,
    /// Pole target name is empty.
    EmptyPoleName,
    /// Pole has both position and bone specified.
    ConflictingPoleConfig,
}

impl std::fmt::Display for IkChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IkChainError::EmptyName => write!(f, "IK chain name cannot be empty"),
            IkChainError::InvalidChainLength => write!(f, "IK chain length must be at least 1"),
            IkChainError::EmptyTargetName => write!(f, "IK target name cannot be empty"),
            IkChainError::ConflictingTargetConfig => {
                write!(f, "IK target cannot have both position and bone specified")
            }
            IkChainError::EmptyPoleName => write!(f, "Pole target name cannot be empty"),
            IkChainError::ConflictingPoleConfig => {
                write!(f, "Pole target cannot have both position and bone specified")
            }
        }
    }
}

impl std::error::Error for IkChainError {}

// =============================================================================
// IK Preset Setup Functions
// =============================================================================

/// Creates IK chains for humanoid legs.
/// Returns chains for left and right legs.
pub fn setup_humanoid_legs() -> Vec<IkChain> {
    vec![
        IkChain::new(
            "ik_leg_l",
            2,
            IkTargetConfig::new("ik_foot_l"),
        )
        .with_pole(PoleConfig::at_position("pole_knee_l", [0.1, 0.3, 0.5])),
        IkChain::new(
            "ik_leg_r",
            2,
            IkTargetConfig::new("ik_foot_r"),
        )
        .with_pole(PoleConfig::at_position("pole_knee_r", [-0.1, 0.3, 0.5])),
    ]
}

/// Creates IK chains for humanoid arms.
/// Returns chains for left and right arms.
pub fn setup_humanoid_arms() -> Vec<IkChain> {
    vec![
        IkChain::new(
            "ik_arm_l",
            2,
            IkTargetConfig::new("ik_hand_l"),
        )
        .with_pole(PoleConfig::at_position("pole_elbow_l", [0.45, -0.3, 1.35])),
        IkChain::new(
            "ik_arm_r",
            2,
            IkTargetConfig::new("ik_hand_r"),
        )
        .with_pole(PoleConfig::at_position("pole_elbow_r", [-0.45, -0.3, 1.35])),
    ]
}

/// Creates IK chains for quadruped forelegs.
/// Returns chains for left and right forelegs.
pub fn setup_quadruped_forelegs() -> Vec<IkChain> {
    vec![
        IkChain::new(
            "ik_foreleg_l",
            2,
            IkTargetConfig::new("ik_front_paw_l"),
        )
        .with_pole(PoleConfig::at_position("pole_front_knee_l", [0.15, 0.3, 0.0])),
        IkChain::new(
            "ik_foreleg_r",
            2,
            IkTargetConfig::new("ik_front_paw_r"),
        )
        .with_pole(PoleConfig::at_position("pole_front_knee_r", [-0.15, 0.3, 0.0])),
    ]
}

/// Creates IK chains for quadruped hindlegs.
/// Returns chains for left and right hindlegs.
pub fn setup_quadruped_hindlegs() -> Vec<IkChain> {
    vec![
        IkChain::new(
            "ik_hindleg_l",
            2,
            IkTargetConfig::new("ik_back_paw_l"),
        )
        .with_pole(PoleConfig::at_position("pole_back_knee_l", [0.15, -0.3, 0.0])),
        IkChain::new(
            "ik_hindleg_r",
            2,
            IkTargetConfig::new("ik_back_paw_r"),
        )
        .with_pole(PoleConfig::at_position("pole_back_knee_r", [-0.15, -0.3, 0.0])),
    ]
}

/// Creates an IK chain for a tentacle.
/// Tentacles use longer chains without pole targets.
pub fn setup_tentacle(name: &str, chain_length: u8) -> IkChain {
    IkChain::new(
        format!("ik_{}", name),
        chain_length.max(2),
        IkTargetConfig::new(format!("ik_{}_tip", name)),
    )
}

/// Creates an IK chain for a tail.
/// Tails use multi-bone chains without pole targets.
pub fn setup_tail(chain_length: u8) -> IkChain {
    IkChain::new(
        "ik_tail",
        chain_length.max(2),
        IkTargetConfig::new("ik_tail_tip"),
    )
}

/// Creates IK chains for a given preset.
pub fn setup_ik_preset(preset: IkPreset) -> Vec<IkChain> {
    match preset {
        IkPreset::HumanoidLegs => setup_humanoid_legs(),
        IkPreset::HumanoidArms => setup_humanoid_arms(),
        IkPreset::QuadrupedForelegs => setup_quadruped_forelegs(),
        IkPreset::QuadrupedHindlegs => setup_quadruped_hindlegs(),
        IkPreset::Tentacle => vec![setup_tentacle("tentacle", 4)],
        IkPreset::Tail => vec![setup_tail(4)],
    }
}

// =============================================================================
// Bone Constraint Types
// =============================================================================

/// Axis specification for constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ConstraintAxis {
    /// X axis (pitch).
    #[default]
    X,
    /// Y axis (yaw).
    Y,
    /// Z axis (roll).
    Z,
}

impl ConstraintAxis {
    /// Returns the axis name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ConstraintAxis::X => "X",
            ConstraintAxis::Y => "Y",
            ConstraintAxis::Z => "Z",
        }
    }
}

/// Bone constraint types for limiting bone rotations.
///
/// These constraints map to Blender's constraint system and are used to create
/// realistic joint limits for skeletal animations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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

// =============================================================================
// Rig Setup Configuration
// =============================================================================

// =============================================================================
// Foot System Configuration
// =============================================================================

/// Configuration for an IK foot roll system.
/// Provides automatic heel-toe roll during foot plants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

// =============================================================================
// Aim Constraint Configuration
// =============================================================================

/// Aim axis options for aim constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum AimAxis {
    /// Positive X axis.
    #[default]
    #[serde(rename = "X")]
    PosX,
    /// Negative X axis.
    #[serde(rename = "-X")]
    NegX,
    /// Positive Y axis.
    #[serde(rename = "Y")]
    PosY,
    /// Negative Y axis.
    #[serde(rename = "-Y")]
    NegY,
    /// Positive Z axis.
    #[serde(rename = "Z")]
    PosZ,
    /// Negative Z axis.
    #[serde(rename = "-Z")]
    NegZ,
}

impl AimAxis {
    /// Returns the Blender track axis name.
    pub fn blender_track_axis(&self) -> &'static str {
        match self {
            AimAxis::PosX => "TRACK_X",
            AimAxis::NegX => "TRACK_NEGATIVE_X",
            AimAxis::PosY => "TRACK_Y",
            AimAxis::NegY => "TRACK_NEGATIVE_Y",
            AimAxis::PosZ => "TRACK_Z",
            AimAxis::NegZ => "TRACK_NEGATIVE_Z",
        }
    }
}

/// Configuration for an aim (look-at) constraint.
/// Makes a bone always point toward a target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

// =============================================================================
// Stretch Settings
// =============================================================================

/// Stretch settings for IK chains.
/// Allows bones to stretch beyond their rest length.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StretchSettings {
    /// Whether stretch is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Maximum stretch factor (1.0 = no stretch, 2.0 = double length).
    #[serde(default = "default_max_stretch")]
    pub max_stretch: f64,
    /// Minimum stretch factor (0.5 = half length).
    #[serde(default = "default_min_stretch")]
    pub min_stretch: f64,
    /// Volume preservation mode.
    #[serde(default)]
    pub volume_preservation: VolumePreservation,
}

fn default_max_stretch() -> f64 {
    1.5
}

fn default_min_stretch() -> f64 {
    0.5
}

/// Volume preservation mode for stretch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumePreservation {
    /// No volume preservation.
    #[default]
    None,
    /// Preserve volume by scaling perpendicular axes.
    Uniform,
    /// Preserve volume on X axis.
    X,
    /// Preserve volume on Z axis.
    Z,
}

impl StretchSettings {
    /// Creates new stretch settings with stretch enabled.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            max_stretch: default_max_stretch(),
            min_stretch: default_min_stretch(),
            volume_preservation: VolumePreservation::default(),
        }
    }

    /// Sets the stretch limits.
    pub fn with_limits(mut self, min: f64, max: f64) -> Self {
        self.min_stretch = min.max(0.0);
        self.max_stretch = max.max(min);
        self
    }

    /// Sets the volume preservation mode.
    pub fn with_volume_preservation(mut self, mode: VolumePreservation) -> Self {
        self.volume_preservation = mode;
        self
    }
}

// =============================================================================
// Bake Settings
// =============================================================================

/// Settings for baking animation to keyframes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BakeSettings {
    /// Simplify curves after baking (remove redundant keyframes).
    #[serde(default = "default_true")]
    pub simplify: bool,
    /// Start frame for baking (None = use scene start).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_frame: Option<i32>,
    /// End frame for baking (None = use scene end).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_frame: Option<i32>,
    /// Use visual keying (bake visual transforms, not raw values).
    #[serde(default = "default_true")]
    pub visual_keying: bool,
    /// Clear constraints after baking.
    #[serde(default = "default_true")]
    pub clear_constraints: bool,
    /// Frame step for baking (1 = every frame).
    #[serde(default = "default_frame_step")]
    pub frame_step: u32,
    /// Tolerance for curve simplification.
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
    /// Remove IK control bones after baking.
    #[serde(default = "default_true")]
    pub remove_ik_bones: bool,
}

fn default_frame_step() -> u32 {
    1
}

fn default_tolerance() -> f64 {
    0.001
}

impl Default for BakeSettings {
    fn default() -> Self {
        Self {
            simplify: true,
            start_frame: None,
            end_frame: None,
            visual_keying: true,
            clear_constraints: true,
            frame_step: default_frame_step(),
            tolerance: default_tolerance(),
            remove_ik_bones: true,
        }
    }
}

impl BakeSettings {
    /// Creates new bake settings with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the frame range for baking.
    pub fn with_frame_range(mut self, start: i32, end: i32) -> Self {
        self.start_frame = Some(start);
        self.end_frame = Some(end);
        self
    }

    /// Sets the frame step.
    pub fn with_frame_step(mut self, step: u32) -> Self {
        self.frame_step = step.max(1);
        self
    }

    /// Sets whether to simplify curves.
    pub fn with_simplify(mut self, simplify: bool) -> Self {
        self.simplify = simplify;
        self
    }

    /// Sets the simplification tolerance.
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance.abs();
        self
    }

    /// Sets whether to clear constraints after baking.
    pub fn with_clear_constraints(mut self, clear: bool) -> Self {
        self.clear_constraints = clear;
        self
    }

    /// Sets whether to use visual keying.
    pub fn with_visual_keying(mut self, visual: bool) -> Self {
        self.visual_keying = visual;
        self
    }
}

/// Complete rig setup configuration for an armature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RigSetup {
    /// IK presets to apply.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub presets: Vec<IkPreset>,
    /// Custom IK chains (in addition to or overriding presets).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ik_chains: Vec<IkChain>,
    /// Bone constraints for joint limits.
    #[serde(default, skip_serializing_if = "ConstraintConfig::is_empty")]
    pub constraints: ConstraintConfig,
    /// Foot roll systems for IK feet.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub foot_systems: Vec<FootSystem>,
    /// Aim (look-at) constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aim_constraints: Vec<AimConstraint>,
    /// Twist bone setups.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub twist_bones: Vec<TwistBone>,
    /// Stretch settings for IK chains.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stretch: Option<StretchSettings>,
    /// Bake settings for animation export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bake: Option<BakeSettings>,
}

impl RigSetup {
    /// Creates a new empty rig setup.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a preset to the rig setup.
    pub fn with_preset(mut self, preset: IkPreset) -> Self {
        self.presets.push(preset);
        self
    }

    /// Adds a custom IK chain to the rig setup.
    pub fn with_chain(mut self, chain: IkChain) -> Self {
        self.ik_chains.push(chain);
        self
    }

    /// Adds a bone constraint to the rig setup.
    pub fn with_constraint(mut self, constraint: BoneConstraint) -> Self {
        self.constraints.constraints.push(constraint);
        self
    }

    /// Adds a foot system to the rig setup.
    pub fn with_foot_system(mut self, foot_system: FootSystem) -> Self {
        self.foot_systems.push(foot_system);
        self
    }

    /// Adds an aim constraint to the rig setup.
    pub fn with_aim_constraint(mut self, aim_constraint: AimConstraint) -> Self {
        self.aim_constraints.push(aim_constraint);
        self
    }

    /// Adds a twist bone setup to the rig setup.
    pub fn with_twist_bone(mut self, twist_bone: TwistBone) -> Self {
        self.twist_bones.push(twist_bone);
        self
    }

    /// Sets the stretch settings.
    pub fn with_stretch(mut self, stretch: StretchSettings) -> Self {
        self.stretch = Some(stretch);
        self
    }

    /// Sets the bake settings.
    pub fn with_bake(mut self, bake: BakeSettings) -> Self {
        self.bake = Some(bake);
        self
    }

    /// Expands all presets into IK chains.
    pub fn expand_chains(&self) -> Vec<IkChain> {
        let mut chains: Vec<IkChain> = self
            .presets
            .iter()
            .flat_map(|p| setup_ik_preset(*p))
            .collect();
        chains.extend(self.ik_chains.clone());
        chains
    }

    /// Validates all chains in the rig setup.
    pub fn validate(&self) -> Result<(), IkChainError> {
        for chain in self.expand_chains() {
            chain.validate()?;
        }
        Ok(())
    }

    /// Validates all constraints in the rig setup.
    pub fn validate_constraints(&self) -> Result<(), BoneConstraintError> {
        self.constraints.validate()
    }
}

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
    /// Save .blend file alongside GLB output.
    #[serde(default)]
    pub save_blend: bool,
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
            save_blend: false,
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

// =============================================================================
// Procedural Animation Layers
// =============================================================================

/// Types of procedural animation layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProceduralLayerType {
    /// Breathing animation (subtle chest/torso expansion).
    Breathing,
    /// Swaying motion (side-to-side idle motion).
    Sway,
    /// Bobbing motion (up-down motion, e.g., for floating).
    Bob,
    /// Noise-based random motion.
    Noise,
}

/// Rotation axis for procedural layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProceduralAxis {
    /// Pitch rotation (X axis).
    #[default]
    Pitch,
    /// Yaw rotation (Y axis).
    Yaw,
    /// Roll rotation (Z axis).
    Roll,
}

impl ProceduralAxis {
    /// Converts to a rotation axis index (0=X, 1=Y, 2=Z).
    pub fn to_index(&self) -> usize {
        match self {
            ProceduralAxis::Pitch => 0,
            ProceduralAxis::Yaw => 1,
            ProceduralAxis::Roll => 2,
        }
    }
}

/// Procedural animation layer configuration.
/// Adds automatic motion overlays to bones.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProceduralLayer {
    /// Type of procedural animation.
    #[serde(rename = "type")]
    pub layer_type: ProceduralLayerType,
    /// Target bone name.
    pub target: String,
    /// Rotation axis for the motion.
    #[serde(default)]
    pub axis: ProceduralAxis,
    /// Period in frames for sine-based animations.
    #[serde(default = "default_period_frames")]
    pub period_frames: u32,
    /// Amplitude of the motion (in radians for sine, degrees for noise).
    #[serde(default = "default_amplitude")]
    pub amplitude: f64,
    /// Phase offset (0.0-1.0) for staggered animations.
    #[serde(default)]
    pub phase_offset: f64,
    /// Frequency for noise-based animations.
    #[serde(default = "default_frequency")]
    pub frequency: f64,
}

fn default_period_frames() -> u32 {
    60
}

fn default_amplitude() -> f64 {
    0.01
}

fn default_frequency() -> f64 {
    0.3
}

impl ProceduralLayer {
    /// Creates a new breathing layer.
    pub fn breathing(target: impl Into<String>) -> Self {
        Self {
            layer_type: ProceduralLayerType::Breathing,
            target: target.into(),
            axis: ProceduralAxis::Pitch,
            period_frames: 90, // ~3 seconds at 30fps
            amplitude: 0.02,
            phase_offset: 0.0,
            frequency: default_frequency(),
        }
    }

    /// Creates a new sway layer.
    pub fn sway(target: impl Into<String>) -> Self {
        Self {
            layer_type: ProceduralLayerType::Sway,
            target: target.into(),
            axis: ProceduralAxis::Roll,
            period_frames: 120, // ~4 seconds at 30fps
            amplitude: 0.03,
            phase_offset: 0.0,
            frequency: default_frequency(),
        }
    }

    /// Creates a new bob layer.
    pub fn bob(target: impl Into<String>) -> Self {
        Self {
            layer_type: ProceduralLayerType::Bob,
            target: target.into(),
            axis: ProceduralAxis::Pitch,
            period_frames: 60,
            amplitude: 0.02,
            phase_offset: 0.0,
            frequency: default_frequency(),
        }
    }

    /// Creates a new noise layer.
    pub fn noise(target: impl Into<String>) -> Self {
        Self {
            layer_type: ProceduralLayerType::Noise,
            target: target.into(),
            axis: ProceduralAxis::Roll,
            period_frames: default_period_frames(),
            amplitude: 1.0, // degrees
            phase_offset: 0.0,
            frequency: 0.3,
        }
    }

    /// Sets the rotation axis.
    pub fn with_axis(mut self, axis: ProceduralAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Sets the period in frames.
    pub fn with_period(mut self, frames: u32) -> Self {
        self.period_frames = frames.max(1);
        self
    }

    /// Sets the amplitude.
    pub fn with_amplitude(mut self, amplitude: f64) -> Self {
        self.amplitude = amplitude;
        self
    }

    /// Sets the phase offset.
    pub fn with_phase_offset(mut self, offset: f64) -> Self {
        self.phase_offset = offset;
        self
    }

    /// Sets the frequency for noise layers.
    pub fn with_frequency(mut self, frequency: f64) -> Self {
        self.frequency = frequency.max(0.0);
        self
    }
}

// =============================================================================
// Pose and Phase System
// =============================================================================

/// Timing curve types for phase interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimingCurve {
    /// Linear interpolation.
    #[default]
    Linear,
    /// Ease in (slow start).
    EaseIn,
    /// Ease out (slow end).
    EaseOut,
    /// Ease in and out (slow start and end).
    EaseInOut,
    /// Exponential ease in.
    ExponentialIn,
    /// Exponential ease out.
    ExponentialOut,
    /// Constant (no interpolation, snap).
    Constant,
}

/// A named pose definition containing bone rotations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoseDefinition {
    /// Bone transforms in this pose.
    /// Keys are bone names, values are the rotations.
    pub bones: std::collections::HashMap<String, PoseBoneTransform>,
}

impl PoseDefinition {
    /// Creates a new empty pose definition.
    pub fn new() -> Self {
        Self {
            bones: std::collections::HashMap::new(),
        }
    }

    /// Adds a bone transform to this pose.
    pub fn with_bone(mut self, name: impl Into<String>, transform: PoseBoneTransform) -> Self {
        self.bones.insert(name.into(), transform);
        self
    }
}

impl Default for PoseDefinition {
    fn default() -> Self {
        Self::new()
    }
}

/// Bone transform within a pose.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PoseBoneTransform {
    /// Pitch rotation in degrees (X axis).
    #[serde(default)]
    pub pitch: f64,
    /// Yaw rotation in degrees (Y axis).
    #[serde(default)]
    pub yaw: f64,
    /// Roll rotation in degrees (Z axis).
    #[serde(default)]
    pub roll: f64,
    /// Location offset [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<[f64; 3]>,
}

impl PoseBoneTransform {
    /// Creates a new pose bone transform with rotation.
    pub fn rotation(pitch: f64, yaw: f64, roll: f64) -> Self {
        Self {
            pitch,
            yaw,
            roll,
            location: None,
        }
    }

    /// Creates a new pose bone transform with only pitch.
    pub fn pitch(angle: f64) -> Self {
        Self::rotation(angle, 0.0, 0.0)
    }

    /// Creates a new pose bone transform with only yaw.
    pub fn yaw(angle: f64) -> Self {
        Self::rotation(0.0, angle, 0.0)
    }

    /// Creates a new pose bone transform with only roll.
    pub fn roll(angle: f64) -> Self {
        Self::rotation(0.0, 0.0, angle)
    }

    /// Sets the location offset.
    pub fn with_location(mut self, location: [f64; 3]) -> Self {
        self.location = Some(location);
        self
    }

    /// Returns the rotation as an array [pitch, yaw, roll] in degrees.
    pub fn as_euler_degrees(&self) -> [f64; 3] {
        [self.pitch, self.yaw, self.roll]
    }
}

/// IK target keyframe within a phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseIkTarget {
    /// Frame number for this keyframe.
    pub frame: i32,
    /// World position [X, Y, Z].
    pub location: [f64; 3],
    /// IK/FK blend value (0.0 = FK, 1.0 = IK).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ikfk: Option<f64>,
}

impl PhaseIkTarget {
    /// Creates a new IK target keyframe.
    pub fn new(frame: i32, location: [f64; 3]) -> Self {
        Self {
            frame,
            location,
            ikfk: None,
        }
    }

    /// Sets the IK/FK blend value.
    pub fn with_ikfk(mut self, blend: f64) -> Self {
        self.ikfk = Some(blend.clamp(0.0, 1.0));
        self
    }
}

/// Animation phase definition.
/// Defines a segment of the animation with timing and pose/IK targets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationPhase {
    /// Name of this phase (e.g., "contact", "passing", "lift").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Start frame of this phase.
    pub start_frame: i32,
    /// End frame of this phase.
    pub end_frame: i32,
    /// Timing curve for interpolation.
    #[serde(default)]
    pub curve: TimingCurve,
    /// Named pose to apply during this phase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pose: Option<String>,
    /// IK target keyframes for this phase.
    /// Keys are IK chain names, values are lists of keyframes.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub ik_targets: std::collections::HashMap<String, Vec<PhaseIkTarget>>,
    /// IK/FK blend keyframes for this phase.
    /// Keys are IK chain names, values are lists of (frame, blend) pairs.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub ikfk_blend: std::collections::HashMap<String, Vec<(i32, f64)>>,
}

impl AnimationPhase {
    /// Creates a new animation phase.
    pub fn new(start_frame: i32, end_frame: i32) -> Self {
        Self {
            name: None,
            start_frame,
            end_frame,
            curve: TimingCurve::Linear,
            pose: None,
            ik_targets: std::collections::HashMap::new(),
            ikfk_blend: std::collections::HashMap::new(),
        }
    }

    /// Sets the phase name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the timing curve.
    pub fn with_curve(mut self, curve: TimingCurve) -> Self {
        self.curve = curve;
        self
    }

    /// Sets the pose to apply.
    pub fn with_pose(mut self, pose: impl Into<String>) -> Self {
        self.pose = Some(pose.into());
        self
    }

    /// Adds IK target keyframes for a chain.
    pub fn with_ik_targets(mut self, chain: impl Into<String>, targets: Vec<PhaseIkTarget>) -> Self {
        self.ik_targets.insert(chain.into(), targets);
        self
    }

    /// Returns the duration in frames.
    pub fn duration_frames(&self) -> i32 {
        self.end_frame - self.start_frame
    }
}

// =============================================================================
// Rigged Animation Recipe (v2 with IK support)
// =============================================================================

/// Parameters for the `skeletal_animation.blender_rigged_v1` recipe.
/// This is the IK-enabled version of the animation recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletalAnimationBlenderRiggedV1Params {
    /// Skeleton rig to animate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton_preset: Option<SkeletonPreset>,
    /// Name of the animation clip.
    pub clip_name: String,
    /// Path to existing armature file (GLB/GLTF).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_armature: Option<String>,
    /// Character spec reference (alternative to input_armature).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    /// Duration of the animation in frames.
    pub duration_frames: u32,
    /// Duration of the animation in seconds (alternative to duration_frames).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    /// Frames per second.
    #[serde(default = "default_fps")]
    pub fps: u8,
    /// Whether the animation should loop.
    #[serde(default)]
    pub r#loop: bool,
    /// Root bone Z offset (for ground contact).
    #[serde(default)]
    pub ground_offset: f64,
    /// Rig setup configuration (IK chains, presets).
    #[serde(default)]
    pub rig_setup: RigSetup,
    /// Named pose definitions.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub poses: std::collections::HashMap<String, PoseDefinition>,
    /// Animation phases with timing and IK targets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phases: Vec<AnimationPhase>,
    /// Procedural animation layers (breathing, sway, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub procedural_layers: Vec<ProceduralLayer>,
    /// Keyframe definitions (for FK animation or IK target animation).
    #[serde(default)]
    pub keyframes: Vec<AnimationKeyframe>,
    /// IK target keyframes (for animating IK targets).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ik_keyframes: Vec<IkKeyframe>,
    /// Interpolation method between keyframes.
    #[serde(default)]
    pub interpolation: InterpolationMode,
    /// Export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<AnimationExportSettings>,
    /// Animator rig configuration (visual aids for animators).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animator_rig: Option<AnimatorRigConfig>,
    /// Save .blend file alongside output.
    #[serde(default)]
    pub save_blend: bool,
    /// Validation conventions configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conventions: Option<ConventionsConfig>,
}

fn default_fps() -> u8 {
    30
}

/// Validation conventions configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConventionsConfig {
    /// Fail on validation errors (strict mode).
    #[serde(default)]
    pub strict: bool,
}

// =============================================================================
// Animator Rig Configuration
// =============================================================================

/// Widget shape styles for bone visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetStyle {
    /// Wireframe circle (default, good for rotation controls).
    #[default]
    WireCircle,
    /// Wireframe cube (good for position controls).
    WireCube,
    /// Wireframe sphere (good for ball joints).
    WireSphere,
    /// Wireframe diamond (good for IK targets).
    WireDiamond,
    /// Custom mesh widget (requires external mesh reference).
    CustomMesh,
}

impl WidgetStyle {
    /// Returns the Blender object name for this widget style.
    pub fn blender_name(&self) -> &'static str {
        match self {
            WidgetStyle::WireCircle => "WGT_circle",
            WidgetStyle::WireCube => "WGT_cube",
            WidgetStyle::WireSphere => "WGT_sphere",
            WidgetStyle::WireDiamond => "WGT_diamond",
            WidgetStyle::CustomMesh => "WGT_custom",
        }
    }

    /// Returns all standard widget styles (excluding CustomMesh).
    pub fn standard_styles() -> &'static [WidgetStyle] {
        &[
            WidgetStyle::WireCircle,
            WidgetStyle::WireCube,
            WidgetStyle::WireSphere,
            WidgetStyle::WireDiamond,
        ]
    }
}

/// Bone collection definition for organizing bones in groups.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoneCollection {
    /// Name of the collection (e.g., "IK Controls", "FK Controls").
    pub name: String,
    /// List of bone names in this collection.
    pub bones: Vec<String>,
    /// Whether this collection is visible by default.
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Whether bones in this collection are selectable.
    #[serde(default = "default_true")]
    pub selectable: bool,
}

impl BoneCollection {
    /// Creates a new bone collection with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bones: Vec::new(),
            visible: true,
            selectable: true,
        }
    }

    /// Adds a bone to this collection.
    pub fn with_bone(mut self, bone: impl Into<String>) -> Self {
        self.bones.push(bone.into());
        self
    }

    /// Adds multiple bones to this collection.
    pub fn with_bones(mut self, bones: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.bones.extend(bones.into_iter().map(|b| b.into()));
        self
    }

    /// Sets the visibility of this collection.
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets the selectability of this collection.
    pub fn with_selectability(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Validates the bone collection.
    pub fn validate(&self) -> Result<(), AnimatorRigError> {
        if self.name.is_empty() {
            return Err(AnimatorRigError::EmptyCollectionName);
        }
        Ok(())
    }
}

/// Standard bone collection presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoneCollectionPreset {
    /// IK control bones (targets and poles).
    IkControls,
    /// FK control bones (direct rotation controls).
    FkControls,
    /// Deformation bones (actual skin deformers).
    Deform,
    /// Mechanism bones (helper bones, not for direct animation).
    Mechanism,
}

impl BoneCollectionPreset {
    /// Returns the default name for this preset.
    pub fn default_name(&self) -> &'static str {
        match self {
            BoneCollectionPreset::IkControls => "IK Controls",
            BoneCollectionPreset::FkControls => "FK Controls",
            BoneCollectionPreset::Deform => "Deform",
            BoneCollectionPreset::Mechanism => "Mechanism",
        }
    }

    /// Returns whether bones in this collection should be visible by default.
    pub fn default_visibility(&self) -> bool {
        match self {
            BoneCollectionPreset::IkControls => true,
            BoneCollectionPreset::FkControls => true,
            BoneCollectionPreset::Deform => false,
            BoneCollectionPreset::Mechanism => false,
        }
    }

    /// Returns whether bones in this collection should be selectable by default.
    pub fn default_selectability(&self) -> bool {
        match self {
            BoneCollectionPreset::IkControls => true,
            BoneCollectionPreset::FkControls => true,
            BoneCollectionPreset::Deform => false,
            BoneCollectionPreset::Mechanism => false,
        }
    }

    /// Creates a bone collection from this preset.
    pub fn to_collection(&self) -> BoneCollection {
        BoneCollection {
            name: self.default_name().to_string(),
            bones: Vec::new(),
            visible: self.default_visibility(),
            selectable: self.default_selectability(),
        }
    }
}

/// RGB color value for bone coloring (0.0-1.0 range).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoneColor {
    /// Red component (0.0-1.0).
    pub r: f64,
    /// Green component (0.0-1.0).
    pub g: f64,
    /// Blue component (0.0-1.0).
    pub b: f64,
}

impl BoneColor {
    /// Creates a new bone color from RGB values.
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
        }
    }

    /// Standard blue color for left-side bones.
    pub fn left_blue() -> Self {
        Self::new(0.2, 0.4, 1.0)
    }

    /// Standard red color for right-side bones.
    pub fn right_red() -> Self {
        Self::new(1.0, 0.3, 0.3)
    }

    /// Standard yellow color for center bones.
    pub fn center_yellow() -> Self {
        Self::new(1.0, 0.9, 0.2)
    }

    /// White color (default/neutral).
    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    /// Returns the color as an array [R, G, B].
    pub fn as_array(&self) -> [f64; 3] {
        [self.r, self.g, self.b]
    }
}

impl Default for BoneColor {
    fn default() -> Self {
        Self::white()
    }
}

/// Bone color scheme for automatic bone coloring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "scheme", rename_all = "snake_case")]
pub enum BoneColorScheme {
    /// Standard scheme: left=blue, right=red, center=yellow.
    Standard,
    /// Custom color mapping by bone side.
    Custom {
        /// Color for left-side bones (suffix `_l` or `_L`).
        left: BoneColor,
        /// Color for right-side bones (suffix `_r` or `_R`).
        right: BoneColor,
        /// Color for center bones (no side suffix).
        center: BoneColor,
    },
    /// Per-bone custom colors.
    PerBone {
        /// Map of bone name to color.
        colors: std::collections::HashMap<String, BoneColor>,
        /// Default color for bones not in the map.
        #[serde(default)]
        default: BoneColor,
    },
}

impl Default for BoneColorScheme {
    fn default() -> Self {
        BoneColorScheme::Standard
    }
}

impl BoneColorScheme {
    /// Creates the standard color scheme (L=blue, R=red, center=yellow).
    pub fn standard() -> Self {
        BoneColorScheme::Standard
    }

    /// Creates a custom color scheme with specified colors.
    pub fn custom(left: BoneColor, right: BoneColor, center: BoneColor) -> Self {
        BoneColorScheme::Custom { left, right, center }
    }

    /// Returns the color for a given bone name based on the scheme.
    pub fn color_for_bone(&self, bone_name: &str) -> BoneColor {
        match self {
            BoneColorScheme::Standard => {
                if bone_name.ends_with("_l") || bone_name.ends_with("_L") {
                    BoneColor::left_blue()
                } else if bone_name.ends_with("_r") || bone_name.ends_with("_R") {
                    BoneColor::right_red()
                } else {
                    BoneColor::center_yellow()
                }
            }
            BoneColorScheme::Custom { left, right, center } => {
                if bone_name.ends_with("_l") || bone_name.ends_with("_L") {
                    *left
                } else if bone_name.ends_with("_r") || bone_name.ends_with("_R") {
                    *right
                } else {
                    *center
                }
            }
            BoneColorScheme::PerBone { colors, default } => {
                colors.get(bone_name).copied().unwrap_or(*default)
            }
        }
    }
}

/// Armature display type in Blender viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArmatureDisplay {
    /// Octahedral bone shapes (default).
    #[default]
    Octahedral,
    /// Stick bone shapes.
    Stick,
    /// B-Bone (bendy bone) shapes.
    Bbone,
    /// Envelope shapes.
    Envelope,
    /// Wire shapes.
    Wire,
}

impl ArmatureDisplay {
    /// Returns the Blender enum name for this display type.
    pub fn blender_name(&self) -> &'static str {
        match self {
            ArmatureDisplay::Octahedral => "OCTAHEDRAL",
            ArmatureDisplay::Stick => "STICK",
            ArmatureDisplay::Bbone => "BBONE",
            ArmatureDisplay::Envelope => "ENVELOPE",
            ArmatureDisplay::Wire => "WIRE",
        }
    }
}

/// Configuration for animator rig visual aids.
///
/// This configuration controls how the rig appears to animators in Blender,
/// including bone collections, custom shapes, and color coding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimatorRigConfig {
    /// Whether to organize bones into collections.
    #[serde(default = "default_true")]
    pub collections: bool,
    /// Whether to add custom bone shapes (widgets).
    #[serde(default = "default_true")]
    pub shapes: bool,
    /// Whether to apply color coding to bones.
    #[serde(default = "default_true")]
    pub colors: bool,
    /// Armature display type in viewport.
    #[serde(default)]
    pub display: ArmatureDisplay,
    /// Widget style for control bones.
    #[serde(default)]
    pub widget_style: WidgetStyle,
    /// Custom bone collections (in addition to or replacing defaults).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bone_collections: Vec<BoneCollection>,
    /// Bone color scheme.
    #[serde(default)]
    pub bone_colors: BoneColorScheme,
}

impl Default for AnimatorRigConfig {
    fn default() -> Self {
        Self {
            collections: true,
            shapes: true,
            colors: true,
            display: ArmatureDisplay::default(),
            widget_style: WidgetStyle::default(),
            bone_collections: Vec::new(),
            bone_colors: BoneColorScheme::default(),
        }
    }
}

impl AnimatorRigConfig {
    /// Creates a new animator rig config with all features enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a minimal config with no visual aids.
    pub fn minimal() -> Self {
        Self {
            collections: false,
            shapes: false,
            colors: false,
            display: ArmatureDisplay::Octahedral,
            widget_style: WidgetStyle::WireCircle,
            bone_collections: Vec::new(),
            bone_colors: BoneColorScheme::Standard,
        }
    }

    /// Sets whether to organize bones into collections.
    pub fn with_collections(mut self, enabled: bool) -> Self {
        self.collections = enabled;
        self
    }

    /// Sets whether to add custom bone shapes.
    pub fn with_shapes(mut self, enabled: bool) -> Self {
        self.shapes = enabled;
        self
    }

    /// Sets whether to apply color coding.
    pub fn with_colors(mut self, enabled: bool) -> Self {
        self.colors = enabled;
        self
    }

    /// Sets the armature display type.
    pub fn with_display(mut self, display: ArmatureDisplay) -> Self {
        self.display = display;
        self
    }

    /// Sets the widget style for control bones.
    pub fn with_widget_style(mut self, style: WidgetStyle) -> Self {
        self.widget_style = style;
        self
    }

    /// Adds a bone collection.
    pub fn with_bone_collection(mut self, collection: BoneCollection) -> Self {
        self.bone_collections.push(collection);
        self
    }

    /// Sets the bone color scheme.
    pub fn with_bone_colors(mut self, scheme: BoneColorScheme) -> Self {
        self.bone_colors = scheme;
        self
    }

    /// Validates the animator rig configuration.
    pub fn validate(&self) -> Result<(), AnimatorRigError> {
        for collection in &self.bone_collections {
            collection.validate()?;
        }
        Ok(())
    }

    /// Creates default bone collections for a humanoid rig.
    pub fn default_humanoid_collections() -> Vec<BoneCollection> {
        vec![
            BoneCollectionPreset::IkControls.to_collection()
                .with_bones(["ik_foot_l", "ik_foot_r", "ik_hand_l", "ik_hand_r",
                            "pole_knee_l", "pole_knee_r", "pole_elbow_l", "pole_elbow_r"]),
            BoneCollectionPreset::FkControls.to_collection()
                .with_bones(["root", "hips", "spine", "chest", "neck", "head",
                            "shoulder_l", "shoulder_r"]),
            BoneCollectionPreset::Deform.to_collection()
                .with_bones(["upper_arm_l", "lower_arm_l", "hand_l",
                            "upper_arm_r", "lower_arm_r", "hand_r",
                            "upper_leg_l", "lower_leg_l", "foot_l",
                            "upper_leg_r", "lower_leg_r", "foot_r"]),
            BoneCollectionPreset::Mechanism.to_collection(),
        ]
    }
}

/// Errors that can occur when validating animator rig configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimatorRigError {
    /// Bone collection name is empty.
    EmptyCollectionName,
    /// Duplicate collection name.
    DuplicateCollectionName(String),
    /// Invalid widget style for bone type.
    InvalidWidgetStyle {
        bone: String,
        style: String,
    },
}

impl std::fmt::Display for AnimatorRigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimatorRigError::EmptyCollectionName => {
                write!(f, "Bone collection name cannot be empty")
            }
            AnimatorRigError::DuplicateCollectionName(name) => {
                write!(f, "Duplicate bone collection name: {}", name)
            }
            AnimatorRigError::InvalidWidgetStyle { bone, style } => {
                write!(f, "Invalid widget style '{}' for bone '{}'", style, bone)
            }
        }
    }
}

impl std::error::Error for AnimatorRigError {}

/// Keyframe for IK target animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IkKeyframe {
    /// Time in seconds.
    pub time: f64,
    /// IK target transforms at this keyframe.
    /// Keys are IK chain names (e.g., "ik_leg_l"), values are world positions.
    pub targets: std::collections::HashMap<String, IkTargetTransform>,
}

/// Transform for an IK target at a keyframe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    // =========================================================================
    // IK Chain Tests
    // =========================================================================

    #[test]
    fn test_ik_preset_serde() {
        let preset = IkPreset::HumanoidLegs;
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(json, "\"humanoid_legs\"");

        let parsed: IkPreset = serde_json::from_str("\"humanoid_arms\"").unwrap();
        assert_eq!(parsed, IkPreset::HumanoidArms);

        // Test all variants
        let presets = [
            (IkPreset::HumanoidLegs, "\"humanoid_legs\""),
            (IkPreset::HumanoidArms, "\"humanoid_arms\""),
            (IkPreset::QuadrupedForelegs, "\"quadruped_forelegs\""),
            (IkPreset::QuadrupedHindlegs, "\"quadruped_hindlegs\""),
            (IkPreset::Tentacle, "\"tentacle\""),
            (IkPreset::Tail, "\"tail\""),
        ];
        for (preset, expected) in presets {
            let json = serde_json::to_string(&preset).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_ik_preset_defaults() {
        assert_eq!(IkPreset::HumanoidLegs.default_chain_length(), 2);
        assert_eq!(IkPreset::Tentacle.default_chain_length(), 4);

        assert!(IkPreset::HumanoidLegs.uses_pole_target());
        assert!(!IkPreset::Tentacle.uses_pole_target());

        assert!(IkPreset::HumanoidLegs.default_pole_offset().is_some());
        assert!(IkPreset::Tail.default_pole_offset().is_none());
    }

    #[test]
    fn test_ik_target_config_serde() {
        let target = IkTargetConfig::new("ik_foot_l");
        let json = serde_json::to_string(&target).unwrap();
        assert!(json.contains("ik_foot_l"));

        let target_with_pos = IkTargetConfig::at_position("ik_foot_l", [0.1, 0.0, 0.0]);
        let json = serde_json::to_string(&target_with_pos).unwrap();
        assert!(json.contains("position"));

        let parsed: IkTargetConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "ik_foot_l");
        assert!(parsed.position.is_some());
    }

    #[test]
    fn test_pole_config_serde() {
        let pole = PoleConfig::at_position("pole_knee_l", [0.1, 0.3, 0.5]).with_angle(90.0);
        let json = serde_json::to_string(&pole).unwrap();
        assert!(json.contains("pole_knee_l"));
        assert!(json.contains("angle"));
        assert!(json.contains("90"));

        let parsed: PoleConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "pole_knee_l");
        assert_eq!(parsed.angle, 90.0);
    }

    #[test]
    fn test_ik_chain_serde() {
        let chain = IkChain::new("ik_leg_l", 2, IkTargetConfig::new("ik_foot_l"))
            .with_pole(PoleConfig::at_position("pole_knee_l", [0.1, 0.3, 0.5]))
            .with_influence(0.8);

        let json = serde_json::to_string(&chain).unwrap();
        assert!(json.contains("ik_leg_l"));
        assert!(json.contains("chain_length"));
        assert!(json.contains("pole"));
        assert!(json.contains("influence"));

        let parsed: IkChain = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "ik_leg_l");
        assert_eq!(parsed.chain_length, 2);
        assert!(parsed.pole.is_some());
        assert_eq!(parsed.influence, 0.8);
    }

    #[test]
    fn test_ik_chain_validation() {
        // Valid chain
        let chain = IkChain::new("ik_leg_l", 2, IkTargetConfig::new("ik_foot_l"));
        assert!(chain.validate().is_ok());

        // Empty name
        let chain = IkChain::new("", 2, IkTargetConfig::new("ik_foot_l"));
        assert_eq!(chain.validate(), Err(IkChainError::EmptyName));

        // Zero chain length
        let chain = IkChain::new("ik_leg_l", 0, IkTargetConfig::new("ik_foot_l"));
        assert_eq!(chain.validate(), Err(IkChainError::InvalidChainLength));

        // Empty target name
        let chain = IkChain::new("ik_leg_l", 2, IkTargetConfig::new(""));
        assert_eq!(chain.validate(), Err(IkChainError::EmptyTargetName));

        // Conflicting target config (both position and bone)
        let mut target = IkTargetConfig::new("ik_foot_l");
        target.position = Some([0.0, 0.0, 0.0]);
        target.bone = Some("foot_l".to_string());
        let chain = IkChain::new("ik_leg_l", 2, target);
        assert_eq!(chain.validate(), Err(IkChainError::ConflictingTargetConfig));

        // Empty pole name
        let mut pole = PoleConfig::new("");
        pole.position = Some([0.0, 0.0, 0.0]);
        let chain = IkChain::new("ik_leg_l", 2, IkTargetConfig::new("ik_foot_l")).with_pole(pole);
        assert_eq!(chain.validate(), Err(IkChainError::EmptyPoleName));
    }

    #[test]
    fn test_setup_humanoid_legs() {
        let chains = setup_humanoid_legs();
        assert_eq!(chains.len(), 2);

        let left = &chains[0];
        assert_eq!(left.name, "ik_leg_l");
        assert_eq!(left.chain_length, 2);
        assert!(left.pole.is_some());
        assert!(left.validate().is_ok());

        let right = &chains[1];
        assert_eq!(right.name, "ik_leg_r");
        assert!(right.validate().is_ok());
    }

    #[test]
    fn test_setup_humanoid_arms() {
        let chains = setup_humanoid_arms();
        assert_eq!(chains.len(), 2);

        for chain in &chains {
            assert_eq!(chain.chain_length, 2);
            assert!(chain.pole.is_some());
            assert!(chain.validate().is_ok());
        }
    }

    #[test]
    fn test_setup_ik_preset() {
        // Test all presets create valid chains
        let presets = [
            IkPreset::HumanoidLegs,
            IkPreset::HumanoidArms,
            IkPreset::QuadrupedForelegs,
            IkPreset::QuadrupedHindlegs,
            IkPreset::Tentacle,
            IkPreset::Tail,
        ];

        for preset in presets {
            let chains = setup_ik_preset(preset);
            assert!(!chains.is_empty(), "Preset {:?} should create chains", preset);
            for chain in chains {
                assert!(
                    chain.validate().is_ok(),
                    "Chain {} from preset {:?} should be valid",
                    chain.name,
                    preset
                );
            }
        }
    }

    #[test]
    fn test_rig_setup_serde() {
        let rig_setup = RigSetup::new()
            .with_preset(IkPreset::HumanoidLegs)
            .with_preset(IkPreset::HumanoidArms)
            .with_chain(setup_tail(3));

        let json = serde_json::to_string(&rig_setup).unwrap();
        assert!(json.contains("humanoid_legs"));
        assert!(json.contains("humanoid_arms"));
        assert!(json.contains("ik_tail"));

        let parsed: RigSetup = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.presets.len(), 2);
        assert_eq!(parsed.ik_chains.len(), 1);
    }

    #[test]
    fn test_rig_setup_expand_chains() {
        let rig_setup = RigSetup::new()
            .with_preset(IkPreset::HumanoidLegs)
            .with_chain(setup_tail(3));

        let chains = rig_setup.expand_chains();
        // HumanoidLegs creates 2 chains (left + right), plus 1 custom tail chain
        assert_eq!(chains.len(), 3);

        // Verify all are valid
        assert!(rig_setup.validate().is_ok());
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

    #[test]
    fn test_rigged_params_serde() {
        let params = SkeletalAnimationBlenderRiggedV1Params {
            skeleton_preset: Some(SkeletonPreset::HumanoidBasicV1),
            clip_name: "walk".to_string(),
            input_armature: None,
            character: None,
            duration_frames: 30,
            duration_seconds: Some(1.0),
            fps: 30,
            r#loop: true,
            ground_offset: 0.0,
            rig_setup: RigSetup::new().with_preset(IkPreset::HumanoidLegs),
            poses: std::collections::HashMap::new(),
            phases: vec![],
            procedural_layers: vec![],
            keyframes: vec![],
            ik_keyframes: vec![],
            interpolation: InterpolationMode::Linear,
            export: None,
            animator_rig: None,
            save_blend: false,
            conventions: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("walk"));
        assert!(json.contains("humanoid_legs"));
        assert!(json.contains("humanoid_basic_v1"));

        let parsed: SkeletalAnimationBlenderRiggedV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.clip_name, "walk");
        assert!(parsed.r#loop);
    }

    #[test]
    fn test_rigged_params_with_animator_rig() {
        let params = SkeletalAnimationBlenderRiggedV1Params {
            skeleton_preset: Some(SkeletonPreset::HumanoidBasicV1),
            clip_name: "idle".to_string(),
            input_armature: None,
            character: None,
            duration_frames: 60,
            duration_seconds: Some(2.0),
            fps: 30,
            r#loop: true,
            ground_offset: 0.0,
            rig_setup: RigSetup::new(),
            poses: std::collections::HashMap::new(),
            phases: vec![],
            procedural_layers: vec![],
            keyframes: vec![],
            ik_keyframes: vec![],
            interpolation: InterpolationMode::Linear,
            export: None,
            animator_rig: Some(AnimatorRigConfig::new()
                .with_widget_style(WidgetStyle::WireDiamond)
                .with_bone_colors(BoneColorScheme::Standard)),
            save_blend: false,
            conventions: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("animator_rig"));
        assert!(json.contains("wire_diamond"));

        let parsed: SkeletalAnimationBlenderRiggedV1Params = serde_json::from_str(&json).unwrap();
        assert!(parsed.animator_rig.is_some());
        let rig = parsed.animator_rig.unwrap();
        assert_eq!(rig.widget_style, WidgetStyle::WireDiamond);
    }

    // =========================================================================
    // Bone Constraint Tests
    // =========================================================================

    #[test]
    fn test_constraint_axis_serde() {
        // Test default
        assert_eq!(ConstraintAxis::default(), ConstraintAxis::X);

        // Test all variants serialize correctly
        let axes = [
            (ConstraintAxis::X, "\"X\""),
            (ConstraintAxis::Y, "\"Y\""),
            (ConstraintAxis::Z, "\"Z\""),
        ];
        for (axis, expected) in axes {
            let json = serde_json::to_string(&axis).unwrap();
            assert_eq!(json, expected);
        }

        // Test as_str method
        assert_eq!(ConstraintAxis::X.as_str(), "X");
        assert_eq!(ConstraintAxis::Y.as_str(), "Y");
        assert_eq!(ConstraintAxis::Z.as_str(), "Z");
    }

    #[test]
    fn test_hinge_constraint_serde() {
        let constraint = BoneConstraint::hinge("lower_arm_l", ConstraintAxis::X, 0.0, 160.0);

        let json = serde_json::to_string(&constraint).unwrap();
        assert!(json.contains("\"type\":\"hinge\""));
        assert!(json.contains("\"bone\":\"lower_arm_l\""));
        assert!(json.contains("\"axis\":\"X\""));
        assert!(json.contains("\"min_angle\":0.0"));
        assert!(json.contains("\"max_angle\":160.0"));

        let parsed: BoneConstraint = serde_json::from_str(&json).unwrap();
        if let BoneConstraint::Hinge { bone, axis, min_angle, max_angle } = parsed {
            assert_eq!(bone, "lower_arm_l");
            assert_eq!(axis, ConstraintAxis::X);
            assert_eq!(min_angle, 0.0);
            assert_eq!(max_angle, 160.0);
        } else {
            panic!("Expected Hinge constraint");
        }
    }

    #[test]
    fn test_hinge_constraint_defaults() {
        // Test that defaults are applied when parsing minimal JSON
        let json = r#"{"type":"hinge","bone":"elbow"}"#;
        let parsed: BoneConstraint = serde_json::from_str(json).unwrap();

        if let BoneConstraint::Hinge { bone, axis, min_angle, max_angle } = parsed {
            assert_eq!(bone, "elbow");
            assert_eq!(axis, ConstraintAxis::X); // default
            assert_eq!(min_angle, 0.0); // default
            assert_eq!(max_angle, 160.0); // default from default_hinge_max_angle
        } else {
            panic!("Expected Hinge constraint");
        }
    }

    #[test]
    fn test_ball_constraint_serde() {
        let constraint = BoneConstraint::ball("upper_arm_l", 90.0, -60.0, 60.0);

        let json = serde_json::to_string(&constraint).unwrap();
        assert!(json.contains("\"type\":\"ball\""));
        assert!(json.contains("\"bone\":\"upper_arm_l\""));
        assert!(json.contains("\"cone_angle\":90.0"));
        assert!(json.contains("\"twist_min\":-60.0"));
        assert!(json.contains("\"twist_max\":60.0"));

        let parsed: BoneConstraint = serde_json::from_str(&json).unwrap();
        if let BoneConstraint::Ball { bone, cone_angle, twist_min, twist_max } = parsed {
            assert_eq!(bone, "upper_arm_l");
            assert_eq!(cone_angle, 90.0);
            assert_eq!(twist_min, -60.0);
            assert_eq!(twist_max, 60.0);
        } else {
            panic!("Expected Ball constraint");
        }
    }

    #[test]
    fn test_ball_constraint_defaults() {
        let json = r#"{"type":"ball","bone":"shoulder"}"#;
        let parsed: BoneConstraint = serde_json::from_str(json).unwrap();

        if let BoneConstraint::Ball { bone, cone_angle, twist_min, twist_max } = parsed {
            assert_eq!(bone, "shoulder");
            assert_eq!(cone_angle, 45.0); // default
            assert_eq!(twist_min, -45.0); // default
            assert_eq!(twist_max, 45.0); // default
        } else {
            panic!("Expected Ball constraint");
        }
    }

    #[test]
    fn test_planar_constraint_serde() {
        let constraint = BoneConstraint::planar("hand_l", ConstraintAxis::Y, -45.0, 45.0);

        let json = serde_json::to_string(&constraint).unwrap();
        assert!(json.contains("\"type\":\"planar\""));
        assert!(json.contains("\"bone\":\"hand_l\""));
        assert!(json.contains("\"plane_normal\":\"Y\""));
        assert!(json.contains("\"min_angle\":-45.0"));
        assert!(json.contains("\"max_angle\":45.0"));

        let parsed: BoneConstraint = serde_json::from_str(&json).unwrap();
        if let BoneConstraint::Planar { bone, plane_normal, min_angle, max_angle } = parsed {
            assert_eq!(bone, "hand_l");
            assert_eq!(plane_normal, ConstraintAxis::Y);
            assert_eq!(min_angle, -45.0);
            assert_eq!(max_angle, 45.0);
        } else {
            panic!("Expected Planar constraint");
        }
    }

    #[test]
    fn test_planar_constraint_defaults() {
        let json = r#"{"type":"planar","bone":"wrist"}"#;
        let parsed: BoneConstraint = serde_json::from_str(json).unwrap();

        if let BoneConstraint::Planar { bone, plane_normal, min_angle, max_angle } = parsed {
            assert_eq!(bone, "wrist");
            assert_eq!(plane_normal, ConstraintAxis::X); // default
            assert_eq!(min_angle, -30.0); // default
            assert_eq!(max_angle, 30.0); // default
        } else {
            panic!("Expected Planar constraint");
        }
    }

    #[test]
    fn test_soft_constraint_serde() {
        let constraint = BoneConstraint::soft("tail_01", 0.7, 0.3);

        let json = serde_json::to_string(&constraint).unwrap();
        assert!(json.contains("\"type\":\"soft\""));
        assert!(json.contains("\"bone\":\"tail_01\""));
        assert!(json.contains("\"stiffness\":0.7"));
        assert!(json.contains("\"damping\":0.3"));

        let parsed: BoneConstraint = serde_json::from_str(&json).unwrap();
        if let BoneConstraint::Soft { bone, stiffness, damping } = parsed {
            assert_eq!(bone, "tail_01");
            assert_eq!(stiffness, 0.7);
            assert_eq!(damping, 0.3);
        } else {
            panic!("Expected Soft constraint");
        }
    }

    #[test]
    fn test_soft_constraint_defaults() {
        let json = r#"{"type":"soft","bone":"hair"}"#;
        let parsed: BoneConstraint = serde_json::from_str(json).unwrap();

        if let BoneConstraint::Soft { bone, stiffness, damping } = parsed {
            assert_eq!(bone, "hair");
            assert_eq!(stiffness, 0.5); // default
            assert_eq!(damping, 0.5); // default
        } else {
            panic!("Expected Soft constraint");
        }
    }

    #[test]
    fn test_soft_constraint_clamping() {
        // Values should be clamped to 0-1 range
        let constraint = BoneConstraint::soft("bone", 1.5, -0.5);
        if let BoneConstraint::Soft { stiffness, damping, .. } = constraint {
            assert_eq!(stiffness, 1.0); // clamped from 1.5
            assert_eq!(damping, 0.0); // clamped from -0.5
        } else {
            panic!("Expected Soft constraint");
        }
    }

    #[test]
    fn test_bone_constraint_bone_name() {
        let hinge = BoneConstraint::hinge("elbow", ConstraintAxis::X, 0.0, 160.0);
        assert_eq!(hinge.bone_name(), "elbow");

        let ball = BoneConstraint::ball("shoulder", 45.0, -45.0, 45.0);
        assert_eq!(ball.bone_name(), "shoulder");

        let planar = BoneConstraint::planar("wrist", ConstraintAxis::Y, -30.0, 30.0);
        assert_eq!(planar.bone_name(), "wrist");

        let soft = BoneConstraint::soft("tail", 0.5, 0.5);
        assert_eq!(soft.bone_name(), "tail");
    }

    #[test]
    fn test_bone_constraint_validation_valid() {
        // Valid hinge
        let hinge = BoneConstraint::hinge("elbow", ConstraintAxis::X, 0.0, 160.0);
        assert!(hinge.validate().is_ok());

        // Valid ball
        let ball = BoneConstraint::ball("shoulder", 45.0, -45.0, 45.0);
        assert!(ball.validate().is_ok());

        // Valid planar
        let planar = BoneConstraint::planar("wrist", ConstraintAxis::Y, -30.0, 30.0);
        assert!(planar.validate().is_ok());

        // Valid soft
        let soft = BoneConstraint::soft("tail", 0.5, 0.5);
        assert!(soft.validate().is_ok());
    }

    #[test]
    fn test_bone_constraint_validation_empty_bone() {
        let hinge = BoneConstraint::hinge("", ConstraintAxis::X, 0.0, 160.0);
        assert_eq!(hinge.validate(), Err(BoneConstraintError::EmptyBoneName));

        let ball = BoneConstraint::ball("", 45.0, -45.0, 45.0);
        assert_eq!(ball.validate(), Err(BoneConstraintError::EmptyBoneName));

        let planar = BoneConstraint::planar("", ConstraintAxis::Y, -30.0, 30.0);
        assert_eq!(planar.validate(), Err(BoneConstraintError::EmptyBoneName));

        let soft = BoneConstraint::soft("", 0.5, 0.5);
        assert_eq!(soft.validate(), Err(BoneConstraintError::EmptyBoneName));
    }

    #[test]
    fn test_bone_constraint_validation_invalid_angle_range() {
        // Hinge with min > max
        let hinge = BoneConstraint::hinge("elbow", ConstraintAxis::X, 160.0, 0.0);
        assert_eq!(
            hinge.validate(),
            Err(BoneConstraintError::InvalidAngleRange { min: 160.0, max: 0.0 })
        );

        // Ball with twist_min > twist_max
        let ball = BoneConstraint::ball("shoulder", 45.0, 45.0, -45.0);
        assert_eq!(
            ball.validate(),
            Err(BoneConstraintError::InvalidAngleRange { min: 45.0, max: -45.0 })
        );

        // Planar with min > max
        let planar = BoneConstraint::planar("wrist", ConstraintAxis::Y, 30.0, -30.0);
        assert_eq!(
            planar.validate(),
            Err(BoneConstraintError::InvalidAngleRange { min: 30.0, max: -30.0 })
        );
    }

    #[test]
    fn test_bone_constraint_validation_invalid_cone_angle() {
        // Cone angle too small
        let ball = BoneConstraint::ball("shoulder", -10.0, -45.0, 45.0);
        assert_eq!(
            ball.validate(),
            Err(BoneConstraintError::InvalidConeAngle(-10.0))
        );

        // Cone angle too large
        let ball = BoneConstraint::ball("shoulder", 200.0, -45.0, 45.0);
        assert_eq!(
            ball.validate(),
            Err(BoneConstraintError::InvalidConeAngle(200.0))
        );
    }

    #[test]
    fn test_bone_constraint_validation_invalid_stiffness_damping() {
        // Note: The soft() constructor clamps values, so we need to create directly
        let soft = BoneConstraint::Soft {
            bone: "tail".to_string(),
            stiffness: 1.5,
            damping: 0.5,
        };
        assert_eq!(
            soft.validate(),
            Err(BoneConstraintError::InvalidStiffness(1.5))
        );

        let soft = BoneConstraint::Soft {
            bone: "tail".to_string(),
            stiffness: 0.5,
            damping: -0.5,
        };
        assert_eq!(
            soft.validate(),
            Err(BoneConstraintError::InvalidDamping(-0.5))
        );
    }

    #[test]
    fn test_constraint_config_serde() {
        let config = ConstraintConfig::new()
            .with_constraint(BoneConstraint::hinge("lower_arm_l", ConstraintAxis::X, 0.0, 160.0))
            .with_constraint(BoneConstraint::ball("upper_arm_l", 90.0, -60.0, 60.0));

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("hinge"));
        assert!(json.contains("ball"));
        assert!(json.contains("lower_arm_l"));
        assert!(json.contains("upper_arm_l"));

        let parsed: ConstraintConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.constraints.len(), 2);
    }

    #[test]
    fn test_constraint_config_validation() {
        // Valid config
        let config = ConstraintConfig::new()
            .with_constraint(BoneConstraint::hinge("elbow", ConstraintAxis::X, 0.0, 160.0))
            .with_constraint(BoneConstraint::ball("shoulder", 45.0, -45.0, 45.0));
        assert!(config.validate().is_ok());

        // Invalid config - empty bone name
        let config = ConstraintConfig::new()
            .with_constraint(BoneConstraint::hinge("", ConstraintAxis::X, 0.0, 160.0));
        assert_eq!(config.validate(), Err(BoneConstraintError::EmptyBoneName));
    }

    #[test]
    fn test_constraint_config_is_empty() {
        let empty = ConstraintConfig::new();
        assert!(empty.is_empty());

        let non_empty = ConstraintConfig::new()
            .with_constraint(BoneConstraint::hinge("elbow", ConstraintAxis::X, 0.0, 160.0));
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_rig_setup_with_constraints() {
        let rig_setup = RigSetup::new()
            .with_preset(IkPreset::HumanoidArms)
            .with_constraint(BoneConstraint::hinge("lower_arm_l", ConstraintAxis::X, 0.0, 160.0))
            .with_constraint(BoneConstraint::hinge("lower_arm_r", ConstraintAxis::X, 0.0, 160.0))
            .with_constraint(BoneConstraint::ball("upper_arm_l", 90.0, -60.0, 60.0))
            .with_constraint(BoneConstraint::ball("upper_arm_r", 90.0, -60.0, 60.0));

        // Validate IK chains
        assert!(rig_setup.validate().is_ok());
        // Validate constraints
        assert!(rig_setup.validate_constraints().is_ok());

        // Test serialization
        let json = serde_json::to_string(&rig_setup).unwrap();
        assert!(json.contains("humanoid_arms"));
        assert!(json.contains("constraints"));
        assert!(json.contains("hinge"));
        assert!(json.contains("ball"));

        let parsed: RigSetup = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.presets.len(), 1);
        assert_eq!(parsed.constraints.constraints.len(), 4);
    }

    #[test]
    fn test_rig_setup_constraints_skip_serializing_if_empty() {
        let rig_setup = RigSetup::new().with_preset(IkPreset::HumanoidLegs);

        let json = serde_json::to_string(&rig_setup).unwrap();
        // Constraints should not appear in JSON when empty
        assert!(!json.contains("constraints"));
    }

    #[test]
    fn test_bone_constraint_error_display() {
        assert_eq!(
            BoneConstraintError::EmptyBoneName.to_string(),
            "Bone name cannot be empty"
        );
        assert_eq!(
            BoneConstraintError::InvalidAngleRange { min: 10.0, max: 5.0 }.to_string(),
            "Invalid angle range: min (10) > max (5)"
        );
        assert_eq!(
            BoneConstraintError::InvalidConeAngle(200.0).to_string(),
            "Cone angle must be between 0 and 180 degrees, got 200"
        );
        assert_eq!(
            BoneConstraintError::InvalidStiffness(1.5).to_string(),
            "Stiffness must be between 0 and 1, got 1.5"
        );
        assert_eq!(
            BoneConstraintError::InvalidDamping(-0.5).to_string(),
            "Damping must be between 0 and 1, got -0.5"
        );
    }

    // =========================================================================
    // Animator Rig Config Tests
    // =========================================================================

    #[test]
    fn test_widget_style_serde() {
        // Test all variants serialize correctly
        let styles = [
            (WidgetStyle::WireCircle, "\"wire_circle\""),
            (WidgetStyle::WireCube, "\"wire_cube\""),
            (WidgetStyle::WireSphere, "\"wire_sphere\""),
            (WidgetStyle::WireDiamond, "\"wire_diamond\""),
            (WidgetStyle::CustomMesh, "\"custom_mesh\""),
        ];
        for (style, expected) in styles {
            let json = serde_json::to_string(&style).unwrap();
            assert_eq!(json, expected);
        }

        // Test default
        assert_eq!(WidgetStyle::default(), WidgetStyle::WireCircle);

        // Test blender_name
        assert_eq!(WidgetStyle::WireCircle.blender_name(), "WGT_circle");
        assert_eq!(WidgetStyle::WireDiamond.blender_name(), "WGT_diamond");

        // Test standard_styles
        let standard = WidgetStyle::standard_styles();
        assert_eq!(standard.len(), 4);
        assert!(!standard.contains(&WidgetStyle::CustomMesh));
    }

    #[test]
    fn test_bone_collection_serde() {
        let collection = BoneCollection::new("IK Controls")
            .with_bone("ik_foot_l")
            .with_bone("ik_foot_r")
            .with_visibility(true)
            .with_selectability(true);

        let json = serde_json::to_string(&collection).unwrap();
        assert!(json.contains("IK Controls"));
        assert!(json.contains("ik_foot_l"));
        assert!(json.contains("ik_foot_r"));

        let parsed: BoneCollection = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "IK Controls");
        assert_eq!(parsed.bones.len(), 2);
        assert!(parsed.visible);
        assert!(parsed.selectable);
    }

    #[test]
    fn test_bone_collection_with_bones() {
        let collection = BoneCollection::new("Deform")
            .with_bones(["arm_l", "arm_r", "leg_l", "leg_r"]);

        assert_eq!(collection.bones.len(), 4);
        assert!(collection.bones.contains(&"arm_l".to_string()));
    }

    #[test]
    fn test_bone_collection_validation() {
        // Valid collection
        let valid = BoneCollection::new("Test");
        assert!(valid.validate().is_ok());

        // Invalid - empty name
        let invalid = BoneCollection::new("");
        assert_eq!(invalid.validate(), Err(AnimatorRigError::EmptyCollectionName));
    }

    #[test]
    fn test_bone_collection_preset() {
        // Test all presets
        let presets = [
            (BoneCollectionPreset::IkControls, "IK Controls", true, true),
            (BoneCollectionPreset::FkControls, "FK Controls", true, true),
            (BoneCollectionPreset::Deform, "Deform", false, false),
            (BoneCollectionPreset::Mechanism, "Mechanism", false, false),
        ];

        for (preset, name, visible, selectable) in presets {
            assert_eq!(preset.default_name(), name);
            assert_eq!(preset.default_visibility(), visible);
            assert_eq!(preset.default_selectability(), selectable);

            let collection = preset.to_collection();
            assert_eq!(collection.name, name);
            assert_eq!(collection.visible, visible);
            assert_eq!(collection.selectable, selectable);
        }
    }

    #[test]
    fn test_bone_color() {
        // Test constructors
        let color = BoneColor::new(0.5, 0.7, 0.9);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.7);
        assert_eq!(color.b, 0.9);

        // Test clamping
        let clamped = BoneColor::new(1.5, -0.5, 0.5);
        assert_eq!(clamped.r, 1.0);
        assert_eq!(clamped.g, 0.0);
        assert_eq!(clamped.b, 0.5);

        // Test standard colors
        let blue = BoneColor::left_blue();
        assert!(blue.b > blue.r);
        assert!(blue.b > blue.g);

        let red = BoneColor::right_red();
        assert!(red.r > red.g);
        assert!(red.r > red.b);

        let yellow = BoneColor::center_yellow();
        assert!(yellow.r > 0.9);
        assert!(yellow.g > 0.8);

        // Test as_array
        let arr = color.as_array();
        assert_eq!(arr, [0.5, 0.7, 0.9]);

        // Test default
        let default = BoneColor::default();
        assert_eq!(default, BoneColor::white());
    }

    #[test]
    fn test_bone_color_serde() {
        let color = BoneColor::new(0.2, 0.4, 1.0);
        let json = serde_json::to_string(&color).unwrap();
        assert!(json.contains("0.2"));
        assert!(json.contains("0.4"));
        assert!(json.contains("1.0"));

        let parsed: BoneColor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.r, 0.2);
        assert_eq!(parsed.g, 0.4);
        assert_eq!(parsed.b, 1.0);
    }

    #[test]
    fn test_bone_color_scheme_standard() {
        let scheme = BoneColorScheme::Standard;

        // Left bones (suffix _l)
        let left_color = scheme.color_for_bone("arm_l");
        assert_eq!(left_color, BoneColor::left_blue());

        // Right bones (suffix _r)
        let right_color = scheme.color_for_bone("arm_r");
        assert_eq!(right_color, BoneColor::right_red());

        // Center bones (no suffix)
        let center_color = scheme.color_for_bone("spine");
        assert_eq!(center_color, BoneColor::center_yellow());

        // Test uppercase suffix
        let left_upper = scheme.color_for_bone("arm_L");
        assert_eq!(left_upper, BoneColor::left_blue());
    }

    #[test]
    fn test_bone_color_scheme_custom() {
        let scheme = BoneColorScheme::custom(
            BoneColor::new(0.0, 1.0, 0.0),  // green for left
            BoneColor::new(1.0, 0.0, 1.0),  // magenta for right
            BoneColor::new(0.5, 0.5, 0.5),  // gray for center
        );

        let left = scheme.color_for_bone("leg_l");
        assert_eq!(left.g, 1.0);

        let right = scheme.color_for_bone("leg_r");
        assert_eq!(right.r, 1.0);
        assert_eq!(right.b, 1.0);

        let center = scheme.color_for_bone("head");
        assert_eq!(center.r, 0.5);
    }

    #[test]
    fn test_bone_color_scheme_per_bone() {
        let mut colors = std::collections::HashMap::new();
        colors.insert("special_bone".to_string(), BoneColor::new(1.0, 0.5, 0.0));

        let scheme = BoneColorScheme::PerBone {
            colors,
            default: BoneColor::white(),
        };

        let special = scheme.color_for_bone("special_bone");
        assert_eq!(special.r, 1.0);
        assert_eq!(special.g, 0.5);

        let other = scheme.color_for_bone("other_bone");
        assert_eq!(other, BoneColor::white());
    }

    #[test]
    fn test_bone_color_scheme_serde() {
        // Standard scheme
        let standard = BoneColorScheme::Standard;
        let json = serde_json::to_string(&standard).unwrap();
        assert!(json.contains("standard"));

        let parsed: BoneColorScheme = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, BoneColorScheme::Standard));

        // Custom scheme
        let custom = BoneColorScheme::custom(
            BoneColor::left_blue(),
            BoneColor::right_red(),
            BoneColor::center_yellow(),
        );
        let json = serde_json::to_string(&custom).unwrap();
        assert!(json.contains("custom"));
        assert!(json.contains("left"));
        assert!(json.contains("right"));
        assert!(json.contains("center"));
    }

    #[test]
    fn test_armature_display_serde() {
        let displays = [
            (ArmatureDisplay::Octahedral, "\"OCTAHEDRAL\""),
            (ArmatureDisplay::Stick, "\"STICK\""),
            (ArmatureDisplay::Bbone, "\"BBONE\""),
            (ArmatureDisplay::Envelope, "\"ENVELOPE\""),
            (ArmatureDisplay::Wire, "\"WIRE\""),
        ];

        for (display, expected) in displays {
            let json = serde_json::to_string(&display).unwrap();
            assert_eq!(json, expected);
            assert_eq!(display.blender_name(), expected.trim_matches('"'));
        }

        // Test default
        assert_eq!(ArmatureDisplay::default(), ArmatureDisplay::Octahedral);
    }

    #[test]
    fn test_animator_rig_config_default() {
        let config = AnimatorRigConfig::new();

        // All features enabled by default
        assert!(config.collections);
        assert!(config.shapes);
        assert!(config.colors);
        assert_eq!(config.display, ArmatureDisplay::Octahedral);
        assert_eq!(config.widget_style, WidgetStyle::WireCircle);
        assert!(config.bone_collections.is_empty());
        assert!(matches!(config.bone_colors, BoneColorScheme::Standard));
    }

    #[test]
    fn test_animator_rig_config_minimal() {
        let config = AnimatorRigConfig::minimal();

        // All features disabled
        assert!(!config.collections);
        assert!(!config.shapes);
        assert!(!config.colors);
    }

    #[test]
    fn test_animator_rig_config_builder() {
        let config = AnimatorRigConfig::new()
            .with_collections(true)
            .with_shapes(true)
            .with_colors(false)
            .with_display(ArmatureDisplay::Stick)
            .with_widget_style(WidgetStyle::WireDiamond)
            .with_bone_collection(BoneCollection::new("Custom"))
            .with_bone_colors(BoneColorScheme::Standard);

        assert!(config.collections);
        assert!(config.shapes);
        assert!(!config.colors);
        assert_eq!(config.display, ArmatureDisplay::Stick);
        assert_eq!(config.widget_style, WidgetStyle::WireDiamond);
        assert_eq!(config.bone_collections.len(), 1);
    }

    #[test]
    fn test_animator_rig_config_serde() {
        let config = AnimatorRigConfig::new()
            .with_display(ArmatureDisplay::Bbone)
            .with_widget_style(WidgetStyle::WireSphere)
            .with_bone_collection(
                BoneCollection::new("Test Collection")
                    .with_bones(["bone1", "bone2"])
            );

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("BBONE"));
        assert!(json.contains("wire_sphere"));
        assert!(json.contains("Test Collection"));
        assert!(json.contains("bone1"));

        let parsed: AnimatorRigConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.display, ArmatureDisplay::Bbone);
        assert_eq!(parsed.widget_style, WidgetStyle::WireSphere);
        assert_eq!(parsed.bone_collections.len(), 1);
    }

    #[test]
    fn test_animator_rig_config_validation() {
        // Valid config
        let valid = AnimatorRigConfig::new()
            .with_bone_collection(BoneCollection::new("Valid"));
        assert!(valid.validate().is_ok());

        // Invalid - empty collection name
        let invalid = AnimatorRigConfig::new()
            .with_bone_collection(BoneCollection::new(""));
        assert_eq!(invalid.validate(), Err(AnimatorRigError::EmptyCollectionName));
    }

    #[test]
    fn test_animator_rig_config_default_humanoid_collections() {
        let collections = AnimatorRigConfig::default_humanoid_collections();

        assert_eq!(collections.len(), 4);

        // Check IK Controls
        let ik = &collections[0];
        assert_eq!(ik.name, "IK Controls");
        assert!(ik.bones.contains(&"ik_foot_l".to_string()));
        assert!(ik.visible);

        // Check FK Controls
        let fk = &collections[1];
        assert_eq!(fk.name, "FK Controls");
        assert!(fk.bones.contains(&"root".to_string()));

        // Check Deform
        let deform = &collections[2];
        assert_eq!(deform.name, "Deform");
        assert!(!deform.visible);
        assert!(!deform.selectable);

        // Check Mechanism
        let mechanism = &collections[3];
        assert_eq!(mechanism.name, "Mechanism");
        assert!(!mechanism.visible);
    }

    #[test]
    fn test_animator_rig_error_display() {
        assert_eq!(
            AnimatorRigError::EmptyCollectionName.to_string(),
            "Bone collection name cannot be empty"
        );
        assert_eq!(
            AnimatorRigError::DuplicateCollectionName("Test".to_string()).to_string(),
            "Duplicate bone collection name: Test"
        );
        assert_eq!(
            AnimatorRigError::InvalidWidgetStyle {
                bone: "arm".to_string(),
                style: "invalid".to_string()
            }.to_string(),
            "Invalid widget style 'invalid' for bone 'arm'"
        );
    }

    // =========================================================================
    // Procedural Layer Tests
    // =========================================================================

    #[test]
    fn test_procedural_layer_type_serde() {
        let types = [
            (ProceduralLayerType::Breathing, "\"breathing\""),
            (ProceduralLayerType::Sway, "\"sway\""),
            (ProceduralLayerType::Bob, "\"bob\""),
            (ProceduralLayerType::Noise, "\"noise\""),
        ];

        for (layer_type, expected) in types {
            let json = serde_json::to_string(&layer_type).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_procedural_axis_serde() {
        let axes = [
            (ProceduralAxis::Pitch, "\"pitch\""),
            (ProceduralAxis::Yaw, "\"yaw\""),
            (ProceduralAxis::Roll, "\"roll\""),
        ];

        for (axis, expected) in axes {
            let json = serde_json::to_string(&axis).unwrap();
            assert_eq!(json, expected);
        }

        // Test default
        assert_eq!(ProceduralAxis::default(), ProceduralAxis::Pitch);

        // Test to_index
        assert_eq!(ProceduralAxis::Pitch.to_index(), 0);
        assert_eq!(ProceduralAxis::Yaw.to_index(), 1);
        assert_eq!(ProceduralAxis::Roll.to_index(), 2);
    }

    #[test]
    fn test_procedural_layer_breathing() {
        let layer = ProceduralLayer::breathing("chest");

        assert_eq!(layer.layer_type, ProceduralLayerType::Breathing);
        assert_eq!(layer.target, "chest");
        assert_eq!(layer.axis, ProceduralAxis::Pitch);
        assert_eq!(layer.period_frames, 90);
        assert_eq!(layer.amplitude, 0.02);
    }

    #[test]
    fn test_procedural_layer_sway() {
        let layer = ProceduralLayer::sway("spine");

        assert_eq!(layer.layer_type, ProceduralLayerType::Sway);
        assert_eq!(layer.target, "spine");
        assert_eq!(layer.axis, ProceduralAxis::Roll);
        assert_eq!(layer.period_frames, 120);
        assert_eq!(layer.amplitude, 0.03);
    }

    #[test]
    fn test_procedural_layer_bob() {
        let layer = ProceduralLayer::bob("body");

        assert_eq!(layer.layer_type, ProceduralLayerType::Bob);
        assert_eq!(layer.target, "body");
        assert_eq!(layer.axis, ProceduralAxis::Pitch);
        assert_eq!(layer.period_frames, 60);
        assert_eq!(layer.amplitude, 0.02);
    }

    #[test]
    fn test_procedural_layer_noise() {
        let layer = ProceduralLayer::noise("head");

        assert_eq!(layer.layer_type, ProceduralLayerType::Noise);
        assert_eq!(layer.target, "head");
        assert_eq!(layer.axis, ProceduralAxis::Roll);
        assert_eq!(layer.amplitude, 1.0);
        assert_eq!(layer.frequency, 0.3);
    }

    #[test]
    fn test_procedural_layer_builder() {
        let layer = ProceduralLayer::breathing("chest")
            .with_axis(ProceduralAxis::Yaw)
            .with_period(120)
            .with_amplitude(0.05)
            .with_phase_offset(0.5)
            .with_frequency(0.5);

        assert_eq!(layer.axis, ProceduralAxis::Yaw);
        assert_eq!(layer.period_frames, 120);
        assert_eq!(layer.amplitude, 0.05);
        assert_eq!(layer.phase_offset, 0.5);
        assert_eq!(layer.frequency, 0.5);
    }

    #[test]
    fn test_procedural_layer_serde() {
        let layer = ProceduralLayer::breathing("chest")
            .with_amplitude(0.05)
            .with_period(100);

        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("breathing"));
        assert!(json.contains("chest"));
        assert!(json.contains("0.05"));
        assert!(json.contains("100"));

        let parsed: ProceduralLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.layer_type, ProceduralLayerType::Breathing);
        assert_eq!(parsed.target, "chest");
        assert_eq!(parsed.amplitude, 0.05);
        assert_eq!(parsed.period_frames, 100);
    }

    // =========================================================================
    // Pose and Phase Tests
    // =========================================================================

    #[test]
    fn test_timing_curve_serde() {
        let curves = [
            (TimingCurve::Linear, "\"linear\""),
            (TimingCurve::EaseIn, "\"ease_in\""),
            (TimingCurve::EaseOut, "\"ease_out\""),
            (TimingCurve::EaseInOut, "\"ease_in_out\""),
            (TimingCurve::ExponentialIn, "\"exponential_in\""),
            (TimingCurve::ExponentialOut, "\"exponential_out\""),
            (TimingCurve::Constant, "\"constant\""),
        ];

        for (curve, expected) in curves {
            let json = serde_json::to_string(&curve).unwrap();
            assert_eq!(json, expected);
        }

        // Test default
        assert_eq!(TimingCurve::default(), TimingCurve::Linear);
    }

    #[test]
    fn test_pose_bone_transform() {
        // Test rotation constructor
        let rot = PoseBoneTransform::rotation(15.0, 30.0, 45.0);
        assert_eq!(rot.pitch, 15.0);
        assert_eq!(rot.yaw, 30.0);
        assert_eq!(rot.roll, 45.0);
        assert!(rot.location.is_none());

        // Test pitch only
        let pitch = PoseBoneTransform::pitch(20.0);
        assert_eq!(pitch.pitch, 20.0);
        assert_eq!(pitch.yaw, 0.0);

        // Test yaw only
        let yaw = PoseBoneTransform::yaw(30.0);
        assert_eq!(yaw.yaw, 30.0);
        assert_eq!(yaw.pitch, 0.0);

        // Test roll only
        let roll = PoseBoneTransform::roll(40.0);
        assert_eq!(roll.roll, 40.0);
        assert_eq!(roll.pitch, 0.0);

        // Test with location
        let with_loc = PoseBoneTransform::rotation(10.0, 20.0, 30.0)
            .with_location([0.1, 0.2, 0.3]);
        assert!(with_loc.location.is_some());
        assert_eq!(with_loc.location.unwrap(), [0.1, 0.2, 0.3]);

        // Test as_euler_degrees
        let euler = rot.as_euler_degrees();
        assert_eq!(euler, [15.0, 30.0, 45.0]);
    }

    #[test]
    fn test_pose_bone_transform_serde() {
        let transform = PoseBoneTransform::rotation(10.0, 20.0, 30.0)
            .with_location([0.1, 0.2, 0.3]);

        let json = serde_json::to_string(&transform).unwrap();
        assert!(json.contains("\"pitch\":10.0"));
        assert!(json.contains("\"yaw\":20.0"));
        assert!(json.contains("\"roll\":30.0"));
        assert!(json.contains("location"));

        let parsed: PoseBoneTransform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.pitch, 10.0);
        assert_eq!(parsed.yaw, 20.0);
        assert_eq!(parsed.roll, 30.0);
    }

    #[test]
    fn test_pose_definition() {
        let pose = PoseDefinition::new()
            .with_bone("arm_l", PoseBoneTransform::pitch(15.0))
            .with_bone("arm_r", PoseBoneTransform::pitch(-15.0));

        assert_eq!(pose.bones.len(), 2);
        assert!(pose.bones.contains_key("arm_l"));
        assert!(pose.bones.contains_key("arm_r"));
    }

    #[test]
    fn test_pose_definition_serde() {
        let pose = PoseDefinition::new()
            .with_bone("leg_l", PoseBoneTransform::rotation(20.0, 0.0, 0.0));

        let json = serde_json::to_string(&pose).unwrap();
        assert!(json.contains("leg_l"));
        assert!(json.contains("pitch"));

        let parsed: PoseDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.bones.len(), 1);
    }

    #[test]
    fn test_phase_ik_target() {
        let target = PhaseIkTarget::new(10, [0.1, 0.2, 0.3]);
        assert_eq!(target.frame, 10);
        assert_eq!(target.location, [0.1, 0.2, 0.3]);
        assert!(target.ikfk.is_none());

        let with_blend = target.with_ikfk(0.7);
        assert_eq!(with_blend.ikfk, Some(0.7));
    }

    #[test]
    fn test_phase_ik_target_serde() {
        let target = PhaseIkTarget::new(15, [0.5, 0.6, 0.7])
            .with_ikfk(0.8);

        let json = serde_json::to_string(&target).unwrap();
        assert!(json.contains("\"frame\":15"));
        assert!(json.contains("location"));
        assert!(json.contains("0.8"));

        let parsed: PhaseIkTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.frame, 15);
        assert_eq!(parsed.ikfk, Some(0.8));
    }

    #[test]
    fn test_animation_phase() {
        let phase = AnimationPhase::new(0, 30)
            .with_name("contact")
            .with_curve(TimingCurve::EaseInOut)
            .with_pose("standing");

        assert_eq!(phase.name, Some("contact".to_string()));
        assert_eq!(phase.start_frame, 0);
        assert_eq!(phase.end_frame, 30);
        assert_eq!(phase.curve, TimingCurve::EaseInOut);
        assert_eq!(phase.pose, Some("standing".to_string()));
        assert_eq!(phase.duration_frames(), 30);
    }

    #[test]
    fn test_animation_phase_with_ik_targets() {
        let targets = vec![
            PhaseIkTarget::new(0, [0.0, 0.0, 0.0]),
            PhaseIkTarget::new(15, [0.1, 0.0, 0.2]),
        ];

        let phase = AnimationPhase::new(0, 30)
            .with_ik_targets("ik_foot_l", targets);

        assert!(phase.ik_targets.contains_key("ik_foot_l"));
        assert_eq!(phase.ik_targets["ik_foot_l"].len(), 2);
    }

    #[test]
    fn test_animation_phase_serde() {
        let phase = AnimationPhase::new(0, 60)
            .with_name("walk_cycle")
            .with_curve(TimingCurve::Linear)
            .with_pose("neutral");

        let json = serde_json::to_string(&phase).unwrap();
        assert!(json.contains("walk_cycle"));
        assert!(json.contains("linear"));
        assert!(json.contains("neutral"));

        let parsed: AnimationPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, Some("walk_cycle".to_string()));
        assert_eq!(parsed.start_frame, 0);
        assert_eq!(parsed.end_frame, 60);
    }

    // =========================================================================
    // Foot System Tests
    // =========================================================================

    #[test]
    fn test_foot_system() {
        let foot = FootSystem::new("foot_l", "ik_foot_l", "heel_l", "toe_l");

        assert_eq!(foot.name, "foot_l");
        assert_eq!(foot.ik_target, "ik_foot_l");
        assert_eq!(foot.heel_bone, "heel_l");
        assert_eq!(foot.toe_bone, "toe_l");
        assert!(foot.ball_bone.is_none());
        assert_eq!(foot.roll_limits, [-30.0, 60.0]);
    }

    #[test]
    fn test_foot_system_with_ball_bone() {
        let foot = FootSystem::new("foot_r", "ik_foot_r", "heel_r", "toe_r")
            .with_ball_bone("ball_r")
            .with_roll_limits(-20.0, 70.0);

        assert_eq!(foot.ball_bone, Some("ball_r".to_string()));
        assert_eq!(foot.roll_limits, [-20.0, 70.0]);
    }

    #[test]
    fn test_foot_system_serde() {
        let foot = FootSystem::new("foot_l", "ik_foot_l", "heel_l", "toe_l")
            .with_ball_bone("ball_l");

        let json = serde_json::to_string(&foot).unwrap();
        assert!(json.contains("foot_l"));
        assert!(json.contains("ik_foot_l"));
        assert!(json.contains("heel_l"));
        assert!(json.contains("toe_l"));
        assert!(json.contains("ball_l"));

        let parsed: FootSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "foot_l");
        assert!(parsed.ball_bone.is_some());
    }

    // =========================================================================
    // Aim Constraint Tests
    // =========================================================================

    #[test]
    fn test_aim_axis_serde() {
        let axes = [
            (AimAxis::PosX, "\"X\""),
            (AimAxis::NegX, "\"-X\""),
            (AimAxis::PosY, "\"Y\""),
            (AimAxis::NegY, "\"-Y\""),
            (AimAxis::PosZ, "\"Z\""),
            (AimAxis::NegZ, "\"-Z\""),
        ];

        for (axis, expected) in axes {
            let json = serde_json::to_string(&axis).unwrap();
            assert_eq!(json, expected);
        }

        // Test default
        assert_eq!(AimAxis::default(), AimAxis::PosX);

        // Test blender_track_axis
        assert_eq!(AimAxis::PosX.blender_track_axis(), "TRACK_X");
        assert_eq!(AimAxis::NegX.blender_track_axis(), "TRACK_NEGATIVE_X");
        assert_eq!(AimAxis::PosY.blender_track_axis(), "TRACK_Y");
        assert_eq!(AimAxis::NegY.blender_track_axis(), "TRACK_NEGATIVE_Y");
        assert_eq!(AimAxis::PosZ.blender_track_axis(), "TRACK_Z");
        assert_eq!(AimAxis::NegZ.blender_track_axis(), "TRACK_NEGATIVE_Z");
    }

    #[test]
    fn test_aim_constraint() {
        let aim = AimConstraint::new("head", "head_bone", "look_target");

        assert_eq!(aim.name, "head");
        assert_eq!(aim.bone, "head_bone");
        assert_eq!(aim.target, "look_target");
        assert_eq!(aim.track_axis, AimAxis::PosX);
        assert_eq!(aim.up_axis, ConstraintAxis::Z);
        assert_eq!(aim.influence, 1.0);
    }

    #[test]
    fn test_aim_constraint_builder() {
        let aim = AimConstraint::new("eyes", "eye_bone", "target")
            .with_track_axis(AimAxis::PosY)
            .with_up_axis(ConstraintAxis::X)
            .with_influence(0.8);

        assert_eq!(aim.track_axis, AimAxis::PosY);
        assert_eq!(aim.up_axis, ConstraintAxis::X);
        assert_eq!(aim.influence, 0.8);
    }

    #[test]
    fn test_aim_constraint_serde() {
        let aim = AimConstraint::new("head_track", "head", "look_target")
            .with_track_axis(AimAxis::PosZ)
            .with_influence(0.9);

        let json = serde_json::to_string(&aim).unwrap();
        assert!(json.contains("head_track"));
        assert!(json.contains("head"));
        assert!(json.contains("look_target"));
        assert!(json.contains("\"Z\""));

        let parsed: AimConstraint = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "head_track");
        assert_eq!(parsed.track_axis, AimAxis::PosZ);
    }

    // =========================================================================
    // Twist Bone Tests
    // =========================================================================

    #[test]
    fn test_twist_bone() {
        let twist = TwistBone::new("upper_arm", "upper_arm_twist");

        assert!(twist.name.is_none());
        assert_eq!(twist.source, "upper_arm");
        assert_eq!(twist.target, "upper_arm_twist");
        assert_eq!(twist.axis, ConstraintAxis::Y);
        assert_eq!(twist.influence, 0.5);
    }

    #[test]
    fn test_twist_bone_builder() {
        let twist = TwistBone::new("forearm", "forearm_twist")
            .with_name("forearm_twist_setup")
            .with_axis(ConstraintAxis::X)
            .with_influence(0.7);

        assert_eq!(twist.name, Some("forearm_twist_setup".to_string()));
        assert_eq!(twist.axis, ConstraintAxis::X);
        assert_eq!(twist.influence, 0.7);
    }

    #[test]
    fn test_twist_bone_serde() {
        let twist = TwistBone::new("source_bone", "target_bone")
            .with_name("twist_test")
            .with_axis(ConstraintAxis::Z)
            .with_influence(0.6);

        let json = serde_json::to_string(&twist).unwrap();
        assert!(json.contains("twist_test"));
        assert!(json.contains("source_bone"));
        assert!(json.contains("target_bone"));
        assert!(json.contains("\"Z\""));
        assert!(json.contains("0.6"));

        let parsed: TwistBone = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, Some("twist_test".to_string()));
        assert_eq!(parsed.source, "source_bone");
        assert_eq!(parsed.target, "target_bone");
    }

    // =========================================================================
    // Stretch Settings Tests
    // =========================================================================

    #[test]
    fn test_volume_preservation_serde() {
        let modes = [
            (VolumePreservation::None, "\"none\""),
            (VolumePreservation::Uniform, "\"uniform\""),
            (VolumePreservation::X, "\"x\""),
            (VolumePreservation::Z, "\"z\""),
        ];

        for (mode, expected) in modes {
            let json = serde_json::to_string(&mode).unwrap();
            assert_eq!(json, expected);
        }

        // Test default
        assert_eq!(VolumePreservation::default(), VolumePreservation::None);
    }

    #[test]
    fn test_stretch_settings_default() {
        let stretch = StretchSettings::default();

        assert!(!stretch.enabled);
        // Default trait uses Rust defaults (0.0), not serde defaults
        assert_eq!(stretch.max_stretch, 0.0);
        assert_eq!(stretch.min_stretch, 0.0);
        assert_eq!(stretch.volume_preservation, VolumePreservation::None);
    }

    #[test]
    fn test_stretch_settings_enabled() {
        let stretch = StretchSettings::enabled();

        assert!(stretch.enabled);
        assert_eq!(stretch.max_stretch, 1.5);
        assert_eq!(stretch.min_stretch, 0.5);
    }

    #[test]
    fn test_stretch_settings_builder() {
        let stretch = StretchSettings::enabled()
            .with_limits(0.7, 2.0)
            .with_volume_preservation(VolumePreservation::Uniform);

        assert_eq!(stretch.min_stretch, 0.7);
        assert_eq!(stretch.max_stretch, 2.0);
        assert_eq!(stretch.volume_preservation, VolumePreservation::Uniform);
    }

    #[test]
    fn test_stretch_settings_serde() {
        let stretch = StretchSettings::enabled()
            .with_limits(0.8, 1.8)
            .with_volume_preservation(VolumePreservation::X);

        let json = serde_json::to_string(&stretch).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("0.8"));
        assert!(json.contains("1.8"));
        assert!(json.contains("\"x\""));

        let parsed: StretchSettings = serde_json::from_str(&json).unwrap();
        assert!(parsed.enabled);
        assert_eq!(parsed.min_stretch, 0.8);
        assert_eq!(parsed.max_stretch, 1.8);
    }

    // =========================================================================
    // Bake Settings Tests
    // =========================================================================

    #[test]
    fn test_bake_settings_default() {
        let bake = BakeSettings::default();

        assert!(bake.simplify);
        assert!(bake.start_frame.is_none());
        assert!(bake.end_frame.is_none());
        assert!(bake.visual_keying);
        assert!(bake.clear_constraints);
        assert_eq!(bake.frame_step, 1);
        assert_eq!(bake.tolerance, 0.001);
        assert!(bake.remove_ik_bones);
    }

    #[test]
    fn test_bake_settings_builder() {
        let bake = BakeSettings::new()
            .with_frame_range(10, 100)
            .with_frame_step(2)
            .with_simplify(false)
            .with_tolerance(0.01)
            .with_clear_constraints(false)
            .with_visual_keying(true);

        assert_eq!(bake.start_frame, Some(10));
        assert_eq!(bake.end_frame, Some(100));
        assert_eq!(bake.frame_step, 2);
        assert!(!bake.simplify);
        assert_eq!(bake.tolerance, 0.01);
        assert!(!bake.clear_constraints);
        assert!(bake.visual_keying);
    }

    #[test]
    fn test_bake_settings_serde() {
        let bake = BakeSettings::new()
            .with_frame_range(0, 60)
            .with_frame_step(1);

        let json = serde_json::to_string(&bake).unwrap();
        assert!(json.contains("\"start_frame\":0"));
        assert!(json.contains("\"end_frame\":60"));
        assert!(json.contains("\"frame_step\":1"));

        let parsed: BakeSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.start_frame, Some(0));
        assert_eq!(parsed.end_frame, Some(60));
    }

    // =========================================================================
    // Rig Setup Integration Tests
    // =========================================================================

    #[test]
    fn test_rig_setup_complete() {
        let rig_setup = RigSetup::new()
            .with_preset(IkPreset::HumanoidLegs)
            .with_preset(IkPreset::HumanoidArms)
            .with_chain(IkChain::new("ik_spine", 3, IkTargetConfig::new("ik_spine_tip")))
            .with_constraint(BoneConstraint::hinge("lower_arm_l", ConstraintAxis::X, 0.0, 160.0))
            .with_foot_system(FootSystem::new("foot_l", "ik_foot_l", "heel_l", "toe_l"))
            .with_aim_constraint(AimConstraint::new("head_track", "head", "look_target"))
            .with_twist_bone(TwistBone::new("upper_arm_l", "upper_arm_twist_l"))
            .with_stretch(StretchSettings::enabled())
            .with_bake(BakeSettings::new());

        assert_eq!(rig_setup.presets.len(), 2);
        assert_eq!(rig_setup.ik_chains.len(), 1);
        assert_eq!(rig_setup.constraints.constraints.len(), 1);
        assert_eq!(rig_setup.foot_systems.len(), 1);
        assert_eq!(rig_setup.aim_constraints.len(), 1);
        assert_eq!(rig_setup.twist_bones.len(), 1);
        assert!(rig_setup.stretch.is_some());
        assert!(rig_setup.bake.is_some());

        // Test validation
        assert!(rig_setup.validate().is_ok());
        assert!(rig_setup.validate_constraints().is_ok());

        // Test serialization
        let json = serde_json::to_string(&rig_setup).unwrap();
        assert!(json.contains("humanoid_legs"));
        assert!(json.contains("humanoid_arms"));
        assert!(json.contains("ik_spine"));
        assert!(json.contains("constraints"));
        assert!(json.contains("foot_systems"));
        assert!(json.contains("aim_constraints"));
        assert!(json.contains("twist_bones"));
        assert!(json.contains("stretch"));
        assert!(json.contains("bake"));
    }

    // =========================================================================
    // Full Animation Params Tests
    // =========================================================================

    #[test]
    fn test_skeletal_animation_blender_rigged_v1_params_complete() {
        let mut poses = std::collections::HashMap::new();
        poses.insert(
            "standing".to_string(),
            PoseDefinition::new()
                .with_bone("leg_l", PoseBoneTransform::pitch(10.0))
                .with_bone("leg_r", PoseBoneTransform::pitch(10.0)),
        );

        let phases = vec![
            AnimationPhase::new(0, 30)
                .with_name("start")
                .with_curve(TimingCurve::EaseIn)
                .with_pose("standing"),
        ];

        let procedural_layers = vec![
            ProceduralLayer::breathing("chest"),
            ProceduralLayer::sway("spine"),
        ];

        let params = SkeletalAnimationBlenderRiggedV1Params {
            skeleton_preset: Some(SkeletonPreset::HumanoidBasicV1),
            clip_name: "idle".to_string(),
            input_armature: Some("character.glb".to_string()),
            character: None,
            duration_frames: 60,
            duration_seconds: Some(2.0),
            fps: 30,
            r#loop: true,
            ground_offset: 0.05,
            rig_setup: RigSetup::new()
                .with_preset(IkPreset::HumanoidLegs)
                .with_preset(IkPreset::HumanoidArms),
            poses,
            phases,
            procedural_layers,
            keyframes: vec![],
            ik_keyframes: vec![],
            interpolation: InterpolationMode::Bezier,
            export: Some(AnimationExportSettings {
                bake_transforms: true,
                optimize_keyframes: true,
                separate_file: false,
                save_blend: true,
            }),
            animator_rig: Some(AnimatorRigConfig::new()
                .with_display(ArmatureDisplay::Stick)
                .with_widget_style(WidgetStyle::WireDiamond)),
            save_blend: true,
            conventions: Some(ConventionsConfig { strict: false }),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("idle"));
        assert!(json.contains("character.glb"));
        assert!(json.contains("humanoid_legs"));
        assert!(json.contains("standing"));
        assert!(json.contains("breathing"));
        assert!(json.contains("bezier"));
        assert!(json.contains("animator_rig"));
        assert!(json.contains("save_blend"));

        let parsed: SkeletalAnimationBlenderRiggedV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.clip_name, "idle");
        assert_eq!(parsed.duration_frames, 60);
        assert_eq!(parsed.fps, 30);
        assert!(parsed.r#loop);
        assert_eq!(parsed.ground_offset, 0.05);
        assert_eq!(parsed.poses.len(), 1);
        assert_eq!(parsed.phases.len(), 1);
        assert_eq!(parsed.procedural_layers.len(), 2);
    }

    #[test]
    fn test_conventions_config_serde() {
        let config = ConventionsConfig { strict: true };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"strict\":true"));

        let parsed: ConventionsConfig = serde_json::from_str(&json).unwrap();
        assert!(parsed.strict);

        // Test default
        let default = ConventionsConfig::default();
        assert!(!default.strict);
    }

    #[test]
    fn test_animation_export_settings_default() {
        let export = AnimationExportSettings::default();

        assert!(export.bake_transforms);
        assert!(!export.optimize_keyframes);
        assert!(!export.separate_file);
        assert!(!export.save_blend);
    }

    #[test]
    fn test_animation_export_settings_serde() {
        let export = AnimationExportSettings {
            bake_transforms: false,
            optimize_keyframes: true,
            separate_file: true,
            save_blend: true,
        };

        let json = serde_json::to_string(&export).unwrap();
        assert!(json.contains("\"bake_transforms\":false"));
        assert!(json.contains("\"optimize_keyframes\":true"));
        assert!(json.contains("\"separate_file\":true"));
        assert!(json.contains("\"save_blend\":true"));

        let parsed: AnimationExportSettings = serde_json::from_str(&json).unwrap();
        assert!(!parsed.bake_transforms);
        assert!(parsed.optimize_keyframes);
        assert!(parsed.separate_file);
        assert!(parsed.save_blend);
    }

    // =========================================================================
    // Complete Legacy Key Coverage Tests
    // =========================================================================

    #[test]
    fn test_all_top_level_keys() {
        // Test that all top-level ANIMATION keys can be serialized/deserialized
        let params = SkeletalAnimationBlenderRiggedV1Params {
            skeleton_preset: Some(SkeletonPreset::HumanoidBasicV1), // skeleton_preset
            clip_name: "test".to_string(),                           // name
            input_armature: Some("armature.glb".to_string()),        // input_armature
            character: Some("hero".to_string()),                     // character
            duration_frames: 30,                                      // duration_frames
            duration_seconds: Some(1.0),                              // (alternative)
            fps: 24,                                                  // fps
            r#loop: true,                                             // loop
            ground_offset: 0.1,                                       // ground_offset
            rig_setup: RigSetup::default(),                           // rig_setup
            poses: std::collections::HashMap::new(),                  // poses
            phases: vec![],                                           // phases
            procedural_layers: vec![],                                // procedural_layers
            keyframes: vec![],                                        // (bone_transforms via keyframes)
            ik_keyframes: vec![],                                     // (IK keyframes)
            interpolation: InterpolationMode::Linear,                 // (interpolation)
            export: Some(AnimationExportSettings::default()),         // export settings
            animator_rig: Some(AnimatorRigConfig::default()),         // animator_rig
            save_blend: true,                                         // save_blend
            conventions: Some(ConventionsConfig::default()),          // conventions
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: SkeletalAnimationBlenderRiggedV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.clip_name, "test");
        assert_eq!(parsed.fps, 24);
        assert_eq!(parsed.ground_offset, 0.1);
        assert!(parsed.save_blend);
    }

    #[test]
    fn test_all_rig_setup_components() {
        // Test all components of rig_setup
        let rig = RigSetup {
            presets: vec![IkPreset::HumanoidLegs],                   // presets
            ik_chains: vec![IkChain::new(                             // ik_chains
                "test",
                2,
                IkTargetConfig::new("target"),
            )],
            constraints: ConstraintConfig::new()                      // constraints
                .with_constraint(BoneConstraint::hinge("bone", ConstraintAxis::X, 0.0, 160.0)),
            foot_systems: vec![FootSystem::new(                       // foot_systems
                "foot_l",
                "ik_foot_l",
                "heel_l",
                "toe_l",
            )],
            aim_constraints: vec![AimConstraint::new(                 // aim_constraints
                "aim",
                "bone",
                "target",
            )],
            twist_bones: vec![TwistBone::new("source", "target")],    // twist_bones
            stretch: Some(StretchSettings::enabled()),                // stretch
            bake: Some(BakeSettings::new()),                          // bake
        };

        let json = serde_json::to_string(&rig).unwrap();
        let parsed: RigSetup = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.presets.len(), 1);
        assert_eq!(parsed.ik_chains.len(), 1);
        assert_eq!(parsed.constraints.constraints.len(), 1);
        assert_eq!(parsed.foot_systems.len(), 1);
        assert_eq!(parsed.aim_constraints.len(), 1);
        assert_eq!(parsed.twist_bones.len(), 1);
        assert!(parsed.stretch.is_some());
        assert!(parsed.bake.is_some());
    }
}
