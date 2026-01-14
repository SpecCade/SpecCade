//! Albedo (base color) map generation.
//!
//! Generates the base color map for materials, including pattern-based coloring,
//! noise variation, color adjustments from layers, and palette/ramp effects.

use speccade_spec::recipe::texture::{TextureLayer, TextureMapType};

use crate::color::{BlendMode, Color};
use crate::maps::{AlbedoGenerator, GrayscaleBuffer};
use crate::noise::{Fbm, Noise2D, PerlinNoise};
use crate::png::{self, PngConfig};
use crate::rng::DeterministicRng;

use super::color_utils::{apply_color_ramp, apply_palette_quantization, parse_hex_color_list};
use super::masks::{build_streak_mask, build_threshold_mask};
use super::{GenerateError, MapResult};

/// Generate albedo map.
///
/// The height_map is used to ensure albedo follows the same pattern as other maps
/// (e.g., differentiating brick from mortar in a brick material).
#[allow(clippy::too_many_arguments)]
pub fn generate_albedo_map(
    base_color: &Color,
    height_map: &GrayscaleBuffer,
    layers: &[TextureLayer],
    palette: Option<&[String]>,
    color_ramp: Option<&[String]>,
    width: u32,
    height: u32,
    seed: u32,
    _tileable: bool,
) -> Result<MapResult, GenerateError> {
    let generator = AlbedoGenerator::new(*base_color, seed).with_variation(0.1);

    // Start with pattern-based coloring using the height map
    // Higher values in height map = raised surface (brick) = base color
    // Lower values = recessed areas (mortar) = darker/different color
    let mortar_color = Color::rgb(
        (base_color.r * 0.4).clamp(0.0, 1.0),
        (base_color.g * 0.4).clamp(0.0, 1.0),
        (base_color.b * 0.4).clamp(0.0, 1.0),
    );

    // Generate base albedo following the height map pattern
    let mut buffer = crate::maps::TextureBuffer::new(width, height, *base_color);

    // Use height map to blend between brick and mortar colors
    // Height values typically range from ~0.2 (mortar) to ~0.9-1.0 (brick)
    // We'll use a threshold to create sharp transitions
    let mortar_threshold = 0.5;

    for y in 0..height {
        for x in 0..width {
            let h = height_map.get(x, y);

            // Calculate blend factor: 0 = mortar, 1 = brick
            let blend = if h < mortar_threshold {
                // In mortar region
                0.0
            } else if h < mortar_threshold + 0.1 {
                // Transition zone
                (h - mortar_threshold) / 0.1
            } else {
                // In brick region
                1.0
            };

            // Lerp between mortar and base color
            let color = mortar_color.lerp(base_color, blend);
            buffer.set(x, y, color);
        }
    }

    // Add noise-based variation on top (follows pattern via height map influence)
    let noise = Fbm::new(PerlinNoise::new(seed))
        .with_octaves(4)
        .with_persistence(0.5);

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 * 0.02;
            let ny = y as f64 * 0.02;

            let current = buffer.get(x, y);
            let (h_val, s, v) = current.to_hsv();

            // Apply subtle HSV variation
            let h_noise = noise.sample(nx, ny) * 10.0; // Subtle hue variation
            let v_noise = noise.sample(nx + 100.0, ny + 100.0) * 0.1;

            let new_h = (h_val + h_noise) % 360.0;
            let new_v = (v + v_noise).clamp(0.0, 1.0);

            let new_color = Color::from_hsv(new_h, s, new_v);
            buffer.set(
                x,
                y,
                Color::rgba(new_color.r, new_color.g, new_color.b, current.a),
            );
        }
    }

    // Apply color variation layers
    for (i, layer) in layers.iter().enumerate() {
        let layer_seed = DeterministicRng::derive_layer_seed(seed, i as u32);

        match layer {
            TextureLayer::ColorVariation {
                hue_range,
                saturation_range,
                value_range,
                noise_scale,
            } => {
                let noise = Fbm::new(PerlinNoise::new(layer_seed)).with_octaves(3);

                for y in 0..height {
                    for x in 0..width {
                        let nx = x as f64 * noise_scale;
                        let ny = y as f64 * noise_scale;

                        let h_noise = noise.sample(nx, ny) * hue_range;
                        let s_noise = noise.sample(nx + 100.0, ny) * saturation_range;
                        let v_noise = noise.sample(nx, ny + 100.0) * value_range;

                        let current = buffer.get(x, y);
                        let (h, s, v) = current.to_hsv();

                        let new_h = (h + h_noise) % 360.0;
                        let new_s = (s + s_noise).clamp(0.0, 1.0);
                        let new_v = (v + v_noise).clamp(0.0, 1.0);

                        let new_color = Color::from_hsv(new_h, new_s, new_v);
                        buffer.set(
                            x,
                            y,
                            Color::rgba(new_color.r, new_color.g, new_color.b, current.a),
                        );
                    }
                }
            }
            TextureLayer::Dirt { density, color, .. } => {
                let dirt_color = Color::rgb(color[0], color[1], color[2]);
                generator.apply_dirt(&mut buffer, *density, dirt_color, layer_seed);
            }
            TextureLayer::Stains {
                noise,
                threshold,
                color,
                affects,
                strength,
            } => {
                if affects.contains(&TextureMapType::Albedo) {
                    let mask = build_threshold_mask(
                        width, height, noise, layer_seed, *threshold, *strength,
                    );
                    let stain_color = Color::rgb(color[0], color[1], color[2]);
                    generator.apply_pattern_color(
                        &mut buffer,
                        &mask,
                        stain_color,
                        BlendMode::Multiply,
                    );
                }
            }
            TextureLayer::WaterStreaks {
                noise,
                threshold,
                direction,
                color,
                affects,
                strength,
            } => {
                if affects.contains(&TextureMapType::Albedo) {
                    let mask = build_streak_mask(
                        width, height, noise, layer_seed, *threshold, *strength, *direction,
                    );
                    let streak_color = Color::rgb(color[0], color[1], color[2]);
                    generator.apply_pattern_color(
                        &mut buffer,
                        &mask,
                        streak_color,
                        BlendMode::Multiply,
                    );
                }
            }
            TextureLayer::Checkerboard {
                tile_size,
                color1,
                color2,
                affects,
                strength,
            } => {
                if affects.contains(&TextureMapType::Albedo) {
                    use crate::pattern::{CheckerPattern, Pattern2D};
                    let checker = CheckerPattern::new(*tile_size).with_colors(*color1, *color2);

                    for y in 0..height {
                        for x in 0..width {
                            let pattern_val = checker.sample(x, y);
                            let current = buffer.get(x, y);

                            // Lerp from current color to grayscale pattern value
                            let new_val = current.r * (1.0 - strength) + pattern_val * strength;
                            buffer.set(x, y, Color::rgba(new_val, new_val, new_val, current.a));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(color_ramp) = color_ramp {
        let ramp = parse_hex_color_list(color_ramp, "color_ramp")?;
        apply_color_ramp(&mut buffer, &ramp);
    }

    if let Some(palette) = palette {
        let palette = parse_hex_color_list(palette, "palette")?;
        apply_palette_quantization(&mut buffer, &palette);
    }

    let config = PngConfig::default();
    let (data, hash) = png::write_rgba_to_vec_with_hash(&buffer, &config)?;

    Ok(MapResult {
        map_type: TextureMapType::Albedo,
        data,
        width,
        height,
        hash,
        is_color: true,
    })
}
