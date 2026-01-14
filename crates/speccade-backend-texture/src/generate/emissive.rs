//! Emissive map generation.
//!
//! Generates emissive (glow) maps from various layer types.

use speccade_spec::recipe::texture::{GradientDirection, StripeDirection, TextureLayer, TextureMapType};

use crate::color::Color;
use crate::maps::TextureBuffer;
use crate::pattern::{CheckerPattern, GradientPattern, Pattern2D, StripesPattern};
use crate::png::{self, PngConfig};
use crate::rng::DeterministicRng;

use super::color_utils::add_emissive_from_mask;
use super::helpers::create_noise_generator;
use super::masks::{build_streak_mask, build_threshold_mask};
use super::{GenerateError, MapResult};

/// Generate emissive map.
pub fn generate_emissive_map(
    layers: &[TextureLayer],
    width: u32,
    height: u32,
    seed: u32,
) -> Result<MapResult, GenerateError> {
    // Default: no emission.
    let mut buffer = TextureBuffer::new(width, height, Color::black());

    for (i, layer) in layers.iter().enumerate() {
        let layer_seed = DeterministicRng::derive_layer_seed(seed, i as u32);

        match layer {
            TextureLayer::Stains {
                noise,
                threshold,
                color,
                affects,
                strength,
            } => {
                if !affects.contains(&TextureMapType::Emissive) {
                    continue;
                }

                let mask =
                    build_threshold_mask(width, height, noise, layer_seed, *threshold, *strength);
                let emit_color = Color::rgb(color[0], color[1], color[2]);
                add_emissive_from_mask(&mut buffer, &mask, emit_color);
            }
            TextureLayer::WaterStreaks {
                noise,
                threshold,
                direction,
                color,
                affects,
                strength,
            } => {
                if !affects.contains(&TextureMapType::Emissive) {
                    continue;
                }

                let mask = build_streak_mask(
                    width, height, noise, layer_seed, *threshold, *strength, *direction,
                );
                let emit_color = Color::rgb(color[0], color[1], color[2]);
                add_emissive_from_mask(&mut buffer, &mask, emit_color);
            }
            TextureLayer::NoisePattern {
                noise,
                affects,
                strength,
            } => {
                if !affects.contains(&TextureMapType::Emissive) || *strength <= 0.0 {
                    continue;
                }

                let noise_gen = create_noise_generator(noise, layer_seed);
                let scale = noise.scale;
                let strength = strength.clamp(0.0, 1.0);

                for y in 0..height {
                    for x in 0..width {
                        let nx = x as f64 * scale;
                        let ny = y as f64 * scale;
                        let t = noise_gen.sample_01(nx, ny) * strength;
                        if t <= 0.0 {
                            continue;
                        }
                        let current = buffer.get(x, y);
                        let added = Color::white().scale(t);
                        buffer.set(x, y, current.add(&added).clamp());
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
                if !affects.contains(&TextureMapType::Emissive) || *strength <= 0.0 {
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
                        let t = gradient.sample(x, y) * strength;
                        if t <= 0.0 {
                            continue;
                        }
                        let current = buffer.get(x, y);
                        let added = Color::white().scale(t);
                        buffer.set(x, y, current.add(&added).clamp());
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
                if !affects.contains(&TextureMapType::Emissive) || *strength <= 0.0 {
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
                        let t = stripes.sample(x, y) * strength;
                        if t <= 0.0 {
                            continue;
                        }
                        let current = buffer.get(x, y);
                        let added = Color::white().scale(t);
                        buffer.set(x, y, current.add(&added).clamp());
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
                if !affects.contains(&TextureMapType::Emissive) || *strength <= 0.0 {
                    continue;
                }

                let checker = CheckerPattern::new(*tile_size)
                    .with_colors(color1.clamp(0.0, 1.0), color2.clamp(0.0, 1.0));
                let strength = strength.clamp(0.0, 1.0);

                for y in 0..height {
                    for x in 0..width {
                        let t = checker.sample(x, y) * strength;
                        if t <= 0.0 {
                            continue;
                        }
                        let current = buffer.get(x, y);
                        let added = Color::white().scale(t);
                        buffer.set(x, y, current.add(&added).clamp());
                    }
                }
            }
            _ => {}
        }
    }

    let config = PngConfig::default();
    let (data, hash) = png::write_rgb_to_vec_with_hash(&buffer, &config)?;

    Ok(MapResult {
        map_type: TextureMapType::Emissive,
        data,
        width,
        height,
        hash,
        is_color: true,
    })
}
