//! Preview-optimized backends for the editor.
//!
//! These backends generate fast, low-fidelity previews for real-time feedback
//! during spec authoring. They trade output quality for speed.

pub mod animation;
pub mod audio;
pub mod lod;
pub mod mesh;
pub mod music;
pub mod texture;

use crate::commands::lint::LintOutput;
use serde::{Deserialize, Serialize};

/// Preview generation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewSettings {
    /// Maximum duration for audio previews in seconds.
    pub audio_max_duration: f64,
    /// Sample rate for audio previews.
    pub audio_sample_rate: u32,
    /// Maximum texture dimension for previews.
    pub texture_max_dimension: u32,
    /// Maximum triangle count for mesh previews.
    pub mesh_max_triangles: u32,
    /// Triangle threshold for LOD proxy generation.
    /// Meshes with more triangles than this will use proxy first.
    pub mesh_lod_threshold: u32,
    /// Target triangle count for LOD proxy meshes.
    pub mesh_lod_target: u32,
    /// Whether to use LOD proxy for large meshes.
    pub use_lod_proxy: bool,
}

impl Default for PreviewSettings {
    fn default() -> Self {
        Self {
            audio_max_duration: 0.5,
            audio_sample_rate: 22050,
            texture_max_dimension: 256,
            mesh_max_triangles: 1000,
            mesh_lod_threshold: 10_000,
            mesh_lod_target: 1000,
            use_lod_proxy: true,
        }
    }
}

/// Quality level for preview rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreviewQuality {
    /// Proxy/LOD preview (fast, lower fidelity).
    Proxy,
    /// Full quality preview.
    Full,
}

impl Default for PreviewQuality {
    fn default() -> Self {
        Self::Full
    }
}

/// Result of preview generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResult {
    /// Whether preview generation succeeded.
    pub success: bool,
    /// The asset type that was previewed.
    pub asset_type: String,
    /// Preview data encoded as base64.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// MIME type of the preview data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Error message if preview failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Additional metadata about the preview.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Quality level of this preview.
    #[serde(default)]
    pub quality: PreviewQuality,
    /// Whether full-quality version can be requested.
    #[serde(default)]
    pub can_refine: bool,
    /// Lint results for the generated asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lint: Option<LintOutput>,
}

impl PreviewResult {
    /// Creates a successful preview result.
    pub fn success(asset_type: &str, data: Vec<u8>, mime_type: &str) -> Self {
        use base64::Engine;
        Self {
            success: true,
            asset_type: asset_type.to_string(),
            data: Some(base64::engine::general_purpose::STANDARD.encode(&data)),
            mime_type: Some(mime_type.to_string()),
            error: None,
            metadata: None,
            quality: PreviewQuality::Full,
            can_refine: false,
            lint: None,
        }
    }

    /// Creates a successful preview result with metadata.
    pub fn success_with_metadata(
        asset_type: &str,
        data: Vec<u8>,
        mime_type: &str,
        metadata: serde_json::Value,
    ) -> Self {
        use base64::Engine;
        Self {
            success: true,
            asset_type: asset_type.to_string(),
            data: Some(base64::engine::general_purpose::STANDARD.encode(&data)),
            mime_type: Some(mime_type.to_string()),
            error: None,
            metadata: Some(metadata),
            quality: PreviewQuality::Full,
            can_refine: false,
            lint: None,
        }
    }

    /// Creates a successful preview result with metadata and quality info.
    pub fn success_with_quality(
        asset_type: &str,
        data: Vec<u8>,
        mime_type: &str,
        metadata: serde_json::Value,
        quality: PreviewQuality,
        can_refine: bool,
    ) -> Self {
        use base64::Engine;
        Self {
            success: true,
            asset_type: asset_type.to_string(),
            data: Some(base64::engine::general_purpose::STANDARD.encode(&data)),
            mime_type: Some(mime_type.to_string()),
            error: None,
            metadata: Some(metadata),
            quality,
            can_refine,
            lint: None,
        }
    }

    /// Creates a failed preview result.
    pub fn failure(asset_type: &str, error: impl Into<String>) -> Self {
        Self {
            success: false,
            asset_type: asset_type.to_string(),
            data: None,
            mime_type: None,
            error: Some(error.into()),
            metadata: None,
            quality: PreviewQuality::Full,
            can_refine: false,
            lint: None,
        }
    }

    /// Attaches lint results to this preview.
    pub fn with_lint(mut self, lint: LintOutput) -> Self {
        self.lint = Some(lint);
        self
    }
}
