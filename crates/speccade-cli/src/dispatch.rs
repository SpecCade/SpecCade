//! Backend dispatch module
//!
//! Dispatches generation requests to the appropriate backend based on recipe.kind.

use speccade_spec::{OutputFormat, OutputKind, OutputResult, Spec};
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

/// Dispatch generation to the appropriate backend
///
/// # Arguments
/// * `spec` - The validated spec to generate from
/// * `out_root` - The output root directory
///
/// # Returns
/// A vector of output results on success, or a dispatch error
pub fn dispatch_generate(spec: &Spec, out_root: &str) -> Result<Vec<OutputResult>, DispatchError> {
    // Get the recipe kind
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let kind = &recipe.kind;

    // Create output directory if it doesn't exist
    let out_root_path = Path::new(out_root);
    fs::create_dir_all(out_root_path)
        .map_err(|e| DispatchError::BackendError(format!("Failed to create output directory: {}", e)))?;

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
            generate_music(spec, out_root_path)
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

/// Generate audio SFX using the audio backend
fn generate_audio_sfx(spec: &Spec, out_root: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_audio::generate(spec)
        .map_err(|e| DispatchError::BackendError(format!("Audio generation failed: {}", e)))?;

    // Write WAV file to the output path from spec
    let primary_output = spec.outputs.iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;

    let output_path = out_root.join(&primary_output.path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| DispatchError::BackendError(format!("Failed to create output directory: {}", e)))?;
    }

    fs::write(&output_path, &result.wav.wav_data)
        .map_err(|e| DispatchError::BackendError(format!("Failed to write WAV file: {}", e)))?;

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
    let primary_output = spec.outputs.iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;

    let output_path = out_root.join(&primary_output.path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| DispatchError::BackendError(format!("Failed to create output directory: {}", e)))?;
    }

    fs::write(&output_path, &result.wav.wav_data)
        .map_err(|e| DispatchError::BackendError(format!("Failed to write WAV file: {}", e)))?;

    Ok(vec![OutputResult::tier1(
        OutputKind::Primary,
        OutputFormat::Wav,
        PathBuf::from(&primary_output.path),
        result.wav.pcm_hash,
    )])
}

/// Generate music using the music backend
fn generate_music(spec: &Spec, out_root: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_music_tracker_song()
        .map_err(|e| DispatchError::BackendError(format!("Invalid music params: {}", e)))?;

    let result = speccade_backend_music::generate_music(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Music generation failed: {}", e)))?;

    // Write tracker module file to the output path from spec
    let primary_output = spec.outputs.iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;

    let output_path = out_root.join(&primary_output.path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| DispatchError::BackendError(format!("Failed to create output directory: {}", e)))?;
    }

    fs::write(&output_path, &result.data)
        .map_err(|e| DispatchError::BackendError(format!("Failed to write music file: {}", e)))?;

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

    let mut outputs = Vec::new();

    // Write each generated map
    for (_map_type, map_result) in result.maps {
        // Find the corresponding output spec
        let output_spec = spec.outputs.iter()
            .find(|o| {
                // Match by format - each map type should have a corresponding output
                o.format == OutputFormat::Png
            });

        if let Some(output) = output_spec {
            let output_path = out_root.join(&output.path);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| DispatchError::BackendError(format!("Failed to create output directory: {}", e)))?;
            }

            fs::write(&output_path, &map_result.data)
                .map_err(|e| DispatchError::BackendError(format!("Failed to write texture file: {}", e)))?;

            outputs.push(OutputResult::tier1(
                output.kind.clone(),
                OutputFormat::Png,
                PathBuf::from(&output.path),
                map_result.hash,
            ));
        }
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
    let primary_output = spec.outputs.iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;

    let output_path = out_root.join(&primary_output.path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| DispatchError::BackendError(format!("Failed to create output directory: {}", e)))?;
    }

    fs::write(&output_path, &result.data)
        .map_err(|e| DispatchError::BackendError(format!("Failed to write normal map file: {}", e)))?;

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
}
