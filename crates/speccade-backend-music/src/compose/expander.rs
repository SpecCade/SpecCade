//! Core pattern expander implementation.
//!
//! This module has been split into separate files for better organization:
//! - expander_context: Expander struct and context resolution methods
//! - expander_eval: Pattern expression evaluation logic
//! - expander_time: Time expression evaluation logic

// Re-export public API.
pub(super) use crate::compose::expander_context::Expander;
