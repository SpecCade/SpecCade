//! Texture preview generation.
//!
//! Generates quick texture previews at 256x256 for real-time feedback.

use super::{PreviewQuality, PreviewResult, PreviewSettings};
use crate::commands::lint::lint_asset_bytes;
use speccade_spec::Spec;

/// Generate a texture preview from a spec.
///
/// This generates a low-resolution preview suitable for quick display in the editor.
/// Settings: 256x256 maximum dimension.
pub fn generate_texture_preview(spec: &Spec, settings: &PreviewSettings) -> PreviewResult {
    // Check if spec has a recipe
    let recipe = match &spec.recipe {
        Some(r) => r,
        None => return PreviewResult::failure("texture", "No recipe defined"),
    };

    // Only handle texture-related recipes
    let is_texture_recipe = recipe.kind.starts_with("texture.")
        || recipe.kind.starts_with("sprite.")
        || recipe.kind.starts_with("vfx.")
        || recipe.kind.starts_with("ui.")
        || recipe.kind.starts_with("font.");

    if !is_texture_recipe {
        return PreviewResult::failure(
            "texture",
            format!("Recipe kind '{}' is not a texture recipe", recipe.kind),
        );
    }

    // Create a temporary directory for preview generation
    let tmp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            return PreviewResult::failure("texture", format!("Failed to create temp dir: {}", e))
        }
    };

    let tmp_path = tmp_dir.path();
    let spec_path = tmp_path.join("preview.star");

    // For texture previews, we modify the spec to use a smaller resolution
    // Create a modified spec with reduced resolution
    let preview_spec = create_preview_spec(spec, settings);

    // Use the existing dispatch
    use speccade_cli::dispatch::dispatch_generate;

    match dispatch_generate(&preview_spec, tmp_path.to_str().unwrap(), &spec_path, None) {
        Ok(outputs) => {
            // Find the primary PNG output
            let png_output = outputs
                .iter()
                .find(|o| matches!(o.format, speccade_spec::OutputFormat::Png));

            match png_output {
                Some(output) => {
                    // Read the generated PNG file
                    let png_path = tmp_path.join(&output.path);
                    match std::fs::read(&png_path) {
                        Ok(png_bytes) => {
                            // Run lint on the generated texture
                            let lint_result = lint_asset_bytes(&png_path, &png_bytes, Some(spec));

                            let metadata = serde_json::json!({
                                "max_dimension": settings.texture_max_dimension,
                                "path": output.path,
                            });
                            PreviewResult::success_with_quality(
                                "texture",
                                png_bytes,
                                "image/png",
                                metadata,
                                PreviewQuality::Proxy,
                                false,
                            )
                            .with_lint(lint_result)
                        }
                        Err(e) => {
                            PreviewResult::failure("texture", format!("Failed to read PNG: {}", e))
                        }
                    }
                }
                None => PreviewResult::failure("texture", "No PNG output generated"),
            }
        }
        Err(e) => PreviewResult::failure("texture", format!("Generation failed: {}", e)),
    }
}

/// Create a modified spec with reduced resolution for preview.
fn create_preview_spec(spec: &Spec, settings: &PreviewSettings) -> Spec {
    let mut preview_spec = spec.clone();

    // Modify the recipe params to use preview resolution
    if let Some(ref mut recipe) = preview_spec.recipe {
        if let Some(resolution) = recipe.params.get("resolution") {
            if let Some(arr) = resolution.as_array() {
                if arr.len() >= 2 {
                    let orig_width = arr[0].as_u64().unwrap_or(256) as u32;
                    let orig_height = arr[1].as_u64().unwrap_or(256) as u32;

                    // Scale down to max dimension while maintaining aspect ratio
                    let max_dim = settings.texture_max_dimension;
                    let scale = if orig_width > orig_height {
                        max_dim as f32 / orig_width as f32
                    } else {
                        max_dim as f32 / orig_height as f32
                    };

                    // Only scale down, never up
                    let scale = scale.min(1.0);
                    let new_width = ((orig_width as f32 * scale) as u32).max(1);
                    let new_height = ((orig_height as f32 * scale) as u32).max(1);

                    recipe.params["resolution"] = serde_json::json!([new_width, new_height]);
                }
            }
        }
    }

    preview_spec
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe};

    #[test]
    fn test_texture_preview_no_recipe() {
        let spec = Spec::builder("test-texture", AssetType::Texture)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Png, "test.png"))
            .build();

        let settings = PreviewSettings::default();
        let result = generate_texture_preview(&spec, &settings);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("No recipe"));
    }

    #[test]
    fn test_texture_preview_wrong_recipe_type() {
        let recipe = Recipe::new("audio_v1", serde_json::json!({}));
        let spec = Spec::builder("test-texture", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(recipe)
            .build();

        let settings = PreviewSettings::default();
        let result = generate_texture_preview(&spec, &settings);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("not a texture recipe"));
    }

    #[test]
    fn test_create_preview_spec_scales_resolution() {
        let recipe = Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({
                "resolution": [1024, 512],
                "tileable": true,
                "nodes": []
            }),
        );

        let spec = Spec::builder("test-texture", AssetType::Texture)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Png, "test.png"))
            .recipe(recipe)
            .build();

        let settings = PreviewSettings {
            texture_max_dimension: 256,
            ..Default::default()
        };

        let preview_spec = create_preview_spec(&spec, &settings);
        let preview_resolution = preview_spec
            .recipe
            .as_ref()
            .unwrap()
            .params
            .get("resolution")
            .unwrap()
            .as_array()
            .unwrap();

        // 1024x512 scaled to 256 max dimension should be 256x128
        assert_eq!(preview_resolution[0].as_u64().unwrap(), 256);
        assert_eq!(preview_resolution[1].as_u64().unwrap(), 128);
    }
}
