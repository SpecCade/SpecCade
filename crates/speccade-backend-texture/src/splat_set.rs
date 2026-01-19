//! Splat set texture generation.
//!
//! Generates terrain splat/blend textures with multiple material layers,
//! RGBA splat masks, per-layer PBR outputs, and macro variation overlays.

use speccade_spec::recipe::texture::{
    SplatLayer, SplatLayerMetadata, SplatMaskMode, SplatSetOutputMetadata, TextureSplatSetV1Params,
};
use thiserror::Error;

use crate::color::Color;
use crate::maps::{GrayscaleBuffer, NormalGenerator, TextureBuffer};
use crate::noise::{Fbm, Noise2D, PerlinNoise, SimplexNoise};
use crate::png::{write_grayscale_to_vec_with_hash, write_rgba_to_vec_with_hash, PngConfig, PngError};
use crate::rng::DeterministicRng;

/// Errors that can occur during splat set generation.
#[derive(Debug, Error)]
pub enum SplatSetError {
    /// PNG encoding failed.
    #[error("PNG encoding failed: {0}")]
    PngError(#[from] PngError),

    /// Invalid parameter.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result of a single texture output (PNG).
#[derive(Debug)]
pub struct SplatTextureResult {
    /// PNG-encoded image data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
}

/// Per-layer output textures.
#[derive(Debug)]
pub struct SplatLayerOutput {
    /// Layer identifier.
    pub id: String,
    /// Albedo texture (RGBA).
    pub albedo: SplatTextureResult,
    /// Normal map texture (RGB).
    pub normal: SplatTextureResult,
    /// Roughness texture (grayscale).
    pub roughness: SplatTextureResult,
}

/// Result of splat set generation.
#[derive(Debug)]
pub struct SplatSetResult {
    /// Per-layer outputs.
    pub layer_outputs: Vec<SplatLayerOutput>,
    /// Splat mask textures (RGBA, each channel is a layer weight).
    pub splat_masks: Vec<SplatTextureResult>,
    /// Optional macro variation texture.
    pub macro_variation: Option<SplatTextureResult>,
    /// Metadata for the splat set.
    pub metadata: SplatSetOutputMetadata,
}

/// Generate a splat set from parameters.
///
/// # Arguments
/// * `params` - Splat set parameters including layers, mask mode, and macro settings.
/// * `seed` - Deterministic seed for noise operations.
///
/// # Returns
/// A `SplatSetResult` containing all PNG outputs and metadata.
pub fn generate_splat_set(
    params: &TextureSplatSetV1Params,
    seed: u32,
) -> Result<SplatSetResult, SplatSetError> {
    let width = params.resolution[0];
    let height = params.resolution[1];

    // Validate parameters
    if width == 0 || height == 0 {
        return Err(SplatSetError::InvalidParameter(
            "Resolution must be non-zero".to_string(),
        ));
    }

    if params.layers.is_empty() {
        return Err(SplatSetError::InvalidParameter(
            "At least one layer is required".to_string(),
        ));
    }

    let config = PngConfig::default();
    let mut rng = DeterministicRng::new(seed);

    // Generate per-layer outputs
    let mut layer_outputs = Vec::with_capacity(params.layers.len());
    for (i, layer) in params.layers.iter().enumerate() {
        let layer_seed = rng.gen_u32();
        let output = generate_layer_textures(layer, width, height, layer_seed, &config)?;
        layer_outputs.push(output);

        // Use index for additional seed mixing
        let _ = i;
    }

    // Generate splat masks (up to 4 layers per mask)
    let splat_masks = generate_splat_masks(params, width, height, seed, &config)?;

    // Generate macro variation if enabled
    let macro_variation = if params.macro_variation {
        let macro_seed = seed.wrapping_add(0x12345678);
        Some(generate_macro_variation(
            width,
            height,
            params.macro_scale,
            params.macro_intensity,
            macro_seed,
            &config,
        )?)
    } else {
        None
    };

    // Build metadata
    let layers_meta: Vec<SplatLayerMetadata> = params
        .layers
        .iter()
        .enumerate()
        .map(|(i, layer)| SplatLayerMetadata {
            id: layer.id.clone(),
            mask_index: (i / 4) as u32,
            mask_channel: (i % 4) as u32,
            roughness: layer.roughness,
        })
        .collect();

    let metadata = SplatSetOutputMetadata {
        resolution: params.resolution,
        layers: layers_meta,
        mask_mode: format!("{:?}", params.mask_mode).to_lowercase(),
        has_macro_variation: params.macro_variation,
        splat_mask_count: splat_masks.len() as u32,
    };

    Ok(SplatSetResult {
        layer_outputs,
        splat_masks,
        macro_variation,
        metadata,
    })
}

/// Generate textures for a single layer.
fn generate_layer_textures(
    layer: &SplatLayer,
    width: u32,
    height: u32,
    seed: u32,
    config: &PngConfig,
) -> Result<SplatLayerOutput, SplatSetError> {
    // Generate detail noise for the layer
    let detail_noise = Fbm::new(PerlinNoise::new(seed))
        .with_octaves(4)
        .with_persistence(0.5)
        .with_lacunarity(2.0);

    // Generate albedo texture
    let mut albedo_buffer = TextureBuffer::new(
        width,
        height,
        Color::rgba(
            layer.albedo_color[0],
            layer.albedo_color[1],
            layer.albedo_color[2],
            layer.albedo_color[3],
        ),
    );

    // Apply detail variation to albedo
    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 * layer.detail_scale / width as f64;
            let ny = y as f64 * layer.detail_scale / height as f64;
            let detail = detail_noise.sample_01(nx * 10.0, ny * 10.0);
            let variation = (detail - 0.5) * layer.detail_intensity;

            let base = albedo_buffer.get(x, y);
            albedo_buffer.set(
                x,
                y,
                Color::rgba(
                    (base.r + variation).clamp(0.0, 1.0),
                    (base.g + variation).clamp(0.0, 1.0),
                    (base.b + variation).clamp(0.0, 1.0),
                    base.a,
                ),
            );
        }
    }

