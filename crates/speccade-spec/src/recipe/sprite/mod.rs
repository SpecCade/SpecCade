//! Sprite asset recipe types (spritesheets and animation clips).
//!
//! This module defines three recipe kinds:
//! - `sprite.sheet_v1` - Deterministic spritesheet packing with frame metadata
//! - `sprite.animation_v1` - Animation clip definitions referencing spritesheet frames
//! - `sprite.render_from_mesh_v1` - Render 3D mesh to sprite atlas (Tier 2, Blender backend)

mod animation;
mod render_from_mesh;
mod sheet;

pub use animation::*;
pub use render_from_mesh::*;
pub use sheet::*;

#[cfg(test)]
mod tests;
