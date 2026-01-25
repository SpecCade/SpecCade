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
        speccade_spec::AssetType::Audio => preview::audio::generate_audio_preview(&spec, &settings),
        speccade_spec::AssetType::Texture
        | speccade_spec::AssetType::Sprite
        | speccade_spec::AssetType::Ui
        | speccade_spec::AssetType::Font
        | speccade_spec::AssetType::Vfx => {
            preview::texture::generate_texture_preview(&spec, &settings)
        }
        speccade_spec::AssetType::StaticMesh | speccade_spec::AssetType::SkeletalMesh => {
            preview::mesh::generate_mesh_preview(&spec, &settings)
        }
        speccade_spec::AssetType::Music => {
            preview::music::generate_music_preview(&spec, std::path::Path::new(&filename))
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
        speccade_spec::AssetType::StaticMesh | speccade_spec::AssetType::SkeletalMesh => {
            preview::mesh::generate_full_quality_mesh_preview(&spec, &settings)
        }
        _ => PreviewResult::failure("mesh", "Refinement is only supported for mesh assets"),
    };

    GeneratePreviewOutput {
        compile_success: true,
        compile_error: None,
        preview: Some(preview),
    }
}

/// Output from full asset generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateFullOutput {
    /// Whether the generation succeeded.
    pub success: bool,
    /// List of generated output files.
    pub outputs: Vec<GeneratedFile>,
    /// Error message if generation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Elapsed time in milliseconds.
    pub elapsed_ms: u64,
}

/// A generated output file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// Relative path to the generated file.
    pub path: String,
    /// Size of the file in bytes.
    pub size_bytes: u64,
    /// Output format (e.g., "wav", "png", "glb").
    pub format: String,
}

/// Generate full assets from a spec.
///
/// This command compiles the Starlark source and generates all output files
/// to the specified output directory.
#[tauri::command]
pub fn generate_full(source: String, filename: String, output_dir: String) -> GenerateFullOutput {
    use speccade_cli::dispatch::{dispatch_generate, DispatchError};
    use std::path::Path;
    use std::time::Instant;

    let start = Instant::now();

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

            return GenerateFullOutput {
                success: false,
                outputs: vec![],
                error: Some(error_msg),
                elapsed_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    // Create a dummy spec path for the dispatch (uses the filename as a hint)
    let spec_path = Path::new(&filename);

    // Dispatch generation to the appropriate backend
    let output_results = match dispatch_generate(&spec, &output_dir, spec_path, None) {
        Ok(results) => results,
        Err(e) => {
            let error_msg = match &e {
                DispatchError::NoRecipe => "No recipe defined in the spec".to_string(),
                DispatchError::BackendNotImplemented(kind) => {
                    format!("Backend not implemented for recipe kind: {}", kind)
                }
                DispatchError::BackendError(msg) => format!("Backend error: {}", msg),
            };

            return GenerateFullOutput {
                success: false,
                outputs: vec![],
                error: Some(error_msg),
                elapsed_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    // Map OutputResults to GeneratedFiles
    let output_path = Path::new(&output_dir);
    let outputs: Vec<GeneratedFile> = output_results
        .iter()
        .filter_map(|result| {
            let full_path = output_path.join(&result.path);
            let size_bytes = std::fs::metadata(&full_path).map(|m| m.len()).unwrap_or(0);

            Some(GeneratedFile {
                path: result.path.to_string_lossy().to_string(),
                size_bytes,
                format: format!("{:?}", result.format).to_lowercase(),
            })
        })
        .collect();

    GenerateFullOutput {
        success: true,
        outputs,
        error: None,
        elapsed_ms: start.elapsed().as_millis() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_preview_compile_error() {
        let source = "{ invalid }";
        let result = generate_preview(source.to_string(), "test.star".to_string(), None);

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
        let result = generate_preview(source.to_string(), "test.star".to_string(), None);

        assert!(result.compile_success);
        assert!(result.compile_error.is_none());
        assert!(result.preview.is_some());

        let preview = result.preview.unwrap();
        assert!(!preview.success);
        assert!(preview.error.is_some());
        assert!(preview.error.unwrap().contains("No recipe"));
    }

    #[test]
    fn test_generate_preview_music_success() {
        let tmp = tempfile::tempdir().unwrap();
        let filename = tmp.path().join("test.star");
        std::fs::write(&filename, "# test").unwrap();

        let source = r#"
{
    "spec_version": 1,
    "asset_id": "test-music",
    "asset_type": "music",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {"kind": "primary", "format": "xm", "path": "songs/test.xm"}
    ],
    "recipe": {
        "kind": "music.tracker_song_v1",
        "params": {
            "format": "xm",
            "bpm": 120,
            "speed": 6,
            "channels": 4,
            "loop": True,
            "instruments": [
                {
                    "name": "Test",
                    "synthesis": { "type": "sine" },
                    "default_volume": 64
                }
            ],
            "patterns": {
                "intro": {
                    "rows": 16,
                    "data": [
                        {"row": 0, "channel": 0, "note": "C4", "inst": 0, "vol": 64},
                        {"row": 4, "channel": 0, "note": "OFF", "inst": 0}
                    ]
                }
            },
            "arrangement": [
                {"pattern": "intro", "repeat": 1}
            ]
        }
    }
}
"#;

        let result = generate_preview(
            source.to_string(),
            filename.to_string_lossy().to_string(),
            None,
        );

        assert!(result.compile_success, "{:?}", result.compile_error);
        let preview = result.preview.expect("expected preview");
        assert!(preview.success, "{:?}", preview.error);
        assert!(
            matches!(
                preview.mime_type.as_deref(),
                Some("audio/x-xm") | Some("audio/x-it")
            ),
            "unexpected mime_type: {:?}",
            preview.mime_type
        );
    }
}
