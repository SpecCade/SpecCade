//! Conversion logic for .studio to JSON migration
//!
//! Handles the actual migration of legacy specs to canonical JSON format.

use anyhow::{bail, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

use speccade_spec::{AssetType, OutputFormat, OutputKind, OutputSpec, Recipe, Spec};

use super::audit::{classify_legacy_keys, KeyClassification};
use super::legacy_parser::{parse_legacy_spec_exec, parse_legacy_spec_static, LegacySpec};

/// Migration report entry
#[derive(Debug)]
pub struct MigrationEntry {
    pub source_path: PathBuf,
    pub target_path: Option<PathBuf>,
    pub success: bool,
    pub warnings: Vec<String>,
    pub error: Option<String>,
    /// Key classification for parity analysis
    pub key_classification: KeyClassification,
}

/// Migrate a single legacy spec file
pub fn migrate_spec(
    spec_file: &Path,
    project_root: &Path,
    allow_exec: bool,
) -> Result<MigrationEntry> {
    let mut warnings = Vec::new();

    // Parse the legacy spec
    let legacy = if allow_exec {
        parse_legacy_spec_exec(spec_file)?
    } else {
        parse_legacy_spec_static(spec_file)?
    };

    // Classify legacy keys against parity matrix
    let key_classification = classify_legacy_keys(&legacy);

    // Determine asset type and recipe kind from category
    let (asset_type, recipe_kind) = map_category_to_type(&legacy.category)?;

    // Extract asset_id from filename
    let asset_id = extract_asset_id(spec_file)?;

    // Generate seed from filename hash
    let seed = generate_seed_from_filename(spec_file);

    // Map legacy keys to canonical format
    let (params, mut key_warnings) = map_legacy_keys_to_params(&legacy, &recipe_kind)?;
    warnings.append(&mut key_warnings);

    // Determine output paths
    let outputs = generate_outputs(&asset_id, &asset_type, &legacy.category)?;

    // Build the canonical spec
    let mut spec = Spec::builder(asset_id.clone(), asset_type)
        .license("UNKNOWN")
        .seed(seed);

    // Add outputs
    for output in outputs {
        spec = spec.output(output);
    }

    // Add recipe with params
    spec = spec.recipe(Recipe::new(recipe_kind, params));

    // Add migration notes
    spec = spec.migration_notes({
        let mut notes = vec![
            "Migrated from legacy .spec.py format".to_string(),
            "Please review and update the license field".to_string(),
        ];
        if !warnings.is_empty() {
            notes.push("See migrator warnings for manual review items".to_string());
        }
        notes
    });

    let spec = spec.build();

    // Determine target path
    let target_path = project_root
        .join("specs")
        .join(asset_type.as_str())
        .join(format!("{}.json", asset_id));

    // Create directory if needed
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the spec
    let json = spec.to_json_pretty()?;
    fs::write(&target_path, json)?;

    Ok(MigrationEntry {
        source_path: spec_file.to_path_buf(),
        target_path: Some(target_path),
        success: true,
        warnings,
        error: None,
        key_classification,
    })
}

/// Map category to asset type and recipe kind
pub fn map_category_to_type(category: &str) -> Result<(AssetType, String)> {
    match category {
        "sounds" => Ok((AssetType::Audio, "audio_v1".to_string())),
        "instruments" => Ok((AssetType::Audio, "audio_v1".to_string())),
        "music" => Ok((AssetType::Music, "music.tracker_song_v1".to_string())),
        "textures" | "normals" => Ok((AssetType::Texture, "texture.procedural_v1".to_string())),
        "meshes" => Ok((
            AssetType::StaticMesh,
            "static_mesh.blender_primitives_v1".to_string(),
        )),
        "characters" => Ok((
            AssetType::SkeletalMesh,
            "skeletal_mesh.blender_rigged_mesh_v1".to_string(),
        )),
        "animations" => Ok((
            AssetType::SkeletalAnimation,
            "skeletal_animation.blender_clip_v1".to_string(),
        )),
        "fonts" => Ok((AssetType::Font, "font.bitmap_v1".to_string())),
        _ => bail!("Unknown category: {}", category),
    }
}

/// Extract asset_id from filename
pub fn extract_asset_id(spec_file: &Path) -> Result<String> {
    let file_stem = spec_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

    // Remove .spec suffix if present
    let asset_id = file_stem.strip_suffix(".spec").unwrap_or(file_stem);

    // Convert to lowercase and replace invalid characters
    let asset_id = asset_id.to_lowercase().replace("_", "-");

    // Validate format: [a-z][a-z0-9_-]{2,63}
    let re = Regex::new(r"^[a-z][a-z0-9_-]{2,63}$")?;
    if !re.is_match(&asset_id) {
        bail!("Invalid asset_id format: {}", asset_id);
    }

    Ok(asset_id)
}

/// Generate seed from filename hash
pub fn generate_seed_from_filename(spec_file: &Path) -> u32 {
    let filename = spec_file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let hash = blake3::hash(filename.as_bytes());
    let bytes = hash.as_bytes();

    // Take first 4 bytes and convert to u32
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// Map legacy keys to canonical params
pub fn map_legacy_keys_to_params(
    legacy: &LegacySpec,
    recipe_kind: &str,
) -> Result<(serde_json::Value, Vec<String>)> {
    let mut warnings = Vec::new();

    if recipe_kind == "texture.procedural_v1" {
        let data = &legacy.data;

        let resolution = match data.get("size").or_else(|| data.get("resolution")) {
            Some(serde_json::Value::Number(n)) => {
                let size = n.as_u64().unwrap_or(256) as u32;
                [size, size]
            }
            Some(serde_json::Value::Array(values)) if values.len() == 2 => {
                let w = values[0].as_u64().unwrap_or(256) as u32;
                let h = values[1].as_u64().unwrap_or(256) as u32;
                [w, h]
            }
            _ => [256, 256],
        };

        let tileable = data
            .get("tileable")
            .or_else(|| data.get("tile"))
            .or_else(|| data.get("seamless"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let nodes = if legacy.category == "normals" {
            serde_json::json!([
                { "id": "height", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.08 } },
                { "id": "normal", "type": "normal_from_height", "input": "height", "strength": 1.0 }
            ])
        } else {
            serde_json::json!([
                { "id": "height", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.08 } },
                { "id": "albedo", "type": "color_ramp", "input": "height", "ramp": ["#2b2b2b", "#cfcfcf"] }
            ])
        };

        warnings.push(
            "Legacy texture specs migrated to placeholder procedural graphs. Manual review recommended."
                .to_string(),
        );

        return Ok((
            serde_json::json!({
                "resolution": resolution,
                "tileable": tileable,
                "nodes": nodes
            }),
            warnings,
        ));
    }

    // For now, just pass through the legacy data as params
    // TODO: Map legacy keys to canonical params using docs/legacy/PARITY_MATRIX_LEGACY_SPEC_PY.md (SSOT for mapping rules).

    // Remove the 'name' field as it's used for asset_id
    let mut params = legacy.data.clone();
    params.remove("name");

    // Add warning for manual review
    if !params.is_empty() {
        warnings.push(format!(
            "Legacy params dict '{}' passed through for {}. Manual review recommended (TODO: key mapping per docs/legacy/PARITY_MATRIX_LEGACY_SPEC_PY.md).",
            legacy.dict_name, recipe_kind
        ));
    }

    Ok((serde_json::json!(params), warnings))
}

/// Generate output specs based on asset type
pub fn generate_outputs(
    asset_id: &str,
    asset_type: &AssetType,
    category: &str,
) -> Result<Vec<OutputSpec>> {
    let outputs = match asset_type {
        AssetType::Audio => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Wav,
                path: format!("audio/{}.wav", asset_id),
                source: None,
            }]
        }
        AssetType::Music => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Xm,
                path: format!("music/{}.xm", asset_id),
                source: None,
            }]
        }
        AssetType::Texture => {
            let source = if category == "normals" {
                Some("normal".to_string())
            } else {
                Some("albedo".to_string())
            };
            if category == "normals" {
                vec![OutputSpec {
                    kind: OutputKind::Primary,
                    format: OutputFormat::Png,
                    path: format!("textures/{}_normal.png", asset_id),
                    source,
                }]
            } else {
                vec![OutputSpec {
                    kind: OutputKind::Primary,
                    format: OutputFormat::Png,
                    path: format!("textures/{}.png", asset_id),
                    source,
                }]
            }
        }
        AssetType::StaticMesh => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Glb,
                path: format!("meshes/{}.glb", asset_id),
                source: None,
            }]
        }
        AssetType::SkeletalMesh => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Glb,
                path: format!("characters/{}.glb", asset_id),
                source: None,
            }]
        }
        AssetType::SkeletalAnimation => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Glb,
                path: format!("animations/{}.glb", asset_id),
                source: None,
            }]
        }
        AssetType::Sprite => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Png,
                path: format!("sprites/{}.png", asset_id),
                source: None,
            }]
        }
        AssetType::Vfx => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Png,
                path: format!("vfx/{}.png", asset_id),
                source: None,
            }]
        }
        AssetType::Ui => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Png,
                path: format!("ui/{}.png", asset_id),
                source: None,
            }]
        }
        AssetType::Font => {
            vec![
                OutputSpec {
                    kind: OutputKind::Primary,
                    format: OutputFormat::Png,
                    path: format!("fonts/{}_atlas.png", asset_id),
                    source: None,
                },
                OutputSpec {
                    kind: OutputKind::Metadata,
                    format: OutputFormat::Json,
                    path: format!("fonts/{}_metrics.json", asset_id),
                    source: None,
                },
            ]
        }
    };

    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_map_category_to_type() {
        let (asset_type, kind) = map_category_to_type("sounds").unwrap();
        assert_eq!(asset_type, AssetType::Audio);
        assert_eq!(kind, "audio_v1");

        let (asset_type, kind) = map_category_to_type("textures").unwrap();
        assert_eq!(asset_type, AssetType::Texture);
        assert_eq!(kind, "texture.procedural_v1");
    }

    #[test]
    fn test_extract_asset_id() {
        let id = extract_asset_id(Path::new("laser_blast_01.spec.py")).unwrap();
        assert_eq!(id, "laser-blast-01");

        assert!(
            extract_asset_id(Path::new("AB.spec.py")).is_err(),
            "too short"
        );
        assert!(
            extract_asset_id(Path::new("INVALID!.spec.py")).is_err(),
            "invalid chars"
        );
    }

    #[test]
    fn test_generate_seed_from_filename_is_deterministic() {
        let seed1 = generate_seed_from_filename(Path::new("a.spec.py"));
        let seed2 = generate_seed_from_filename(Path::new("a.spec.py"));
        let seed3 = generate_seed_from_filename(Path::new("b.spec.py"));

        assert_eq!(seed1, seed2);
        assert_ne!(seed1, seed3);
    }

    #[test]
    fn test_map_legacy_keys_to_params_removes_name_and_warns() {
        let legacy = LegacySpec {
            dict_name: "SOUND".to_string(),
            category: "sounds".to_string(),
            data: HashMap::from([
                ("name".to_string(), serde_json::json!("laser")),
                ("duration".to_string(), serde_json::json!(0.5)),
            ]),
        };

        let (params, warnings) = map_legacy_keys_to_params(&legacy, "audio_v1").unwrap();
        assert!(warnings
            .iter()
            .any(|w| w.contains("docs/legacy/PARITY_MATRIX_LEGACY_SPEC_PY.md")));
        assert!(params.get("name").is_none(), "name should be removed");
        assert!(params.get("duration").is_some());
    }

    #[test]
    fn test_generate_outputs_normals() {
        let outputs = generate_outputs("wall-01", &AssetType::Texture, "normals").unwrap();
        assert_eq!(outputs.len(), 1);
        assert!(outputs[0].path.ends_with("_normal.png"));
        assert_eq!(outputs[0].source.as_deref(), Some("normal"));
    }

    #[test]
    fn test_generate_outputs_textures() {
        let outputs = generate_outputs("brick-01", &AssetType::Texture, "textures").unwrap();
        assert_eq!(outputs.len(), 1);
        assert!(outputs[0].path.ends_with(".png"));
        assert_eq!(outputs[0].source.as_deref(), Some("albedo"));
    }
}
