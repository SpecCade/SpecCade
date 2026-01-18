//! Node operation evaluation.

use std::collections::{HashMap, HashSet};

use speccade_spec::recipe::texture::TextureProceduralOp;

use crate::rng::DeterministicRng;

use super::super::GenerateError;
use super::helpers::{expect_color, expect_gray};
use super::ops_color::{
    eval_color_ramp, eval_compose_rgba, eval_normal_from_height, eval_palette, eval_to_grayscale,
};
use super::ops_filter::{
    eval_blend_difference, eval_blend_overlay, eval_blend_screen, eval_blend_soft_light, eval_blur,
    eval_dilate, eval_erode, eval_uv_rotate, eval_uv_scale, eval_uv_translate, eval_warp,
};
use super::ops_math::{
    eval_add, eval_clamp, eval_invert, eval_lerp, eval_multiply, eval_threshold,
};
use super::ops_primitive::{
    eval_checkerboard, eval_constant, eval_gradient, eval_noise, eval_stripes,
};
use super::GraphValue;

/// Helper macro to evaluate a dependency node.
macro_rules! eval_dep {
    ($id:expr, $nodes_by_id:expr, $cache:expr, $visiting:expr, $w:expr, $h:expr, $tile:expr, $seed:expr) => {
        eval_node($id, $nodes_by_id, $cache, $visiting, $w, $h, $tile, $seed)?
    };
}

