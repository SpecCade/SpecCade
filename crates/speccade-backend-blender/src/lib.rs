//! SpecCade Blender Backend
//!
//! This crate provides Blender-based asset generation for SpecCade. It implements
//! Tier 2 backends for mesh and animation generation using Blender as a subprocess.
//!
//! # Overview
//!
//! The Blender backend supports six recipe kinds:
//!
//! - **`static_mesh.blender_primitives_v1`** - Generate static meshes from primitives
//! - **`static_mesh.modular_kit_v1`** - Generate modular kit meshes (walls, pipes, doors)
//! - **`skeletal_mesh.blender_rigged_mesh_v1`** - Generate rigged character meshes
//! - **`skeletal_animation.blender_clip_v1`** - Generate animation clips (simple keyframes)
//! - **`skeletal_animation.blender_rigged_v1`** - Generate IK/rig-aware animations
//! - **`sprite.render_from_mesh_v1`** - Render 3D mesh to sprite atlas
//!
//! # Architecture
//!
//! The backend uses a two-part architecture:
//!
//! 1. **Rust Orchestrator** - Spawns Blender, passes specs, and collects results
//! 2. **Python Entrypoint** - Runs inside Blender to perform actual generation
//!
//! Communication happens via JSON files:
//! - Spec JSON is written to a temp file and passed to Blender
//! - Blender writes a report JSON with metrics and output path
//!
//! # Tier 2 Validation
//!
//! Unlike Tier 1 backends (audio, music, texture), Blender backends do not guarantee
//! byte-identical output. Instead, determinism is validated via metrics:
//!
//! | Metric | Tolerance |
//! |--------|-----------|
//! | Triangle count | Exact match |
//! | Bounding box | +/- 0.001 units |
//! | UV island count | Exact match |
//! | Bone count | Exact match |
//! | Material slot count | Exact match |
//! | Animation frame count | Exact match |
//! | Animation duration | +/- 0.001 seconds |
//!
//! # Example
//!
//! ```ignore
//! use speccade_backend_blender::static_mesh;
//! use speccade_spec::Spec;
//! use std::path::Path;
//!
//! let spec = Spec::from_json(json_string)?;
//! let result = static_mesh::generate(&spec, Path::new("output"))?;
//!
//! println!("Generated: {:?}", result.output_path);
//! println!("Triangles: {:?}", result.metrics.triangle_count);
//! ```
//!
//! # Blender Requirements
//!
//! This crate requires Blender to be installed. The orchestrator searches for Blender in:
//!
//! 1. `BLENDER_PATH` environment variable
//! 2. System PATH
//! 3. Common installation locations (platform-specific)
//!
//! Recommended Blender version: 3.6 LTS or 4.0+
//!
//! # Crate Structure
//!
//! - [`static_mesh`] - Static mesh generation
//! - [`modular_kit`] - Modular kit mesh generation (walls, pipes, doors)
//! - [`skeletal_mesh`] - Skeletal mesh generation
//! - [`animation`] - Animation clip generation (simple keyframes)
//! - [`rigged_animation`] - IK/rig-aware animation generation
//! - [`mesh_to_sprite`] - Mesh-to-sprite atlas generation
//! - [`orchestrator`] - Blender subprocess management
//! - [`metrics`] - Tier 2 validation metrics
//! - [`error`] - Error types

pub mod animation;
pub mod error;
pub mod mesh_to_sprite;
pub mod metrics;
pub mod modular_kit;
pub mod orchestrator;
pub mod organic_sculpt;
pub mod rigged_animation;
pub mod skeletal_mesh;
pub mod static_mesh;

// Re-export main types at crate root
pub use error::{BlenderError, BlenderResult};
pub use metrics::{BlenderMetrics, BlenderReport, BoundingBox, MetricTolerances};
pub use orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

// Re-export result types
pub use animation::AnimationResult;
pub use mesh_to_sprite::MeshToSpriteResult;
pub use modular_kit::ModularKitResult;
pub use organic_sculpt::OrganicSculptResult;
pub use rigged_animation::RiggedAnimationResult;
pub use skeletal_mesh::SkeletalMeshResult;
pub use static_mesh::StaticMeshResult;

