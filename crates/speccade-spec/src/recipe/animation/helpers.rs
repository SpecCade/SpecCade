//! Animation helper presets for procedural locomotion cycles.
//!
//! This module provides preset-based animation helpers that generate
//! IK targets and constraints for common locomotion patterns like
//! walk cycles, run cycles, and idle sway animations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::skeletal::{FootSystem, IkChain, IkPreset};

// =============================================================================
// Animation Helper Preset Type
// =============================================================================

/// Available animation helper presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationHelperPreset {
    /// Walk cycle animation with foot plants and arm swing.
    WalkCycle,
    /// Run cycle animation with faster timing and more dynamic motion.
    RunCycle,
    /// Idle sway animation with subtle breathing and weight shifting.
    IdleSway,
}

impl AnimationHelperPreset {
    /// Returns the preset as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            AnimationHelperPreset::WalkCycle => "walk_cycle",
            AnimationHelperPreset::RunCycle => "run_cycle",
            AnimationHelperPreset::IdleSway => "idle_sway",
        }
    }

    /// Returns the default cycle frames for this preset at 30 fps.
    pub fn default_cycle_frames(&self) -> u32 {
        match self {
            AnimationHelperPreset::WalkCycle => 60,  // 2 seconds
            AnimationHelperPreset::RunCycle => 30,   // 1 second
            AnimationHelperPreset::IdleSway => 120,  // 4 seconds
        }
    }

    /// Returns whether this preset requires foot roll systems.
    pub fn uses_foot_roll(&self) -> bool {
        matches!(
            self,
            AnimationHelperPreset::WalkCycle | AnimationHelperPreset::RunCycle
        )
    }

    /// Returns the default stride length for locomotion presets.
    pub fn default_stride_length(&self) -> f64 {
        match self {
            AnimationHelperPreset::WalkCycle => 0.8,
            AnimationHelperPreset::RunCycle => 1.2,
            AnimationHelperPreset::IdleSway => 0.0, // No forward motion
        }
    }

    /// Returns the default arm swing amplitude (0.0-1.0).
    pub fn default_arm_swing(&self) -> f64 {
        match self {
            AnimationHelperPreset::WalkCycle => 0.3,
            AnimationHelperPreset::RunCycle => 0.5,
            AnimationHelperPreset::IdleSway => 0.05,
        }
    }
}

impl std::fmt::Display for AnimationHelperPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =============================================================================
// Skeleton Type
// =============================================================================

/// Skeleton types for animation helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkeletonType {
    /// Standard humanoid rig with arms and legs.
    #[default]
    Humanoid,
    /// Quadruped rig with four legs.
    Quadruped,
}

impl SkeletonType {
    /// Returns the IK presets for this skeleton type.
    pub fn ik_presets(&self) -> Vec<IkPreset> {
        match self {
            SkeletonType::Humanoid => vec![IkPreset::HumanoidLegs, IkPreset::HumanoidArms],
            SkeletonType::Quadruped => {
                vec![IkPreset::QuadrupedForelegs, IkPreset::QuadrupedHindlegs]
            }
        }
    }
}

// =============================================================================
// IK Target Configuration
// =============================================================================

/// Per-limb IK target configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IkTargetSettings {
    /// Pole angle in degrees for knee/elbow direction.
    #[serde(default = "default_pole_angle")]
    pub pole_angle: f64,
    /// Number of bones in the IK chain.
    #[serde(default = "default_chain_length")]
    pub chain_length: u8,
}

fn default_pole_angle() -> f64 {
    90.0
}

fn default_chain_length() -> u8 {
    2
}

impl Default for IkTargetSettings {
    fn default() -> Self {
        Self {
            pole_angle: default_pole_angle(),
            chain_length: default_chain_length(),
        }
    }
}

impl IkTargetSettings {
    /// Creates new IK target settings with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the pole angle.
    pub fn with_pole_angle(mut self, angle: f64) -> Self {
        self.pole_angle = angle;
        self
    }

    /// Sets the chain length.
    pub fn with_chain_length(mut self, length: u8) -> Self {
        self.chain_length = length.max(1);
        self
    }
}

