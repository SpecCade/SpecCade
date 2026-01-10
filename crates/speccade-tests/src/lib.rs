//! SpecCade End-to-End Test Infrastructure
//!
//! This crate provides integration tests for parity-critical flows:
//!
//! - Migration: Legacy .spec.py -> canonical JSON
//! - Generation: Spec JSON -> output files
//! - Validation: Output file existence and validity
//! - Audit: Migrator audit mode on test fixtures
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

pub mod fixtures;
pub mod harness;
