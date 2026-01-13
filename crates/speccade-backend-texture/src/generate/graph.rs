//! Map-agnostic procedural texture generator.

use std::collections::{HashMap, HashSet};

use speccade_spec::recipe::texture::{TextureProceduralOp, TextureProceduralV1Params};

use crate::color::Color;
use crate::maps::{GrayscaleBuffer, NormalGenerator, TextureBuffer};
use crate::noise::lerp as lerp_f64;
use crate::pattern::{CheckerPattern, GradientPattern, Pattern2D, StripesPattern};
use crate::png;
use crate::rng::DeterministicRng;

use super::helpers::{create_noise_generator, validate_resolution};
use super::GenerateError;

/// A graph node's evaluated value.
#[derive(Debug, Clone)]
pub enum GraphValue {
    Grayscale(GrayscaleBuffer),
    Color(TextureBuffer),
}

impl GraphValue {
    pub fn as_grayscale(&self) -> Option<&GrayscaleBuffer> {
        match self {
            GraphValue::Grayscale(v) => Some(v),
            GraphValue::Color(_) => None,
        }
    }

    pub fn as_color(&self) -> Option<&TextureBuffer> {
        match self {
            GraphValue::Color(v) => Some(v),
            GraphValue::Grayscale(_) => None,
        }
    }
}

/// Generate all nodes for a `texture.procedural_v1` recipe.
pub fn generate_graph(
    params: &TextureProceduralV1Params,
    seed: u32,
) -> Result<HashMap<String, GraphValue>, GenerateError> {
    let [width, height] = params.resolution;
    validate_resolution(width, height)?;

    if params.nodes.is_empty() {
        return Err(GenerateError::InvalidParameter(
            "texture.procedural_v1 requires at least 1 node".to_string(),
        ));
    }

    let mut nodes_by_id: HashMap<&str, &speccade_spec::recipe::texture::TextureProceduralNode> =
        HashMap::new();
    for node in &params.nodes {
        if nodes_by_id
            .insert(node.id.as_str(), node)
            .is_some()
        {
            return Err(GenerateError::InvalidParameter(format!(
                "duplicate node id: '{}'",
                node.id
            )));
        }
    }

    let mut cache: HashMap<&str, GraphValue> = HashMap::new();
    let mut visiting: HashSet<&str> = HashSet::new();

    // Evaluate everything (small graphs; keeps output binding simple).
    let node_ids: Vec<&str> = nodes_by_id.keys().copied().collect();
    for node_id in node_ids {
        eval_node(
            node_id,
            &nodes_by_id,
            &mut cache,
            &mut visiting,
            width,
            height,
            params.tileable,
            seed,
        )?;
    }

    Ok(cache
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect())
}

