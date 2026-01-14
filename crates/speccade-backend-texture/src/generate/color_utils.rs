//! Color manipulation utilities for texture generation.
//!
//! Provides functions for color parsing, palette quantization, color ramps,
//! and emissive effects.

use crate::color::Color;
use crate::maps::{GrayscaleBuffer, TextureBuffer};

use super::GenerateError;

/// Parse a list of hex color strings into Color objects.
pub fn parse_hex_color_list(colors: &[String], name: &str) -> Result<Vec<Color>, GenerateError> {
    if colors.is_empty() {
        return Err(GenerateError::InvalidParameter(format!(
            "{} must contain at least 1 color",
            name
        )));
    }

    colors
        .iter()
        .enumerate()
        .map(|(i, color)| {
            Color::from_hex_rgb(color).map_err(|e| {
                GenerateError::InvalidParameter(format!("{}[{}] '{}': {}", name, i, color, e))
            })
        })
        .collect()
}

/// Sample a color from a ramp at position t (0.0 to 1.0).
pub fn sample_color_ramp(ramp: &[Color], t: f64) -> Color {
    debug_assert!(!ramp.is_empty(), "ramp must not be empty");
    if ramp.len() == 1 {
        return ramp[0];
    }

    let t = t.clamp(0.0, 1.0);
    let scaled = t * (ramp.len() - 1) as f64;
    let idx = scaled.floor() as usize;
    let frac = scaled - idx as f64;

    if idx >= ramp.len() - 1 {
        return ramp[ramp.len() - 1];
    }

    ramp[idx].lerp(&ramp[idx + 1], frac)
}

/// Apply a color ramp to a texture buffer based on luminance.
pub fn apply_color_ramp(buffer: &mut TextureBuffer, ramp: &[Color]) {
    if ramp.is_empty() {
        return;
    }

    for pixel in &mut buffer.data {
        let a = pixel.a;
        let mapped = sample_color_ramp(ramp, pixel.luminance());
        *pixel = Color::rgba(mapped.r, mapped.g, mapped.b, a);
    }
}

/// Apply palette quantization by mapping each pixel to the nearest palette color.
pub fn apply_palette_quantization(buffer: &mut TextureBuffer, palette: &[Color]) {
    if palette.is_empty() {
        return;
    }

    for pixel in &mut buffer.data {
        let a = pixel.a;
        let mapped = nearest_palette_color(palette, *pixel);
        *pixel = Color::rgba(mapped.r, mapped.g, mapped.b, a);
    }
}

/// Find the nearest color in the palette to the given color.
fn nearest_palette_color(palette: &[Color], color: Color) -> Color {
    debug_assert!(!palette.is_empty(), "palette must not be empty");

    let mut best = palette[0];
    let mut best_dist = color_distance_sq(best, color);

    for &candidate in palette.iter().skip(1) {
        let dist = color_distance_sq(candidate, color);
        if dist < best_dist {
            best_dist = dist;
            best = candidate;
        }
    }

    best
}

/// Calculate squared euclidean distance between two colors in RGB space.
fn color_distance_sq(a: Color, b: Color) -> f64 {
    let dr = a.r - b.r;
    let dg = a.g - b.g;
    let db = a.b - b.b;
    dr * dr + dg * dg + db * db
}

/// Add emissive glow from a mask to a texture buffer.
///
/// The mask determines the strength of emission at each pixel,
/// and the color is the emissive glow color.
pub fn add_emissive_from_mask(buffer: &mut TextureBuffer, mask: &GrayscaleBuffer, color: Color) {
    if buffer.width != mask.width || buffer.height != mask.height {
        return;
    }

    for y in 0..mask.height {
        for x in 0..mask.width {
            let t = mask.get(x, y);
            if t <= 0.0 {
                continue;
            }

            let current = buffer.get(x, y);
            let added = color.scale(t);
            buffer.set(x, y, current.add(&added).clamp());
        }
    }
}
