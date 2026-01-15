//! Integration tests for complete animation parameters and rig setup.

use crate::recipe::{animation::*, character::SkeletonPreset};

// =========================================================================
// Rig Setup Integration Tests
// =========================================================================

#[test]
fn test_rig_setup_complete() {
    let rig_setup = RigSetup::new()
        .with_preset(IkPreset::HumanoidLegs)
        .with_preset(IkPreset::HumanoidArms)
        .with_chain(IkChain::new(
            "ik_spine",
            3,
            IkTargetConfig::new("ik_spine_tip"),
        ))
        .with_constraint(BoneConstraint::hinge(
            "lower_arm_l",
            ConstraintAxis::X,
            0.0,
            160.0,
        ))
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

    let phases = vec![AnimationPhase::new(0, 30)
        .with_name("start")
        .with_curve(TimingCurve::EaseIn)
        .with_pose("standing")];

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
        animator_rig: Some(
            AnimatorRigConfig::new()
                .with_display(ArmatureDisplay::Stick)
                .with_widget_style(WidgetStyle::WireDiamond),
        ),
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

// =========================================================================
// Complete Legacy Key Coverage Tests
// =========================================================================

#[test]
fn test_all_top_level_keys() {
    // Test that all top-level ANIMATION keys can be serialized/deserialized
    let params = SkeletalAnimationBlenderRiggedV1Params {
        skeleton_preset: Some(SkeletonPreset::HumanoidBasicV1), // skeleton_preset
        clip_name: "test".to_string(),                          // name
        input_armature: Some("armature.glb".to_string()),       // input_armature
        character: Some("hero".to_string()),                    // character
        duration_frames: 30,                                    // duration_frames
        duration_seconds: Some(1.0),                            // (alternative)
        fps: 24,                                                // fps
        r#loop: true,                                           // loop
        ground_offset: 0.1,                                     // ground_offset
        rig_setup: RigSetup::default(),                         // rig_setup
        poses: std::collections::HashMap::new(),                // poses
        phases: vec![],                                         // phases
        procedural_layers: vec![],                              // procedural_layers
        keyframes: vec![],                                      // (bone_transforms via keyframes)
        ik_keyframes: vec![],                                   // (IK keyframes)
        interpolation: InterpolationMode::Linear,               // (interpolation)
        export: Some(AnimationExportSettings::default()),       // export settings
        animator_rig: Some(AnimatorRigConfig::default()),       // animator_rig
        save_blend: true,                                       // save_blend
        conventions: Some(ConventionsConfig::default()),        // conventions
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
    let rig =
        RigSetup {
            presets: vec![IkPreset::HumanoidLegs], // presets
            ik_chains: vec![IkChain::new(
                // ik_chains
                "test",
                2,
                IkTargetConfig::new("target"),
            )],
            constraints:
                ConstraintConfig::new() // constraints
                    .with_constraint(BoneConstraint::hinge(
                        "bone",
                        ConstraintAxis::X,
                        0.0,
                        160.0,
                    )),
            foot_systems: vec![FootSystem::new(
                // foot_systems
                "foot_l",
                "ik_foot_l",
                "heel_l",
                "toe_l",
            )],
            aim_constraints: vec![AimConstraint::new(
                // aim_constraints
                "aim", "bone", "target",
            )],
            twist_bones: vec![TwistBone::new("source", "target")], // twist_bones
            stretch: Some(StretchSettings::enabled()),             // stretch
            bake: Some(BakeSettings::new()),                       // bake
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
