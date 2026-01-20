//! SpecCade Canonical Spec Library
//!
//! This crate provides types, validation, and hashing for SpecCade canonical specs.
//! Specs are JSON documents that describe declarative asset generation requests.
//!
//! # Overview
//!
//! SpecCade specs follow the structure defined in RFC-0001:
//!
//! - **Contract fields**: Required metadata like `asset_id`, `asset_type`, `seed`, and `outputs`
//! - **Recipe**: Backend-specific generation parameters (optional for validation, required for generation)
//!
//! # Example
//!
//! ```
//! use speccade_spec::{Spec, AssetType, OutputSpec, OutputFormat};
//! use speccade_spec::validation::validate_spec;
//! use speccade_spec::hash::canonical_spec_hash;
//!
//! // Build a spec
//! let spec = Spec::builder("laser-blast-01", AssetType::Audio)
//!     .license("CC0-1.0")
//!     .seed(42)
//!     .description("Sci-fi laser blast sound effect")
//!     .tag("retro")
//!     .tag("scifi")
//!     .output(OutputSpec::primary(OutputFormat::Wav, "sounds/laser_blast.wav"))
//!     .build();
//!
//! // Validate the spec
//! let result = validate_spec(&spec);
//! assert!(result.is_ok());
//!
//! // Compute the canonical hash
//! let hash = canonical_spec_hash(&spec).unwrap();
//! println!("Spec hash: {}", hash);
//! ```
//!
//! # Modules
//!
//! - [`error`]: Error and warning types for validation
//! - [`output`]: Output specification types (kind, format, path)
//! - [`recipe`]: Recipe types for all supported backends
//! - [`report`]: Report types and builder for generation results
//! - [`spec`]: Main spec type and builder
//! - [`validation`]: Spec validation functions
//! - [`hash`]: Canonical hashing and seed derivation

pub mod error;
pub mod hash;
pub mod output;
pub mod recipe;
pub mod report;
pub mod spec;
pub mod validation;

