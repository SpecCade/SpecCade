//! Mesh preview generation.
//!
//! Generates quick mesh previews with LOD proxy (target ~1000 triangles).
//! For meshes >10k triangles, a decimated proxy is shown first (<100ms),
//! with full quality available on request.

use super::lod::{generate_lod_proxy, LodConfig};
use super::{PreviewQuality, PreviewResult, PreviewSettings};
use speccade_spec::Spec;

/// Generate a mesh preview from a spec.
///
/// This generates a low-poly proxy mesh suitable for quick display in the editor.
/// Settings: ~1000 triangles maximum.
///
/// Note: Mesh generation requires Blender, so this may fail if Blender is not available.
pub fn generate_mesh_preview(spec: &Spec, settings: &PreviewSettings) -> PreviewResult {
    // Check if spec has a recipe
    let recipe = match &spec.recipe {
        Some(r) => r,
        None => return PreviewResult::failure("mesh", "No recipe defined"),
    };

    // Only handle mesh-related recipes
    let is_mesh_recipe =
        recipe.kind.starts_with("static_mesh.") || recipe.kind.starts_with("skeletal_mesh.");

    if !is_mesh_recipe {
        return PreviewResult::failure(
            "mesh",
            format!("Recipe kind '{}' is not a mesh recipe", recipe.kind),
        );
    }

    // Create a temporary directory for preview generation
    let tmp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            return PreviewResult::failure("mesh", format!("Failed to create temp dir: {}", e))
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
                            // Try to generate LOD proxy if enabled and mesh is large
                            if settings.use_lod_proxy {
                                let lod_config = LodConfig {
                                    target_triangles: settings.mesh_lod_target,
                                    ..Default::default()
                                };

                                match generate_lod_proxy(&glb_bytes, &lod_config) {
                                    Ok(lod_result) => {
                                        let metadata = extract_mesh_metadata_with_lod(
                                            &lod_result.glb_data,
                                            settings,
                                            lod_result.original_triangles,
                                            lod_result.proxy_triangles,
                                            lod_result.is_proxy,
                                        );

                                        let quality = if lod_result.is_proxy {
                                            PreviewQuality::Proxy
                                        } else {
                                            PreviewQuality::Full
                                        };

                                        PreviewResult::success_with_quality(
                                            "mesh",
                                            lod_result.glb_data,
                                            "model/gltf-binary",
                                            metadata,
                                            quality,
                                            lod_result.is_proxy, // can_refine if proxy
                                        )
                                    }
                                    Err(_) => {
                                        // LOD generation failed, use original
                                        let metadata = extract_mesh_metadata(&glb_bytes, settings);
                                        PreviewResult::success_with_metadata(
                                            "mesh",
                                            glb_bytes,
                                            "model/gltf-binary",
                                            metadata,
                                        )
                                    }
                                }
                            } else {
                                // LOD disabled, use original
                                let metadata = extract_mesh_metadata(&glb_bytes, settings);
                                PreviewResult::success_with_metadata(
                                    "mesh",
                                    glb_bytes,
                                    "model/gltf-binary",
                                    metadata,
                                )
                            }
                        }
                        Err(e) => {
                            PreviewResult::failure("mesh", format!("Failed to read GLB: {}", e))
                        }
                    }
                }
                None => PreviewResult::failure("mesh", "No GLB output generated"),
            }
        }
        Err(e) => PreviewResult::failure("mesh", format!("Generation failed: {}", e)),
    }
}

/// Generate full-quality mesh preview (for refinement requests).
///
/// This bypasses LOD proxy generation and returns the full-quality mesh.
pub fn generate_full_quality_mesh_preview(
    spec: &Spec,
    settings: &PreviewSettings,
) -> PreviewResult {
    // Use settings with LOD disabled
    let full_settings = PreviewSettings {
        use_lod_proxy: false,
        ..settings.clone()
    };

    generate_mesh_preview(spec, &full_settings)
}

