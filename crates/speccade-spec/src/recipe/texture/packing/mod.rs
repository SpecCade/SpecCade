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

mod types;
mod packed;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{ColorComponent, ChannelSource, ExtendedBuilder};
pub use packed::PackedChannels;
