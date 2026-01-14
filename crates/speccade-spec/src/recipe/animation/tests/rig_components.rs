//! Tests for rig component types (foot systems, aim constraints, twist bones, stretch, bake).

use crate::recipe::animation::*;

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
