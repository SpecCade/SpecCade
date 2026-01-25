//! Audio preview generation.
//!
//! Generates quick audio previews at 22kHz, max 0.5s for real-time feedback.

use super::{PreviewResult, PreviewSettings};
use speccade_spec::Spec;

/// Generate an audio preview from a spec.
///
/// This generates a low-fidelity preview suitable for quick playback in the editor.
/// Settings: 22kHz sample rate, max 0.5s duration.
pub fn generate_audio_preview(spec: &Spec, settings: &PreviewSettings) -> PreviewResult {
    // Check if spec has a recipe
    let recipe = match &spec.recipe {
        Some(r) => r,
        None => return PreviewResult::failure("audio", "No recipe defined"),
    };

    // Only handle audio recipes
    if !recipe.kind.starts_with("audio") {
        return PreviewResult::failure(
            "audio",
            format!("Recipe kind '{}' is not an audio recipe", recipe.kind),
        );
    }

    // Create a temporary directory for preview generation
    let tmp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            return PreviewResult::failure("audio", format!("Failed to create temp dir: {}", e))
        }
    };

    let tmp_path = tmp_dir.path();
    let spec_path = tmp_path.join("preview.star");

    // Use the existing dispatch with preview duration
    use speccade_cli::dispatch::dispatch_generate;

    let preview_duration = Some(settings.audio_max_duration);

    match dispatch_generate(
        spec,
        tmp_path.to_str().unwrap(),
        &spec_path,
        preview_duration,
    ) {
        Ok(outputs) => {
            // Find the primary WAV output
            let wav_output = outputs
                .iter()
                .find(|o| matches!(o.format, speccade_spec::OutputFormat::Wav));

            match wav_output {
                Some(output) => {
                    // Read the generated WAV file
                    let wav_path = tmp_path.join(&output.path);
                    match std::fs::read(&wav_path) {
                        Ok(wav_bytes) => {
                            // Include sample count and duration in metadata
                            let metadata = serde_json::json!({
                                "sample_rate": settings.audio_sample_rate,
                                "max_duration": settings.audio_max_duration,
                                "path": output.path,
                            });
                            PreviewResult::success_with_metadata(
                                "audio",
                                wav_bytes,
                                "audio/wav",
                                metadata,
                            )
                        }
                        Err(e) => {
                            PreviewResult::failure("audio", format!("Failed to read WAV: {}", e))
                        }
                    }
                }
                None => PreviewResult::failure("audio", "No WAV output generated"),
            }
        }
        Err(e) => PreviewResult::failure("audio", format!("Generation failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe};

    #[test]
    fn test_audio_preview_no_recipe() {
        let spec = Spec::builder("test-audio", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .build();

        let settings = PreviewSettings::default();
        let result = generate_audio_preview(&spec, &settings);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("No recipe"));
    }

    #[test]
    fn test_audio_preview_wrong_recipe_type() {
        let recipe = Recipe::new("texture.procedural_v1", serde_json::json!({}));
        let spec = Spec::builder("test-audio", AssetType::Texture)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Png, "test.png"))
            .recipe(recipe)
            .build();

        let settings = PreviewSettings::default();
        let result = generate_audio_preview(&spec, &settings);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("not an audio recipe"));
    }
}
