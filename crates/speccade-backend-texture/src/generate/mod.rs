//! Main entry point for texture generation.
//!
//! This module provides the high-level API for generating PBR material maps
//! from a spec.

mod helpers;
mod layers;
mod materials;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use speccade_spec::BackendError;
use speccade_spec::recipe::texture::{
    TextureLayer, TextureMapType, Texture2dMaterialMapsV1Params,
};

use crate::color::Color;
use crate::maps::{
    AlbedoGenerator, AoGenerator, EmissiveGenerator, GrayscaleBuffer, MetallicGenerator,
    NormalGenerator, RoughnessGenerator,
};
use crate::noise::{Fbm, Noise2D, PerlinNoise};
use crate::pattern::ScratchesPattern;
use crate::png::{self, PngConfig, PngError};
use crate::rng::DeterministicRng;

use helpers::{
    apply_pattern_to_buffer, get_default_metallic, get_default_roughness_range,
    validate_base_material, validate_map_list, validate_resolution,
};
use layers::apply_layer_to_height;
use materials::apply_material_pattern;

/// Errors from texture generation.
#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("PNG error: {0}")]
    Png(#[from] PngError),

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
    params: &Texture2dMaterialMapsV1Params,
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
            let r_range =
                mat.roughness_range
                    .unwrap_or(get_default_roughness_range(&mat.material_type));
            let m = mat.metallic.unwrap_or(get_default_metallic(&mat.material_type));
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
            TextureMapType::Metallic => {
                generate_metallic_map(&height_map, metallic, width, height, map_seed)?
            }
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
    params: &Texture2dMaterialMapsV1Params,
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

/// Generate albedo map.
///
/// The height_map is used to ensure albedo follows the same pattern as other maps
/// (e.g., differentiating brick from mortar in a brick material).
fn generate_albedo_map(
    base_color: &Color,
    height_map: &GrayscaleBuffer,
    layers: &[TextureLayer],
    width: u32,
    height: u32,
    seed: u32,
    _tileable: bool,
) -> Result<MapResult, GenerateError> {
    let generator = AlbedoGenerator::new(*base_color, seed).with_variation(0.1);

    // Start with pattern-based coloring using the height map
    // Higher values in height map = raised surface (brick) = base color
    // Lower values = recessed areas (mortar) = darker/different color
    let mortar_color = Color::rgb(
        (base_color.r * 0.4).clamp(0.0, 1.0),
        (base_color.g * 0.4).clamp(0.0, 1.0),
        (base_color.b * 0.4).clamp(0.0, 1.0),
    );

    // Generate base albedo following the height map pattern
    let mut buffer = crate::maps::TextureBuffer::new(width, height, *base_color);

    // Use height map to blend between brick and mortar colors
    // Height values typically range from ~0.2 (mortar) to ~0.9-1.0 (brick)
    // We'll use a threshold to create sharp transitions
    let mortar_threshold = 0.5;

    for y in 0..height {
        for x in 0..width {
            let h = height_map.get(x, y);

            // Calculate blend factor: 0 = mortar, 1 = brick
            let blend = if h < mortar_threshold {
                // In mortar region
                0.0
            } else if h < mortar_threshold + 0.1 {
                // Transition zone
                (h - mortar_threshold) / 0.1
            } else {
                // In brick region
                1.0
            };

            // Lerp between mortar and base color
            let color = mortar_color.lerp(base_color, blend);
            buffer.set(x, y, color);
        }
    }

    // Add noise-based variation on top (follows pattern via height map influence)
    let noise = Fbm::new(PerlinNoise::new(seed)).with_octaves(4).with_persistence(0.5);

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 * 0.02;
            let ny = y as f64 * 0.02;

            let current = buffer.get(x, y);
            let (h_val, s, v) = current.to_hsv();

            // Apply subtle HSV variation
            let h_noise = noise.sample(nx, ny) * 10.0; // Subtle hue variation
            let v_noise = noise.sample(nx + 100.0, ny + 100.0) * 0.1;

            let new_h = (h_val + h_noise) % 360.0;
            let new_v = (v + v_noise).clamp(0.0, 1.0);

            let new_color = Color::from_hsv(new_h, s, new_v);
            buffer.set(x, y, Color::rgba(new_color.r, new_color.g, new_color.b, current.a));
        }
    }

    // Apply color variation layers
    for (i, layer) in layers.iter().enumerate() {
        let layer_seed = DeterministicRng::derive_layer_seed(seed, i as u32);

        match layer {
            TextureLayer::ColorVariation {
                hue_range,
                saturation_range,
                value_range,
                noise_scale,
            } => {
                let noise = Fbm::new(PerlinNoise::new(layer_seed)).with_octaves(3);

                for y in 0..height {
                    for x in 0..width {
                        let nx = x as f64 * noise_scale;
                        let ny = y as f64 * noise_scale;

                        let h_noise = noise.sample(nx, ny) * hue_range;
                        let s_noise = noise.sample(nx + 100.0, ny) * saturation_range;
                        let v_noise = noise.sample(nx, ny + 100.0) * value_range;

                        let current = buffer.get(x, y);
                        let (h, s, v) = current.to_hsv();

                        let new_h = (h + h_noise) % 360.0;
                        let new_s = (s + s_noise).clamp(0.0, 1.0);
                        let new_v = (v + v_noise).clamp(0.0, 1.0);

                        let new_color = Color::from_hsv(new_h, new_s, new_v);
                        buffer.set(
                            x,
                            y,
                            Color::rgba(new_color.r, new_color.g, new_color.b, current.a),
                        );
                    }
                }
            }
            TextureLayer::Dirt {
                density, color, ..
            } => {
                let dirt_color = Color::rgb(color[0], color[1], color[2]);
                generator.apply_dirt(&mut buffer, *density, dirt_color, layer_seed);
            }
            TextureLayer::Checkerboard {
                tile_size,
                color1,
                color2,
                affects,
                strength,
            } => {
                if affects.contains(&TextureMapType::Albedo) {
                    use crate::pattern::{CheckerPattern, Pattern2D};
                    let checker = CheckerPattern::new(*tile_size).with_colors(*color1, *color2);

                    for y in 0..height {
                        for x in 0..width {
                            let pattern_val = checker.sample(x, y);
                            let current = buffer.get(x, y);

                            // Lerp from current color to grayscale pattern value
                            let new_val = current.r * (1.0 - strength) + pattern_val * strength;
                            buffer.set(x, y, Color::rgba(new_val, new_val, new_val, current.a));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let config = PngConfig::default();
    let (data, hash) = png::write_rgba_to_vec_with_hash(&buffer, &config)?;

    Ok(MapResult {
        map_type: TextureMapType::Albedo,
        data,
        width,
        height,
        hash,
        is_color: true,
    })
}

/// Generate roughness map.
fn generate_roughness_map(
    height_map: &GrayscaleBuffer,
    layers: &[TextureLayer],
    roughness_range: [f64; 2],
    width: u32,
    height: u32,
    seed: u32,
) -> Result<MapResult, GenerateError> {
    let base_roughness = (roughness_range[0] + roughness_range[1]) / 2.0;
    let generator = RoughnessGenerator::new(base_roughness, seed)
        .with_range(roughness_range[0], roughness_range[1]);

    let mut buffer = generator.generate_from_height(height_map, true);

    // Apply scratch layers (scratches increase roughness)
    for (i, layer) in layers.iter().enumerate() {
        if let TextureLayer::Scratches {
            affects, strength, ..
        } = layer
        {
            if affects.contains(&TextureMapType::Roughness) {
                let layer_seed = DeterministicRng::derive_layer_seed(seed, i as u32);
                let scratches = ScratchesPattern::new(width, height, layer_seed);

                let mut scratch_map = GrayscaleBuffer::new(width, height, 1.0);
                apply_pattern_to_buffer(&scratches, &mut scratch_map);

                generator.apply_scratches(&mut buffer, &scratch_map, 1.0 - strength);
            }
        }
    }

    let config = PngConfig::default();
    let (data, hash) = png::write_grayscale_to_vec_with_hash(&buffer, &config)?;

    Ok(MapResult {
        map_type: TextureMapType::Roughness,
        data,
        width,
        height,
        hash,
        is_color: false,
    })
}

/// Generate metallic map.
fn generate_metallic_map(
    _height_map: &GrayscaleBuffer,
    metallic: f64,
    width: u32,
    height: u32,
    seed: u32,
) -> Result<MapResult, GenerateError> {
    let generator = MetallicGenerator::new(metallic, seed);
    let buffer = generator.generate_with_variation(width, height);

    let config = PngConfig::default();
    let (data, hash) = png::write_grayscale_to_vec_with_hash(&buffer, &config)?;

    Ok(MapResult {
        map_type: TextureMapType::Metallic,
        data,
        width,
        height,
        hash,
        is_color: false,
    })
}

/// Generate normal map.
fn generate_normal_map(
    height_map: &GrayscaleBuffer,
    strength: f64,
) -> Result<MapResult, GenerateError> {
    let generator = NormalGenerator::new().with_strength(strength);
    let buffer = generator.generate_from_height(height_map);

    let config = PngConfig::default();
    let (data, hash) = png::write_rgb_to_vec_with_hash(&buffer, &config)?;

    Ok(MapResult {
        map_type: TextureMapType::Normal,
        data,
        width: height_map.width,
        height: height_map.height,
        hash,
        is_color: true,
    })
}

/// Generate AO map.
fn generate_ao_map(
    height_map: &GrayscaleBuffer,
    strength: f64,
) -> Result<MapResult, GenerateError> {
    let generator = AoGenerator::new().with_strength(strength);
    let buffer = generator.generate_from_height(height_map);

    let config = PngConfig::default();
    let (data, hash) = png::write_grayscale_to_vec_with_hash(&buffer, &config)?;

    Ok(MapResult {
        map_type: TextureMapType::Ao,
        data,
        width: height_map.width,
        height: height_map.height,
        hash,
        is_color: false,
    })
}

/// Generate emissive map.
fn generate_emissive_map(
    _layers: &[TextureLayer],
    width: u32,
    height: u32,
    seed: u32,
) -> Result<MapResult, GenerateError> {
    // Default: no emission
    let generator = EmissiveGenerator::new(Color::black(), seed);
    let buffer = generator.generate_none(width, height);

    // TODO: Apply emissive layers if any are defined

    let config = PngConfig::default();
    let (data, hash) = png::write_rgb_to_vec_with_hash(&buffer, &config)?;

    Ok(MapResult {
        map_type: TextureMapType::Emissive,
        data,
        width,
        height,
        hash,
        is_color: true,
    })
}

/// Generate height output (just converts the height map to PNG).
fn generate_height_output(height_map: &GrayscaleBuffer) -> Result<MapResult, GenerateError> {
    let config = PngConfig::default();
    let (data, hash) = png::write_grayscale_to_vec_with_hash(height_map, &config)?;

    Ok(MapResult {
        map_type: TextureMapType::Height,
        data,
        width: height_map.width,
        height: height_map.height,
        hash,
        is_color: false,
    })
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