    let (albedo_png, albedo_hash) = write_rgba_to_vec_with_hash(&albedo_buffer, config)?;

    // Generate height map for normal generation
    let mut height_buffer = GrayscaleBuffer::new(width, height, 0.5);
    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 * layer.detail_scale / width as f64;
            let ny = y as f64 * layer.detail_scale / height as f64;
            let h = detail_noise.sample_01(nx * 10.0, ny * 10.0);
            height_buffer.set(x, y, h);
        }
    }

    // Generate normal map from height
    let normal_generator = NormalGenerator::new().with_strength(layer.normal_strength);
    let normal_buffer = normal_generator.generate_from_height(&height_buffer);
    let (normal_png, normal_hash) = write_rgba_to_vec_with_hash(&normal_buffer, config)?;

    // Generate roughness texture (base roughness with slight variation)
    let roughness_seed = seed.wrapping_add(0x87654321);
    let roughness_noise = SimplexNoise::new(roughness_seed);
    let mut roughness_buffer = GrayscaleBuffer::new(width, height, layer.roughness);

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 * layer.detail_scale * 2.0 / width as f64;
            let ny = y as f64 * layer.detail_scale * 2.0 / height as f64;
            let variation = (roughness_noise.sample_01(nx * 10.0, ny * 10.0) - 0.5) * 0.1;
            let r = (layer.roughness + variation).clamp(0.0, 1.0);
            roughness_buffer.set(x, y, r);
        }
    }

    let (roughness_png, roughness_hash) =
        write_grayscale_to_vec_with_hash(&roughness_buffer, config)?;

    Ok(SplatLayerOutput {
        id: layer.id.clone(),
        albedo: SplatTextureResult {
            png_data: albedo_png,
            hash: albedo_hash,
        },
        normal: SplatTextureResult {
            png_data: normal_png,
            hash: normal_hash,
        },
        roughness: SplatTextureResult {
            png_data: roughness_png,
            hash: roughness_hash,
        },
    })
}

