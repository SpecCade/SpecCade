//! Backend dispatch module
//!
//! Dispatches generation requests to the appropriate backend based on recipe.kind.

use speccade_spec::recipe::texture::TextureMapType;
use speccade_spec::{BackendError, OutputFormat, OutputKind, OutputResult, Spec};
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Errors that can occur during backend dispatch
#[derive(Debug)]
pub enum DispatchError {
    /// The spec has no recipe
    NoRecipe,
    /// The backend for this recipe kind is not yet implemented
    BackendNotImplemented(String),
    /// The backend execution failed
    BackendError(String),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DispatchError::NoRecipe => write!(f, "Spec has no recipe defined"),
            DispatchError::BackendNotImplemented(kind) => {
                write!(f, "Backend not implemented for recipe kind: {}", kind)
            }
            DispatchError::BackendError(msg) => {
                write!(f, "Backend error: {}", msg)
            }
        }
    }
}

impl std::error::Error for DispatchError {}

impl BackendError for DispatchError {
    fn code(&self) -> &'static str {
        match self {
            DispatchError::NoRecipe => "DISPATCH_001",
            DispatchError::BackendNotImplemented(_) => "DISPATCH_002",
            DispatchError::BackendError(_) => "DISPATCH_003",
        }
    }

    fn category(&self) -> &'static str {
        "dispatch"
    }
}

/// Dispatch generation to the appropriate backend
///
/// # Arguments
/// * `spec` - The validated spec to generate from
/// * `out_root` - The output root directory
/// * `spec_path` - Path to the spec file (for resolving relative paths)
///
/// # Returns
/// A vector of output results on success, or a dispatch error
pub fn dispatch_generate(spec: &Spec, out_root: &str, spec_path: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    // Get the recipe kind
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let kind = &recipe.kind;

    // Create output directory if it doesn't exist
    let out_root_path = Path::new(out_root);
    fs::create_dir_all(out_root_path)
        .map_err(|e| DispatchError::BackendError(format!("Failed to create output directory: {}", e)))?;

    // Get spec directory for resolving relative paths
    let spec_dir = spec_path.parent().ok_or_else(|| {
        DispatchError::BackendError("Invalid spec path: no parent directory".to_string())
    })?;

    // Dispatch based on recipe kind prefix
    match kind.as_str() {
        // Audio SFX backend
        "audio_sfx.layered_synth_v1" => {
            generate_audio_sfx(spec, out_root_path)
        }

        // Audio instrument backend
        "audio_instrument.synth_patch_v1" => {
            generate_audio_instrument(spec, out_root_path)
        }

        // Music backend
        "music.tracker_song_v1" => {
            generate_music(spec, out_root_path, spec_dir)
        }

        // Texture material maps backend
        "texture_2d.material_maps_v1" => {
            generate_texture_material_maps(spec, out_root_path)
        }

        // Texture normal map backend
        "texture_2d.normal_map_v1" => {
            generate_texture_normal_map(spec, out_root_path)
        }

        // Blender static mesh backend
        "static_mesh.blender_primitives_v1" => {
            generate_blender_static_mesh(spec, out_root_path)
        }

        // Blender skeletal mesh backend
        "skeletal_mesh.blender_rigged_mesh_v1" => {
            generate_blender_skeletal_mesh(spec, out_root_path)
        }

        // Blender animation backend
        "skeletal_animation.blender_clip_v1" => {
            generate_blender_animation(spec, out_root_path)
        }

        // Unknown recipe kind
        _ => Err(DispatchError::BackendNotImplemented(kind.clone())),
    }
}

fn get_primary_output(spec: &Spec) -> Result<&speccade_spec::OutputSpec, DispatchError> {
    spec.outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))
}

fn write_output_bytes(out_root: &Path, rel_path: &str, bytes: &[u8]) -> Result<(), DispatchError> {
    let output_path = out_root.join(rel_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            DispatchError::BackendError(format!("Failed to create output directory: {}", e))
        })?;
    }

    fs::write(&output_path, bytes)
        .map_err(|e| DispatchError::BackendError(format!("Failed to write output file: {}", e)))?;
    Ok(())
}