/// Generate any supported Blender asset from a spec.
///
/// This is a convenience function that dispatches to the appropriate
/// generator based on the recipe kind.
pub fn generate(
    spec: &speccade_spec::Spec,
    out_root: &std::path::Path,
) -> BlenderResult<GenerateResult> {
    let recipe = spec.recipe.as_ref().ok_or(BlenderError::MissingRecipe)?;

    match recipe.kind.as_str() {
        "static_mesh.blender_primitives_v1" => {
            let result = static_mesh::generate(spec, out_root)?;
            Ok(GenerateResult::StaticMesh(result))
        }
        "static_mesh.modular_kit_v1" => {
            let result = modular_kit::generate(spec, out_root)?;
            Ok(GenerateResult::ModularKit(result))
        }
        "static_mesh.organic_sculpt_v1" => {
            let result = organic_sculpt::generate(spec, out_root)?;
            Ok(GenerateResult::OrganicSculpt(result))
        }
        "skeletal_mesh.blender_rigged_mesh_v1" => {
            let result = skeletal_mesh::generate(spec, out_root)?;
            Ok(GenerateResult::SkeletalMesh(result))
        }
        "skeletal_animation.blender_clip_v1" => {
            let result = animation::generate(spec, out_root)?;
            Ok(GenerateResult::Animation(result))
        }
        "skeletal_animation.blender_rigged_v1" => {
            let result = rigged_animation::generate(spec, out_root)?;
            Ok(GenerateResult::RiggedAnimation(result))
        }
        "sprite.render_from_mesh_v1" => {
            let result = mesh_to_sprite::generate(spec, out_root)?;
            Ok(GenerateResult::MeshToSprite(result))
        }
        _ => Err(BlenderError::InvalidRecipeKind {
            kind: recipe.kind.clone(),
        }),
    }
}

/// Result of generating a Blender asset.
#[derive(Debug)]
pub enum GenerateResult {
    /// Static mesh result.
    StaticMesh(StaticMeshResult),
    /// Modular kit mesh result (walls, pipes, doors).
    ModularKit(ModularKitResult),
    /// Organic sculpt mesh result (metaballs, remesh, smooth, displacement).
    OrganicSculpt(OrganicSculptResult),
    /// Skeletal mesh result.
    SkeletalMesh(SkeletalMeshResult),
    /// Animation result (simple keyframes).
    Animation(AnimationResult),
    /// Rigged animation result (IK/rig-aware).
    RiggedAnimation(RiggedAnimationResult),
    /// Mesh-to-sprite result.
    MeshToSprite(MeshToSpriteResult),
}

impl GenerateResult {
    /// Returns the output path for the generated asset.
    pub fn output_path(&self) -> &std::path::Path {
        match self {
            GenerateResult::StaticMesh(r) => &r.output_path,
            GenerateResult::ModularKit(r) => &r.output_path,
            GenerateResult::OrganicSculpt(r) => &r.output_path,
            GenerateResult::SkeletalMesh(r) => &r.output_path,
            GenerateResult::Animation(r) => &r.output_path,
            GenerateResult::RiggedAnimation(r) => &r.output_path,
            GenerateResult::MeshToSprite(r) => &r.output_path,
        }
    }

    /// Returns the metrics for the generated asset.
    /// Returns None for mesh-to-sprite results (use mesh_to_sprite_metrics instead).
    pub fn metrics(&self) -> Option<&BlenderMetrics> {
        match self {
            GenerateResult::StaticMesh(r) => Some(&r.metrics),
            GenerateResult::ModularKit(r) => Some(&r.metrics),
            GenerateResult::OrganicSculpt(r) => Some(&r.metrics),
            GenerateResult::SkeletalMesh(r) => Some(&r.metrics),
            GenerateResult::Animation(r) => Some(&r.metrics),
            GenerateResult::RiggedAnimation(r) => Some(&r.metrics),
            GenerateResult::MeshToSprite(_) => None,
        }
    }

