//! Tests for the extrusion parts system.

use super::super::*;

#[test]
fn test_skeleton_bone_with_mirror() {
    let json = r#"{
        "bone": "arm_r",
        "mirror": "arm_l"
    }"#;
    let bone: SkeletonBone = serde_json::from_str(json).unwrap();
    assert_eq!(bone.bone, "arm_r");
    assert_eq!(bone.mirror, Some("arm_l".to_string()));
    assert!(bone.head.is_none());
    assert!(bone.tail.is_none());
}

#[test]
fn test_skeleton_bone_with_positions() {
    let json = r#"{
        "bone": "spine",
        "head": [0, 0, 1.0],
        "tail": [0, 0, 1.2],
        "parent": "hips"
    }"#;
    let bone: SkeletonBone = serde_json::from_str(json).unwrap();
    assert_eq!(bone.bone, "spine");
    assert_eq!(bone.head, Some([0.0, 0.0, 1.0]));
    assert_eq!(bone.tail, Some([0.0, 0.0, 1.2]));
    assert_eq!(bone.parent, Some("hips".to_string()));
}

#[test]
fn test_base_radius_uniform() {
    let json = r#"0.15"#;
    let radius: BaseRadius = serde_json::from_str(json).unwrap();
    assert_eq!(radius, BaseRadius::Uniform(0.15));
}

#[test]
fn test_base_radius_tapered() {
    let json = r#"[0.1, 0.05]"#;
    let radius: BaseRadius = serde_json::from_str(json).unwrap();
    assert_eq!(radius, BaseRadius::Tapered([0.1, 0.05]));
}

#[test]
fn test_step_shorthand() {
    let json = r#""0.1""#;
    let step: Step = serde_json::from_str(json).unwrap();
    assert!(matches!(step, Step::Shorthand(s) if s == "0.1"));
}

#[test]
fn test_step_full_definition() {
    let json = r#"{
        "extrude": 0.15,
        "scale": 0.8,
        "rotate": 15.0,
        "bulge": [1.1, 0.9],
        "tilt": [5.0, -3.0]
    }"#;
    let step: Step = serde_json::from_str(json).unwrap();
    if let Step::Full(def) = step {
        assert_eq!(def.extrude, Some(0.15));
        assert_eq!(def.scale, Some(ScaleFactor::Uniform(0.8)));
        assert_eq!(def.rotate, Some(15.0));
        assert_eq!(def.bulge, Some(BulgeFactor::Asymmetric([1.1, 0.9])));
        assert_eq!(def.tilt, Some(TiltFactor::PerAxis([5.0, -3.0])));
    } else {
        panic!("Expected Full step definition");
    }
}

#[test]
fn test_step_with_translate() {
    let json = r#"{
        "extrude": 0.1,
        "translate": [0.05, 0.0, 0.0]
    }"#;
    let step: Step = serde_json::from_str(json).unwrap();
    if let Step::Full(def) = step {
        assert_eq!(def.translate, Some([0.05, 0.0, 0.0]));
    } else {
        panic!("Expected Full step definition");
    }
}

#[test]
fn test_scale_factor_uniform() {
    let json = r#"0.9"#;
    let scale: ScaleFactor = serde_json::from_str(json).unwrap();
    assert_eq!(scale, ScaleFactor::Uniform(0.9));
}

#[test]
fn test_scale_factor_per_axis() {
    let json = r#"[0.8, 1.2]"#;
    let scale: ScaleFactor = serde_json::from_str(json).unwrap();
    assert_eq!(scale, ScaleFactor::PerAxis([0.8, 1.2]));
}

#[test]
fn test_extrusion_part_basic() {
    let json = r#"{
        "bone": "chest",
        "base": "hexagon(6)",
        "base_radius": 0.2,
        "cap_start": true,
        "cap_end": false
    }"#;
    let part: ExtrusionPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.bone, Some("chest".to_string()));
    assert_eq!(part.base, Some("hexagon(6)".to_string()));
    assert_eq!(part.base_radius, Some(BaseRadius::Uniform(0.2)));
    assert_eq!(part.cap_start, Some(true));
    assert_eq!(part.cap_end, Some(false));
}