/// Generate audio SFX using the audio backend
fn generate_audio_sfx(spec: &Spec, out_root: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_audio::generate(spec)
        .map_err(|e| DispatchError::BackendError(format!("Audio generation failed: {}", e)))?;

    // Write WAV file to the output path from spec
    let primary_output = get_primary_output(spec)?;
    write_output_bytes(out_root, &primary_output.path, &result.wav.wav_data)?;

    Ok(vec![OutputResult::tier1(
        OutputKind::Primary,
        OutputFormat::Wav,
        PathBuf::from(&primary_output.path),
        result.wav.pcm_hash,
    )])
}

/// Generate audio instrument using the audio backend
fn generate_audio_instrument(spec: &Spec, out_root: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_audio_instrument_synth_patch()
        .map_err(|e| DispatchError::BackendError(format!("Invalid instrument params: {}", e)))?;

    let result = speccade_backend_audio::generate_instrument(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Instrument generation failed: {}", e)))?;

    // Write WAV file to the output path from spec
    let primary_output = get_primary_output(spec)?;
    write_output_bytes(out_root, &primary_output.path, &result.wav.wav_data)?;

    Ok(vec![OutputResult::tier1(
        OutputKind::Primary,
        OutputFormat::Wav,
        PathBuf::from(&primary_output.path),
        result.wav.pcm_hash,
    )])
}

/// Generate music using the music backend
fn generate_music(spec: &Spec, out_root: &Path, spec_dir: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_music_tracker_song()
        .map_err(|e| DispatchError::BackendError(format!("Invalid music params: {}", e)))?;

    let result = speccade_backend_music::generate_music(&params, spec.seed, spec_dir)
        .map_err(|e| DispatchError::BackendError(format!("Music generation failed: {}", e)))?;

    // Write tracker module file to the output path from spec
    let primary_output = get_primary_output(spec)?;
    write_output_bytes(out_root, &primary_output.path, &result.data)?;

    // Determine format based on extension
    let format = match result.extension {
        "xm" => OutputFormat::Xm,
        "it" => OutputFormat::It,
        _ => return Err(DispatchError::BackendError(format!("Unknown music format: {}", result.extension))),
    };

    Ok(vec![OutputResult::tier1(
        OutputKind::Primary,
        format,
        PathBuf::from(&primary_output.path),
        result.hash,
    )])
}

/// Generate texture material maps using the texture backend
fn generate_texture_material_maps(spec: &Spec, out_root: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_2d_material_maps()
        .map_err(|e| DispatchError::BackendError(format!("Invalid texture params: {}", e)))?;

    let result = speccade_backend_texture::generate_material_maps(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Texture generation failed: {}", e)))?;

    // Collect PNG outputs once and map them deterministically to the requested map types.
    //
    // Prefer explicit filename suffixes (e.g. `_albedo.png`) to avoid silently overwriting
    // multiple maps to the same output path.
    let mut unused_png_output_indices: Vec<usize> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.format == OutputFormat::Png)
        .map(|(i, _)| i)
        .collect();

    if unused_png_output_indices.len() < params.maps.len() {
        return Err(DispatchError::BackendError(format!(
            "Not enough PNG outputs for material maps: {} requested, but only {} PNG outputs declared",
            params.maps.len(),
            unused_png_output_indices.len()
        )));
    }

    let mut used_paths: HashSet<&str> = HashSet::new();
    let mut outputs = Vec::with_capacity(params.maps.len());

    for (map_index, map_type) in params.maps.iter().enumerate() {
        let map_result = result.maps.get(map_type).ok_or_else(|| {
            DispatchError::BackendError(format!("Missing generated map for {:?}", map_type))
        })?;

        let suffix = texture_map_suffix(*map_type);
        let mut matching_indices: Vec<usize> = unused_png_output_indices
            .iter()
            .copied()
            .filter(|&i| output_path_matches_suffix(&spec.outputs[i].path, suffix))
            .collect();

        let chosen_index = match matching_indices.len() {
            0 => {
                // Fallback: if the caller provided exactly one PNG output per requested map,
                // map them in order.
                let remaining_maps = params.maps.len() - map_index;
                if unused_png_output_indices.len() == remaining_maps {
                    unused_png_output_indices[0]
                } else {
                    return Err(DispatchError::BackendError(format!(
                        "No PNG output path found for map {:?}. Expected a path ending with '_{}.png' (or provide exactly one PNG output per map in the same order as 'recipe.params.maps')",
                        map_type, suffix
                    )));
                }
            }
            1 => matching_indices.pop().unwrap(),
            _ => {
                matching_indices.sort();
                let matching_paths: Vec<String> = matching_indices
                    .iter()
                    .map(|&i| spec.outputs[i].path.clone())
                    .collect();
                return Err(DispatchError::BackendError(format!(
                    "Multiple PNG outputs match map {:?} (suffix '{}'): {}",
                    map_type,
                    suffix,
                    matching_paths.join(", ")
                )));
            }
        };

        unused_png_output_indices.retain(|&i| i != chosen_index);

        let output_spec = &spec.outputs[chosen_index];
        if !used_paths.insert(output_spec.path.as_str()) {
            return Err(DispatchError::BackendError(format!(
                "Output path matched more than once while mapping texture maps: {}",
                output_spec.path
            )));
        }

        write_output_bytes(out_root, &output_spec.path, &map_result.data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            map_result.hash.clone(),
        ));
    }

    Ok(outputs)
}

