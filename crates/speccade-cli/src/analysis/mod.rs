//! Asset analysis module for extracting quality metrics.
//!
//! This module provides deterministic analysis of generated assets (audio and texture)
//! to enable LLM-driven iteration loops and quality gating. All outputs are designed
//! to be byte-identical across runs on the same input.
//!
//! ## Supported Formats
//!
//! - **Audio**: WAV files (PCM, 8/16/24/32-bit)
//! - **Texture**: PNG files (Grayscale, RGB, RGBA)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use speccade_cli::analysis::{audio, texture};
//!
//! // Analyze audio
//! let wav_data = std::fs::read("sound.wav").unwrap();
//! let audio_metrics = audio::analyze_wav(&wav_data).unwrap();
//!
//! // Analyze texture
//! let png_data = std::fs::read("texture.png").unwrap();
//! let texture_metrics = texture::analyze_png(&png_data).unwrap();
//! ```

pub mod audio;
pub mod embeddings;
pub mod perceptual;
pub mod texture;

/// Recognized audio extensions.
pub const AUDIO_EXTENSIONS: &[&str] = &["wav"];

/// Recognized texture extensions.
pub const TEXTURE_EXTENSIONS: &[&str] = &["png"];

/// Detect asset type from file extension.
pub fn detect_asset_type(path: &std::path::Path) -> Option<AssetAnalysisType> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    if AUDIO_EXTENSIONS.contains(&ext.as_str()) {
        Some(AssetAnalysisType::Audio)
    } else if TEXTURE_EXTENSIONS.contains(&ext.as_str()) {
        Some(AssetAnalysisType::Texture)
    } else {
        None
    }
}

/// Type of asset for analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetAnalysisType {
    /// Audio asset (WAV)
    Audio,
    /// Texture asset (PNG)
    Texture,
}

impl AssetAnalysisType {
    /// Returns the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetAnalysisType::Audio => "audio",
            AssetAnalysisType::Texture => "texture",
        }
    }
}

impl std::fmt::Display for AssetAnalysisType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_detect_audio() {
        assert_eq!(
            detect_asset_type(Path::new("sound.wav")),
            Some(AssetAnalysisType::Audio)
        );
        assert_eq!(
            detect_asset_type(Path::new("SOUND.WAV")),
            Some(AssetAnalysisType::Audio)
        );
    }

    #[test]
    fn test_detect_texture() {
        assert_eq!(
            detect_asset_type(Path::new("texture.png")),
            Some(AssetAnalysisType::Texture)
        );
        assert_eq!(
            detect_asset_type(Path::new("TEXTURE.PNG")),
            Some(AssetAnalysisType::Texture)
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_asset_type(Path::new("file.mp3")), None);
        assert_eq!(detect_asset_type(Path::new("file.jpg")), None);
        assert_eq!(detect_asset_type(Path::new("file")), None);
    }

    #[test]
    fn test_asset_type_display() {
        assert_eq!(AssetAnalysisType::Audio.to_string(), "audio");
        assert_eq!(AssetAnalysisType::Texture.to_string(), "texture");
    }
}
