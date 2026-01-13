//! Backend dispatch module
//!
//! Dispatches generation requests to the appropriate backend based on recipe.kind.

use speccade_spec::recipe::music::{MusicTrackerSongV1Params, TrackerFormat};
use speccade_spec::{BackendError, OutputFormat, OutputKind, OutputResult, Spec};
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
pub fn dispatch_generate(
    spec: &Spec,
    out_root: &str,
    spec_path: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    // Get the recipe kind
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let kind = &recipe.kind;

    // Create output directory if it doesn't exist
    let out_root_path = Path::new(out_root);
    fs::create_dir_all(out_root_path).map_err(|e| {
        DispatchError::BackendError(format!("Failed to create output directory: {}", e))
    })?;

    // Get spec directory for resolving relative paths
    let spec_dir = spec_path.parent().ok_or_else(|| {
        DispatchError::BackendError("Invalid spec path: no parent directory".to_string())
    })?;

    // Dispatch based on recipe kind prefix
    match kind.as_str() {
        // Unified audio backend (handles both SFX and instruments)
        "audio_v1" => generate_audio(spec, out_root_path),

        // Music backend
        "music.tracker_song_v1" => generate_music(spec, out_root_path, spec_dir),
        "music.tracker_song_compose_v1" => generate_music_compose(spec, out_root_path, spec_dir),

        // Unified procedural texture backend
        "texture.procedural_v1" => generate_texture_procedural(spec, out_root_path),

        // Blender static mesh backend
        "static_mesh.blender_primitives_v1" => generate_blender_static_mesh(spec, out_root_path),

        // Blender skeletal mesh backend
        "skeletal_mesh.blender_rigged_mesh_v1" => {
            generate_blender_skeletal_mesh(spec, out_root_path)
        }

        // Blender animation backend
        "skeletal_animation.blender_clip_v1" => generate_blender_animation(spec, out_root_path),

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

/// Generate audio using the unified audio backend
fn generate_audio(spec: &Spec, out_root: &Path) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_audio::generate(spec)
        .map_err(|e| DispatchError::BackendError(format!("Audio generation failed: {}", e)))?;

    // Write WAV file to the output path from spec
    let primary_output = get_primary_output(spec)?;
    if primary_output.format != OutputFormat::Wav {
        return Err(DispatchError::BackendError(format!(
            "audio_v1 requires primary output format 'wav', got '{}'",
            primary_output.format
        )));
    }
    write_output_bytes(out_root, &primary_output.path, &result.wav.wav_data)?;

    Ok(vec![OutputResult::tier1(
        OutputKind::Primary,
        OutputFormat::Wav,
        PathBuf::from(&primary_output.path),
        result.wav.pcm_hash,
    )])
}

/// Generate music using the music backend
fn generate_music(
    spec: &Spec,
    out_root: &Path,
    spec_dir: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_music_tracker_song()
        .map_err(|e| DispatchError::BackendError(format!("Invalid music params: {}", e)))?;

    generate_music_from_params(&params, &recipe.kind, spec, out_root, spec_dir)
}

fn generate_music_compose(
    spec: &Spec,
    out_root: &Path,
    spec_dir: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_music_tracker_song_compose().map_err(|e| {
        DispatchError::BackendError(format!("Invalid music compose params: {}", e))
    })?;
    let expanded = speccade_backend_music::expand_compose(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Compose expansion failed: {}", e))
    })?;

    generate_music_from_params(&expanded, &recipe.kind, spec, out_root, spec_dir)
}

