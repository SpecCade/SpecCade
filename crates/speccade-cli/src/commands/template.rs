//! Template (preset/kit) command implementations.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
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

    bail!(
        "Could not find 'packs/{}' in current directory ancestry",
        PACK_NAME
    )
}

fn _find_packs_root() -> Result<PathBuf> {
    let dir = std::env::current_dir().context("Failed to read current directory")?;
    find_packs_root_from(&dir)
}

/// Template entry for JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateEntry {
    pub asset_id: String,
    pub asset_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kit_name: Option<String>,
}

/// Music kit metadata (lightweight, non-Spec format).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MusicKitMetadata {
    spec_version: u32,
    asset_id: String,
    asset_type: String,
    license: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    kit: KitData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KitData {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
}

#[derive(Debug)]
enum Template {
    Spec(Spec),
    MusicKit(MusicKitMetadata),
}

fn load_templates_from(start: &Path, asset_type: &str) -> Result<Vec<(Template, PathBuf)>> {
    let packs_root = find_packs_root_from(start)?;
    let base_dir = packs_root.join(PACK_NAME);

    let mut templates = Vec::new();

    match asset_type {
        "texture" => {
            let templates_dir = base_dir.join("texture");
            if !templates_dir.is_dir() {
                bail!("Template directory not found: {}", templates_dir.display());
            }

            collect_templates_from_dir(&templates_dir, AssetType::Texture, &mut templates)?;
        }
        "audio" => {
            let audio_dir = base_dir.join("audio");
            if !audio_dir.is_dir() {
                bail!("Audio preset directory not found: {}", audio_dir.display());
            }

            // Recursively walk subdirectories (bass, drums/kicks, etc.)
            walk_audio_presets(&audio_dir, &mut templates)?;
        }
        "music" => {
            let music_dir = base_dir.join("music");
            if !music_dir.is_dir() {
                bail!("Music kit directory not found: {}", music_dir.display());
            }

            // Music kits are in a flat directory
            for entry in fs::read_dir(&music_dir).with_context(|| {
                format!("Failed to read music directory: {}", music_dir.display())
            })? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }

                // Skip report files
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".report.json"))
                    .unwrap_or(false)
                {
                    continue;
                }

                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read music file: {}", path.display()))?;

                // Try parsing as music_kit first (they don't follow standard Spec format)
                if let Ok(kit) = serde_json::from_str::<MusicKitMetadata>(&content) {
                    if kit.asset_type == "music_kit" {
                        templates.push((Template::MusicKit(kit), path));
                    }
                } else if let Ok(spec) = serde_json::from_str::<Spec>(&content) {
                    // Also include regular music specs
                    if spec.asset_type == AssetType::Music {
                        templates.push((Template::Spec(spec), path));
                    }
                }
            }
        }
        _ => {
            bail!(
                "Unsupported asset_type '{}'. Supported types: texture, audio, music",
                asset_type
            );
        }
    }

    templates.sort_by(|(a, _), (b, _)| match (a, b) {
        (Template::Spec(a), Template::Spec(b)) => a.asset_id.cmp(&b.asset_id),
        (Template::MusicKit(a), Template::MusicKit(b)) => a.asset_id.cmp(&b.asset_id),
        (Template::Spec(_), Template::MusicKit(_)) => std::cmp::Ordering::Less,
        (Template::MusicKit(_), Template::Spec(_)) => std::cmp::Ordering::Greater,
    });
    Ok(templates)
}

fn walk_audio_presets(dir: &Path, templates: &mut Vec<(Template, PathBuf)>) -> Result<()> {
    for entry in fs::read_dir(dir)
        .with_context(|| format!("Failed to read audio directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            walk_audio_presets(&path, templates)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            // Skip report files
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".report.json"))
                .unwrap_or(false)
            {
                continue;
            }

            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read audio preset: {}", path.display()))?;
            let spec: Spec = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse audio preset: {}", path.display()))?;

            if spec.asset_type == AssetType::Audio {
                templates.push((Template::Spec(spec), path));
            }
        }
    }
    Ok(())
}

