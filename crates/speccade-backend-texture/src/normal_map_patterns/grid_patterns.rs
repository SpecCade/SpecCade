//! Grid-based geometric height map patterns.
//!
//! This module contains height map generators for geometric patterns
//! based on regular grids: simple grids, brick layouts, and hexagon tilings.

use crate::maps::GrayscaleBuffer;
use crate::rng::DeterministicRng;

/// Generate grid pattern height map.
pub fn generate_grid_height(
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
pub fn generate_brick_height(
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
pub fn generate_hexagon_height(width: u32, height: u32, size: u32, gap: u32) -> GrayscaleBuffer {
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