#[test]
fn test_extrusion_part_with_steps() {
    let json = r#"{
        "bone": "arm_upper_l",
        "base": "circle(8)",
        "base_radius": [0.08, 0.06],
        "steps": [
            "0.1",
            {"extrude": 0.15, "scale": 0.9},
            {"extrude": 0.1, "scale": 0.8, "rotate": 10}
        ]
    }"#;
    let part: ExtrusionPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.steps.len(), 3);
    assert!(matches!(&part.steps[0], Step::Shorthand(s) if s == "0.1"));
    assert!(matches!(&part.steps[1], Step::Full(_)));
    assert!(matches!(&part.steps[2], Step::Full(_)));
}

#[test]
fn test_extrusion_part_with_mirror() {
    let json = r#"{
        "bone": "arm_upper_r",
        "mirror": "arm_upper_l"
    }"#;
    let part: ExtrusionPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.mirror, Some("arm_upper_l".to_string()));
}

#[test]
fn test_instance() {
    let json = r#"{
        "position": [0.1, 0.0, 0.5],
        "rotation": [0, 45, 0]
    }"#;
    let instance: Instance = serde_json::from_str(json).unwrap();
    assert_eq!(instance.position, Some([0.1, 0.0, 0.5]));
    assert_eq!(instance.rotation, Some([0.0, 45.0, 0.0]));
}

#[test]
fn test_extrusion_part_with_instances() {
    let json = r#"{
        "bone": "spike",
        "base": "circle(6)",
        "base_radius": 0.05,
        "instances": [
            {"position": [0.0, 0.0, 0.1]},
            {"position": [0.1, 0.0, 0.1], "rotation": [0, 0, 30]},
            {"position": [-0.1, 0.0, 0.1], "rotation": [0, 0, -30]}
        ]
    }"#;
    let part: ExtrusionPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.instances.len(), 3);
    assert_eq!(part.instances[0].position, Some([0.0, 0.0, 0.1]));
    assert_eq!(part.instances[1].rotation, Some([0.0, 0.0, 30.0]));
}

#[test]
fn test_sub_part() {
    let json = r#"{
        "bone": "finger_index_1",
        "base": "circle(4)",
        "base_radius": 0.02,
        "steps": [{"extrude": 0.03, "scale": 0.9}],
        "cap_start": false,
        "cap_end": true
    }"#;
    let sub: SubPart = serde_json::from_str(json).unwrap();
    assert_eq!(sub.bone, Some("finger_index_1".to_string()));
    assert_eq!(sub.cap_end, Some(true));
}

#[test]
fn test_sub_part_or_list_single() {
    let json = r#"{"bone": "thumb_1", "base_radius": 0.015}"#;
    let sub: SubPartOrList = serde_json::from_str(json).unwrap();
    assert!(matches!(sub, SubPartOrList::Single(_)));
}

#[test]
fn test_sub_part_or_list_multiple() {
    let json = r#"[
        {"bone": "thumb_1", "base_radius": 0.015},
        {"bone": "thumb_2", "base_radius": 0.012}
    ]"#;
    let sub: SubPartOrList = serde_json::from_str(json).unwrap();
    if let SubPartOrList::List(list) = sub {
        assert_eq!(list.len(), 2);
    } else {
        panic!("Expected List variant");
    }
}

#[test]
fn test_extrusion_part_with_fingers() {
    let json = r#"{
        "bone": "hand_l",
        "base": "circle(8)",
        "base_radius": 0.04,
        "thumb": {"bone": "thumb_l", "base_radius": 0.015},
        "fingers": [
            {"bone": "finger_index_l", "base_radius": 0.012},
            {"bone": "finger_middle_l", "base_radius": 0.012},
            {"bone": "finger_ring_l", "base_radius": 0.011},
            {"bone": "finger_pinky_l", "base_radius": 0.01}
        ]
    }"#;
    let part: ExtrusionPart = serde_json::from_str(json).unwrap();
    assert!(part.thumb.is_some());
    assert_eq!(part.fingers.len(), 4);
}

