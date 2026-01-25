//! Music preview generation.
//!
//! Generates tracker module bytes (XM/IT) for in-editor playback.

use super::PreviewResult;
use speccade_spec::{OutputFormat, OutputKind, Spec};

/// Generate a music preview from a spec.
///
/// This generates a tracker module (XM or IT) suitable for playback in the editor.
pub fn generate_music_preview(spec: &Spec, spec_path: &std::path::Path) -> PreviewResult {
    let recipe = match &spec.recipe {
        Some(r) => r,
        None => return PreviewResult::failure("music", "No recipe defined"),
    };

    if !recipe.kind.starts_with("music.") {
        return PreviewResult::failure(
            "music",
            format!("Recipe kind '{}' is not a music recipe", recipe.kind),
        );
    }

    let tmp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            return PreviewResult::failure("music", format!("Failed to create temp dir: {}", e))
        }
    };

    let tmp_path = tmp_dir.path();

    let out_root = match tmp_path.to_str() {
        Some(s) => s,
        None => {
            return PreviewResult::failure("music", "Temp output directory path is not valid UTF-8")
        }
    };

    use speccade_cli::dispatch::dispatch_generate;
    let outputs = match dispatch_generate(spec, out_root, spec_path, None) {
        Ok(outputs) => outputs,
        Err(e) => return PreviewResult::failure("music", format!("Generation failed: {}", e)),
    };

    let module_output = outputs.iter().find(|o| {
        o.kind == OutputKind::Primary && matches!(o.format, OutputFormat::Xm | OutputFormat::It)
    });

    let module_output = match module_output {
        Some(o) => o,
        None => return PreviewResult::failure("music", "No XM/IT primary output generated"),
    };

    let (format_str, mime) = match module_output.format {
        OutputFormat::Xm => ("xm", "audio/x-xm"),
        OutputFormat::It => ("it", "audio/x-it"),
        _ => return PreviewResult::failure("music", "No XM/IT primary output generated"),
    };

    let module_path = tmp_path.join(&module_output.path);
    let bytes = match std::fs::read(&module_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            return PreviewResult::failure(
                "music",
                format!("Failed to read generated module: {}", e),
            )
        }
    };

    let metadata = serde_json::json!({
        "path": module_output.path.to_string_lossy(),
        "format": format_str,
    });

    PreviewResult::success_with_metadata("music", bytes, mime, metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use speccade_spec::{AssetType, OutputSpec, Recipe};

    #[test]
    fn music_preview_no_recipe() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("test.star");
        std::fs::write(&spec_path, "# test").unwrap();

        let spec = Spec::builder("test-music", AssetType::Music)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
            .build();

        let result = generate_music_preview(&spec, &spec_path);
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("No recipe"));
    }

    #[test]
    fn music_preview_wrong_recipe_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("test.star");
        std::fs::write(&spec_path, "# test").unwrap();

        let recipe = Recipe::new("texture.procedural_v1", serde_json::json!({}));
        let spec = Spec::builder("test-music", AssetType::Texture)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Png, "textures/test.png"))
            .recipe(recipe)
            .build();

        let result = generate_music_preview(&spec, &spec_path);
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("not a music recipe"));
    }

    #[test]
    fn music_preview_tracker_song_v1_success() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("test.star");
        std::fs::write(&spec_path, "# test").unwrap();

        let recipe = Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 120,
                "speed": 6,
                "channels": 4,
                "loop": true,
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
            }),
        );

        let spec = Spec::builder("test-music", AssetType::Music)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
            .recipe(recipe)
            .build();

        let result = generate_music_preview(&spec, &spec_path);
        assert!(result.success, "{:?}", result.error);
        assert!(result.data.is_some());
        assert_eq!(result.mime_type.as_deref(), Some("audio/x-xm"));

        // Decode and sanity-check XM header.
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(result.data.unwrap())
            .unwrap();
        assert!(decoded.starts_with(b"Extended Module:"));

        let metadata = result.metadata.unwrap();
        assert_eq!(metadata["format"], "xm");
        assert!(metadata["path"]
            .as_str()
            .unwrap()
            .ends_with("songs/test.xm"));
    }
}
