//! Pattern-based height map generation for normal maps.
//!
//! This module contains height map generators for various patterns used
//! in normal map generation. Each generator creates a grayscale height map
//! that is later converted to a normal map using Sobel operators.

use speccade_spec::recipe::texture::{NoiseConfig, NormalMapPattern};

use crate::maps::GrayscaleBuffer;
use crate::rng::DeterministicRng;
use crate::shared::create_noise_generator;

/// Generate height map from a normal map pattern.
pub fn generate_height_from_pattern(
    pattern: &NormalMapPattern,
    width: u32,
    height: u32,
    seed: u32,
    tileable: bool,
) -> GrayscaleBuffer {
    match pattern {
        NormalMapPattern::Grid {
            cell_size,
            line_width,
            bevel,
        } => generate_grid_height(width, height, *cell_size, *line_width, *bevel),
        NormalMapPattern::Bricks {
            brick_width,
            brick_height,
            mortar_width,
            offset,
        } => generate_brick_height(
            width,
            height,
            *brick_width,
            *brick_height,
            *mortar_width,
            *offset,
            seed,
        ),
        NormalMapPattern::Hexagons { size, gap } => {
            generate_hexagon_height(width, height, *size, *gap)
        }
        NormalMapPattern::NoiseBumps { noise } => {
            generate_noise_height(width, height, noise, seed, tileable)
        }
        NormalMapPattern::DiamondPlate {
            diamond_size,
            height: plate_height,
        } => generate_diamond_plate_height(width, height, *diamond_size, *plate_height),
        NormalMapPattern::Tiles {
            tile_size,
            gap_width,
            gap_depth,
            seed: tile_seed,
        } => generate_tiles_height(
            width, height, *tile_size, *gap_width, *gap_depth, *tile_seed,
        ),
        NormalMapPattern::Rivets {
            spacing,
            radius,
            height: rivet_height,
            seed: rivet_seed,
        } => generate_rivets_height(width, height, *spacing, *radius, *rivet_height, *rivet_seed),
        NormalMapPattern::Weave {
            thread_width,
            gap,
            depth,
        } => generate_weave_height(width, height, *thread_width, *gap, *depth),
    }
}

/// Generate grid pattern height map.
fn generate_grid_height(
    width: u32,
    height: u32,
    cell_size: u32,
    line_width: u32,
    bevel: f64,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 1.0);

    for y in 0..height {
        for x in 0..width {
            let cell_x = x % cell_size;
            let cell_y = y % cell_size;

            // Check if we're in a grid line
            let in_line = cell_x < line_width || cell_y < line_width;

            if in_line {
                // Grid line: recessed
                buffer.set(x, y, 0.3);
            } else {
                // Inside cell: apply bevel near edges
                let dist_x = cell_x.saturating_sub(line_width).min(cell_size - cell_x);
                let dist_y = cell_y.saturating_sub(line_width).min(cell_size - cell_y);
                let edge_dist = dist_x.min(dist_y) as f64;

                let bevel_dist = (cell_size / 4) as f64;
                let bevel_factor = if edge_dist < bevel_dist {
                    (edge_dist / bevel_dist) * bevel
                } else {
                    1.0
                };

                buffer.set(x, y, 0.7 + bevel_factor * 0.3);
            }
        }
    }

    buffer
}

