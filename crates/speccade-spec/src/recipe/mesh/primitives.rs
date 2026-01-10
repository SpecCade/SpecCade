//! Primitive mesh types.

use serde::{Deserialize, Serialize};

/// Base mesh primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshPrimitive {
    /// Cube/box.
    Cube,
    /// UV sphere.
    Sphere,
    /// Cylinder.
    Cylinder,
    /// Cone.
    Cone,
    /// Torus.
    Torus,
    /// Plane.
    Plane,
    /// Ico sphere.
    IcoSphere,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // MeshPrimitive Tests - All primitive types
    // ========================================================================

    #[test]
    fn test_mesh_primitive_cube() {
        let prim = MeshPrimitive::Cube;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"cube\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Cube);
    }

    #[test]
    fn test_mesh_primitive_sphere() {
        let prim = MeshPrimitive::Sphere;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"sphere\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Sphere);
    }

    #[test]
    fn test_mesh_primitive_cylinder() {
        let prim = MeshPrimitive::Cylinder;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"cylinder\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Cylinder);
    }

    #[test]
    fn test_mesh_primitive_cone() {
        let prim = MeshPrimitive::Cone;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"cone\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Cone);
    }

    #[test]
    fn test_mesh_primitive_torus() {
        let prim = MeshPrimitive::Torus;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"torus\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Torus);
    }

    #[test]
    fn test_mesh_primitive_plane() {
        let prim = MeshPrimitive::Plane;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"plane\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::Plane);
    }

    #[test]
    fn test_mesh_primitive_icosphere() {
        let prim = MeshPrimitive::IcoSphere;
        let json = serde_json::to_string(&prim).unwrap();
        assert_eq!(json, "\"ico_sphere\"");
        let parsed: MeshPrimitive = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MeshPrimitive::IcoSphere);
    }
}
