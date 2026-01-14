//! Basic tests for character recipe types.

use crate::recipe::mesh::MeshPrimitive;

use super::super::*;

#[test]
fn test_skeleton_preset_bones() {
    let preset = SkeletonPreset::HumanoidBasicV1;
    let bones = preset.bone_names();
    assert!(bones.contains(&"root"));
    assert!(bones.contains(&"head"));
    assert!(bones.contains(&"hand_l"));
    assert!(bones.contains(&"foot_r"));
}

#[test]
fn test_body_part_serde() {
    let part = BodyPart {
        bone: "head".to_string(),
        mesh: BodyPartMesh {
            primitive: MeshPrimitive::Cube,
            dimensions: [0.25, 0.3, 0.25],
            segments: None,
            offset: None,
            rotation: None,
        },
        material_index: Some(0),
    };

    let json = serde_json::to_string(&part).unwrap();
    assert!(json.contains("head"));
    assert!(json.contains("cube"));

    let parsed: BodyPart = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.bone, "head");
}

#[test]
fn test_skinning_settings_defaults() {
    let settings = SkinningSettings::default();
    assert_eq!(settings.max_bone_influences, 4);
    assert!(settings.auto_weights);
}
