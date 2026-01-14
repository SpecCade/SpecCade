//! Determinism testing framework for SpecCade.
//!
//! This module provides utilities for verifying that asset generation produces
//! byte-identical output across multiple runs (Tier 1 requirement).
//!
//! # Overview
//!
//! SpecCade guarantees deterministic output: given the same spec and seed, the
//! generated assets must be identical. This module provides tools to verify
//! this property across:
//!
//! - Multiple runs of the same generation function
//! - Different asset types (audio, texture, music, mesh)
//! - Multiple spec files in batch
//!
//! # Example
//!
//! ```rust,ignore
//! use speccade_tests::determinism::{verify_determinism, DeterminismFixture};
//!
//! // Verify a single generation function
//! let result = verify_determinism(|| generate_audio(&spec), 3);
//! assert!(result.is_deterministic);
//!
//! // Verify multiple specs
//! let fixture = DeterminismFixture::new()
//!     .add_spec("path/to/spec1.json")
//!     .add_spec("path/to/spec2.json")
//!     .runs(5);
//! let report = fixture.run();
//! assert!(report.all_deterministic());
//! ```

pub mod builder;
pub mod core;
pub mod fixture;
#[macro_use]
pub mod macros;
pub mod report;

#[cfg(test)]
mod tests;

// Re-export core types and functions for convenience
pub use builder::DeterminismBuilder;
pub use core::{
    assert_deterministic, compute_hash, verify_determinism, verify_hash_determinism,
    DeterminismResult, DiffContext, DiffInfo,
};
pub use fixture::DeterminismFixture;
pub use report::{DeterminismError, DeterminismReport, DeterminismReportEntry};
