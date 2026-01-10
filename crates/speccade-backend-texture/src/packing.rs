//! Channel packing utilities for combining multiple texture maps into a single RGBA texture.
//!
//! This module provides functionality to pack multiple grayscale or color texture maps
//! into the channels of a single RGBA texture. This is commonly used to pack PBR material
//! maps (e.g., metallic, roughness, AO) into a single texture for efficient GPU sampling.
//!
//! # Example
//!
//! ```ignore
//! use speccade_backend_texture::packing::{PackedChannels, ChannelSource, pack_channels};
//! use speccade_backend_texture::{TextureBuffer, Color};
//! use std::collections::HashMap;
//!
//! let mut maps = HashMap::new();
//! maps.insert("roughness".to_string(), roughness_buffer);
//! maps.insert("metallic".to_string(), metallic_buffer);
//! maps.insert("ao".to_string(), ao_buffer);
//!
//! let packed = PackedChannels {
//!     r: ChannelSource::Key("roughness".to_string()),
//!     g: ChannelSource::Key("metallic".to_string()),
//!     b: ChannelSource::Key("ao".to_string()),
//!     a: Some(ChannelSource::Constant { constant: 1.0 }),
//! };
//!
//! let result = pack_channels(&packed, &maps, 512, 512)?;
//! ```

use std::collections::HashMap;

use crate::color::Color;
use crate::maps::TextureBuffer;

/// Color component to extract from a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorComponent {
    /// Red channel
    R,
    /// Green channel
    G,
    /// Blue channel
    B,
    /// Alpha channel
    A,
    /// Luminance (weighted average of RGB)
    #[default]
    Luminance,
}

/// Source for a channel in a packed texture.
///
/// This can be a simple key reference to another map, an extended reference
/// with component and inversion options, or a constant value.
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelSource {
    /// Simple key reference - uses luminance of the referenced map
    Key(String),
    /// Extended reference with component selection and optional inversion
    Extended {
        /// Key of the source map
        key: String,
        /// Which component to extract (defaults to Luminance if None)
        component: Option<ColorComponent>,
        /// Whether to invert the value (1.0 - value)
        invert: bool,
    },
    /// Constant value for the channel
    Constant {
        /// The constant value (0.0 to 1.0)
        constant: f64,
    },
}

impl ChannelSource {
    /// Create a simple key reference.
    pub fn key(key: impl Into<String>) -> Self {
        ChannelSource::Key(key.into())
    }

    /// Create an extended reference.
    pub fn extended(key: impl Into<String>, component: Option<ColorComponent>, invert: bool) -> Self {
        ChannelSource::Extended {
            key: key.into(),
            component,
            invert,
        }
    }

    /// Create a constant source.
    pub fn constant(value: f64) -> Self {
        ChannelSource::Constant { constant: value }
    }
}

/// Specification for packing multiple maps into RGBA channels.
#[derive(Debug, Clone, PartialEq)]
pub struct PackedChannels {
    /// Source for the red channel
    pub r: ChannelSource,
    /// Source for the green channel
    pub g: ChannelSource,
    /// Source for the blue channel
    pub b: ChannelSource,
    /// Source for the alpha channel (defaults to 1.0 if None)
    pub a: Option<ChannelSource>,
}

impl PackedChannels {
    /// Create a new packed channels specification.
    pub fn new(r: ChannelSource, g: ChannelSource, b: ChannelSource) -> Self {
        Self { r, g, b, a: None }
    }

    /// Create with an alpha channel source.
    pub fn with_alpha(r: ChannelSource, g: ChannelSource, b: ChannelSource, a: ChannelSource) -> Self {
        Self {
            r,
            g,
            b,
            a: Some(a),
        }
    }
}

/// Errors that can occur during channel packing.
#[derive(Debug, thiserror::Error)]
pub enum PackingError {
    /// A referenced map was not found in the maps collection.
    #[error("Referenced map '{0}' not found")]
    MissingMap(String),
}

