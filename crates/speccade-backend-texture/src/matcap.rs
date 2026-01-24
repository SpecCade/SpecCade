//! Matcap texture generation.
//!
//! Generates stylized NPR (non-photorealistic rendering) shading textures.
//! A matcap (material capture) is a 2D sphere texture that encodes lighting and shading,
//! allowing fast stylized rendering by mapping surface normals to colors.

use speccade_spec::recipe::texture::{
    CavityMaskConfig, CurvatureMaskConfig, MatcapPreset, OutlineConfig, TextureMatcapV1Params,
};
use thiserror::Error;

use crate::color::Color;
use crate::maps::TextureBuffer;
use crate::png::{write_rgba_to_vec_with_hash, PngConfig, PngError};
use crate::rng::DeterministicRng;

/// Errors that can occur during matcap generation.
#[derive(Debug, Error)]
pub enum MatcapError {
    /// PNG encoding failed.
    #[error("PNG encoding failed: {0}")]
    PngError(#[from] PngError),

    /// Invalid parameter.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result of matcap generation.
#[derive(Debug)]
pub struct MatcapResult {
    /// PNG-encoded matcap texture.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
}

/// Generate a matcap texture from parameters.
///
/// # Arguments
/// * `params` - Matcap parameters including preset, resolution, and overrides.
/// * `seed` - Deterministic seed for any random operations.
///
/// # Returns
/// A `MatcapResult` containing the PNG-encoded matcap texture.
pub fn generate_matcap(
    params: &TextureMatcapV1Params,
    seed: u32,
) -> Result<MatcapResult, MatcapError> {
    let width = params.resolution[0];
    let height = params.resolution[1];

    // Validate parameters
    if width == 0 || height == 0 {
        return Err(MatcapError::InvalidParameter(
            "Resolution must be non-zero".to_string(),
        ));
    }

    if let Some(steps) = params.toon_steps {
        if !(2..=16).contains(&steps) {
            return Err(MatcapError::InvalidParameter(format!(
                "toon_steps must be between 2 and 16, got {}",
                steps
            )));
        }
    }

    if let Some(ref outline) = params.outline {
        if !(1..=10).contains(&outline.width) {
            return Err(MatcapError::InvalidParameter(format!(
                "outline width must be between 1 and 10, got {}",
                outline.width
            )));
        }
    }

    // Initialize RNG for any random variations
    let mut _rng = DeterministicRng::new(seed);

    // Create buffer
    let mut buffer = TextureBuffer::new(width, height, Color::rgba(0.5, 0.5, 0.5, 1.0));

    // Get base color (from override or preset default)
    let base_color = params
        .base_color
        .unwrap_or_else(|| preset_base_color(params.preset));

    // Render matcap sphere
    render_matcap_sphere(
        &mut buffer,
        params.preset,
        base_color,
        params.toon_steps,
        width,
        height,
    );

    // Apply curvature mask if enabled
    if let Some(ref curvature_config) = params.curvature_mask {
        if curvature_config.enabled {
            apply_curvature_mask(&mut buffer, curvature_config, width, height);
        }
    }

    // Apply cavity mask if enabled
    if let Some(ref cavity_config) = params.cavity_mask {
        if cavity_config.enabled {
            apply_cavity_mask(&mut buffer, cavity_config, width, height);
        }
    }

    // Apply outline if specified
    if let Some(ref outline_config) = params.outline {
        apply_outline(&mut buffer, outline_config, width, height);
    }

    // Encode as PNG
    let config = PngConfig::default();
    let (png_data, hash) = write_rgba_to_vec_with_hash(&buffer, &config)?;

    Ok(MatcapResult { png_data, hash })
}

/// Get the default base color for a preset.
fn preset_base_color(preset: MatcapPreset) -> [f64; 3] {
    match preset {
        MatcapPreset::ToonBasic => [0.8, 0.8, 0.8],
        MatcapPreset::ToonRim => [0.7, 0.7, 0.9],
        MatcapPreset::Metallic => [0.9, 0.9, 0.95],
        MatcapPreset::Ceramic => [0.95, 0.95, 0.98],
        MatcapPreset::Clay => [0.8, 0.6, 0.5],
        MatcapPreset::Skin => [0.95, 0.8, 0.7],
        MatcapPreset::Plastic => [0.85, 0.85, 0.9],
        MatcapPreset::Velvet => [0.6, 0.4, 0.5],
    }
}

/// Render the matcap sphere with lighting based on the preset.
///
/// Maps each pixel to a normalized direction on a sphere, then calculates
/// lighting based on the preset's shading model.
fn render_matcap_sphere(
    buffer: &mut TextureBuffer,
    preset: MatcapPreset,
    base_color: [f64; 3],
    toon_steps: Option<u32>,
    width: u32,
    height: u32,
) {
    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;
    let radius = (width.min(height) as f64 / 2.0) * 0.95; // Slightly smaller than half to fit in bounds

    for y in 0..height {
        for x in 0..width {
            // Map pixel to [-1, 1] normalized sphere coordinates
            let nx = (x as f64 - center_x) / radius;
            let ny = (y as f64 - center_y) / radius;
            let dist_sq = nx * nx + ny * ny;

            if dist_sq <= 1.0 {
                // Point is on the sphere
                let nz = (1.0 - dist_sq).sqrt();
                let normal = [nx, ny, nz];

                // Calculate lighting based on preset
                let color = calculate_lighting(preset, base_color, normal, toon_steps);
                buffer.set(x, y, color);
            } else {
                // Outside sphere - transparent or background
                buffer.set(x, y, Color::rgba(0.0, 0.0, 0.0, 0.0));
            }
        }
    }
}

/// Calculate lighting color for a given surface normal based on the preset.
fn calculate_lighting(
    preset: MatcapPreset,
    base_color: [f64; 3],
    normal: [f64; 3],
    toon_steps: Option<u32>,
) -> Color {
    // Standard light direction (top-left, slightly forward)
    let light_dir = normalize([0.3, 0.4, 0.8]);

    // Standard view direction (straight on)
    let view_dir = [0.0, 0.0, 1.0];

    let n_dot_l = dot(normal, light_dir).max(0.0);
    let n_dot_v = dot(normal, view_dir).max(0.0);

    let mut diffuse = n_dot_l;
    let mut specular = 0.0;

    match preset {
        MatcapPreset::ToonBasic => {
            // Simple toon with clear light/shadow
            diffuse = if diffuse > 0.5 { 1.0 } else { 0.3 };
        }
        MatcapPreset::ToonRim => {
            // Toon with rim lighting
            diffuse = if diffuse > 0.5 { 1.0 } else { 0.3 };
            let rim = 1.0 - n_dot_v;
            let rim_strength = if rim > 0.7 { 0.4 } else { 0.0 };
            diffuse += rim_strength;
        }
        MatcapPreset::Metallic => {
            // Strong specular highlights
            let half_vec = normalize(add(light_dir, view_dir));
            let n_dot_h = dot(normal, half_vec).max(0.0);
            specular = n_dot_h.powf(64.0);
            diffuse *= 0.3; // Reduce diffuse for metallic look
        }
        MatcapPreset::Ceramic => {
            // Soft diffuse with subtle specular
            let half_vec = normalize(add(light_dir, view_dir));
            let n_dot_h = dot(normal, half_vec).max(0.0);
            specular = n_dot_h.powf(16.0) * 0.3;
        }
        MatcapPreset::Clay => {
            // Pure diffuse, no specular
            // diffuse is already calculated
        }
        MatcapPreset::Skin => {
            // Subsurface-like soft falloff
            diffuse = diffuse * 0.7 + 0.3; // Soften shadows
        }
        MatcapPreset::Plastic => {
            // Sharp highlights
            let half_vec = normalize(add(light_dir, view_dir));
            let n_dot_h = dot(normal, half_vec).max(0.0);
            specular = n_dot_h.powf(128.0) * 0.8;
        }
        MatcapPreset::Velvet => {
            // Anisotropic-like rim
            let rim = 1.0 - n_dot_v;
            diffuse = diffuse * 0.6 + rim * 0.4;
        }
    }

    // Apply toon steps if specified
    if let Some(steps) = toon_steps {
        let steps_f = steps as f64;
        diffuse = (diffuse * steps_f).floor() / (steps_f - 1.0).max(1.0);
        specular = (specular * steps_f).floor() / (steps_f - 1.0).max(1.0);
    }

    // Combine lighting
    let total_light = (diffuse + specular).min(1.0);

    Color::rgba(
        (base_color[0] * total_light).min(1.0),
        (base_color[1] * total_light).min(1.0),
        (base_color[2] * total_light).min(1.0),
        1.0,
    )
}

/// Apply curvature masking (highlights edges/ridges based on normal variation).
fn apply_curvature_mask(
    buffer: &mut TextureBuffer,
    config: &CurvatureMaskConfig,
    width: u32,
    height: u32,
) {
    let strength = config.strength.clamp(0.0, 1.0);
    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;
    let radius = (width.min(height) as f64 / 2.0) * 0.95;

    // Simple edge detection: approximate curvature by distance from center
    for y in 0..height {
        for x in 0..width {
            let nx = (x as f64 - center_x) / radius;
            let ny = (y as f64 - center_y) / radius;
            let dist_sq = nx * nx + ny * ny;

            if dist_sq <= 1.0 {
                // Approximate curvature: higher near edges
                let curvature = dist_sq.sqrt(); // 0 at center, 1 at edge
                let highlight = curvature * strength;

                let color = buffer.get(x, y);
                buffer.set(
                    x,
                    y,
                    Color::rgba(
                        (color.r + highlight).min(1.0),
                        (color.g + highlight).min(1.0),
                        (color.b + highlight).min(1.0),
                        color.a,
                    ),
                );
            }
        }
    }
}

/// Apply cavity masking (darkens concave areas).
fn apply_cavity_mask(
    buffer: &mut TextureBuffer,
    config: &CavityMaskConfig,
    width: u32,
    height: u32,
) {
    let strength = config.strength.clamp(0.0, 1.0);
    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;
    let radius = (width.min(height) as f64 / 2.0) * 0.95;

    // Simple cavity: darken areas away from edges (inverse of curvature)
    for y in 0..height {
        for x in 0..width {
            let nx = (x as f64 - center_x) / radius;
            let ny = (y as f64 - center_y) / radius;
            let dist_sq = nx * nx + ny * ny;

            if dist_sq <= 1.0 {
                // Cavity is higher near center (concave approximation)
                let cavity = (1.0 - dist_sq.sqrt()) * 0.5; // 0.5 at center, 0 at edge
                let darken = cavity * strength;

                let color = buffer.get(x, y);
                buffer.set(
                    x,
                    y,
                    Color::rgba(
                        (color.r - darken).max(0.0),
                        (color.g - darken).max(0.0),
                        (color.b - darken).max(0.0),
                        color.a,
                    ),
                );
            }
        }
    }
}

/// Apply outline by detecting sphere edges.
fn apply_outline(buffer: &mut TextureBuffer, config: &OutlineConfig, width: u32, height: u32) {
    let outline_color = Color::rgba(config.color[0], config.color[1], config.color[2], 1.0);
    let outline_width = config.width;

    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;
    let radius = (width.min(height) as f64 / 2.0) * 0.95;

    for y in 0..height {
        for x in 0..width {
            let nx = (x as f64 - center_x) / radius;
            let ny = (y as f64 - center_y) / radius;
            let dist_sq = nx * nx + ny * ny;

            // Check if we're near the edge of the sphere
            if dist_sq > 1.0 {
                // Check if any neighbors are inside the sphere (edge detection)
                let mut is_edge = false;
                for dy in -(outline_width as i32)..=(outline_width as i32) {
                    for dx in -(outline_width as i32)..=(outline_width as i32) {
                        let nx_check = x as i32 + dx;
                        let ny_check = y as i32 + dy;

                        if nx_check >= 0
                            && nx_check < width as i32
                            && ny_check >= 0
                            && ny_check < height as i32
                        {
                            let check_x = nx_check as u32;
                            let check_y = ny_check as u32;
                            let check_nx = (check_x as f64 - center_x) / radius;
                            let check_ny = (check_y as f64 - center_y) / radius;
                            let check_dist_sq = check_nx * check_nx + check_ny * check_ny;

                            if check_dist_sq <= 1.0 {
                                is_edge = true;
                                break;
                            }
                        }
                    }
                    if is_edge {
                        break;
                    }
                }

                if is_edge {
                    buffer.set(x, y, outline_color);
                }
            } else {
                // Inside sphere - check if we're near the edge
                let edge_threshold = 1.0 - (outline_width as f64 / radius).min(0.2);
                if dist_sq > edge_threshold * edge_threshold {
                    buffer.set(x, y, outline_color);
                }
            }
        }
    }
}

/// Vector dot product.
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Vector addition.
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Normalize a vector.
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_basic_matcap() {
        let params = TextureMatcapV1Params {
            resolution: [256, 256],
            preset: MatcapPreset::ToonBasic,
            base_color: None,
            toon_steps: None,
            outline: None,
            curvature_mask: None,
            cavity_mask: None,
        };

        let result = generate_matcap(&params, 42).unwrap();
        assert!(!result.png_data.is_empty());
        assert!(!result.hash.is_empty());
    }

    #[test]
    fn test_generate_matcap_with_toon_steps() {
        let params = TextureMatcapV1Params {
            resolution: [128, 128],
            preset: MatcapPreset::ToonBasic,
            base_color: Some([0.8, 0.2, 0.2]),
            toon_steps: Some(4),
            outline: None,
            curvature_mask: None,
            cavity_mask: None,
        };

        let result = generate_matcap(&params, 42).unwrap();
        assert!(!result.png_data.is_empty());
    }

    #[test]
    fn test_generate_matcap_with_outline() {
        let params = TextureMatcapV1Params {
            resolution: [128, 128],
            preset: MatcapPreset::Clay,
            base_color: None,
            toon_steps: None,
            outline: Some(OutlineConfig {
                width: 2,
                color: [0.0, 0.0, 0.0],
            }),
            curvature_mask: None,
            cavity_mask: None,
        };

        let result = generate_matcap(&params, 42).unwrap();
        assert!(!result.png_data.is_empty());
    }

    #[test]
    fn test_generate_matcap_with_masks() {
        let params = TextureMatcapV1Params {
            resolution: [128, 128],
            preset: MatcapPreset::Metallic,
            base_color: None,
            toon_steps: None,
            outline: None,
            curvature_mask: Some(CurvatureMaskConfig {
                enabled: true,
                strength: 0.6,
            }),
            cavity_mask: Some(CavityMaskConfig {
                enabled: true,
                strength: 0.4,
            }),
        };

        let result = generate_matcap(&params, 42).unwrap();
        assert!(!result.png_data.is_empty());
    }

    #[test]
    fn test_generate_matcap_determinism() {
        let params = TextureMatcapV1Params {
            resolution: [64, 64],
            preset: MatcapPreset::Plastic,
            base_color: Some([0.9, 0.9, 0.9]),
            toon_steps: Some(3),
            outline: Some(OutlineConfig {
                width: 1,
                color: [0.1, 0.1, 0.1],
            }),
            curvature_mask: None,
            cavity_mask: None,
        };

        let result1 = generate_matcap(&params, 42).unwrap();
        let result2 = generate_matcap(&params, 42).unwrap();

        assert_eq!(result1.png_data, result2.png_data);
        assert_eq!(result1.hash, result2.hash);
    }

    #[test]
    fn test_generate_matcap_zero_resolution() {
        let params = TextureMatcapV1Params {
            resolution: [0, 256],
            preset: MatcapPreset::Clay,
            base_color: None,
            toon_steps: None,
            outline: None,
            curvature_mask: None,
            cavity_mask: None,
        };

        let result = generate_matcap(&params, 42);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MatcapError::InvalidParameter(_)
        ));
    }

    #[test]
    fn test_generate_matcap_invalid_toon_steps() {
        let params = TextureMatcapV1Params {
            resolution: [128, 128],
            preset: MatcapPreset::ToonBasic,
            base_color: None,
            toon_steps: Some(20), // Too high
            outline: None,
            curvature_mask: None,
            cavity_mask: None,
        };

        let result = generate_matcap(&params, 42);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_matcap_invalid_outline_width() {
        let params = TextureMatcapV1Params {
            resolution: [128, 128],
            preset: MatcapPreset::Clay,
            base_color: None,
            toon_steps: None,
            outline: Some(OutlineConfig {
                width: 15, // Too high
                color: [0.0, 0.0, 0.0],
            }),
            curvature_mask: None,
            cavity_mask: None,
        };

        let result = generate_matcap(&params, 42);
        assert!(result.is_err());
    }

    #[test]
    fn test_preset_base_colors() {
        // Just ensure all presets have valid base colors
        let presets = [
            MatcapPreset::ToonBasic,
            MatcapPreset::ToonRim,
            MatcapPreset::Metallic,
            MatcapPreset::Ceramic,
            MatcapPreset::Clay,
            MatcapPreset::Skin,
            MatcapPreset::Plastic,
            MatcapPreset::Velvet,
        ];

        for preset in presets {
            let color = preset_base_color(preset);
            assert!(color[0] >= 0.0 && color[0] <= 1.0);
            assert!(color[1] >= 0.0 && color[1] <= 1.0);
            assert!(color[2] >= 0.0 && color[2] <= 1.0);
        }
    }
}
