//! Sprite asset recipe types (spritesheets and animation clips).
//!
//! This module defines two recipe kinds:
//! - `sprite.sheet_v1` - Deterministic spritesheet packing with frame metadata
//! - `sprite.animation_v1` - Animation clip definitions referencing spritesheet frames

mod animation;
mod sheet;

pub use animation::*;
pub use sheet::*;

#[cfg(test)]
mod tests;
