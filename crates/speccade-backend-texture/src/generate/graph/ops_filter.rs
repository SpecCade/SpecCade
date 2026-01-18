//! Filter and transform operations (blur, erode, dilate, warp, blend modes, UV transforms).

use crate::maps::GrayscaleBuffer;

use super::filters::{
    apply_blend, apply_blur, apply_morphology, apply_uv_rotate, apply_uv_scale, apply_uv_translate,
    apply_warp, BlendMode, MorphOp,
};
use super::GraphValue;

/// Apply Gaussian blur.
pub(super) fn eval_blur(input: &GrayscaleBuffer, radius: f32) -> GraphValue {
    GraphValue::Grayscale(apply_blur(input, radius))
}

/// Apply erosion (shrink bright regions).
pub(super) fn eval_erode(input: &GrayscaleBuffer, radius: u32) -> GraphValue {
    GraphValue::Grayscale(apply_morphology(input, radius, MorphOp::Erode))
}

/// Apply dilation (expand bright regions).
pub(super) fn eval_dilate(input: &GrayscaleBuffer, radius: u32) -> GraphValue {
    GraphValue::Grayscale(apply_morphology(input, radius, MorphOp::Dilate))
}

/// Apply domain warp using a displacement map.
pub(super) fn eval_warp(
    input: &GrayscaleBuffer,
    displacement: &GrayscaleBuffer,
    strength: f32,
) -> GraphValue {
    GraphValue::Grayscale(apply_warp(input, displacement, strength))
}

/// Apply screen blend mode.
pub(super) fn eval_blend_screen(base: &GrayscaleBuffer, blend: &GrayscaleBuffer) -> GraphValue {
    GraphValue::Grayscale(apply_blend(base, blend, BlendMode::Screen))
}

/// Apply overlay blend mode.
pub(super) fn eval_blend_overlay(base: &GrayscaleBuffer, blend: &GrayscaleBuffer) -> GraphValue {
    GraphValue::Grayscale(apply_blend(base, blend, BlendMode::Overlay))
}

/// Apply soft light blend mode.
pub(super) fn eval_blend_soft_light(base: &GrayscaleBuffer, blend: &GrayscaleBuffer) -> GraphValue {
    GraphValue::Grayscale(apply_blend(base, blend, BlendMode::SoftLight))
}

/// Apply difference blend mode.
pub(super) fn eval_blend_difference(base: &GrayscaleBuffer, blend: &GrayscaleBuffer) -> GraphValue {
    GraphValue::Grayscale(apply_blend(base, blend, BlendMode::Difference))
}

/// Apply UV scale transform.
pub(super) fn eval_uv_scale(input: &GrayscaleBuffer, scale_x: f32, scale_y: f32) -> GraphValue {
    GraphValue::Grayscale(apply_uv_scale(input, scale_x as f64, scale_y as f64))
}

/// Apply UV rotation transform.
pub(super) fn eval_uv_rotate(input: &GrayscaleBuffer, angle: f32) -> GraphValue {
    GraphValue::Grayscale(apply_uv_rotate(input, angle as f64))
}

/// Apply UV translation transform.
pub(super) fn eval_uv_translate(
    input: &GrayscaleBuffer,
    offset_x: f32,
    offset_y: f32,
) -> GraphValue {
    GraphValue::Grayscale(apply_uv_translate(input, offset_x as f64, offset_y as f64))
}
