//! Color comparison metrics.
//!
//! Provides DeltaE (CIE76) computation and RGB-Lab conversion.

use super::{round_f64, FLOAT_PRECISION};
use crate::analysis::texture::TextureMetrics;

/// Histogram difference metrics.
#[derive(Debug, Clone)]
pub struct HistogramDiff {
    /// Red/grayscale channel histogram difference
    pub red: f64,
    /// Green channel histogram difference (None for grayscale)
    pub green: Option<f64>,
    /// Blue channel histogram difference (None for grayscale)
    pub blue: Option<f64>,
    /// Alpha channel histogram difference (None if no alpha)
    pub alpha: Option<f64>,
}

/// Calculate DeltaE (CIE76) between two images.
///
/// Returns mean and max DeltaE values.
pub fn calculate_delta_e(
    pixels_a: &[u8],
    pixels_b: &[u8],
    width: u32,
    height: u32,
    channels: u8,
) -> (f64, f64) {
    if pixels_a.len() != pixels_b.len() {
        return (f64::MAX, f64::MAX);
    }

    let pixel_count = (width * height) as usize;
    let ch = channels as usize;

    let mut sum_delta_e = 0.0;
    let mut max_delta_e = 0.0;

    for i in 0..pixel_count {
        let offset = i * ch;

        // Get RGB values (or grayscale)
        let (r_a, g_a, b_a) = extract_rgb(pixels_a, offset, ch);
        let (r_b, g_b, b_b) = extract_rgb(pixels_b, offset, ch);

        // Convert to Lab
        let lab_a = rgb_to_lab(r_a, g_a, b_a);
        let lab_b = rgb_to_lab(r_b, g_b, b_b);

        // Euclidean distance in Lab space
        let delta_e = ((lab_a.0 - lab_b.0).powi(2)
            + (lab_a.1 - lab_b.1).powi(2)
            + (lab_a.2 - lab_b.2).powi(2))
        .sqrt();

        sum_delta_e += delta_e;
        if delta_e > max_delta_e {
            max_delta_e = delta_e;
        }
    }

    let mean_delta_e = if pixel_count > 0 {
        sum_delta_e / pixel_count as f64
    } else {
        0.0
    };

    (
        round_f64(mean_delta_e, FLOAT_PRECISION),
        round_f64(max_delta_e, FLOAT_PRECISION),
    )
}

/// Extract RGB from pixel data at given offset.
fn extract_rgb(pixels: &[u8], offset: usize, channels: usize) -> (u8, u8, u8) {
    match channels {
        1 => {
            let v = pixels[offset];
            (v, v, v)
        }
        2 => {
            let v = pixels[offset];
            (v, v, v)
        }
        3 | 4 => (pixels[offset], pixels[offset + 1], pixels[offset + 2]),
        _ => (0, 0, 0),
    }
}

/// Convert sRGB to CIE Lab color space.
pub(super) fn rgb_to_lab(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    // Step 1: sRGB to linear RGB
    let r_lin = srgb_to_linear(r as f64 / 255.0);
    let g_lin = srgb_to_linear(g as f64 / 255.0);
    let b_lin = srgb_to_linear(b as f64 / 255.0);

    // Step 2: Linear RGB to XYZ (D65 illuminant)
    let x = r_lin * 0.4124564 + g_lin * 0.3575761 + b_lin * 0.1804375;
    let y = r_lin * 0.2126729 + g_lin * 0.7151522 + b_lin * 0.0721750;
    let z = r_lin * 0.0193339 + g_lin * 0.1191920 + b_lin * 0.9503041;

    // Step 3: XYZ to Lab (D65 reference white)
    const REF_X: f64 = 0.95047;
    const REF_Y: f64 = 1.00000;
    const REF_Z: f64 = 1.08883;

    let fx = lab_f(x / REF_X);
    let fy = lab_f(y / REF_Y);
    let fz = lab_f(z / REF_Z);

    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b_val = 200.0 * (fy - fz);

    (l, a, b_val)
}

/// sRGB to linear conversion.
fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Lab f function.
fn lab_f(t: f64) -> f64 {
    const DELTA: f64 = 6.0 / 29.0;
    const DELTA_CUBED: f64 = DELTA * DELTA * DELTA;

    if t > DELTA_CUBED {
        t.cbrt()
    } else {
        t / (3.0 * DELTA * DELTA) + 4.0 / 29.0
    }
}

/// Calculate histogram difference between two TextureMetrics.
pub fn calculate_histogram_diff(a: &TextureMetrics, b: &TextureMetrics) -> HistogramDiff {
    let red_diff = (a.histogram.red.mean - b.histogram.red.mean).abs();

    let green_diff = match (&a.histogram.green, &b.histogram.green) {
        (Some(ga), Some(gb)) => Some(round_f64((ga.mean - gb.mean).abs(), FLOAT_PRECISION)),
        _ => None,
    };

    let blue_diff = match (&a.histogram.blue, &b.histogram.blue) {
        (Some(ba), Some(bb)) => Some(round_f64((ba.mean - bb.mean).abs(), FLOAT_PRECISION)),
        _ => None,
    };

    let alpha_diff = match (&a.histogram.alpha, &b.histogram.alpha) {
        (Some(aa), Some(ab)) => Some(round_f64((aa.mean - ab.mean).abs(), FLOAT_PRECISION)),
        _ => None,
    };

    HistogramDiff {
        red: round_f64(red_diff, FLOAT_PRECISION),
        green: green_diff,
        blue: blue_diff,
        alpha: alpha_diff,
    }
}
