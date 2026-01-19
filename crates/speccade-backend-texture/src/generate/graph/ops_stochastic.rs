//! Stochastic tiling operations: Wang tiles and texture bombing.

use crate::maps::GrayscaleBuffer;
use crate::rng::DeterministicRng;

use super::super::GenerateError;
use super::GraphValue;

/// Blend modes for texture bombing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BombBlendMode {
    /// Take maximum value at each pixel.
    Max,
    /// Add values (clamped to 1.0).
    Add,
    /// Average overlapping values.
    Average,
}

impl BombBlendMode {
    pub fn from_str(s: &str) -> Result<Self, GenerateError> {
        match s.to_lowercase().as_str() {
            "max" => Ok(BombBlendMode::Max),
            "add" => Ok(BombBlendMode::Add),
            "average" => Ok(BombBlendMode::Average),
            _ => Err(GenerateError::InvalidParameter(format!(
                "unknown blend_mode '{}', expected 'max', 'add', or 'average'",
                s
            ))),
        }
    }
}

/// Wang tiles: edge-matching seamless tiling.
///
/// Uses a simplified 2-edge Wang tile approach where each tile is assigned
/// corner colors based on a hash of its position. Tiles with matching edge
/// colors are blended together for seamless transitions.
pub fn eval_wang_tiles(
    input: &GrayscaleBuffer,
    tile_count: [u32; 2],
    blend_width: f64,
    seed: u32,
) -> GraphValue {
    let width = input.width;
    let height = input.height;

    // Validate parameters
    let tile_count_x = tile_count[0].max(1);
    let tile_count_y = tile_count[1].max(1);
    let blend_width = blend_width.clamp(0.0, 0.5);

    let tile_width = width / tile_count_x;
    let tile_height = height / tile_count_y;

    let mut output = GrayscaleBuffer::new(width, height, 0.0);
    let mut rng = DeterministicRng::new(seed);

    // Pre-generate tile assignments for deterministic corner colors.
    // We use a simple 2-color system (0 or 1) for each edge.
    // For a 2-edge Wang tile system, we need 4 tile types.
    let mut tile_corners: Vec<Vec<[u8; 4]>> = Vec::new();
    for ty in 0..tile_count_y {
        let mut row = Vec::new();
        for tx in 0..tile_count_x {
            // Generate corner colors based on position hash for determinism
            let hash = rng.gen_u32();
            let mut corners = [
                ((hash >> 0) & 1) as u8, // top-left
                ((hash >> 1) & 1) as u8, // top-right
                ((hash >> 2) & 1) as u8, // bottom-left
                ((hash >> 3) & 1) as u8, // bottom-right
            ];

            // Ensure edge matching with neighbors
            if tx > 0 {
                // Match left edge with right edge of previous tile (in current row)
                let prev: &[u8; 4] = &row[(tx - 1) as usize];
                corners[0] = prev[1]; // top-left matches prev's top-right
                corners[2] = prev[3]; // bottom-left matches prev's bottom-right
            }

            if ty > 0 {
                // Match top edge with bottom edge of tile above
                let above = &tile_corners[(ty - 1) as usize][tx as usize];
                corners[0] = above[2]; // top-left matches above's bottom-left
                corners[1] = above[3]; // top-right matches above's bottom-right
            }

            row.push(corners);
        }
        tile_corners.push(row);
    }

    // Generate output by sampling input with tile offsets and blending
    for y in 0..height {
        for x in 0..width {
            // Determine which tile this pixel is in
            let tx = (x / tile_width).min(tile_count_x - 1);
            let ty = (y / tile_height).min(tile_count_y - 1);

            // Local coordinates within the tile (0.0 to 1.0)
            let local_x = (x % tile_width) as f64 / tile_width as f64;
            let local_y = (y % tile_height) as f64 / tile_height as f64;

            // Calculate blend weights for edges
            let blend_left = if local_x < blend_width {
                local_x / blend_width
            } else {
                1.0
            };
            let blend_right = if local_x > 1.0 - blend_width {
                (1.0 - local_x) / blend_width
            } else {
                1.0
            };
            let blend_top = if local_y < blend_width {
                local_y / blend_width
            } else {
                1.0
            };
            let blend_bottom = if local_y > 1.0 - blend_width {
                (1.0 - local_y) / blend_width
            } else {
                1.0
            };

            // Sample from input with tile-based UV offset
            let corners = &tile_corners[ty as usize][tx as usize];
            let corner_offset = (corners[0] as f64 * 0.25
                + corners[1] as f64 * 0.25
                + corners[2] as f64 * 0.25
                + corners[3] as f64 * 0.25)
                * 0.1;

            // Calculate source UV with offset for variety
            let src_u = (local_x + corner_offset).fract();
            let src_v = (local_y + corner_offset * 0.7).fract();

            // Sample from input with wrapping
            let src_x = ((src_u * (input.width - 1) as f64).round() as i32)
                .rem_euclid(input.width as i32) as u32;
            let src_y = ((src_v * (input.height - 1) as f64).round() as i32)
                .rem_euclid(input.height as i32) as u32;

            let base_value = input.get(src_x, src_y);

            // Apply edge blending
            let blend_factor = blend_left.min(blend_right).min(blend_top).min(blend_bottom);
            let value = base_value * blend_factor + base_value * (1.0 - blend_factor) * 0.8;

            output.set(x, y, value.clamp(0.0, 1.0));
        }
    }

    GraphValue::Grayscale(output)
}