// =============================================================================
// Cycle Settings
// =============================================================================

/// Settings for locomotion cycle generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CycleSettings {
    /// Distance traveled per cycle in world units.
    #[serde(default = "default_stride_length")]
    pub stride_length: f64,
    /// Number of frames in one complete cycle.
    #[serde(default = "default_cycle_frames")]
    pub cycle_frames: u32,
    /// Whether to generate foot roll systems.
    #[serde(default = "default_foot_roll")]
    pub foot_roll: bool,
    /// Arm swing amplitude (0.0-1.0).
    #[serde(default = "default_arm_swing")]
    pub arm_swing: f64,
    /// Hip sway amplitude in degrees.
    #[serde(default = "default_hip_sway")]
    pub hip_sway: f64,
    /// Spine twist amplitude in degrees.
    #[serde(default = "default_spine_twist")]
    pub spine_twist: f64,
    /// Maximum foot lift height in world units.
    #[serde(default = "default_foot_lift")]
    pub foot_lift: f64,
}

fn default_stride_length() -> f64 {
    0.8
}

fn default_cycle_frames() -> u32 {
    60
}

fn default_foot_roll() -> bool {
    true
}

fn default_arm_swing() -> f64 {
    0.3
}

fn default_hip_sway() -> f64 {
    3.0
}

fn default_spine_twist() -> f64 {
    5.0
}

fn default_foot_lift() -> f64 {
    0.15
}

impl Default for CycleSettings {
    fn default() -> Self {
        Self {
            stride_length: default_stride_length(),
            cycle_frames: default_cycle_frames(),
            foot_roll: default_foot_roll(),
            arm_swing: default_arm_swing(),
            hip_sway: default_hip_sway(),
            spine_twist: default_spine_twist(),
            foot_lift: default_foot_lift(),
        }
    }
}

impl CycleSettings {
    /// Creates new cycle settings with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates settings optimized for a walk cycle.
    pub fn walk_cycle() -> Self {
        Self {
            stride_length: 0.8,
            cycle_frames: 60,
            foot_roll: true,
            arm_swing: 0.3,
            hip_sway: 3.0,
            spine_twist: 5.0,
            foot_lift: 0.15,
        }
    }

    /// Creates settings optimized for a run cycle.
    pub fn run_cycle() -> Self {
        Self {
            stride_length: 1.2,
            cycle_frames: 30,
            foot_roll: true,
            arm_swing: 0.5,
            hip_sway: 5.0,
            spine_twist: 8.0,
            foot_lift: 0.25,
        }
    }

    /// Creates settings optimized for idle sway.
    pub fn idle_sway() -> Self {
        Self {
            stride_length: 0.0,
            cycle_frames: 120,
            foot_roll: false,
            arm_swing: 0.05,
            hip_sway: 2.0,
            spine_twist: 1.0,
            foot_lift: 0.0,
        }
    }

    /// Sets the stride length.
    pub fn with_stride_length(mut self, length: f64) -> Self {
        self.stride_length = length.max(0.0);
        self
    }

    /// Sets the cycle frame count.
    pub fn with_cycle_frames(mut self, frames: u32) -> Self {
        self.cycle_frames = frames.max(1);
        self
    }

    /// Enables or disables foot roll.
    pub fn with_foot_roll(mut self, enabled: bool) -> Self {
        self.foot_roll = enabled;
        self
    }

    /// Sets the arm swing amplitude.
    pub fn with_arm_swing(mut self, amplitude: f64) -> Self {
        self.arm_swing = amplitude.clamp(0.0, 1.0);
        self
    }

    /// Sets the hip sway amplitude.
    pub fn with_hip_sway(mut self, degrees: f64) -> Self {
        self.hip_sway = degrees.abs();
        self
    }

    /// Sets the foot lift height.
    pub fn with_foot_lift(mut self, height: f64) -> Self {
        self.foot_lift = height.max(0.0);
        self
    }
}

// =============================================================================
// Animation Helpers Params
// =============================================================================

