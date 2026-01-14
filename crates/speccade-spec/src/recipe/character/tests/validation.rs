//! Validation and round-trip tests.

use std::collections::HashMap;

use super::super::*;

/// Validation: Skeleton bone with neither head/tail nor mirror should be invalid in practice
#[test]
fn test_validation_skeleton_bone_incomplete() {
    // This tests that we can deserialize it, but validation would catch it
    let json = r#"{"bone": "incomplete"}"#;
    let bone: SkeletonBone = serde_json::from_str(json).unwrap();
    assert!(bone.head.is_none());
    assert!(bone.tail.is_none());
    assert!(bone.mirror.is_none());
}

/// Validation: Empty skeleton and empty parts
#[test]
fn test_validation_empty_character() {
    let json = r#"{"skeleton": [], "parts": {}}"#;
    let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
    assert!(params.skeleton.is_empty());
    assert!(params.parts.is_empty());
}

/// Validation: All UV modes
#[test]
fn test_validation_all_uv_modes() {
    let modes = [
        "smart_project",
        "region_based",
        "lightmap_pack",
        "cube_project",
        "cylinder_project",
        "sphere_project",
    ];
    for mode in &modes {
        let json = format!(r#"{{"uv_mode": "{}"}}"#, mode);
        let tex: Texturing = serde_json::from_str(&json).unwrap();
        assert!(tex.uv_mode.is_some());
    }
}

/// Validation: Skinning type values
#[test]
fn test_validation_skinning_types() {
    let soft: SkinningType = serde_json::from_str(r#""soft""#).unwrap();
    assert_eq!(soft, SkinningType::Soft);

    let rigid: SkinningType = serde_json::from_str(r#""rigid""#).unwrap();
    assert_eq!(rigid, SkinningType::Rigid);
}

/// Validation: Max bone influences bounds
#[test]
fn test_validation_max_bone_influences_bounds() {
    // Test valid values
    let json = r#"{"max_bone_influences": 1}"#;
    let skinning: SkinningSettings = serde_json::from_str(json).unwrap();
    assert_eq!(skinning.max_bone_influences, 1);

    let json = r#"{"max_bone_influences": 8}"#;
    let skinning: SkinningSettings = serde_json::from_str(json).unwrap();
    assert_eq!(skinning.max_bone_influences, 8);
}

/// Round-trip serialization test
#[test]
fn test_roundtrip_full_character() {
    let params = SkeletalMeshBlenderRiggedMeshV1Params {
        skeleton_preset: None,
        skeleton: vec![SkeletonBone {
            bone: "root".to_string(),
            head: Some([0.0, 0.0, 0.0]),
            tail: Some([0.0, 0.0, 1.0]),
            parent: None,
            mirror: None,
        }],
        body_parts: vec![],
        parts: {
            let mut map = HashMap::new();
            map.insert(
                "torso".to_string(),
                LegacyPart {
                    bone: "root".to_string(),
                    base: Some("hexagon(6)".to_string()),
                    base_radius: Some(BaseRadius::Uniform(0.2)),
                    steps: vec![Step::Full(StepDefinition {
                        extrude: Some(0.1),
                        scale: Some(ScaleFactor::Uniform(0.9)),
                        translate: None,
                        rotate: None,
                        bulge: None,
                        tilt: None,
                    })],
                    mirror: None,
                    offset: None,
                    rotation: None,
                    cap_start: Some(true),
                    cap_end: Some(true),
                    skinning_type: Some(SkinningType::Soft),
                    thumb: None,
                    fingers: vec![],
                    instances: vec![],
                },
            );
            map
        },
        material_slots: vec![],
        skinning: Some(SkinningSettings {
            max_bone_influences: 4,
            auto_weights: true,
        }),
        export: Some(SkeletalMeshExportSettings {
            include_armature: true,
            include_normals: true,
            include_uvs: true,
            triangulate: true,
            include_skin_weights: true,
            save_blend: false,
        }),
        constraints: None,
        tri_budget: Some(500),
        texturing: Some(Texturing {
            uv_mode: Some(UvMode::SmartProject),
            regions: HashMap::new(),
        }),
    };

    let json = serde_json::to_string(&params).unwrap();
    let parsed: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.skeleton.len(), 1);
    assert_eq!(parsed.parts.len(), 1);
    assert_eq!(parsed.tri_budget, Some(500));
}
