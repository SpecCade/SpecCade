//! Comprehensive parity matrix coverage tests.
//!
//! These tests ensure every key/parameter from the spec is properly handled.

use super::super::*;

/// Test: Top-level key 'name'
#[test]
fn test_parity_name() {
    let json = r#"{
        "skeleton": [],
        "parts": {},
        "tri_budget": 100
    }"#;
    let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
    // Name is not directly in params, it's in the asset wrapper
    assert!(params.skeleton.is_empty());
}

/// Test: Top-level key 'tri_budget'
#[test]
fn test_parity_tri_budget() {
    let json = r#"{"skeleton": [], "parts": {}, "tri_budget": 500}"#;
    let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
    assert_eq!(params.tri_budget, Some(500));
}

/// Test: Top-level key 'skeleton' (array)
#[test]
fn test_parity_skeleton_array() {
    let json = r#"{
        "skeleton": [
            {"bone": "root", "head": [0, 0, 0], "tail": [0, 0, 1]}
        ],
        "parts": {}
    }"#;
    let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
    assert_eq!(params.skeleton.len(), 1);
    assert_eq!(params.skeleton[0].bone, "root");
}

/// Test: Top-level key 'skeleton_preset'
#[test]
fn test_parity_skeleton_preset() {
    let json = r#"{
        "skeleton_preset": "humanoid_basic_v1",
        "parts": {}
    }"#;
    let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
    assert_eq!(
        params.skeleton_preset,
        Some(SkeletonPreset::HumanoidBasicV1)
    );
    assert!(params.skeleton.is_empty());
}

/// Test: Top-level key 'parts' (dict)
#[test]
fn test_parity_parts_dict() {
    let json = r#"{
        "skeleton": [],
        "parts": {
            "head": {
                "bone": "head_bone",
                "base": "circle(8)",
                "base_radius": 0.1
            }
        }
    }"#;
    let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
    assert_eq!(params.parts.len(), 1);
    assert!(params.parts.contains_key("head"));
}

/// Test: Top-level key 'body_parts' (array, modern style)
#[test]
fn test_parity_body_parts_array() {
    let json = r#"{
        "skeleton": [],
        "body_parts": [
            {
                "bone": "head",
                "mesh": {
                    "primitive": "sphere",
                    "dimensions": [0.2, 0.2, 0.2]
                }
            }
        ],
        "parts": {}
    }"#;
    let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
    assert_eq!(params.body_parts.len(), 1);
    assert_eq!(params.body_parts[0].bone, "head");
}

/// Test: Top-level key 'texturing'
#[test]
fn test_parity_texturing() {
    let json = r#"{
        "skeleton": [],
        "parts": {},
        "texturing": {"uv_mode": "smart_project"}
    }"#;
    let params: SkeletalMeshBlenderRiggedMeshV1Params = serde_json::from_str(json).unwrap();
    assert!(params.texturing.is_some());
}

/// Test: Skeleton bone key 'bone' (required)
#[test]
fn test_parity_skeleton_bone_name() {
    let bone = SkeletonBone {
        bone: "test_bone".to_string(),
        head: None,
        tail: None,
        parent: None,
        mirror: None,
    };
    let json = serde_json::to_string(&bone).unwrap();
    assert!(json.contains("test_bone"));
}

/// Test: Skeleton bone key 'head'
#[test]
fn test_parity_skeleton_bone_head() {
    let json = r#"{"bone": "test", "head": [1.0, 2.0, 3.0]}"#;
    let bone: SkeletonBone = serde_json::from_str(json).unwrap();
    assert_eq!(bone.head, Some([1.0, 2.0, 3.0]));
}

/// Test: Skeleton bone key 'tail'
#[test]
fn test_parity_skeleton_bone_tail() {
    let json = r#"{"bone": "test", "tail": [4.0, 5.0, 6.0]}"#;
    let bone: SkeletonBone = serde_json::from_str(json).unwrap();
    assert_eq!(bone.tail, Some([4.0, 5.0, 6.0]));
}

/// Test: Skeleton bone key 'parent'
#[test]
fn test_parity_skeleton_bone_parent() {
    let json = r#"{"bone": "child", "parent": "parent_bone"}"#;
    let bone: SkeletonBone = serde_json::from_str(json).unwrap();
    assert_eq!(bone.parent, Some("parent_bone".to_string()));
}

/// Test: Skeleton bone key 'mirror'
#[test]
fn test_parity_skeleton_bone_mirror() {
    let json = r#"{"bone": "arm_r", "mirror": "arm_l"}"#;
    let bone: SkeletonBone = serde_json::from_str(json).unwrap();
    assert_eq!(bone.mirror, Some("arm_l".to_string()));
}

/// Test: Part key 'bone' (required)
#[test]
fn test_parity_part_bone() {
    let json = r#"{"bone": "torso", "base": "hexagon(6)", "base_radius": 0.1}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.bone, "torso");
}

/// Test: Part key 'base'
#[test]
fn test_parity_part_base() {
    let json = r#"{"bone": "test", "base": "hexagon(6)"}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.base, Some("hexagon(6)".to_string()));
}