/// Generate brick pattern height map.
fn generate_brick_height(
    width: u32,
    height: u32,
    brick_width: u32,
    brick_height: u32,
    mortar_width: u32,
    offset: f64,
    seed: u32,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 1.0);

    let row_height = brick_height + mortar_width;
    let col_width = brick_width + mortar_width;

    for y in 0..height {
        for x in 0..width {
            let row = y / row_height;

            // Apply offset for alternating rows
            // We subtract the offset and add width to handle negative values correctly
            // This matches the brick.rs pattern behavior for proper tiling
            let row_offset = if row % 2 == 1 {
                (offset * col_width as f64) as u32
            } else {
                0
            };

            let adjusted_x = (x + width - row_offset) % width;
            let col = adjusted_x / col_width;

            let local_x = adjusted_x % col_width;
            let local_y = y % row_height;

            // Check if in mortar
            let in_mortar = local_x >= brick_width || local_y >= brick_height;

            if in_mortar {
                // Mortar: recessed
                buffer.set(x, y, 0.2);
            } else {
                // Brick: raised with slight variation
                let brick_seed = seed
                    .wrapping_add(col.wrapping_mul(374761393))
                    .wrapping_add(row.wrapping_mul(668265263));
                let mut brick_rng = DeterministicRng::new(brick_seed);
                let variation = brick_rng.gen_f64() * 0.1;

                // Distance from edge for beveling
                // Use saturating_sub to avoid underflow when local_x/y equals brick_width/height
                let dist_left = local_x;
                let dist_right = brick_width.saturating_sub(1).saturating_sub(local_x);
                let dist_x = dist_left.min(dist_right);
                let dist_top = local_y;
                let dist_bottom = brick_height.saturating_sub(1).saturating_sub(local_y);
                let dist_y = dist_top.min(dist_bottom);
                let edge_dist = dist_x.min(dist_y) as f64;

                let bevel_dist = 4.0;
                let bevel_factor = if edge_dist < bevel_dist {
                    0.9 + (edge_dist / bevel_dist) * 0.1
                } else {
                    1.0
                };

                buffer.set(x, y, (0.8 + variation) * bevel_factor);
            }
        }
    }

    buffer
}

/// Generate hexagon pattern height map.
///
/// Creates a flat-top hexagon tessellation pattern. Each hexagon has:
/// - Width (corner to corner horizontally) = 2 * size
/// - Height (flat edge to flat edge vertically) = sqrt(3) * size
fn generate_hexagon_height(width: u32, height: u32, size: u32, gap: u32) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 0.3);

    let size_f = size as f64;
    let sqrt3 = 3.0_f64.sqrt();

    // For flat-top hexagons:
    // - Column spacing (horizontal distance between centers) = 1.5 * size
    // - Row spacing (vertical distance between centers) = sqrt(3) * size
    let col_spacing = 1.5 * size_f;
    let row_spacing = sqrt3 * size_f;

    for y in 0..height {
        for x in 0..width {
            let px = x as f64;
            let py = y as f64;

            // Find the best (closest) hexagon center
            let mut best_dist = f64::MAX;

            // Check nearby hex centers to find the closest one
            // Start from an approximate position
            let approx_col = (px / col_spacing).floor() as i32;
            let approx_row = (py / row_spacing).floor() as i32;

            for dc in -1..=1 {
                for dr in -1..=1 {
                    let col = approx_col + dc;
                    let row = approx_row + dr;

                    // Skip negative indices (they're handled via tiling)
                    // But we need to check them for proper distance calculation

                    // Odd columns are offset down by half a row
                    let y_offset = if col.rem_euclid(2) == 1 {
                        row_spacing / 2.0
                    } else {
                        0.0
                    };

                    let center_x = col as f64 * col_spacing;
                    let center_y = row as f64 * row_spacing + y_offset;

                    // Calculate hexagonal distance (flat-top hex)
                    let dx = (px - center_x).abs();
                    let dy = (py - center_y).abs();

                    // Hexagonal distance formula for flat-top hexagons:
                    // The hex boundary is defined by three constraints:
                    // 1. |dx| <= size (left/right edges)
                    // 2. |dy| <= sqrt(3)/2 * size (top/bottom edges)
                    // 3. |dx|/2 + |dy| * sqrt(3)/2 <= sqrt(3)/2 * size (diagonal edges)
                    //
                    // Convert to a normalized distance where 1.0 = on the edge
                    let hex_dist = (dx / size_f)
                        .max(dy / (sqrt3 / 2.0 * size_f))
                        .max((dx / 2.0 + dy * sqrt3 / 2.0) / (sqrt3 / 2.0 * size_f));

                    if hex_dist < best_dist {
                        best_dist = hex_dist;
                    }
                }
            }

            // Account for the gap - scale the distance threshold
            let gap_ratio = gap as f64 / size_f;
            let inner_threshold = 1.0 - gap_ratio;

            if best_dist < inner_threshold {
                // Inside hexagon: raised with bevel near edges
                let edge_proximity = best_dist / inner_threshold;
                let bevel = 0.8 + (1.0 - edge_proximity) * 0.2;
                buffer.set(x, y, bevel);
            }
            // else: stays at background (gap) level
        }
    }

    buffer
}

