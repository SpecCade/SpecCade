//! Normal map generator.

use super::{TextureBuffer, GrayscaleBuffer};
use crate::color::Color;

/// Normal map generator.
pub struct NormalGenerator {
    /// Strength multiplier for the normal map.
    pub strength: f64,
    /// Whether to invert the height map.
    pub invert: bool,
}

impl NormalGenerator {
    /// Create a new normal generator.
    pub fn new() -> Self {
        Self {
            strength: 1.0,
            invert: false,
        }
    }

    /// Set the strength.
    pub fn with_strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }

    /// Set whether to invert the height map.
    pub fn with_invert(mut self, invert: bool) -> Self {
        self.invert = invert;
        self
    }

    /// Generate a normal map from a height map using Sobel operators.
    pub fn generate_from_height(&self, height_map: &GrayscaleBuffer) -> TextureBuffer {
        let width = height_map.width;
        let height = height_map.height;
        let mut buffer = TextureBuffer::new(width, height, Color::rgb(0.5, 0.5, 1.0));

        for y in 0..height {
            for x in 0..width {
                let normal = self.calculate_normal(height_map, x as i32, y as i32);
                buffer.set(x, y, normal);
            }
        }

        buffer
    }

    /// Calculate normal at a specific pixel using Sobel operators.
    #[allow(clippy::needless_range_loop)]
    fn calculate_normal(&self, height_map: &GrayscaleBuffer, x: i32, y: i32) -> Color {
        // Sample 3x3 neighborhood with wrapping
        let mut samples = [[0.0; 3]; 3];
        for dy in 0..3 {
            for dx in 0..3 {
                let sx = x + dx as i32 - 1;
                let sy = y + dy as i32 - 1;
                let h = height_map.get_wrapped(sx, sy);
                samples[dy][dx] = if self.invert { 1.0 - h } else { h };
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
        let gx = gx * self.strength;
        let gy = gy * self.strength;

        // Create normal vector in OpenGL/wgpu convention (Y-up)
        // For tangent-space normal maps:
        // X = right (positive = bump facing right)
        // Y = up (positive = bump facing up) - OpenGL convention
        // Z = out (positive = facing camera)
        //
        // gx > 0 means height increases to the right -> normal tilts left -> nx < 0
        // gy > 0 means height increases downward (image coords) -> in OpenGL Y-up,
        //        this means normal tilts down -> ny < 0
        // But since image Y increases downward and OpenGL Y increases upward,
        // we need to negate gy to get correct Y-up normals.
        // A flat surface encodes as RGB (128, 128, 255) or normalized (0.5, 0.5, 1.0).
        let nx = -gx;
        let ny = gy;  // OpenGL/wgpu Y-up convention (was -gy for DirectX Y-down)
        let nz = 1.0;

        // Normalize
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        let nx = nx / len;
        let ny = ny / len;
        let nz = nz / len;

        // Convert from [-1, 1] to [0, 1] for storage in RGB
        Color::rgb(
            (nx + 1.0) * 0.5,
            (ny + 1.0) * 0.5,
            (nz + 1.0) * 0.5,
        )
    }

    /// Generate a flat normal map (pointing straight up).
    pub fn generate_flat(&self, width: u32, height: u32) -> TextureBuffer {
        // Flat normal: (0, 0, 1) -> RGB: (0.5, 0.5, 1.0)
        TextureBuffer::new(width, height, Color::rgb(0.5, 0.5, 1.0))
    }

    /// Blend two normal maps together.
    pub fn blend_normals(
        &self,
        base: &TextureBuffer,
        detail: &TextureBuffer,
        blend_factor: f64,
    ) -> TextureBuffer {
        let width = base.width.min(detail.width);
        let height = base.height.min(detail.height);
        let mut result = TextureBuffer::new(width, height, Color::rgb(0.5, 0.5, 1.0));

        for y in 0..height {
            for x in 0..width {
                let base_color = base.get(x, y);
                let detail_color = detail.get(x, y);

                // Convert from [0, 1] to [-1, 1]
                let base_n = [
                    base_color.r * 2.0 - 1.0,
                    base_color.g * 2.0 - 1.0,
                    base_color.b * 2.0 - 1.0,
                ];
                let detail_n = [
                    detail_color.r * 2.0 - 1.0,
                    detail_color.g * 2.0 - 1.0,
                    detail_color.b * 2.0 - 1.0,
                ];

                // Reoriented Normal Mapping (RNM) blend
                // This preserves both base and detail normal directions better
                let t = [base_n[0], base_n[1], base_n[2] + 1.0];
                let u = [-detail_n[0] * blend_factor, -detail_n[1] * blend_factor, detail_n[2]];

                let dot = t[0] * u[0] + t[1] * u[1] + t[2] * u[2];
                let result_n = [
                    t[0] * dot - u[0] * (t[2]),
                    t[1] * dot - u[1] * (t[2]),
                    t[2] * dot - u[2] * (t[2]),
                ];

                // Normalize
                let len = (result_n[0] * result_n[0] + result_n[1] * result_n[1] + result_n[2] * result_n[2]).sqrt();
                let nx = result_n[0] / len;
                let ny = result_n[1] / len;
                let nz = result_n[2] / len;

                // Convert back to [0, 1]
                result.set(x, y, Color::rgb(
                    (nx + 1.0) * 0.5,
                    (ny + 1.0) * 0.5,
                    (nz + 1.0) * 0.5,
                ));
            }
        }

        result
    }
}

impl Default for NormalGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_flat() {
        let generator = NormalGenerator::new();
        let buffer = generator.generate_flat(64, 64);

        for y in 0..64 {
            for x in 0..64 {
                let c = buffer.get(x, y);
                // Flat normal should be (0.5, 0.5, 1.0)
                assert!((c.r - 0.5).abs() < 1e-10);
                assert!((c.g - 0.5).abs() < 1e-10);
                assert!((c.b - 1.0).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_normal_from_height() {
        // Create a simple gradient height map
        let mut height_map = GrayscaleBuffer::new(64, 64, 0.5);
        for y in 0..64 {
            for x in 0..64 {
                // Gradient from left to right
                height_map.set(x, y, x as f64 / 63.0);
            }
        }

        let generator = NormalGenerator::new().with_strength(1.0);
        let normal_map = generator.generate_from_height(&height_map);

        // Check that normals are tilted based on gradient
        // With gradient going up to the right, normal X should be < 0.5
        let center = normal_map.get(32, 32);
        assert!(center.r < 0.5, "Normal X should indicate slope");
    }

    #[test]
    fn test_normal_deterministic() {
        let height_map = GrayscaleBuffer::new(64, 64, 0.5);

        let gen1 = NormalGenerator::new();
        let gen2 = NormalGenerator::new();

        let buf1 = gen1.generate_from_height(&height_map);
        let buf2 = gen2.generate_from_height(&height_map);

        for y in 0..64 {
            for x in 0..64 {
                let c1 = buf1.get(x, y);
                let c2 = buf2.get(x, y);
                assert_eq!(c1.r, c2.r);
                assert_eq!(c1.g, c2.g);
                assert_eq!(c1.b, c2.b);
            }
        }
    }

    #[test]
    fn test_normal_y_up_convention() {
        // Create a height map with a bump in the center
        // The top of the bump (smaller Y values in image space) should have green > 0.5
        // because OpenGL convention means Y-up = green = positive
        let mut height_map = GrayscaleBuffer::new(64, 64, 0.0);

        // Create a horizontal stripe of height in the middle
        for y in 28..36 {
            for x in 0..64 {
                height_map.set(x, y, 1.0);
            }
        }

        let generator = NormalGenerator::new().with_strength(1.0);
        let normal_map = generator.generate_from_height(&height_map);

        // At the top edge of the bump (y=28), the surface slopes UP (toward smaller Y)
        // In OpenGL Y-up convention, this should give green > 0.5
        let top_edge = normal_map.get(32, 27);
        assert!(
            top_edge.g > 0.5,
            "Top edge of bump should have green > 0.5 (Y-up convention). Got: {}",
            top_edge.g
        );

        // At the bottom edge of the bump (y=36), the surface slopes DOWN
        // In OpenGL Y-up convention, this should give green < 0.5
        let bottom_edge = normal_map.get(32, 36);
        assert!(
            bottom_edge.g < 0.5,
            "Bottom edge of bump should have green < 0.5 (Y-up convention). Got: {}",
            bottom_edge.g
        );
    }
}
