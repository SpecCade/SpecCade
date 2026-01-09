//! Main entry point for texture generation.
//!
//! This module provides the high-level API for generating PBR material maps
//! from a spec.

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use speccade_spec::recipe::texture::{
    Texture2dMaterialMapsV1Params, TextureMapType, MaterialType, TextureLayer,
    NoiseConfig, NoiseAlgorithm,
};

use crate::color::Color;
use crate::maps::{
    TextureBuffer, GrayscaleBuffer,
    AlbedoGenerator, RoughnessGenerator, MetallicGenerator,
    NormalGenerator, AoGenerator, EmissiveGenerator,
};
use crate::noise::{Noise2D, Fbm, PerlinNoise, SimplexNoise, WorleyNoise};
use crate::pattern::{
    Pattern2D, BrickPattern, CheckerPattern, WoodGrainPattern,
    ScratchesPattern, EdgeWearPattern,
};
use crate::png::{self, PngConfig, PngError};
use crate::rng::DeterministicRng;

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

    let mut results = HashMap::new();

    // Get base material properties
    let (base_color, roughness_range, metallic) = match &params.base_material {
        Some(mat) => {
            let r_range = mat.roughness_range.unwrap_or(get_default_roughness_range(&mat.material_type));
            let m = mat.metallic.unwrap_or(get_default_metallic(&mat.material_type));
            (
                Color::rgb(mat.base_color[0], mat.base_color[1], mat.base_color[2]),
                r_range,
                m,
            )
        }
        None => (
            Color::gray(0.5),
            [0.3, 0.7],
            0.0,
        ),
    };

    // Generate height map first (used by multiple map types)
    let height_map = generate_height_map(params, width, height, seed);

    // Generate each requested map type
    for map_type in &params.maps {
        let map_seed = DeterministicRng::derive_variant_seed(seed, &format!("{:?}", map_type));

        let result = match map_type {
            TextureMapType::Albedo => {
                generate_albedo_map(
                    &base_color,
                    &params.layers,
                    width,
                    height,
                    map_seed,
                    params.tileable,
                )?
            }
            TextureMapType::Roughness => {
                generate_roughness_map(
                    &height_map,
                    &params.layers,
                    roughness_range,
                    width,
                    height,
                    map_seed,
                )?
            }
            TextureMapType::Metallic => {
                generate_metallic_map(
                    &height_map,
                    metallic,
                    width,
                    height,
                    map_seed,
                )?
            }
            TextureMapType::Normal => {
                generate_normal_map(
                    &height_map,
                    1.0,
                )?
            }
            TextureMapType::Ao => {
                generate_ao_map(
                    &height_map,
                    1.0,
                )?
            }
            TextureMapType::Emissive => {
                generate_emissive_map(
                    &params.layers,
                    width,
                    height,
                    map_seed,
                )?
            }
            TextureMapType::Height => {
                generate_height_output(&height_map)?
            }
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
        apply_material_pattern(&mut height_map, &mat.material_type, width, height, seed);
    }

    // Apply layers
    for (i, layer) in params.layers.iter().enumerate() {
        let layer_seed = DeterministicRng::derive_layer_seed(seed, i as u32);
        apply_layer_to_height(&mut height_map, layer, layer_seed);
    }

    height_map
}

/// Apply material-specific base pattern to height map.
fn apply_material_pattern(
    height_map: &mut GrayscaleBuffer,
    material_type: &MaterialType,
    width: u32,
    height: u32,
    seed: u32,
) {
    match material_type {
        MaterialType::Brick => {
            let brick = BrickPattern::new(width, height).with_seed(seed);
            for y in 0..height {
                for x in 0..width {
                    height_map.set(x, y, brick.sample(x, y));
                }
            }
        }
        MaterialType::Wood => {
            let wood = WoodGrainPattern::new(width, height, seed);
            for y in 0..height {
                for x in 0..width {
                    height_map.set(x, y, wood.sample(x, y));
                }
            }
        }
        MaterialType::Metal | MaterialType::Stone | MaterialType::Concrete => {
            // Add noise-based height variation
            let noise = Fbm::new(PerlinNoise::new(seed))
                .with_octaves(4)
                .with_persistence(0.5);

            for y in 0..height {
                for x in 0..width {
                    let nx = x as f64 * 0.02;
                    let ny = y as f64 * 0.02;
                    let v = noise.sample_01(nx, ny);
                    height_map.set(x, y, v);
                }
            }
        }
        _ => {
            // Default: slight noise variation
            let noise = Fbm::new(PerlinNoise::new(seed))
                .with_octaves(2)
                .with_persistence(0.5);

            for y in 0..height {
                for x in 0..width {
                    let nx = x as f64 * 0.01;
                    let ny = y as f64 * 0.01;
                    let v = 0.5 + noise.sample(nx, ny) * 0.1;
                    height_map.set(x, y, v.clamp(0.0, 1.0));
                }
            }
        }
    }
}

