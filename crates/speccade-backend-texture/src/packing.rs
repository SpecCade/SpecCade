//! Channel packing utilities for combining multiple texture maps into a single RGBA texture.
//!
//! This module packs values sampled from source maps into the channels of a new RGBA buffer.
//! The packing *specification* (which map feeds which channel) is defined by the canonical
//! SpecCade spec types in `speccade-spec`.
//!
//! # Example
//!
//! ```ignore
//! use speccade_backend_texture::{pack_channels, TextureBuffer};
//! use speccade_spec::recipe::texture::{ChannelSource, PackedChannels};
//! use std::collections::HashMap;
//!
//! let mut maps: HashMap<String, TextureBuffer> = HashMap::new();
//! // maps.insert("roughness".to_string(), roughness_buffer);
//! // maps.insert("metallic".to_string(), metallic_buffer);
//! // maps.insert("ao".to_string(), ao_buffer);
//!
//! let packed = PackedChannels::rgb(
//!     ChannelSource::key("roughness"),
//!     ChannelSource::key("metallic"),
//!     ChannelSource::key("ao"),
//! );
//! let result = pack_channels(&packed, &maps, 512, 512)?;
//! # Ok::<(), speccade_backend_texture::PackingError>(())
//! ```

use std::collections::HashMap;

use crate::color::Color;
use crate::maps::TextureBuffer;

// Re-export canonical packing types from the spec crate (SSOT).
pub use speccade_spec::recipe::texture::{ChannelSource, ColorComponent, PackedChannels};

/// Errors that can occur during channel packing.
#[derive(Debug, thiserror::Error)]
pub enum PackingError {
    /// A referenced map was not found in the maps collection.
    #[error("Referenced map '{0}' not found")]
    MissingMap(String),
}

/// Extract a single channel value from a texture buffer at given coordinates.
///
/// Returns the extracted channel value in the range [0.0, 1.0].
pub fn extract_channel(buffer: &TextureBuffer, x: u32, y: u32, component: ColorComponent) -> f64 {
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
            let component = component.unwrap_or(ColorComponent::Luminance);
            let value = extract_channel(buffer, x, y, component);
            Ok(if *invert { 1.0 - value } else { value })
        }
        ChannelSource::Constant { constant } => Ok(*constant as f64),
    }
}

