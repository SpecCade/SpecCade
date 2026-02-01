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
