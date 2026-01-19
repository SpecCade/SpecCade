//! Backend dispatch module
//!
//! Dispatches generation requests to the appropriate backend based on recipe.kind.

mod audio;
mod blender;
mod music;
mod texture;
mod waveform;

use speccade_spec::{BackendError, OutputKind, OutputResult, Spec, StageTiming};
use std::fmt;
use std::fs;
use std::path::Path;

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
/// * `preview_duration` - Optional preview duration in seconds (truncates audio generation)
///
/// # Returns
/// A vector of output results on success, or a dispatch error
pub fn dispatch_generate(
    spec: &Spec,
    out_root: &str,
    spec_path: &Path,
    preview_duration: Option<f64>,
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
        "audio_v1" => audio::generate_audio(spec, out_root_path, preview_duration),

        // Music backend
        "music.tracker_song_v1" => music::generate_music(spec, out_root_path, spec_dir),
        "music.tracker_song_compose_v1" => {
            music::generate_music_compose(spec, out_root_path, spec_dir)
        }

        // Unified procedural texture backend
        "texture.procedural_v1" => texture::generate_texture_procedural(spec, out_root_path),

        // Blender static mesh backend
        "static_mesh.blender_primitives_v1" => {
            blender::generate_blender_static_mesh(spec, out_root_path)
        }

        // Blender skeletal mesh backend
        "skeletal_mesh.blender_rigged_mesh_v1" => {
            blender::generate_blender_skeletal_mesh(spec, out_root_path)
        }

        // Blender animation backend (simple keyframes)
        "skeletal_animation.blender_clip_v1" => {
            blender::generate_blender_animation(spec, out_root_path)
        }

        // Blender rigged animation backend (IK/rig-aware)
        "skeletal_animation.blender_rigged_v1" => {
            blender::generate_blender_rigged_animation(spec, out_root_path)
        }

        // Unknown recipe kind
        _ => Err(DispatchError::BackendNotImplemented(kind.clone())),
    }
}

/// Result of a dispatch with optional profiling timings.
pub struct DispatchResult {
    /// The generated output artifacts.
    pub outputs: Vec<OutputResult>,
    /// Per-stage timing breakdown (only present when profiling is enabled).
    pub stages: Option<Vec<StageTiming>>,
}

impl DispatchResult {
    /// Creates a dispatch result without profiling data.
    pub fn new(outputs: Vec<OutputResult>) -> Self {
        Self {
            outputs,
            stages: None,
        }
    }

    /// Creates a dispatch result with profiling data.
    pub fn with_stages(outputs: Vec<OutputResult>, stages: Vec<StageTiming>) -> Self {
        Self {
            outputs,
            stages: if stages.is_empty() {
                None
            } else {
                Some(stages)
            },
        }
    }
}

/// Dispatch generation with optional profiling support.
///
/// When `profile` is true, per-stage timing information is collected and returned.
pub fn dispatch_generate_profiled(
    spec: &Spec,
    out_root: &str,
    spec_path: &Path,
    preview_duration: Option<f64>,
    profile: bool,
) -> Result<DispatchResult, DispatchError> {
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

    // Dispatch based on recipe kind prefix with optional timing instrumentation
    match kind.as_str() {
        "audio_v1" => {
            if profile {
                audio::generate_audio_profiled(spec, out_root_path, preview_duration)
            } else {
                audio::generate_audio(spec, out_root_path, preview_duration)
                    .map(DispatchResult::new)
            }
        }

        "music.tracker_song_v1" => {
            if profile {
                music::generate_music_profiled(spec, out_root_path, spec_dir)
            } else {
                music::generate_music(spec, out_root_path, spec_dir).map(DispatchResult::new)
            }
        }

        "music.tracker_song_compose_v1" => {
            if profile {
                music::generate_music_compose_profiled(spec, out_root_path, spec_dir)
            } else {
                music::generate_music_compose(spec, out_root_path, spec_dir)
                    .map(DispatchResult::new)
            }
        }

        "texture.procedural_v1" => {
            if profile {
                texture::generate_texture_procedural_profiled(spec, out_root_path)
            } else {
                texture::generate_texture_procedural(spec, out_root_path).map(DispatchResult::new)
            }
        }

        // Blender backends (no profiling instrumentation yet)
        "static_mesh.blender_primitives_v1" => {
            blender::generate_blender_static_mesh(spec, out_root_path).map(DispatchResult::new)
        }

        "skeletal_mesh.blender_rigged_mesh_v1" => {
            blender::generate_blender_skeletal_mesh(spec, out_root_path).map(DispatchResult::new)
        }

        "skeletal_animation.blender_clip_v1" => {
            blender::generate_blender_animation(spec, out_root_path).map(DispatchResult::new)
        }

        // Blender rigged animation backend (no profiling instrumentation yet)
        "skeletal_animation.blender_rigged_v1" => {
            blender::generate_blender_rigged_animation(spec, out_root_path).map(DispatchResult::new)
        }

        _ => Err(DispatchError::BackendNotImplemented(kind.clone())),
    }
}

pub(crate) fn get_primary_output(spec: &Spec) -> Result<&speccade_spec::OutputSpec, DispatchError> {
    spec.outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))
}

pub(crate) fn write_output_bytes(
    out_root: &Path,
    rel_path: &str,
    bytes: &[u8],
) -> Result<(), DispatchError> {
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
            | "skeletal_animation.blender_rigged_v1"
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
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe};

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
        assert_eq!(
            get_backend_tier("skeletal_animation.blender_rigged_v1"),
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
        assert!(is_backend_available("skeletal_animation.blender_rigged_v1"));

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
        let outputs =
            dispatch_generate(&spec, tmp.path().to_str().unwrap(), &spec_path, None).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].format, OutputFormat::Png);

        let output_path = tmp.path().join("textures/mask.png");
        assert!(output_path.exists());
        let bytes = std::fs::read(&output_path).unwrap();
        assert!(!bytes.is_empty());
    }
}
