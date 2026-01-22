//! VFX asset recipe types (flipbook/particle effects).
//!
//! This module defines recipe kinds for visual effects:
//! - `vfx.flipbook_v1` - Flipbook/particle effect frame generation with atlas packing

mod flipbook;

pub use flipbook::*;

#[cfg(test)]
mod tests;
