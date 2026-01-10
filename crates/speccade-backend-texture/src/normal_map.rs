//! Dedicated normal map generation from patterns.
//!
//! This module generates normal maps directly from pattern specifications,
//! as opposed to deriving them from height maps. This provides more control
//! over normal map appearance and supports pattern-specific optimizations.

use std::path::Path;

use speccade_spec::recipe::texture::{NormalMapProcessing, TextureNormalV1Params};
use speccade_spec::validation::common as shared_validation;
use speccade_spec::BackendError;

use crate::color::Color;
use crate::maps::{GrayscaleBuffer, TextureBuffer};
use crate::normal_map_patterns::generate_height_from_pattern;
use crate::png::{self, PngConfig, PngError};

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

impl BackendError for NormalMapError {
    fn code(&self) -> &'static str {
        match self {
            NormalMapError::Png(_) => "NORMAL_001",
            NormalMapError::Io(_) => "NORMAL_002",
            NormalMapError::InvalidParameter(_) => "NORMAL_003",
        }
    }

    fn category(&self) -> &'static str {
        "texture"
    }
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
    params: &TextureNormalV1Params,
    seed: u32,
) -> Result<NormalMapResult, NormalMapError> {
    let width = params.resolution[0];
    let height = params.resolution[1];

    // Use shared validation for resolution
    shared_validation::validate_resolution(width, height)
        .map_err(|e| NormalMapError::InvalidParameter(e.message))?;

    // Use shared validation for bump_strength (must be non-negative)
    shared_validation::validate_non_negative("bump_strength", params.bump_strength)
        .map_err(|e| NormalMapError::InvalidParameter(e.message))?;

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
    for (i, kernel_value) in kernel.iter_mut().enumerate() {
        let x = i as f64 - half_kernel as f64;
        let value = (-x * x / (2.0 * sigma * sigma)).exp();
        *kernel_value = value;
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
            for (i, kernel_value) in kernel.iter().enumerate() {
                let offset = i as i32 - half_kernel as i32;
                let sample_x = (x as i32 + offset).rem_euclid(width as i32) as u32;
                sum += height_map.get(sample_x, y) * kernel_value;
            }
            temp[(y * width + x) as usize] = sum;
        }
    }

    // Vertical pass
    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            for (i, kernel_value) in kernel.iter().enumerate() {
                let offset = i as i32 - half_kernel as i32;
                let sample_y = (y as i32 + offset).rem_euclid(height as i32) as u32;
                sum += temp[(sample_y * width + x) as usize] * kernel_value;
            }
            height_map.set(x, y, sum);
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::texture::{NoiseAlgorithm, NoiseConfig, NormalMapPattern};

    #[test]
    fn test_generate_flat_normal() {
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
            resolution: [256, 256],
            tileable: true,
            pattern: Some(NormalMapPattern::Hexagons { size: 20, gap: 2 }),
            bump_strength: 1.2,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 256);
    }

    #[test]
    fn test_pattern_rivets() {
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
            let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
            let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
        let make_params = || TextureNormalV1Params {
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

        let result1 = generate_normal_map(&make_params(), 1).unwrap();
        let result2 = generate_normal_map(&make_params(), 2).unwrap();

        assert_ne!(result1.hash, result2.hash);
    }

    #[test]
    fn test_seed_affects_tiles() {
        let make_params = |seed| TextureNormalV1Params {
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
        let params = TextureNormalV1Params {
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
            let params = TextureNormalV1Params {
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
            let params = TextureNormalV1Params {
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
            let params = TextureNormalV1Params {
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
