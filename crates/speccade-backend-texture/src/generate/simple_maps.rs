//! Simple map generators (normal, AO, height).
//!
//! These maps are derived directly from height data without complex layer processing.

use speccade_spec::recipe::texture::TextureMapType;

use crate::maps::{AoGenerator, GrayscaleBuffer, NormalGenerator};
use crate::png::{self, PngConfig};

use super::{GenerateError, MapResult};

/// Generate normal map.
pub fn generate_normal_map(
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
pub fn generate_ao_map(
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

/// Generate height output (just converts the height map to PNG).
pub fn generate_height_output(height_map: &GrayscaleBuffer) -> Result<MapResult, GenerateError> {
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
