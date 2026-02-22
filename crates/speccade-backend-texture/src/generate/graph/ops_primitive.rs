//! Primitive grayscale node operations (constant, noise, gradient, stripes, checkerboard).

use speccade_spec::recipe::texture::{GradientDirection, NoiseConfig, StripeDirection};

use super::super::GenerateError;
use super::GraphValue;
use crate::generate::helpers::create_noise_generator;
use crate::maps::GrayscaleBuffer;
use crate::noise::lerp as lerp_f64;
use crate::pattern::{CheckerPattern, GradientPattern, Pattern2D, StripesPattern};
use crate::rng::DeterministicRng;

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

/// Generate reaction-diffusion pattern using a deterministic Gray-Scott simulation.
#[allow(clippy::too_many_arguments)]
pub(super) fn eval_reaction_diffusion(
    width: u32,
    height: u32,
    tileable: bool,
    steps: u32,
    feed: f64,
    kill: f64,
    diffuse_a: f64,
    diffuse_b: f64,
    dt: f64,
    seed_density: f64,
    derived_seed: u32,
) -> Result<GraphValue, GenerateError> {
    if steps == 0 || steps > 2000 {
        return Err(GenerateError::InvalidParameter(format!(
            "reaction_diffusion.steps must be in [1, 2000], got {}",
            steps
        )));
    }
    if !feed.is_finite() || !(0.0..=1.0).contains(&feed) || feed == 0.0 {
        return Err(GenerateError::InvalidParameter(format!(
            "reaction_diffusion.feed must be in (0.0, 1.0], got {}",
            feed
        )));
    }
    if !kill.is_finite() || !(0.0..=1.0).contains(&kill) || kill == 0.0 {
        return Err(GenerateError::InvalidParameter(format!(
            "reaction_diffusion.kill must be in (0.0, 1.0], got {}",
            kill
        )));
    }
    if !diffuse_a.is_finite() || diffuse_a <= 0.0 || diffuse_a > 2.0 {
        return Err(GenerateError::InvalidParameter(format!(
            "reaction_diffusion.diffuse_a must be in (0.0, 2.0], got {}",
            diffuse_a
        )));
    }
    if !diffuse_b.is_finite() || diffuse_b <= 0.0 || diffuse_b > 2.0 {
        return Err(GenerateError::InvalidParameter(format!(
            "reaction_diffusion.diffuse_b must be in (0.0, 2.0], got {}",
            diffuse_b
        )));
    }
    if !dt.is_finite() || dt <= 0.0 || dt > 2.0 {
        return Err(GenerateError::InvalidParameter(format!(
            "reaction_diffusion.dt must be in (0.0, 2.0], got {}",
            dt
        )));
    }
    if !seed_density.is_finite() || !(0.0..=0.5).contains(&seed_density) {
        return Err(GenerateError::InvalidParameter(format!(
            "reaction_diffusion.seed_density must be in [0.0, 0.5], got {}",
            seed_density
        )));
    }

    #[inline]
    fn idx(width: u32, height: u32, tileable: bool, x: i32, y: i32) -> usize {
        let (xi, yi) = if tileable {
            (
                x.rem_euclid(width as i32) as u32,
                y.rem_euclid(height as i32) as u32,
            )
        } else {
            (
                x.clamp(0, width as i32 - 1) as u32,
                y.clamp(0, height as i32 - 1) as u32,
            )
        };
        (yi * width + xi) as usize
    }

    let len = (width as usize) * (height as usize);
    let mut a = vec![1.0; len];
    let mut b = vec![0.0; len];
    let mut a_next = vec![1.0; len];
    let mut b_next = vec![0.0; len];

    // Seed B chemical probabilistically per cell for a non-uniform initial condition.
    let mut rng = DeterministicRng::new(derived_seed);
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            if rng.gen_f64() < seed_density {
                let i = idx(width, height, tileable, x, y);
                a[i] = 0.0;
                b[i] = 1.0;
            }
        }
    }

    for _ in 0..steps {
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let i = idx(width, height, tileable, x, y);

                let a_c = a[i];
                let b_c = b[i];

                let lap_a = (a[idx(width, height, tileable, x - 1, y)]
                    + a[idx(width, height, tileable, x + 1, y)]
                    + a[idx(width, height, tileable, x, y - 1)]
                    + a[idx(width, height, tileable, x, y + 1)])
                    * 0.2
                    + (a[idx(width, height, tileable, x - 1, y - 1)]
                        + a[idx(width, height, tileable, x + 1, y - 1)]
                        + a[idx(width, height, tileable, x - 1, y + 1)]
                        + a[idx(width, height, tileable, x + 1, y + 1)])
                        * 0.05
                    - a_c;

                let lap_b = (b[idx(width, height, tileable, x - 1, y)]
                    + b[idx(width, height, tileable, x + 1, y)]
                    + b[idx(width, height, tileable, x, y - 1)]
                    + b[idx(width, height, tileable, x, y + 1)])
                    * 0.2
                    + (b[idx(width, height, tileable, x - 1, y - 1)]
                        + b[idx(width, height, tileable, x + 1, y - 1)]
                        + b[idx(width, height, tileable, x - 1, y + 1)]
                        + b[idx(width, height, tileable, x + 1, y + 1)])
                        * 0.05
                    - b_c;

                let reaction = a_c * b_c * b_c;
                let da = diffuse_a * lap_a - reaction + feed * (1.0 - a_c);
                let db = diffuse_b * lap_b + reaction - (kill + feed) * b_c;

                a_next[i] = (a_c + da * dt).clamp(0.0, 1.0);
                b_next[i] = (b_c + db * dt).clamp(0.0, 1.0);
            }
        }
        std::mem::swap(&mut a, &mut a_next);
        std::mem::swap(&mut b, &mut b_next);
    }

    let (mut min_b, mut max_b) = (f64::INFINITY, f64::NEG_INFINITY);
    for &v in &b {
        min_b = min_b.min(v);
        max_b = max_b.max(v);
    }
    let range = max_b - min_b;

    let mut out = GrayscaleBuffer::new(width, height, 0.0);
    for y in 0..height {
        for x in 0..width {
            let i = (y as usize) * (width as usize) + (x as usize);
            let v = if range > 1e-12 {
                (b[i] - min_b) / range
            } else {
                0.0
            };
            out.set(x, y, v);
        }
    }

    Ok(GraphValue::Grayscale(out))
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
