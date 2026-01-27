//! Tests for IK chain setup and configuration.

use crate::recipe::{animation::*, character::SkeletonPreset};

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
        assert!(
            !chains.is_empty(),
            "Preset {:?} should create chains",
            preset
        );
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
        root_motion: None,
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
        animator_rig: Some(
            AnimatorRigConfig::new()
                .with_widget_style(WidgetStyle::WireDiamond)
                .with_bone_colors(BoneColorScheme::Standard),
        ),
        save_blend: false,
        conventions: None,
        root_motion: None,
    };

    let json = serde_json::to_string(&params).unwrap();
    assert!(json.contains("animator_rig"));
    assert!(json.contains("wire_diamond"));

    let parsed: SkeletalAnimationBlenderRiggedV1Params = serde_json::from_str(&json).unwrap();
    assert!(parsed.animator_rig.is_some());
    let rig = parsed.animator_rig.unwrap();
    assert_eq!(rig.widget_style, WidgetStyle::WireDiamond);
}
