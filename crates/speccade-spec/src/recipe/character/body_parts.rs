//! Modern body_parts array system.

use serde::{Deserialize, Serialize};

use crate::recipe::mesh::MeshPrimitive;

/// Body part definition attached to a bone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyPart {
    /// Name of the bone this part is attached to.
    pub bone: String,
    /// Mesh configuration.
    pub mesh: BodyPartMesh,
    /// Optional material index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_index: Option<u32>,
}

/// Mesh configuration for a body part.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyPartMesh {
    /// Base primitive type.
    pub primitive: MeshPrimitive,
    /// Dimensions [X, Y, Z].
    pub dimensions: [f64; 3],
    /// Number of segments (for cylinders, spheres, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<u8>,
    /// Position offset from bone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<[f64; 3]>,
    /// Rotation in euler angles [X, Y, Z] degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,
}