fn eval_node<'a>(
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

    let node = nodes_by_id.get(node_id).ok_or_else(|| {
        GenerateError::InvalidParameter(format!("unknown node id '{}'", node_id))
    })?;

    let derived_seed =
        DeterministicRng::derive_variant_seed(seed, &format!("texture.procedural_v1/{}", node_id));

    let value = match &node.op {
        // -----------------------------------------------------------------
        // Grayscale primitives
        // -----------------------------------------------------------------
        TextureProceduralOp::Constant { value } => GraphValue::Grayscale(GrayscaleBuffer::new(
            width, height, *value,
        )),
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
            eval_node(input, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
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
            eval_node(input, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
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
            eval_node(a, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
            eval_node(b, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
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
            eval_node(a, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
            eval_node(b, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
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
            eval_node(a, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
            eval_node(b, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
            eval_node(t, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
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
            eval_node(input, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
            let out = {
                let in_buf = expect_gray(cache, input)?;
                let mut out = GrayscaleBuffer::new(width, height, 0.0);
                for i in 0..out.data.len() {
                    out.data[i] = if in_buf.data[i] >= *threshold { 1.0 } else { 0.0 };
                }
                out
            };
            GraphValue::Grayscale(out)
        }

        // -----------------------------------------------------------------
        // Color ops
        // -----------------------------------------------------------------
        TextureProceduralOp::ToGrayscale { input } => {
            eval_node(input, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
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
            eval_node(input, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
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
            eval_node(input, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
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
            eval_node(r, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
            eval_node(g, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
            eval_node(b, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
            if let Some(a) = a.as_deref() {
                eval_node(a, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
            }

            let out = {
                let r_buf = expect_gray(cache, r)?;
                let g_buf = expect_gray(cache, g)?;
                let b_buf = expect_gray(cache, b)?;
                let a_buf = a
                    .as_deref()
                    .map(|id| expect_gray(cache, id))
                    .transpose()?;

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
            eval_node(input, nodes_by_id, cache, visiting, width, height, tileable, seed)?;
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

fn expect_gray<'a>(cache: &'a HashMap<&str, GraphValue>, id: &str) -> Result<&'a GrayscaleBuffer, GenerateError> {
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

fn expect_color<'a>(cache: &'a HashMap<&str, GraphValue>, id: &str) -> Result<&'a TextureBuffer, GenerateError> {
    cache
        .get(id)
        .and_then(GraphValue::as_color)
        .ok_or_else(|| {
            GenerateError::InvalidParameter(format!(
                "node '{}' produced grayscale output but color was required",
                id
            ))
        })
}

fn parse_hex_color_list(colors: &[String], name: &str) -> Result<Vec<Color>, GenerateError> {
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

fn sample_color_ramp(ramp: &[Color], t: f64) -> Color {
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

fn nearest_palette_color(palette: &[Color], color: Color) -> Color {
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

/// Encode a graph value as PNG bytes (deterministic) and return `(bytes, blake3_hash)`.
pub fn encode_graph_value_png(value: &GraphValue) -> Result<(Vec<u8>, String), GenerateError> {
    let config = crate::png::PngConfig::default();

    match value {
        GraphValue::Grayscale(buf) => {
            let (data, hash) = png::write_grayscale_to_vec_with_hash(buf, &config)?;
            Ok((data, hash))
        }
        GraphValue::Color(buf) => {
            let (data, hash) = png::write_rgba_to_vec_with_hash(buf, &config)?;
            Ok((data, hash))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::texture::{NoiseAlgorithm, NoiseConfig, TextureProceduralNode};

    fn make_params(tileable: bool, nodes: Vec<TextureProceduralNode>) -> TextureProceduralV1Params {
        TextureProceduralV1Params {
            resolution: [32, 32],
            tileable,
            nodes,
        }
    }

    #[test]
    fn graph_is_deterministic_for_same_seed() {
        let params = make_params(true, vec![
            TextureProceduralNode {
                id: "n".to_string(),
                op: TextureProceduralOp::Noise {
                    noise: NoiseConfig {
                        algorithm: NoiseAlgorithm::Perlin,
                        scale: 0.1,
                        octaves: 3,
                        persistence: 0.5,
                        lacunarity: 2.0,
                    },
                },
            },
            TextureProceduralNode {
                id: "mask".to_string(),
                op: TextureProceduralOp::Threshold {
                    input: "n".to_string(),
                    threshold: 0.5,
                },
            },
        ]);

        let a = generate_graph(&params, 42).unwrap();
        let b = generate_graph(&params, 42).unwrap();

        let mask_a = a.get("mask").unwrap();
        let mask_b = b.get("mask").unwrap();

        let (bytes_a, hash_a) = encode_graph_value_png(mask_a).unwrap();
        let (bytes_b, hash_b) = encode_graph_value_png(mask_b).unwrap();
        assert_eq!(hash_a, hash_b);
        assert_eq!(bytes_a, bytes_b);
    }

    #[test]
    fn unknown_node_reference_is_error() {
        let params = make_params(true, vec![TextureProceduralNode {
            id: "bad".to_string(),
            op: TextureProceduralOp::Invert {
                input: "missing".to_string(),
            },
        }]);

        let err = generate_graph(&params, 1).unwrap_err();
        assert!(err.to_string().contains("unknown node id") || err.to_string().contains("unknown node reference"));
    }

    #[test]
    fn cycle_is_error() {
        let params = make_params(
            true,
            vec![
                TextureProceduralNode {
                    id: "a".to_string(),
                    op: TextureProceduralOp::Invert {
                        input: "b".to_string(),
                    },
                },
                TextureProceduralNode {
                    id: "b".to_string(),
                    op: TextureProceduralOp::Invert {
                        input: "a".to_string(),
                    },
                },
            ],
        );

        let err = generate_graph(&params, 1).unwrap_err();
        assert!(err.to_string().contains("cycle detected"));
    }

    #[test]
    fn obvious_type_mismatch_is_error() {
        let params = make_params(
            true,
            vec![
                TextureProceduralNode {
                    id: "n".to_string(),
                    op: TextureProceduralOp::Noise {
                        noise: NoiseConfig {
                            algorithm: NoiseAlgorithm::Perlin,
                            scale: 0.1,
                            octaves: 2,
                            persistence: 0.5,
                            lacunarity: 2.0,
                        },
                    },
                },
                TextureProceduralNode {
                    id: "bad".to_string(),
                    op: TextureProceduralOp::Palette {
                        input: "n".to_string(),
                        palette: vec!["#000000".to_string(), "#ffffff".to_string()],
                    },
                },
            ],
        );

        let err = generate_graph(&params, 1).unwrap_err();
        assert!(err.to_string().contains("color output") || err.to_string().contains("color was required"));
    }

    #[test]
    fn tileable_noise_matches_edges() {
        let params = make_params(
            true,
            vec![TextureProceduralNode {
                id: "n".to_string(),
                op: TextureProceduralOp::Noise {
                    noise: NoiseConfig {
                        algorithm: NoiseAlgorithm::Perlin,
                        scale: 0.12,
                        octaves: 3,
                        persistence: 0.5,
                        lacunarity: 2.0,
                    },
                },
            }],
        );

        let nodes = generate_graph(&params, 42).unwrap();
        let n = nodes.get("n").unwrap().as_grayscale().unwrap();

        let w = n.width;
        let h = n.height;

        for y in 0..h {
            let left = n.get(0, y);
            let right = n.get(w - 1, y);
            assert!(
                (left - right).abs() < 1e-12,
                "left/right mismatch at y={}: {} vs {}",
                y,
                left,
                right
            );
        }

        for x in 0..w {
            let top = n.get(x, 0);
            let bottom = n.get(x, h - 1);
            assert!(
                (top - bottom).abs() < 1e-12,
                "top/bottom mismatch at x={}: {} vs {}",
                x,
                top,
                bottom
            );
        }
    }

    #[test]
    fn reorder_does_not_change_output() {
        let n = TextureProceduralNode {
            id: "n".to_string(),
            op: TextureProceduralOp::Noise {
                noise: NoiseConfig {
                    algorithm: NoiseAlgorithm::Perlin,
                    scale: 0.1,
                    octaves: 3,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
            },
        };
        let mask = TextureProceduralNode {
            id: "mask".to_string(),
            op: TextureProceduralOp::Threshold {
                input: "n".to_string(),
                threshold: 0.5,
            },
        };

        let params_a = make_params(true, vec![n.clone(), mask.clone()]);
        let params_b = make_params(true, vec![mask, n]);

        let a = generate_graph(&params_a, 123).unwrap();
        let b = generate_graph(&params_b, 123).unwrap();

        let (bytes_a, hash_a) = encode_graph_value_png(a.get("mask").unwrap()).unwrap();
        let (bytes_b, hash_b) = encode_graph_value_png(b.get("mask").unwrap()).unwrap();

        assert_eq!(hash_a, hash_b);
        assert_eq!(bytes_a, bytes_b);
    }
}
