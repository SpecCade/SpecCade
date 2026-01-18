//! Embedding computation for audio and texture assets.
//!
//! This module provides deterministic feature vector computation for similarity search
//! and other downstream ML applications. Embeddings are hand-crafted features without
//! external ML model dependencies.
//!
//! ## Audio Embedding Format (48 dimensions)
//! - Spectral bins (16 bands): normalized energy per octave band
//! - RMS envelope (16 frames): normalized amplitude over time
//! - Spectral features (16): centroid, spread, rolloff, flatness, crest, etc.
//!
//! ## Texture Embedding Format (48 dimensions)
//! - Color histogram (16): combined luminance histogram bins
//! - Spatial features (16): edge density, contrast, texture measures per region
//! - Channel features (16): per-channel stats and cross-channel correlations

mod audio;
mod texture;

// Re-export public interface
pub use audio::EMBEDDING_DIM as AUDIO_EMBEDDING_DIM;
pub use texture::EMBEDDING_DIM as TEXTURE_EMBEDDING_DIM;

/// Compute audio embedding from mono samples.
///
/// Returns a 48-dimension feature vector capturing:
/// - Spectral energy distribution (16 bands)
/// - Temporal RMS envelope (16 frames)
/// - Spectral shape features (16 values)
pub fn compute_audio_embedding(samples: &[f32], sample_rate: u32) -> Vec<f64> {
    audio::compute(samples, sample_rate)
}

/// Compute texture embedding from pixels.
///
/// Returns a 48-dimension feature vector capturing:
/// - Luminance histogram (16 bins)
/// - Spatial features (16 values): edge density, contrast, texture measures
/// - Channel features (16 values): per-channel stats and correlations
pub fn compute_texture_embedding(pixels: &[u8], width: u32, height: u32, channels: u8) -> Vec<f64> {
    texture::compute(pixels, width, height, channels)
}
