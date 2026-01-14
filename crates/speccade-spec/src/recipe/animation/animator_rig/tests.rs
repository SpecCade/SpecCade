//! Tests for animator rig types.

use super::*;

#[test]
fn test_widget_style_serde() {
    // Test all variants serialize correctly
    let styles = [
        (WidgetStyle::WireCircle, "\"wire_circle\""),
        (WidgetStyle::WireCube, "\"wire_cube\""),
        (WidgetStyle::WireSphere, "\"wire_sphere\""),
        (WidgetStyle::WireDiamond, "\"wire_diamond\""),
        (WidgetStyle::CustomMesh, "\"custom_mesh\""),
    ];
    for (style, expected) in styles {
        let json = serde_json::to_string(&style).unwrap();
        assert_eq!(json, expected);
    }

    // Test default
    assert_eq!(WidgetStyle::default(), WidgetStyle::WireCircle);

    // Test blender_name
    assert_eq!(WidgetStyle::WireCircle.blender_name(), "WGT_circle");
    assert_eq!(WidgetStyle::WireDiamond.blender_name(), "WGT_diamond");

    // Test standard_styles
    let standard = WidgetStyle::standard_styles();
    assert_eq!(standard.len(), 4);
    assert!(!standard.contains(&WidgetStyle::CustomMesh));
}

#[test]
fn test_bone_collection_serde() {
    let collection = BoneCollection::new("IK Controls")
        .with_bone("ik_foot_l")
        .with_bone("ik_foot_r")
        .with_visibility(true)
        .with_selectability(true);

    let json = serde_json::to_string(&collection).unwrap();
    assert!(json.contains("IK Controls"));
    assert!(json.contains("ik_foot_l"));
    assert!(json.contains("ik_foot_r"));

    let parsed: BoneCollection = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "IK Controls");
    assert_eq!(parsed.bones.len(), 2);
    assert!(parsed.visible);
    assert!(parsed.selectable);
}

#[test]
fn test_bone_collection_with_bones() {
    let collection =
        BoneCollection::new("Deform").with_bones(["arm_l", "arm_r", "leg_l", "leg_r"]);

    assert_eq!(collection.bones.len(), 4);
    assert!(collection.bones.contains(&"arm_l".to_string()));
}

#[test]
fn test_bone_collection_validation() {
    // Valid collection
    let valid = BoneCollection::new("Test");
    assert!(valid.validate().is_ok());

    // Invalid - empty name
    let invalid = BoneCollection::new("");
    assert_eq!(
        invalid.validate(),
        Err(AnimatorRigError::EmptyCollectionName)
    );
}

#[test]
fn test_bone_collection_preset() {
    // Test all presets
    let presets = [
        (BoneCollectionPreset::IkControls, "IK Controls", true, true),
        (BoneCollectionPreset::FkControls, "FK Controls", true, true),
        (BoneCollectionPreset::Deform, "Deform", false, false),
        (BoneCollectionPreset::Mechanism, "Mechanism", false, false),
    ];

    for (preset, name, visible, selectable) in presets {
        assert_eq!(preset.default_name(), name);
        assert_eq!(preset.default_visibility(), visible);
        assert_eq!(preset.default_selectability(), selectable);

        let collection = preset.to_collection();
        assert_eq!(collection.name, name);
        assert_eq!(collection.visible, visible);
        assert_eq!(collection.selectable, selectable);
    }
}

#[test]
fn test_bone_color() {
    // Test constructors
    let color = BoneColor::new(0.5, 0.7, 0.9);
    assert_eq!(color.r, 0.5);
    assert_eq!(color.g, 0.7);
    assert_eq!(color.b, 0.9);

    // Test clamping
    let clamped = BoneColor::new(1.5, -0.5, 0.5);
    assert_eq!(clamped.r, 1.0);
    assert_eq!(clamped.g, 0.0);
    assert_eq!(clamped.b, 0.5);

    // Test standard colors
    let blue = BoneColor::left_blue();
    assert!(blue.b > blue.r);
    assert!(blue.b > blue.g);

    let red = BoneColor::right_red();
    assert!(red.r > red.g);
    assert!(red.r > red.b);

    let yellow = BoneColor::center_yellow();
    assert!(yellow.r > 0.9);
    assert!(yellow.g > 0.8);

    // Test as_array
    let arr = color.as_array();
    assert_eq!(arr, [0.5, 0.7, 0.9]);

    // Test default
    let default = BoneColor::default();
    assert_eq!(default, BoneColor::white());
}

