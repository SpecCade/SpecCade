//! Skeletal animation and IK rig types.

mod bone_constraints;
mod finger_controls;
mod foot;
mod ik_chain;
mod ikfk_switch;
mod rig_setup;
mod settings;
mod space_switch;

// Re-export all public types to preserve the original API
pub use bone_constraints::{AimConstraint, TwistBone};
pub use finger_controls::{
    FingerControls, FingerControlsError, FingerKeyframe, FingerName, FingerPose, HandSide,
};
pub use foot::FootSystem;
pub use ik_chain::{IkChain, IkChainError, IkPreset, IkTargetConfig, PoleConfig};
pub use ikfk_switch::{IkFkKeyframe, IkFkMode, IkFkSwitch, IkFkSwitchError};
pub use rig_setup::RigSetup;
pub use settings::{BakeSettings, StretchSettings, VolumePreservation};
pub use space_switch::{
    ParentSpace, SpaceKind, SpaceSwitch, SpaceSwitchError, SpaceSwitchKeyframe,
};
