//! Animator rig configuration types for visual aids in Blender.

mod bone_collection;
mod bone_color;
mod config;
mod error;
mod widget;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use bone_collection::{BoneCollection, BoneCollectionPreset};
pub use bone_color::{BoneColor, BoneColorScheme};
pub use config::{AnimatorRigConfig, ArmatureDisplay};
pub use error::AnimatorRigError;
pub use widget::WidgetStyle;
