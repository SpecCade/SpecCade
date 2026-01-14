//! IK chain types and configuration.

use serde::{Deserialize, Serialize};

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
            IkPreset::HumanoidLegs => Some([0.0, 0.3, 0.0]), // Forward of knee
            IkPreset::HumanoidArms => Some([0.0, -0.3, 0.0]), // Behind elbow
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
                write!(
                    f,
                    "Pole target cannot have both position and bone specified"
                )
            }
        }
    }
}

impl std::error::Error for IkChainError {}
