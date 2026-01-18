//! Texture analysis module for extracting quality metrics from PNG files.
//!
//! This module provides deterministic texture analysis for LLM-driven iteration loops
//! and quality gating. All metrics are computed to produce byte-identical JSON output
//! across runs on the same input.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Precision for floating point values in output (6 decimal places).
const FLOAT_PRECISION: i32 = 6;

/// Round a float to the specified number of decimal places.
fn round_f64(value: f64, decimals: i32) -> f64 {
    let multiplier = 10_f64.powi(decimals);
    (value * multiplier).round() / multiplier
}

/// Texture analysis result containing all extracted metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureMetrics {
    /// Format metadata
    pub format: TextureFormatMetadata,
    /// Histogram statistics per channel
    pub histogram: TextureHistogramStats,
    /// Contrast metrics
    pub contrast: TextureContrastMetrics,
}

/// Texture format metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureFormatMetadata {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Number of color channels
    pub channels: u8,
    /// Color type name (Grayscale, RGB, RGBA, etc.)
    pub color_type: String,
    /// Bit depth
    pub bit_depth: u8,
}

/// Per-channel histogram statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStats {
    /// Mean value (0-255 range)
    pub mean: f64,
    /// Standard deviation
    pub stddev: f64,
    /// Minimum value
    pub min: u8,
    /// Maximum value
    pub max: u8,
}

/// Histogram statistics for all channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureHistogramStats {
    /// Red channel stats (or grayscale for single-channel)
    pub red: ChannelStats,
    /// Green channel stats (None for grayscale)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub green: Option<ChannelStats>,
    /// Blue channel stats (None for grayscale)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blue: Option<ChannelStats>,
    /// Alpha channel stats (None if no alpha)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<ChannelStats>,
}

/// Contrast metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureContrastMetrics {
    /// Global contrast ratio (max luminance / min luminance, or range if min is 0)
    pub contrast_ratio: f64,
    /// Weber contrast for the image
    pub weber_contrast: f64,
}

/// Error type for texture analysis.
#[derive(Debug, Clone)]
pub enum TextureAnalysisError {
    /// File too short
    FileTooShort { expected: usize, actual: usize },
    /// Invalid PNG signature
    InvalidSignature,
    /// Invalid or missing IHDR chunk
    InvalidIhdr { message: String },
    /// PNG decoding error
    DecodeError { message: String },
    /// Empty image
    EmptyImage,
}

impl std::fmt::Display for TextureAnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureAnalysisError::FileTooShort { expected, actual } => {
                write!(
                    f,
                    "PNG data too short: expected at least {} bytes, got {}",
                    expected, actual
                )
            }
            TextureAnalysisError::InvalidSignature => {
                write!(f, "Invalid PNG signature")
            }
            TextureAnalysisError::InvalidIhdr { message } => {
                write!(f, "Invalid IHDR: {}", message)
            }
            TextureAnalysisError::DecodeError { message } => {
                write!(f, "PNG decode error: {}", message)
            }
            TextureAnalysisError::EmptyImage => {
                write!(f, "Image has zero pixels")
            }
        }
    }
}

impl std::error::Error for TextureAnalysisError {}

/// PNG header information (from IHDR chunk).
struct PngHeader {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
}

impl PngHeader {
    fn channels(&self) -> u8 {
        match self.color_type {
            0 => 1, // Grayscale
            2 => 3, // RGB
            3 => 1, // Indexed (palette)
            4 => 2, // Grayscale + Alpha
            6 => 4, // RGBA
            _ => 0,
        }
    }

    fn color_type_name(&self) -> &'static str {
        match self.color_type {
            0 => "Grayscale",
            2 => "RGB",
            3 => "Indexed",
            4 => "Grayscale+Alpha",
            6 => "RGBA",
            _ => "Unknown",
        }
    }
}

/// Parse PNG header (just the IHDR chunk) without full decoding.
fn parse_png_header(png_data: &[u8]) -> Result<PngHeader, TextureAnalysisError> {
    const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    const MIN_HEADER_SIZE: usize = 8 + 8 + 13;

    if png_data.len() < MIN_HEADER_SIZE {
        return Err(TextureAnalysisError::FileTooShort {
            expected: MIN_HEADER_SIZE,
            actual: png_data.len(),
        });
    }

    if png_data[0..8] != PNG_SIGNATURE {
        return Err(TextureAnalysisError::InvalidSignature);
    }

    let chunk_type = &png_data[12..16];
    if chunk_type != b"IHDR" {
        return Err(TextureAnalysisError::InvalidIhdr {
            message: format!("First chunk must be IHDR, got {:?}", chunk_type),
        });
    }

    let ihdr_data = &png_data[16..29];
    let width = u32::from_be_bytes([ihdr_data[0], ihdr_data[1], ihdr_data[2], ihdr_data[3]]);
    let height = u32::from_be_bytes([ihdr_data[4], ihdr_data[5], ihdr_data[6], ihdr_data[7]]);
    let bit_depth = ihdr_data[8];
    let color_type = ihdr_data[9];

    if width == 0 || height == 0 {
        return Err(TextureAnalysisError::EmptyImage);
    }

    Ok(PngHeader {
        width,
        height,
        bit_depth,
        color_type,
    })
}

