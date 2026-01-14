//! Metallic map generation.
//!
//! Generates metallic maps with pattern-based variations for metallic regions.

use speccade_spec::recipe::texture::{GradientDirection, StripeDirection, TextureLayer, TextureMapType};

use crate::maps::{GrayscaleBuffer, MetallicGenerator};
use crate::pattern::{CheckerPattern, GradientPattern, Pattern2D, StripesPattern};
use crate::png::{self, PngConfig};
use crate::rng::DeterministicRng;

use super::helpers::create_noise_generator;
use super::{GenerateError, MapResult};

/// Generate metallic map.
pub fn generate_metallic_map(
    _height_map: &GrayscaleBuffer,
    layers: &[TextureLayer],
    metallic: f64,
    width: u32,
    height: u32,
    seed: u32,
) -> Result<MapResult, GenerateError> {
    let generator = MetallicGenerator::new(metallic, seed);
    let mut buffer = generator.generate_with_variation(width, height);

    for (i, layer) in layers.iter().enumerate() {
        let layer_seed = DeterministicRng::derive_layer_seed(seed, i as u32);

        match layer {
            TextureLayer::NoisePattern {
                noise,
                affects,
                strength,
            } => {
                if !affects.contains(&TextureMapType::Metallic) || *strength <= 0.0 {
                    continue;
                }

                let noise_gen = create_noise_generator(noise, layer_seed);
                let scale = noise.scale;
                let strength = strength.clamp(0.0, 1.0);

                for y in 0..height {
                    for x in 0..width {
                        let nx = x as f64 * scale;
                        let ny = y as f64 * scale;
                        let noise_val = noise_gen.sample_01(nx, ny);
                        let current = buffer.get(x, y);
                        let blended = current * (1.0 - strength) + noise_val * strength;
                        buffer.set(x, y, blended.clamp(0.0, 1.0));
                    }
                }
            }
            TextureLayer::Gradient {
                direction,
                start,
                end,
                center,
                inner,
                outer,
                affects,
                strength,
            } => {
                if !affects.contains(&TextureMapType::Metallic) || *strength <= 0.0 {
                    continue;
                }

                let gradient = match direction {
                    GradientDirection::Horizontal => {
                        let s = start.unwrap_or(0.0);
                        let e = end.unwrap_or(1.0);
                        GradientPattern::new_horizontal(width, height, s, e)
                    }
                    GradientDirection::Vertical => {
                        let s = start.unwrap_or(0.0);
                        let e = end.unwrap_or(1.0);
                        GradientPattern::new_vertical(width, height, s, e)
                    }
                    GradientDirection::Radial => {
                        let c = center.unwrap_or([0.5, 0.5]);
                        let i = inner.unwrap_or(1.0);
                        let o = outer.unwrap_or(0.0);
                        GradientPattern::new_radial(width, height, c, i, o)
                    }
                };

                let strength = strength.clamp(0.0, 1.0);

                for y in 0..height {
                    for x in 0..width {
                        let pattern_val = gradient.sample(x, y);
                        let current = buffer.get(x, y);
                        let blended = current * (1.0 - strength) + pattern_val * strength;
                        buffer.set(x, y, blended.clamp(0.0, 1.0));
                    }
                }
            }
            TextureLayer::Stripes {
                direction,
                stripe_width,
                color1,
                color2,
                affects,
                strength,
            } => {
                if !affects.contains(&TextureMapType::Metallic) || *strength <= 0.0 {
                    continue;
                }

                let stripes = match direction {
                    StripeDirection::Horizontal => StripesPattern::new_horizontal(
                        *stripe_width,
                        color1.clamp(0.0, 1.0),
                        color2.clamp(0.0, 1.0),
                    ),
                    StripeDirection::Vertical => StripesPattern::new_vertical(
                        *stripe_width,
                        color1.clamp(0.0, 1.0),
                        color2.clamp(0.0, 1.0),
                    ),
                };

                let strength = strength.clamp(0.0, 1.0);

                for y in 0..height {
                    for x in 0..width {
                        let pattern_val = stripes.sample(x, y);
                        let current = buffer.get(x, y);
                        let blended = current * (1.0 - strength) + pattern_val * strength;
                        buffer.set(x, y, blended.clamp(0.0, 1.0));
                    }
                }
            }
            TextureLayer::Checkerboard {
                tile_size,
                color1,
                color2,
                affects,
                strength,
            } => {
                if !affects.contains(&TextureMapType::Metallic) || *strength <= 0.0 {
                    continue;
                }

                let checker = CheckerPattern::new(*tile_size)
                    .with_colors(color1.clamp(0.0, 1.0), color2.clamp(0.0, 1.0));
                let strength = strength.clamp(0.0, 1.0);

                for y in 0..height {
                    for x in 0..width {
                        let pattern_val = checker.sample(x, y);
                        let current = buffer.get(x, y);
                        let blended = current * (1.0 - strength) + pattern_val * strength;
                        buffer.set(x, y, blended.clamp(0.0, 1.0));
                    }
                }
            }
            _ => {}
        }
    }

    let config = PngConfig::default();
    let (data, hash) = png::write_grayscale_to_vec_with_hash(&buffer, &config)?;

    Ok(MapResult {
        map_type: TextureMapType::Metallic,
        data,
        width,
        height,
        hash,
        is_color: false,
    })
}