#[test]
fn test_bone_color_serde() {
    let color = BoneColor::new(0.2, 0.4, 1.0);
    let json = serde_json::to_string(&color).unwrap();
    assert!(json.contains("0.2"));
    assert!(json.contains("0.4"));
    assert!(json.contains("1.0"));

    let parsed: BoneColor = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.r, 0.2);
    assert_eq!(parsed.g, 0.4);
    assert_eq!(parsed.b, 1.0);
}

#[test]
fn test_bone_color_scheme_standard() {
    let scheme = BoneColorScheme::Standard;

    // Left bones (suffix _l)
    let left_color = scheme.color_for_bone("arm_l");
    assert_eq!(left_color, BoneColor::left_blue());

    // Right bones (suffix _r)
    let right_color = scheme.color_for_bone("arm_r");
    assert_eq!(right_color, BoneColor::right_red());

    // Center bones (no suffix)
    let center_color = scheme.color_for_bone("spine");
    assert_eq!(center_color, BoneColor::center_yellow());

    // Test uppercase suffix
    let left_upper = scheme.color_for_bone("arm_L");
    assert_eq!(left_upper, BoneColor::left_blue());
}

#[test]
fn test_bone_color_scheme_custom() {
    let scheme = BoneColorScheme::custom(
        BoneColor::new(0.0, 1.0, 0.0), // green for left
        BoneColor::new(1.0, 0.0, 1.0), // magenta for right
        BoneColor::new(0.5, 0.5, 0.5), // gray for center
    );

    let left = scheme.color_for_bone("leg_l");
    assert_eq!(left.g, 1.0);

    let right = scheme.color_for_bone("leg_r");
    assert_eq!(right.r, 1.0);
    assert_eq!(right.b, 1.0);

    let center = scheme.color_for_bone("head");
    assert_eq!(center.r, 0.5);
}

#[test]
fn test_bone_color_scheme_per_bone() {
    let mut colors = std::collections::HashMap::new();
    colors.insert("special_bone".to_string(), BoneColor::new(1.0, 0.5, 0.0));

    let scheme = BoneColorScheme::PerBone {
        colors,
        default: BoneColor::white(),
    };

    let special = scheme.color_for_bone("special_bone");
    assert_eq!(special.r, 1.0);
    assert_eq!(special.g, 0.5);

    let other = scheme.color_for_bone("other_bone");
    assert_eq!(other, BoneColor::white());
}

#[test]
fn test_bone_color_scheme_serde() {
    // Standard scheme
    let standard = BoneColorScheme::Standard;
    let json = serde_json::to_string(&standard).unwrap();
    assert!(json.contains("standard"));

    let parsed: BoneColorScheme = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, BoneColorScheme::Standard));

    // Custom scheme
    let custom = BoneColorScheme::custom(
        BoneColor::left_blue(),
        BoneColor::right_red(),
        BoneColor::center_yellow(),
    );
    let json = serde_json::to_string(&custom).unwrap();
    assert!(json.contains("custom"));
    assert!(json.contains("left"));
    assert!(json.contains("right"));
    assert!(json.contains("center"));
}

#[test]
fn test_armature_display_serde() {
    let displays = [
        (ArmatureDisplay::Octahedral, "\"OCTAHEDRAL\""),
        (ArmatureDisplay::Stick, "\"STICK\""),
        (ArmatureDisplay::Bbone, "\"BBONE\""),
        (ArmatureDisplay::Envelope, "\"ENVELOPE\""),
        (ArmatureDisplay::Wire, "\"WIRE\""),
    ];

    for (display, expected) in displays {
        let json = serde_json::to_string(&display).unwrap();
        assert_eq!(json, expected);
        assert_eq!(display.blender_name(), expected.trim_matches('"'));
    }

    // Test default
    assert_eq!(ArmatureDisplay::default(), ArmatureDisplay::Octahedral);
}

