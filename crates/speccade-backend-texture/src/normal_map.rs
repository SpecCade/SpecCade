//! Dedicated normal map generation from patterns.
//!
//! This module generates normal maps directly from pattern specifications,
//! as opposed to deriving them from height maps. This provides more control
//! over normal map appearance and supports pattern-specific optimizations.

use std::path::Path;

use speccade_spec::recipe::texture::{
    Texture2dNormalMapV1Params, NormalMapPattern, NoiseConfig, NoiseAlgorithm, NormalMapProcessing,
};

use crate::color::Color;
use crate::maps::{TextureBuffer, GrayscaleBuffer};
use crate::noise::{Noise2D, Fbm, PerlinNoise, SimplexNoise, WorleyNoise};
use crate::png::{self, PngConfig, PngError};
use crate::rng::DeterministicRng;

/// Errors from normal map generation.
#[derive(Debug, thiserror::Error)]
pub enum NormalMapError {
    #[error("PNG error: {0}")]
    Png(#[from] PngError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result of generating a normal map.
#[derive(Debug)]
pub struct NormalMapResult {
    /// The generated normal map data (RGB PNG).
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// BLAKE3 hash of the PNG file.
    pub hash: String,
    /// File path if saved.
    pub file_path: Option<String>,
}

/// Generate a normal map from pattern specification.
pub fn generate_normal_map(
    params: &Texture2dNormalMapV1Params,
    seed: u32,
) -> Result<NormalMapResult, NormalMapError> {
    let width = params.resolution[0];
    let height = params.resolution[1];

    if width == 0 || height == 0 {
        return Err(NormalMapError::InvalidParameter(format!(
            "resolution must be at least 1x1, got [{}, {}]",
            width, height
        )));
    }

    (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| NormalMapError::InvalidParameter("resolution is too large".to_string()))?;

    if !params.bump_strength.is_finite() || params.bump_strength < 0.0 {
        return Err(NormalMapError::InvalidParameter(format!(
            "bump_strength must be finite and >= 0, got {}",
            params.bump_strength
        )));
    }

    // Generate height map from pattern
    let mut height_map = if let Some(pattern) = &params.pattern {
        generate_height_from_pattern(pattern, width, height, seed, params.tileable)
    } else {
        // No pattern: generate flat normal map
        GrayscaleBuffer::new(width, height, 0.5)
    };

    // Apply post-processing
    if let Some(processing) = &params.processing {
        apply_processing(&mut height_map, processing);
    }

    // Convert height map to normal map using Sobel operators
    let normal_buffer = height_to_normal(&height_map, params.bump_strength);

    // Encode to PNG with hash
    let config = PngConfig::default();
    let (data, hash) = png::write_rgb_to_vec_with_hash(&normal_buffer, &config)?;

    Ok(NormalMapResult {
        data,
        width,
        height,
        hash,
        file_path: None,
    })
}

/// Save normal map result to file.
pub fn save_normal_map(
    result: &NormalMapResult,
    output_path: &Path,
) -> Result<NormalMapResult, NormalMapError> {
    // Create parent directory if needed
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write PNG to file
    std::fs::write(output_path, &result.data)?;

    Ok(NormalMapResult {
        data: result.data.clone(),
        width: result.width,
        height: result.height,
        hash: result.hash.clone(),
        file_path: Some(output_path.to_string_lossy().to_string()),
    })
}

/// Apply post-processing to height map.
fn apply_processing(height_map: &mut GrayscaleBuffer, processing: &NormalMapProcessing) {
    // Apply blur if specified
    if let Some(sigma) = processing.blur {
        if sigma > 0.0 {
            apply_gaussian_blur(height_map, sigma);
        }
    }

    // Apply invert if specified
    if processing.invert {
        for value in &mut height_map.data {
            *value = 1.0 - *value;
        }
    }
}

/// Apply Gaussian blur to a height map.
fn apply_gaussian_blur(height_map: &mut GrayscaleBuffer, sigma: f64) {
    let width = height_map.width;
    let height = height_map.height;

    // Calculate kernel size (3 sigma on each side)
    let kernel_size = ((sigma * 3.0).ceil() as usize * 2 + 1).max(3);
    let half_kernel = kernel_size / 2;

    // Generate Gaussian kernel
    let mut kernel = vec![0.0; kernel_size];
    let mut sum = 0.0;
    for i in 0..kernel_size {
        let x = i as f64 - half_kernel as f64;
        let value = (-x * x / (2.0 * sigma * sigma)).exp();
        kernel[i] = value;
        sum += value;
    }
    // Normalize kernel
    for value in &mut kernel {
        *value /= sum;
    }

    // Horizontal pass
    let mut temp = vec![0.0; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            for i in 0..kernel_size {
                let offset = i as i32 - half_kernel as i32;
                let sample_x = (x as i32 + offset).rem_euclid(width as i32) as u32;
                sum += height_map.get(sample_x, y) * kernel[i];
            }
            temp[(y * width + x) as usize] = sum;
        }
    }

    // Vertical pass
    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            for i in 0..kernel_size {
                let offset = i as i32 - half_kernel as i32;
                let sample_y = (y as i32 + offset).rem_euclid(height as i32) as u32;
                sum += temp[(sample_y * width + x) as usize] * kernel[i];
            }
            height_map.set(x, y, sum);
        }
    }
}

