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
pub use commands::batch::batch_generate;
pub use commands::batch_validate::batch_validate;
pub use commands::eval::eval_spec;
pub use commands::generate::{generate_full, generate_preview, refine_mesh_preview};
pub use commands::pack::{generate_pack_manifest, write_pack_manifest};
pub use commands::preview_textures::{
    generate_png_output_base64, get_golden_preview_texture_source, list_golden_preview_textures,
    read_binary_file_base64,
};
pub use commands::project::{open_folder, read_file, save_file, scan_project_tree};
pub use commands::templates::{get_template, list_templates};
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
            // Keep in sync with:
            // - crates/speccade-editor/build.rs (COMMANDS)
            // - crates/speccade-editor/permissions/default.toml (default permission set)
            batch_generate,
            batch_validate,
            eval_spec,
            validate_spec,
            generate_preview,
            generate_full,
            refine_mesh_preview,
            list_golden_preview_textures,
            get_golden_preview_texture_source,
            read_binary_file_base64,
            generate_png_output_base64,
            generate_pack_manifest,
            write_pack_manifest,
            watch_file,
            unwatch_file,
            open_folder,
            read_file,
            save_file,
            scan_project_tree,
            list_templates,
            get_template,
        ])
        .setup(|app, _api| {
            app.manage(std::sync::Mutex::new(WatcherState::default()));
            Ok(())
        })
        .build()
}
