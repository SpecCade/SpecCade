//! Helper functions for graph evaluation.

use std::collections::HashMap;

use crate::color::Color;
use crate::maps::{GrayscaleBuffer, TextureBuffer};

use super::super::GenerateError;
use super::GraphValue;

pub(super) fn expect_gray<'a>(
    cache: &'a HashMap<&str, GraphValue>,
    id: &str,
) -> Result<&'a GrayscaleBuffer, GenerateError> {
    cache
        .get(id)
        .and_then(GraphValue::as_grayscale)
        .ok_or_else(|| {
            GenerateError::InvalidParameter(format!(
                "node '{}' produced color output but grayscale was required",
                id
            ))
        })
}

pub(super) fn expect_color<'a>(
    cache: &'a HashMap<&str, GraphValue>,
    id: &str,
) -> Result<&'a TextureBuffer, GenerateError> {
    cache.get(id).and_then(GraphValue::as_color).ok_or_else(|| {
        GenerateError::InvalidParameter(format!(
            "node '{}' produced grayscale output but color was required",
            id
        ))
    })
}

pub(super) fn parse_hex_color_list(colors: &[String], name: &str) -> Result<Vec<Color>, GenerateError> {
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

pub(super) fn sample_color_ramp(ramp: &[Color], t: f64) -> Color {
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

pub(super) fn nearest_palette_color(palette: &[Color], color: Color) -> Color {
    debug_assert!(!palette.is_empty(), "palette must not be empty");

    let mut best = palette[0];
    let mut best_dist = color_distance_sq(best, color);
    for &candidate in palette.iter().skip(1) {
        let dist = color_distance_sq(candidate, color);
        if dist < best_dist {
            best = candidate;
            best_dist = dist;
        }
    }
    best
}

fn color_distance_sq(a: Color, b: Color) -> f64 {
    let dr = a.r - b.r;
    let dg = a.g - b.g;
    let db = a.b - b.b;
    dr * dr + dg * dg + db * db
}
