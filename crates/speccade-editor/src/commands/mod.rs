//! IPC command handlers for the SpecCade editor.
//!
//! These commands are exposed to the frontend via Tauri's IPC mechanism.

pub mod batch;
pub mod batch_validate;
pub mod eval;
pub mod generate;
pub mod lint;
pub mod pack;
pub mod preview_textures;
pub mod project;
pub mod templates;
pub mod validate;
