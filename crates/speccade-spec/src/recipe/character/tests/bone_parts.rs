//! Tests for modular bone part parsing types.

use super::super::*;

#[test]
fn test_bone_mesh_part_parses_primitive_base() {
    let json = r#"{
        "part": {
            "base": {
                "primitive": "cylinder",
                "dimensions": [0.08, 0.08, 1.0]
            },
            "scale": {
                "axes": ["z"],
                "amount_from_z": {"z": 1.0}
            }
        }
    }"#;

    let mesh: ArmatureDrivenBoneMesh = serde_json::from_str(json).unwrap();
    assert!(mesh.part.is_some());
    assert!(mesh.extrusion_steps.is_empty());

    let part = mesh.part.unwrap();
    assert!(matches!(part.base, BonePartShape::Primitive(_)));
    assert!(part.operations.is_empty());
}

#[test]
fn test_bone_mesh_part_parses_boolean_operations_with_asset_shapes() {
    let json = r#"{
        "part": {
            "base": {
                "asset": "./parts/chest.glb",
                "scale": 1.0
            },
            "operations": [
                {
                    "op": "difference",
                    "target": {
                        "asset_ref": "helmet_mesh_01",
                        "offset": [0.0, 0.0, 0.2],
                        "rotation": [0.0, 90.0, 0.0],
                        "scale": 0.8
                    }
                }
            ]
        }
    }"#;

    let mesh: ArmatureDrivenBoneMesh = serde_json::from_str(json).unwrap();
    let part = mesh.part.expect("part should parse");
    assert_eq!(part.operations.len(), 1);
    assert!(matches!(part.base, BonePartShape::Asset(_)));
    assert!(matches!(
        part.operations[0].target,
        BonePartShape::AssetRef(_)
    ));
    assert_eq!(part.operations[0].op, BonePartOpType::Difference);
}

#[test]
fn test_bone_part_primitive_rejects_unknown_fields() {
    let json = r#"{
        "part": {
            "base": {
                "primitive": "cube",
                "dimensions": [0.1, 0.1, 0.1],
                "unexpected": 1
            }
        }
    }"#;

    let err = serde_json::from_str::<ArmatureDrivenBoneMesh>(json).unwrap_err();
    assert!(
        err.to_string().contains("BonePartShape"),
        "expected BonePartShape parse rejection, got: {err}"
    );
}