/// Generate texture normal map using the texture backend
fn generate_texture_normal_map(spec: &Spec, out_root: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_2d_normal_map()
        .map_err(|e| DispatchError::BackendError(format!("Invalid normal map params: {}", e)))?;

    let result = speccade_backend_texture::generate_normal_map(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Normal map generation failed: {}", e)))?;

    // Write normal map file to the output path from spec
    let primary_output = get_primary_output(spec)?;
    write_output_bytes(out_root, &primary_output.path, &result.data)?;

    Ok(vec![OutputResult::tier1(
        OutputKind::Primary,
        OutputFormat::Png,
        PathBuf::from(&primary_output.path),
        result.hash,
    )])
}

/// Generate static mesh using the Blender backend
fn generate_blender_static_mesh(spec: &Spec, out_root: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_blender::static_mesh::generate(spec, out_root)
        .map_err(|e| DispatchError::BackendError(format!("Static mesh generation failed: {}", e)))?;

    // Get primary output path
    let primary_output = spec.outputs.iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;

    // Convert metrics to OutputMetrics
    let metrics = speccade_spec::OutputMetrics {
        triangle_count: result.metrics.triangle_count,
        bounding_box: result.metrics.bounding_box.as_ref().map(|bb| speccade_spec::BoundingBox {
            min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
            max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
        }),
        uv_island_count: result.metrics.uv_island_count,
        bone_count: None,
        material_slot_count: result.metrics.material_slot_count,
        max_bone_influences: None,
        animation_frame_count: None,
        animation_duration_seconds: None,
    };

    Ok(vec![OutputResult::tier2(
        OutputKind::Primary,
        OutputFormat::Glb,
        PathBuf::from(&primary_output.path),
        metrics,
    )])
}

/// Generate skeletal mesh using the Blender backend
fn generate_blender_skeletal_mesh(spec: &Spec, out_root: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_blender::skeletal_mesh::generate(spec, out_root)
        .map_err(|e| DispatchError::BackendError(format!("Skeletal mesh generation failed: {}", e)))?;

    // Get primary output path
    let primary_output = spec.outputs.iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;

    // Convert metrics to OutputMetrics
    let metrics = speccade_spec::OutputMetrics {
        triangle_count: result.metrics.triangle_count,
        bounding_box: result.metrics.bounding_box.as_ref().map(|bb| speccade_spec::BoundingBox {
            min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
            max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
        }),
        uv_island_count: result.metrics.uv_island_count,
        bone_count: result.metrics.bone_count,
        material_slot_count: result.metrics.material_slot_count,
        max_bone_influences: result.metrics.max_bone_influences,
        animation_frame_count: None,
        animation_duration_seconds: None,
    };

    Ok(vec![OutputResult::tier2(
        OutputKind::Primary,
        OutputFormat::Glb,
        PathBuf::from(&primary_output.path),
        metrics,
    )])
}

/// Generate animation using the Blender backend
fn generate_blender_animation(spec: &Spec, out_root: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_blender::animation::generate(spec, out_root)
        .map_err(|e| DispatchError::BackendError(format!("Animation generation failed: {}", e)))?;

    // Get primary output path
    let primary_output = spec.outputs.iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;

    // Convert metrics to OutputMetrics
    let metrics = speccade_spec::OutputMetrics {
        triangle_count: None,
        bounding_box: None,
        uv_island_count: None,
        bone_count: result.metrics.bone_count,
        material_slot_count: None,
        max_bone_influences: None,
        animation_frame_count: result.metrics.animation_frame_count,
        animation_duration_seconds: result.metrics.animation_duration_seconds.map(|d| d as f32),
    };

    Ok(vec![OutputResult::tier2(
        OutputKind::Primary,
        OutputFormat::Glb,
        PathBuf::from(&primary_output.path),
        metrics,
    )])
}

/// Check if a backend is available for a given recipe kind
///
/// # Arguments
/// * `kind` - The recipe kind to check
///
/// # Returns
/// True if the backend is implemented and available
pub fn is_backend_available(kind: &str) -> bool {
    matches!(
        kind,
        "audio_sfx.layered_synth_v1"
            | "audio_instrument.synth_patch_v1"
            | "music.tracker_song_v1"
            | "texture_2d.material_maps_v1"
            | "texture_2d.normal_map_v1"
            | "static_mesh.blender_primitives_v1"
            | "skeletal_mesh.blender_rigged_mesh_v1"
            | "skeletal_animation.blender_clip_v1"
    )
}

fn texture_map_suffix(map_type: TextureMapType) -> &'static str {
    match map_type {
        TextureMapType::Albedo => "albedo",
        TextureMapType::Normal => "normal",
        TextureMapType::Roughness => "roughness",
        TextureMapType::Metallic => "metallic",
        TextureMapType::Ao => "ao",
        TextureMapType::Emissive => "emissive",
        TextureMapType::Height => "height",
    }
}

