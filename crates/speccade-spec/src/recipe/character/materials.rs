//! Material slots and skinning settings.

use serde::{Deserialize, Serialize};

use crate::recipe::mesh::{MeshConstraints, MeshExportSettings};

/// Skinning and weight painting settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkinningSettings {
    /// Maximum bone influences per vertex (1-8).
    #[serde(default = "default_max_bone_influences")]
    pub max_bone_influences: u8,
    /// Use automatic weight painting.
    #[serde(default = "default_true")]
    pub auto_weights: bool,
}

fn default_max_bone_influences() -> u8 {
    4
}

fn default_true() -> bool {
    true
}

impl Default for SkinningSettings {
    fn default() -> Self {
        Self {
            max_bone_influences: 4,
            auto_weights: true,
        }
    }
}

/// Export settings for skeletal meshes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletalMeshExportSettings {
    /// Include armature in export.
    #[serde(default = "default_true")]
    pub include_armature: bool,
    /// Include vertex normals.
    #[serde(default = "default_true")]
    pub include_normals: bool,
    /// Include UV coordinates.
    #[serde(default = "default_true")]
    pub include_uvs: bool,
    /// Triangulate mesh.
    #[serde(default = "default_true")]
    pub triangulate: bool,
    /// Include skin weights.
    #[serde(default = "default_true")]
    pub include_skin_weights: bool,
    /// Save .blend file alongside GLB output.
    #[serde(default)]
    pub save_blend: bool,
}

impl Default for SkeletalMeshExportSettings {
    fn default() -> Self {
        Self {
            include_armature: true,
            include_normals: true,
            include_uvs: true,
            triangulate: true,
            include_skin_weights: true,
            save_blend: false,
        }
    }
}

impl From<SkeletalMeshExportSettings> for MeshExportSettings {
    fn from(settings: SkeletalMeshExportSettings) -> Self {
        Self {
            apply_modifiers: true,
            triangulate: settings.triangulate,
            include_normals: settings.include_normals,
            include_uvs: settings.include_uvs,
            include_vertex_colors: false,
            tangents: false,
        }
    }
}

/// Constraints for skeletal meshes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletalMeshConstraints {
    /// Maximum triangle count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_triangles: Option<u32>,
    /// Maximum bone count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bones: Option<u32>,
    /// Maximum number of materials.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_materials: Option<u32>,
}

impl From<SkeletalMeshConstraints> for MeshConstraints {
    fn from(constraints: SkeletalMeshConstraints) -> Self {
        Self {
            max_triangles: constraints.max_triangles,
            max_materials: constraints.max_materials,
            max_vertices: None,
        }
    }
}
