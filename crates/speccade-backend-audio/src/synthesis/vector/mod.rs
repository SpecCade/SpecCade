//! Vector synthesis with 2D crossfading between multiple sound sources.
//!
//! Vector synthesis places 2-4 sound sources at corners of a 2D space and
//! crossfades between them based on a position within that space. The position
//! can be animated over time to create evolving, morphing textures.
//!
//! Classic examples: Prophet VS, Korg Wavestation.
//!
//! ```text
//! Source A -------- Source B
//!     |                |
//!     |    position    |
//!     |       *        |
//!     |                |
//! Source C -------- Source D
//! ```

mod presets;
mod synth;
mod types;

#[cfg(test)]
mod tests;

// Re-export public items to preserve the API
pub use presets::{evolving_pad, morph_texture, sweep_corners};
pub use synth::VectorSynth;
pub use types::{VectorPath, VectorPathPoint, VectorPosition, VectorSource, VectorSourceType};