#[test]
fn test_texturing_basic() {
    let json = r#"{
        "uv_mode": "smart_project"
    }"#;
    let tex: Texturing = serde_json::from_str(json).unwrap();
    assert_eq!(tex.uv_mode, Some(UvMode::SmartProject));
}

#[test]
fn test_texturing_with_regions() {
    let json = r##"{
        "uv_mode": "region_based",
        "regions": {
            "body": {
                "parts": ["torso", "hips"],
                "material_index": 0,
                "color": "#FF5500"
            },
            "limbs": {
                "parts": ["arm_l", "arm_r", "leg_l", "leg_r"],
                "material_index": 1,
                "color": [0.8, 0.6, 0.4]
            }
        }
    }"##;
    let tex: Texturing = serde_json::from_str(json).unwrap();
    assert_eq!(tex.uv_mode, Some(UvMode::RegionBased));
    assert_eq!(tex.regions.len(), 2);
    assert!(tex.regions.contains_key("body"));
    assert!(tex.regions.contains_key("limbs"));

    let body = tex.regions.get("body").unwrap();
    assert_eq!(body.parts, vec!["torso", "hips"]);
    assert!(matches!(&body.color, Some(RegionColor::Hex(s)) if s == "#FF5500"));
}

#[test]
fn test_uv_modes() {
    let modes = vec![
        (r#""smart_project""#, UvMode::SmartProject),
        (r#""region_based""#, UvMode::RegionBased),
        (r#""lightmap_pack""#, UvMode::LightmapPack),
        (r#""cube_project""#, UvMode::CubeProject),
        (r#""cylinder_project""#, UvMode::CylinderProject),
        (r#""sphere_project""#, UvMode::SphereProject),
    ];
    for (json, expected) in modes {
        let mode: UvMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, expected);
    }
}

#[test]
fn test_region_color_variants() {
    // Hex
    let hex: RegionColor = serde_json::from_str(r##""#AABBCC""##).unwrap();
    assert!(matches!(hex, RegionColor::Hex(s) if s == "#AABBCC"));

    // RGB
    let rgb: RegionColor = serde_json::from_str(r#"[1.0, 0.5, 0.25]"#).unwrap();
    assert!(matches!(rgb, RegionColor::Rgb([1.0, 0.5, 0.25])));

    // RGBA
    let rgba: RegionColor = serde_json::from_str(r#"[1.0, 0.5, 0.25, 0.8]"#).unwrap();
    assert!(matches!(rgba, RegionColor::Rgba([1.0, 0.5, 0.25, 0.8])));
}

#[test]
fn test_skinning_type() {
    let soft: SkinningType = serde_json::from_str(r#""soft""#).unwrap();
    assert_eq!(soft, SkinningType::Soft);

    let rigid: SkinningType = serde_json::from_str(r#""rigid""#).unwrap();
    assert_eq!(rigid, SkinningType::Rigid);
}

#[test]
fn test_bulge_factor_variants() {
    let uniform: BulgeFactor = serde_json::from_str("1.2").unwrap();
    assert_eq!(uniform, BulgeFactor::Uniform(1.2));

    let asymmetric: BulgeFactor = serde_json::from_str("[1.1, 0.9]").unwrap();
    assert_eq!(asymmetric, BulgeFactor::Asymmetric([1.1, 0.9]));
}

#[test]
fn test_tilt_factor_variants() {
    let uniform: TiltFactor = serde_json::from_str("5.0").unwrap();
    assert_eq!(uniform, TiltFactor::Uniform(5.0));

    let per_axis: TiltFactor = serde_json::from_str("[10.0, -5.0]").unwrap();
    assert_eq!(per_axis, TiltFactor::PerAxis([10.0, -5.0]));
}
