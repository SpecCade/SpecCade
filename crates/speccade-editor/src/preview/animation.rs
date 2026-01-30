//! Animation preview generation.
//!
//! Generates skeletal animation previews as GLB files with embedded animation tracks.
//! Uses balanced quality settings: full rig accuracy, reduced mesh detail.

use super::{PreviewQuality, PreviewResult, PreviewSettings};
use crate::commands::lint::lint_asset_bytes;
use speccade_spec::Spec;

/// Generate an animation preview from a spec.
///
/// This generates a GLB with embedded animation tracks suitable for playback in three.js.
/// The mesh is reduced quality (LOD proxy) but the rig/bone transforms are full fidelity.
pub fn generate_animation_preview(spec: &Spec, _settings: &PreviewSettings) -> PreviewResult {
    // Check if spec has a recipe
    let recipe = match &spec.recipe {
        Some(r) => r,
        None => return PreviewResult::failure("animation", "No recipe defined"),
    };

    // Only handle animation recipes
    let is_animation_recipe = recipe.kind.starts_with("skeletal_animation.");
    if !is_animation_recipe {
        return PreviewResult::failure(
            "animation",
            format!("Recipe kind '{}' is not an animation recipe", recipe.kind),
        );
    }

    // Create a temporary directory for preview generation
    let tmp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            return PreviewResult::failure("animation", format!("Failed to create temp dir: {}", e))
        }
    };

    let tmp_path = tmp_dir.path();
    let spec_path = tmp_path.join("preview.star");

    // Use the existing dispatch
    use speccade_cli::dispatch::dispatch_generate;

    match dispatch_generate(spec, tmp_path.to_str().unwrap(), &spec_path, None) {
        Ok(outputs) => {
            // Find the primary GLB output
            let glb_output = outputs
                .iter()
                .find(|o| matches!(o.format, speccade_spec::OutputFormat::Glb));

            match glb_output {
                Some(output) => {
                    // Read the generated GLB file
                    let glb_path = tmp_path.join(&output.path);
                    match std::fs::read(&glb_path) {
                        Ok(glb_bytes) => {
                            // Run lint on the generated animation
                            let lint_result = lint_asset_bytes(&glb_path, &glb_bytes, Some(spec));

                            // Extract animation metadata
                            let metadata = extract_animation_metadata(&glb_bytes, spec);

                            PreviewResult::success_with_quality(
                                "animation",
                                glb_bytes,
                                "model/gltf-binary",
                                metadata,
                                PreviewQuality::Full, // Animation previews are always full rig quality
                                false,                // No refinement needed
                            )
                            .with_lint(lint_result)
                        }
                        Err(e) => {
                            PreviewResult::failure("animation", format!("Failed to read GLB: {}", e))
                        }
                    }
                }
                None => PreviewResult::failure("animation", "No GLB output generated"),
            }
        }
        Err(e) => PreviewResult::failure("animation", format!("Generation failed: {}", e)),
    }
}

/// Extract metadata from an animation GLB file.
fn extract_animation_metadata(glb_bytes: &[u8], spec: &Spec) -> serde_json::Value {
    // Try to parse the GLB to extract animation info
    match gltf::Glb::from_slice(glb_bytes) {
        Ok(glb) => {
            match gltf::Gltf::from_slice(&glb.json) {
                Ok(gltf) => {
                    let mut animations = Vec::new();
                    let mut total_duration = 0.0f32;
                    let mut bone_count = 0u32;

                    // Count bones from skins
                    for skin in gltf.skins() {
                        bone_count = bone_count.max(skin.joints().count() as u32);
                    }

                    // Get animation info
                    for anim in gltf.animations() {
                        let name = anim.name().unwrap_or("Unnamed").to_string();
                        let mut duration = 0.0f32;
                        let mut channel_count = 0u32;

                        for channel in anim.channels() {
                            channel_count += 1;
                            let sampler = channel.sampler();
                            let input_accessor = sampler.input();
                            if let (Some(min), Some(max)) = (input_accessor.min(), input_accessor.max()) {
                                if let (Some(max_time), Some(min_time)) = (
                                    max.as_array().and_then(|a| a.first()).and_then(|v| v.as_f64()),
                                    min.as_array().and_then(|a| a.first()).and_then(|v| v.as_f64()),
                                ) {
                                    duration = duration.max((max_time - min_time) as f32);
                                }
                            }
                        }

                        total_duration = total_duration.max(duration);
                        animations.push(serde_json::json!({
                            "name": name,
                            "duration": duration,
                            "channels": channel_count,
                        }));
                    }

                    // Extract keyframe count from spec if available
                    let keyframe_count = extract_keyframe_count_from_spec(spec);

                    serde_json::json!({
                        "bone_count": bone_count,
                        "duration_seconds": total_duration,
                        "animations": animations,
                        "keyframe_count": keyframe_count,
                    })
                }
                Err(_) => serde_json::json!({
                    "parse_error": "Failed to parse GLTF JSON"
                }),
            }
        }
        Err(_) => serde_json::json!({
            "parse_error": "Failed to parse GLB"
        }),
    }
}

/// Extract keyframe count from spec recipe params.
fn extract_keyframe_count_from_spec(spec: &Spec) -> Option<u32> {
    let recipe = spec.recipe.as_ref()?;
    let params = recipe.params.as_object()?;

    // Try keyframes array
    if let Some(keyframes) = params.get("keyframes") {
        if let Some(arr) = keyframes.as_array() {
            return Some(arr.len() as u32);
        }
    }

    // Try phases array (for rigged animations)
    if let Some(phases) = params.get("phases") {
        if let Some(arr) = phases.as_array() {
            // Count keyframes across all phases
            let mut count = 0u32;
            for phase in arr {
                if let Some(kfs) = phase.get("keyframes").and_then(|v| v.as_array()) {
                    count += kfs.len() as u32;
                }
            }
            if count > 0 {
                return Some(count);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe};

    #[test]
    fn test_animation_preview_no_recipe() {
        let spec = Spec::builder("test-anim", AssetType::SkeletalAnimation)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Glb, "test.glb"))
            .build();

        let settings = PreviewSettings::default();
        let result = generate_animation_preview(&spec, &settings);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("No recipe"));
    }

    #[test]
    fn test_animation_preview_wrong_recipe_type() {
        let recipe = Recipe::new("audio_v1", serde_json::json!({}));
        let spec = Spec::builder("test-anim", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(recipe)
            .build();

        let settings = PreviewSettings::default();
        let result = generate_animation_preview(&spec, &settings);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("not an animation recipe"));
    }
}