/// Extract metadata from a GLB file with LOD information.
fn extract_mesh_metadata_with_lod(
    glb_bytes: &[u8],
    settings: &PreviewSettings,
    original_triangles: u32,
    proxy_triangles: u32,
    is_proxy: bool,
) -> serde_json::Value {
    // Try to parse the GLB to extract bounding box
    match gltf::Glb::from_slice(glb_bytes) {
        Ok(glb) => {
            match gltf::Gltf::from_slice(&glb.json) {
                Ok(gltf) => {
                    let mut min_bounds = [f32::MAX, f32::MAX, f32::MAX];
                    let mut max_bounds = [f32::MIN, f32::MIN, f32::MIN];

                    for mesh in gltf.meshes() {
                        for primitive in mesh.primitives() {
                            if let Some(accessor) = primitive.get(&gltf::Semantic::Positions) {
                                if let Some(bounds) = accessor
                                    .min()
                                    .and_then(|min| accessor.max().map(|max| (min, max)))
                                {
                                    let (min, max) = bounds;
                                    if let (
                                        serde_json::Value::Array(min_arr),
                                        serde_json::Value::Array(max_arr),
                                    ) = (min, max)
                                    {
                                        for (i, (min_val, max_val)) in
                                            min_arr.iter().zip(max_arr.iter()).enumerate()
                                        {
                                            if i < 3 {
                                                if let (Some(min_f), Some(max_f)) = (
                                                    min_val.as_f64().map(|v| v as f32),
                                                    max_val.as_f64().map(|v| v as f32),
                                                ) {
                                                    min_bounds[i] = min_bounds[i].min(min_f);
                                                    max_bounds[i] = max_bounds[i].max(max_f);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    serde_json::json!({
                        "vertices": proxy_triangles * 3, // Approximate
                        "triangles": proxy_triangles,
                        "original_triangles": original_triangles,
                        "is_proxy": is_proxy,
                        "max_triangles": settings.mesh_max_triangles,
                        "lod_threshold": settings.mesh_lod_threshold,
                        "bounds": {
                            "min": min_bounds,
                            "max": max_bounds,
                        }
                    })
                }
                Err(_) => serde_json::json!({
                    "triangles": proxy_triangles,
                    "original_triangles": original_triangles,
                    "is_proxy": is_proxy,
                    "max_triangles": settings.mesh_max_triangles,
                    "parse_error": "Failed to parse GLTF JSON"
                }),
            }
        }
        Err(_) => serde_json::json!({
            "triangles": proxy_triangles,
            "original_triangles": original_triangles,
            "is_proxy": is_proxy,
            "max_triangles": settings.mesh_max_triangles,
            "parse_error": "Failed to parse GLB"
        }),
    }
}

/// Extract metadata from a GLB file.
fn extract_mesh_metadata(glb_bytes: &[u8], settings: &PreviewSettings) -> serde_json::Value {
    // Try to parse the GLB to extract vertex/triangle counts
    match gltf::Glb::from_slice(glb_bytes) {
        Ok(glb) => {
            // Parse the JSON portion
            match gltf::Gltf::from_slice(&glb.json) {
                Ok(gltf) => {
                    let mut total_vertices = 0u64;
                    let mut total_triangles = 0u64;
                    let mut min_bounds = [f32::MAX, f32::MAX, f32::MAX];
                    let mut max_bounds = [f32::MIN, f32::MIN, f32::MIN];

                    for mesh in gltf.meshes() {
                        for primitive in mesh.primitives() {
                            // Count vertices
                            if let Some(accessor) = primitive.get(&gltf::Semantic::Positions) {
                                total_vertices += accessor.count() as u64;

                                // Get bounding box
                                if let Some(bounds) = accessor
                                    .min()
                                    .and_then(|min| accessor.max().map(|max| (min, max)))
                                {
                                    let (min, max) = bounds;
                                    if let (
                                        serde_json::Value::Array(min_arr),
                                        serde_json::Value::Array(max_arr),
                                    ) = (min, max)
                                    {
                                        for (i, (min_val, max_val)) in
                                            min_arr.iter().zip(max_arr.iter()).enumerate()
                                        {
                                            if i < 3 {
                                                if let (Some(min_f), Some(max_f)) = (
                                                    min_val.as_f64().map(|v| v as f32),
                                                    max_val.as_f64().map(|v| v as f32),
                                                ) {
                                                    min_bounds[i] = min_bounds[i].min(min_f);
                                                    max_bounds[i] = max_bounds[i].max(max_f);
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Count triangles (indices / 3 for triangles mode)
                            if let Some(accessor) = primitive.indices() {
                                total_triangles += accessor.count() as u64 / 3;
                            }
                        }
                    }

                    serde_json::json!({
                        "vertices": total_vertices,
                        "triangles": total_triangles,
                        "max_triangles": settings.mesh_max_triangles,
                        "bounds": {
                            "min": min_bounds,
                            "max": max_bounds,
                        }
                    })
                }
                Err(_) => serde_json::json!({
                    "max_triangles": settings.mesh_max_triangles,
                    "parse_error": "Failed to parse GLTF JSON"
                }),
            }
        }
        Err(_) => serde_json::json!({
            "max_triangles": settings.mesh_max_triangles,
            "parse_error": "Failed to parse GLB"
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe};

    #[test]
    fn test_mesh_preview_no_recipe() {
        let spec = Spec::builder("test-mesh", AssetType::StaticMesh)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Glb, "test.glb"))
            .build();

        let settings = PreviewSettings::default();
        let result = generate_mesh_preview(&spec, &settings);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("No recipe"));
    }

    #[test]
    fn test_mesh_preview_wrong_recipe_type() {
        let recipe = Recipe::new("audio_v1", serde_json::json!({}));
        let spec = Spec::builder("test-mesh", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(recipe)
            .build();

        let settings = PreviewSettings::default();
        let result = generate_mesh_preview(&spec, &settings);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("not a mesh recipe"));
    }
}