/// Parameters for the `skeletal_animation.helpers_v1` recipe.
///
/// This recipe generates procedural locomotion animations using IK targets
/// and constraint presets. It creates cyclic animations suitable for
/// walk cycles, run cycles, and idle animations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationHelpersV1Params {
    /// Skeleton type (humanoid or quadruped).
    #[serde(default)]
    pub skeleton: SkeletonType,
    /// Animation preset to apply.
    pub preset: AnimationHelperPreset,
    /// Cycle settings for the animation.
    #[serde(default)]
    pub settings: CycleSettings,
    /// Per-limb IK target configurations.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub ik_targets: HashMap<String, IkTargetSettings>,
    /// Animation clip name.
    #[serde(default = "default_clip_name")]
    pub clip_name: String,
    /// Frames per second.
    #[serde(default = "default_fps")]
    pub fps: u8,
    /// Save .blend file alongside output.
    #[serde(default)]
    pub save_blend: bool,
}

fn default_clip_name() -> String {
    "locomotion".to_string()
}

fn default_fps() -> u8 {
    30
}

impl AnimationHelpersV1Params {
    /// Creates new animation helpers params with the given preset.
    pub fn new(preset: AnimationHelperPreset) -> Self {
        let settings = match preset {
            AnimationHelperPreset::WalkCycle => CycleSettings::walk_cycle(),
            AnimationHelperPreset::RunCycle => CycleSettings::run_cycle(),
            AnimationHelperPreset::IdleSway => CycleSettings::idle_sway(),
        };

        Self {
            skeleton: SkeletonType::Humanoid,
            preset,
            settings,
            ik_targets: HashMap::new(),
            clip_name: preset.as_str().to_string(),
            fps: 30,
            save_blend: false,
        }
    }

    /// Creates walk cycle params.
    pub fn walk_cycle() -> Self {
        Self::new(AnimationHelperPreset::WalkCycle)
    }

    /// Creates run cycle params.
    pub fn run_cycle() -> Self {
        Self::new(AnimationHelperPreset::RunCycle)
    }

    /// Creates idle sway params.
    pub fn idle_sway() -> Self {
        Self::new(AnimationHelperPreset::IdleSway)
    }

    /// Sets the skeleton type.
    pub fn with_skeleton(mut self, skeleton: SkeletonType) -> Self {
        self.skeleton = skeleton;
        self
    }

