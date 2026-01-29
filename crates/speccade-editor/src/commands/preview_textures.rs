//! Preview texture helpers for the editor.
//!
//! These commands support the mesh preview material layer by providing:
//! - Curated golden texture specs (embedded into the plugin)
//! - Binary PNG reads (for user-provided textures)
//! - "Generate and return a specific declared PNG output" for a spec

use base64::Engine;
use serde::{Deserialize, Serialize};
use speccade_cli::compiler::{self, CompilerConfig};
use speccade_spec::OutputFormat;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenPreviewTexture {
    pub id: String,
    pub label: String,
    /// Hint only (frontend should not enforce).
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenPreviewTextureSource {
    /// Filename hint used for compilation/import resolution.
    pub filename: String,
    /// Starlark source.
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryFileBase64 {
    pub base64: String,
    pub mime_type: String,
}

struct GoldenTextureDef {
    id: &'static str,
    label: &'static str,
    kind: &'static str,
    filename: &'static str,
    source: &'static str,
}

// Curated list.
//
// Note: These sources are embedded so packaged editor builds can use them
// without requiring a repo checkout.
const GOLDEN_TEXTURES: &[GoldenTextureDef] = &[
    GoldenTextureDef {
        id: "matcap_toon_basic",
        label: "Matcap: Toon Basic",
        kind: "matcap",
        filename: "specs:matcap_toon_basic.star",
        source: include_str!("../../../../specs/texture/texture_matcap.star"),
    },
    GoldenTextureDef {
        id: "tex_noise_mask",
        label: "2D: Noise Mask",
        kind: "2d",
        filename: "specs:tex_noise_mask.star",
        source: include_str!("../../../../specs/texture/texture_noise.star"),
    },
    GoldenTextureDef {
        id: "tex_patterns",
        label: "2D: Patterns",
        kind: "2d",
        filename: "specs:tex_patterns.star",
        source: include_str!("../../../../specs/texture/texture_patterns.star"),
    },
    GoldenTextureDef {
        id: "tex_normalmap",
        label: "2D: Normal Map",
        kind: "2d",
        filename: "specs:tex_normalmap.star",
        source: include_str!("../../../../specs/texture/texture_normalmap.star"),
    },
    GoldenTextureDef {
        id: "tex_colored",
        label: "2D: Colored",
        kind: "2d",
        filename: "specs:tex_colored.star",
        source: include_str!("../../../../specs/texture/texture_colored.star"),
    },
    GoldenTextureDef {
        id: "tex_material_preset",
        label: "2D: Material Preset (Albedo)",
        kind: "2d",
        filename: "specs:tex_material_preset.star",
        source: include_str!("../../../../specs/texture/texture_material_preset.star"),
    },
];

fn guess_mime_type(path: &str) -> &'static str {
    let p = path.to_ascii_lowercase();
    if p.ends_with(".png") {
        "image/png"
    } else if p.ends_with(".jpg") || p.ends_with(".jpeg") {
        "image/jpeg"
    } else {
        "application/octet-stream"
    }
}

#[tauri::command]
pub fn list_golden_preview_textures() -> Vec<GoldenPreviewTexture> {
    GOLDEN_TEXTURES
        .iter()
        .map(|t| GoldenPreviewTexture {
            id: t.id.to_string(),
            label: t.label.to_string(),
            kind: t.kind.to_string(),
        })
        .collect()
}

#[tauri::command]
pub fn get_golden_preview_texture_source(id: String) -> Result<GoldenPreviewTextureSource, String> {
    let def = GOLDEN_TEXTURES
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| format!("Unknown golden texture id: {}", id))?;

    Ok(GoldenPreviewTextureSource {
        filename: def.filename.to_string(),
        source: def.source.to_string(),
    })
}

#[tauri::command]
pub fn read_binary_file_base64(path: String) -> Result<BinaryFileBase64, String> {
    let bytes =
        std::fs::read(&path).map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
    Ok(BinaryFileBase64 {
        base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
        mime_type: guess_mime_type(&path).to_string(),
    })
}

#[tauri::command]
pub fn generate_png_output_base64(
    source: String,
    filename: String,
    output_path: String,
) -> Result<BinaryFileBase64, String> {
    use speccade_cli::dispatch::{dispatch_generate, DispatchError};

    let config = CompilerConfig::default();
    let spec = compiler::compile(&filename, &source, &config)
        .map_err(|e| format!("Compile failed: {}", e))?
        .spec;

    let declared_png = spec
        .outputs
        .iter()
        .any(|o| o.path == output_path && o.format == OutputFormat::Png);

    if !declared_png {
        return Err(format!(
            "Selected output '{}' is not a declared PNG output",
            output_path
        ));
    }

    let tmp = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let out_root = tmp.path();

    dispatch_generate(
        &spec,
        out_root.to_str().unwrap_or("."),
        std::path::Path::new(&filename),
        None,
    )
    .map_err(|e| {
        let msg = match &e {
            DispatchError::NoRecipe => "No recipe defined in the spec".to_string(),
            DispatchError::BackendNotImplemented(kind) => {
                format!("Backend not implemented for recipe kind: {}", kind)
            }
            DispatchError::BackendError(msg) => format!("Backend error: {}", msg),
        };
        format!("Generation failed: {}", msg)
    })?;

    let full_path = out_root.join(&output_path);
    let bytes = std::fs::read(&full_path)
        .map_err(|e| format!("Failed to read generated output '{}': {}", output_path, e))?;

    Ok(BinaryFileBase64 {
        base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
        mime_type: "image/png".to_string(),
    })
}