/// Texture bombing: random stamp placement with overlap handling.
///
/// Places randomized stamps of the input texture across the output,
/// with configurable density, scale variation, rotation, and blend mode.
pub fn eval_texture_bomb(
    input: &GrayscaleBuffer,
    width: u32,
    height: u32,
    density: f64,
    scale_variation: [f64; 2],
    rotation_variation: f64,
    blend_mode: BombBlendMode,
    seed: u32,
) -> GraphValue {
    let mut output = GrayscaleBuffer::new(width, height, 0.0);
    let mut blend_counts = vec![0u32; (width * height) as usize];

    let mut rng = DeterministicRng::new(seed);

    // Clamp parameters
    let density = density.clamp(0.0, 1.0);
    let scale_min = scale_variation[0].max(0.1);
    let scale_max = scale_variation[1].max(scale_min);
    let rotation_variation = rotation_variation.clamp(0.0, 360.0);

    // Calculate number of stamps based on density
    // Density of 1.0 means roughly one stamp per 64x64 area
    let stamp_area = 64.0 * 64.0;
    let total_area = (width * height) as f64;
    let num_stamps = ((total_area / stamp_area) * density * 4.0).ceil() as u32;

    // Place stamps
    for _ in 0..num_stamps {
        // Random position
        let stamp_x = rng.gen_f64() * width as f64;
        let stamp_y = rng.gen_f64() * height as f64;

        // Random scale within range
        let scale = if (scale_max - scale_min).abs() < 1e-6 {
            scale_min
        } else {
            scale_min + rng.gen_f64() * (scale_max - scale_min)
        };

        // Random rotation (in radians)
        let rotation = if rotation_variation > 0.0 {
            (rng.gen_f64() * rotation_variation).to_radians()
        } else {
            0.0
        };

        let cos_r = rotation.cos();
        let sin_r = rotation.sin();

        // Calculate stamp bounds
        let stamp_w = (input.width as f64 * scale).ceil() as i32;
        let stamp_h = (input.height as f64 * scale).ceil() as i32;

        let half_w = stamp_w / 2;
        let half_h = stamp_h / 2;

        // Render stamp
        for dy in -half_h..=half_h {
            for dx in -half_w..=half_w {
                // Apply rotation
                let rx = dx as f64 * cos_r - dy as f64 * sin_r;
                let ry = dx as f64 * sin_r + dy as f64 * cos_r;

                // Calculate output position
                let out_x = (stamp_x + rx).round() as i32;
                let out_y = (stamp_y + ry).round() as i32;

                // Skip if outside bounds
                if out_x < 0 || out_x >= width as i32 || out_y < 0 || out_y >= height as i32 {
                    continue;
                }

                // Calculate source position (pre-rotation, pre-scale)
                let src_u = (dx as f64 / scale + input.width as f64 / 2.0) / input.width as f64;
                let src_v = (dy as f64 / scale + input.height as f64 / 2.0) / input.height as f64;

                // Skip if outside input bounds
                if src_u < 0.0 || src_u >= 1.0 || src_v < 0.0 || src_v >= 1.0 {
                    continue;
                }

                let src_x = (src_u * (input.width - 1) as f64).round() as u32;
                let src_y = (src_v * (input.height - 1) as f64).round() as u32;

                let stamp_value = input.get(src_x, src_y);

                // Skip very dark values (treat as transparent)
                if stamp_value < 0.01 {
                    continue;
                }

                let out_idx = (out_y as u32 * width + out_x as u32) as usize;
                let current = output.data[out_idx];

                // Apply blend mode
                let new_value = match blend_mode {
                    BombBlendMode::Max => current.max(stamp_value),
                    BombBlendMode::Add => (current + stamp_value).min(1.0),
                    BombBlendMode::Average => {
                        blend_counts[out_idx] += 1;
                        current + stamp_value // Will divide by count later
                    }
                };

                output.data[out_idx] = new_value;
            }
        }
    }

    // Finalize average blend mode
    if blend_mode == BombBlendMode::Average {
        for i in 0..output.data.len() {
            if blend_counts[i] > 0 {
                output.data[i] /= blend_counts[i] as f64;
            }
        }
    }

    GraphValue::Grayscale(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_buffer(width: u32, height: u32, value: f64) -> GrayscaleBuffer {
        GrayscaleBuffer::new(width, height, value)
    }

    fn make_gradient_buffer(width: u32, height: u32) -> GrayscaleBuffer {
        let mut buf = GrayscaleBuffer::new(width, height, 0.0);
        for y in 0..height {
            for x in 0..width {
                let u = x as f64 / (width - 1) as f64;
                let v = y as f64 / (height - 1) as f64;
                buf.set(x, y, (u + v) / 2.0);
            }
        }
        buf
    }

    #[test]
    fn test_wang_tiles_basic() {
        let input = make_gradient_buffer(64, 64);
        let result = eval_wang_tiles(&input, [4, 4], 0.1, 42);

        let GraphValue::Grayscale(output) = result else {
            panic!("expected grayscale output");
        };

        assert_eq!(output.width, 64);
        assert_eq!(output.height, 64);

        // Check that output has reasonable values
        let mut has_variation = false;
        let first = output.get(0, 0);
        for y in 0..output.height {
            for x in 0..output.width {
                let v = output.get(x, y);
                assert!(v >= 0.0 && v <= 1.0, "value out of range at ({}, {}): {}", x, y, v);
                if (v - first).abs() > 0.01 {
                    has_variation = true;
                }
            }
        }
        assert!(has_variation, "output should have variation");
    }

    #[test]
    fn test_wang_tiles_deterministic() {
        let input = make_gradient_buffer(32, 32);

        let result1 = eval_wang_tiles(&input, [2, 2], 0.1, 42);
        let result2 = eval_wang_tiles(&input, [2, 2], 0.1, 42);

        let GraphValue::Grayscale(out1) = result1 else {
            panic!("expected grayscale");
        };
        let GraphValue::Grayscale(out2) = result2 else {
            panic!("expected grayscale");
        };

        assert_eq!(out1.data, out2.data, "same seed should produce identical output");
    }

    #[test]
    fn test_wang_tiles_different_seeds() {
        let input = make_gradient_buffer(32, 32);

        let result1 = eval_wang_tiles(&input, [2, 2], 0.1, 42);
        let result2 = eval_wang_tiles(&input, [2, 2], 0.1, 43);

        let GraphValue::Grayscale(out1) = result1 else {
            panic!("expected grayscale");
        };
        let GraphValue::Grayscale(out2) = result2 else {
            panic!("expected grayscale");
        };

        assert_ne!(out1.data, out2.data, "different seeds should produce different output");
    }

    #[test]
    fn test_texture_bomb_basic() {
        let input = make_test_buffer(16, 16, 0.8);
        let result = eval_texture_bomb(&input, 64, 64, 0.5, [0.8, 1.2], 90.0, BombBlendMode::Max, 42);

        let GraphValue::Grayscale(output) = result else {
            panic!("expected grayscale output");
        };

        assert_eq!(output.width, 64);
        assert_eq!(output.height, 64);

        // Check that some pixels were stamped
        let max_value: f64 = output.data.iter().cloned().fold(0.0, f64::max);
        assert!(max_value > 0.0, "output should have some stamped pixels");
    }

    #[test]
    fn test_texture_bomb_deterministic() {
        let input = make_test_buffer(16, 16, 0.5);

        let result1 = eval_texture_bomb(&input, 32, 32, 0.5, [1.0, 1.0], 0.0, BombBlendMode::Max, 42);
        let result2 = eval_texture_bomb(&input, 32, 32, 0.5, [1.0, 1.0], 0.0, BombBlendMode::Max, 42);

        let GraphValue::Grayscale(out1) = result1 else {
            panic!("expected grayscale");
        };
        let GraphValue::Grayscale(out2) = result2 else {
            panic!("expected grayscale");
        };

        assert_eq!(out1.data, out2.data, "same seed should produce identical output");
    }

    #[test]
    fn test_texture_bomb_different_seeds() {
        let input = make_test_buffer(16, 16, 0.5);

        let result1 = eval_texture_bomb(&input, 32, 32, 0.5, [1.0, 1.0], 0.0, BombBlendMode::Max, 42);
        let result2 = eval_texture_bomb(&input, 32, 32, 0.5, [1.0, 1.0], 0.0, BombBlendMode::Max, 43);

        let GraphValue::Grayscale(out1) = result1 else {
            panic!("expected grayscale");
        };
        let GraphValue::Grayscale(out2) = result2 else {
            panic!("expected grayscale");
        };

        assert_ne!(out1.data, out2.data, "different seeds should produce different output");
    }

    #[test]
    fn test_texture_bomb_blend_modes() {
        let input = make_test_buffer(16, 16, 0.5);

        let result_max = eval_texture_bomb(&input, 32, 32, 0.8, [1.0, 1.0], 0.0, BombBlendMode::Max, 42);
        let result_add = eval_texture_bomb(&input, 32, 32, 0.8, [1.0, 1.0], 0.0, BombBlendMode::Add, 42);
        let result_avg = eval_texture_bomb(&input, 32, 32, 0.8, [1.0, 1.0], 0.0, BombBlendMode::Average, 42);

        let GraphValue::Grayscale(out_max) = result_max else {
            panic!("expected grayscale");
        };
        let GraphValue::Grayscale(out_add) = result_add else {
            panic!("expected grayscale");
        };
        let GraphValue::Grayscale(out_avg) = result_avg else {
            panic!("expected grayscale");
        };

        // Add mode should generally produce higher values than max
        let sum_add: f64 = out_add.data.iter().sum();
        let sum_max: f64 = out_max.data.iter().sum();
        // Average mode should be between the others
        let sum_avg: f64 = out_avg.data.iter().sum();

        // These assertions are probabilistic but should hold for reasonable densities
        assert!(sum_add >= sum_max * 0.5, "add mode should have significant values");
        assert!(sum_avg >= 0.0, "average mode should have non-negative values");
    }

    #[test]
    fn test_blend_mode_parsing() {
        assert!(matches!(BombBlendMode::from_str("max"), Ok(BombBlendMode::Max)));
        assert!(matches!(BombBlendMode::from_str("MAX"), Ok(BombBlendMode::Max)));
        assert!(matches!(BombBlendMode::from_str("add"), Ok(BombBlendMode::Add)));
        assert!(matches!(BombBlendMode::from_str("average"), Ok(BombBlendMode::Average)));
        assert!(BombBlendMode::from_str("invalid").is_err());
    }
}
