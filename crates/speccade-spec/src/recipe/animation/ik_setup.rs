//! IK preset setup functions.

use super::skeletal::{
    IkChain, IkPreset, IkTargetConfig, PoleConfig,
};

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