/// Pack multiple maps into a single RGBA texture.
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

    /// Helper to create a solid color texture buffer.
    fn create_solid_buffer(width: u32, height: u32, color: Color) -> TextureBuffer {
        TextureBuffer::new(width, height, color)
    }

    /// Helper to create a gradient buffer (varies in x direction).
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
        let source = ChannelSource::constant(0.5);

        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_constant_channel_source_various_values() {
        let maps: HashMap<String, TextureBuffer> = HashMap::new();

        for &expected in &[0.0f32, 0.25, 0.5, 0.75, 1.0] {
            let source = ChannelSource::constant(expected);
            let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
            assert!(
                (value - expected as f64).abs() < 1e-10,
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
        maps.insert(
            "test".to_string(),
            create_solid_buffer(4, 4, Color::white()),
        );

        let source = ChannelSource::Key("test".to_string());
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_key_reference_gray() {
        let mut maps = HashMap::new();
        maps.insert(
            "gray".to_string(),
            create_solid_buffer(4, 4, Color::gray(0.5)),
        );

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

        let source = ChannelSource::extended("color")
            .component(ColorComponent::R)
            .build();
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

        let source = ChannelSource::extended("color")
            .component(ColorComponent::G)
            .build();
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

        let source = ChannelSource::extended("color")
            .component(ColorComponent::B)
            .build();
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

        let source = ChannelSource::extended("color")
            .component(ColorComponent::A)
            .build();
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_inversion() {
        let mut maps = HashMap::new();
        maps.insert(
            "test".to_string(),
            create_solid_buffer(4, 4, Color::gray(0.3)),
        );

        let source = ChannelSource::extended("test")
            .component(ColorComponent::Luminance)
            .invert(true)
            .build();
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_inversion_black_becomes_white() {
        let mut maps = HashMap::new();
        maps.insert(
            "black".to_string(),
            create_solid_buffer(4, 4, Color::black()),
        );

        let source = ChannelSource::extended("black")
            .component(ColorComponent::R)
            .invert(true)
            .build();
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!((value - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_inversion_white_becomes_black() {
        let mut maps = HashMap::new();
        maps.insert(
            "white".to_string(),
            create_solid_buffer(4, 4, Color::white()),
        );

        let source = ChannelSource::extended("white")
            .component(ColorComponent::R)
            .invert(true)
            .build();
        let value = resolve_channel_source(&source, &maps, 0, 0).unwrap();
        assert!(value < 1e-10);
    }

    #[test]
    fn test_pack_channels_basic() {
        let maps: HashMap<String, TextureBuffer> = HashMap::new();

        let packed = PackedChannels::rgb(
            ChannelSource::constant(0.5),
            ChannelSource::constant(0.25),
            ChannelSource::constant(0.75),
        );

        let result = pack_channels(&packed, &maps, 4, 4).unwrap();
        let pixel = result.get(0, 0);

        assert!((pixel.r - 0.5).abs() < 1e-10);
        assert!((pixel.g - 0.25).abs() < 1e-10);
        assert!((pixel.b - 0.75).abs() < 1e-10);
        assert!((pixel.a - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_pack_channels_with_alpha() {
        let maps: HashMap<String, TextureBuffer> = HashMap::new();

        let packed = PackedChannels::rgba(
            ChannelSource::constant(0.1),
            ChannelSource::constant(0.2),
            ChannelSource::constant(0.3),
            ChannelSource::constant(0.4),
        );

        let result = pack_channels(&packed, &maps, 1, 1).unwrap();
        let pixel = result.get(0, 0);
        // ChannelSource::Constant uses f32; compare with a tolerance appropriate for f32->f64.
        assert!((pixel.a - (0.4f32 as f64)).abs() < 1e-6);
    }

    #[test]
    fn test_pack_channels_missing_map() {
        let maps: HashMap<String, TextureBuffer> = HashMap::new();

        let packed = PackedChannels::rgb(
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

        let packed = PackedChannels::rgb(
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

        let packed = PackedChannels::rgb(
            ChannelSource::key("gradient"),
            ChannelSource::constant(0.0),
            ChannelSource::constant(0.0),
        );

        let result = pack_channels(&packed, &maps, 16, 16).unwrap();

        // Check that gradient is preserved.
        let left = result.get(0, 0);
        let right = result.get(15, 0);

        assert!(left.r < 0.1, "Left side should be near black");
        assert!(right.r > 0.9, "Right side should be near white");
    }

    #[test]
    fn test_extract_channel_luminance_formula() {
        // Test the luminance formula with pure colors.
        let mut buffer = TextureBuffer::new(1, 1, Color::black());

        // Pure red.
        buffer.set(0, 0, Color::rgb(1.0, 0.0, 0.0));
        let lum_r = extract_channel(&buffer, 0, 0, ColorComponent::Luminance);
        assert!((lum_r - 0.299).abs() < 1e-10);

        // Pure green.
        buffer.set(0, 0, Color::rgb(0.0, 1.0, 0.0));
        let lum_g = extract_channel(&buffer, 0, 0, ColorComponent::Luminance);
        assert!((lum_g - 0.587).abs() < 1e-10);

        // Pure blue.
        buffer.set(0, 0, Color::rgb(0.0, 0.0, 1.0));
        let lum_b = extract_channel(&buffer, 0, 0, ColorComponent::Luminance);
        assert!((lum_b - 0.114).abs() < 1e-10);
    }

    #[test]
    fn test_channel_source_constructors() {
        // Key constructor.
        let key_source = ChannelSource::key("test");
        assert!(matches!(key_source, ChannelSource::Key(k) if k == "test"));

        // Extended builder.
        let ext_source = ChannelSource::extended("test")
            .component(ColorComponent::R)
            .invert(true)
            .build();
        assert!(matches!(
            ext_source,
            ChannelSource::Extended {
                key,
                component: Some(ColorComponent::R),
                invert: true
            } if key == "test"
        ));

        // Constant constructor.
        let const_source = ChannelSource::constant(0.75);
        assert!(matches!(
            const_source,
            ChannelSource::Constant { constant } if (constant - 0.75).abs() < 1e-6
        ));
    }

    #[test]
    fn test_packed_channels_constructors() {
        let packed = PackedChannels::rgb(
            ChannelSource::constant(0.1),
            ChannelSource::constant(0.2),
            ChannelSource::constant(0.3),
        );
        assert!(packed.a.is_none());

        let packed_with_alpha = PackedChannels::rgba(
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
        // Create a buffer with varying values.
        let mut test_buffer = TextureBuffer::new(4, 4, Color::black());
        for y in 0..4 {
            for x in 0..4 {
                let v = (x + y * 4) as f64 / 15.0;
                test_buffer.set(x, y, Color::gray(v));
            }
        }
        maps.insert("test".to_string(), test_buffer);

        let packed = PackedChannels::rgb(
            ChannelSource::key("test"),
            ChannelSource::key("test"),
            ChannelSource::key("test"),
        );

        let result = pack_channels(&packed, &maps, 4, 4).unwrap();

        // Verify all pixels are correctly packed.
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