fn output_path_matches_suffix(path: &str, suffix: &str) -> bool {
    let path_lower = path.to_ascii_lowercase();
    let stem = match path_lower.strip_suffix(".png") {
        Some(stem) => stem,
        None => path_lower.as_str(),
    };

    stem.ends_with(&format!("_{}", suffix))
        || stem.ends_with(&format!("-{}", suffix))
        || stem.ends_with(&format!("/{}", suffix))
}

/// Get the backend tier for a recipe kind
///
/// # Arguments
/// * `kind` - The recipe kind
///
/// # Returns
/// The backend tier (1 = deterministic, 2 = metric validation)
pub fn get_backend_tier(kind: &str) -> Option<u8> {
    match kind {
        // Tier 1: Rust backends (deterministic hash guarantee)
        k if k.starts_with("audio_sfx.") => Some(1),
        k if k.starts_with("audio_instrument.") => Some(1),
        k if k.starts_with("music.") => Some(1),
        k if k.starts_with("texture_2d.") => Some(1),

        // Tier 2: Blender backends (metric validation only)
        k if k.starts_with("static_mesh.") => Some(2),
        k if k.starts_with("skeletal_mesh.") => Some(2),
        k if k.starts_with("skeletal_animation.") => Some(2),

        // Unknown
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::texture::Texture2dMaterialMapsV1Params;
    use speccade_spec::{AssetType, OutputSpec, Recipe};

    #[test]
    fn test_backend_tier_classification() {
        // Tier 1 - Rust backends
        assert_eq!(get_backend_tier("audio_sfx.layered_synth_v1"), Some(1));
        assert_eq!(get_backend_tier("music.tracker_song_v1"), Some(1));
        assert_eq!(get_backend_tier("texture_2d.material_maps_v1"), Some(1));

        // Tier 2 - Blender backends
        assert_eq!(get_backend_tier("static_mesh.blender_primitives_v1"), Some(2));
        assert_eq!(get_backend_tier("skeletal_mesh.blender_rigged_mesh_v1"), Some(2));
        assert_eq!(get_backend_tier("skeletal_animation.blender_clip_v1"), Some(2));

        // Unknown
        assert_eq!(get_backend_tier("unknown.kind"), None);
    }

    #[test]
    fn test_backends_available() {
        // All implemented backends should be available
        assert!(is_backend_available("audio_sfx.layered_synth_v1"));
        assert!(is_backend_available("audio_instrument.synth_patch_v1"));
        assert!(is_backend_available("music.tracker_song_v1"));
        assert!(is_backend_available("texture_2d.material_maps_v1"));
        assert!(is_backend_available("texture_2d.normal_map_v1"));
        assert!(is_backend_available("static_mesh.blender_primitives_v1"));
        assert!(is_backend_available("skeletal_mesh.blender_rigged_mesh_v1"));
        assert!(is_backend_available("skeletal_animation.blender_clip_v1"));

        // Unknown backends should not be available
        assert!(!is_backend_available("unknown.kind"));
    }

    #[test]
    fn test_dispatch_texture_material_maps_matches_by_suffix() {
        let tmp = tempfile::tempdir().unwrap();

        let params = Texture2dMaterialMapsV1Params {
            resolution: [16, 16],
            tileable: true,
            maps: vec![TextureMapType::Albedo, TextureMapType::Normal],
            base_material: None,
            layers: vec![],
            color_ramp: None,
            palette: None,
        };

        let recipe = Recipe::new(
            "texture_2d.material_maps_v1",
            serde_json::to_value(&params).unwrap(),
        );

        let spec = Spec::builder("test-tex-01", AssetType::Texture2d)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(
                OutputFormat::Png,
                "textures/test-tex-01_albedo.png",
            ))
            .output(OutputSpec::primary(
                OutputFormat::Png,
                "textures/test-tex-01_normal.png",
            ))
            .recipe(recipe)
            .build();

        let spec_path = tmp.path().join("test.spec.json");
        let outputs = dispatch_generate(&spec, tmp.path().to_str().unwrap(), &spec_path).unwrap();
        assert_eq!(outputs.len(), 2);

        let albedo_path = tmp.path().join("textures/test-tex-01_albedo.png");
        let normal_path = tmp.path().join("textures/test-tex-01_normal.png");

        assert!(albedo_path.exists());
        assert!(normal_path.exists());

        let albedo_bytes = std::fs::read(&albedo_path).unwrap();
        let normal_bytes = std::fs::read(&normal_path).unwrap();
        assert!(!albedo_bytes.is_empty());
        assert!(!normal_bytes.is_empty());
        assert_ne!(albedo_bytes, normal_bytes, "maps should not overwrite each other");
    }

    #[test]
    fn test_dispatch_texture_material_maps_fallbacks_to_order() {
        let tmp = tempfile::tempdir().unwrap();

        let params = Texture2dMaterialMapsV1Params {
            resolution: [8, 8],
            tileable: true,
            maps: vec![TextureMapType::Roughness, TextureMapType::Metallic],
            base_material: None,
            layers: vec![],
            color_ramp: None,
            palette: None,
        };

        let recipe = Recipe::new(
            "texture_2d.material_maps_v1",
            serde_json::to_value(&params).unwrap(),
        );

        let spec = Spec::builder("test-tex-02", AssetType::Texture2d)
            .license("CC0-1.0")
            .seed(123)
            .output(OutputSpec::primary(OutputFormat::Png, "textures/out1.png"))
            .output(OutputSpec::primary(OutputFormat::Png, "textures/out2.png"))
            .recipe(recipe)
            .build();

        let spec_path = tmp.path().join("test.spec.json");
        let outputs = dispatch_generate(&spec, tmp.path().to_str().unwrap(), &spec_path).unwrap();
        assert_eq!(outputs.len(), 2);
        assert!(tmp.path().join("textures/out1.png").exists());
        assert!(tmp.path().join("textures/out2.png").exists());
    }

    #[test]
    fn test_dispatch_texture_material_maps_requires_enough_outputs() {
        let tmp = tempfile::tempdir().unwrap();

        let params = Texture2dMaterialMapsV1Params {
            resolution: [8, 8],
            tileable: true,
            maps: vec![TextureMapType::Albedo, TextureMapType::Normal],
            base_material: None,
            layers: vec![],
            color_ramp: None,
            palette: None,
        };

        let recipe = Recipe::new(
            "texture_2d.material_maps_v1",
            serde_json::to_value(&params).unwrap(),
        );

        let spec = Spec::builder("test-tex-03", AssetType::Texture2d)
            .license("CC0-1.0")
            .seed(1)
            .output(OutputSpec::primary(OutputFormat::Png, "textures/only_one.png"))
            .recipe(recipe)
            .build();

        let spec_path = tmp.path().join("test.spec.json");
        let err = dispatch_generate(&spec, tmp.path().to_str().unwrap(), &spec_path).unwrap_err();
        assert!(err.to_string().contains("Not enough PNG outputs"));
    }
}
