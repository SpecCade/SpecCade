//! Surface texture height map patterns.
//!
//! This module contains height map generators for various surface textures:
//! noise-based bumps, diamond plate, tiles, rivets, and woven fabrics.

use speccade_spec::recipe::texture::NoiseConfig;

use crate::maps::GrayscaleBuffer;
use crate::rng::DeterministicRng;
use crate::shared::create_noise_generator;

/// Generate noise-based bumps height map.
pub fn generate_noise_height(
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
pub fn generate_diamond_plate_height(
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
pub fn generate_tiles_height(
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
pub fn generate_rivets_height(
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
pub fn generate_weave_height(
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