    /// Sets the cycle settings.
    pub fn with_settings(mut self, settings: CycleSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Adds an IK target configuration.
    pub fn with_ik_target(mut self, limb: impl Into<String>, settings: IkTargetSettings) -> Self {
        self.ik_targets.insert(limb.into(), settings);
        self
    }

    /// Sets the clip name.
    pub fn with_clip_name(mut self, name: impl Into<String>) -> Self {
        self.clip_name = name.into();
        self
    }

    /// Sets the frames per second.
    pub fn with_fps(mut self, fps: u8) -> Self {
        self.fps = fps.max(1);
        self
    }

    /// Enables saving the .blend file.
    pub fn with_save_blend(mut self, save: bool) -> Self {
        self.save_blend = save;
        self
    }

    /// Returns the total animation duration in seconds.
    pub fn duration_seconds(&self) -> f64 {
        self.settings.cycle_frames as f64 / self.fps as f64
    }

    /// Generates the IK chains for this animation.
    pub fn generate_ik_chains(&self) -> Vec<IkChain> {
        let mut chains = Vec::new();

        // Get default chains from skeleton type
        for preset in self.skeleton.ik_presets() {
            let default_chains = super::ik_setup::setup_ik_preset(preset);
            chains.extend(default_chains);
        }

        // Apply custom IK target settings
        for (limb, settings) in &self.ik_targets {
            for chain in &mut chains {
                if chain.name.contains(limb) || chain.target.name.contains(limb) {
                    chain.chain_length = settings.chain_length;
                    if let Some(pole) = &mut chain.pole {
                        pole.angle = settings.pole_angle;
                    }
                }
            }
        }

        chains
    }

    /// Generates foot roll systems if enabled.
    pub fn generate_foot_systems(&self) -> Vec<FootSystem> {
        if !self.settings.foot_roll {
            return Vec::new();
        }

        match self.skeleton {
            SkeletonType::Humanoid => vec![
                FootSystem::new("foot_l", "ik_foot_l", "heel_l", "toe_l")
                    .with_ball_bone("ball_l")
                    .with_roll_limits(-30.0, 60.0),
                FootSystem::new("foot_r", "ik_foot_r", "heel_r", "toe_r")
                    .with_ball_bone("ball_r")
                    .with_roll_limits(-30.0, 60.0),
            ],
            SkeletonType::Quadruped => vec![
                FootSystem::new("front_paw_l", "ik_front_paw_l", "front_heel_l", "front_toe_l"),
                FootSystem::new("front_paw_r", "ik_front_paw_r", "front_heel_r", "front_toe_r"),
                FootSystem::new("back_paw_l", "ik_back_paw_l", "back_heel_l", "back_toe_l"),
                FootSystem::new("back_paw_r", "ik_back_paw_r", "back_heel_r", "back_toe_r"),
            ],
        }
    }

    /// Validates the parameters.
    pub fn validate(&self) -> Result<(), AnimationHelpersError> {
        if self.settings.cycle_frames == 0 {
            return Err(AnimationHelpersError::InvalidCycleFrames);
        }
        if self.fps == 0 {
            return Err(AnimationHelpersError::InvalidFps);
        }
        if self.clip_name.is_empty() {
            return Err(AnimationHelpersError::EmptyClipName);
        }

        // Validate IK target settings
        for (limb, settings) in &self.ik_targets {
            if settings.chain_length == 0 {
                return Err(AnimationHelpersError::InvalidChainLength(limb.clone()));
            }
        }

        Ok(())
    }
}

// =============================================================================
// Error Types
// =============================================================================

/// Errors that can occur when validating animation helpers params.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimationHelpersError {
    /// Cycle frames must be at least 1.
    InvalidCycleFrames,
    /// FPS must be at least 1.
    InvalidFps,
    /// Clip name cannot be empty.
    EmptyClipName,
    /// Chain length must be at least 1.
    InvalidChainLength(String),
}

impl std::fmt::Display for AnimationHelpersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimationHelpersError::InvalidCycleFrames => {
                write!(f, "cycle_frames must be at least 1")
            }
            AnimationHelpersError::InvalidFps => {
                write!(f, "fps must be at least 1")
            }
            AnimationHelpersError::EmptyClipName => {
                write!(f, "clip_name cannot be empty")
            }
            AnimationHelpersError::InvalidChainLength(limb) => {
                write!(f, "chain_length for '{}' must be at least 1", limb)
            }
        }
    }
}

impl std::error::Error for AnimationHelpersError {}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_helper_preset_serde() {
        let presets = [
            (AnimationHelperPreset::WalkCycle, "\"walk_cycle\""),
            (AnimationHelperPreset::RunCycle, "\"run_cycle\""),
            (AnimationHelperPreset::IdleSway, "\"idle_sway\""),
        ];

