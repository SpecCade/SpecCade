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

        // Trimsheet atlas backend
        "texture.trimsheet_v1" => texture::generate_texture_trimsheet(spec, out_root_path),

        // Decal texture backend
        "texture.decal_v1" => texture::generate_texture_decal(spec, out_root_path),

        // Splat set texture backend
        "texture.splat_set_v1" => texture::generate_texture_splat_set(spec, out_root_path),

        // Matcap texture backend
        "texture.matcap_v1" => texture::generate_texture_matcap(spec, out_root_path),

        // Material preset texture backend
        "texture.material_preset_v1" => {
            texture::generate_texture_material_preset(spec, out_root_path)
        }

        // Sprite sheet backend
        "sprite.sheet_v1" => texture::generate_sprite_sheet(spec, out_root_path),

        // Sprite animation backend
        "sprite.animation_v1" => texture::generate_sprite_animation(spec, out_root_path),

        // Sprite render-from-mesh backend (Blender Tier 2)
        "sprite.render_from_mesh_v1" => {
            blender::generate_blender_sprite_from_mesh(spec, out_root_path)
        }

        // VFX flipbook backend
        "vfx.flipbook_v1" => texture::generate_vfx_flipbook(spec, out_root_path),

        // VFX particle profile backend (metadata-only)
        "vfx.particle_profile_v1" => texture::generate_vfx_particle_profile(spec, out_root_path),

        // UI nine-slice backend
        "ui.nine_slice_v1" => texture::generate_ui_nine_slice(spec, out_root_path),

        // UI icon set backend
        "ui.icon_set_v1" => texture::generate_ui_icon_set(spec, out_root_path),

        // UI item card backend
        "ui.item_card_v1" => texture::generate_ui_item_card(spec, out_root_path),

        // UI damage number backend
        "ui.damage_number_v1" => texture::generate_ui_damage_number(spec, out_root_path),

        // Bitmap font backend
        "font.bitmap_v1" => texture::generate_font_bitmap(spec, out_root_path),

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

        "texture.trimsheet_v1" => {
            if profile {
                texture::generate_texture_trimsheet_profiled(spec, out_root_path)
            } else {
                texture::generate_texture_trimsheet(spec, out_root_path).map(DispatchResult::new)
            }
        }

        "texture.decal_v1" => {
            if profile {
                texture::generate_texture_decal_profiled(spec, out_root_path)
            } else {
                texture::generate_texture_decal(spec, out_root_path).map(DispatchResult::new)
            }
        }

        "texture.splat_set_v1" => {
            if profile {
                texture::generate_texture_splat_set_profiled(spec, out_root_path)
            } else {
                texture::generate_texture_splat_set(spec, out_root_path).map(DispatchResult::new)
            }
        }

        "texture.matcap_v1" => {
            if profile {
                texture::generate_texture_matcap_profiled(spec, out_root_path)
            } else {
                texture::generate_texture_matcap(spec, out_root_path).map(DispatchResult::new)
            }
        }

        "texture.material_preset_v1" => {
            if profile {
                texture::generate_texture_material_preset_profiled(spec, out_root_path)
            } else {
                texture::generate_texture_material_preset(spec, out_root_path)
                    .map(DispatchResult::new)
            }
        }

        "sprite.sheet_v1" => {
            if profile {
                texture::generate_sprite_sheet_profiled(spec, out_root_path)
            } else {
                texture::generate_sprite_sheet(spec, out_root_path).map(DispatchResult::new)
            }
        }

        "sprite.animation_v1" => {
            if profile {
                texture::generate_sprite_animation_profiled(spec, out_root_path)
            } else {
                texture::generate_sprite_animation(spec, out_root_path).map(DispatchResult::new)
            }
        }

        // Blender sprite-from-mesh backend (no profiling instrumentation yet)
        "sprite.render_from_mesh_v1" => {
            blender::generate_blender_sprite_from_mesh(spec, out_root_path).map(DispatchResult::new)
        }

        "vfx.flipbook_v1" => {
            if profile {
                texture::generate_vfx_flipbook_profiled(spec, out_root_path)
            } else {
                texture::generate_vfx_flipbook(spec, out_root_path).map(DispatchResult::new)
            }
        }

        "vfx.particle_profile_v1" => {
            if profile {
                texture::generate_vfx_particle_profile_profiled(spec, out_root_path)
            } else {
                texture::generate_vfx_particle_profile(spec, out_root_path).map(DispatchResult::new)
            }
        }

        "ui.nine_slice_v1" => {
            if profile {
                texture::generate_ui_nine_slice_profiled(spec, out_root_path)
            } else {
                texture::generate_ui_nine_slice(spec, out_root_path).map(DispatchResult::new)
            }
        }

        "ui.icon_set_v1" => {
            if profile {
                texture::generate_ui_icon_set_profiled(spec, out_root_path)
            } else {
                texture::generate_ui_icon_set(spec, out_root_path).map(DispatchResult::new)
            }
        }

        "ui.item_card_v1" => {
            if profile {
                texture::generate_ui_item_card_profiled(spec, out_root_path)
            } else {
                texture::generate_ui_item_card(spec, out_root_path).map(DispatchResult::new)
            }
        }

        "ui.damage_number_v1" => {
            if profile {
                texture::generate_ui_damage_number_profiled(spec, out_root_path)
            } else {
                texture::generate_ui_damage_number(spec, out_root_path).map(DispatchResult::new)
            }
        }

        "font.bitmap_v1" => {
            if profile {
                texture::generate_font_bitmap_profiled(spec, out_root_path)
            } else {
                texture::generate_font_bitmap(spec, out_root_path).map(DispatchResult::new)
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
            | "texture.trimsheet_v1"
            | "texture.decal_v1"
            | "texture.splat_set_v1"
            | "texture.matcap_v1"
            | "sprite.sheet_v1"
            | "sprite.animation_v1"
            | "vfx.flipbook_v1"
            | "vfx.particle_profile_v1"
            | "ui.nine_slice_v1"
            | "ui.icon_set_v1"
            | "ui.item_card_v1"
            | "ui.damage_number_v1"
            | "font.bitmap_v1"
            | "static_mesh.blender_primitives_v1"
            | "skeletal_mesh.blender_rigged_mesh_v1"
            | "skeletal_animation.blender_clip_v1"
            | "skeletal_animation.blender_rigged_v1"
            | "sprite.render_from_mesh_v1"
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
        // Note: sprite.render_from_mesh_v1 is Tier 2 (Blender), handled below
        k if k.starts_with("sprite.") && k != "sprite.render_from_mesh_v1" => Some(1),
        k if k.starts_with("vfx.") => Some(1),
        k if k.starts_with("ui.") => Some(1),
        k if k.starts_with("font.") => Some(1),

        // Tier 2: Blender backends (metric validation only)
        k if k.starts_with("static_mesh.") => Some(2),
        k if k.starts_with("skeletal_mesh.") => Some(2),
        k if k.starts_with("skeletal_animation.") => Some(2),
        "sprite.render_from_mesh_v1" => Some(2),

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
        assert_eq!(get_backend_tier("texture.trimsheet_v1"), Some(1));

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
        assert!(is_backend_available("texture.trimsheet_v1"));
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

    #[test]
    fn test_dispatch_texture_trimsheet_generates_outputs() {
        let tmp = tempfile::tempdir().unwrap();

        let primary_output = OutputSpec::primary(OutputFormat::Png, "atlas/trimsheet.png");
        let metadata_output = OutputSpec::metadata("atlas/trimsheet.json");

        let recipe = Recipe::new(
            "texture.trimsheet_v1",
            serde_json::json!({
                "resolution": [256, 256],
                "padding": 2,
                "tiles": [
                    { "id": "grass", "width": 64, "height": 64, "color": [0.2, 0.6, 0.2, 1.0] },
                    { "id": "stone", "width": 32, "height": 32, "color": [0.5, 0.5, 0.5, 1.0] }
                ]
            }),
        );

        let spec = Spec::builder("test-trimsheet-01", AssetType::Texture)
            .license("CC0-1.0")
            .seed(42)
            .output(primary_output)
            .output(metadata_output)
            .recipe(recipe)
            .build();

        let spec_path = tmp.path().join("test.spec.json");
        let outputs =
            dispatch_generate(&spec, tmp.path().to_str().unwrap(), &spec_path, None).unwrap();

        // Should have 2 outputs: PNG atlas and JSON metadata
        assert_eq!(outputs.len(), 2);

        // Check PNG output
        let png_output = outputs
            .iter()
            .find(|o| o.format == OutputFormat::Png)
            .unwrap();
        let png_path = tmp.path().join("atlas/trimsheet.png");
        assert!(png_path.exists());
        let png_bytes = std::fs::read(&png_path).unwrap();
        assert!(!png_bytes.is_empty());
        assert!(png_output.hash.is_some());

        // Check JSON metadata output
        let json_output = outputs
            .iter()
            .find(|o| o.format == OutputFormat::Json)
            .unwrap();
        let json_path = tmp.path().join("atlas/trimsheet.json");
        assert!(json_path.exists());
        let json_str = std::fs::read_to_string(&json_path).unwrap();
        let metadata: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(metadata["atlas_width"], 256);
        assert_eq!(metadata["atlas_height"], 256);
        assert_eq!(metadata["padding"], 2);
        assert!(metadata["tiles"].is_array());
        assert_eq!(metadata["tiles"].as_array().unwrap().len(), 2);
        assert!(json_output.hash.is_some());
    }

    #[test]
    fn test_dispatch_texture_trimsheet_determinism() {
        let tmp1 = tempfile::tempdir().unwrap();
        let tmp2 = tempfile::tempdir().unwrap();

        let recipe = Recipe::new(
            "texture.trimsheet_v1",
            serde_json::json!({
                "resolution": [128, 128],
                "padding": 1,
                "tiles": [
                    { "id": "a", "width": 32, "height": 32, "color": [1.0, 0.0, 0.0, 1.0] },
                    { "id": "b", "width": 48, "height": 48, "color": [0.0, 1.0, 0.0, 1.0] }
                ]
            }),
        );

        let make_spec = || {
            Spec::builder("test-determinism", AssetType::Texture)
                .license("CC0-1.0")
                .seed(42)
                .output(OutputSpec::primary(OutputFormat::Png, "atlas.png"))
                .output(OutputSpec::metadata("atlas.json"))
                .recipe(recipe.clone())
                .build()
        };

        let spec1 = make_spec();
        let spec2 = make_spec();

        let spec_path1 = tmp1.path().join("test.spec.json");
        let spec_path2 = tmp2.path().join("test.spec.json");

        let outputs1 =
            dispatch_generate(&spec1, tmp1.path().to_str().unwrap(), &spec_path1, None).unwrap();
        let outputs2 =
            dispatch_generate(&spec2, tmp2.path().to_str().unwrap(), &spec_path2, None).unwrap();

        // Hashes should be identical
        let hash1 = outputs1
            .iter()
            .find(|o| o.format == OutputFormat::Png)
            .unwrap()
            .hash
            .as_ref()
            .unwrap();
        let hash2 = outputs2
            .iter()
            .find(|o| o.format == OutputFormat::Png)
            .unwrap()
            .hash
            .as_ref()
            .unwrap();
        assert_eq!(
            hash1, hash2,
            "PNG hashes should be identical for same input"
        );

        // PNG bytes should be byte-identical
        let png1 = std::fs::read(tmp1.path().join("atlas.png")).unwrap();
        let png2 = std::fs::read(tmp2.path().join("atlas.png")).unwrap();
        assert_eq!(png1, png2, "PNG bytes should be identical");
    }
}
