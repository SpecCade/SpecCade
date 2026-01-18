//! Image processing filter functions for procedural texture operations.

use crate::maps::GrayscaleBuffer;

/// Apply Gaussian blur approximated via box blur (3 passes for better quality).
pub(super) fn apply_blur(input: &GrayscaleBuffer, radius: f32) -> GrayscaleBuffer {
    let r = radius.max(0.0) as usize;
    if r == 0 {
        return input.clone();
    }

    let w = input.width as usize;
    let h = input.height as usize;

    // Box blur: horizontal pass
    fn box_blur_h(src: &[f64], dst: &mut [f64], w: usize, h: usize, r: usize) {
        let d = (2 * r + 1) as f64;
        for y in 0..h {
            let row_start = y * w;
            let mut sum = 0.0;
            // Initialize the sum for the first pixel
            for dx in 0..=r {
                sum += src[row_start + dx];
            }
            // Left edge: mirror
            for dx in 1..=r {
                sum += src[row_start + dx.min(w - 1)];
            }
            dst[row_start] = sum / d;

            for x in 1..w {
                let left_idx = if x > r { x - r - 1 } else { r - x };
                let right_idx = (x + r).min(w - 1);
                sum = sum - src[row_start + left_idx] + src[row_start + right_idx];
                dst[row_start + x] = sum / d;
            }
        }
    }

    // Box blur: vertical pass
    fn box_blur_v(src: &[f64], dst: &mut [f64], w: usize, h: usize, r: usize) {
        let d = (2 * r + 1) as f64;
        for x in 0..w {
            let mut sum = 0.0;
            // Initialize the sum for the first pixel
            for dy in 0..=r {
                sum += src[dy * w + x];
            }
            // Top edge: mirror
            for dy in 1..=r {
                sum += src[dy.min(h - 1) * w + x];
            }
            dst[x] = sum / d;

            for y in 1..h {
                let top_idx = if y > r { y - r - 1 } else { r - y };
                let bottom_idx = (y + r).min(h - 1);
                sum = sum - src[top_idx * w + x] + src[bottom_idx * w + x];
                dst[y * w + x] = sum / d;
            }
        }
    }

    let mut buf1 = input.data.clone();
    let mut buf2 = vec![0.0; buf1.len()];

    // 3 passes for approximate Gaussian
    for _ in 0..3 {
        box_blur_h(&buf1, &mut buf2, w, h, r);
        box_blur_v(&buf2, &mut buf1, w, h, r);
    }

    GrayscaleBuffer {
        width: input.width,
        height: input.height,
        data: buf1,
    }
}

/// Morphology operation type.
pub(super) enum MorphOp {
    Erode,
    Dilate,
}

/// Apply morphological operation (erode = min, dilate = max within radius).
pub(super) fn apply_morphology(
    input: &GrayscaleBuffer,
    radius: u32,
    op: MorphOp,
) -> GrayscaleBuffer {
    let r = radius as i32;
    let w = input.width;
    let h = input.height;
    let mut out = GrayscaleBuffer::new(w, h, 0.0);

    for y in 0..h {
        for x in 0..w {
            let mut extremum = match op {
                MorphOp::Erode => f64::MAX,
                MorphOp::Dilate => f64::MIN,
            };

            for dy in -r..=r {
                for dx in -r..=r {
                    let val = input.get_wrapped(x as i32 + dx, y as i32 + dy);
                    extremum = match op {
                        MorphOp::Erode => extremum.min(val),
                        MorphOp::Dilate => extremum.max(val),
                    };
                }
            }

            out.set(x, y, extremum);
        }
    }

    out
}

