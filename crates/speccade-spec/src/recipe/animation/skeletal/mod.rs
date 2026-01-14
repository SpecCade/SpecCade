//! Skeletal animation and IK rig types.

mod bone_constraints;
mod foot;
mod ik_chain;
mod rig_setup;
mod settings;

// Re-export all public types to preserve the original API
pub use bone_constraints::{AimConstraint, TwistBone};
pub use foot::FootSystem;
pub use ik_chain::{IkChain, IkChainError, IkPreset, IkTargetConfig, PoleConfig};
pub use rig_setup::RigSetup;
pub use settings::{BakeSettings, StretchSettings, VolumePreservation};
