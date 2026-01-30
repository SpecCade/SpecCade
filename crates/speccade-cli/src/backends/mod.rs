//! External backend extension system.
//!
//! This module provides support for external backends that integrate with SpecCade
//! via subprocess invocation or WASM plugins.
//!
//! # Overview
//!
//! External backends extend SpecCade's generation capabilities without modifying
//! the core codebase. They communicate through a well-defined I/O contract:
//!
//! 1. **Discovery**: Extensions are registered via manifest files
//! 2. **Invocation**: SpecCade spawns the extension with spec and seed
//! 3. **Output**: Extension writes files and a manifest describing results
//! 4. **Verification**: SpecCade validates output manifest and determinism
//!
//! # Subprocess Protocol
//!
//! For subprocess-based extensions:
//!
//! 1. CLI spawns subprocess with: `<exe> --spec <path> --out <path> --seed <u64>`
//! 2. Subprocess reads JSON spec from the spec path
//! 3. Subprocess writes outputs to the output directory
//! 4. Subprocess writes `manifest.json` with:
//!    - `output_files`: list of `{ path, hash, size }`
//!    - `determinism_report`: `{ input_hash, output_hash, tier }`
//!    - `errors`: list of `{ code, message }`
//! 5. Exit code 0 = success, non-zero = failure
//!
//! # Example
//!
//! ```ignore
//! use speccade_cli::backends::{ExtensionRegistry, SubprocessRunner};
//! use speccade_spec::extension::ExtensionManifest;
//!
//! // Load an extension manifest
//! let manifest = ExtensionManifest::subprocess(
//!     "custom-texture-gen",
//!     "1.0.0",
//!     "custom-texture-gen",
//!     vec!["texture.custom_v1".to_string()],
//!     speccade_spec::extension::DeterminismLevel::ByteIdentical,
//! );
//!
//! // Register it
//! let mut registry = ExtensionRegistry::new();
//! registry.register(manifest);
//!
//! // Run it
//! let runner = SubprocessRunner::new();
//! let result = runner.run(&spec, &extension, out_dir).await?;
//! ```

mod registry;
mod subprocess;

pub use registry::{ExtensionRegistry, RegistryError};
pub use subprocess::{SubprocessConfig, SubprocessRunner};

#[cfg(test)]
mod tests;