/// Apply domain warp using a displacement map.
/// The displacement grayscale (0-1) is centered at 0.5, meaning:
/// - 0.0 = maximum negative offset
/// - 0.5 = no offset
/// - 1.0 = maximum positive offset
pub(super) fn apply_warp(
    input: &GrayscaleBuffer,
    displacement: &GrayscaleBuffer,
    strength: f32,
) -> GrayscaleBuffer {
    let w = input.width;
    let h = input.height;
    let strength = strength as f64;
    let mut out = GrayscaleBuffer::new(w, h, 0.0);

    for y in 0..h {
        for x in 0..w {
            // Get displacement value and center it around 0
            let disp = displacement.get(x, y);
            let offset = (disp - 0.5) * 2.0 * strength;

            // Apply offset to both x and y for a radial warp effect
            let src_x = x as i32 + offset as i32;
            let src_y = y as i32 + offset as i32;

            out.set(x, y, input.get_wrapped(src_x, src_y));
        }
    }

    out
}

/// Blend mode type.
pub(super) enum BlendMode {
    Screen,
    Overlay,
    SoftLight,
    Difference,
}

/// Apply blend mode between two grayscale buffers.
pub(super) fn apply_blend(
    base: &GrayscaleBuffer,
    blend: &GrayscaleBuffer,
    mode: BlendMode,
) -> GrayscaleBuffer {
    let mut out = GrayscaleBuffer::new(base.width, base.height, 0.0);

    for i in 0..out.data.len() {
        let b = base.data[i];
        let l = blend.data[i];

        out.data[i] = match mode {
            // Screen: 1 - (1 - base) * (1 - blend)
            BlendMode::Screen => 1.0 - (1.0 - b) * (1.0 - l),
            // Overlay: if base < 0.5: 2 * base * blend, else 1 - 2 * (1 - base) * (1 - blend)
            BlendMode::Overlay => {
                if b < 0.5 {
                    2.0 * b * l
                } else {
                    1.0 - 2.0 * (1.0 - b) * (1.0 - l)
                }
            }
            // Soft Light (Pegtop formula): (1 - 2*blend) * base^2 + 2 * blend * base
            BlendMode::SoftLight => (1.0 - 2.0 * l) * b * b + 2.0 * l * b,
            // Difference: |base - blend|
            BlendMode::Difference => (b - l).abs(),
        };
    }

    out
}

/// Apply UV scale transform (sample input at scaled coordinates).
pub(super) fn apply_uv_scale(
    input: &GrayscaleBuffer,
    scale_x: f64,
    scale_y: f64,
) -> GrayscaleBuffer {
    let w = input.width;
    let h = input.height;
    let mut out = GrayscaleBuffer::new(w, h, 0.0);

    // Center of texture
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;

    for y in 0..h {
        for x in 0..w {
            // Transform relative to center
            let px = (x as f64 - cx) * scale_x + cx;
            let py = (y as f64 - cy) * scale_y + cy;

            out.set(x, y, input.get_wrapped(px as i32, py as i32));
        }
    }

    out
}

/// Apply UV rotation transform (sample input at rotated coordinates).
pub(super) fn apply_uv_rotate(input: &GrayscaleBuffer, angle: f64) -> GrayscaleBuffer {
    let w = input.width;
    let h = input.height;
    let mut out = GrayscaleBuffer::new(w, h, 0.0);

    let cos_a = angle.cos();
    let sin_a = angle.sin();

    // Center of texture
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;

    for y in 0..h {
        for x in 0..w {
            // Transform relative to center
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;

            // Rotate
            let px = dx * cos_a - dy * sin_a + cx;
            let py = dx * sin_a + dy * cos_a + cy;

            out.set(x, y, input.get_wrapped(px as i32, py as i32));
        }
    }

    out
}

/// Apply UV translation transform (sample input at offset coordinates).
pub(super) fn apply_uv_translate(
    input: &GrayscaleBuffer,
    offset_x: f64,
    offset_y: f64,
) -> GrayscaleBuffer {
    let w = input.width;
    let h = input.height;
    let mut out = GrayscaleBuffer::new(w, h, 0.0);

    // Convert normalized offset to pixel offset
    let px_offset_x = (offset_x * w as f64) as i32;
    let px_offset_y = (offset_y * h as f64) as i32;

    for y in 0..h {
        for x in 0..w {
            let src_x = x as i32 + px_offset_x;
            let src_y = y as i32 + px_offset_y;
            out.set(x, y, input.get_wrapped(src_x, src_y));
        }
    }

    out
}
