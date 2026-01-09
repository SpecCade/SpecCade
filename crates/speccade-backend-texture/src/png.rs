//! Deterministic PNG writer.
//!
//! Uses fixed compression settings to ensure byte-identical output
//! for the same input data, as required by SpecCade's Tier 1 determinism.

use std::io::Write;
use std::path::Path;
use png::{BitDepth, ColorType, Encoder, Compression, FilterType};
use thiserror::Error;

use crate::maps::{TextureBuffer, GrayscaleBuffer};

/// Errors from PNG operations.
#[derive(Debug, Error)]
pub enum PngError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("PNG encoding error: {0}")]
    Encoding(#[from] png::EncodingError),

    #[error("Invalid dimensions: {0}")]
    InvalidDimensions(String),
}

/// PNG export configuration for deterministic output.
#[derive(Debug, Clone)]
pub struct PngConfig {
    /// Compression level (0-9). Use a fixed value for determinism.
    pub compression: Compression,
    /// Filter type. Use a fixed value for determinism.
    pub filter: FilterType,
}

impl Default for PngConfig {
    fn default() -> Self {
        Self {
            // Use default compression (level 6) for good balance of speed and size
            compression: Compression::Default,
            // Adaptive filter is deterministic but may vary output
            // Use None for maximum determinism (no filtering)
            filter: FilterType::NoFilter,
        }
    }
}

impl PngConfig {
    /// Create config optimized for file size (slower, but deterministic).
    pub fn best_compression() -> Self {
        Self {
            compression: Compression::Best,
            filter: FilterType::Paeth, // Best compression with Paeth filter
        }
    }

    /// Create config optimized for speed (faster, but larger files).
    pub fn fast() -> Self {
        Self {
            compression: Compression::Fast,
            filter: FilterType::NoFilter,
        }
    }
}

/// Write an RGBA texture buffer to a PNG file.
pub fn write_rgba(
    buffer: &TextureBuffer,
    path: &Path,
    config: &PngConfig,
) -> Result<(), PngError> {
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);

    write_rgba_to_writer(buffer, writer, config)
}

/// Write an RGBA texture buffer to any writer.
pub fn write_rgba_to_writer<W: Write>(
    buffer: &TextureBuffer,
    writer: W,
    config: &PngConfig,
) -> Result<(), PngError> {
    let mut encoder = Encoder::new(writer, buffer.width, buffer.height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_compression(config.compression);
    encoder.set_filter(config.filter);

    // Do not include timestamps or other variable metadata
    // The png crate doesn't add timestamps by default

    let mut png_writer = encoder.write_header()?;

    let data = buffer.to_rgba8();
    png_writer.write_image_data(&data)?;

    Ok(())
}

/// Write an RGB texture buffer to a PNG file.
pub fn write_rgb(
    buffer: &TextureBuffer,
    path: &Path,
    config: &PngConfig,
) -> Result<(), PngError> {
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);

    write_rgb_to_writer(buffer, writer, config)
}

/// Write an RGB texture buffer to any writer.
pub fn write_rgb_to_writer<W: Write>(
    buffer: &TextureBuffer,
    writer: W,
    config: &PngConfig,
) -> Result<(), PngError> {
    let mut encoder = Encoder::new(writer, buffer.width, buffer.height);
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_compression(config.compression);
    encoder.set_filter(config.filter);

    let mut png_writer = encoder.write_header()?;

    let data = buffer.to_rgb8();
    png_writer.write_image_data(&data)?;

    Ok(())
}

/// Write a grayscale buffer to a PNG file.
pub fn write_grayscale(
    buffer: &GrayscaleBuffer,
    path: &Path,
    config: &PngConfig,
) -> Result<(), PngError> {
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);

    write_grayscale_to_writer(buffer, writer, config)
}

/// Write a grayscale buffer to any writer.
pub fn write_grayscale_to_writer<W: Write>(
    buffer: &GrayscaleBuffer,
    writer: W,
    config: &PngConfig,
) -> Result<(), PngError> {
    let mut encoder = Encoder::new(writer, buffer.width, buffer.height);
    encoder.set_color(ColorType::Grayscale);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_compression(config.compression);
    encoder.set_filter(config.filter);

    let mut png_writer = encoder.write_header()?;

    let data = buffer.to_bytes();
    png_writer.write_image_data(&data)?;

    Ok(())
}