        for (preset, expected) in presets {
            let json = serde_json::to_string(&preset).unwrap();
            assert_eq!(json, expected);

            let parsed: AnimationHelperPreset = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, preset);
        }
    }

    #[test]
    fn test_animation_helper_preset_defaults() {
        assert_eq!(AnimationHelperPreset::WalkCycle.default_cycle_frames(), 60);
        assert_eq!(AnimationHelperPreset::RunCycle.default_cycle_frames(), 30);
        assert_eq!(AnimationHelperPreset::IdleSway.default_cycle_frames(), 120);

        assert!(AnimationHelperPreset::WalkCycle.uses_foot_roll());
        assert!(AnimationHelperPreset::RunCycle.uses_foot_roll());
        assert!(!AnimationHelperPreset::IdleSway.uses_foot_roll());
    }

    #[test]
    fn test_skeleton_type_serde() {
        let humanoid = SkeletonType::Humanoid;
        let json = serde_json::to_string(&humanoid).unwrap();
        assert_eq!(json, "\"humanoid\"");

        let quadruped = SkeletonType::Quadruped;
        let json = serde_json::to_string(&quadruped).unwrap();
        assert_eq!(json, "\"quadruped\"");
    }

    #[test]
    fn test_skeleton_type_ik_presets() {
        let humanoid_presets = SkeletonType::Humanoid.ik_presets();
        assert!(humanoid_presets.contains(&IkPreset::HumanoidLegs));
        assert!(humanoid_presets.contains(&IkPreset::HumanoidArms));

        let quadruped_presets = SkeletonType::Quadruped.ik_presets();
        assert!(quadruped_presets.contains(&IkPreset::QuadrupedForelegs));
        assert!(quadruped_presets.contains(&IkPreset::QuadrupedHindlegs));
    }

    #[test]
    fn test_ik_target_settings() {
        let settings = IkTargetSettings::new()
            .with_pole_angle(45.0)
            .with_chain_length(3);

        assert_eq!(settings.pole_angle, 45.0);
        assert_eq!(settings.chain_length, 3);

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("45.0"));
        assert!(json.contains("3"));
    }

    #[test]
    fn test_cycle_settings_presets() {
        let walk = CycleSettings::walk_cycle();
        assert_eq!(walk.stride_length, 0.8);
        assert_eq!(walk.cycle_frames, 60);
        assert!(walk.foot_roll);

        let run = CycleSettings::run_cycle();
        assert_eq!(run.stride_length, 1.2);
        assert_eq!(run.cycle_frames, 30);
        assert!(run.foot_roll);

        let idle = CycleSettings::idle_sway();
        assert_eq!(idle.stride_length, 0.0);
        assert_eq!(idle.cycle_frames, 120);
        assert!(!idle.foot_roll);
    }

    #[test]
    fn test_cycle_settings_builder() {
        let settings = CycleSettings::new()
            .with_stride_length(1.0)
            .with_cycle_frames(48)
            .with_foot_roll(false)
            .with_arm_swing(0.4)
            .with_hip_sway(4.0)
            .with_foot_lift(0.2);

        assert_eq!(settings.stride_length, 1.0);
        assert_eq!(settings.cycle_frames, 48);
        assert!(!settings.foot_roll);
        assert_eq!(settings.arm_swing, 0.4);
        assert_eq!(settings.hip_sway, 4.0);
        assert_eq!(settings.foot_lift, 0.2);
    }

    #[test]
    fn test_animation_helpers_v1_params_walk() {
        let params = AnimationHelpersV1Params::walk_cycle();

        assert_eq!(params.preset, AnimationHelperPreset::WalkCycle);
        assert_eq!(params.skeleton, SkeletonType::Humanoid);
        assert_eq!(params.settings.stride_length, 0.8);
        assert!(params.settings.foot_roll);
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_animation_helpers_v1_params_run() {
        let params = AnimationHelpersV1Params::run_cycle();

        assert_eq!(params.preset, AnimationHelperPreset::RunCycle);
        assert_eq!(params.settings.stride_length, 1.2);
        assert_eq!(params.settings.cycle_frames, 30);
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_animation_helpers_v1_params_idle() {
        let params = AnimationHelpersV1Params::idle_sway();

        assert_eq!(params.preset, AnimationHelperPreset::IdleSway);
        assert_eq!(params.settings.stride_length, 0.0);
        assert!(!params.settings.foot_roll);
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_animation_helpers_v1_params_with_ik_targets() {
        let params = AnimationHelpersV1Params::walk_cycle()
            .with_ik_target("foot_l", IkTargetSettings::new().with_pole_angle(90.0))
            .with_ik_target("foot_r", IkTargetSettings::new().with_pole_angle(90.0));

        assert_eq!(params.ik_targets.len(), 2);
        assert!(params.ik_targets.contains_key("foot_l"));
        assert!(params.ik_targets.contains_key("foot_r"));
    }

    #[test]
    fn test_animation_helpers_v1_params_serde() {
        let params = AnimationHelpersV1Params::walk_cycle()
            .with_clip_name("character_walk")
            .with_fps(24)
            .with_save_blend(true);

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("walk_cycle"));
        assert!(json.contains("character_walk"));
        assert!(json.contains("24"));
        assert!(json.contains("true"));

        let parsed: AnimationHelpersV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.preset, AnimationHelperPreset::WalkCycle);
        assert_eq!(parsed.clip_name, "character_walk");
        assert_eq!(parsed.fps, 24);
        assert!(parsed.save_blend);
    }

    #[test]
    fn test_animation_helpers_v1_params_duration() {
        let params = AnimationHelpersV1Params::walk_cycle()
            .with_settings(CycleSettings::new().with_cycle_frames(60))
            .with_fps(30);

        assert_eq!(params.duration_seconds(), 2.0);
    }

    #[test]
    fn test_animation_helpers_v1_params_generate_ik_chains() {
        let params = AnimationHelpersV1Params::walk_cycle();
        let chains = params.generate_ik_chains();

        // Should have at least leg chains
        assert!(!chains.is_empty());

        // Check for leg IK chains
        let has_leg_l = chains.iter().any(|c| c.name.contains("leg_l"));
        let has_leg_r = chains.iter().any(|c| c.name.contains("leg_r"));
        assert!(has_leg_l || chains.iter().any(|c| c.target.name.contains("foot_l")));
        assert!(has_leg_r || chains.iter().any(|c| c.target.name.contains("foot_r")));
    }

    #[test]
    fn test_animation_helpers_v1_params_generate_foot_systems() {
        let params = AnimationHelpersV1Params::walk_cycle();
        let foot_systems = params.generate_foot_systems();

        // Walk cycle should have foot systems
        assert_eq!(foot_systems.len(), 2);

        let idle = AnimationHelpersV1Params::idle_sway();
        let no_foot_systems = idle.generate_foot_systems();

        // Idle should not have foot systems
        assert!(no_foot_systems.is_empty());
    }

    #[test]
    fn test_animation_helpers_v1_params_validation() {
        // Valid params
        let valid = AnimationHelpersV1Params::walk_cycle();
        assert!(valid.validate().is_ok());

        // Invalid cycle frames
        let mut invalid_frames = AnimationHelpersV1Params::walk_cycle();
        invalid_frames.settings.cycle_frames = 0;
        assert_eq!(
            invalid_frames.validate(),
            Err(AnimationHelpersError::InvalidCycleFrames)
        );

        // Invalid FPS
        let mut invalid_fps = AnimationHelpersV1Params::walk_cycle();
        invalid_fps.fps = 0;
        assert_eq!(
            invalid_fps.validate(),
            Err(AnimationHelpersError::InvalidFps)
        );

        // Empty clip name
        let mut empty_name = AnimationHelpersV1Params::walk_cycle();
        empty_name.clip_name = String::new();
        assert_eq!(
            empty_name.validate(),
            Err(AnimationHelpersError::EmptyClipName)
        );

        // Invalid chain length - use direct construction to bypass clamp in with_chain_length
        let mut invalid_chain = AnimationHelpersV1Params::walk_cycle();
        invalid_chain.ik_targets.insert(
            "foot_l".to_string(),
            IkTargetSettings {
                pole_angle: 90.0,
                chain_length: 0,
            },
        );
        assert!(matches!(
            invalid_chain.validate(),
            Err(AnimationHelpersError::InvalidChainLength(_))
        ));
    }

    #[test]
    fn test_animation_helpers_v1_full_spec_json() {
        let json = r#"{
            "skeleton": "humanoid",
            "preset": "walk_cycle",
            "settings": {
                "stride_length": 0.8,
                "cycle_frames": 24,
                "foot_roll": true,
                "arm_swing": 0.3
            },
            "ik_targets": {
                "foot_l": { "pole_angle": 90, "chain_length": 2 },
                "foot_r": { "pole_angle": 90, "chain_length": 2 }
            },
            "clip_name": "walk",
            "fps": 30,
            "save_blend": false
        }"#;

        let params: AnimationHelpersV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.skeleton, SkeletonType::Humanoid);
        assert_eq!(params.preset, AnimationHelperPreset::WalkCycle);
        assert_eq!(params.settings.stride_length, 0.8);
        assert_eq!(params.settings.cycle_frames, 24);
        assert!(params.settings.foot_roll);
        assert_eq!(params.ik_targets.len(), 2);
        assert_eq!(params.clip_name, "walk");
        assert!(params.validate().is_ok());
    }
}
