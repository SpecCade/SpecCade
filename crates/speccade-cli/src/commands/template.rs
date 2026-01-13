//! Template (texture kit) command implementations.

use anyhow::{bail, Context, Result};
use speccade_spec::{AssetType, Spec};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const PACK_NAME: &str = "preset_library_v1";

fn find_packs_root_from(start: &Path) -> Result<PathBuf> {
    let mut dir = start.to_path_buf();

    loop {
        let candidate = dir.join("packs");
        if candidate.join(PACK_NAME).is_dir() {
            return Ok(candidate);
        }
        if !dir.pop() {
            break;
        }
    }

    bail!("Could not find 'packs/{}' in current directory ancestry", PACK_NAME)
}

fn find_packs_root() -> Result<PathBuf> {
    let dir = std::env::current_dir().context("Failed to read current directory")?;
    find_packs_root_from(&dir)
}

fn load_templates_from(start: &Path, asset_type: &str) -> Result<Vec<(Spec, PathBuf)>> {
    if asset_type != "texture" {
        bail!("Only asset_type 'texture' is supported for templates in this version");
    }

    let packs_root = find_packs_root_from(start)?;
    let templates_dir = packs_root.join(PACK_NAME).join(asset_type);
    if !templates_dir.is_dir() {
        bail!("Template directory not found: {}", templates_dir.display());
    }

    let mut templates = Vec::new();
    for entry in fs::read_dir(&templates_dir)
        .with_context(|| format!("Failed to read template directory: {}", templates_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read template: {}", path.display()))?;
        let spec: Spec = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse template: {}", path.display()))?;
        if spec.asset_type != AssetType::Texture {
            continue;
        }
        templates.push((spec, path));
    }

    templates.sort_by(|(a, _), (b, _)| a.asset_id.cmp(&b.asset_id));
    Ok(templates)
}

fn load_templates(asset_type: &str) -> Result<Vec<(Spec, PathBuf)>> {
    let dir = std::env::current_dir().context("Failed to read current directory")?;
    load_templates_from(&dir, asset_type)
}

fn find_template_by_id(asset_type: &str, template_id: &str) -> Result<(Spec, PathBuf)> {
    let templates = load_templates(asset_type)?;
    templates
        .into_iter()
        .find(|(spec, _)| spec.asset_id == template_id)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))
}

pub fn list(asset_type: &str) -> Result<ExitCode> {
    let templates = load_templates(asset_type)?;
    if templates.is_empty() {
        println!("No templates found for asset_type '{}'", asset_type);
        return Ok(ExitCode::SUCCESS);
    }

    for (spec, _) in templates {
        if let Some(desc) = spec.description.as_deref() {
            println!("{} - {}", spec.asset_id, desc);
        } else {
            println!("{}", spec.asset_id);
        }
    }

    Ok(ExitCode::SUCCESS)
}

pub fn show(asset_type: &str, template_id: &str) -> Result<ExitCode> {
    let (spec, path) = find_template_by_id(asset_type, template_id)?;

    println!("id: {}", spec.asset_id);
    println!("path: {}", path.display());
    if let Some(desc) = spec.description.as_deref() {
        println!("description: {}", desc);
    }

    Ok(ExitCode::SUCCESS)
}

pub fn copy(asset_type: &str, template_id: &str, dest: &Path) -> Result<ExitCode> {
    let (_, source_path) = find_template_by_id(asset_type, template_id)?;

    if dest.exists() {
        bail!("Destination already exists: {}", dest.display());
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    fs::copy(&source_path, dest)
        .with_context(|| format!("Failed to copy to {}", dest.display()))?;

    println!("Copied {} -> {}", source_path.display(), dest.display());

    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_template(dir: &Path, asset_id: &str, asset_type: &str) {
        let spec = serde_json::json!({
            "spec_version": 1,
            "asset_id": asset_id,
            "asset_type": asset_type,
            "license": "CC0-1.0",
            "seed": 1,
            "outputs": [
                { "kind": "primary", "format": "png", "path": "textures/out.png", "source": "out" }
            ],
            "recipe": {
                "kind": "texture.procedural_v1",
                "params": {
                    "resolution": [16, 16],
                    "tileable": true,
                    "nodes": [
                        { "id": "out", "type": "constant", "value": 0.5 }
                    ]
                }
            }
        });
        let path = dir.join(format!("{}.json", asset_id));
        fs::write(path, serde_json::to_string_pretty(&spec).unwrap()).unwrap();
    }

    #[test]
    fn load_templates_from_reads_texture_templates() {
        let tmp = tempfile::tempdir().unwrap();
        let templates_dir = tmp
            .path()
            .join("packs")
            .join(PACK_NAME)
            .join("texture");
        fs::create_dir_all(&templates_dir).unwrap();

        write_template(&templates_dir, "preset_texture_alpha", "texture");
        write_template(&templates_dir, "preset_texture_beta", "texture");
        write_template(&templates_dir, "not_a_texture", "audio");

        let templates = load_templates_from(tmp.path(), "texture").unwrap();
        let ids: Vec<String> = templates.into_iter().map(|(s, _)| s.asset_id).collect();
        assert_eq!(ids, vec!["preset_texture_alpha", "preset_texture_beta"]);
    }

    #[test]
    fn load_templates_from_rejects_other_asset_types() {
        let tmp = tempfile::tempdir().unwrap();
        let err = load_templates_from(tmp.path(), "audio").unwrap_err();
        assert!(err.to_string().contains("Only asset_type 'texture'"));
    }
}