/// Generate height map from a normal map pattern.
fn generate_height_from_pattern(
    pattern: &NormalMapPattern,
    width: u32,
    height: u32,
    seed: u32,
    tileable: bool,
) -> GrayscaleBuffer {
    match pattern {
        NormalMapPattern::Grid { cell_size, line_width, bevel } => {
            generate_grid_height(width, height, *cell_size, *line_width, *bevel)
        }
        NormalMapPattern::Bricks { brick_width, brick_height, mortar_width, offset } => {
            generate_brick_height(width, height, *brick_width, *brick_height, *mortar_width, *offset, seed)
        }
        NormalMapPattern::Hexagons { size, gap } => {
            generate_hexagon_height(width, height, *size, *gap)
        }
        NormalMapPattern::NoiseBumps { noise } => {
            generate_noise_height(width, height, noise, seed, tileable)
        }
        NormalMapPattern::DiamondPlate { diamond_size, height: plate_height } => {
            generate_diamond_plate_height(width, height, *diamond_size, *plate_height)
        }
        NormalMapPattern::Tiles { tile_size, gap_width, gap_depth, seed: tile_seed } => {
            generate_tiles_height(width, height, *tile_size, *gap_width, *gap_depth, *tile_seed)
        }
        NormalMapPattern::Rivets { spacing, radius, height: rivet_height, seed: rivet_seed } => {
            generate_rivets_height(width, height, *spacing, *radius, *rivet_height, *rivet_seed)
        }
        NormalMapPattern::Weave { thread_width, gap, depth } => {
            generate_weave_height(width, height, *thread_width, *gap, *depth)
        }
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
            let row_offset = if row % 2 == 1 {
                (offset * col_width as f64) as u32
            } else {
                0
            };

            let adjusted_x = (x + row_offset) % width;
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
                let dist_x = local_x.min(brick_width - local_x);
                let dist_y = local_y.min(brick_height - local_y);
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
fn generate_hexagon_height(width: u32, height: u32, size: u32, gap: u32) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 0.3);

    let hex_width = size * 2;
    let hex_height = ((size as f64) * 1.732).round() as u32; // sqrt(3) ≈ 1.732

    let col_spacing = (hex_width as f64 * 0.75) as u32;
    let row_spacing = hex_height;

    for y in 0..height {
        for x in 0..width {
            // Find nearest hexagon center
            let col = x / col_spacing;
            let row = y / row_spacing;

            // Offset odd rows
            let x_offset = if row % 2 == 1 { col_spacing / 2 } else { 0 };

            let center_x = col * col_spacing + x_offset;
            let center_y = row * row_spacing;

            // Distance to hexagon center
            let dx = (x as i32 - center_x as i32).abs() as f64;
            let dy = (y as i32 - center_y as i32).abs() as f64;

            // Approximate hexagon distance (simplified)
            let hex_dist = (dx + dy * 0.866).max(dx);
            let hex_radius = (size - gap) as f64;

            if hex_dist < hex_radius {
                // Inside hexagon: raised
                let edge_factor = 1.0 - (hex_dist / hex_radius);
                let bevel = 0.8 + edge_factor * 0.2;
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
                (
                    x as f64 * noise_config.scale,
                    y as f64 * noise_config.scale,
                )
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
            let dist = (cell_x as i32 - center as i32).abs() + (cell_y as i32 - center as i32).abs();
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

/// Create a noise generator from config.
fn create_noise_generator(config: &NoiseConfig, seed: u32) -> Box<dyn Noise2D> {
    let base_noise: Box<dyn Noise2D> = match config.algorithm {
        NoiseAlgorithm::Perlin => Box::new(PerlinNoise::new(seed)),
        NoiseAlgorithm::Simplex => Box::new(SimplexNoise::new(seed)),
        NoiseAlgorithm::Worley => Box::new(WorleyNoise::new(seed)),
        NoiseAlgorithm::Value => Box::new(PerlinNoise::new(seed)), // Use Perlin as fallback
        NoiseAlgorithm::Fbm => {
            Box::new(
                Fbm::new(PerlinNoise::new(seed))
                    .with_octaves(config.octaves)
                    .with_persistence(config.persistence)
                    .with_lacunarity(config.lacunarity)
            )
        }
    };

    // Wrap in FBM if octaves > 1 and not already FBM
    if config.octaves > 1 && config.algorithm != NoiseAlgorithm::Fbm {
        match config.algorithm {
            NoiseAlgorithm::Perlin => {
                Box::new(
                    Fbm::new(PerlinNoise::new(seed))
                        .with_octaves(config.octaves)
                        .with_persistence(config.persistence)
                        .with_lacunarity(config.lacunarity)
                )
            }
            NoiseAlgorithm::Simplex => {
                Box::new(
                    Fbm::new(SimplexNoise::new(seed))
                        .with_octaves(config.octaves)
                        .with_persistence(config.persistence)
                        .with_lacunarity(config.lacunarity)
                )
            }
            _ => base_noise,
        }
    } else {
        base_noise
    }
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

    let cols = (width + spacing - 1) / spacing;
    let rows = (height + spacing - 1) / spacing;

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
            let h_thread = pattern_x < thread_width || (pattern_x >= half_pattern && pattern_x < half_pattern + thread_width);
            let v_thread = pattern_y < thread_width || (pattern_y >= half_pattern && pattern_y < half_pattern + thread_width);

            let in_gap_x = pattern_x >= thread_width && pattern_x < half_pattern;
            let in_gap_y = pattern_y >= thread_width && pattern_y < half_pattern;
            let in_gap_x2 = pattern_x >= half_pattern + thread_width;
            let in_gap_y2 = pattern_y >= half_pattern + thread_width;

            if (in_gap_x || in_gap_x2) && (in_gap_y || in_gap_y2) {
                // In gap
                buffer.set(x, y, 0.5 - depth * 0.5);
            } else if h_thread && v_thread {
                // Weave intersection
                let h_pos = if pattern_x < half_pattern { pattern_x } else { pattern_x - half_pattern };
                let v_pos = if pattern_y < half_pattern { pattern_y } else { pattern_y - half_pattern };

                // Alternate which thread is on top
                let h_on_top = (pattern_x < half_pattern) == (pattern_y < half_pattern);

                if h_on_top {
                    // Horizontal thread on top
                    let height_val = 0.5 + depth * 0.5 - (v_pos as f64 / thread_width as f64) * depth * 0.3;
                    buffer.set(x, y, height_val.clamp(0.0, 1.0));
                } else {
                    // Vertical thread on top
                    let height_val = 0.5 + depth * 0.5 - (h_pos as f64 / thread_width as f64) * depth * 0.3;
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

/// Convert height map to normal map using Sobel operators.
#[allow(clippy::needless_range_loop)]
fn height_to_normal(height_map: &GrayscaleBuffer, strength: f64) -> TextureBuffer {
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

            // Create normal vector
            // For tangent-space normal maps:
            // X = right (positive = bump facing right)
            // Y = up (positive = bump facing up)
            // Z = out (positive = facing camera)
            let nx = -gx;
            let ny = -gy;
            let nz = 1.0;

            // Normalize
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            let nx = nx / len;
            let ny = ny / len;
            let nz = nz / len;

            // Convert from [-1, 1] to [0, 1] for storage in RGB
            buffer.set(x, y, Color::rgb(
                (nx + 1.0) * 0.5,
                (ny + 1.0) * 0.5,
                (nz + 1.0) * 0.5,
            ));
        }
    }

    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_flat_normal() {
        let params = Texture2dNormalMapV1Params {
            resolution: [64, 64],
            tileable: false,
            pattern: None,
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 64);
        assert_eq!(result.height, 64);
        assert!(!result.hash.is_empty());
    }

    #[test]
    fn test_generate_grid_normal() {
        let params = Texture2dNormalMapV1Params {
            resolution: [128, 128],
            tileable: true,
            pattern: Some(NormalMapPattern::Grid {
                cell_size: 32,
                line_width: 4,
                bevel: 0.5,
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 128);
        assert_eq!(result.height, 128);
    }

    #[test]
    fn test_generate_brick_normal() {
        let params = Texture2dNormalMapV1Params {
            resolution: [256, 256],
            tileable: true,
            pattern: Some(NormalMapPattern::Bricks {
                brick_width: 64,
                brick_height: 32,
                mortar_width: 4,
                offset: 0.5,
            }),
            bump_strength: 1.5,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 256);
        assert_eq!(result.height, 256);
    }

    #[test]
    fn test_deterministic() {
        let params = Texture2dNormalMapV1Params {
            resolution: [64, 64],
            tileable: false,
            pattern: Some(NormalMapPattern::NoiseBumps {
                noise: NoiseConfig {
                    algorithm: NoiseAlgorithm::Perlin,
                    scale: 0.05,
                    octaves: 4,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result1 = generate_normal_map(&params, 42).unwrap();
        let result2 = generate_normal_map(&params, 42).unwrap();

        assert_eq!(result1.hash, result2.hash);
        assert_eq!(result1.data, result2.data);
    }

    #[test]
    fn test_generate_normal_map_invalid_resolution() {
        let params = Texture2dNormalMapV1Params {
            resolution: [0, 64],
            tileable: false,
            pattern: None,
            bump_strength: 1.0,
            processing: None,
        };

        let err = generate_normal_map(&params, 42).unwrap_err();
        assert!(err.to_string().contains("resolution"));
    }

    #[test]
    fn test_generate_normal_map_invalid_bump_strength() {
        let params = Texture2dNormalMapV1Params {
            resolution: [64, 64],
            tileable: false,
            pattern: None,
            bump_strength: -1.0,
            processing: None,
        };

        let err = generate_normal_map(&params, 42).unwrap_err();
        assert!(err.to_string().contains("bump_strength"));
    }

    // ========================================================================
    // All Pattern Types Tests
    // ========================================================================

    #[test]
    fn test_pattern_tiles() {
        let params = Texture2dNormalMapV1Params {
            resolution: [128, 128],
            tileable: true,
            pattern: Some(NormalMapPattern::Tiles {
                tile_size: 32,
                gap_width: 3,
                gap_depth: 0.4,
                seed: 42,
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 128);
        assert_eq!(result.height, 128);
        assert!(!result.hash.is_empty());
    }

    #[test]
    fn test_pattern_hexagons() {
        let params = Texture2dNormalMapV1Params {
            resolution: [256, 256],
            tileable: true,
            pattern: Some(NormalMapPattern::Hexagons {
                size: 20,
                gap: 2,
            }),
            bump_strength: 1.2,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 256);
    }

    #[test]
    fn test_pattern_rivets() {
        let params = Texture2dNormalMapV1Params {
            resolution: [128, 128],
            tileable: false,
            pattern: Some(NormalMapPattern::Rivets {
                spacing: 24,
                radius: 3,
                height: 0.25,
                seed: 123,
            }),
            bump_strength: 1.5,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 128);
    }

    #[test]
    fn test_pattern_weave() {
        let params = Texture2dNormalMapV1Params {
            resolution: [128, 128],
            tileable: true,
            pattern: Some(NormalMapPattern::Weave {
                thread_width: 6,
                gap: 1,
                depth: 0.2,
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 128);
    }

    #[test]
    fn test_pattern_diamond_plate() {
        let params = Texture2dNormalMapV1Params {
            resolution: [256, 256],
            tileable: true,
            pattern: Some(NormalMapPattern::DiamondPlate {
                diamond_size: 40,
                height: 0.35,
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 256);
    }

    #[test]
    fn test_all_noise_algorithms() {
        for algo in [
            NoiseAlgorithm::Perlin,
            NoiseAlgorithm::Simplex,
            NoiseAlgorithm::Worley,
            NoiseAlgorithm::Value,
            NoiseAlgorithm::Fbm,
        ] {
            let params = Texture2dNormalMapV1Params {
                resolution: [64, 64],
                tileable: false,
                pattern: Some(NormalMapPattern::NoiseBumps {
                    noise: NoiseConfig {
                        algorithm: algo,
                        scale: 0.05,
                        octaves: 4,
                        persistence: 0.5,
                        lacunarity: 2.0,
                    },
                }),
                bump_strength: 1.0,
                processing: None,
            };

            let result = generate_normal_map(&params, 42).unwrap();
            assert_eq!(result.width, 64);
            assert_eq!(result.height, 64);
        }
    }

    // ========================================================================
    // Processing Options Tests
    // ========================================================================

    #[test]
    fn test_processing_blur() {
        let params = Texture2dNormalMapV1Params {
            resolution: [64, 64],
            tileable: false,
            pattern: Some(NormalMapPattern::Grid {
                cell_size: 16,
                line_width: 2,
                bevel: 0.3,
            }),
            bump_strength: 1.0,
            processing: Some(NormalMapProcessing {
                blur: Some(1.5),
                invert: false,
            }),
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert!(!result.data.is_empty());
    }

    #[test]
    fn test_processing_invert() {
        let params = Texture2dNormalMapV1Params {
            resolution: [64, 64],
            tileable: false,
            pattern: Some(NormalMapPattern::Bricks {
                brick_width: 32,
                brick_height: 16,
                mortar_width: 3,
                offset: 0.5,
            }),
            bump_strength: 1.0,
            processing: Some(NormalMapProcessing {
                blur: None,
                invert: true,
            }),
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert!(!result.data.is_empty());
    }

    #[test]
    fn test_processing_blur_and_invert() {
        let params = Texture2dNormalMapV1Params {
            resolution: [64, 64],
            tileable: false,
            pattern: Some(NormalMapPattern::Tiles {
                tile_size: 20,
                gap_width: 2,
                gap_depth: 0.3,
                seed: 42,
            }),
            bump_strength: 1.0,
            processing: Some(NormalMapProcessing {
                blur: Some(2.0),
                invert: true,
            }),
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert!(!result.data.is_empty());
    }

    // ========================================================================
    // Bump Strength Tests
    // ========================================================================

    #[test]
    fn test_various_bump_strengths() {
        for strength in [0.5, 1.0, 1.5, 2.0, 3.0] {
            let params = Texture2dNormalMapV1Params {
                resolution: [32, 32],
                tileable: false,
                pattern: Some(NormalMapPattern::Grid {
                    cell_size: 8,
                    line_width: 1,
                    bevel: 0.5,
                }),
                bump_strength: strength,
                processing: None,
            };

            let result = generate_normal_map(&params, 42).unwrap();
            assert_eq!(result.width, 32);
            assert_eq!(result.height, 32);
        }
    }

    // ========================================================================
    // Tileable Tests
    // ========================================================================

    #[test]
    fn test_tileable_noise() {
        let params = Texture2dNormalMapV1Params {
            resolution: [128, 128],
            tileable: true,
            pattern: Some(NormalMapPattern::NoiseBumps {
                noise: NoiseConfig {
                    algorithm: NoiseAlgorithm::Perlin,
                    scale: 0.1,
                    octaves: 4,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 128);
    }

    #[test]
    fn test_non_tileable_noise() {
        let params = Texture2dNormalMapV1Params {
            resolution: [128, 128],
            tileable: false,
            pattern: Some(NormalMapPattern::NoiseBumps {
                noise: NoiseConfig {
                    algorithm: NoiseAlgorithm::Simplex,
                    scale: 0.05,
                    octaves: 6,
                    persistence: 0.6,
                    lacunarity: 2.2,
                },
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 128);
    }

    // ========================================================================
    // Seed Variation Tests
    // ========================================================================

    #[test]
    fn test_seed_affects_bricks() {
        let make_params = |seed| Texture2dNormalMapV1Params {
            resolution: [64, 64],
            tileable: false,
            pattern: Some(NormalMapPattern::Bricks {
                brick_width: 32,
                brick_height: 16,
                mortar_width: 3,
                offset: 0.5,
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result1 = generate_normal_map(&make_params(1), 1).unwrap();
        let result2 = generate_normal_map(&make_params(2), 2).unwrap();

        assert_ne!(result1.hash, result2.hash);
    }

    #[test]
    fn test_seed_affects_tiles() {
        let make_params = |seed| Texture2dNormalMapV1Params {
            resolution: [64, 64],
            tileable: false,
            pattern: Some(NormalMapPattern::Tiles {
                tile_size: 20,
                gap_width: 2,
                gap_depth: 0.3,
                seed,
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result1 = generate_normal_map(&make_params(42), 1).unwrap();
        let result2 = generate_normal_map(&make_params(100), 2).unwrap();

        assert_ne!(result1.hash, result2.hash);
    }

    // ========================================================================
    // File Save Tests
    // ========================================================================

    #[test]
    fn test_save_normal_map() {
        let params = Texture2dNormalMapV1Params {
            resolution: [64, 64],
            tileable: false,
            pattern: Some(NormalMapPattern::Grid {
                cell_size: 16,
                line_width: 2,
                bevel: 0.5,
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let output_path = tmp.path().join("test_normal.png");

        let saved = save_normal_map(&result, &output_path).unwrap();
        assert!(output_path.exists());
        assert!(saved.file_path.is_some());
    }

    // ========================================================================
    // Pattern Parameter Variation Tests
    // ========================================================================

    #[test]
    fn test_bricks_with_different_sizes() {
        for brick_width in [32, 64, 128] {
            let params = Texture2dNormalMapV1Params {
                resolution: [256, 256],
                tileable: true,
                pattern: Some(NormalMapPattern::Bricks {
                    brick_width,
                    brick_height: brick_width / 2,
                    mortar_width: 4,
                    offset: 0.5,
                }),
                bump_strength: 1.0,
                processing: None,
            };

            let result = generate_normal_map(&params, 42).unwrap();
            assert_eq!(result.width, 256);
        }
    }

    #[test]
    fn test_tiles_with_different_gap_depths() {
        for gap_depth in [0.1, 0.3, 0.5, 0.7] {
            let params = Texture2dNormalMapV1Params {
                resolution: [128, 128],
                tileable: true,
                pattern: Some(NormalMapPattern::Tiles {
                    tile_size: 32,
                    gap_width: 3,
                    gap_depth,
                    seed: 42,
                }),
                bump_strength: 1.0,
                processing: None,
            };

            let result = generate_normal_map(&params, 42).unwrap();
            assert_eq!(result.width, 128);
        }
    }

    #[test]
    fn test_weave_with_different_thread_widths() {
        for thread_width in [4, 8, 12, 16] {
            let params = Texture2dNormalMapV1Params {
                resolution: [128, 128],
                tileable: true,
                pattern: Some(NormalMapPattern::Weave {
                    thread_width,
                    gap: 2,
                    depth: 0.15,
                }),
                bump_strength: 1.0,
                processing: None,
            };

            let result = generate_normal_map(&params, 42).unwrap();
            assert_eq!(result.width, 128);
        }
    }
}
