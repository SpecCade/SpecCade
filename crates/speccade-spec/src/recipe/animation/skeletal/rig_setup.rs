//! Complete rig setup configuration for skeletal animation.

use serde::{Deserialize, Serialize};

use super::super::constraints::{BoneConstraint, BoneConstraintError, ConstraintConfig};
use super::super::ik_setup::setup_ik_preset;
use super::bone_constraints::{AimConstraint, TwistBone};
use super::foot::FootSystem;
use super::ik_chain::{IkChain, IkChainError, IkPreset};
use super::settings::{BakeSettings, StretchSettings};

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
