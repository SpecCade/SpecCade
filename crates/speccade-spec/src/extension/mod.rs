//! Extension system types for external backends.
//!
//! This module defines the types used to integrate external backends with SpecCade,
//! including subprocess-based backends and WASM plugins.
//!
//! # Overview
//!
//! External backends communicate with SpecCade through a well-defined I/O contract:
//!
//! 1. **Registration**: Extensions declare their capabilities via a manifest
//! 2. **Invocation**: SpecCade spawns the extension with a spec and seed
//! 3. **Output**: The extension writes files and a manifest describing results
//! 4. **Verification**: SpecCade validates the output manifest and determinism tier
//!
//! # Determinism Tiers for Extensions
//!
//! - **Tier 1** (`byte_identical`): Output must be byte-for-byte identical for the same
//!   input spec and seed. This is enforced via hash verification.
//! - **Tier 2** (`semantic_equivalent`): Output may vary in representation but must be
//!   semantically equivalent. Validated via metrics rather than hashes.
//! - **Tier 3** (`non_deterministic`): No determinism guarantee. Outputs are accepted
//!   but not cached or reproduced.
//!
//! # Example
//!
//! ```
//! use speccade_spec::extension::{ExtensionManifest, DeterminismLevel};
//!
//! // Register an extension using the builder method
//! let manifest = ExtensionManifest::subprocess(
//!     "custom-texture-gen",
//!     "1.0.0",
//!     "custom-texture-gen",
//!     vec!["texture.custom_v1".to_string()],
//!     DeterminismLevel::ByteIdentical,
//! );
//!
//! assert_eq!(manifest.tier, 1);
//! assert!(manifest.handles_recipe("texture.custom_v1"));
//! ```

mod contract;
mod manifest;

pub use contract::{
    ExtensionError, ExtensionErrorEntry, ExtensionOutputFile, ExtensionOutputManifest,
    DeterminismReport, OutputManifestValidationError,
    validate_output_manifest, validate_determinism_report,
};
pub use manifest::{
    ExtensionManifest, ExtensionInterface, DeterminismLevel,
    ManifestValidationError, validate_extension_manifest,
};

#[cfg(test)]
mod tests;