fn generate_music_from_params(
    params: &MusicTrackerSongV1Params,
    recipe_kind: &str,
    spec: &Spec,
    out_root: &Path,
    spec_dir: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let primary_outputs: Vec<&speccade_spec::OutputSpec> = spec
        .outputs
        .iter()
        .filter(|o| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "No primary output specified".to_string(),
        ));
    }

    // Keep a defensive check even though validate_for_generate enforces this.
    let expected = match params.format {
        TrackerFormat::Xm => OutputFormat::Xm,
        TrackerFormat::It => OutputFormat::It,
    };

    // Single-output mode (legacy behavior).
    if primary_outputs.len() == 1 {
        let primary_output = primary_outputs[0];
        if primary_output.format != expected {
            return Err(DispatchError::BackendError(format!(
                "{} requires primary output format '{}', got '{}'",
                recipe_kind, expected, primary_output.format
            )));
        }

        let result = speccade_backend_music::generate_music(params, spec.seed, spec_dir)
            .map_err(|e| DispatchError::BackendError(format!("Music generation failed: {}", e)))?;

        write_output_bytes(out_root, &primary_output.path, &result.data)?;

        let mut outputs = vec![OutputResult::tier1(
            OutputKind::Primary,
            primary_output.format,
            PathBuf::from(&primary_output.path),
            result.hash,
        )];

        if let Some(loop_report) = result.loop_report.as_ref() {
            let loop_path = format!("{}.loops.json", primary_output.path);
            let bytes = serde_json::to_vec_pretty(loop_report).map_err(|e| {
                DispatchError::BackendError(format!(
                    "Failed to serialize music loop report JSON: {}",
                    e
                ))
            })?;
            write_output_bytes(out_root, &loop_path, &bytes)?;
            let hash = blake3::hash(&bytes).to_hex().to_string();
            outputs.push(OutputResult::tier1(
                OutputKind::Metadata,
                OutputFormat::Json,
                PathBuf::from(&loop_path),
                hash,
            ));
        }

        return Ok(outputs);
    }

    // Multi-output mode: one XM and/or one IT primary output.
    let mut seen_xm = false;
    let mut seen_it = false;
    let mut results = Vec::new();

    for output in primary_outputs {
        let format = match output.format {
            OutputFormat::Xm => {
                if seen_xm {
                    return Err(DispatchError::BackendError(format!(
                        "Duplicate primary output format 'xm' for {}",
                        recipe_kind
                    )));
                }
                seen_xm = true;
                TrackerFormat::Xm
            }
            OutputFormat::It => {
                if seen_it {
                    return Err(DispatchError::BackendError(format!(
                        "Duplicate primary output format 'it' for {}",
                        recipe_kind
                    )));
                }
                seen_it = true;
                TrackerFormat::It
            }
            _ => {
                return Err(DispatchError::BackendError(format!(
                    "{} primary outputs must have format 'xm' or 'it', got '{}'",
                    recipe_kind, output.format
                )))
            }
        };

        let mut per_output_params = params.clone();
        per_output_params.format = format;

        let gen = speccade_backend_music::generate_music(&per_output_params, spec.seed, spec_dir)
            .map_err(|e| {
            DispatchError::BackendError(format!("Music generation failed: {}", e))
        })?;

        // Defensive: ensure backend output matches requested format.
        let actual_format = match gen.extension {
            "xm" => OutputFormat::Xm,
            "it" => OutputFormat::It,
            _ => {
                return Err(DispatchError::BackendError(format!(
                    "Unknown music format: {}",
                    gen.extension
                )))
            }
        };
        if actual_format != output.format {
            return Err(DispatchError::BackendError(format!(
                "Music backend returned '{}' but output was declared as '{}'",
                actual_format, output.format
            )));
        }

        write_output_bytes(out_root, &output.path, &gen.data)?;
        results.push(OutputResult::tier1(
            OutputKind::Primary,
            output.format,
            PathBuf::from(&output.path),
            gen.hash,
        ));

        if let Some(loop_report) = gen.loop_report.as_ref() {
            let loop_path = format!("{}.loops.json", output.path);
            let bytes = serde_json::to_vec_pretty(loop_report).map_err(|e| {
                DispatchError::BackendError(format!(
                    "Failed to serialize music loop report JSON: {}",
                    e
                ))
            })?;
            write_output_bytes(out_root, &loop_path, &bytes)?;
            let hash = blake3::hash(&bytes).to_hex().to_string();
            results.push(OutputResult::tier1(
                OutputKind::Metadata,
                OutputFormat::Json,
                PathBuf::from(&loop_path),
                hash,
            ));
        }
    }

    Ok(results)
}

/// Generate procedural texture outputs using the texture backend.
fn generate_texture_procedural(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_procedural().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture procedural params: {}", e))
    })?;

    let nodes = speccade_backend_texture::generate_graph(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Procedural texture generation failed: {}", e))
    })?;

    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.procedural_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    let mut outputs = Vec::with_capacity(primary_outputs.len());

    for (output_index, output_spec) in primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        let source = output_spec.source.as_ref().ok_or_else(|| {
            DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must set 'source' (outputs[{}].source)",
                output_index
            ))
        })?;

        let value = nodes.get(source).ok_or_else(|| {
            DispatchError::BackendError(format!(
                "outputs[{}].source '{}' does not match any node id",
                output_index, source
            ))
        })?;

        let (png_data, hash) =
            speccade_backend_texture::encode_graph_value_png(value).map_err(|e| {
                DispatchError::BackendError(format!("PNG encoding failed: {}", e))
            })?;

        write_output_bytes(out_root, &output_spec.path, &png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            hash,
        ));
    }

    Ok(outputs)
}

