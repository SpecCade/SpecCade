//! Mathematical grayscale operations (invert, clamp, add, multiply, lerp, threshold).

use crate::maps::GrayscaleBuffer;

use super::GraphValue;

/// Invert a grayscale buffer (1 - value).
pub(super) fn eval_invert(input: &GrayscaleBuffer, width: u32, height: u32) -> GraphValue {
    let mut out = GrayscaleBuffer::new(width, height, 0.0);
    for i in 0..out.data.len() {
        out.data[i] = 1.0 - input.data[i];
    }
    GraphValue::Grayscale(out)
}

/// Clamp values to a range.
pub(super) fn eval_clamp(
    input: &GrayscaleBuffer,
    width: u32,
    height: u32,
    min: f64,
    max: f64,
) -> GraphValue {
    let mut out = GrayscaleBuffer::new(width, height, 0.0);
    for i in 0..out.data.len() {
        out.data[i] = input.data[i].clamp(min, max);
    }
    GraphValue::Grayscale(out)
}

/// Add two grayscale buffers.
pub(super) fn eval_add(
    a: &GrayscaleBuffer,
    b: &GrayscaleBuffer,
    width: u32,
    height: u32,
) -> GraphValue {
    let mut out = GrayscaleBuffer::new(width, height, 0.0);
    for i in 0..out.data.len() {
        out.data[i] = a.data[i] + b.data[i];
    }
    GraphValue::Grayscale(out)
}

/// Multiply two grayscale buffers.
pub(super) fn eval_multiply(
    a: &GrayscaleBuffer,
    b: &GrayscaleBuffer,
    width: u32,
    height: u32,
) -> GraphValue {
    let mut out = GrayscaleBuffer::new(width, height, 0.0);
    for i in 0..out.data.len() {
        out.data[i] = a.data[i] * b.data[i];
    }
    GraphValue::Grayscale(out)
}

/// Linear interpolation between two grayscale buffers.
pub(super) fn eval_lerp(
    a: &GrayscaleBuffer,
    b: &GrayscaleBuffer,
    t: &GrayscaleBuffer,
    width: u32,
    height: u32,
) -> GraphValue {
    let mut out = GrayscaleBuffer::new(width, height, 0.0);
    for i in 0..out.data.len() {
        let tt = t.data[i];
        out.data[i] = a.data[i] * (1.0 - tt) + b.data[i] * tt;
    }
    GraphValue::Grayscale(out)
}

/// Threshold a grayscale buffer to binary values.
pub(super) fn eval_threshold(
    input: &GrayscaleBuffer,
    width: u32,
    height: u32,
    threshold: f64,
) -> GraphValue {
    let mut out = GrayscaleBuffer::new(width, height, 0.0);
    for i in 0..out.data.len() {
        out.data[i] = if input.data[i] >= threshold { 1.0 } else { 0.0 };
    }
    GraphValue::Grayscale(out)
}
