//! Project management commands for the editor.
//!
//! These commands handle file operations like opening folders,
//! reading files, and saving files.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A file or directory entry in the project tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Full path to the file or directory
    pub path: String,
    /// Display name (filename without path)
    pub name: String,
    /// Whether this entry is a directory
    pub is_dir: bool,
    /// Detected asset type (audio, mesh, texture, etc.) for spec files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<String>,
}

/// A node in the project file tree (recursive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    /// Display name (filename without path)
    pub name: String,
    /// Path relative to the project root
    pub path: String,
    /// Whether this node is a directory
    pub is_dir: bool,
    /// Detected asset type for spec files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<String>,
    /// Child nodes (only for directories)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
}

/// Detect the asset type from a spec file's content.
///
/// Checks for patterns like:
/// - `asset_type = "audio"` or `"asset_type": "audio"` -> Some("audio")
/// - `music_spec(` -> Some("music")
/// - `skeletal_mesh_spec(` -> Some("skeletal_mesh")
/// - `skeletal_animation_spec(` -> Some("skeletal_animation")
/// - etc.
fn detect_asset_type(path: &PathBuf) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;

    // Check for Starlark function patterns first (more specific)
    if content.contains("music_spec(") {
        return Some("music".to_string());
    }
    if content.contains("skeletal_mesh_spec(") {
        return Some("skeletal_mesh".to_string());
    }
    if content.contains("skeletal_animation_spec(") {
        return Some("skeletal_animation".to_string());
    }
    if content.contains("audio_spec(") {
        return Some("audio".to_string());
    }
    if content.contains("texture_spec(") {
        return Some("texture".to_string());
    }
    if content.contains("mesh_spec(") {
        return Some("mesh".to_string());
    }
    if content.contains("font_spec(") {
        return Some("font".to_string());
    }

    // Check for JSON-style patterns: "asset_type": "..."
    // Use regex-like matching with simple string searches
    for asset_type in &[
        "audio",
        "music",
        "texture",
        "mesh",
        "skeletal_mesh",
        "skeletal_animation",
        "font",
    ] {
        // JSON format: "asset_type": "audio"
        let json_pattern = format!(r#""asset_type": "{}""#, asset_type);
        if content.contains(&json_pattern) {
            return Some(asset_type.to_string());
        }

        // JSON format with no space: "asset_type":"audio"
        let json_pattern_no_space = format!(r#""asset_type":"{}""#, asset_type);
        if content.contains(&json_pattern_no_space) {
            return Some(asset_type.to_string());
        }

        // Starlark format: asset_type = "audio"
        let star_pattern = format!(r#"asset_type = "{}""#, asset_type);
        if content.contains(&star_pattern) {
            return Some(asset_type.to_string());
        }
    }

    None
}

/// Maximum recursion depth for project tree scanning.
const MAX_SCAN_DEPTH: usize = 100;

/// Recursively scan a project directory and return a tree of file nodes.
///
/// Returns only .star and .json files plus directories that contain them.
/// Hidden files/directories (starting with .) are excluded.
/// Symlinks are not followed to prevent loops and path traversal.
#[tauri::command]
pub fn scan_project_tree(path: String) -> Result<Vec<FileNode>, String> {
    let root = PathBuf::from(&path);

    if !root.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !root.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    fn scan_dir(dir: &std::path::Path, root: &std::path::Path, depth: usize) -> Vec<FileNode> {
        if depth >= MAX_SCAN_DEPTH {
            return Vec::new();
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        let mut dirs: Vec<FileNode> = Vec::new();
        let mut files: Vec<FileNode> = Vec::new();

        for entry in entries.flatten() {
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            // Skip symlinks to prevent loops and path traversal
            let metadata = match fs::symlink_metadata(&entry_path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if metadata.is_symlink() {
                continue;
            }

            let rel_path = match entry_path.strip_prefix(root) {
                Ok(p) => p.to_string_lossy().replace('\\', "/"),
                Err(_) => continue, // Outside root — skip
            };

            if metadata.is_dir() {
                let children = scan_dir(&entry_path, root, depth + 1);
                // Only include directories that contain spec files (directly or nested)
                if !children.is_empty() {
                    dirs.push(FileNode {
                        name,
                        path: rel_path,
                        is_dir: true,
                        asset_type: None,
                        children: Some(children),
                    });
                }
            } else {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if ext == "star" || ext == "json" {
                    let asset_type = detect_asset_type(&entry_path);
                    files.push(FileNode {
                        name,
                        path: rel_path,
                        is_dir: false,
                        asset_type,
                        children: None,
                    });
                }
            }
        }

        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        dirs.extend(files);
        dirs
    }

    Ok(scan_dir(&root, &root, 0))
}

/// Open a folder and list its contents.
///
/// Returns only .star, .json files and directories.
/// Directories are sorted first, then files alphabetically.
#[tauri::command]
pub fn open_folder(path: String) -> Result<Vec<FileEntry>, String> {
    let dir_path = PathBuf::from(&path);

    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let entries =
        fs::read_dir(&dir_path).map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut dirs: Vec<FileEntry> = Vec::new();
    let mut files: Vec<FileEntry> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/directories (starting with .)
        if name.starts_with('.') {
            continue;
        }

        if entry_path.is_dir() {
            dirs.push(FileEntry {
                path: entry_path.to_string_lossy().to_string(),
                name,
                is_dir: true,
                asset_type: None,
            });
        } else {
            // Only include .star and .json files
            let extension = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            if extension == "star" || extension == "json" {
                let asset_type = detect_asset_type(&entry_path);
                files.push(FileEntry {
                    path: entry_path.to_string_lossy().to_string(),
                    name,
                    is_dir: false,
                    asset_type,
                });
            }
        }
    }

    // Sort directories and files alphabetically (case-insensitive)
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Combine: directories first, then files
    dirs.extend(files);
    Ok(dirs)
}

/// Read a file's content as a string.
#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    let file_path = PathBuf::from(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if !file_path.is_file() {
        return Err(format!("Path is not a file: {}", path));
    }

    fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Save content to a file.
#[tauri::command]
pub fn save_file(path: String, content: String) -> Result<(), String> {
    let file_path = PathBuf::from(&path);

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directories: {}", e))?;
        }
    }

    fs::write(&file_path, &content).map_err(|e| format!("Failed to write file: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_open_folder_lists_files() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test files
        fs::write(dir_path.join("test.star"), "spec()").unwrap();
        fs::write(dir_path.join("config.json"), "{}").unwrap();
        fs::write(dir_path.join("readme.txt"), "ignored").unwrap();
        fs::create_dir(dir_path.join("subdir")).unwrap();

        let result = open_folder(dir_path.to_string_lossy().to_string()).unwrap();

        // Should have subdir, config.json, test.star (directories first)
        assert_eq!(result.len(), 3);
        assert!(result[0].is_dir);
        assert_eq!(result[0].name, "subdir");
        assert!(!result[1].is_dir);
        assert!(!result[2].is_dir);
    }

    #[test]
    fn test_open_folder_sorts_alphabetically() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        fs::write(dir_path.join("zebra.star"), "").unwrap();
        fs::write(dir_path.join("alpha.star"), "").unwrap();
        fs::write(dir_path.join("beta.json"), "").unwrap();
        fs::create_dir(dir_path.join("zdir")).unwrap();
        fs::create_dir(dir_path.join("adir")).unwrap();

        let result = open_folder(dir_path.to_string_lossy().to_string()).unwrap();

        // Directories first (sorted), then files (sorted)
        assert_eq!(result[0].name, "adir");
        assert_eq!(result[1].name, "zdir");
        assert_eq!(result[2].name, "alpha.star");
        assert_eq!(result[3].name, "beta.json");
        assert_eq!(result[4].name, "zebra.star");
    }

    #[test]
    fn test_open_folder_skips_hidden() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        fs::write(dir_path.join(".hidden.star"), "").unwrap();
        fs::write(dir_path.join("visible.star"), "").unwrap();

        let result = open_folder(dir_path.to_string_lossy().to_string()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "visible.star");
    }

    #[test]
    fn test_open_folder_nonexistent() {
        let result = open_folder("/nonexistent/path".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.star");
        fs::write(&file_path, "spec_content").unwrap();

        let result = read_file(file_path.to_string_lossy().to_string()).unwrap();
        assert_eq!(result, "spec_content");
    }

    #[test]
    fn test_read_file_nonexistent() {
        let result = read_file("/nonexistent/file.star".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_save_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new_file.star");

        save_file(
            file_path.to_string_lossy().to_string(),
            "new content".to_string(),
        )
        .unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "new content");
    }

    #[test]
    fn test_save_file_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nested").join("dir").join("file.star");

        save_file(
            file_path.to_string_lossy().to_string(),
            "nested content".to_string(),
        )
        .unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "nested content");
    }

    #[test]
    fn test_detect_asset_type_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("audio.json");
        fs::write(&file_path, r#"{"asset_type": "audio"}"#).unwrap();

        let result = detect_asset_type(&file_path);
        assert_eq!(result, Some("audio".to_string()));
    }

    #[test]
    fn test_detect_asset_type_starlark_format() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("texture.star");
        fs::write(&file_path, r#"asset_type = "texture""#).unwrap();

        let result = detect_asset_type(&file_path);
        assert_eq!(result, Some("texture".to_string()));
    }

    #[test]
    fn test_detect_asset_type_function_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("music.star");
        fs::write(&file_path, "music_spec(\n  title = \"Song\"\n)").unwrap();

        let result = detect_asset_type(&file_path);
        assert_eq!(result, Some("music".to_string()));
    }

    #[test]
    fn test_detect_asset_type_skeletal_mesh() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("char.star");
        fs::write(&file_path, "skeletal_mesh_spec()").unwrap();

        let result = detect_asset_type(&file_path);
        assert_eq!(result, Some("skeletal_mesh".to_string()));
    }

    #[test]
    fn test_detect_asset_type_no_match() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("unknown.star");
        fs::write(&file_path, "some random content").unwrap();

        let result = detect_asset_type(&file_path);
        assert_eq!(result, None);
    }

    #[test]
    fn test_scan_project_tree_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create nested structure
        fs::create_dir_all(root.join("audio")).unwrap();
        fs::create_dir_all(root.join("textures/walls")).unwrap();
        fs::write(root.join("audio/sfx.star"), "audio_spec()").unwrap();
        fs::write(root.join("textures/walls/brick.star"), r#"texture_spec()"#).unwrap();
        fs::write(root.join("root.star"), "spec()").unwrap();
        // Empty dir should be excluded
        fs::create_dir(root.join("empty")).unwrap();
        // Hidden dir should be excluded
        fs::create_dir(root.join(".hidden")).unwrap();
        fs::write(root.join(".hidden/secret.star"), "").unwrap();

        let result = scan_project_tree(root.to_string_lossy().to_string()).unwrap();

        // Should have: audio/ (dir), textures/ (dir), root.star (file)
        assert_eq!(result.len(), 3);
        // Dirs first, sorted
        assert_eq!(result[0].name, "audio");
        assert!(result[0].is_dir);
        assert_eq!(result[0].children.as_ref().unwrap().len(), 1);
        assert_eq!(result[0].children.as_ref().unwrap()[0].name, "sfx.star");
        assert_eq!(result[0].children.as_ref().unwrap()[0].path, "audio/sfx.star");
        assert_eq!(
            result[0].children.as_ref().unwrap()[0].asset_type,
            Some("audio".to_string())
        );

        assert_eq!(result[1].name, "textures");
        assert!(result[1].is_dir);
        let walls = &result[1].children.as_ref().unwrap()[0];
        assert_eq!(walls.name, "walls");
        assert_eq!(walls.children.as_ref().unwrap()[0].name, "brick.star");

        assert_eq!(result[2].name, "root.star");
        assert!(!result[2].is_dir);
        assert!(result[2].children.is_none());
    }

    #[test]
    fn test_scan_project_tree_uses_forward_slashes() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::create_dir(root.join("sub")).unwrap();
        fs::write(root.join("sub/file.star"), "").unwrap();

        let result = scan_project_tree(root.to_string_lossy().to_string()).unwrap();
        let sub = &result[0];
        assert_eq!(sub.path, "sub");
        assert_eq!(sub.children.as_ref().unwrap()[0].path, "sub/file.star");
    }

    #[test]
    fn test_open_folder_detects_asset_types() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        fs::write(
            dir_path.join("audio.star"),
            r#"audio_spec(asset_id = "sfx")"#,
        )
        .unwrap();
        fs::write(dir_path.join("mesh.json"), r#"{"asset_type": "mesh"}"#).unwrap();

        let result = open_folder(dir_path.to_string_lossy().to_string()).unwrap();

        assert_eq!(result.len(), 2);
        // Files sorted alphabetically
        assert_eq!(result[0].name, "audio.star");
        assert_eq!(result[0].asset_type, Some("audio".to_string()));
        assert_eq!(result[1].name, "mesh.json");
        assert_eq!(result[1].asset_type, Some("mesh".to_string()));
    }
}