/// Generate noise-based bumps height map.
fn generate_noise_height(
    width: u32,
    height: u32,
    noise_config: &NoiseConfig,
    seed: u32,
    tileable: bool,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 0.5);

    let noise_gen = create_noise_generator(noise_config, seed);

    for y in 0..height {
        for x in 0..width {
            let (nx, ny) = if tileable {
                // Seamless tiling using domain warping
                let u = x as f64 / width as f64;
                let v = y as f64 / height as f64;
                let angle_x = u * 2.0 * std::f64::consts::PI;
                let angle_y = v * 2.0 * std::f64::consts::PI;

                (
                    angle_x.cos() * noise_config.scale,
                    angle_y.sin() * noise_config.scale,
                )
            } else {
                (x as f64 * noise_config.scale, y as f64 * noise_config.scale)
            };

            let noise_val = noise_gen.sample_01(nx, ny);
            buffer.set(x, y, noise_val);
        }
    }

    buffer
}

/// Generate diamond plate height map.
fn generate_diamond_plate_height(
    width: u32,
    height: u32,
    diamond_size: u32,
    plate_height: f64,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 0.3);

    for y in 0..height {
        for x in 0..width {
            let cell_x = x % diamond_size;
            let cell_y = y % diamond_size;

            let center = diamond_size / 2;

            // Diamond shape using Manhattan distance
            let dist =
                (cell_x as i32 - center as i32).abs() + (cell_y as i32 - center as i32).abs();
            let max_dist = center as i32;

            if dist < max_dist {
                // Inside diamond: raised
                let factor = 1.0 - (dist as f64 / max_dist as f64);
                let height_val = 0.3 + plate_height * factor;
                buffer.set(x, y, height_val.clamp(0.0, 1.0));
            }
            // else: background level
        }
    }

    buffer
}

/// Generate tiles pattern height map.
fn generate_tiles_height(
    width: u32,
    height: u32,
    tile_size: u32,
    gap_width: u32,
    gap_depth: f64,
    seed: u32,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 1.0);

    let tile_with_gap = tile_size + gap_width;

    for y in 0..height {
        for x in 0..width {
            let tile_x = x / tile_with_gap;
            let tile_y = y / tile_with_gap;

            let local_x = x % tile_with_gap;
            let local_y = y % tile_with_gap;

            // Check if in gap
            let in_gap = local_x >= tile_size || local_y >= tile_size;

            if in_gap {
                // Gap: recessed
                buffer.set(x, y, 1.0 - gap_depth);
            } else {
                // Tile: raised with slight variation
                let tile_seed = seed
                    .wrapping_add(tile_x.wrapping_mul(374761393))
                    .wrapping_add(tile_y.wrapping_mul(668265263));
                let mut tile_rng = DeterministicRng::new(tile_seed);
                let variation = tile_rng.gen_f64() * 0.05;

                // Distance from edge for beveling
                let dist_x = local_x.min(tile_size - 1 - local_x);
                let dist_y = local_y.min(tile_size - 1 - local_y);
                let edge_dist = dist_x.min(dist_y) as f64;

                let bevel_dist = 3.0;
                let bevel_factor = if edge_dist < bevel_dist {
                    0.95 + (edge_dist / bevel_dist) * 0.05
                } else {
                    1.0
                };

                buffer.set(x, y, (1.0 + variation) * bevel_factor);
            }
        }
    }

    buffer
}

/// Generate rivets pattern height map.
fn generate_rivets_height(
    width: u32,
    height: u32,
    spacing: u32,
    radius: u32,
    rivet_height: f64,
    seed: u32,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 0.5);

    let cols = width.div_ceil(spacing);
    let rows = height.div_ceil(spacing);

    for row in 0..rows {
        for col in 0..cols {
            // Calculate rivet center with slight random offset
            let rivet_seed = seed
                .wrapping_add(col.wrapping_mul(374761393))
                .wrapping_add(row.wrapping_mul(668265263));
            let mut rivet_rng = DeterministicRng::new(rivet_seed);

            let jitter = 3.0;
            let offset_x = (rivet_rng.gen_f64() - 0.5) * jitter;
            let offset_y = (rivet_rng.gen_f64() - 0.5) * jitter;

            let center_x = (col * spacing) as f64 + offset_x;
            let center_y = (row * spacing) as f64 + offset_y;

            // Draw rivet influence
            let rivet_radius = radius as f64;
            let influence_radius = rivet_radius * 2.0;

            let min_x = ((center_x - influence_radius).max(0.0) as u32).min(width);
            let max_x = ((center_x + influence_radius).ceil() as u32).min(width);
            let min_y = ((center_y - influence_radius).max(0.0) as u32).min(height);
            let max_y = ((center_y + influence_radius).ceil() as u32).min(height);

            for y in min_y..max_y {
                for x in min_x..max_x {
                    let dx = x as f64 - center_x;
                    let dy = y as f64 - center_y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist < rivet_radius {
                        // Inside rivet: raised dome
                        let factor = 1.0 - (dist / rivet_radius);
                        let height_val = 0.5 + rivet_height * factor * 0.5;
                        buffer.set(x, y, height_val.clamp(0.0, 1.0));
                    } else if dist < rivet_radius + 2.0 {
                        // Edge transition
                        let blend = (dist - rivet_radius) / 2.0;
                        let current = buffer.get(x, y);
                        let target = 0.5 - 0.1;
                        let height_val = current * blend + target * (1.0 - blend);
                        buffer.set(x, y, height_val.clamp(0.0, 1.0));
                    }
                }
            }
        }
    }

    buffer
}

