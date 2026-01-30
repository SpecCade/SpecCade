//! Conversion logic for .studio to JSON migration
//!
//! Handles the actual migration of legacy specs to canonical JSON format.

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use speccade_spec::{Recipe, Spec};

use super::audit::{classify_legacy_keys, KeyClassification};
use super::legacy_parser::{parse_legacy_spec_exec, parse_legacy_spec_static, LegacySpec};

mod animation;
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
        "skeletal_animation.blender_clip_v1" => animation::map_animation_params(&legacy.data),
        "skeletal_mesh.armature_driven_v1" => {
            let mut warnings = vec![
                "Character migration is lossy: legacy parts/body_parts are approximated as bone_meshes".to_string(),
            ];

            let mut params = serde_json::Map::new();

            // Skeleton: prefer explicit preset, otherwise default.
            if let Some(preset) = legacy.data.get("skeleton_preset").and_then(|v| v.as_str()) {
                params.insert(
                    "skeleton_preset".to_string(),
                    serde_json::Value::String(preset.to_string()),
                );
            } else {
                params.insert(
                    "skeleton_preset".to_string(),
                    serde_json::Value::String("humanoid_basic_v1".to_string()),
                );
                warnings
                    .push("Missing skeleton_preset; defaulting to humanoid_basic_v1".to_string());
            }

            if let Some(skel) = legacy.data.get("skeleton") {
                if skel.is_array() {
                    params.insert("skeleton".to_string(), skel.clone());
                }
            }

            // Bone meshes: derive from legacy parts if possible.
            let mut bone_meshes = serde_json::Map::new();
            if let Some(parts) = legacy.data.get("parts").and_then(|v| v.as_object()) {
                // Build a helper map for resolving mirrored bones using the legacy skeleton.
                // Example: if skeleton contains { bone: "arm_upper_R", mirror: "arm_upper_L" },
                // we record arm_upper_L -> arm_upper_R.
                let mut mirror_bone_of: HashMap<String, String> = HashMap::new();
                if let Some(skel) = legacy.data.get("skeleton").and_then(|v| v.as_array()) {
                    for bone_val in skel {
                        let Some(bone_obj) = bone_val.as_object() else {
                            continue;
                        };
                        let Some(bone_name) = bone_obj.get("bone").and_then(|v| v.as_str()) else {
                            continue;
                        };
                        let Some(mirror_src) = bone_obj.get("mirror").and_then(|v| v.as_str())
                        else {
                            continue;
                        };
                        mirror_bone_of.insert(mirror_src.to_string(), bone_name.to_string());
                    }
                }

                for (part_name, part_val) in parts {
                    let Some(part_obj) = part_val.as_object() else {
                        continue;
                    };

                    // Mirror parts (e.g. {"mirror": "arm_L"}) should become a bone_meshes mirror
                    // entry for the mirrored *bone*, not the legacy part name.
                    if let Some(mirror_part) = part_obj.get("mirror").and_then(|v| v.as_str()) {
                        if let Some(src_part_obj) =
                            parts.get(mirror_part).and_then(|v| v.as_object())
                        {
                            let src_bone_name = src_part_obj
                                .get("bone")
                                .and_then(|v| v.as_str())
                                .unwrap_or(mirror_part);

                            let mirrored_bone_name = mirror_bone_of
                                .get(src_bone_name)
                                .cloned()
                                .or_else(|| {
                                    if let Some(stem) = src_bone_name.strip_suffix("_L") {
                                        Some(format!("{}_R", stem))
                                    } else if let Some(stem) = src_bone_name.strip_suffix("_l") {
                                        Some(format!("{}_r", stem))
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_else(|| part_name.to_string());

                            bone_meshes.insert(
                                mirrored_bone_name,
                                serde_json::json!({ "mirror": src_bone_name }),
                            );
                            continue;
                        }
                    }

                    let bone_name = part_obj
                        .get("bone")
                        .and_then(|v| v.as_str())
                        .unwrap_or(part_name);

                    let profile = part_obj
                        .get("base")
                        .and_then(|v| v.as_str())
                        .unwrap_or("circle(8)");

                    // Legacy base_radius is treated as absolute units.
                    let profile_radius = match part_obj.get("base_radius") {
                        Some(v) if v.is_number() => {
                            serde_json::json!({"absolute": v.as_f64().unwrap_or(0.05)})
                        }
                        Some(v) if v.is_array() => {
                            let arr = v.as_array().unwrap();
                            let a = arr.get(0).and_then(|x| x.as_f64()).unwrap_or(0.05);
                            let b = arr.get(1).and_then(|x| x.as_f64()).unwrap_or(a);
                            serde_json::json!({"absolute": (a + b) * 0.5})
                        }
                        _ => serde_json::json!({"absolute": 0.05}),
                    };

                    let mut mesh_obj = serde_json::Map::new();
                    mesh_obj.insert(
                        "profile".to_string(),
                        serde_json::Value::String(profile.to_string()),
                    );
                    mesh_obj.insert("profile_radius".to_string(), profile_radius);

                    if let Some(v) = part_obj.get("cap_start") {
                        mesh_obj.insert("cap_start".to_string(), v.clone());
                    }
                    if let Some(v) = part_obj.get("cap_end") {
                        mesh_obj.insert("cap_end".to_string(), v.clone());
                    }

                    bone_meshes.insert(bone_name.to_string(), serde_json::Value::Object(mesh_obj));
                }
            }

            if bone_meshes.is_empty() {
                warnings.push(
                    "No legacy parts found; using default bone_meshes for 'spine'".to_string(),
                );
                bone_meshes.insert(
                    "spine".to_string(),
                    serde_json::json!({
                        "profile": "circle(8)",
                        "profile_radius": {"absolute": 0.05}
                    }),
                );
            }

            params.insert(
                "bone_meshes".to_string(),
                serde_json::Value::Object(bone_meshes),
            );

            Ok((serde_json::Value::Object(params), warnings))
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
