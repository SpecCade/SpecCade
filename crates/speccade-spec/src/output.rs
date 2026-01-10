//! Output specification types.

use serde::{Deserialize, Serialize};

use crate::recipe::texture::PackedChannels;

/// Output kind (what role the output serves).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputKind {
    /// Primary asset output (the main generated file).
    Primary,
    /// Metadata sidecar file (JSON, etc.).
    Metadata,
    /// Preview file (thumbnail, preview audio, etc.).
    Preview,
    /// Packed texture output (channel packing).
    Packed,
}

impl std::fmt::Display for OutputKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputKind::Primary => write!(f, "primary"),
            OutputKind::Metadata => write!(f, "metadata"),
            OutputKind::Preview => write!(f, "preview"),
            OutputKind::Packed => write!(f, "packed"),
        }
    }
}

/// Output format (file type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// WAV audio format.
    Wav,
    /// OGG Vorbis audio format.
    Ogg,
    /// FastTracker II Extended Module.
    Xm,
    /// Impulse Tracker module.
    It,
    /// PNG image format.
    Png,
    /// Binary glTF format.
    Glb,
    /// Text glTF format.
    Gltf,
    /// JSON metadata format.
    Json,
    /// Blender project file.
    Blend,
}

impl OutputFormat {
    /// Returns the expected file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Wav => "wav",
            OutputFormat::Ogg => "ogg",
            OutputFormat::Xm => "xm",
            OutputFormat::It => "it",
            OutputFormat::Png => "png",
            OutputFormat::Glb => "glb",
            OutputFormat::Gltf => "gltf",
            OutputFormat::Json => "json",
            OutputFormat::Blend => "blend",
        }
    }

    /// Checks if this format is an audio format.
    pub fn is_audio(&self) -> bool {
        matches!(self, OutputFormat::Wav | OutputFormat::Ogg)
    }

    /// Checks if this format is a music/tracker format.
    pub fn is_music(&self) -> bool {
        matches!(self, OutputFormat::Xm | OutputFormat::It)
    }

    /// Checks if this format is an image format.
    pub fn is_image(&self) -> bool {
        matches!(self, OutputFormat::Png)
    }

    /// Checks if this format is a 3D model format.
    pub fn is_mesh(&self) -> bool {
        matches!(self, OutputFormat::Glb | OutputFormat::Gltf)
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

/// Specification for a single output artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputSpec {
    /// The kind of output (primary, metadata, preview, packed).
    pub kind: OutputKind,
    /// The file format.
    pub format: OutputFormat,
    /// Relative path under the output root.
    pub path: String,
    /// Channel packing specification (only for kind=packed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<PackedChannels>,
}

impl OutputSpec {
    /// Creates a new output specification.
    pub fn new(kind: OutputKind, format: OutputFormat, path: impl Into<String>) -> Self {
        Self {
            kind,
            format,
            path: path.into(),
            channels: None,
        }
    }

    /// Creates a primary output specification.
    pub fn primary(format: OutputFormat, path: impl Into<String>) -> Self {
        Self::new(OutputKind::Primary, format, path)
    }

    /// Creates a metadata output specification.
    pub fn metadata(path: impl Into<String>) -> Self {
        Self::new(OutputKind::Metadata, OutputFormat::Json, path)
    }

    /// Creates a preview output specification.
    pub fn preview(format: OutputFormat, path: impl Into<String>) -> Self {
        Self::new(OutputKind::Preview, format, path)
    }

    /// Creates a packed texture output specification with channel mapping.
    pub fn packed(format: OutputFormat, path: impl Into<String>, channels: PackedChannels) -> Self {
        Self {
            kind: OutputKind::Packed,
            format,
            path: path.into(),
            channels: Some(channels),
        }
    }

    /// Returns the expected file extension based on the format.
    pub fn expected_extension(&self) -> &'static str {
        self.format.extension()
    }

    /// Extracts the file extension from the path.
    pub fn path_extension(&self) -> Option<&str> {
        self.path.rsplit('.').next()
    }

    /// Checks if the path extension matches the format.
    pub fn extension_matches(&self) -> bool {
        self.path_extension()
            .map(|ext| ext.eq_ignore_ascii_case(self.expected_extension()))
            .unwrap_or(false)
    }
}

/// Variant specification for producing multiple related outputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantSpec {
    /// Identifier for the variant.
    pub variant_id: String,
    /// Offset added to base seed for this variant.
    pub seed_offset: u32,
}

impl VariantSpec {
    /// Creates a new variant specification.
    pub fn new(variant_id: impl Into<String>, seed_offset: u32) -> Self {
        Self {
            variant_id: variant_id.into(),
            seed_offset,
        }
    }
}

/// Target game engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineTarget {
    /// Godot game engine.
    Godot,
    /// Unity game engine.
    Unity,
    /// Unreal Engine.
    Unreal,
}

impl std::fmt::Display for EngineTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineTarget::Godot => write!(f, "godot"),
            EngineTarget::Unity => write!(f, "unity"),
            EngineTarget::Unreal => write!(f, "unreal"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Wav.extension(), "wav");
        assert_eq!(OutputFormat::Png.extension(), "png");
        assert_eq!(OutputFormat::Glb.extension(), "glb");
    }

    #[test]
    fn test_output_format_categories() {
        assert!(OutputFormat::Wav.is_audio());
        assert!(OutputFormat::Ogg.is_audio());
        assert!(!OutputFormat::Png.is_audio());

        assert!(OutputFormat::Xm.is_music());
        assert!(OutputFormat::It.is_music());
        assert!(!OutputFormat::Wav.is_music());

        assert!(OutputFormat::Glb.is_mesh());
        assert!(OutputFormat::Gltf.is_mesh());
        assert!(!OutputFormat::Png.is_mesh());
    }

    #[test]
    fn test_output_spec_extension_matches() {
        let spec = OutputSpec::primary(OutputFormat::Wav, "sounds/laser.wav");
        assert!(spec.extension_matches());

        let spec_bad = OutputSpec::primary(OutputFormat::Wav, "sounds/laser.png");
        assert!(!spec_bad.extension_matches());

        // Case insensitive
        let spec_upper = OutputSpec::primary(OutputFormat::Wav, "sounds/laser.WAV");
        assert!(spec_upper.extension_matches());
    }

    #[test]
    fn test_output_kind_serde() {
        let json = serde_json::to_string(&OutputKind::Primary).unwrap();
        assert_eq!(json, "\"primary\"");

        let kind: OutputKind = serde_json::from_str("\"metadata\"").unwrap();
        assert_eq!(kind, OutputKind::Metadata);
    }

    #[test]
    fn test_variant_spec() {
        let variant = VariantSpec::new("soft", 0);
        assert_eq!(variant.variant_id, "soft");
        assert_eq!(variant.seed_offset, 0);
    }
}
