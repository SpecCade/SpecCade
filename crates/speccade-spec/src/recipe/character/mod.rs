//! Skeletal mesh (character) recipe types.

mod armature_driven;
mod body_parts;
mod legacy;
mod materials;
mod skeleton;
mod skinned_mesh;
mod texturing;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use armature_driven::*;
pub use body_parts::{BodyPart, BodyPartMesh};
pub use legacy::{
    BaseRadius, BulgeFactor, Instance, LegacyPart, ScaleFactor, SkinningType, Step, StepDefinition,
    SubPart, SubPartOrList, TiltFactor,
};
pub use materials::{SkeletalMeshConstraints, SkeletalMeshExportSettings, SkinningSettings};
pub use skeleton::{SkeletonBone, SkeletonPreset};
pub use skinned_mesh::*;
pub use texturing::{RegionColor, TextureRegion, Texturing, UvMode};