fn collect_templates_from_dir(
    dir: &Path,
    expected_type: AssetType,
    templates: &mut Vec<(Template, PathBuf)>,
) -> Result<()> {
    for entry in
        fs::read_dir(dir).with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // Skip report files
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".report.json"))
            .unwrap_or(false)
        {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read template: {}", path.display()))?;
        let spec: Spec = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse template: {}", path.display()))?;

        if spec.asset_type == expected_type {
            templates.push((Template::Spec(spec), path));
        }
    }
    Ok(())
}

fn load_templates(asset_type: &str) -> Result<Vec<(Template, PathBuf)>> {
    let dir = std::env::current_dir().context("Failed to read current directory")?;
    load_templates_from(&dir, asset_type)
}

fn find_template_by_id(asset_type: &str, template_id: &str) -> Result<(Template, PathBuf)> {
    let templates = load_templates(asset_type)?;
    templates
        .into_iter()
        .find(|(tmpl, _)| match tmpl {
            Template::Spec(s) => s.asset_id == template_id,
            Template::MusicKit(k) => k.asset_id == template_id,
        })
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))
}

fn template_to_entry(tmpl: &Template) -> TemplateEntry {
    match tmpl {
        Template::Spec(spec) => TemplateEntry {
            asset_id: spec.asset_id.clone(),
            asset_type: spec.asset_type.as_str().to_string(),
            description: spec.description.clone(),
            style_tags: spec.style_tags.clone(),
            kit_name: None,
        },
        Template::MusicKit(kit) => TemplateEntry {
            asset_id: kit.asset_id.clone(),
            asset_type: "music".to_string(),
            description: kit.description.clone(),
            style_tags: kit.kit.tags.clone(),
            kit_name: Some(kit.kit.name.clone()),
        },
    }
}

