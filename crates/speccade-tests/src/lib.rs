//! SpecCade End-to-End Test Infrastructure
//!
//! This crate provides integration tests for parity-critical flows:
//!
//! - Generation: Spec -> output files
//! - Validation: Output file existence and validity
//! - **Determinism**: Verify byte-identical output across runs (Tier 1)
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all Tier 1 tests (no Blender required)
//! cargo test -p speccade-tests
//!
//! # Run Tier 2 tests (requires Blender)
//! SPECCADE_RUN_BLENDER_TESTS=1 cargo test -p speccade-tests --ignored
//! ```
//!
//! ## Determinism Testing
//!
//! The `determinism` module provides tools for verifying that generation
//! produces byte-identical output across runs:
//!
//! ```rust,ignore
//! use speccade_tests::determinism::{verify_determinism, DeterminismFixture};
//! use speccade_tests::test_determinism;
//!
//! // Verify a single generation function
//! let result = verify_determinism(|| generate_audio(&spec), 3);
//! assert!(result.is_deterministic);
//!
//! // Use the macro for quick tests
//! test_determinism!(audio_laser, {
//!     generate_audio(&spec).wav_data
//! });
//! ```

pub mod audio_analysis;
pub mod determinism;
pub mod fixtures;
pub mod format_validators;
pub mod harness;

// Re-export commonly used items
pub use determinism::{
    assert_deterministic, compute_hash, verify_determinism, verify_hash_determinism,
    DeterminismError, DeterminismFixture, DeterminismReport, DeterminismResult,
};
