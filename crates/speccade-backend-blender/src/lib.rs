//! SpecCade Blender Backend
//!
//! This crate provides Blender-based asset generation for SpecCade. It implements
//! Tier 2 backends for mesh and animation generation using Blender as a subprocess.
//!
//! # Overview
//!
//! The Blender backend supports four recipe kinds:
//!
//! - **`static_mesh.blender_primitives_v1`** - Generate static meshes from primitives
//! - **`skeletal_mesh.blender_rigged_mesh_v1`** - Generate rigged character meshes
//! - **`skeletal_animation.blender_clip_v1`** - Generate animation clips (simple keyframes)
//! - **`skeletal_animation.blender_rigged_v1`** - Generate IK/rig-aware animations
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
//! - [`skeletal_mesh`] - Skeletal mesh generation
//! - [`animation`] - Animation clip generation (simple keyframes)
//! - [`rigged_animation`] - IK/rig-aware animation generation
//! - [`orchestrator`] - Blender subprocess management
//! - [`metrics`] - Tier 2 validation metrics
//! - [`error`] - Error types

pub mod animation;
pub mod error;
pub mod metrics;
pub mod orchestrator;
pub mod rigged_animation;
pub mod skeletal_mesh;
pub mod static_mesh;

// Re-export main types at crate root
pub use error::{BlenderError, BlenderResult};
pub use metrics::{BlenderMetrics, BlenderReport, BoundingBox, MetricTolerances};
pub use orchestrator::{GenerationMode, Orchestrator, OrchestratorConfig};

// Re-export result types
pub use animation::AnimationResult;
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
    /// Skeletal mesh result.
    SkeletalMesh(SkeletalMeshResult),
    /// Animation result (simple keyframes).
    Animation(AnimationResult),
    /// Rigged animation result (IK/rig-aware).
    RiggedAnimation(RiggedAnimationResult),
}

impl GenerateResult {
    /// Returns the output path for the generated asset.
    pub fn output_path(&self) -> &std::path::Path {
        match self {
            GenerateResult::StaticMesh(r) => &r.output_path,
            GenerateResult::SkeletalMesh(r) => &r.output_path,
            GenerateResult::Animation(r) => &r.output_path,
            GenerateResult::RiggedAnimation(r) => &r.output_path,
        }
    }

    /// Returns the metrics for the generated asset.
    pub fn metrics(&self) -> &BlenderMetrics {
        match self {
            GenerateResult::StaticMesh(r) => &r.metrics,
            GenerateResult::SkeletalMesh(r) => &r.metrics,
            GenerateResult::Animation(r) => &r.metrics,
            GenerateResult::RiggedAnimation(r) => &r.metrics,
        }
    }

    /// Returns the Blender report.
    pub fn report(&self) -> &BlenderReport {
        match self {
            GenerateResult::StaticMesh(r) => &r.report,
            GenerateResult::SkeletalMesh(r) => &r.report,
            GenerateResult::Animation(r) => &r.report,
            GenerateResult::RiggedAnimation(r) => &r.report,
        }
    }

    /// Returns true if this is a static mesh result.
    pub fn is_static_mesh(&self) -> bool {
        matches!(self, GenerateResult::StaticMesh(_))
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_dispatch() {
        assert!(orchestrator::mode_from_recipe_kind("static_mesh.blender_primitives_v1").is_ok());
        assert!(
            orchestrator::mode_from_recipe_kind("skeletal_mesh.blender_rigged_mesh_v1").is_ok()
        );
        assert!(orchestrator::mode_from_recipe_kind("skeletal_animation.blender_clip_v1").is_ok());
        assert!(orchestrator::mode_from_recipe_kind("skeletal_animation.blender_rigged_v1").is_ok());
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
        assert_eq!(result.output_path(), std::path::Path::new("test.glb"));
        assert_eq!(result.metrics().triangle_count, Some(100));
    }
}