/// Extract a single channel value from a texture buffer at given coordinates.
///
/// # Arguments
///
/// * `buffer` - The texture buffer to sample from
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `component` - Which color component to extract
///
/// # Returns
///
/// The extracted channel value in the range [0.0, 1.0]
pub fn extract_channel(
    buffer: &TextureBuffer,
    x: u32,
    y: u32,
    component: ColorComponent,
) -> f64 {
    let color = buffer.get(x, y);
    match component {
        ColorComponent::R => color.r,
        ColorComponent::G => color.g,
        ColorComponent::B => color.b,
        ColorComponent::A => color.a,
        ColorComponent::Luminance => {
            // Standard luminance formula (Rec. 601)
            0.299 * color.r + 0.587 * color.g + 0.114 * color.b
        }
    }
}

/// Resolve a channel source to a value at given coordinates.
///
/// # Arguments
///
/// * `source` - The channel source specification
/// * `maps` - Collection of available texture maps keyed by name
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
///
/// The resolved channel value, or an error if a referenced map is not found
pub fn resolve_channel_source(
    source: &ChannelSource,
    maps: &HashMap<String, TextureBuffer>,
    x: u32,
    y: u32,
) -> Result<f64, PackingError> {
    match source {
        ChannelSource::Key(key) => {
            let buffer = maps
                .get(key)
                .ok_or_else(|| PackingError::MissingMap(key.clone()))?;
            // Default: luminance for RGB, first channel for grayscale
            Ok(extract_channel(buffer, x, y, ColorComponent::Luminance))
        }
        ChannelSource::Extended {
            key,
            component,
            invert,
        } => {
            let buffer = maps
                .get(key)
                .ok_or_else(|| PackingError::MissingMap(key.clone()))?;
            let comp = component.unwrap_or(ColorComponent::Luminance);
            let value = extract_channel(buffer, x, y, comp);
            Ok(if *invert { 1.0 - value } else { value })
        }
        ChannelSource::Constant { constant } => Ok(*constant),
    }
}

