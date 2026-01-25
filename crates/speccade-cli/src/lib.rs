//! SpecCade CLI library.
//!
//! This crate provides the core functionality for the SpecCade CLI,
//! including input loading, Starlark compilation, and asset generation commands.

pub mod analysis;
pub mod backends;
pub mod cache;
pub mod input;

#[cfg(feature = "starlark")]
pub mod compiler;

pub mod commands;
pub mod dispatch;
pub mod parity_data;
pub mod parity_matrix;