/// Decode PNG to raw RGBA pixels.
fn decode_png_pixels(png_data: &[u8]) -> Result<(PngHeader, Vec<u8>), TextureAnalysisError> {
    let header = parse_png_header(png_data)?;

    let decoder = png::Decoder::new(std::io::Cursor::new(png_data));
    let mut reader = decoder
        .read_info()
        .map_err(|e| TextureAnalysisError::DecodeError {
            message: e.to_string(),
        })?;

    let mut pixels = vec![0u8; reader.output_buffer_size()];
    reader
        .next_frame(&mut pixels)
        .map_err(|e| TextureAnalysisError::DecodeError {
            message: e.to_string(),
        })?;

    Ok((header, pixels))
}

/// Calculate channel statistics.
fn calculate_channel_stats(values: &[u8]) -> ChannelStats {
    if values.is_empty() {
        return ChannelStats {
            mean: 0.0,
            stddev: 0.0,
            min: 0,
            max: 0,
        };
    }

    let sum: u64 = values.iter().map(|&v| v as u64).sum();
    let mean = sum as f64 / values.len() as f64;

    let variance: f64 = values
        .iter()
        .map(|&v| {
            let diff = v as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / values.len() as f64;

    let stddev = variance.sqrt();
    let min = *values.iter().min().unwrap_or(&0);
    let max = *values.iter().max().unwrap_or(&0);

    ChannelStats {
        mean: round_f64(mean, FLOAT_PRECISION),
        stddev: round_f64(stddev, FLOAT_PRECISION),
        min,
        max,
    }
}

/// Calculate luminance from RGB values (ITU-R BT.601).
fn rgb_to_luminance(r: u8, g: u8, b: u8) -> f64 {
    0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64
}

/// Calculate contrast metrics.
fn calculate_contrast(pixels: &[u8], channels: u8) -> TextureContrastMetrics {
    if pixels.is_empty() {
        return TextureContrastMetrics {
            contrast_ratio: 0.0,
            weber_contrast: 0.0,
        };
    }

    let step = channels as usize;
    let pixel_count = pixels.len() / step;

    if pixel_count == 0 {
        return TextureContrastMetrics {
            contrast_ratio: 0.0,
            weber_contrast: 0.0,
        };
    }

    // Calculate luminance values
    let luminances: Vec<f64> = (0..pixel_count)
        .map(|i| {
            let offset = i * step;
            match channels {
                1 => pixels[offset] as f64,
                2 => pixels[offset] as f64, // Grayscale + alpha
                3 => rgb_to_luminance(pixels[offset], pixels[offset + 1], pixels[offset + 2]),
                4 => rgb_to_luminance(pixels[offset], pixels[offset + 1], pixels[offset + 2]),
                _ => 0.0,
            }
        })
        .collect();

    let min_lum = luminances.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_lum = luminances.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean_lum: f64 = luminances.iter().sum::<f64>() / luminances.len() as f64;

    // Contrast ratio: max/min (or range if min is 0)
    let contrast_ratio = if min_lum > 0.0 {
        round_f64(max_lum / min_lum, FLOAT_PRECISION)
    } else {
        round_f64(max_lum - min_lum, FLOAT_PRECISION)
    };

    // Weber contrast: (max - min) / mean
    let weber_contrast = if mean_lum > 0.0 {
        round_f64((max_lum - min_lum) / mean_lum, FLOAT_PRECISION)
    } else {
        0.0
    };

    TextureContrastMetrics {
        contrast_ratio,
        weber_contrast,
    }
}

/// Extract pixel data from PNG for embedding computation.
///
/// Returns raw pixels, dimensions (width, height), and channel count.
pub fn extract_png_pixels(
    png_data: &[u8],
) -> Result<(Vec<u8>, u32, u32, u8), TextureAnalysisError> {
    let (header, pixels) = decode_png_pixels(png_data)?;
    let channels = header.channels();

    if pixels.is_empty() || header.width == 0 || header.height == 0 {
        return Err(TextureAnalysisError::EmptyImage);
    }

    Ok((pixels, header.width, header.height, channels))
}

/// Analyze a PNG file and return metrics.
pub fn analyze_png(png_data: &[u8]) -> Result<TextureMetrics, TextureAnalysisError> {
    let (header, pixels) = decode_png_pixels(png_data)?;

    let channels = header.channels();
    let step = channels as usize;
    let pixel_count = pixels.len() / step;

    if pixel_count == 0 {
        return Err(TextureAnalysisError::EmptyImage);
    }

    // Extract channel values
    let mut red_values = Vec::with_capacity(pixel_count);
    let mut green_values: Option<Vec<u8>> = if channels >= 3 {
        Some(Vec::with_capacity(pixel_count))
    } else {
        None
    };
    let mut blue_values: Option<Vec<u8>> = if channels >= 3 {
        Some(Vec::with_capacity(pixel_count))
    } else {
        None
    };
    let mut alpha_values: Option<Vec<u8>> = if channels == 2 || channels == 4 {
        Some(Vec::with_capacity(pixel_count))
    } else {
        None
    };

    for i in 0..pixel_count {
        let offset = i * step;
        red_values.push(pixels[offset]);

        if let Some(ref mut green) = green_values {
            green.push(pixels[offset + 1]);
        }
        if let Some(ref mut blue) = blue_values {
            blue.push(pixels[offset + 2]);
        }
        if let Some(ref mut alpha) = alpha_values {
            let alpha_offset = if channels == 2 { 1 } else { 3 };
            alpha.push(pixels[offset + alpha_offset]);
        }
    }

    // Calculate histogram stats
    let red_stats = calculate_channel_stats(&red_values);
    let green_stats = green_values.as_ref().map(|v| calculate_channel_stats(v));
    let blue_stats = blue_values.as_ref().map(|v| calculate_channel_stats(v));
    let alpha_stats = alpha_values.as_ref().map(|v| calculate_channel_stats(v));

    // Calculate contrast
    let contrast = calculate_contrast(&pixels, channels);

    Ok(TextureMetrics {
        format: TextureFormatMetadata {
            width: header.width,
            height: header.height,
            channels,
            color_type: header.color_type_name().to_string(),
            bit_depth: header.bit_depth,
        },
        histogram: TextureHistogramStats {
            red: red_stats,
            green: green_stats,
            blue: blue_stats,
            alpha: alpha_stats,
        },
        contrast,
    })
}

/// Convert TextureMetrics to a BTreeMap for deterministic JSON serialization.
pub fn metrics_to_btree(metrics: &TextureMetrics) -> BTreeMap<String, serde_json::Value> {
    let mut map = BTreeMap::new();

    // Contrast section
    let mut contrast = BTreeMap::new();
    contrast.insert(
        "contrast_ratio".to_string(),
        serde_json::json!(metrics.contrast.contrast_ratio),
    );
    contrast.insert(
        "weber_contrast".to_string(),
        serde_json::json!(metrics.contrast.weber_contrast),
    );
    map.insert("contrast".to_string(), serde_json::json!(contrast));

    // Format section
    let mut format = BTreeMap::new();
    format.insert(
        "bit_depth".to_string(),
        serde_json::json!(metrics.format.bit_depth),
    );
    format.insert(
        "channels".to_string(),
        serde_json::json!(metrics.format.channels),
    );
    format.insert(
        "color_type".to_string(),
        serde_json::json!(metrics.format.color_type),
    );
    format.insert(
        "height".to_string(),
        serde_json::json!(metrics.format.height),
    );
    format.insert("width".to_string(), serde_json::json!(metrics.format.width));
    map.insert("format".to_string(), serde_json::json!(format));

    // Histogram section
    let mut histogram = BTreeMap::new();

    // Alpha (if present)
    if let Some(ref alpha) = metrics.histogram.alpha {
        let mut alpha_map = BTreeMap::new();
        alpha_map.insert("max".to_string(), serde_json::json!(alpha.max));
        alpha_map.insert("mean".to_string(), serde_json::json!(alpha.mean));
        alpha_map.insert("min".to_string(), serde_json::json!(alpha.min));
        alpha_map.insert("stddev".to_string(), serde_json::json!(alpha.stddev));
        histogram.insert("alpha".to_string(), serde_json::json!(alpha_map));
    }

    // Blue (if present)
    if let Some(ref blue) = metrics.histogram.blue {
        let mut blue_map = BTreeMap::new();
        blue_map.insert("max".to_string(), serde_json::json!(blue.max));
        blue_map.insert("mean".to_string(), serde_json::json!(blue.mean));
        blue_map.insert("min".to_string(), serde_json::json!(blue.min));
        blue_map.insert("stddev".to_string(), serde_json::json!(blue.stddev));
        histogram.insert("blue".to_string(), serde_json::json!(blue_map));
    }

    // Green (if present)
    if let Some(ref green) = metrics.histogram.green {
        let mut green_map = BTreeMap::new();
        green_map.insert("max".to_string(), serde_json::json!(green.max));
        green_map.insert("mean".to_string(), serde_json::json!(green.mean));
        green_map.insert("min".to_string(), serde_json::json!(green.min));
        green_map.insert("stddev".to_string(), serde_json::json!(green.stddev));
        histogram.insert("green".to_string(), serde_json::json!(green_map));
    }

    // Red (always present)
    let mut red_map = BTreeMap::new();
    red_map.insert(
        "max".to_string(),
        serde_json::json!(metrics.histogram.red.max),
    );
    red_map.insert(
        "mean".to_string(),
        serde_json::json!(metrics.histogram.red.mean),
    );
    red_map.insert(
        "min".to_string(),
        serde_json::json!(metrics.histogram.red.min),
    );
    red_map.insert(
        "stddev".to_string(),
        serde_json::json!(metrics.histogram.red.stddev),
    );
    histogram.insert("red".to_string(), serde_json::json!(red_map));

    map.insert("histogram".to_string(), serde_json::json!(histogram));

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_png(width: u32, height: u32, color_type: u8, pixels: &[u8]) -> Vec<u8> {
        let mut png_data = Vec::new();

        // Use the png crate to create a valid PNG
        let color_type_png = match color_type {
            0 => png::ColorType::Grayscale,
            2 => png::ColorType::Rgb,
            4 => png::ColorType::GrayscaleAlpha,
            6 => png::ColorType::Rgba,
            _ => png::ColorType::Rgba,
        };

        let mut encoder = png::Encoder::new(&mut png_data, width, height);
        encoder.set_color(color_type_png);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(pixels).unwrap();
        drop(writer);

        png_data
    }

    #[test]
    fn test_analyze_rgba_image() {
        // 2x2 RGBA image with different colors
        let pixels: Vec<u8> = vec![
            255, 0, 0, 255, // Red
            0, 255, 0, 255, // Green
            0, 0, 255, 255, // Blue
            255, 255, 255, 255, // White
        ];
        let png = create_test_png(2, 2, 6, &pixels);
        let metrics = analyze_png(&png).unwrap();

        assert_eq!(metrics.format.width, 2);
        assert_eq!(metrics.format.height, 2);
        assert_eq!(metrics.format.channels, 4);
        assert_eq!(metrics.format.color_type, "RGBA");
        assert!(metrics.histogram.green.is_some());
        assert!(metrics.histogram.blue.is_some());
        assert!(metrics.histogram.alpha.is_some());
    }

    #[test]
    fn test_analyze_grayscale_image() {
        // 2x2 grayscale image
        let pixels: Vec<u8> = vec![0, 128, 200, 255];
        let png = create_test_png(2, 2, 0, &pixels);
        let metrics = analyze_png(&png).unwrap();

        assert_eq!(metrics.format.width, 2);
        assert_eq!(metrics.format.height, 2);
        assert_eq!(metrics.format.channels, 1);
        assert_eq!(metrics.format.color_type, "Grayscale");
        assert!(metrics.histogram.green.is_none());
        assert!(metrics.histogram.blue.is_none());
        assert!(metrics.histogram.alpha.is_none());
    }

    #[test]
    fn test_channel_stats() {
        let values: Vec<u8> = vec![0, 50, 100, 150, 200, 250];
        let stats = calculate_channel_stats(&values);

        assert_eq!(stats.min, 0);
        assert_eq!(stats.max, 250);
        assert!(stats.mean > 100.0 && stats.mean < 150.0);
        assert!(stats.stddev > 0.0);
    }

    #[test]
    fn test_deterministic_output() {
        let pixels: Vec<u8> = vec![
            100, 150, 200, 255, 50, 75, 100, 255, 200, 180, 160, 255, 25, 50, 75, 128,
        ];
        let png = create_test_png(2, 2, 6, &pixels);

        let metrics1 = analyze_png(&png).unwrap();
        let metrics2 = analyze_png(&png).unwrap();

        let json1 = serde_json::to_string(&metrics_to_btree(&metrics1)).unwrap();
        let json2 = serde_json::to_string(&metrics_to_btree(&metrics2)).unwrap();

        assert_eq!(json1, json2);
    }

    #[test]
    fn test_metrics_to_btree_sorted_keys() {
        let pixels: Vec<u8> = vec![128, 128, 128, 255];
        let png = create_test_png(1, 1, 6, &pixels);
        let metrics = analyze_png(&png).unwrap();

        let btree = metrics_to_btree(&metrics);
        let keys: Vec<_> = btree.keys().collect();

        // Keys should be alphabetically sorted
        assert_eq!(keys, vec!["contrast", "format", "histogram"]);
    }

    #[test]
    fn test_invalid_png() {
        let invalid_data = vec![0u8; 100];
        let result = analyze_png(&invalid_data);
        assert!(result.is_err());
    }
}
