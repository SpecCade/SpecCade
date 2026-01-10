//! Skeletal mesh (character) recipe types.

mod body_parts;
mod legacy;
mod materials;
mod skeleton;
mod texturing;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::mesh::MaterialSlot;

// Re-export all public types
pub use body_parts::{BodyPart, BodyPartMesh};
pub use legacy::{
    BaseRadius, BulgeFactor, Instance, LegacyPart, ScaleFactor, SkinningType, Step,
    StepDefinition, SubPart, SubPartOrList, TiltFactor,
};
pub use materials::{SkinningSettings, SkeletalMeshConstraints, SkeletalMeshExportSettings};
pub use skeleton::{SkeletonBone, SkeletonPreset};
pub use texturing::{RegionColor, TextureRegion, Texturing, UvMode};

/// Parameters for the `skeletal_mesh.blender_rigged_mesh_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletalMeshBlenderRiggedMeshV1Params {
    /// Predefined skeleton rig.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton_preset: Option<SkeletonPreset>,
    /// Custom skeleton definition (alternative to preset).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skeleton: Vec<SkeletonBone>,
    /// Body part mesh definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body_parts: Vec<BodyPart>,
    /// Legacy parts definition (dict-style, keyed by part name).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub parts: HashMap<String, LegacyPart>,
    /// Material slot definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_slots: Vec<MaterialSlot>,
    /// Skinning settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skinning: Option<SkinningSettings>,
    /// GLB export settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<SkeletalMeshExportSettings>,
    /// Mesh constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<SkeletalMeshConstraints>,
    /// Triangle budget for validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tri_budget: Option<u32>,
    /// Texturing options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texturing: Option<Texturing>,
}