/// Pack multiple maps into a single RGBA texture.
///
/// This function creates a new texture buffer by sampling from multiple source maps
/// according to the channel packing specification. Each channel of the output texture
/// can come from a different source map, a specific component of a source map, or
/// a constant value.
///
/// # Arguments
///
/// * `packed` - The channel packing specification
/// * `maps` - Collection of available texture maps keyed by name
/// * `width` - Width of the output texture
/// * `height` - Height of the output texture
///
/// # Returns
///
/// A new texture buffer with the packed channels, or an error if a referenced map is not found
///
/// # Example
///
/// ```ignore
/// let packed = PackedChannels::new(
///     ChannelSource::key("roughness"),
///     ChannelSource::key("metallic"),
///     ChannelSource::key("ao"),
/// );
/// let result = pack_channels(&packed, &maps, 512, 512)?;
/// ```
pub fn pack_channels(
    packed: &PackedChannels,
    maps: &HashMap<String, TextureBuffer>,
    width: u32,
    height: u32,
) -> Result<TextureBuffer, PackingError> {
    let mut result = TextureBuffer::new(width, height, Color::black());

    for y in 0..height {
        for x in 0..width {
            let r = resolve_channel_source(&packed.r, maps, x, y)?;
            let g = resolve_channel_source(&packed.g, maps, x, y)?;
            let b = resolve_channel_source(&packed.b, maps, x, y)?;
            let a = match &packed.a {
                Some(src) => resolve_channel_source(src, maps, x, y)?,
                None => 1.0,
            };
            result.set(x, y, Color::rgba(r, g, b, a));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a solid color texture buffer
    fn create_solid_buffer(width: u32, height: u32, color: Color) -> TextureBuffer {
        TextureBuffer::new(width, height, color)
    }

    /// Helper to create a gradient buffer (varies in x direction)
    fn create_gradient_buffer(width: u32, height: u32) -> TextureBuffer {
        let mut buffer = TextureBuffer::new(width, height, Color::black());
        for y in 0..height {
            for x in 0..width {
                let t = x as f64 / (width - 1) as f64;
                buffer.set(x, y, Color::gray(t));
            }
        }
        buffer
    }

    #[test]
    fn test_constant_channel_source() {
        let maps: HashMap<String, TextureBuffer> = HashMap::new();
        let source = ChannelSource::Constant { constant: 0.5 };

        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_constant_channel_source_various_values() {
        let maps: HashMap<String, TextureBuffer> = HashMap::new();

        for &expected in &[0.0, 0.25, 0.5, 0.75, 1.0] {
            let source = ChannelSource::constant(expected);
            let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
            assert!(
                (value - expected).abs() < 1e-10,
                "Expected {}, got {}",
                expected,
                value
            );
        }
    }

    #[test]
    fn test_key_reference_with_luminance() {
        let mut maps = HashMap::new();
        // White has luminance of 1.0
        maps.insert("test".to_string(), create_solid_buffer(4, 4, Color::white()));

        let source = ChannelSource::Key("test".to_string());
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_key_reference_gray() {
        let mut maps = HashMap::new();
        maps.insert("gray".to_string(), create_solid_buffer(4, 4, Color::gray(0.5)));

        let source = ChannelSource::key("gray");
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_component_extraction_r() {
        let mut maps = HashMap::new();
        maps.insert(
            "color".to_string(),
            create_solid_buffer(4, 4, Color::rgb(0.8, 0.3, 0.5)),
        );

        let source = ChannelSource::extended("color", Some(ColorComponent::R), false);
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_component_extraction_g() {
        let mut maps = HashMap::new();
        maps.insert(
            "color".to_string(),
            create_solid_buffer(4, 4, Color::rgb(0.8, 0.3, 0.5)),
        );

        let source = ChannelSource::extended("color", Some(ColorComponent::G), false);
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_component_extraction_b() {
        let mut maps = HashMap::new();
        maps.insert(
            "color".to_string(),
            create_solid_buffer(4, 4, Color::rgb(0.8, 0.3, 0.5)),
        );

        let source = ChannelSource::extended("color", Some(ColorComponent::B), false);
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_component_extraction_a() {
        let mut maps = HashMap::new();
        maps.insert(
            "color".to_string(),
            create_solid_buffer(4, 4, Color::rgba(0.8, 0.3, 0.5, 0.7)),
        );

        let source = ChannelSource::extended("color", Some(ColorComponent::A), false);
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_inversion() {
        let mut maps = HashMap::new();
        maps.insert("test".to_string(), create_solid_buffer(4, 4, Color::gray(0.3)));

        let source = ChannelSource::Extended {
            key: "test".to_string(),
            component: Some(ColorComponent::Luminance),
            invert: true,
        };
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_inversion_black_becomes_white() {
        let mut maps = HashMap::new();
        maps.insert("black".to_string(), create_solid_buffer(4, 4, Color::black()));

        let source = ChannelSource::extended("black", Some(ColorComponent::R), true);
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_inversion_white_becomes_black() {
        let mut maps = HashMap::new();
        maps.insert("white".to_string(), create_solid_buffer(4, 4, Color::white()));

        let source = ChannelSource::extended("white", Some(ColorComponent::R), true);
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!(value.abs() < 1e-10);
    }

    #[test]
    fn test_missing_map_error() {
        let maps: HashMap<String, TextureBuffer> = HashMap::new();
        let source = ChannelSource::Key("nonexistent".to_string());

        let result = resolve_channel_source(&source, &maps, 0, 0);
        assert!(result.is_err());

        match result {
            Err(PackingError::MissingMap(key)) => assert_eq!(key, "nonexistent"),
            _ => panic!("Expected MissingMap error"),
        }
    }

    #[test]
    fn test_missing_map_error_message() {
        let error = PackingError::MissingMap("my_map".to_string());
        assert!(error.to_string().contains("my_map"));
        assert!(error.to_string().contains("not found"));
    }

    #[test]
    fn test_pack_channels_with_constants() {
        let maps: HashMap<String, TextureBuffer> = HashMap::new();

        let packed = PackedChannels {
            r: ChannelSource::constant(0.2),
            g: ChannelSource::constant(0.4),
            b: ChannelSource::constant(0.6),
            a: Some(ChannelSource::constant(0.8)),
        };

        let result = pack_channels(&packed, &maps, 4, 4).unwrap();

        // Check a pixel
        let color = result.get(0, 0);
        assert!((color.r - 0.2).abs() < 1e-10);
        assert!((color.g - 0.4).abs() < 1e-10);
        assert!((color.b - 0.6).abs() < 1e-10);
        assert!((color.a - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_pack_channels_default_alpha() {
        let maps: HashMap<String, TextureBuffer> = HashMap::new();

        let packed = PackedChannels::new(
            ChannelSource::constant(0.5),
            ChannelSource::constant(0.5),
            ChannelSource::constant(0.5),
        );

        let result = pack_channels(&packed, &maps, 4, 4).unwrap();

        // Alpha should default to 1.0
        let color = result.get(0, 0);
        assert!((color.a - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_pack_channels_with_multiple_sources() {
        let mut maps = HashMap::new();
        maps.insert(
            "roughness".to_string(),
            create_solid_buffer(8, 8, Color::gray(0.3)),
        );
        maps.insert(
            "metallic".to_string(),
            create_solid_buffer(8, 8, Color::gray(1.0)),
        );
        maps.insert(
            "ao".to_string(),
            create_solid_buffer(8, 8, Color::gray(0.8)),
        );

        let packed = PackedChannels::new(
            ChannelSource::key("roughness"),
            ChannelSource::key("metallic"),
            ChannelSource::key("ao"),
        );

        let result = pack_channels(&packed, &maps, 8, 8).unwrap();

        let color = result.get(4, 4);
        assert!((color.r - 0.3).abs() < 1e-10, "Red (roughness) should be 0.3");
        assert!((color.g - 1.0).abs() < 1e-10, "Green (metallic) should be 1.0");
        assert!((color.b - 0.8).abs() < 1e-10, "Blue (ao) should be 0.8");
    }

    #[test]
    fn test_pack_channels_with_inversion() {
        let mut maps = HashMap::new();
        maps.insert("smooth".to_string(), create_solid_buffer(4, 4, Color::gray(0.2)));

        let packed = PackedChannels::new(
            // Convert smoothness to roughness by inverting
            ChannelSource::extended("smooth", Some(ColorComponent::Luminance), true),
            ChannelSource::constant(0.0),
            ChannelSource::constant(1.0),
        );

        let result = pack_channels(&packed, &maps, 4, 4).unwrap();

        let color = result.get(0, 0);
        assert!((color.r - 0.8).abs() < 1e-10, "Inverted 0.2 should be 0.8");
    }

    #[test]
    fn test_pack_channels_missing_map() {
        let maps: HashMap<String, TextureBuffer> = HashMap::new();

        let packed = PackedChannels::new(
            ChannelSource::key("missing"),
            ChannelSource::constant(0.0),
            ChannelSource::constant(0.0),
        );

        let result = pack_channels(&packed, &maps, 4, 4);
        assert!(result.is_err());
    }

    #[test]
    fn test_pack_channels_output_dimensions() {
        let maps: HashMap<String, TextureBuffer> = HashMap::new();

        let packed = PackedChannels::new(
            ChannelSource::constant(0.5),
            ChannelSource::constant(0.5),
            ChannelSource::constant(0.5),
        );

        let result = pack_channels(&packed, &maps, 128, 64).unwrap();
        assert_eq!(result.width, 128);
        assert_eq!(result.height, 64);
    }

    #[test]
    fn test_pack_channels_with_gradient() {
        let mut maps = HashMap::new();
        maps.insert("gradient".to_string(), create_gradient_buffer(16, 16));

        let packed = PackedChannels::new(
            ChannelSource::key("gradient"),
            ChannelSource::constant(0.0),
            ChannelSource::constant(0.0),
        );

        let result = pack_channels(&packed, &maps, 16, 16).unwrap();

        // Check that gradient is preserved
        let left = result.get(0, 0);
        let right = result.get(15, 0);

        assert!(left.r < 0.1, "Left side should be near black");
        assert!(right.r > 0.9, "Right side should be near white");
    }

    #[test]
    fn test_extract_channel_luminance_formula() {
        // Test the luminance formula with pure colors
        let mut buffer = TextureBuffer::new(1, 1, Color::black());

        // Pure red
        buffer.set(0, 0, Color::rgb(1.0, 0.0, 0.0));
        let lum_r = extract_channel(&buffer, 0, 0, ColorComponent::Luminance);
        assert!((lum_r - 0.299).abs() < 1e-10);

        // Pure green
        buffer.set(0, 0, Color::rgb(0.0, 1.0, 0.0));
        let lum_g = extract_channel(&buffer, 0, 0, ColorComponent::Luminance);
        assert!((lum_g - 0.587).abs() < 1e-10);

        // Pure blue
        buffer.set(0, 0, Color::rgb(0.0, 0.0, 1.0));
        let lum_b = extract_channel(&buffer, 0, 0, ColorComponent::Luminance);
        assert!((lum_b - 0.114).abs() < 1e-10);
    }

    #[test]
    fn test_color_component_default() {
        assert_eq!(ColorComponent::default(), ColorComponent::Luminance);
    }

    #[test]
    fn test_channel_source_constructors() {
        // Test key constructor
        let key_source = ChannelSource::key("test");
        assert!(matches!(key_source, ChannelSource::Key(k) if k == "test"));

        // Test extended constructor
        let ext_source = ChannelSource::extended("test", Some(ColorComponent::R), true);
        assert!(matches!(
            ext_source,
            ChannelSource::Extended {
                key,
                component: Some(ColorComponent::R),
                invert: true
            } if key == "test"
        ));

        // Test constant constructor
        let const_source = ChannelSource::constant(0.75);
        assert!(matches!(const_source, ChannelSource::Constant { constant } if (constant - 0.75).abs() < 1e-10));
    }

    #[test]
    fn test_packed_channels_constructors() {
        let packed = PackedChannels::new(
            ChannelSource::constant(0.1),
            ChannelSource::constant(0.2),
            ChannelSource::constant(0.3),
        );
        assert!(packed.a.is_none());

        let packed_with_alpha = PackedChannels::with_alpha(
            ChannelSource::constant(0.1),
            ChannelSource::constant(0.2),
            ChannelSource::constant(0.3),
            ChannelSource::constant(0.4),
        );
        assert!(packed_with_alpha.a.is_some());
    }

    #[test]
    fn test_pack_channels_all_pixels() {
        let mut maps = HashMap::new();
        // Create a buffer with varying values
        let mut test_buffer = TextureBuffer::new(4, 4, Color::black());
        for y in 0..4 {
            for x in 0..4 {
                let v = (x + y * 4) as f64 / 15.0;
                test_buffer.set(x, y, Color::gray(v));
            }
        }
        maps.insert("test".to_string(), test_buffer);

        let packed = PackedChannels::new(
            ChannelSource::key("test"),
            ChannelSource::key("test"),
            ChannelSource::key("test"),
        );

        let result = pack_channels(&packed, &maps, 4, 4).unwrap();

        // Verify all pixels are correctly packed
        for y in 0..4 {
            for x in 0..4 {
                let expected = (x + y * 4) as f64 / 15.0;
                let color = result.get(x, y);
                assert!(
                    (color.r - expected).abs() < 1e-10,
                    "Pixel ({}, {}) should have value {}",
                    x,
                    y,
                    expected
                );
            }
        }
    }
}
