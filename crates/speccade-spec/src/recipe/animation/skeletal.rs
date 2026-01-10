//! Skeletal animation and IK rig types.

use serde::{Deserialize, Serialize};

use super::common::{AimAxis, ConstraintAxis};
use super::constraints::{BoneConstraint, BoneConstraintError, ConstraintConfig};
use super::ik_setup::setup_ik_preset;

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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

// IK Preset Setup Functions are in ik_setup.rs

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

// =============================================================================
// Stretch Settings
// =============================================================================

/// Stretch settings for IK chains.
/// Allows bones to stretch beyond their rest length.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

fn default_true() -> bool {
    true
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

// =============================================================================
// Rig Setup Configuration
// =============================================================================

/// Complete rig setup configuration for an armature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
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