/// Generate static mesh using the Blender backend
fn generate_blender_static_mesh(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_blender::static_mesh::generate(spec, out_root).map_err(|e| {
        DispatchError::BackendError(format!("Static mesh generation failed: {}", e))
    })?;

    // Get primary output path
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;
    if primary_output.format != OutputFormat::Glb {
        return Err(DispatchError::BackendError(format!(
            "static_mesh.blender_primitives_v1 requires primary output format 'glb', got '{}'",
            primary_output.format
        )));
    }

    // Convert metrics to OutputMetrics
    let metrics =
        speccade_spec::OutputMetrics {
            triangle_count: result.metrics.triangle_count,
            bounding_box: result.metrics.bounding_box.as_ref().map(|bb| {
                speccade_spec::BoundingBox {
                    min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
                    max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
                }
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
fn generate_blender_skeletal_mesh(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let result =
        speccade_backend_blender::skeletal_mesh::generate(spec, out_root).map_err(|e| {
            DispatchError::BackendError(format!("Skeletal mesh generation failed: {}", e))
        })?;

    // Get primary output path
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;
    if primary_output.format != OutputFormat::Glb {
        return Err(DispatchError::BackendError(format!(
            "skeletal_mesh.blender_rigged_mesh_v1 requires primary output format 'glb', got '{}'",
            primary_output.format
        )));
    }

    // Convert metrics to OutputMetrics
    let metrics =
        speccade_spec::OutputMetrics {
            triangle_count: result.metrics.triangle_count,
            bounding_box: result.metrics.bounding_box.as_ref().map(|bb| {
                speccade_spec::BoundingBox {
                    min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
                    max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
                }
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
fn generate_blender_animation(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_blender::animation::generate(spec, out_root)
        .map_err(|e| DispatchError::BackendError(format!("Animation generation failed: {}", e)))?;

    // Get primary output path
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;
    if primary_output.format != OutputFormat::Glb {
        return Err(DispatchError::BackendError(format!(
            "skeletal_animation.blender_clip_v1 requires primary output format 'glb', got '{}'",
            primary_output.format
        )));
    }

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
        "audio_v1"
            | "music.tracker_song_v1"
            | "music.tracker_song_compose_v1"
            | "texture.procedural_v1"
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
        "audio_v1" => Some(1),
        k if k.starts_with("music.") => Some(1),
        k if k.starts_with("texture.") => Some(1),

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
    use speccade_spec::{AssetType, OutputSpec, Recipe};

    #[test]
    fn test_backend_tier_classification() {
        // Tier 1 - Rust backends
        assert_eq!(get_backend_tier("audio_v1"), Some(1));
        assert_eq!(get_backend_tier("music.tracker_song_v1"), Some(1));
        assert_eq!(get_backend_tier("music.tracker_song_compose_v1"), Some(1));
        assert_eq!(get_backend_tier("texture.procedural_v1"), Some(1));

        // Tier 2 - Blender backends
        assert_eq!(
            get_backend_tier("static_mesh.blender_primitives_v1"),
            Some(2)
        );
        assert_eq!(
            get_backend_tier("skeletal_mesh.blender_rigged_mesh_v1"),
            Some(2)
        );
        assert_eq!(
            get_backend_tier("skeletal_animation.blender_clip_v1"),
            Some(2)
        );

        // Unknown
        assert_eq!(get_backend_tier("unknown.kind"), None);
    }

    #[test]
    fn test_backends_available() {
        // All implemented backends should be available
        assert!(is_backend_available("audio_v1"));
        assert!(is_backend_available("music.tracker_song_v1"));
        assert!(is_backend_available("music.tracker_song_compose_v1"));
        assert!(is_backend_available("texture.procedural_v1"));
        assert!(is_backend_available("static_mesh.blender_primitives_v1"));
        assert!(is_backend_available("skeletal_mesh.blender_rigged_mesh_v1"));
        assert!(is_backend_available("skeletal_animation.blender_clip_v1"));

        // Unknown backends should not be available
        assert!(!is_backend_available("unknown.kind"));
    }

    #[test]
    fn test_dispatch_texture_procedural_generates_outputs() {
        let tmp = tempfile::tempdir().unwrap();

        let mut output = OutputSpec::primary(OutputFormat::Png, "textures/mask.png");
        output.source = Some("mask".to_string());

        let recipe = Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({
                "resolution": [16, 16],
                "tileable": true,
                "nodes": [
                    { "id": "n", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.08 } },
                    { "id": "mask", "type": "threshold", "input": "n", "threshold": 0.5 }
                ]
            }),
        );

        let spec = Spec::builder("test-procedural-01", AssetType::Texture)
            .license("CC0-1.0")
            .seed(42)
            .output(output)
            .recipe(recipe)
            .build();

        let spec_path = tmp.path().join("test.spec.json");
        let outputs = dispatch_generate(&spec, tmp.path().to_str().unwrap(), &spec_path).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].format, OutputFormat::Png);

        let output_path = tmp.path().join("textures/mask.png");
        assert!(output_path.exists());
        let bytes = std::fs::read(&output_path).unwrap();
        assert!(!bytes.is_empty());
    }
}