    /// Returns mesh-to-sprite metrics if this is a mesh-to-sprite result.
    pub fn mesh_to_sprite_metrics(&self) -> Option<&mesh_to_sprite::MeshToSpriteMetrics> {
        match self {
            GenerateResult::MeshToSprite(r) => Some(&r.metrics),
            _ => None,
        }
    }

    /// Returns the Blender report.
    pub fn report(&self) -> &BlenderReport {
        match self {
            GenerateResult::StaticMesh(r) => &r.report,
            GenerateResult::ModularKit(r) => &r.report,
            GenerateResult::OrganicSculpt(r) => &r.report,
            GenerateResult::SkeletalMesh(r) => &r.report,
            GenerateResult::Animation(r) => &r.report,
            GenerateResult::RiggedAnimation(r) => &r.report,
            GenerateResult::MeshToSprite(r) => &r.report,
        }
    }

    /// Returns true if this is a static mesh result.
    pub fn is_static_mesh(&self) -> bool {
        matches!(self, GenerateResult::StaticMesh(_))
    }

    /// Returns true if this is a modular kit mesh result.
    pub fn is_modular_kit(&self) -> bool {
        matches!(self, GenerateResult::ModularKit(_))
    }

    /// Returns true if this is an organic sculpt mesh result.
    pub fn is_organic_sculpt(&self) -> bool {
        matches!(self, GenerateResult::OrganicSculpt(_))
    }

    /// Returns true if this is a skeletal mesh result.
    pub fn is_skeletal_mesh(&self) -> bool {
        matches!(self, GenerateResult::SkeletalMesh(_))
    }

    /// Returns true if this is an animation result (simple keyframes).
    pub fn is_animation(&self) -> bool {
        matches!(self, GenerateResult::Animation(_))
    }

    /// Returns true if this is a rigged animation result (IK/rig-aware).
    pub fn is_rigged_animation(&self) -> bool {
        matches!(self, GenerateResult::RiggedAnimation(_))
    }

    /// Returns true if this is a mesh-to-sprite result.
    pub fn is_mesh_to_sprite(&self) -> bool {
        matches!(self, GenerateResult::MeshToSprite(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_dispatch() {
        assert!(orchestrator::mode_from_recipe_kind("static_mesh.blender_primitives_v1").is_ok());
        assert!(orchestrator::mode_from_recipe_kind("static_mesh.modular_kit_v1").is_ok());
        assert!(
            orchestrator::mode_from_recipe_kind("skeletal_mesh.blender_rigged_mesh_v1").is_ok()
        );
        assert!(orchestrator::mode_from_recipe_kind("skeletal_animation.blender_clip_v1").is_ok());
        assert!(
            orchestrator::mode_from_recipe_kind("skeletal_animation.blender_rigged_v1").is_ok()
        );
        assert!(orchestrator::mode_from_recipe_kind("invalid.kind").is_err());
    }

    #[test]
    fn test_generate_result_methods() {
        let metrics = BlenderMetrics::for_static_mesh(
            100,
            BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 1.0, 1.0]),
            4,
            2,
        );

        let result = GenerateResult::StaticMesh(StaticMeshResult {
            output_path: std::path::PathBuf::from("test.glb"),
            metrics: metrics.clone(),
            report: BlenderReport::success(metrics, "test.glb".to_string()),
        });

        assert!(result.is_static_mesh());
        assert!(!result.is_skeletal_mesh());
        assert!(!result.is_animation());
        assert!(!result.is_mesh_to_sprite());
        assert_eq!(result.output_path(), std::path::Path::new("test.glb"));
        assert_eq!(result.metrics().unwrap().triangle_count, Some(100));
    }

    #[test]
    fn test_mesh_to_sprite_mode() {
        assert!(orchestrator::mode_from_recipe_kind("sprite.render_from_mesh_v1").is_ok());
        assert_eq!(
            orchestrator::mode_from_recipe_kind("sprite.render_from_mesh_v1").unwrap(),
            GenerationMode::MeshToSprite
        );
    }
}
