//! Material preset texture generation.
//!
//! Generates multiple PBR texture outputs (albedo, roughness, metallic, normal)
//! from predefined material style presets with optional parameter overrides.

mod presets;

use speccade_spec::recipe::texture::{
    MaterialPresetOutputMetadata, MaterialPresetType, TextureMaterialPresetV1Params,
};
use thiserror::Error;

use crate::color::Color;
use crate::maps::{GrayscaleBuffer, TextureBuffer};
use crate::png::{
    write_grayscale_to_vec_with_hash, write_rgba_to_vec_with_hash, PngConfig, PngError,
};
use crate::rng::DeterministicRng;

pub use presets::*;

/// Errors that can occur during material preset generation.
#[derive(Debug, Error)]
pub enum MaterialPresetError {
    /// PNG encoding failed.
    #[error("PNG encoding failed: {0}")]
    PngError(#[from] PngError),

    /// Invalid parameter.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result of a single texture output (PNG).
#[derive(Debug)]
pub struct MaterialPresetTextureResult {
    /// PNG-encoded image data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
}

/// Result of material preset generation.
#[derive(Debug)]
pub struct MaterialPresetResult {
    /// Albedo texture (RGB).
    pub albedo: MaterialPresetTextureResult,
    /// Roughness texture (grayscale).
    pub roughness: MaterialPresetTextureResult,
    /// Metallic texture (grayscale).
    pub metallic: MaterialPresetTextureResult,
    /// Normal map texture (RGB).
    pub normal: MaterialPresetTextureResult,
    /// Metadata for the generated material.
    pub metadata: MaterialPresetOutputMetadata,
}

/// Generate material preset textures from parameters.
///
/// # Arguments
/// * `params` - Material preset parameters including preset type and overrides.
/// * `seed` - Deterministic seed for procedural generation.
///
/// # Returns
/// A `MaterialPresetResult` containing PNG outputs for albedo, roughness, metallic, and normal.
pub fn generate_material_preset(
    params: &TextureMaterialPresetV1Params,
    seed: u32,
) -> Result<MaterialPresetResult, MaterialPresetError> {
    let width = params.resolution[0];
    let height = params.resolution[1];

    // Validate parameters
    if width == 0 || height == 0 {
        return Err(MaterialPresetError::InvalidParameter(
            "Resolution must be non-zero".to_string(),
        ));
    }

    // Resolve final values (overrides or defaults)
    let base_color = params
        .base_color
        .unwrap_or_else(|| params.preset.default_base_color());
    let roughness_range = params
        .roughness_range
        .unwrap_or_else(|| params.preset.default_roughness_range());
    let metallic_value = params
        .metallic
        .unwrap_or_else(|| params.preset.default_metallic());
    let noise_scale = params
        .noise_scale
        .unwrap_or_else(|| params.preset.default_noise_scale());
    let pattern_scale = params
        .pattern_scale
        .unwrap_or_else(|| params.preset.default_pattern_scale());

    // Initialize deterministic RNG
    let mut rng = DeterministicRng::new(seed);

    // Generate textures based on preset
    let (albedo_buf, roughness_buf, metallic_buf, height_buf) = match params.preset {
        MaterialPresetType::ToonMetal => generate_toon_metal(
            width,
            height,
            base_color,
            roughness_range,
            metallic_value,
            noise_scale,
            pattern_scale,
            params.tileable,
            &mut rng,
        ),
        MaterialPresetType::StylizedWood => generate_stylized_wood(
            width,
            height,
            base_color,
            roughness_range,
            metallic_value,
            noise_scale,
            pattern_scale,
            params.tileable,
            &mut rng,
        ),
        MaterialPresetType::NeonGlow => generate_neon_glow(
            width,
            height,
            base_color,
            roughness_range,
            metallic_value,
            noise_scale,
            pattern_scale,
            params.tileable,
            &mut rng,
        ),
        MaterialPresetType::CeramicGlaze => generate_ceramic_glaze(
            width,
            height,
            base_color,
            roughness_range,
            metallic_value,
            noise_scale,
            pattern_scale,
            params.tileable,
            &mut rng,
        ),
        MaterialPresetType::SciFiPanel => generate_scifi_panel(
            width,
            height,
            base_color,
            roughness_range,
            metallic_value,
            noise_scale,
            pattern_scale,
            params.tileable,
            &mut rng,
        ),
        MaterialPresetType::CleanPlastic => generate_clean_plastic(
            width,
            height,
            base_color,
            roughness_range,
            metallic_value,
            noise_scale,
            pattern_scale,
            params.tileable,
            &mut rng,
        ),
        MaterialPresetType::RoughStone => generate_rough_stone(
            width,
            height,
            base_color,
            roughness_range,
            metallic_value,
            noise_scale,
            pattern_scale,
            params.tileable,
            &mut rng,
        ),
        MaterialPresetType::BrushedMetal => generate_brushed_metal(
            width,
            height,
            base_color,
            roughness_range,
            metallic_value,
            noise_scale,
            pattern_scale,
            params.tileable,
            &mut rng,
        ),
    };

    // Generate normal map from height buffer
    let normal_buf = generate_normal_from_height(&height_buf, width, height);

    // Encode as PNG
    let config = PngConfig::default();
    let (albedo_png, albedo_hash) = write_rgba_to_vec_with_hash(&albedo_buf, &config)?;
    let (roughness_png, roughness_hash) =
        write_grayscale_to_vec_with_hash(&roughness_buf, &config)?;
    let (metallic_png, metallic_hash) = write_grayscale_to_vec_with_hash(&metallic_buf, &config)?;
    let (normal_png, normal_hash) = write_rgba_to_vec_with_hash(&normal_buf, &config)?;

    // Build metadata
    let preset_name = match params.preset {
        MaterialPresetType::ToonMetal => "toon_metal",
        MaterialPresetType::StylizedWood => "stylized_wood",
        MaterialPresetType::NeonGlow => "neon_glow",
        MaterialPresetType::CeramicGlaze => "ceramic_glaze",
        MaterialPresetType::SciFiPanel => "sci_fi_panel",
        MaterialPresetType::CleanPlastic => "clean_plastic",
        MaterialPresetType::RoughStone => "rough_stone",
        MaterialPresetType::BrushedMetal => "brushed_metal",
    };

    let metadata = MaterialPresetOutputMetadata {
        resolution: params.resolution,
        tileable: params.tileable,
        preset: preset_name.to_string(),
        base_color,
        roughness_range,
        metallic: metallic_value,
        generated_maps: vec![
            "albedo".to_string(),
            "roughness".to_string(),
            "metallic".to_string(),
            "normal".to_string(),
        ],
    };

    Ok(MaterialPresetResult {
        albedo: MaterialPresetTextureResult {
            png_data: albedo_png,
            hash: albedo_hash,
        },
        roughness: MaterialPresetTextureResult {
            png_data: roughness_png,
            hash: roughness_hash,
        },
        metallic: MaterialPresetTextureResult {
            png_data: metallic_png,
            hash: metallic_hash,
        },
        normal: MaterialPresetTextureResult {
            png_data: normal_png,
            hash: normal_hash,
        },
        metadata,
    })
}

/// Generate normal map from height field.
pub(crate) fn generate_normal_from_height(
    height: &GrayscaleBuffer,
    width: u32,
    height_dim: u32,
) -> TextureBuffer {
    let mut normal_buf = TextureBuffer::new(width, height_dim, Color::rgba(0.5, 0.5, 1.0, 1.0));
    let strength = 1.0;

    for y in 0..height_dim {
        for x in 0..width {
            // Sample neighbors for gradient (with wrapping for tileability)
            let left = height.get((x + width - 1) % width, y);
            let right = height.get((x + 1) % width, y);
            let up = height.get(x, (y + height_dim - 1) % height_dim);
            let down = height.get(x, (y + 1) % height_dim);

            // Compute gradient
            let dx = (right - left) * strength;
            let dy = (down - up) * strength;

            // Normal vector
            let nx = -dx;
            let ny = -dy;
            let nz = 1.0;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();

            // Normalize and map to [0, 1] range
            let r = (nx / len * 0.5 + 0.5).clamp(0.0, 1.0);
            let g = (ny / len * 0.5 + 0.5).clamp(0.0, 1.0);
            let b = (nz / len * 0.5 + 0.5).clamp(0.0, 1.0);

            normal_buf.set(x, y, Color::rgba(r, g, b, 1.0));
        }
    }

    normal_buf
}

/// Helper to create tileable noise coordinates.
pub(crate) fn tileable_coord(x: u32, y: u32, width: u32, height: u32, scale: f64) -> (f64, f64) {
    let fx = x as f64 / width as f64;
    let fy = y as f64 / height as f64;
    (fx * scale * width as f64, fy * scale * height as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_params(preset: MaterialPresetType) -> TextureMaterialPresetV1Params {
        TextureMaterialPresetV1Params {
            preset,
            resolution: [64, 64],
            tileable: true,
            base_color: None,
            roughness_range: None,
            metallic: None,
            noise_scale: None,
            pattern_scale: None,
        }
    }

    #[test]
    fn test_generate_toon_metal() {
        let params = make_test_params(MaterialPresetType::ToonMetal);
        let result = generate_material_preset(&params, 42).unwrap();

        assert!(!result.albedo.png_data.is_empty());
        assert!(!result.roughness.png_data.is_empty());
        assert!(!result.metallic.png_data.is_empty());
        assert!(!result.normal.png_data.is_empty());
        assert_eq!(result.metadata.preset, "toon_metal");
        assert_eq!(result.metadata.generated_maps.len(), 4);
    }

    #[test]
    fn test_generate_all_presets() {
        let presets = vec![
            MaterialPresetType::ToonMetal,
            MaterialPresetType::StylizedWood,
            MaterialPresetType::NeonGlow,
            MaterialPresetType::CeramicGlaze,
            MaterialPresetType::SciFiPanel,
            MaterialPresetType::CleanPlastic,
            MaterialPresetType::RoughStone,
            MaterialPresetType::BrushedMetal,
        ];

        for preset in presets {
            let params = make_test_params(preset);
            let result = generate_material_preset(&params, 42).unwrap();
            assert!(!result.albedo.png_data.is_empty());
            assert!(!result.albedo.hash.is_empty());
        }
    }

    #[test]
    fn test_generate_with_overrides() {
        let params = TextureMaterialPresetV1Params {
            preset: MaterialPresetType::ToonMetal,
            resolution: [32, 32],
            tileable: false,
            base_color: Some([1.0, 0.0, 0.0]),
            roughness_range: Some([0.1, 0.3]),
            metallic: Some(1.0),
            noise_scale: Some(0.05),
            pattern_scale: Some(0.2),
        };

        let result = generate_material_preset(&params, 42).unwrap();
        assert_eq!(result.metadata.base_color, [1.0, 0.0, 0.0]);
        assert_eq!(result.metadata.roughness_range, [0.1, 0.3]);
        assert_eq!(result.metadata.metallic, 1.0);
        assert!(!result.metadata.tileable);
    }

    #[test]
    fn test_determinism() {
        let params = make_test_params(MaterialPresetType::StylizedWood);

        let result1 = generate_material_preset(&params, 42).unwrap();
        let result2 = generate_material_preset(&params, 42).unwrap();

        assert_eq!(result1.albedo.png_data, result2.albedo.png_data);
        assert_eq!(result1.roughness.png_data, result2.roughness.png_data);
        assert_eq!(result1.metallic.png_data, result2.metallic.png_data);
        assert_eq!(result1.normal.png_data, result2.normal.png_data);
        assert_eq!(result1.albedo.hash, result2.albedo.hash);
    }

    #[test]
    fn test_different_seeds_produce_different_output() {
        let params = make_test_params(MaterialPresetType::RoughStone);

        let result1 = generate_material_preset(&params, 42).unwrap();
        let result2 = generate_material_preset(&params, 123).unwrap();

        assert_ne!(result1.albedo.hash, result2.albedo.hash);
    }

    #[test]
    fn test_zero_resolution_error() {
        let params = TextureMaterialPresetV1Params {
            preset: MaterialPresetType::CleanPlastic,
            resolution: [0, 64],
            tileable: true,
            base_color: None,
            roughness_range: None,
            metallic: None,
            noise_scale: None,
            pattern_scale: None,
        };

        let result = generate_material_preset(&params, 42);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MaterialPresetError::InvalidParameter(_)
        ));
    }
}
