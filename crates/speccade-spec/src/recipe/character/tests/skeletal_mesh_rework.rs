//! Parsing tests for skeletal mesh rework recipe params.

use super::super::*;

#[test]
fn test_armature_driven_params_parses_yaml_style_examples() {
    let json = r#"{
        "skeleton_preset": "humanoid_basic_v1",
        "bone_meshes": {
            "spine": {
                "profile": "hexagon(8)",
                "profile_radius": 0.15,
                "taper": 0.9,
                "translate": [0, 0, 0],
                "rotate": [0, 0, 0],
                "bulge": [
                    {"at": 0.0, "scale": 0.8},
                    {"at": 0.5, "scale": 1.2},
                    {"at": 1.0, "scale": 0.9}
                ],
                "twist": 0,
                "cap_start": true,
                "cap_end": false,
                "modifiers": [
                    {"bevel": {"width": 0.02, "segments": 2}},
                    {"subdivide": {"cuts": 1}},
                    {"bool": {"operation": "subtract", "target": "eye_socket_L"}}
                ],
                "attachments": [
                    {
                        "primitive": "sphere",
                        "dimensions": [0.08, 0.06, 0.08],
                        "offset": [0.15, 0.05, 0.3],
                        "rotation": [0, 0, 15],
                        "material_index": 1
                    },
                    {
                        "extrude": {
                            "profile": "hexagon(4)",
                            "start": [0, 0.1, 0.5],
                            "end": [0, 0.2, 0.6],
                            "profile_radius": {"absolute": 0.05},
                            "taper": 0.3
                        }
                    },
                    {
                        "asset": "props/shoulder_armor.glb",
                        "offset": [0.18, 0, 0.4],
                        "rotation": [0, 0, 0],
                        "scale": 1.0
                    }
                ]
            },
            "arm_upper_R": {"mirror": "arm_upper_L"}
        },
        "bool_shapes": {
            "eye_socket_L": {
                "primitive": "sphere",
                "dimensions": [0.06, 0.08, 0.06],
                "position": [0.05, 0.15, 0.6]
            },
            "eye_socket_R": {"mirror": "eye_socket_L"}
        }
    }"#;

    let params: SkeletalMeshArmatureDrivenV1Params = serde_json::from_str(json).unwrap();
    assert_eq!(
        params.skeleton_preset,
        Some(SkeletonPreset::HumanoidBasicV1)
    );
    assert!(params.skeleton.is_empty());
    assert!(params.bone_meshes.contains_key("spine"));
    assert!(params.bool_shapes.contains_key("eye_socket_L"));
}

#[test]
fn test_skinned_mesh_params_parses_minimal_and_binding_options() {
    let json = r#"{
        "mesh_file": "path/to/mesh.glb",
        "skeleton_preset": "humanoid_basic_v1",
        "binding": {
            "mode": "auto_weights",
            "vertex_group_map": {"Arm.L": "arm_upper_L"},
            "max_bone_influences": 4
        }
    }"#;

    let params: SkeletalMeshSkinnedMeshV1Params = serde_json::from_str(json).unwrap();
    assert_eq!(params.mesh_file.as_deref(), Some("path/to/mesh.glb"));
    assert!(matches!(
        params.binding.mode,
        SkinnedMeshBindingMode::AutoWeights
    ));
    assert_eq!(
        params
            .binding
            .vertex_group_map
            .get("Arm.L")
            .map(|s| s.as_str()),
        Some("arm_upper_L")
    );
}
