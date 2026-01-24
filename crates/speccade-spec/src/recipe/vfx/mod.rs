//! VFX asset recipe types (flipbook/particle effects).
//!
//! This module defines recipe kinds for visual effects:
//! - `vfx.flipbook_v1` - Flipbook/particle effect frame generation with atlas packing
//! - `vfx.particle_profile_v1` - Particle rendering profile presets (metadata-only)

mod flipbook;
mod particle_profile;

pub use flipbook::*;
pub use particle_profile::*;

#[cfg(test)]
mod tests;
