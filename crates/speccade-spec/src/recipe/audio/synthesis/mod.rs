//! Shared synthesis types for audio asset generation.
//!
//! This module contains types shared between SFX and instrument audio recipes.

mod basic_types;
mod modulation;
mod synthesis_advanced;
mod synthesis_core;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve API
pub use basic_types::*;
pub use modulation::*;
pub use synthesis_advanced::*;
pub use synthesis_core::*;