#[test]
fn test_animator_rig_config_default() {
    let config = AnimatorRigConfig::new();

    // All features enabled by default
    assert!(config.collections);
    assert!(config.shapes);
    assert!(config.colors);
    assert_eq!(config.display, ArmatureDisplay::Octahedral);
    assert_eq!(config.widget_style, WidgetStyle::WireCircle);
    assert!(config.bone_collections.is_empty());
    assert!(matches!(config.bone_colors, BoneColorScheme::Standard));
}

#[test]
fn test_animator_rig_config_minimal() {
    let config = AnimatorRigConfig::minimal();

    // All features disabled
    assert!(!config.collections);
    assert!(!config.shapes);
    assert!(!config.colors);
}

#[test]
fn test_animator_rig_config_builder() {
    let config = AnimatorRigConfig::new()
        .with_collections(true)
        .with_shapes(true)
        .with_colors(false)
        .with_display(ArmatureDisplay::Stick)
        .with_widget_style(WidgetStyle::WireDiamond)
        .with_bone_collection(BoneCollection::new("Custom"))
        .with_bone_colors(BoneColorScheme::Standard);

    assert!(config.collections);
    assert!(config.shapes);
    assert!(!config.colors);
    assert_eq!(config.display, ArmatureDisplay::Stick);
    assert_eq!(config.widget_style, WidgetStyle::WireDiamond);
    assert_eq!(config.bone_collections.len(), 1);
}

#[test]
fn test_animator_rig_config_serde() {
    let config = AnimatorRigConfig::new()
        .with_display(ArmatureDisplay::Bbone)
        .with_widget_style(WidgetStyle::WireSphere)
        .with_bone_collection(
            BoneCollection::new("Test Collection").with_bones(["bone1", "bone2"]),
        );

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("BBONE"));
    assert!(json.contains("wire_sphere"));
    assert!(json.contains("Test Collection"));
    assert!(json.contains("bone1"));

    let parsed: AnimatorRigConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.display, ArmatureDisplay::Bbone);
    assert_eq!(parsed.widget_style, WidgetStyle::WireSphere);
    assert_eq!(parsed.bone_collections.len(), 1);
}

#[test]
fn test_animator_rig_config_validation() {
    // Valid config
    let valid = AnimatorRigConfig::new().with_bone_collection(BoneCollection::new("Valid"));
    assert!(valid.validate().is_ok());

    // Invalid - empty collection name
    let invalid = AnimatorRigConfig::new().with_bone_collection(BoneCollection::new(""));
    assert_eq!(
        invalid.validate(),
        Err(AnimatorRigError::EmptyCollectionName)
    );
}

#[test]
fn test_animator_rig_config_default_humanoid_collections() {
    let collections = AnimatorRigConfig::default_humanoid_collections();

    assert_eq!(collections.len(), 4);

    // Check IK Controls
    let ik = &collections[0];
    assert_eq!(ik.name, "IK Controls");
    assert!(ik.bones.contains(&"ik_foot_l".to_string()));
    assert!(ik.visible);

    // Check FK Controls
    let fk = &collections[1];
    assert_eq!(fk.name, "FK Controls");
    assert!(fk.bones.contains(&"root".to_string()));

    // Check Deform
    let deform = &collections[2];
    assert_eq!(deform.name, "Deform");
    assert!(!deform.visible);
    assert!(!deform.selectable);

    // Check Mechanism
    let mechanism = &collections[3];
    assert_eq!(mechanism.name, "Mechanism");
    assert!(!mechanism.visible);
}

#[test]
fn test_animator_rig_error_display() {
    assert_eq!(
        AnimatorRigError::EmptyCollectionName.to_string(),
        "Bone collection name cannot be empty"
    );
    assert_eq!(
        AnimatorRigError::DuplicateCollectionName("Test".to_string()).to_string(),
        "Duplicate bone collection name: Test"
    );
    assert_eq!(
        AnimatorRigError::InvalidWidgetStyle {
            bone: "arm".to_string(),
            style: "invalid".to_string()
        }
        .to_string(),
        "Invalid widget style 'invalid' for bone 'arm'"
    );
}
