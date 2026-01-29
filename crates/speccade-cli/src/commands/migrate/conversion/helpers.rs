//! Helper functions for legacy spec conversion

use anyhow::{bail, Result};
use regex::Regex;
use serde_json::Value;
use std::path::Path;

use speccade_spec::{AssetType, OutputFormat, OutputKind, OutputSpec};

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
            "skeletal_mesh.armature_driven_v1".to_string(),
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

/// Create a default audio layer
pub fn create_default_layer() -> Value {
    serde_json::json!({
        "synthesis": {
            "type": "oscillator",
            "waveform": "sine",
            "frequency": 440.0
        },
        "envelope": {
            "attack": 0.01,
            "decay": 0.1,
            "sustain": 0.5,
            "release": 0.2
        },
        "volume": 1.0,
        "pan": 0.0
    })
}

/// Default envelope JSON value
pub fn default_envelope_json() -> Value {
    serde_json::json!({
        "attack": 0.01,
        "decay": 0.1,
        "sustain": 0.5,
        "release": 0.2
    })
}
