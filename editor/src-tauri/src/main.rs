//! SpecCade Editor - Tauri application entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .plugin(speccade_editor::init())
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