/// Evaluate a node in the texture graph, recursively evaluating dependencies.
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
        TextureProceduralOp::Constant { value } => eval_constant(width, height, *value),

        TextureProceduralOp::Noise { noise } => {
            eval_noise(width, height, tileable, noise, derived_seed)
        }

        TextureProceduralOp::Gradient {
            direction,
            start,
            end,
            center,
            inner,
            outer,
        } => eval_gradient(
            width, height, direction, *start, *end, *center, *inner, *outer,
        ),

        TextureProceduralOp::Stripes {
            direction,
            stripe_width,
            color1,
            color2,
        } => eval_stripes(width, height, direction, *stripe_width, *color1, *color2),

        TextureProceduralOp::Checkerboard {
            tile_size,
            color1,
            color2,
        } => eval_checkerboard(width, height, *tile_size, *color1, *color2),

        // -----------------------------------------------------------------
        // Grayscale math ops
        // -----------------------------------------------------------------
        TextureProceduralOp::Invert { input } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            eval_invert(in_buf, width, height)
        }

        TextureProceduralOp::Clamp { input, min, max } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            eval_clamp(in_buf, width, height, *min, *max)
        }

        TextureProceduralOp::Add { a, b } => {
            eval_dep!(
                a,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            eval_dep!(
                b,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let a_buf = expect_gray(cache, a)?;
            let b_buf = expect_gray(cache, b)?;
            eval_add(a_buf, b_buf, width, height)
        }

        TextureProceduralOp::Multiply { a, b } => {
            eval_dep!(
                a,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            eval_dep!(
                b,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let a_buf = expect_gray(cache, a)?;
            let b_buf = expect_gray(cache, b)?;
            eval_multiply(a_buf, b_buf, width, height)
        }

        TextureProceduralOp::Lerp { a, b, t } => {
            eval_dep!(
                a,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            eval_dep!(
                b,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            eval_dep!(
                t,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let a_buf = expect_gray(cache, a)?;
            let b_buf = expect_gray(cache, b)?;
            let t_buf = expect_gray(cache, t)?;
            eval_lerp(a_buf, b_buf, t_buf, width, height)
        }

        TextureProceduralOp::Threshold { input, threshold } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            eval_threshold(in_buf, width, height, *threshold)
        }

        // -----------------------------------------------------------------
        // Filter ops
        // -----------------------------------------------------------------
        TextureProceduralOp::Blur { input, radius } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            eval_blur(in_buf, *radius)
        }

        TextureProceduralOp::Erode { input, radius } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            eval_erode(in_buf, *radius)
        }

        TextureProceduralOp::Dilate { input, radius } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            eval_dilate(in_buf, *radius)
        }

        TextureProceduralOp::Warp {
            input,
            displacement,
            strength,
        } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            eval_dep!(
                displacement,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            let disp_buf = expect_gray(cache, displacement)?;
            eval_warp(in_buf, disp_buf, *strength)
        }

        // -----------------------------------------------------------------
        // Blend modes
        // -----------------------------------------------------------------
        TextureProceduralOp::BlendScreen { base, blend } => {
            eval_dep!(
                base,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            eval_dep!(
                blend,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let base_buf = expect_gray(cache, base)?;
            let blend_buf = expect_gray(cache, blend)?;
            eval_blend_screen(base_buf, blend_buf)
        }

        TextureProceduralOp::BlendOverlay { base, blend } => {
            eval_dep!(
                base,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            eval_dep!(
                blend,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let base_buf = expect_gray(cache, base)?;
            let blend_buf = expect_gray(cache, blend)?;
            eval_blend_overlay(base_buf, blend_buf)
        }

        TextureProceduralOp::BlendSoftLight { base, blend } => {
            eval_dep!(
                base,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            eval_dep!(
                blend,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let base_buf = expect_gray(cache, base)?;
            let blend_buf = expect_gray(cache, blend)?;
            eval_blend_soft_light(base_buf, blend_buf)
        }

        TextureProceduralOp::BlendDifference { base, blend } => {
            eval_dep!(
                base,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            eval_dep!(
                blend,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let base_buf = expect_gray(cache, base)?;
            let blend_buf = expect_gray(cache, blend)?;
            eval_blend_difference(base_buf, blend_buf)
        }

        // -----------------------------------------------------------------
        // UV transforms
        // -----------------------------------------------------------------
        TextureProceduralOp::UvScale {
            input,
            scale_x,
            scale_y,
        } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            eval_uv_scale(in_buf, *scale_x, *scale_y)
        }

        TextureProceduralOp::UvRotate { input, angle } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            eval_uv_rotate(in_buf, *angle)
        }

        TextureProceduralOp::UvTranslate {
            input,
            offset_x,
            offset_y,
        } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            eval_uv_translate(in_buf, *offset_x, *offset_y)
        }

        // -----------------------------------------------------------------
        // Color ops
        // -----------------------------------------------------------------
        TextureProceduralOp::ToGrayscale { input } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_color(cache, input)?;
            eval_to_grayscale(in_buf, width, height)
        }

        TextureProceduralOp::ColorRamp { input, ramp } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            eval_color_ramp(in_buf, width, height, ramp)?
        }

        TextureProceduralOp::Palette { input, palette } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_color(cache, input)?;
            eval_palette(in_buf, palette)?
        }

        TextureProceduralOp::ComposeRgba { r, g, b, a } => {
            eval_dep!(
                r,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            eval_dep!(
                g,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            eval_dep!(
                b,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            if let Some(a) = a.as_deref() {
                eval_dep!(
                    a,
                    nodes_by_id,
                    cache,
                    visiting,
                    width,
                    height,
                    tileable,
                    seed
                );
            }

            let r_buf = expect_gray(cache, r)?;
            let g_buf = expect_gray(cache, g)?;
            let b_buf = expect_gray(cache, b)?;
            let a_buf = a.as_deref().map(|id| expect_gray(cache, id)).transpose()?;
            eval_compose_rgba(r_buf, g_buf, b_buf, a_buf, width, height)
        }

        TextureProceduralOp::NormalFromHeight { input, strength } => {
            eval_dep!(
                input,
                nodes_by_id,
                cache,
                visiting,
                width,
                height,
                tileable,
                seed
            );
            let in_buf = expect_gray(cache, input)?;
            eval_normal_from_height(in_buf, *strength)
        }
    };

    visiting.remove(node_id);
    cache.insert(node_id, value);
    Ok(())
}