/// Generate splat mask textures.
fn generate_splat_masks(
    params: &TextureSplatSetV1Params,
    width: u32,
    height: u32,
    seed: u32,
    config: &PngConfig,
) -> Result<Vec<SplatTextureResult>, SplatSetError> {
    let num_layers = params.layers.len();
    let num_masks = (num_layers + 3) / 4; // Ceiling division by 4

    let mut masks = Vec::with_capacity(num_masks);

    for mask_idx in 0..num_masks {
        let start_layer = mask_idx * 4;
        let end_layer = (start_layer + 4).min(num_layers);
        let layers_in_mask = end_layer - start_layer;

        // Generate individual layer weights
        let mut weights: Vec<GrayscaleBuffer> = Vec::with_capacity(layers_in_mask);

        for i in 0..layers_in_mask {
            let layer_idx = start_layer + i;
            let layer_seed = seed.wrapping_add((layer_idx as u32).wrapping_mul(0x9E3779B9));
            let weight = generate_layer_weight(params, layer_idx, width, height, layer_seed);
            weights.push(weight);
        }

        // Normalize weights so they sum to 1.0 at each pixel
        let mut mask_buffer =
            TextureBuffer::new(width, height, Color::rgba(0.0, 0.0, 0.0, 0.0));

        for y in 0..height {
            for x in 0..width {
                let mut total = 0.0;
                let mut channel_values = [0.0f64; 4];

                for (i, weight_buf) in weights.iter().enumerate() {
                    let w = weight_buf.get(x, y);
                    channel_values[i] = w;
                    total += w;
                }

                // Normalize
                if total > 0.0 {
                    for v in channel_values.iter_mut() {
                        *v /= total;
                    }
                } else if !weights.is_empty() {
                    // Default to first layer if all weights are zero
                    channel_values[0] = 1.0;
                }

                mask_buffer.set(
                    x,
                    y,
                    Color::rgba(
                        channel_values[0],
                        channel_values[1],
                        channel_values[2],
                        channel_values[3],
                    ),
                );
            }
        }

        let (png_data, hash) = write_rgba_to_vec_with_hash(&mask_buffer, config)?;
        masks.push(SplatTextureResult { png_data, hash });
    }

    Ok(masks)
}

