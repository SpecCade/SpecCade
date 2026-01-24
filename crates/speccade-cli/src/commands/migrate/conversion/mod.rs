//! Conversion logic for .studio to JSON migration
//!
//! Handles the actual migration of legacy specs to canonical JSON format.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use speccade_spec::{Recipe, Spec};

use super::audit::{classify_legacy_keys, KeyClassification};
use super::legacy_parser::{parse_legacy_spec_exec, parse_legacy_spec_static, LegacySpec};

mod audio;
mod helpers;
mod mesh;
mod music;
mod texture;

#[cfg(test)]
mod tests;

// Re-export public functions
pub use helpers::{
    extract_asset_id, generate_outputs, generate_seed_from_filename, map_category_to_type,
};

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

/// Map legacy keys to canonical params
pub fn map_legacy_keys_to_params(
    legacy: &LegacySpec,
    recipe_kind: &str,
) -> Result<(serde_json::Value, Vec<String>)> {
    match recipe_kind {
        "texture.procedural_v1" => texture::map_texture_params(&legacy.data, &legacy.category),
        "audio_v1" => audio::map_audio_params(&legacy.data, &legacy.dict_name),
        "music.tracker_song_v1" => music::map_music_params(&legacy.data),
        "static_mesh.blender_primitives_v1" => mesh::map_mesh_params(&legacy.data),
        // CHARACTER and ANIMATION are handled by MIGRATE-003/004
        "skeletal_mesh.blender_rigged_mesh_v1" | "skeletal_animation.blender_clip_v1" => {
            let warnings = vec![format!(
                "Category '{}' migration not yet implemented. Passing through legacy data.",
                legacy.category
            )];
            let mut params = legacy.data.clone();
            params.remove("name");
            Ok((serde_json::json!(params), warnings))
        }
        _ => {
            let warnings = vec![format!(
                "Unknown recipe kind '{}'. Passing through legacy data.",
                recipe_kind
            )];
            let mut params = legacy.data.clone();
            params.remove("name");
            Ok((serde_json::json!(params), warnings))
        }
    }
}
