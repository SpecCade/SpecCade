//! Skeletal animation recipe types.

mod animator_rig;
mod clip;
mod common;
mod constraints;
mod ik_setup;
mod pose;
mod procedural;
mod rig;
mod skeletal;

// Re-export all public types
pub use animator_rig::*;
pub use clip::*;
pub use common::*;
pub use constraints::*;
pub use ik_setup::*;
pub use pose::*;
pub use procedural::*;
pub use rig::*;
pub use skeletal::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::character::SkeletonPreset;

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
    fn test_hinge_constraint_serde() {
        let constraint = BoneConstraint::hinge("lower_arm_l", ConstraintAxis::X, 0.0, 160.0);

        let json = serde_json::to_string(&constraint).unwrap();
        assert!(json.contains("\"type\":\"hinge\""));
        assert!(json.contains("\"bone\":\"lower_arm_l\""));
        assert!(json.contains("\"axis\":\"X\""));
        assert!(json.contains("\"min_angle\":0.0"));
        assert!(json.contains("\"max_angle\":160.0"));

        let parsed: BoneConstraint = serde_json::from_str(&json).unwrap();
        if let BoneConstraint::Hinge {
            bone,
            axis,
            min_angle,
            max_angle,
        } = parsed
        {
            assert_eq!(bone, "lower_arm_l");
            assert_eq!(axis, ConstraintAxis::X);
            assert_eq!(min_angle, 0.0);
            assert_eq!(max_angle, 160.0);
        } else {
            panic!("Expected Hinge constraint");
        }
    }

    #[test]
    fn test_hinge_constraint_serde_defaults() {
        // Exercise serde defaults in `constraints.rs` (axis/min/max).
        let json = r#"{"type":"hinge","bone":"knee_l"}"#;
        let parsed: BoneConstraint = serde_json::from_str(json).unwrap();
        match parsed {
            BoneConstraint::Hinge {
                bone,
                axis,
                min_angle,
                max_angle,
            } => {
                assert_eq!(bone, "knee_l");
                assert_eq!(axis, ConstraintAxis::X);
                assert_eq!(min_angle, 0.0);
                assert_eq!(max_angle, 160.0);
            }
            _ => panic!("Expected Hinge constraint"),
        }
    }

    #[test]
    fn test_bone_constraint_validation_errors() {
        let empty = BoneConstraint::hinge("", ConstraintAxis::X, 0.0, 160.0);
        assert_eq!(empty.validate(), Err(BoneConstraintError::EmptyBoneName));

        let bad_range = BoneConstraint::ball("upper_arm_l", 45.0, 10.0, 0.0);
        assert_eq!(
            bad_range.validate(),
            Err(BoneConstraintError::InvalidAngleRange {
                min: 10.0,
                max: 0.0
            })
        );

        let bad_cone = BoneConstraint::ball("upper_arm_l", 200.0, -45.0, 45.0);
        assert_eq!(
            bad_cone.validate(),
            Err(BoneConstraintError::InvalidConeAngle(200.0))
        );

        let bad_stiffness = BoneConstraint::soft("tail", 1.1, 0.5);
        assert_eq!(
            bad_stiffness.validate(),
            Err(BoneConstraintError::InvalidStiffness(1.1))
        );

        let bad_damping = BoneConstraint::soft("tail", 0.5, -0.1);
        assert_eq!(
            bad_damping.validate(),
            Err(BoneConstraintError::InvalidDamping(-0.1))
        );
    }

    #[test]
    fn test_constraint_config_serde() {
        let config = ConstraintConfig::new()
            .with_constraint(BoneConstraint::hinge(
                "lower_arm_l",
                ConstraintAxis::X,
                0.0,
                160.0,
            ))
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
    fn test_rig_setup_with_constraints() {
        let rig_setup = RigSetup::new()
            .with_preset(IkPreset::HumanoidArms)
            .with_constraint(BoneConstraint::hinge(
                "lower_arm_l",
                ConstraintAxis::X,
                0.0,
                160.0,
            ))
            .with_constraint(BoneConstraint::hinge(
                "lower_arm_r",
                ConstraintAxis::X,
                0.0,
                160.0,
            ))
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
    fn test_foot_system_serde() {
        let foot =
            FootSystem::new("foot_l", "ik_foot_l", "heel_l", "toe_l").with_ball_bone("ball_l");

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

    #[test]
    fn test_foot_system_roll_limits_serde_default() {
        // If roll_limits is omitted, the default should still be applied.
        let json =
            r#"{"name":"foot_l","ik_target":"ik_foot_l","heel_bone":"heel_l","toe_bone":"toe_l"}"#;
        let foot: FootSystem = serde_json::from_str(json).unwrap();
        assert_eq!(foot.roll_limits, [-30.0, 60.0]);
    }

    // =========================================================================
    // Aim Constraint Tests
    // =========================================================================

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
    fn test_stretch_settings_enabled() {
        let stretch = StretchSettings::enabled();

        assert!(stretch.enabled);
        assert_eq!(stretch.max_stretch, 1.5);
        assert_eq!(stretch.min_stretch, 0.5);
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
            keyframes: vec![],    // (bone_transforms via keyframes)
            ik_keyframes: vec![], // (IK keyframes)
            interpolation: InterpolationMode::Linear, // (interpolation)
            export: Some(AnimationExportSettings::default()), // export settings
            animator_rig: Some(AnimatorRigConfig::default()), // animator_rig
            save_blend: true,     // save_blend
            conventions: Some(ConventionsConfig::default()), // conventions
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
}
