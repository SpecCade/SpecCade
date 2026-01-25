//! SpecCade Editor - Tauri plugin for the standalone editor.
//!
//! This crate provides the Rust backend for the SpecCade editor,
//! exposing IPC commands for spec evaluation, validation, and preview generation.

mod commands;
pub mod preview;
pub mod watcher;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

// Re-export the commands at crate level for Tauri's generate_handler!
pub use commands::eval::eval_spec;
pub use commands::generate::{generate_preview, refine_mesh_preview};
pub use commands::validate::validate_spec;
pub use watcher::{unwatch_file, watch_file, WatcherState};

/// Initialize the SpecCade Tauri plugin.
///
/// # Example
///
/// ```ignore
/// fn main() {
///     tauri::Builder::default()
///         .plugin(speccade_editor::init())
///         .run(tauri::generate_context!())
///         .expect("error running tauri");
/// }
/// ```
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("speccade")
        .invoke_handler(tauri::generate_handler![
            eval_spec,
            validate_spec,
            generate_preview,
            refine_mesh_preview,
            watch_file,
            unwatch_file,
        ])
        .setup(|app, _api| {
            app.manage(std::sync::Mutex::new(WatcherState::default()));
            Ok(())
        })
        .build()
}