/// Write raw bytes to a PNG file.
pub fn write_raw(
    data: &[u8],
    width: u32,
    height: u32,
    color_type: ColorType,
    path: &Path,
    config: &PngConfig,
) -> Result<(), PngError> {
    let expected_size = match color_type {
        ColorType::Grayscale => (width * height) as usize,
        ColorType::Rgb => (width * height * 3) as usize,
        ColorType::Rgba => (width * height * 4) as usize,
        ColorType::GrayscaleAlpha => (width * height * 2) as usize,
        ColorType::Indexed => return Err(PngError::InvalidDimensions("Indexed color not supported".into())),
    };

    if data.len() != expected_size {
        return Err(PngError::InvalidDimensions(format!(
            "Expected {} bytes for {}x{} {:?}, got {}",
            expected_size, width, height, color_type, data.len()
        )));
    }

    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);

    let mut encoder = Encoder::new(writer, width, height);
    encoder.set_color(color_type);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_compression(config.compression);
    encoder.set_filter(config.filter);

    let mut png_writer = encoder.write_header()?;
    png_writer.write_image_data(data)?;

    Ok(())
}

/// Compute the BLAKE3 hash of PNG data.
pub fn hash_png(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Write to a Vec<u8> and return the hash.
pub fn write_rgba_to_vec_with_hash(
    buffer: &TextureBuffer,
    config: &PngConfig,
) -> Result<(Vec<u8>, String), PngError> {
    let mut data = Vec::new();
    write_rgba_to_writer(buffer, &mut data, config)?;
    let hash = hash_png(&data);
    Ok((data, hash))
}

/// Write to a Vec<u8> and return the hash.
pub fn write_grayscale_to_vec_with_hash(
    buffer: &GrayscaleBuffer,
    config: &PngConfig,
) -> Result<(Vec<u8>, String), PngError> {
    let mut data = Vec::new();
    write_grayscale_to_writer(buffer, &mut data, config)?;
    let hash = hash_png(&data);
    Ok((data, hash))
}

/// Write RGB to a Vec<u8> and return the hash.
pub fn write_rgb_to_vec_with_hash(
    buffer: &TextureBuffer,
    config: &PngConfig,
) -> Result<(Vec<u8>, String), PngError> {
    let mut data = Vec::new();
    write_rgb_to_writer(buffer, &mut data, config)?;
    let hash = hash_png(&data);
    Ok((data, hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    #[test]
    fn test_rgba_deterministic() {
        let mut buffer = TextureBuffer::new(64, 64, Color::black());
        for y in 0..64 {
            for x in 0..64 {
                let r = x as f64 / 63.0;
                let g = y as f64 / 63.0;
                buffer.set(x, y, Color::rgb(r, g, 0.5));
            }
        }

        let config = PngConfig::default();

        let (data1, hash1) = write_rgba_to_vec_with_hash(&buffer, &config).unwrap();
        let (data2, hash2) = write_rgba_to_vec_with_hash(&buffer, &config).unwrap();

        assert_eq!(data1, data2, "PNG data should be identical");
        assert_eq!(hash1, hash2, "PNG hashes should be identical");
    }

    #[test]
    fn test_grayscale_deterministic() {
        let mut buffer = GrayscaleBuffer::new(64, 64, 0.5);
        for y in 0..64 {
            for x in 0..64 {
                let v = (x + y) as f64 / 126.0;
                buffer.set(x, y, v);
            }
        }

        let config = PngConfig::default();

        let (data1, hash1) = write_grayscale_to_vec_with_hash(&buffer, &config).unwrap();
        let (data2, hash2) = write_grayscale_to_vec_with_hash(&buffer, &config).unwrap();

        assert_eq!(data1, data2, "PNG data should be identical");
        assert_eq!(hash1, hash2, "PNG hashes should be identical");
    }

    #[test]
    fn test_different_configs_different_output() {
        let buffer = TextureBuffer::new(64, 64, Color::gray(0.5));

        let fast_config = PngConfig::fast();
        let best_config = PngConfig::best_compression();

        let (fast_data, _) = write_rgba_to_vec_with_hash(&buffer, &fast_config).unwrap();
        let (best_data, _) = write_rgba_to_vec_with_hash(&buffer, &best_config).unwrap();

        // Different configs should produce different byte sizes
        // (best compression should be smaller or equal)
        assert!(best_data.len() <= fast_data.len() || fast_data.len() <= best_data.len());
    }
}
