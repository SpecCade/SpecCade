//! Backend dispatch module
//!
//! Dispatches generation requests to the appropriate backend based on recipe.kind.
//! Currently returns NotImplemented for all backends - actual implementations
//! will be added in Phase 4.

use speccade_spec::{OutputResult, Spec};
use std::fmt;

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
pub fn dispatch_generate(spec: &Spec, _out_root: &str) -> Result<Vec<OutputResult>, DispatchError> {
    // Get the recipe kind
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let kind = &recipe.kind;

    // Dispatch based on recipe kind prefix
    // For now, all backends return NotImplemented
    match kind.as_str() {
        // Audio backends (Phase 4)
        k if k.starts_with("audio_sfx.") => {
            Err(DispatchError::BackendNotImplemented(kind.clone()))
        }
        k if k.starts_with("audio_instrument.") => {
            Err(DispatchError::BackendNotImplemented(kind.clone()))
        }

        // Music backends (Phase 4)
        k if k.starts_with("music.") => {
            Err(DispatchError::BackendNotImplemented(kind.clone()))
        }

        // Texture backends (Phase 5)
        k if k.starts_with("texture_2d.") => {
            Err(DispatchError::BackendNotImplemented(kind.clone()))
        }

        // Mesh backends (Phase 6)
        k if k.starts_with("static_mesh.") => {
            Err(DispatchError::BackendNotImplemented(kind.clone()))
        }
        k if k.starts_with("skeletal_mesh.") => {
            Err(DispatchError::BackendNotImplemented(kind.clone()))
        }

        // Animation backends (Phase 6)
        k if k.starts_with("skeletal_animation.") => {
            Err(DispatchError::BackendNotImplemented(kind.clone()))
        }

        // Unknown recipe kind
        _ => Err(DispatchError::BackendNotImplemented(kind.clone())),
    }
}

/// Check if a backend is available for a given recipe kind
///
/// # Arguments
/// * `kind` - The recipe kind to check
///
/// # Returns
/// True if the backend is implemented and available
pub fn is_backend_available(kind: &str) -> bool {
    // Currently no backends are implemented
    // This will be updated as backends are added in Phase 4+
    match kind {
        // Future: return true for implemented backends
        _ => false,
    }
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
    fn test_no_backends_available() {
        // All backends should be unavailable in Phase 3
        assert!(!is_backend_available("audio_sfx.layered_synth_v1"));
        assert!(!is_backend_available("music.tracker_song_v1"));
        assert!(!is_backend_available("static_mesh.blender_primitives_v1"));
    }
}