/// Test: Part key 'base_radius' (uniform)
#[test]
fn test_parity_part_base_radius_uniform() {
    let json = r#"{"bone": "test", "base_radius": 0.15}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.base_radius, Some(BaseRadius::Uniform(0.15)));
}

/// Test: Part key 'base_radius' (tapered array)
#[test]
fn test_parity_part_base_radius_tapered() {
    let json = r#"{"bone": "test", "base_radius": [0.2, 0.1]}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.base_radius, Some(BaseRadius::Tapered([0.2, 0.1])));
}

/// Test: Part key 'steps'
#[test]
fn test_parity_part_steps() {
    let json = r#"{"bone": "test", "steps": ["0.1", {"extrude": 0.2}]}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.steps.len(), 2);
}

/// Test: Part key 'mirror'
#[test]
fn test_parity_part_mirror() {
    let json = r#"{"bone": "arm_r", "mirror": "arm_l"}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.mirror, Some("arm_l".to_string()));
}

/// Test: Part key 'offset'
#[test]
fn test_parity_part_offset() {
    let json = r#"{"bone": "test", "offset": [0.5, 0.0, 0.1]}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.offset, Some([0.5, 0.0, 0.1]));
}

/// Test: Part key 'rotation'
#[test]
fn test_parity_part_rotation() {
    let json = r#"{"bone": "test", "rotation": [45.0, 0.0, -30.0]}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.rotation, Some([45.0, 0.0, -30.0]));
}

/// Test: Part key 'cap_start'
#[test]
fn test_parity_part_cap_start() {
    let json = r#"{"bone": "test", "cap_start": false}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.cap_start, Some(false));
}

/// Test: Part key 'cap_end'
#[test]
fn test_parity_part_cap_end() {
    let json = r#"{"bone": "test", "cap_end": true}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.cap_end, Some(true));
}

/// Test: Part key 'skinning_type'
#[test]
fn test_parity_part_skinning_type() {
    let json = r#"{"bone": "test", "skinning_type": "rigid"}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.skinning_type, Some(SkinningType::Rigid));
}

/// Test: Part key 'thumb'
#[test]
fn test_parity_part_thumb() {
    let json = r#"{"bone": "hand", "thumb": {"bone": "thumb_1", "base_radius": 0.015}}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert!(part.thumb.is_some());
}

/// Test: Part key 'fingers'
#[test]
fn test_parity_part_fingers() {
    let json = r#"{"bone": "hand", "fingers": [{"bone": "index_1"}, {"bone": "middle_1"}]}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.fingers.len(), 2);
}

/// Test: Part key 'instances'
#[test]
fn test_parity_part_instances() {
    let json = r#"{"bone": "spike", "instances": [{"position": [0, 0, 1]}]}"#;
    let part: LegacyPart = serde_json::from_str(json).unwrap();
    assert_eq!(part.instances.len(), 1);
}

/// Test: Step key 'extrude'
#[test]
fn test_parity_step_extrude() {
    let json = r#"{"extrude": 0.25}"#;
    let step: StepDefinition = serde_json::from_str(json).unwrap();
    assert_eq!(step.extrude, Some(0.25));
}

/// Test: Step key 'scale' (uniform)
#[test]
fn test_parity_step_scale_uniform() {
    let json = r#"{"scale": 0.8}"#;
    let step: StepDefinition = serde_json::from_str(json).unwrap();
    assert_eq!(step.scale, Some(ScaleFactor::Uniform(0.8)));
}

/// Test: Step key 'scale' (per-axis)
#[test]
fn test_parity_step_scale_per_axis() {
    let json = r#"{"scale": [1.2, 0.9]}"#;
    let step: StepDefinition = serde_json::from_str(json).unwrap();
    assert_eq!(step.scale, Some(ScaleFactor::PerAxis([1.2, 0.9])));
}

/// Test: Step key 'translate'
#[test]
fn test_parity_step_translate() {
    let json = r#"{"translate": [0.1, 0.2, 0.3]}"#;
    let step: StepDefinition = serde_json::from_str(json).unwrap();
    assert_eq!(step.translate, Some([0.1, 0.2, 0.3]));
}

/// Test: Step key 'rotate'
#[test]
fn test_parity_step_rotate() {
    let json = r#"{"rotate": 45.0}"#;
    let step: StepDefinition = serde_json::from_str(json).unwrap();
    assert_eq!(step.rotate, Some(45.0));
}

/// Test: Step key 'bulge' (uniform)
#[test]
fn test_parity_step_bulge_uniform() {
    let json = r#"{"bulge": 1.3}"#;
    let step: StepDefinition = serde_json::from_str(json).unwrap();
    assert_eq!(step.bulge, Some(BulgeFactor::Uniform(1.3)));
}

/// Test: Step key 'bulge' (asymmetric)
#[test]
fn test_parity_step_bulge_asymmetric() {
    let json = r#"{"bulge": [1.1, 0.9]}"#;
    let step: StepDefinition = serde_json::from_str(json).unwrap();
    assert_eq!(step.bulge, Some(BulgeFactor::Asymmetric([1.1, 0.9])));
}

