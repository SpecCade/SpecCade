//! Primitive grayscale node operations (constant, noise, gradient, stripes, checkerboard).

use speccade_spec::recipe::texture::{GradientDirection, NoiseConfig, StripeDirection};

use super::GraphValue;
use crate::generate::helpers::create_noise_generator;
use crate::maps::GrayscaleBuffer;
use crate::noise::lerp as lerp_f64;
use crate::pattern::{CheckerPattern, GradientPattern, Pattern2D, StripesPattern};

/// Generate a constant grayscale buffer filled with a single value.
pub(super) fn eval_constant(width: u32, height: u32, value: f64) -> GraphValue {
    GraphValue::Grayscale(GrayscaleBuffer::new(width, height, value))
}

/// Generate a noise pattern.
pub(super) fn eval_noise(
    width: u32,
    height: u32,
    tileable: bool,
    noise_config: &NoiseConfig,
    derived_seed: u32,
) -> GraphValue {
    let noise_gen = create_noise_generator(noise_config, derived_seed);
    let scale = noise_config.scale;
    let mut buf = GrayscaleBuffer::new(width, height, 0.0);

    if tileable && width > 1 && height > 1 && scale != 0.0 {
        let denom_x = (width as f64 - 1.0).max(1.0);
        let denom_y = (height as f64 - 1.0).max(1.0);
        let period_x = denom_x * scale;
        let period_y = denom_y * scale;

        for y in 0..height {
            let v = y as f64 / denom_y;
            let ny = v * period_y;
            for x in 0..width {
                let u = x as f64 / denom_x;
                let nx = u * period_x;

                let n00 = noise_gen.sample_01(nx, ny);
                let n10 = noise_gen.sample_01(nx - period_x, ny);
                let n01 = noise_gen.sample_01(nx, ny - period_y);
                let n11 = noise_gen.sample_01(nx - period_x, ny - period_y);

                let n0 = lerp_f64(n00, n10, u);
                let n1 = lerp_f64(n01, n11, u);
                let value = lerp_f64(n0, n1, v);
                buf.set(x, y, value);
            }
        }
    } else {
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 * scale;
                let ny = y as f64 * scale;
                buf.set(x, y, noise_gen.sample_01(nx, ny));
            }
        }
    }

    GraphValue::Grayscale(buf)
}

/// Generate a gradient pattern.
#[allow(clippy::too_many_arguments)]
pub(super) fn eval_gradient(
    width: u32,
    height: u32,
    direction: &GradientDirection,
    start: Option<f64>,
    end: Option<f64>,
    center: Option<[f64; 2]>,
    inner: Option<f64>,
    outer: Option<f64>,
) -> GraphValue {
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

    let mut buf = GrayscaleBuffer::new(width, height, 0.0);
    for y in 0..height {
        for x in 0..width {
            buf.set(x, y, gradient.sample(x, y));
        }
    }
    GraphValue::Grayscale(buf)
}

/// Generate a stripes pattern.
pub(super) fn eval_stripes(
    width: u32,
    height: u32,
    direction: &StripeDirection,
    stripe_width: u32,
    color1: f64,
    color2: f64,
) -> GraphValue {
    let stripes = match direction {
        StripeDirection::Horizontal => StripesPattern::new_horizontal(stripe_width, color1, color2),
        StripeDirection::Vertical => StripesPattern::new_vertical(stripe_width, color1, color2),
    };

    let mut buf = GrayscaleBuffer::new(width, height, 0.0);
    for y in 0..height {
        for x in 0..width {
            buf.set(x, y, stripes.sample(x, y));
        }
    }
    GraphValue::Grayscale(buf)
}

/// Generate a checkerboard pattern.
pub(super) fn eval_checkerboard(
    width: u32,
    height: u32,
    tile_size: u32,
    color1: f64,
    color2: f64,
) -> GraphValue {
    let checker = CheckerPattern::new(tile_size).with_colors(color1, color2);
    let mut buf = GrayscaleBuffer::new(width, height, 0.0);
    for y in 0..height {
        for x in 0..width {
            buf.set(x, y, checker.sample(x, y));
        }
    }
    GraphValue::Grayscale(buf)
}