pub fn list(asset_type: &str, json: bool) -> Result<ExitCode> {
    let templates = load_templates(asset_type)?;
    if templates.is_empty() {
        if !json {
            println!("No templates found for asset_type '{}'", asset_type);
        } else {
            println!("[]");
        }
        return Ok(ExitCode::SUCCESS);
    }

    if json {
        let entries: Vec<TemplateEntry> = templates
            .iter()
            .map(|(tmpl, _)| template_to_entry(tmpl))
            .collect();
        let output = serde_json::to_string_pretty(&entries)?;
        println!("{}", output);
    } else {
        for (tmpl, _) in templates {
            let entry = template_to_entry(&tmpl);
            if let Some(desc) = entry.description.as_deref() {
                println!("{} - {}", entry.asset_id, desc);
            } else {
                println!("{}", entry.asset_id);
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}

pub fn show(asset_type: &str, template_id: &str) -> Result<ExitCode> {
    let (tmpl, path) = find_template_by_id(asset_type, template_id)?;
    let entry = template_to_entry(&tmpl);

    println!("id: {}", entry.asset_id);
    println!("path: {}", path.display());
    if let Some(desc) = entry.description.as_deref() {
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

pub fn search(
    tags: Option<Vec<String>>,
    query: Option<String>,
    asset_type_filter: Option<String>,
    json: bool,
) -> Result<ExitCode> {
    // Load templates from all asset types if no filter specified
    let asset_types = if let Some(ref filter) = asset_type_filter {
        vec![filter.as_str()]
    } else {
        vec!["texture", "audio", "music"]
    };

    let mut all_templates = Vec::new();
    for asset_type in asset_types {
        // Silently skip types that don't exist
        if let Ok(templates) = load_templates(asset_type) {
            all_templates.extend(templates);
        }
    }

    // Filter by tags and/or query
    let mut results: Vec<(Template, PathBuf)> = all_templates
        .into_iter()
        .filter(|(tmpl, _)| {
            let entry = template_to_entry(tmpl);

            // Check tag filter
            let tags_match = if let Some(ref tag_filter) = tags {
                if let Some(ref tmpl_tags) = entry.style_tags {
                    tag_filter.iter().any(|filter_tag| {
                        tmpl_tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(&filter_tag.to_lowercase()))
                    })
                } else {
                    false
                }
            } else {
                true
            };

            // Check query filter
            let query_match = if let Some(ref q) = query {
                let q_lower = q.to_lowercase();
                entry.asset_id.to_lowercase().contains(&q_lower)
                    || entry
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&q_lower))
                        .unwrap_or(false)
            } else {
                true
            };

            tags_match && query_match
        })
        .collect();

    // Sort deterministically
    results.sort_by(|(a, _), (b, _)| {
        let entry_a = template_to_entry(a);
        let entry_b = template_to_entry(b);
        entry_a.asset_id.cmp(&entry_b.asset_id)
    });

    if json {
        let entries: Vec<TemplateEntry> = results
            .iter()
            .map(|(tmpl, _)| template_to_entry(tmpl))
            .collect();
        let output = serde_json::to_string_pretty(&entries)?;
        println!("{}", output);
    } else if results.is_empty() {
        println!("No matching templates found");
    } else {
        for (tmpl, _) in results {
            let entry = template_to_entry(&tmpl);
            let tags_str = entry
                .style_tags
                .as_ref()
                .map(|t| format!(" [{}]", t.join(", ")))
                .unwrap_or_default();

            if let Some(desc) = entry.description.as_deref() {
                println!("{} - {}{}", entry.asset_id, desc, tags_str);
            } else {
                println!("{}{}", entry.asset_id, tags_str);
            }
        }
    }

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
        let templates_dir = tmp.path().join("packs").join(PACK_NAME).join("texture");
        fs::create_dir_all(&templates_dir).unwrap();

        write_template(&templates_dir, "preset_texture_alpha", "texture");
        write_template(&templates_dir, "preset_texture_beta", "texture");
        write_template(&templates_dir, "not_a_texture", "audio");

        let templates = load_templates_from(tmp.path(), "texture").unwrap();
        let ids: Vec<String> = templates
            .into_iter()
            .map(|(t, _)| match t {
                Template::Spec(s) => s.asset_id,
                Template::MusicKit(k) => k.asset_id,
            })
            .collect();
        assert_eq!(ids, vec!["preset_texture_alpha", "preset_texture_beta"]);
    }

    #[test]
    fn load_templates_from_supports_audio() {
        let tmp = tempfile::tempdir().unwrap();
        let audio_dir = tmp
            .path()
            .join("packs")
            .join(PACK_NAME)
            .join("audio")
            .join("bass");
        fs::create_dir_all(&audio_dir).unwrap();

        let spec = serde_json::json!({
            "spec_version": 1,
            "asset_id": "preset_bass_test",
            "asset_type": "audio",
            "license": "CC0-1.0",
            "seed": 1,
            "outputs": [
                { "kind": "primary", "format": "wav", "path": "audio/bass_test.wav" }
            ],
        });
        let path = audio_dir.join("preset_bass_test.json");
        fs::write(path, serde_json::to_string_pretty(&spec).unwrap()).unwrap();

        let templates = load_templates_from(tmp.path(), "audio").unwrap();
        assert_eq!(templates.len(), 1);
        let entry = template_to_entry(&templates[0].0);
        assert_eq!(entry.asset_id, "preset_bass_test");
    }

    #[test]
    fn load_templates_from_supports_music() {
        let tmp = tempfile::tempdir().unwrap();
        let music_dir = tmp.path().join("packs").join(PACK_NAME).join("music");
        fs::create_dir_all(&music_dir).unwrap();

        let spec = serde_json::json!({
            "spec_version": 1,
            "asset_id": "kit_test",
            "asset_type": "music_kit",
            "license": "CC0-1.0",
            "description": "Test kit",
            "kit": {
                "name": "Test Kit",
                "tags": ["test"]
            }
        });
        let path = music_dir.join("kit_test.json");
        fs::write(path, serde_json::to_string_pretty(&spec).unwrap()).unwrap();

        let templates = load_templates_from(tmp.path(), "music").unwrap();
        assert_eq!(templates.len(), 1);
        let entry = template_to_entry(&templates[0].0);
        assert_eq!(entry.asset_id, "kit_test");
    }

    #[test]
    fn load_templates_from_rejects_unsupported_types() {
        let tmp = tempfile::tempdir().unwrap();
        // Need to create the packs directory even though the asset type won't exist
        let packs_dir = tmp.path().join("packs").join(PACK_NAME);
        fs::create_dir_all(&packs_dir).unwrap();

        let err = load_templates_from(tmp.path(), "unsupported").unwrap_err();
        assert!(err.to_string().contains("Unsupported asset_type"));
    }
}