/// Test: Step key 'tilt' (uniform)
#[test]
fn test_parity_step_tilt_uniform() {
    let json = r#"{"tilt": 10.0}"#;
    let step: StepDefinition = serde_json::from_str(json).unwrap();
    assert_eq!(step.tilt, Some(TiltFactor::Uniform(10.0)));
}

/// Test: Step key 'tilt' (per-axis)
#[test]
fn test_parity_step_tilt_per_axis() {
    let json = r#"{"tilt": [15.0, -5.0]}"#;
    let step: StepDefinition = serde_json::from_str(json).unwrap();
    assert_eq!(step.tilt, Some(TiltFactor::PerAxis([15.0, -5.0])));
}

/// Test: Instance key 'position'
#[test]
fn test_parity_instance_position() {
    let json = r#"{"position": [1.0, 2.0, 3.0]}"#;
    let instance: Instance = serde_json::from_str(json).unwrap();
    assert_eq!(instance.position, Some([1.0, 2.0, 3.0]));
}

/// Test: Instance key 'rotation'
#[test]
fn test_parity_instance_rotation() {
    let json = r#"{"rotation": [90.0, 0.0, 45.0]}"#;
    let instance: Instance = serde_json::from_str(json).unwrap();
    assert_eq!(instance.rotation, Some([90.0, 0.0, 45.0]));
}

/// Test: Texturing key 'uv_mode'
#[test]
fn test_parity_texturing_uv_mode() {
    let json = r#"{"uv_mode": "region_based"}"#;
    let tex: Texturing = serde_json::from_str(json).unwrap();
    assert_eq!(tex.uv_mode, Some(UvMode::RegionBased));
}

/// Test: Texturing key 'regions'
#[test]
fn test_parity_texturing_regions() {
    let json = r#"{
        "regions": {
            "head": {"parts": ["head_part"]},
            "body": {"parts": ["torso", "hips"]}
        }
    }"#;
    let tex: Texturing = serde_json::from_str(json).unwrap();
    assert_eq!(tex.regions.len(), 2);
    assert!(tex.regions.contains_key("head"));
    assert!(tex.regions.contains_key("body"));
}

/// Test: Skinning settings key 'max_bone_influences'
#[test]
fn test_parity_skinning_max_bone_influences() {
    let json = r#"{"max_bone_influences": 8}"#;
    let skinning: SkinningSettings = serde_json::from_str(json).unwrap();
    assert_eq!(skinning.max_bone_influences, 8);
}

/// Test: Export settings key 'save_blend'
#[test]
fn test_parity_export_save_blend() {
    let json = r#"{"save_blend": true}"#;
    let export: SkeletalMeshExportSettings = serde_json::from_str(json).unwrap();
    assert!(export.save_blend);
}

/// Test: Export settings key 'include_armature'
#[test]
fn test_parity_export_include_armature() {
    let json = r#"{"include_armature": false}"#;
    let export: SkeletalMeshExportSettings = serde_json::from_str(json).unwrap();
    assert!(!export.include_armature);
}

/// Test: Body part mesh key 'primitive'
#[test]
fn test_parity_body_part_primitive() {
    let json = r#"{
        "primitive": "cylinder",
        "dimensions": [0.1, 0.1, 0.5]
    }"#;
    let mesh: BodyPartMesh = serde_json::from_str(json).unwrap();
    assert_eq!(mesh.primitive, crate::recipe::mesh::MeshPrimitive::Cylinder);
}

/// Test: Body part mesh key 'dimensions'
#[test]
fn test_parity_body_part_dimensions() {
    let json = r#"{
        "primitive": "cube",
        "dimensions": [0.3, 0.4, 0.5]
    }"#;
    let mesh: BodyPartMesh = serde_json::from_str(json).unwrap();
    assert_eq!(mesh.dimensions, [0.3, 0.4, 0.5]);
}

/// Test: Body part mesh key 'segments'
#[test]
fn test_parity_body_part_segments() {
    let json = r#"{
        "primitive": "sphere",
        "dimensions": [0.2, 0.2, 0.2],
        "segments": 16
    }"#;
    let mesh: BodyPartMesh = serde_json::from_str(json).unwrap();
    assert_eq!(mesh.segments, Some(16));
}

/// Test: Body part mesh key 'offset'
#[test]
fn test_parity_body_part_offset() {
    let json = r#"{
        "primitive": "cube",
        "dimensions": [1.0, 1.0, 1.0],
        "offset": [0.5, 0.0, 0.1]
    }"#;
    let mesh: BodyPartMesh = serde_json::from_str(json).unwrap();
    assert_eq!(mesh.offset, Some([0.5, 0.0, 0.1]));
}

/// Test: Body part mesh key 'rotation'
#[test]
fn test_parity_body_part_rotation() {
    let json = r#"{
        "primitive": "cube",
        "dimensions": [1.0, 1.0, 1.0],
        "rotation": [90.0, 0.0, 0.0]
    }"#;
    let mesh: BodyPartMesh = serde_json::from_str(json).unwrap();
    assert_eq!(mesh.rotation, Some([90.0, 0.0, 0.0]));
}