/// Apply a layer to the height map.
fn apply_layer_to_height(
    height_map: &mut GrayscaleBuffer,
    layer: &TextureLayer,
    seed: u32,
) {
    let width = height_map.width;
    let height = height_map.height;

    match layer {
        TextureLayer::NoisePattern { noise, strength, .. } => {
            let noise_gen = create_noise_generator(noise, seed);

            for y in 0..height {
                for x in 0..width {
                    let nx = x as f64 * noise.scale;
                    let ny = y as f64 * noise.scale;
                    let noise_val = noise_gen.sample_01(nx, ny);

                    let current = height_map.get(x, y);
                    let new_val = current + (noise_val - 0.5) * strength;
                    height_map.set(x, y, new_val.clamp(0.0, 1.0));
                }
            }
        }
        TextureLayer::Scratches { density, length_range, width: scratch_width, strength, .. } => {
            let count = (density * 100.0) as u32;
            let scratches = ScratchesPattern::new(width, height, seed)
                .with_count(count)
                .with_length_range(length_range[0], length_range[1])
                .with_width(*scratch_width * width as f64)
                .with_depth(*strength);

            for y in 0..height {
                for x in 0..width {
                    let scratch_val = scratches.sample(x, y);
                    let current = height_map.get(x, y);
                    let new_val = current.min(scratch_val);
                    height_map.set(x, y, new_val);
                }
            }
        }
        TextureLayer::EdgeWear { amount, .. } => {
            let edge_wear = EdgeWearPattern::new(width, height, seed)
                .with_amount(*amount)
                .with_height_map(height_map.data.clone());

            for y in 0..height {
                for x in 0..width {
                    let wear = edge_wear.sample(x, y);
                    let current = height_map.get(x, y);
                    // Edge wear creates worn areas
                    let new_val = current * (1.0 - wear * 0.3);
                    height_map.set(x, y, new_val.clamp(0.0, 1.0));
                }
            }
        }
        _ => {}
    }
}

/// Create a noise generator from config.
fn create_noise_generator(config: &NoiseConfig, seed: u32) -> Box<dyn Noise2D> {
    let base_noise: Box<dyn Noise2D> = match config.algorithm {
        NoiseAlgorithm::Perlin => Box::new(PerlinNoise::new(seed)),
        NoiseAlgorithm::Simplex => Box::new(SimplexNoise::new(seed)),
        NoiseAlgorithm::Worley => Box::new(WorleyNoise::new(seed)),
        NoiseAlgorithm::Value => Box::new(PerlinNoise::new(seed)), // Use Perlin as fallback
        NoiseAlgorithm::Fbm => {
            Box::new(
                Fbm::new(PerlinNoise::new(seed))
                    .with_octaves(config.octaves)
                    .with_persistence(config.persistence)
                    .with_lacunarity(config.lacunarity)
            )
        }
    };

    // Wrap in FBM if octaves > 1 and not already FBM
    if config.octaves > 1 && config.algorithm != NoiseAlgorithm::Fbm {
        // We need to recreate with FBM wrapper
        match config.algorithm {
            NoiseAlgorithm::Perlin => {
                Box::new(
                    Fbm::new(PerlinNoise::new(seed))
                        .with_octaves(config.octaves)
                        .with_persistence(config.persistence)
                        .with_lacunarity(config.lacunarity)
                )
            }
            NoiseAlgorithm::Simplex => {
                Box::new(
                    Fbm::new(SimplexNoise::new(seed))
                        .with_octaves(config.octaves)
                        .with_persistence(config.persistence)
                        .with_lacunarity(config.lacunarity)
                )
            }
            _ => base_noise,
        }
    } else {
        base_noise
    }
}

/// Generate albedo map.
fn generate_albedo_map(
    base_color: &Color,
    layers: &[TextureLayer],
    width: u32,
    height: u32,
    seed: u32,
    _tileable: bool,
) -> Result<MapResult, GenerateError> {
    let generator = AlbedoGenerator::new(*base_color, seed).with_variation(0.1);
    let mut buffer = generator.generate_with_variation(width, height);

    // Apply color variation layers
    for (i, layer) in layers.iter().enumerate() {
        let layer_seed = DeterministicRng::derive_layer_seed(seed, i as u32);

        match layer {
            TextureLayer::ColorVariation { hue_range, saturation_range, value_range, noise_scale } => {
                let noise = Fbm::new(PerlinNoise::new(layer_seed))
                    .with_octaves(3);

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
                        buffer.set(x, y, Color::rgba(new_color.r, new_color.g, new_color.b, current.a));
                    }
                }
            }
            TextureLayer::Dirt { density, color, strength, .. } => {
                let dirt_color = Color::rgb(color[0], color[1], color[2]);
                generator.apply_dirt(&mut buffer, *density, dirt_color, layer_seed);
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
        if let TextureLayer::Scratches { affects, strength, .. } = layer {
            if affects.contains(&TextureMapType::Roughness) {
                let layer_seed = DeterministicRng::derive_layer_seed(seed, i as u32);
                let scratches = ScratchesPattern::new(width, height, layer_seed);

                let mut scratch_map = GrayscaleBuffer::new(width, height, 1.0);
                for y in 0..height {
                    for x in 0..width {
                        scratch_map.set(x, y, scratches.sample(x, y));
                    }
                }

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
    layers: &[TextureLayer],
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
fn generate_height_output(
    height_map: &GrayscaleBuffer,
) -> Result<MapResult, GenerateError> {
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

/// Get default roughness range for a material type.
fn get_default_roughness_range(material_type: &MaterialType) -> [f64; 2] {
    match material_type {
        MaterialType::Metal => [0.2, 0.5],
        MaterialType::Wood => [0.5, 0.8],
        MaterialType::Stone => [0.5, 0.9],
        MaterialType::Fabric => [0.8, 1.0],
        MaterialType::Plastic => [0.3, 0.5],
        MaterialType::Concrete => [0.6, 0.95],
        MaterialType::Brick => [0.6, 0.9],
        MaterialType::Procedural => [0.3, 0.7],
    }
}

/// Get default metallic value for a material type.
fn get_default_metallic(material_type: &MaterialType) -> f64 {
    match material_type {
        MaterialType::Metal => 1.0,
        _ => 0.0,
    }
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
