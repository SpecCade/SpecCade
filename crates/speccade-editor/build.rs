// Keep in sync with:
// - crates/speccade-editor/src/lib.rs (generate_handler![...])
// - crates/speccade-editor/permissions/default.toml (default permission set)
const COMMANDS: &[&str] = &[
    "batch_generate",
    "eval_spec",
    "validate_spec",
    "generate_preview",
    "generate_full",
    "refine_mesh_preview",
    "list_golden_preview_textures",
    "get_golden_preview_texture_source",
    "read_binary_file_base64",
    "generate_png_output_base64",
    "generate_pack_manifest",
    "write_pack_manifest",
    "watch_file",
    "unwatch_file",
    "open_folder",
    "read_file",
    "save_file",
    "scan_project_tree",
    "list_templates",
    "get_template",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .try_build()
        .expect("failed to build tauri plugin permissions");
}
