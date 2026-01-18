//! Color transformation operations (to_grayscale, color_ramp, palette, compose_rgba, normal_from_height).

use crate::color::Color;
use crate::maps::{GrayscaleBuffer, NormalGenerator, TextureBuffer};

use super::super::GenerateError;
use super::helpers::{nearest_palette_color, parse_hex_color_list, sample_color_ramp};
use super::GraphValue;

/// Convert a color buffer to grayscale using luminance.
pub(super) fn eval_to_grayscale(input: &TextureBuffer, width: u32, height: u32) -> GraphValue {
    let mut out = GrayscaleBuffer::new(width, height, 0.0);
    for i in 0..out.data.len() {
        out.data[i] = input.data[i].luminance();
    }
    GraphValue::Grayscale(out)
}

/// Map grayscale values to colors using a color ramp.
pub(super) fn eval_color_ramp(
    input: &GrayscaleBuffer,
    width: u32,
    height: u32,
    ramp_colors: &[String],
) -> Result<GraphValue, GenerateError> {
    let ramp = parse_hex_color_list(ramp_colors, "ramp")?;
    let mut out = TextureBuffer::new(width, height, Color::black());
    for i in 0..input.data.len() {
        let mapped = sample_color_ramp(&ramp, input.data[i]);
        out.data[i] = Color::rgba(mapped.r, mapped.g, mapped.b, 1.0);
    }
    Ok(GraphValue::Color(out))
}

/// Quantize colors to nearest palette color.
pub(super) fn eval_palette(
    input: &TextureBuffer,
    palette_colors: &[String],
) -> Result<GraphValue, GenerateError> {
    let palette = parse_hex_color_list(palette_colors, "palette")?;
    let mut out = input.clone();
    for pixel in &mut out.data {
        let a = pixel.a;
        let mapped = nearest_palette_color(&palette, *pixel);
        *pixel = Color::rgba(mapped.r, mapped.g, mapped.b, a);
    }
    Ok(GraphValue::Color(out))
}

/// Compose RGBA from separate grayscale channels.
pub(super) fn eval_compose_rgba(
    r: &GrayscaleBuffer,
    g: &GrayscaleBuffer,
    b: &GrayscaleBuffer,
    a: Option<&GrayscaleBuffer>,
    width: u32,
    height: u32,
) -> GraphValue {
    let mut out = TextureBuffer::new(width, height, Color::black());
    for i in 0..out.data.len() {
        let alpha = a.map_or(1.0, |buf| buf.data[i]);
        out.data[i] = Color::rgba(r.data[i], g.data[i], b.data[i], alpha);
    }
    GraphValue::Color(out)
}

/// Generate a normal map from a height map.
pub(super) fn eval_normal_from_height(input: &GrayscaleBuffer, strength: f64) -> GraphValue {
    let out = NormalGenerator::new()
        .with_strength(strength)
        .generate_from_height(input);
    GraphValue::Color(out)
}