/// Generate weave/fabric pattern height map.
fn generate_weave_height(
    width: u32,
    height: u32,
    thread_width: u32,
    gap: u32,
    depth: f64,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 0.5);

    let pattern_size = (thread_width + gap) * 2;

    for y in 0..height {
        for x in 0..width {
            let pattern_x = x % pattern_size;
            let pattern_y = y % pattern_size;

            let half_pattern = pattern_size / 2;

            // Determine if horizontal or vertical thread is on top
            let h_thread = pattern_x < thread_width
                || (pattern_x >= half_pattern && pattern_x < half_pattern + thread_width);
            let v_thread = pattern_y < thread_width
                || (pattern_y >= half_pattern && pattern_y < half_pattern + thread_width);

            let in_gap_x = pattern_x >= thread_width && pattern_x < half_pattern;
            let in_gap_y = pattern_y >= thread_width && pattern_y < half_pattern;
            let in_gap_x2 = pattern_x >= half_pattern + thread_width;
            let in_gap_y2 = pattern_y >= half_pattern + thread_width;

            if (in_gap_x || in_gap_x2) && (in_gap_y || in_gap_y2) {
                // In gap
                buffer.set(x, y, 0.5 - depth * 0.5);
            } else if h_thread && v_thread {
                // Weave intersection
                let h_pos = if pattern_x < half_pattern {
                    pattern_x
                } else {
                    pattern_x - half_pattern
                };
                let v_pos = if pattern_y < half_pattern {
                    pattern_y
                } else {
                    pattern_y - half_pattern
                };

                // Alternate which thread is on top
                let h_on_top = (pattern_x < half_pattern) == (pattern_y < half_pattern);

                if h_on_top {
                    // Horizontal thread on top
                    let height_val =
                        0.5 + depth * 0.5 - (v_pos as f64 / thread_width as f64) * depth * 0.3;
                    buffer.set(x, y, height_val.clamp(0.0, 1.0));
                } else {
                    // Vertical thread on top
                    let height_val =
                        0.5 + depth * 0.5 - (h_pos as f64 / thread_width as f64) * depth * 0.3;
                    buffer.set(x, y, height_val.clamp(0.0, 1.0));
                }
            } else if h_thread {
                // Horizontal thread only
                buffer.set(x, y, 0.5 + depth * 0.25);
            } else if v_thread {
                // Vertical thread only
                buffer.set(x, y, 0.5 + depth * 0.25);
            }
        }
    }

    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::texture::NoiseAlgorithm;

    #[test]
    fn test_generate_grid_height() {
        let buffer = generate_grid_height(64, 64, 16, 2, 0.5);
        assert_eq!(buffer.width, 64);
        assert_eq!(buffer.height, 64);

        // Check that grid lines are recessed
        assert!(buffer.get(0, 0) < 0.5);
        assert!(buffer.get(1, 0) < 0.5);
    }

    #[test]
    fn test_generate_brick_height() {
        let buffer = generate_brick_height(128, 128, 32, 16, 3, 0.5, 42);
        assert_eq!(buffer.width, 128);
        assert_eq!(buffer.height, 128);
    }

    #[test]
    fn test_generate_hexagon_height() {
        let buffer = generate_hexagon_height(64, 64, 10, 2);
        assert_eq!(buffer.width, 64);
        assert_eq!(buffer.height, 64);
    }

    #[test]
    fn test_generate_noise_height() {
        let config = NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        };

        let buffer = generate_noise_height(64, 64, &config, 42, false);
        assert_eq!(buffer.width, 64);
        assert_eq!(buffer.height, 64);
    }

    #[test]
    fn test_generate_noise_height_tileable() {
        let config = NoiseConfig {
            algorithm: NoiseAlgorithm::Simplex,
            scale: 0.05,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        };

        let buffer = generate_noise_height(64, 64, &config, 42, true);
        assert_eq!(buffer.width, 64);
        assert_eq!(buffer.height, 64);
    }

    #[test]
    fn test_generate_diamond_plate_height() {
        let buffer = generate_diamond_plate_height(64, 64, 16, 0.5);
        assert_eq!(buffer.width, 64);
        assert_eq!(buffer.height, 64);
    }

    #[test]
    fn test_generate_tiles_height() {
        let buffer = generate_tiles_height(64, 64, 16, 2, 0.4, 42);
        assert_eq!(buffer.width, 64);
        assert_eq!(buffer.height, 64);
    }

    #[test]
    fn test_generate_rivets_height() {
        let buffer = generate_rivets_height(64, 64, 16, 3, 0.5, 42);
        assert_eq!(buffer.width, 64);
        assert_eq!(buffer.height, 64);
    }

    #[test]
    fn test_generate_weave_height() {
        let buffer = generate_weave_height(64, 64, 6, 1, 0.2);
        assert_eq!(buffer.width, 64);
        assert_eq!(buffer.height, 64);
    }

    #[test]
    fn test_generate_height_from_pattern_grid() {
        let pattern = NormalMapPattern::Grid {
            cell_size: 16,
            line_width: 2,
            bevel: 0.5,
        };

        let buffer = generate_height_from_pattern(&pattern, 64, 64, 42, false);
        assert_eq!(buffer.width, 64);
        assert_eq!(buffer.height, 64);
    }

    #[test]
    fn test_generate_height_from_pattern_all_types() {
        let patterns = vec![
            NormalMapPattern::Grid {
                cell_size: 16,
                line_width: 2,
                bevel: 0.5,
            },
            NormalMapPattern::Bricks {
                brick_width: 32,
                brick_height: 16,
                mortar_width: 3,
                offset: 0.5,
            },
            NormalMapPattern::Hexagons { size: 10, gap: 2 },
            NormalMapPattern::NoiseBumps {
                noise: NoiseConfig {
                    algorithm: NoiseAlgorithm::Perlin,
                    scale: 0.1,
                    octaves: 4,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
            },
            NormalMapPattern::DiamondPlate {
                diamond_size: 16,
                height: 0.5,
            },
            NormalMapPattern::Tiles {
                tile_size: 16,
                gap_width: 2,
                gap_depth: 0.4,
                seed: 42,
            },
            NormalMapPattern::Rivets {
                spacing: 16,
                radius: 3,
                height: 0.5,
                seed: 42,
            },
            NormalMapPattern::Weave {
                thread_width: 6,
                gap: 1,
                depth: 0.2,
            },
        ];

        for pattern in patterns {
            let buffer = generate_height_from_pattern(&pattern, 32, 32, 42, false);
            assert_eq!(buffer.width, 32);
            assert_eq!(buffer.height, 32);
        }
    }

    #[test]
    fn test_determinism() {
        let pattern = NormalMapPattern::Bricks {
            brick_width: 32,
            brick_height: 16,
            mortar_width: 3,
            offset: 0.5,
        };

        let buffer1 = generate_height_from_pattern(&pattern, 64, 64, 42, false);
        let buffer2 = generate_height_from_pattern(&pattern, 64, 64, 42, false);

        assert_eq!(buffer1.data, buffer2.data);
    }

    #[test]
    fn test_different_seeds_produce_different_output() {
        let pattern = NormalMapPattern::Bricks {
            brick_width: 32,
            brick_height: 16,
            mortar_width: 3,
            offset: 0.5,
        };

        let buffer1 = generate_height_from_pattern(&pattern, 64, 64, 42, false);
        let buffer2 = generate_height_from_pattern(&pattern, 64, 64, 100, false);

        assert_ne!(buffer1.data, buffer2.data);
    }
}
