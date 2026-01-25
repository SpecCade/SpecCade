//! Pack manifest generation for bundling assets.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackEntry {
    pub asset_id: String,
    pub asset_type: String,
    pub path: String,
    pub size_bytes: u64,
    pub spec_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub version: u32,
    pub name: String,
    pub entries: Vec<PackEntry>,
    pub total_size_bytes: u64,
}

#[tauri::command]
pub fn generate_pack_manifest(name: String, output_dir: String) -> Result<PackManifest, String> {
    let path = Path::new(&output_dir);
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", output_dir));
    }

    let mut entries = Vec::new();
    let mut total_size = 0u64;

    walk_dir(path, &mut entries, &mut total_size)?;

    Ok(PackManifest {
        version: 1,
        name,
        entries,
        total_size_bytes: total_size,
    })
}

fn walk_dir(dir: &Path, entries: &mut Vec<PackEntry>, total_size: &mut u64) -> Result<(), String> {
    let read_dir = std::fs::read_dir(dir).map_err(|e| e.to_string())?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_dir(&path, entries, total_size)?;
        } else {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let asset_type = match ext {
                "wav" | "ogg" => "audio",
                "xm" | "it" => "music",
                "png" | "jpg" => "texture",
                "glb" | "gltf" => "mesh",
                _ => continue,
            };

            let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
            let size = meta.len();
            *total_size += size;

            let name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let hash = blake3::hash(path.to_string_lossy().as_bytes())
                .to_hex()
                .to_string();

            entries.push(PackEntry {
                asset_id: name,
                asset_type: asset_type.to_string(),
                path: path.to_string_lossy().to_string(),
                size_bytes: size,
                spec_hash: hash[..16].to_string(),
            });
        }
    }
    Ok(())
}

#[tauri::command]
pub fn write_pack_manifest(manifest: PackManifest, output_path: String) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(&output_path, json).map_err(|e| e.to_string())
}
