//! Main entry point for texture generation.
//!
//! This module provides the high-level API for procedural graphs and legacy
//! PBR material map helpers.

mod albedo;
mod color_utils;
mod emissive;
mod graph;
mod helpers;
mod layers;
mod masks;
mod materials;
mod metallic;
mod packed;
mod roughness;
mod simple_maps;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use speccade_spec::recipe::texture::{TextureMapType, TextureMaterialV1Params};
use speccade_spec::BackendError;

use crate::color::Color;
use crate::maps::GrayscaleBuffer;
use crate::rng::DeterministicRng;

pub use graph::{encode_graph_value_png, generate_graph, GraphValue};
use helpers::{
    get_default_metallic, get_default_roughness_range, validate_base_material, validate_map_list,
    validate_resolution,
};
use layers::apply_layer_to_height;
use materials::apply_material_pattern;
pub use packed::generate_packed_maps;

// Re-export map generators
use albedo::generate_albedo_map;
use emissive::generate_emissive_map;
use metallic::generate_metallic_map;
use roughness::generate_roughness_map;
use simple_maps::{generate_ao_map, generate_height_output, generate_normal_map};

/// Errors from texture generation.
#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("PNG error: {0}")]
    Png(#[from] crate::png::PngError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

impl BackendError for GenerateError {
    fn code(&self) -> &'static str {
        match self {
            GenerateError::Png(_) => "TEXTURE_001",
            GenerateError::Io(_) => "TEXTURE_002",
            GenerateError::InvalidParameter(_) => "TEXTURE_003",
        }
    }

    fn category(&self) -> &'static str {
        "texture"
    }
}

/// Result of generating a texture set.
#[derive(Debug)]
pub struct TextureResult {
    /// Generated map buffers keyed by map type.
    pub maps: HashMap<TextureMapType, MapResult>,
}

/// Result of generating a single map.
#[derive(Debug)]
pub struct MapResult {
    /// The map type.
    pub map_type: TextureMapType,
    /// The generated texture data (RGBA for color maps, grayscale for others).
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// BLAKE3 hash of the PNG file.
    pub hash: String,
    /// Whether this is a color (RGB/RGBA) or grayscale map.
    pub is_color: bool,
}

/// Generate PBR material maps from parameters.
pub fn generate_material_maps(
    params: &TextureMaterialV1Params,
    seed: u32,
) -> Result<TextureResult, GenerateError> {
    let width = params.resolution[0];
    let height = params.resolution[1];

    validate_resolution(width, height)?;
    validate_map_list(&params.maps)?;
    validate_base_material(params)?;

    let mut results = HashMap::new();

    // Get base material properties
    let (base_color, roughness_range, metallic) = match &params.base_material {
        Some(mat) => {
            let r_range = mat
                .roughness_range
                .unwrap_or(get_default_roughness_range(&mat.material_type));
            let m = mat
                .metallic
                .unwrap_or(get_default_metallic(&mat.material_type));
            (
                Color::rgb(mat.base_color[0], mat.base_color[1], mat.base_color[2]),
                r_range,
                m,
            )
        }
        None => (Color::gray(0.5), [0.3, 0.7], 0.0),
    };

    // Generate height map first (used by multiple map types)
    let height_map = generate_height_map(params, width, height, seed);

    // Generate each requested map type
    for map_type in &params.maps {
        let map_seed = DeterministicRng::derive_variant_seed(seed, &format!("{:?}", map_type));

        let result = match map_type {
            TextureMapType::Albedo => generate_albedo_map(
                &base_color,
                &height_map,
                &params.layers,
                params.palette.as_deref(),
                params.color_ramp.as_deref(),
                width,
                height,
                map_seed,
                params.tileable,
            )?,
            TextureMapType::Roughness => generate_roughness_map(
                &height_map,
                &params.layers,
                roughness_range,
                width,
                height,
                map_seed,
            )?,
            TextureMapType::Metallic => generate_metallic_map(
                &height_map,
                &params.layers,
                metallic,
                width,
                height,
                map_seed,
            )?,
            TextureMapType::Normal => generate_normal_map(&height_map, 1.0)?,
            TextureMapType::Ao => generate_ao_map(&height_map, 1.0)?,
            TextureMapType::Emissive => {
                generate_emissive_map(&params.layers, width, height, map_seed)?
            }
            TextureMapType::Height => generate_height_output(&height_map)?,
        };

        results.insert(*map_type, result);
    }

    Ok(TextureResult { maps: results })
}

/// Generate height map from layers.
fn generate_height_map(
    params: &TextureMaterialV1Params,
    width: u32,
    height: u32,
    seed: u32,
) -> GrayscaleBuffer {
    let mut height_map = GrayscaleBuffer::new(width, height, 0.5);

    // Apply material-specific base pattern
    if let Some(mat) = &params.base_material {
        apply_material_pattern(&mut height_map, mat, width, height, seed);
    }

    // Apply layers
    for (i, layer) in params.layers.iter().enumerate() {
        let layer_seed = DeterministicRng::derive_layer_seed(seed, i as u32);
        apply_layer_to_height(&mut height_map, layer, layer_seed);
    }

    height_map
}

/// Save texture result to files.
pub fn save_texture_result(
    result: &TextureResult,
    output_dir: &Path,
    base_name: &str,
) -> Result<HashMap<TextureMapType, String>, GenerateError> {
    std::fs::create_dir_all(output_dir)?;

    let mut paths = HashMap::new();

    for (map_type, map_result) in &result.maps {
        let suffix = match map_type {
            TextureMapType::Albedo => "albedo",
            TextureMapType::Normal => "normal",
            TextureMapType::Roughness => "roughness",
            TextureMapType::Metallic => "metallic",
            TextureMapType::Ao => "ao",
            TextureMapType::Emissive => "emissive",
            TextureMapType::Height => "height",
        };

        let filename = format!("{}_{}.png", base_name, suffix);
        let path = output_dir.join(&filename);

        std::fs::write(&path, &map_result.data)?;

        paths.insert(*map_type, path.to_string_lossy().to_string());
    }

    Ok(paths)
}