/// Generate weight map for a single layer.
fn generate_layer_weight(
    params: &TextureSplatSetV1Params,
    layer_idx: usize,
    width: u32,
    height: u32,
    seed: u32,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 0.0);
    let num_layers = params.layers.len();

    match params.mask_mode {
        SplatMaskMode::Noise => {
            // Each layer gets a noise pattern
            let noise = Fbm::new(PerlinNoise::new(seed))
                .with_octaves(4)
                .with_persistence(0.5)
                .with_lacunarity(2.0);

            for y in 0..height {
                for x in 0..width {
                    let nx = x as f64 * params.noise_scale;
                    let ny = y as f64 * params.noise_scale;
                    let val = noise.sample_01(nx, ny);
                    buffer.set(x, y, val);
                }
            }
        }
        SplatMaskMode::Height => {
            // Layers are distributed by vertical position
            // Lower layers at bottom, higher layers at top
            let layer_range = 1.0 / num_layers as f64;
            let layer_center = (layer_idx as f64 + 0.5) * layer_range;

            // Add noise for variation
            let noise = SimplexNoise::new(seed);

            for y in 0..height {
                for x in 0..width {
                    let normalized_y = y as f64 / height as f64;
                    let nx = x as f64 * params.noise_scale;
                    let ny = y as f64 * params.noise_scale;
                    let noise_val = (noise.sample_01(nx, ny) - 0.5) * 0.3;

                    // Distance from layer center (with noise offset)
                    let dist = (normalized_y + noise_val - layer_center).abs();
                    let weight = (1.0 - dist / layer_range).max(0.0);
                    buffer.set(x, y, weight);
                }
            }
        }
        SplatMaskMode::Slope => {
            // Generate a pseudo-slope from noise
            let slope_noise = Fbm::new(PerlinNoise::new(seed))
                .with_octaves(3)
                .with_persistence(0.6)
                .with_lacunarity(2.0);

            let layer_range = 1.0 / num_layers as f64;
            let layer_center = (layer_idx as f64 + 0.5) * layer_range;

            for y in 0..height {
                for x in 0..width {
                    let nx = x as f64 * params.noise_scale;
                    let ny = y as f64 * params.noise_scale;

                    // Compute pseudo-slope from height differences
                    let h = slope_noise.sample_01(nx, ny);
                    let h_dx = slope_noise.sample_01(nx + 0.01, ny);
                    let h_dy = slope_noise.sample_01(nx, ny + 0.01);
                    let slope = ((h_dx - h).powi(2) + (h_dy - h).powi(2)).sqrt() * 10.0;
                    let slope_normalized = slope.min(1.0);

                    // Distance from layer center based on slope
                    let dist = (slope_normalized - layer_center).abs();
                    let weight = (1.0 - dist / layer_range).max(0.0);
                    buffer.set(x, y, weight);
                }
            }
        }
        SplatMaskMode::HeightSlope => {
            // Combine height and slope
            let noise = Fbm::new(PerlinNoise::new(seed))
                .with_octaves(4)
                .with_persistence(0.5)
                .with_lacunarity(2.0);

            let layer_range = 1.0 / num_layers as f64;
            let layer_center = (layer_idx as f64 + 0.5) * layer_range;

            for y in 0..height {
                for x in 0..width {
                    let nx = x as f64 * params.noise_scale;
                    let ny = y as f64 * params.noise_scale;

                    let normalized_y = y as f64 / height as f64;

                    // Compute slope
                    let h = noise.sample_01(nx, ny);
                    let h_dx = noise.sample_01(nx + 0.01, ny);
                    let h_dy = noise.sample_01(nx, ny + 0.01);
                    let slope = ((h_dx - h).powi(2) + (h_dy - h).powi(2)).sqrt() * 10.0;
                    let slope_normalized = slope.min(1.0);

                    // Combine height and slope
                    let combined = normalized_y * 0.5 + slope_normalized * 0.5;

                    // Distance from layer center
                    let dist = (combined - layer_center).abs();
                    let weight = (1.0 - dist / layer_range).max(0.0);
                    buffer.set(x, y, weight);
                }
            }
        }
    }

    buffer
}

