//! Tests for bone constraints.

use crate::recipe::animation::*;

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
