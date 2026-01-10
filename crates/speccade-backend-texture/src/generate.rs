//! Main entry point for texture generation.
//!
//! This module provides the high-level API for generating PBR material maps
//! from a spec.

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

use speccade_spec::recipe::texture::{
    Texture2dMaterialMapsV1Params, TextureMapType, MaterialType, TextureLayer,
    NoiseConfig, NoiseAlgorithm, GradientDirection, StripeDirection,
};

use crate::color::Color;
use crate::maps::{
    GrayscaleBuffer,
    AlbedoGenerator, RoughnessGenerator, MetallicGenerator,
    NormalGenerator, AoGenerator, EmissiveGenerator,
};
use crate::noise::{Noise2D, Fbm, PerlinNoise, SimplexNoise, WorleyNoise};
use crate::pattern::{
    Pattern2D, BrickPattern, WoodGrainPattern,
    ScratchesPattern, EdgeWearPattern, StripesPattern, GradientPattern,
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

fn validate_resolution(width: u32, height: u32) -> Result<(), GenerateError> {
    if width == 0 || height == 0 {
        return Err(GenerateError::InvalidParameter(format!(
            "resolution must be at least 1x1, got [{}, {}]",
            width, height
        )));
    }

    (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| GenerateError::InvalidParameter("resolution is too large".to_string()))?;

    Ok(())
}

fn validate_map_list(map_types: &[TextureMapType]) -> Result<(), GenerateError> {
    if map_types.is_empty() {
        return Err(GenerateError::InvalidParameter(
            "maps must contain at least one map type".to_string(),
        ));
    }

    let mut seen: HashSet<TextureMapType> = HashSet::new();
    for map_type in map_types {
        if !seen.insert(*map_type) {
            return Err(GenerateError::InvalidParameter(format!(
                "duplicate map type: {:?}",
                map_type
            )));
        }
    }

    Ok(())
}

fn validate_unit_interval(name: &str, value: f64) -> Result<(), GenerateError> {
    if !value.is_finite() {
        return Err(GenerateError::InvalidParameter(format!(
            "{} must be finite, got {}",
            name, value
        )));
    }
    if !(0.0..=1.0).contains(&value) {
        return Err(GenerateError::InvalidParameter(format!(
            "{} must be in [0, 1], got {}",
            name, value
        )));
    }
    Ok(())
}

fn validate_base_material(params: &Texture2dMaterialMapsV1Params) -> Result<(), GenerateError> {
    let Some(mat) = &params.base_material else {
        return Ok(());
    };

    for (i, c) in mat.base_color.iter().enumerate() {
        validate_unit_interval(&format!("base_material.base_color[{}]", i), *c)?;
    }

    if let Some([min, max]) = mat.roughness_range {
        validate_unit_interval("base_material.roughness_range[0]", min)?;
        validate_unit_interval("base_material.roughness_range[1]", max)?;
        if min > max {
            return Err(GenerateError::InvalidParameter(format!(
                "base_material.roughness_range must have min <= max, got [{}, {}]",
                min, max
            )));
        }
    }

    if let Some(metallic) = mat.metallic {
        validate_unit_interval("base_material.metallic", metallic)?;
    }

    Ok(())
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
        TextureLayer::Gradient { direction, start, end, center, inner, outer, strength, .. } => {
            let gradient = match direction {
                GradientDirection::Horizontal => {
                    let s = start.unwrap_or(0.0);
                    let e = end.unwrap_or(1.0);
                    GradientPattern::new_horizontal(width, height, s, e)
                }
                GradientDirection::Vertical => {
                    let s = start.unwrap_or(0.0);
                    let e = end.unwrap_or(1.0);
                    GradientPattern::new_vertical(width, height, s, e)
                }
                GradientDirection::Radial => {
                    let c = center.unwrap_or([0.5, 0.5]);
                    let i = inner.unwrap_or(1.0);
                    let o = outer.unwrap_or(0.0);
                    GradientPattern::new_radial(width, height, c, i, o)
                }
            };

            for y in 0..height {
                for x in 0..width {
                    let gradient_val = gradient.sample(x, y);
                    let current = height_map.get(x, y);
                    // Blend gradient with current value
                    let new_val = current * (1.0 - strength) + gradient_val * strength;
                    height_map.set(x, y, new_val.clamp(0.0, 1.0));
                }
            }
        }
        TextureLayer::Stripes { direction, stripe_width, color1, color2, strength, .. } => {
            let stripes = match direction {
                StripeDirection::Horizontal => {
                    StripesPattern::new_horizontal(width, height, *stripe_width, *color1, *color2)
                }
                StripeDirection::Vertical => {
                    StripesPattern::new_vertical(width, height, *stripe_width, *color1, *color2)
                }
            };

            for y in 0..height {
                for x in 0..width {
                    let stripe_val = stripes.sample(x, y);
                    let current = height_map.get(x, y);
                    // Blend stripes with current value
                    let new_val = current * (1.0 - strength) + stripe_val * strength;
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
            TextureLayer::Dirt { density, color, .. } => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::texture::BaseMaterial;

    fn make_params() -> Texture2dMaterialMapsV1Params {
        Texture2dMaterialMapsV1Params {
            resolution: [32, 32],
            tileable: true,
            maps: vec![
                TextureMapType::Albedo,
                TextureMapType::Normal,
                TextureMapType::Roughness,
            ],
            base_material: Some(BaseMaterial {
                material_type: MaterialType::Metal,
                base_color: [0.8, 0.2, 0.1],
                roughness_range: Some([0.2, 0.5]),
                metallic: Some(1.0),
            }),
            layers: vec![],
            palette: None,
            color_ramp: None,
        }
    }

    // ========================================================================
    // Determinism Tests
    // ========================================================================

    #[test]
    fn test_generate_material_maps_deterministic() {
        let params = make_params();
        let result1 = generate_material_maps(&params, 42).unwrap();
        let result2 = generate_material_maps(&params, 42).unwrap();

        assert_eq!(result1.maps.len(), params.maps.len());
        assert_eq!(result2.maps.len(), params.maps.len());

        for map_type in &params.maps {
            let m1 = result1.maps.get(map_type).unwrap();
            let m2 = result2.maps.get(map_type).unwrap();
            assert_eq!(m1.hash, m2.hash);
            assert_eq!(m1.data, m2.data);
            assert_eq!(m1.width, params.resolution[0]);
            assert_eq!(m1.height, params.resolution[1]);
            assert!(!m1.data.is_empty());
        }
    }

    #[test]
    fn test_generate_material_maps_seed_changes_output() {
        let params = make_params();
        let result1 = generate_material_maps(&params, 1).unwrap();
        let result2 = generate_material_maps(&params, 2).unwrap();

        let hash1 = &result1.maps.get(&TextureMapType::Albedo).unwrap().hash;
        let hash2 = &result2.maps.get(&TextureMapType::Albedo).unwrap().hash;
        assert_ne!(hash1, hash2);
    }

    // ========================================================================
    // Validation Tests
    // ========================================================================

    #[test]
    fn test_generate_material_maps_invalid_resolution() {
        let mut params = make_params();
        params.resolution = [0, 32];
        let err = generate_material_maps(&params, 42).unwrap_err();
        assert!(err.to_string().contains("resolution"));
    }

    #[test]
    fn test_generate_material_maps_empty_maps_is_error() {
        let mut params = make_params();
        params.maps.clear();
        let err = generate_material_maps(&params, 42).unwrap_err();
        assert!(err.to_string().contains("maps"));
    }

    #[test]
    fn test_generate_material_maps_duplicate_maps_is_error() {
        let mut params = make_params();
        params.maps = vec![TextureMapType::Albedo, TextureMapType::Albedo];
        let err = generate_material_maps(&params, 42).unwrap_err();
        assert!(err.to_string().contains("duplicate"));
    }

    // ========================================================================
    // Layer Pattern Generation Tests
    // ========================================================================

    #[test]
    fn test_layer_noise_pattern_generation() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::NoisePattern {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Perlin,
                scale: 0.1,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            affects: vec![TextureMapType::Roughness],
            strength: 0.5,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Roughness));
    }

    #[test]
    fn test_layer_gradient_horizontal_generation() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::Gradient {
            direction: GradientDirection::Horizontal,
            start: Some(0.0),
            end: Some(1.0),
            center: None,
            inner: None,
            outer: None,
            affects: vec![TextureMapType::Albedo],
            strength: 0.75,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Albedo));
    }

    #[test]
    fn test_layer_gradient_vertical_generation() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::Gradient {
            direction: GradientDirection::Vertical,
            start: Some(0.2),
            end: Some(0.8),
            center: None,
            inner: None,
            outer: None,
            affects: vec![TextureMapType::Roughness],
            strength: 1.0,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Roughness));
    }

    #[test]
    fn test_layer_gradient_radial_generation() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::Gradient {
            direction: GradientDirection::Radial,
            start: None,
            end: None,
            center: Some([0.5, 0.5]),
            inner: Some(1.0),
            outer: Some(0.0),
            affects: vec![TextureMapType::Albedo],
            strength: 0.8,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Albedo));
    }

    #[test]
    fn test_layer_stripes_horizontal_generation() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::Stripes {
            direction: StripeDirection::Horizontal,
            stripe_width: 4,
            color1: 0.0,
            color2: 1.0,
            affects: vec![TextureMapType::Albedo],
            strength: 1.0,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Albedo));
    }

    #[test]
    fn test_layer_stripes_vertical_generation() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::Stripes {
            direction: StripeDirection::Vertical,
            stripe_width: 8,
            color1: 0.3,
            color2: 0.7,
            affects: vec![TextureMapType::Roughness],
            strength: 0.5,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Roughness));
    }

    #[test]
    fn test_layer_scratches_generation() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::Scratches {
            density: 0.2,
            length_range: [0.1, 0.4],
            width: 0.002,
            affects: vec![TextureMapType::Roughness],
            strength: 0.6,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Roughness));
    }

    #[test]
    fn test_layer_edge_wear_generation() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::EdgeWear {
            amount: 0.3,
            affects: vec![TextureMapType::Roughness],
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Roughness));
    }

    #[test]
    fn test_layer_dirt_generation() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::Dirt {
            density: 0.15,
            color: [0.3, 0.25, 0.2],
            affects: vec![TextureMapType::Albedo],
            strength: 0.4,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Albedo));
    }

    #[test]
    fn test_layer_color_variation_generation() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::ColorVariation {
            hue_range: 10.0,
            saturation_range: 0.1,
            value_range: 0.15,
            noise_scale: 0.05,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Albedo));
    }

    // ========================================================================
    // Material Type Tests
    // ========================================================================

    #[test]
    fn test_material_type_metal_generation() {
        let mut params = make_params();
        params.base_material = Some(BaseMaterial {
            material_type: MaterialType::Metal,
            base_color: [0.8, 0.8, 0.8],
            roughness_range: Some([0.2, 0.4]),
            metallic: Some(1.0),
        });

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Albedo));
    }

    #[test]
    fn test_material_type_wood_generation() {
        let mut params = make_params();
        params.base_material = Some(BaseMaterial {
            material_type: MaterialType::Wood,
            base_color: [0.6, 0.4, 0.2],
            roughness_range: Some([0.5, 0.8]),
            metallic: Some(0.0),
        });

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Albedo));
    }

    #[test]
    fn test_material_type_brick_generation() {
        let mut params = make_params();
        params.base_material = Some(BaseMaterial {
            material_type: MaterialType::Brick,
            base_color: [0.7, 0.3, 0.2],
            roughness_range: Some([0.6, 0.9]),
            metallic: Some(0.0),
        });

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Albedo));
    }

    #[test]
    fn test_all_material_types() {
        for mat_type in [
            MaterialType::Metal,
            MaterialType::Wood,
            MaterialType::Stone,
            MaterialType::Fabric,
            MaterialType::Plastic,
            MaterialType::Concrete,
            MaterialType::Brick,
            MaterialType::Procedural,
        ] {
            let mut params = make_params();
            params.base_material = Some(BaseMaterial {
                material_type: mat_type,
                base_color: [0.5, 0.5, 0.5],
                roughness_range: None,
                metallic: None,
            });

            let result = generate_material_maps(&params, 42).unwrap();
            assert!(result.maps.len() > 0);
        }
    }

    // ========================================================================
    // Noise Algorithm Tests
    // ========================================================================

    #[test]
    fn test_noise_algorithm_perlin() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::NoisePattern {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Perlin,
                scale: 0.1,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            affects: vec![TextureMapType::Roughness],
            strength: 0.5,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Roughness));
    }

    #[test]
    fn test_noise_algorithm_simplex() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::NoisePattern {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Simplex,
                scale: 0.05,
                octaves: 6,
                persistence: 0.6,
                lacunarity: 2.2,
            },
            affects: vec![TextureMapType::Roughness],
            strength: 0.7,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Roughness));
    }

    #[test]
    fn test_noise_algorithm_worley() {
        let mut params = make_params();
        params.layers = vec![TextureLayer::NoisePattern {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Worley,
                scale: 0.02,
                octaves: 3,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            affects: vec![TextureMapType::Roughness],
            strength: 0.4,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Roughness));
    }

    #[test]
    fn test_noise_algorithm_all() {
        for algo in [
            NoiseAlgorithm::Perlin,
            NoiseAlgorithm::Simplex,
            NoiseAlgorithm::Worley,
            NoiseAlgorithm::Value,
            NoiseAlgorithm::Fbm,
        ] {
            let mut params = make_params();
            params.layers = vec![TextureLayer::NoisePattern {
                noise: NoiseConfig {
                    algorithm: algo,
                    scale: 0.1,
                    octaves: 4,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
                affects: vec![TextureMapType::Roughness],
                strength: 0.5,
            }];

            let result = generate_material_maps(&params, 42).unwrap();
            assert!(result.maps.len() > 0);
        }
    }

    // ========================================================================
    // File I/O Tests
    // ========================================================================

    #[test]
    fn test_save_texture_result_writes_files() {
        let params = make_params();
        let result = generate_material_maps(&params, 42).unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let paths = save_texture_result(&result, tmp.path(), "material").unwrap();

        assert!(paths.contains_key(&TextureMapType::Albedo));
        assert!(paths.contains_key(&TextureMapType::Normal));
        assert!(paths.contains_key(&TextureMapType::Roughness));

        for path in paths.values() {
            assert!(std::path::Path::new(path).exists());
        }
    }

    #[test]
    fn test_save_all_map_types() {
        let mut params = make_params();
        params.maps = vec![
            TextureMapType::Albedo,
            TextureMapType::Normal,
            TextureMapType::Roughness,
            TextureMapType::Metallic,
            TextureMapType::Ao,
            TextureMapType::Emissive,
            TextureMapType::Height,
        ];

        let result = generate_material_maps(&params, 42).unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let paths = save_texture_result(&result, tmp.path(), "test").unwrap();

        assert_eq!(paths.len(), 7);
        for map_type in params.maps {
            assert!(paths.contains_key(&map_type));
        }
    }

    // ========================================================================
    // Multi-Layer Tests
    // ========================================================================

    #[test]
    fn test_multiple_layers_combined() {
        let mut params = make_params();
        params.layers = vec![
            TextureLayer::NoisePattern {
                noise: NoiseConfig {
                    algorithm: NoiseAlgorithm::Perlin,
                    scale: 0.1,
                    octaves: 4,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
                affects: vec![TextureMapType::Roughness],
                strength: 0.3,
            },
            TextureLayer::Scratches {
                density: 0.1,
                length_range: [0.1, 0.3],
                width: 0.001,
                affects: vec![TextureMapType::Roughness],
                strength: 0.5,
            },
            TextureLayer::EdgeWear {
                amount: 0.2,
                affects: vec![TextureMapType::Roughness],
            },
        ];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.contains_key(&TextureMapType::Roughness));
    }

    // ========================================================================
    // Palette and Color Ramp Tests
    // ========================================================================

    #[test]
    fn test_palette_specified() {
        let mut params = make_params();
        params.palette = Some(vec![
            "#FF0000".to_string(),
            "#00FF00".to_string(),
            "#0000FF".to_string(),
        ]);

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.len() > 0);
    }

    #[test]
    fn test_color_ramp_specified() {
        let mut params = make_params();
        params.color_ramp = Some(vec![
            "#000000".to_string(),
            "#808080".to_string(),
            "#FFFFFF".to_string(),
        ]);

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(result.maps.len() > 0);
    }
}
