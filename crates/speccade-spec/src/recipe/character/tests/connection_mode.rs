//! Tests for ConnectionMode enum.

use super::super::*;

#[test]
fn test_connection_mode_parses_bridge() {
    let json = r#""bridge""#;
    let mode: ConnectionMode = serde_json::from_str(json).unwrap();
    assert_eq!(mode, ConnectionMode::Bridge);
}

#[test]
fn test_connection_mode_parses_segmented() {
    let json = r#""segmented""#;
    let mode: ConnectionMode = serde_json::from_str(json).unwrap();
    assert_eq!(mode, ConnectionMode::Segmented);
}

#[test]
fn test_connection_mode_default_is_none() {
    let json = r#"{"profile": "circle(8)"}"#;
    let mesh: ArmatureDrivenBoneMesh = serde_json::from_str(json).unwrap();
    assert!(mesh.connect_start.is_none());
    assert!(mesh.connect_end.is_none());
}

#[test]
fn test_bone_mesh_with_bridge_connections() {
    let json = r#"{
        "profile": "circle(8)",
        "connect_start": "bridge",
        "connect_end": "segmented"
    }"#;
    let mesh: ArmatureDrivenBoneMesh = serde_json::from_str(json).unwrap();
    assert_eq!(mesh.connect_start, Some(ConnectionMode::Bridge));
    assert_eq!(mesh.connect_end, Some(ConnectionMode::Segmented));
}

#[test]
fn test_full_bone_meshes_with_bridging() {
    let json = r#"{
        "skeleton_preset": "humanoid_basic_v1",
        "bone_meshes": {
            "spine": {
                "profile": "circle(8)",
                "connect_end": "bridge"
            },
            "chest": {
                "profile": "circle(8)",
                "connect_start": "bridge",
                "connect_end": "bridge"
            },
            "neck": {
                "profile": "circle(8)",
                "connect_start": "bridge"
            }
        }
    }"#;
    let params: SkeletalMeshArmatureDrivenV1Params = serde_json::from_str(json).unwrap();

    let spine = params.bone_meshes.get("spine").unwrap();
    if let ArmatureDrivenBoneMeshDef::Mesh(m) = spine {
        assert_eq!(m.connect_end, Some(ConnectionMode::Bridge));
        assert!(m.connect_start.is_none());
    } else {
        panic!("Expected Mesh, got Mirror");
    }
}