/// Generate macro variation texture.
fn generate_macro_variation(
    width: u32,
    height: u32,
    scale: f64,
    intensity: f64,
    seed: u32,
    config: &PngConfig,
) -> Result<SplatTextureResult, SplatSetError> {
    let noise = Fbm::new(PerlinNoise::new(seed))
        .with_octaves(3)
        .with_persistence(0.5)
        .with_lacunarity(2.0);

    let mut buffer = GrayscaleBuffer::new(width, height, 0.5);

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 * scale;
            let ny = y as f64 * scale;
            let val = noise.sample_01(nx, ny);
            // Map to intensity range centered around 0.5
            let mapped = 0.5 + (val - 0.5) * intensity;
            buffer.set(x, y, mapped.clamp(0.0, 1.0));
        }
    }

    let (png_data, hash) = write_grayscale_to_vec_with_hash(&buffer, config)?;
    Ok(SplatTextureResult { png_data, hash })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_layer(id: &str, color: [f64; 4], roughness: f64) -> SplatLayer {
        SplatLayer {
            id: id.to_string(),
            albedo_color: color,
            normal_strength: 1.0,
            roughness,
            detail_scale: 0.2,
            detail_intensity: 0.3,
        }
    }

    #[test]
    fn test_generate_basic_splat_set() {
        let params = TextureSplatSetV1Params {
            resolution: [64, 64],
            layers: vec![
                make_test_layer("grass", [0.2, 0.5, 0.1, 1.0], 0.8),
                make_test_layer("dirt", [0.4, 0.3, 0.2, 1.0], 0.9),
            ],
            mask_mode: SplatMaskMode::Noise,
            noise_scale: 0.1,
            macro_variation: false,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let result = generate_splat_set(&params, 42).unwrap();

        assert_eq!(result.layer_outputs.len(), 2);
        assert_eq!(result.splat_masks.len(), 1); // 2 layers fit in 1 mask
        assert!(result.macro_variation.is_none());

        // Verify layer outputs
        assert_eq!(result.layer_outputs[0].id, "grass");
        assert!(!result.layer_outputs[0].albedo.png_data.is_empty());
        assert!(!result.layer_outputs[0].normal.png_data.is_empty());
        assert!(!result.layer_outputs[0].roughness.png_data.is_empty());

        // Verify metadata
        assert_eq!(result.metadata.resolution, [64, 64]);
        assert_eq!(result.metadata.layers.len(), 2);
        assert_eq!(result.metadata.layers[0].mask_channel, 0);
        assert_eq!(result.metadata.layers[1].mask_channel, 1);
    }

    #[test]
    fn test_generate_splat_set_with_macro() {
        let params = TextureSplatSetV1Params {
            resolution: [32, 32],
            layers: vec![make_test_layer("base", [0.5, 0.5, 0.5, 1.0], 0.5)],
            mask_mode: SplatMaskMode::Noise,
            noise_scale: 0.1,
            macro_variation: true,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let result = generate_splat_set(&params, 42).unwrap();

        assert!(result.macro_variation.is_some());
        assert!(result.metadata.has_macro_variation);
    }

    #[test]
    fn test_generate_splat_set_four_layers() {
        let params = TextureSplatSetV1Params {
            resolution: [32, 32],
            layers: vec![
                make_test_layer("grass", [0.2, 0.5, 0.1, 1.0], 0.8),
                make_test_layer("dirt", [0.4, 0.3, 0.2, 1.0], 0.9),
                make_test_layer("rock", [0.5, 0.5, 0.5, 1.0], 0.7),
                make_test_layer("sand", [0.8, 0.7, 0.5, 1.0], 0.6),
            ],
            mask_mode: SplatMaskMode::Noise,
            noise_scale: 0.1,
            macro_variation: false,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let result = generate_splat_set(&params, 42).unwrap();

        assert_eq!(result.layer_outputs.len(), 4);
        assert_eq!(result.splat_masks.len(), 1); // 4 layers fit in 1 RGBA mask

        // Verify channel assignments
        assert_eq!(result.metadata.layers[0].mask_channel, 0); // R
        assert_eq!(result.metadata.layers[1].mask_channel, 1); // G
        assert_eq!(result.metadata.layers[2].mask_channel, 2); // B
        assert_eq!(result.metadata.layers[3].mask_channel, 3); // A
    }

    #[test]
    fn test_generate_splat_set_height_mode() {
        let params = TextureSplatSetV1Params {
            resolution: [32, 32],
            layers: vec![
                make_test_layer("grass", [0.2, 0.5, 0.1, 1.0], 0.8),
                make_test_layer("dirt", [0.4, 0.3, 0.2, 1.0], 0.9),
            ],
            mask_mode: SplatMaskMode::Height,
            noise_scale: 0.1,
            macro_variation: false,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let result = generate_splat_set(&params, 42).unwrap();

        assert_eq!(result.metadata.mask_mode, "height");
        assert!(!result.splat_masks.is_empty());
    }

    #[test]
    fn test_generate_splat_set_slope_mode() {
        let params = TextureSplatSetV1Params {
            resolution: [32, 32],
            layers: vec![
                make_test_layer("flat", [0.5, 0.5, 0.5, 1.0], 0.5),
                make_test_layer("steep", [0.3, 0.3, 0.3, 1.0], 0.9),
            ],
            mask_mode: SplatMaskMode::Slope,
            noise_scale: 0.1,
            macro_variation: false,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let result = generate_splat_set(&params, 42).unwrap();

        assert_eq!(result.metadata.mask_mode, "slope");
    }

    #[test]
    fn test_generate_splat_set_height_slope_mode() {
        let params = TextureSplatSetV1Params {
            resolution: [32, 32],
            layers: vec![
                make_test_layer("base", [0.5, 0.5, 0.5, 1.0], 0.5),
                make_test_layer("top", [0.3, 0.3, 0.3, 1.0], 0.9),
            ],
            mask_mode: SplatMaskMode::HeightSlope,
            noise_scale: 0.1,
            macro_variation: false,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let result = generate_splat_set(&params, 42).unwrap();

        assert_eq!(result.metadata.mask_mode, "heightslope");
    }

    #[test]
    fn test_generate_splat_set_determinism() {
        let params = TextureSplatSetV1Params {
            resolution: [32, 32],
            layers: vec![
                make_test_layer("grass", [0.2, 0.5, 0.1, 1.0], 0.8),
                make_test_layer("dirt", [0.4, 0.3, 0.2, 1.0], 0.9),
            ],
            mask_mode: SplatMaskMode::Noise,
            noise_scale: 0.1,
            macro_variation: true,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let result1 = generate_splat_set(&params, 42).unwrap();
        let result2 = generate_splat_set(&params, 42).unwrap();

        // Same seed should produce identical results
        assert_eq!(
            result1.layer_outputs[0].albedo.hash,
            result2.layer_outputs[0].albedo.hash
        );
        assert_eq!(
            result1.splat_masks[0].hash,
            result2.splat_masks[0].hash
        );
        assert_eq!(
            result1.macro_variation.as_ref().unwrap().hash,
            result2.macro_variation.as_ref().unwrap().hash
        );
    }

    #[test]
    fn test_generate_splat_set_different_seeds() {
        let params = TextureSplatSetV1Params {
            resolution: [32, 32],
            layers: vec![make_test_layer("grass", [0.2, 0.5, 0.1, 1.0], 0.8)],
            mask_mode: SplatMaskMode::Noise,
            noise_scale: 0.1,
            macro_variation: false,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let result1 = generate_splat_set(&params, 42).unwrap();
        let result2 = generate_splat_set(&params, 100).unwrap();

        // Different seeds should produce different results
        assert_ne!(
            result1.layer_outputs[0].albedo.hash,
            result2.layer_outputs[0].albedo.hash
        );
    }

    #[test]
    fn test_generate_splat_set_invalid_resolution() {
        let params = TextureSplatSetV1Params {
            resolution: [0, 64],
            layers: vec![make_test_layer("base", [0.5, 0.5, 0.5, 1.0], 0.5)],
            mask_mode: SplatMaskMode::Noise,
            noise_scale: 0.1,
            macro_variation: false,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let result = generate_splat_set(&params, 42);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SplatSetError::InvalidParameter(_)));
    }

    #[test]
    fn test_generate_splat_set_no_layers() {
        let params = TextureSplatSetV1Params {
            resolution: [64, 64],
            layers: vec![],
            mask_mode: SplatMaskMode::Noise,
            noise_scale: 0.1,
            macro_variation: false,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let result = generate_splat_set(&params, 42);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SplatSetError::InvalidParameter(_)));
    }

    #[test]
    fn test_metadata_serialization() {
        let params = TextureSplatSetV1Params {
            resolution: [64, 64],
            layers: vec![
                make_test_layer("grass", [0.2, 0.5, 0.1, 1.0], 0.8),
                make_test_layer("dirt", [0.4, 0.3, 0.2, 1.0], 0.9),
            ],
            mask_mode: SplatMaskMode::Noise,
            noise_scale: 0.1,
            macro_variation: true,
            macro_scale: 0.05,
            macro_intensity: 0.3,
        };

        let result = generate_splat_set(&params, 42).unwrap();
        let json = serde_json::to_string_pretty(&result.metadata).unwrap();
        let parsed: SplatSetOutputMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, result.metadata);
    }
}
