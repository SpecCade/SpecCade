//! Layer mixing with volume and stereo panning.
//!
//! This module combines multiple audio layers with independent volume and pan
//! controls, producing either mono or stereo output.

#[allow(clippy::module_inception)]
mod mixer;
mod processing;
mod types;

#[cfg(test)]
mod tests_delay;
#[cfg(test)]
mod tests_layer;
#[cfg(test)]
mod tests_mixer_basic;
#[cfg(test)]
mod tests_mixer_output;
#[cfg(test)]
mod tests_mixing;
#[cfg(test)]
mod tests_normalization;
#[cfg(test)]
mod tests_soft_clip;
#[cfg(test)]
mod tests_stereo_output;

// Re-export public API
pub use mixer::Mixer;
pub use processing::{normalize, normalize_stereo, soft_clip, soft_clip_buffer};
pub use types::{Layer, MixerOutput, StereoOutput};
