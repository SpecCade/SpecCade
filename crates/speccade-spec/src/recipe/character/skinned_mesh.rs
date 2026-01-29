//! Parameters and supporting types for `skeletal_mesh.skinned_mesh_v1`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::recipe::mesh::MaterialSlot;

use super::{SkeletalMeshConstraints, SkeletalMeshExportSettings, SkeletonBone, SkeletonPreset};

/// Parameters for the `skeletal_mesh.skinned_mesh_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletalMeshSkinnedMeshV1Params {
    /// External mesh file path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh_file: Option<String>,

    /// Reference to a Speccade asset id containing a mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh_asset: Option<String>,

    /// Predefined skeleton rig.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton_preset: Option<SkeletonPreset>,

    /// Custom skeleton definition (alternative to preset).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skeleton: Vec<SkeletonBone>,

    /// Binding configuration.
    pub binding: SkinnedMeshBindingConfig,

    /// Material slot overrides.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_slots: Vec<MaterialSlot>,

    /// GLB export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<SkeletalMeshExportSettings>,

    /// Mesh constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<SkeletalMeshConstraints>,
}

/// Mesh-to-armature binding configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkinnedMeshBindingConfig {
    pub mode: SkinnedMeshBindingMode,

    /// Map mesh vertex-group name -> bone name.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub vertex_group_map: HashMap<String, String>,

    /// Maximum bone influences per vertex (used for `auto_weights`).
    #[serde(default = "default_max_bone_influences")]
    pub max_bone_influences: u8,
}

fn default_max_bone_influences() -> u8 {
    4
}

/// Binding mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkinnedMeshBindingMode {
    /// Each vertex belongs 100% to one bone (vertex groups).
    Rigid,
    /// Blender computes smooth skinning weights.
    AutoWeights,
}
