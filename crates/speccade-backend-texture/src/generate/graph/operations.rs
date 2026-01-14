//! Node operation evaluation.

use std::collections::{HashMap, HashSet};

use speccade_spec::recipe::texture::TextureProceduralOp;

use crate::color::Color;
use crate::maps::{GrayscaleBuffer, NormalGenerator, TextureBuffer};
use crate::noise::lerp as lerp_f64;
use crate::pattern::{CheckerPattern, GradientPattern, Pattern2D, StripesPattern};
use crate::rng::DeterministicRng;

use super::super::helpers::create_noise_generator;
use super::super::GenerateError;
use super::GraphValue;
use super::helpers::{expect_color, expect_gray, parse_hex_color_list, nearest_palette_color, sample_color_ramp};

#[allow(clippy::too_many_arguments)]
pub(super) fn eval_node<'a>(
    node_id: &'a str,
    nodes_by_id: &HashMap<&'a str, &'a speccade_spec::recipe::texture::TextureProceduralNode>,
    cache: &mut HashMap<&'a str, GraphValue>,
    visiting: &mut HashSet<&'a str>,
    width: u32,
    height: u32,
    tileable: bool,
    seed: u32,
) -> Result<(), GenerateError> {
    if cache.contains_key(node_id) {
        return Ok(());
    }
    if !visiting.insert(node_id) {
        return Err(GenerateError::InvalidParameter(format!(
            "cycle detected while evaluating node '{}'",
            node_id
        )));
    }

    let node = nodes_by_id
        .get(node_id)
        .ok_or_else(|| GenerateError::InvalidParameter(format!("unknown node id '{}'", node_id)))?;

    let derived_seed =
        DeterministicRng::derive_variant_seed(seed, &format!("texture.procedural_v1/{}", node_id));

    let value = match &node.op {
        // -----------------------------------------------------------------
        // Grayscale primitives
        // -----------------------------------------------------------------
        TextureProceduralOp::Constant { value } => {
            GraphValue::Grayscale(GrayscaleBuffer::new(width, height, *value))
        }
        TextureProceduralOp::Noise { noise } => {
            let noise_gen = create_noise_generator(noise, derived_seed);
            let scale = noise.scale;
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
        TextureProceduralOp::Gradient {
            direction,
            start,
            end,
            center,
            inner,
            outer,
        } => {
            let gradient = match direction {
                speccade_spec::recipe::texture::GradientDirection::Horizontal => {
                    let s = start.unwrap_or(0.0);
                    let e = end.unwrap_or(1.0);
                    GradientPattern::new_horizontal(width, height, s, e)
                }
                speccade_spec::recipe::texture::GradientDirection::Vertical => {
                    let s = start.unwrap_or(0.0);
                    let e = end.unwrap_or(1.0);
                    GradientPattern::new_vertical(width, height, s, e)
                }
                speccade_spec::recipe::texture::GradientDirection::Radial => {
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
        TextureProceduralOp::Stripes {
            direction,
            stripe_width,
            color1,
            color2,
        } => {
            let stripes = match direction {
                speccade_spec::recipe::texture::StripeDirection::Horizontal => {
                    StripesPattern::new_horizontal(*stripe_width, *color1, *color2)
                }
                speccade_spec::recipe::texture::StripeDirection::Vertical => {
                    StripesPattern::new_vertical(*stripe_width, *color1, *color2)
                }
            };

            let mut buf = GrayscaleBuffer::new(width, height, 0.0);
            for y in 0..height {
                for x in 0..width {
                    buf.set(x, y, stripes.sample(x, y));
                }
            }
            GraphValue::Grayscale(buf)
        }
        TextureProceduralOp::Checkerboard {
            tile_size,
            color1,
            color2,
        } => {
            let checker = CheckerPattern::new(*tile_size).with_colors(*color1, *color2);
            let mut buf = GrayscaleBuffer::new(width, height, 0.0);
            for y in 0..height {
                for x in 0..width {
                    buf.set(x, y, checker.sample(x, y));
                }
            }
            GraphValue::Grayscale(buf)
        }

        // -----------------------------------------------------------------
        // Grayscale ops
        // -----------------------------------------------------------------
        TextureProceduralOp::Invert { input } => {
            eval_node(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            let out = {
                let in_buf = expect_gray(cache, input)?;
                let mut out = GrayscaleBuffer::new(width, height, 0.0);
                for i in 0..out.data.len() {
                    out.data[i] = 1.0 - in_buf.data[i];
                }
                out
            };
            GraphValue::Grayscale(out)
        }
        TextureProceduralOp::Clamp { input, min, max } => {
            eval_node(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            let out = {
                let in_buf = expect_gray(cache, input)?;
                let mut out = GrayscaleBuffer::new(width, height, 0.0);
                for i in 0..out.data.len() {
                    out.data[i] = in_buf.data[i].clamp(*min, *max);
                }
                out
            };
            GraphValue::Grayscale(out)
        }
        TextureProceduralOp::Add { a, b } => {
            eval_node(
                a,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            eval_node(
                b,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            let out = {
                let a_buf = expect_gray(cache, a)?;
                let b_buf = expect_gray(cache, b)?;
                let mut out = GrayscaleBuffer::new(width, height, 0.0);
                for i in 0..out.data.len() {
                    out.data[i] = a_buf.data[i] + b_buf.data[i];
                }
                out
            };
            GraphValue::Grayscale(out)
        }
        TextureProceduralOp::Multiply { a, b } => {
            eval_node(
                a,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            eval_node(
                b,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            let out = {
                let a_buf = expect_gray(cache, a)?;
                let b_buf = expect_gray(cache, b)?;
                let mut out = GrayscaleBuffer::new(width, height, 0.0);
                for i in 0..out.data.len() {
                    out.data[i] = a_buf.data[i] * b_buf.data[i];
                }
                out
            };
            GraphValue::Grayscale(out)
        }
        TextureProceduralOp::Lerp { a, b, t } => {
            eval_node(
                a,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            eval_node(
                b,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            eval_node(
                t,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            let out = {
                let a_buf = expect_gray(cache, a)?;
                let b_buf = expect_gray(cache, b)?;
                let t_buf = expect_gray(cache, t)?;
                let mut out = GrayscaleBuffer::new(width, height, 0.0);
                for i in 0..out.data.len() {
                    let tt = t_buf.data[i];
                    out.data[i] = a_buf.data[i] * (1.0 - tt) + b_buf.data[i] * tt;
                }
                out
            };
            GraphValue::Grayscale(out)
        }
        TextureProceduralOp::Threshold { input, threshold } => {
            eval_node(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            let out = {
                let in_buf = expect_gray(cache, input)?;
                let mut out = GrayscaleBuffer::new(width, height, 0.0);
                for i in 0..out.data.len() {
                    out.data[i] = if in_buf.data[i] >= *threshold {
                        1.0
                    } else {
                        0.0
                    };
                }
                out
            };
            GraphValue::Grayscale(out)
        }

        // -----------------------------------------------------------------
        // Color ops
        // -----------------------------------------------------------------
        TextureProceduralOp::ToGrayscale { input } => {
            eval_node(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            let out = {
                let in_buf = expect_color(cache, input)?;
                let mut out = GrayscaleBuffer::new(width, height, 0.0);
                for i in 0..out.data.len() {
                    out.data[i] = in_buf.data[i].luminance();
                }
                out
            };
            GraphValue::Grayscale(out)
        }
        TextureProceduralOp::ColorRamp { input, ramp } => {
            eval_node(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            let ramp = parse_hex_color_list(ramp, "ramp")?;
            let out = {
                let in_buf = expect_gray(cache, input)?;
                let mut out = TextureBuffer::new(width, height, Color::black());
                for i in 0..in_buf.data.len() {
                    let mapped = sample_color_ramp(&ramp, in_buf.data[i]);
                    out.data[i] = Color::rgba(mapped.r, mapped.g, mapped.b, 1.0);
                }
                out
            };
            GraphValue::Color(out)
        }
        TextureProceduralOp::Palette { input, palette } => {
            eval_node(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            let palette = parse_hex_color_list(palette, "palette")?;
            let out = {
                let in_buf = expect_color(cache, input)?;
                let mut out = in_buf.clone();
                for pixel in &mut out.data {
                    let a = pixel.a;
                    let mapped = nearest_palette_color(&palette, *pixel);
                    *pixel = Color::rgba(mapped.r, mapped.g, mapped.b, a);
                }
                out
            };
            GraphValue::Color(out)
        }
        TextureProceduralOp::ComposeRgba { r, g, b, a } => {
            eval_node(
                r,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            eval_node(
                g,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            eval_node(
                b,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            if let Some(a) = a.as_deref() {
                eval_node(
                    a,
                    nodes_by_id,
                    cache,
                    visiting,
                    width,
                    height,
                    tileable,
                    seed,
                )?;
            }

            let out = {
                let r_buf = expect_gray(cache, r)?;
                let g_buf = expect_gray(cache, g)?;
                let b_buf = expect_gray(cache, b)?;
                let a_buf = a.as_deref().map(|id| expect_gray(cache, id)).transpose()?;

                let mut out = TextureBuffer::new(width, height, Color::black());
                for i in 0..out.data.len() {
                    let alpha = a_buf.map_or(1.0, |buf| buf.data[i]);
                    out.data[i] = Color::rgba(r_buf.data[i], g_buf.data[i], b_buf.data[i], alpha);
                }
                out
            };
            GraphValue::Color(out)
        }
        TextureProceduralOp::NormalFromHeight { input, strength } => {
            eval_node(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed,
            )?;
            let out = {
                let in_buf = expect_gray(cache, input)?;
                NormalGenerator::new()
                    .with_strength(*strength)
                    .generate_from_height(in_buf)
            };
            GraphValue::Color(out)
        }
    };

    visiting.remove(node_id);
    cache.insert(node_id, value);
    Ok(())
}
