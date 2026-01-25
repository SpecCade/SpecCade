//! Preview generation command for the editor.
//!
//! This command generates preview assets from compiled specs.

use serde::{Deserialize, Serialize};
use speccade_cli::compiler::{self, CompileError, CompilerConfig};

use crate::preview::{self, PreviewResult, PreviewSettings};

/// Output from the generate_preview command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePreviewOutput {
    /// Whether compilation succeeded.
    pub compile_success: bool,
    /// Compile error if compilation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compile_error: Option<String>,
    /// Preview result if compilation succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<PreviewResult>,
}

/// Generate a preview from Starlark source code.
///
/// This command compiles the source and then generates an appropriate preview
/// based on the asset type (audio, texture, mesh).
#[tauri::command]
pub fn generate_preview(
    source: String,
    filename: String,
    settings: Option<PreviewSettings>,
) -> GeneratePreviewOutput {
    // Use default settings if not provided
    let settings = settings.unwrap_or_default();

    // Configure compiler with default timeout
    let config = CompilerConfig::default();

    // Compile the Starlark source
    let spec = match compiler::compile(&filename, &source, &config) {
        Ok(result) => result.spec,
        Err(e) => {
            let error_msg = match &e {
                CompileError::Syntax { location, message } => {
                    format!("Syntax error at {}: {}", location, message)
                }
                CompileError::Runtime { location, message } => {
                    format!("Runtime error at {}: {}", location, message)
                }
                _ => e.to_string(),
            };

            return GeneratePreviewOutput {
                compile_success: false,
                compile_error: Some(error_msg),
                preview: None,
            };
        }
    };

    // Determine the asset type and generate appropriate preview
    let preview = match spec.asset_type {
        speccade_spec::AssetType::Audio => {
            preview::audio::generate_audio_preview(&spec, &settings)
        }
        speccade_spec::AssetType::Texture
        | speccade_spec::AssetType::Sprite
        | speccade_spec::AssetType::Ui
        | speccade_spec::AssetType::Font
        | speccade_spec::AssetType::Vfx => {
            preview::texture::generate_texture_preview(&spec, &settings)
        }
        speccade_spec::AssetType::StaticMesh
        | speccade_spec::AssetType::SkeletalMesh => {
            preview::mesh::generate_mesh_preview(&spec, &settings)
        }
        speccade_spec::AssetType::Music => {
            // Music uses the audio preview for now
            preview::audio::generate_audio_preview(&spec, &settings)
        }
        speccade_spec::AssetType::SkeletalAnimation => {
            // Animation preview is not yet implemented
            PreviewResult::failure("animation", "Animation preview not yet implemented")
        }
    };

    GeneratePreviewOutput {
        compile_success: true,
        compile_error: None,
        preview: Some(preview),
    }
}

/// Generate a full-quality mesh preview (for refinement after proxy).
///
/// This command bypasses LOD proxy generation and returns the full-quality mesh.
#[tauri::command]
pub fn refine_mesh_preview(
    source: String,
    filename: String,
    settings: Option<PreviewSettings>,
) -> GeneratePreviewOutput {
    // Use settings with LOD disabled for full quality
    let settings = PreviewSettings {
        use_lod_proxy: false,
        ..settings.unwrap_or_default()
    };

    // Configure compiler with default timeout
    let config = CompilerConfig::default();

    // Compile the Starlark source
    let spec = match compiler::compile(&filename, &source, &config) {
        Ok(result) => result.spec,
        Err(e) => {
            let error_msg = match &e {
                CompileError::Syntax { location, message } => {
                    format!("Syntax error at {}: {}", location, message)
                }
                CompileError::Runtime { location, message } => {
                    format!("Runtime error at {}: {}", location, message)
                }
                _ => e.to_string(),
            };

            return GeneratePreviewOutput {
                compile_success: false,
                compile_error: Some(error_msg),
                preview: None,
            };
        }
    };

    // Only process mesh types
    let preview = match spec.asset_type {
        speccade_spec::AssetType::StaticMesh
        | speccade_spec::AssetType::SkeletalMesh => {
            preview::mesh::generate_full_quality_mesh_preview(&spec, &settings)
        }
        _ => {
            PreviewResult::failure("mesh", "Refinement is only supported for mesh assets")
        }
    };

    GeneratePreviewOutput {
        compile_success: true,
        compile_error: None,
        preview: Some(preview),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_preview_compile_error() {
        let source = "{ invalid }";
        let result = generate_preview(
            source.to_string(),
            "test.star".to_string(),
            None,
        );

        assert!(!result.compile_success);
        assert!(result.compile_error.is_some());
        assert!(result.preview.is_none());
    }

    #[test]
    fn test_generate_preview_no_recipe() {
        let source = r#"
{
    "spec_version": 1,
    "asset_id": "test-audio",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {"kind": "primary", "format": "wav", "path": "test.wav"}
    ]
}
"#;
        let result = generate_preview(
            source.to_string(),
            "test.star".to_string(),
            None,
        );

        assert!(result.compile_success);
        assert!(result.compile_error.is_none());
        assert!(result.preview.is_some());

        let preview = result.preview.unwrap();
        assert!(!preview.success);
        assert!(preview.error.is_some());
        assert!(preview.error.unwrap().contains("No recipe"));
    }
}
