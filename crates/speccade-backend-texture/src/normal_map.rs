//! Dedicated normal map generation from patterns.
//!
//! This module generates normal maps directly from pattern specifications,
//! as opposed to deriving them from height maps. This provides more control
//! over normal map appearance and supports pattern-specific optimizations.

use std::path::Path;

use speccade_spec::recipe::texture::{
    Texture2dNormalMapV1Params, NormalMapPattern, NoiseConfig, NoiseAlgorithm,
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

    // Generate height map from pattern
    let height_map = if let Some(pattern) = &params.pattern {
        generate_height_from_pattern(pattern, width, height, seed, params.tileable)
    } else {
        // No pattern: generate flat normal map
        GrayscaleBuffer::new(width, height, 0.5)
    };

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
        };

        let result1 = generate_normal_map(&params, 42).unwrap();
        let result2 = generate_normal_map(&params, 42).unwrap();

        assert_eq!(result1.hash, result2.hash);
        assert_eq!(result1.data, result2.data);
    }
}
