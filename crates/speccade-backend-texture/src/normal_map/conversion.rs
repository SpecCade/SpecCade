//! Height map to normal map conversion.

use crate::color::Color;
use crate::maps::{GrayscaleBuffer, TextureBuffer};

/// Convert height map to normal map using Sobel operators.
///
/// Uses the OpenGL/wgpu normal map convention:
/// - R (X): right is positive (bump slopes right -> brighter red)
/// - G (Y): up is positive (bump slopes up -> brighter green)
/// - B (Z): out/towards viewer is positive
///
/// A flat surface encodes as RGB (128, 128, 255) or normalized (0.5, 0.5, 1.0).
/// This matches the modern standard used by wgpu, Unity, Blender, and most game engines.
/// Note: DirectX uses the opposite Y convention (G = down), but we follow OpenGL/wgpu.
#[allow(clippy::needless_range_loop)]
pub(crate) fn height_to_normal(height_map: &GrayscaleBuffer, strength: f64) -> TextureBuffer {
    let width = height_map.width;
    let height = height_map.height;
    let mut buffer = TextureBuffer::new(width, height, Color::rgb(0.5, 0.5, 1.0));

    for y in 0..height {
        for x in 0..width {
            // Sample 3x3 neighborhood with wrapping
            let mut samples = [[0.0; 3]; 3];
            for dy in 0..3 {
                for dx in 0..3 {
                    let sx = x as i32 + dx as i32 - 1;
                    let sy = y as i32 + dy as i32 - 1;
                    samples[dy][dx] = height_map.get_wrapped(sx, sy);
                }
            }

            // Sobel operators for gradient
            // Gx = | -1  0  1 |    Gy = | -1 -2 -1 |
            //      | -2  0  2 |         |  0  0  0 |
            //      | -1  0  1 |         |  1  2  1 |

            let gx = (samples[0][2] + 2.0 * samples[1][2] + samples[2][2])
                - (samples[0][0] + 2.0 * samples[1][0] + samples[2][0]);

            let gy = (samples[2][0] + 2.0 * samples[2][1] + samples[2][2])
                - (samples[0][0] + 2.0 * samples[0][1] + samples[0][2]);

            // Scale by strength
            let gx = gx * strength;
            let gy = gy * strength;

            // Create normal vector in OpenGL/wgpu convention (Y-up)
            // gx > 0 means height increases to the right -> normal tilts left -> nx < 0
            // gy > 0 means height increases downward (image coords) -> in world coords (Y-up),
            //        this means height decreases upward -> normal tilts down -> ny < 0
            // But we want OpenGL convention where Y-up is positive, so we negate gy.
            // For X: standard convention is that positive gradient = negative normal X
            let nx = -gx;
            let ny = gy; // Inverted for OpenGL/wgpu Y-up convention (was -gy for DirectX Y-down)
            let nz = 1.0;

            // Normalize
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            let nx = nx / len;
            let ny = ny / len;
            let nz = nz / len;

            // Convert from [-1, 1] to [0, 1] for storage in RGB
            buffer.set(
                x,
                y,
                Color::rgb((nx + 1.0) * 0.5, (ny + 1.0) * 0.5, (nz + 1.0) * 0.5),
            );
        }
    }

    buffer
}
