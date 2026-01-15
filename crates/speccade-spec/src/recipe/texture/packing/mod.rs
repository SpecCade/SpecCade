//! Channel packing types for texture generation.
//!
//! This module provides an unopinionated channel packing system where users define
//! their own map keys (not predefined names like "albedo"), then reference those
//! keys when packing channels into output textures.
//!
//! # Core Concept
//!
//! ```text
//! recipe.params.maps = { "my_key": <generation_params> }  // User-defined keys
//! outputs[].channels = { "r": "my_key" }                   // Reference by key
//! ```

mod packed;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use packed::PackedChannels;
pub use types::{ChannelSource, ColorComponent, ExtendedBuilder};