// Re-export commonly used types at the crate root
pub use error::{
    BackendError, ErrorCode, GenerationError, SpecError, ValidationError, ValidationResult,
    ValidationWarning, WarningCode,
};
pub use hash::{
    canonical_recipe_hash, canonical_spec_hash, derive_layer_seed, derive_variant_seed,
    derive_variant_spec_seed,
};
pub use output::{EngineTarget, OutputFormat, OutputKind, OutputSpec, VariantSpec};
pub use recipe::{Recipe, RecipeParamsError};
pub use report::{
    BakedMapInfo, BakingMetrics, BoundingBox, CollisionBoundingBox, CollisionMeshMetrics,
    NavmeshMetrics, OutputMetrics, OutputResult, Report, ReportBuilder, ReportError, ReportWarning,
    StageTiming, StaticMeshLodLevelMetrics, REPORT_VERSION,
};
pub use spec::{AssetType, Spec, SpecBuilder, MAX_SEED, SPEC_VERSION};
pub use validation::constraints::{
    evaluate_constraints, Constraint, ConstraintResult, ConstraintSet, VerifyResult,
};
pub use validation::{
    is_safe_output_path, is_valid_asset_id, validate_for_generate,
    validate_for_generate_with_budget, validate_spec, validate_spec_with_budget, AudioBudget,
    BudgetCategory, BudgetError, BudgetProfile, GeneralBudget, MeshBudget, MusicBudget,
    TextureBudget,
};

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test parsing the example spec from RFC-0001 Section 6.1 (Audio)
    #[test]
    fn test_parse_rfc_example_audio() {
        let json = r#"{
            "spec_version": 1,
            "asset_id": "laser-blast-01",
            "asset_type": "audio",
            "license": "CC0-1.0",
            "seed": 42,
            "description": "Sci-fi laser blast sound effect",
            "style_tags": ["retro", "scifi", "action"],
            "outputs": [
                {
                    "kind": "primary",
                    "format": "wav",
                    "path": "sounds/laser_blast_01.wav"
                }
            ],
            "recipe": {
                "kind": "audio_v1",
                "params": {
                    "duration_seconds": 0.3,
                    "sample_rate": 44100,
                    "layers": [
                        {
                            "synthesis": {
                                "type": "fm_synth",
                                "carrier_freq": 440.0,
                                "modulator_freq": 880.0,
                                "modulation_index": 2.5,
                                "freq_sweep": {
                                    "end_freq": 110.0,
                                    "curve": "exponential"
                                }
                            },
                            "envelope": {
                                "attack": 0.01,
                                "decay": 0.05,
                                "sustain": 0.3,
                                "release": 0.15
                            },
                            "volume": 0.8,
                            "pan": 0.0
                        }
                    ]
                }
            }
        }"#;

        let spec = Spec::from_json(json).expect("should parse");

        assert_eq!(spec.spec_version, 1);
        assert_eq!(spec.asset_id, "laser-blast-01");
        assert_eq!(spec.asset_type, AssetType::Audio);
        assert_eq!(spec.license, "CC0-1.0");
        assert_eq!(spec.seed, 42);
        assert!(spec.description.is_some());
        assert!(spec.has_recipe());
        assert!(spec.has_primary_output());

        // Validate
        let result = validate_spec(&spec);
        assert!(result.is_ok(), "errors: {:?}", result.errors);

        // Should also pass generate validation since recipe is present
        let result = validate_for_generate(&spec);
        assert!(result.is_ok(), "errors: {:?}", result.errors);
    }

    /// Test parsing the example spec from RFC-0001 Section 6.3 (Texture)
    #[test]
    fn test_parse_rfc_example_texture() {
        let json = r#"{
            "spec_version": 1,
            "asset_id": "metal-panel-01",
            "asset_type": "texture",
            "license": "CC0-1.0",
            "seed": 98765,
            "description": "Worn metal panel texture with scratches",
            "style_tags": ["metal", "industrial", "scifi"],
            "outputs": [
                {
                    "kind": "primary",
                    "format": "png",
                    "path": "textures/metal_panel_01_albedo.png"
                },
                {
                    "kind": "primary",
                    "format": "png",
                    "path": "textures/metal_panel_01_normal.png"
                }
            ]
        }"#;

        let spec = Spec::from_json(json).expect("should parse");

        assert_eq!(spec.asset_id, "metal-panel-01");
        assert_eq!(spec.asset_type, AssetType::Texture);
        assert_eq!(spec.outputs.len(), 2);
        assert_eq!(spec.primary_outputs().count(), 2);

        let result = validate_spec(&spec);
        assert!(result.is_ok(), "errors: {:?}", result.errors);
    }

    /// Test parsing the example spec from RFC-0001 Section 6.4 (Static Mesh)
    #[test]
    fn test_parse_rfc_example_static_mesh() {
        let json = r#"{
            "spec_version": 1,
            "asset_id": "crate-wooden-01",
            "asset_type": "static_mesh",
            "license": "CC0-1.0",
            "seed": 54321,
            "description": "Simple wooden storage crate",
            "style_tags": ["prop", "container", "lowpoly"],
            "engine_targets": ["godot", "unity"],
            "outputs": [
                {
                    "kind": "primary",
                    "format": "glb",
                    "path": "meshes/crate_wooden_01.glb"
                }
            ],
            "recipe": {
                "kind": "static_mesh.blender_primitives_v1",
                "params": {
                    "base_primitive": "cube",
                    "dimensions": [1.0, 1.0, 1.0]
                }
            }
        }"#;

        let spec = Spec::from_json(json).expect("should parse");

        assert_eq!(spec.asset_id, "crate-wooden-01");
        assert_eq!(spec.asset_type, AssetType::StaticMesh);
        assert!(spec.engine_targets.is_some());

        let targets = spec.engine_targets.as_ref().unwrap();
        assert!(targets.contains(&EngineTarget::Godot));
        assert!(targets.contains(&EngineTarget::Unity));

        let result = validate_spec(&spec);
        assert!(result.is_ok(), "errors: {:?}", result.errors);
    }

    /// Test hash stability
    #[test]
    fn test_hash_stability() {
        let spec = Spec::builder("test-stable-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(12345)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let hash1 = canonical_spec_hash(&spec).unwrap();
        let hash2 = canonical_spec_hash(&spec).unwrap();

        assert_eq!(hash1, hash2, "hash should be stable across calls");
        assert_eq!(hash1.len(), 64, "hash should be 64 hex characters");
    }

    /// Test seed derivation consistency
    #[test]
    fn test_seed_derivation_consistency() {
        let base_seed = 42u32;

        // Layer seeds should be consistent
        let layer0_a = derive_layer_seed(base_seed, 0);
        let layer0_b = derive_layer_seed(base_seed, 0);
        assert_eq!(layer0_a, layer0_b);

        // Variant seeds should be consistent
        let soft_a = derive_variant_seed(base_seed, "soft");
        let soft_b = derive_variant_seed(base_seed, "soft");
        assert_eq!(soft_a, soft_b);

        // Different inputs should produce different outputs
        let layer1 = derive_layer_seed(base_seed, 1);
        assert_ne!(layer0_a, layer1);

        let hard = derive_variant_seed(base_seed, "hard");
        assert_ne!(soft_a, hard);
    }

    /// Test validation error messages for invalid specs
    #[test]
    fn test_validation_error_messages() {
        // Invalid asset_id
        let spec = Spec::builder("INVALID", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .build();

        let result = validate_spec(&spec);
        assert!(!result.is_ok());

        let error = result
            .errors
            .iter()
            .find(|e| e.code == ErrorCode::InvalidAssetId);
        assert!(error.is_some());
        assert!(error.unwrap().message.contains("asset_id"));
    }

    /// Test that we can round-trip a spec through JSON
    #[test]
    fn test_json_round_trip() {
        let spec = Spec::builder("round-trip-01", AssetType::Music)
            .license("CC-BY-4.0")
            .seed(999)
            .description("Test round trip")
            .tag("test")
            .output(OutputSpec::primary(OutputFormat::Xm, "music/test.xm"))
            .build();

        let json = spec.to_json_pretty().unwrap();
        let parsed = Spec::from_json(&json).unwrap();

        assert_eq!(spec.asset_id, parsed.asset_id);
        assert_eq!(spec.asset_type, parsed.asset_type);
        assert_eq!(spec.seed, parsed.seed);
        assert_eq!(spec.license, parsed.license);
        assert_eq!(spec.outputs.len(), parsed.outputs.len());
    }
}
