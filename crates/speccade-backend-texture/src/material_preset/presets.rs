//! Material preset generator functions.
//!
//! Each function generates PBR texture data (albedo, roughness, metallic, height)
//! for a specific material style.

use crate::color::Color;
use crate::maps::{GrayscaleBuffer, TextureBuffer};
use crate::noise::{Fbm, Noise2D, PerlinNoise, SimplexNoise, WorleyNoise};
use crate::rng::DeterministicRng;

use super::tileable_coord;

/// Generate ToonMetal preset: flat albedo with rim highlights, stepped roughness.
#[allow(clippy::too_many_arguments)]
pub fn generate_toon_metal(
    width: u32,
    height: u32,
    base_color: [f64; 3],
    roughness_range: [f64; 2],
    metallic_value: f64,
    noise_scale: f64,
    _pattern_scale: f64,
    tileable: bool,
    rng: &mut DeterministicRng,
) -> (
    TextureBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
) {
    let mut albedo = TextureBuffer::new(
        width,
        height,
        Color::rgba(base_color[0], base_color[1], base_color[2], 1.0),
    );
    let mut roughness = GrayscaleBuffer::new(width, height, roughness_range[0]);
    let mut metallic = GrayscaleBuffer::new(width, height, metallic_value);
    let mut height_buf = GrayscaleBuffer::new(width, height, 0.5);

    let noise = PerlinNoise::new(rng.gen_u32());
    let steps = 4;

    for y in 0..height {
        for x in 0..width {
            let (nx, ny) = if tileable {
                tileable_coord(x, y, width, height, noise_scale)
            } else {
                (x as f64 * noise_scale, y as f64 * noise_scale)
            };

            let n = (noise.sample(nx, ny) + 1.0) * 0.5;

            // Stepped/quantized effect for toon look
            let stepped = (n * steps as f64).floor() / (steps - 1) as f64;

            // Add rim-like variation based on position
            let rim = (((x as f64 / width as f64) - 0.5).powi(2)
                + ((y as f64 / height as f64) - 0.5).powi(2))
            .sqrt();
            let rim_highlight = (1.0 - rim * 1.5).max(0.0);

            let brightness = 0.9 + stepped * 0.1 + rim_highlight * 0.1;
            albedo.set(
                x,
                y,
                Color::rgba(
                    (base_color[0] * brightness).min(1.0),
                    (base_color[1] * brightness).min(1.0),
                    (base_color[2] * brightness).min(1.0),
                    1.0,
                ),
            );

            // Stepped roughness
            let r = roughness_range[0] + stepped * (roughness_range[1] - roughness_range[0]);
            roughness.set(x, y, r);

            // Slight variation in metallic
            metallic.set(x, y, metallic_value * (0.95 + n * 0.05));

            // Height for normal map
            height_buf.set(x, y, stepped * 0.5 + 0.5);
        }
    }

    (albedo, roughness, metallic, height_buf)
}

/// Generate StylizedWood preset: wood grain pattern with warm tones, organic noise.
#[allow(clippy::too_many_arguments)]
pub fn generate_stylized_wood(
    width: u32,
    height: u32,
    base_color: [f64; 3],
    roughness_range: [f64; 2],
    metallic_value: f64,
    noise_scale: f64,
    pattern_scale: f64,
    tileable: bool,
    rng: &mut DeterministicRng,
) -> (
    TextureBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
) {
    let mut albedo = TextureBuffer::new(
        width,
        height,
        Color::rgba(base_color[0], base_color[1], base_color[2], 1.0),
    );
    let mut roughness = GrayscaleBuffer::new(width, height, roughness_range[0]);
    let mut metallic = GrayscaleBuffer::new(width, height, metallic_value);
    let mut height_buf = GrayscaleBuffer::new(width, height, 0.5);

    let grain_noise = SimplexNoise::new(rng.gen_u32());
    let detail_noise = PerlinNoise::new(rng.gen_u32());

    for y in 0..height {
        for x in 0..width {
            let (nx, ny) = if tileable {
                tileable_coord(x, y, width, height, noise_scale)
            } else {
                (x as f64 * noise_scale, y as f64 * noise_scale)
            };

            // Wood grain using distorted sine waves
            let distortion = grain_noise.sample(nx * 0.5, ny * 0.5) * 2.0;
            let grain_y = y as f64 * pattern_scale + distortion * 10.0;
            let grain = (grain_y.sin() * 0.5 + 0.5).powi(2);

            // Add fine detail
            let detail = (detail_noise.sample(nx * 3.0, ny * 3.0) + 1.0) * 0.5;

            let combined = grain * 0.7 + detail * 0.3;

            // Warm color variations
            let dark_color = [
                base_color[0] * 0.6,
                base_color[1] * 0.5,
                base_color[2] * 0.4,
            ];
            let r = dark_color[0] + (base_color[0] - dark_color[0]) * combined;
            let g = dark_color[1] + (base_color[1] - dark_color[1]) * combined;
            let b = dark_color[2] + (base_color[2] - dark_color[2]) * combined;

            albedo.set(x, y, Color::rgba(r, g, b, 1.0));

            // Roughness varies with grain
            let rough = roughness_range[0] + combined * (roughness_range[1] - roughness_range[0]);
            roughness.set(x, y, rough);

            metallic.set(x, y, metallic_value);
            height_buf.set(x, y, combined * 0.3 + 0.35);
        }
    }

    (albedo, roughness, metallic, height_buf)
}

/// Generate NeonGlow preset: dark base with bright emissive-style highlights.
#[allow(clippy::too_many_arguments)]
pub fn generate_neon_glow(
    width: u32,
    height: u32,
    base_color: [f64; 3],
    roughness_range: [f64; 2],
    metallic_value: f64,
    noise_scale: f64,
    pattern_scale: f64,
    tileable: bool,
    rng: &mut DeterministicRng,
) -> (
    TextureBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
) {
    let mut albedo = TextureBuffer::new(
        width,
        height,
        Color::rgba(base_color[0], base_color[1], base_color[2], 1.0),
    );
    let mut roughness = GrayscaleBuffer::new(width, height, roughness_range[0]);
    let mut metallic = GrayscaleBuffer::new(width, height, metallic_value);
    let mut height_buf = GrayscaleBuffer::new(width, height, 0.5);

    let glow_noise = Fbm::new(SimplexNoise::new(rng.gen_u32()))
        .with_octaves(3)
        .with_persistence(0.5)
        .with_lacunarity(2.0);
    let pattern_noise = WorleyNoise::new(rng.gen_u32());

    for y in 0..height {
        for x in 0..width {
            let (nx, ny) = if tileable {
                tileable_coord(x, y, width, height, noise_scale)
            } else {
                (x as f64 * noise_scale, y as f64 * noise_scale)
            };

            let glow = (glow_noise.sample(nx, ny) + 1.0) * 0.5;
            let pattern =
                pattern_noise.sample(nx * pattern_scale * 10.0, ny * pattern_scale * 10.0);

            // Create glow lines/edges
            let edge = (1.0 - pattern).powf(4.0);
            let combined = glow * 0.3 + edge * 0.7;

            // Neon glow colors (cyan/magenta)
            let glow_color = if (x + y) % 2 == 0 {
                [0.0, 0.9, 1.0] // Cyan
            } else {
                [1.0, 0.0, 0.8] // Magenta
            };

            let r = base_color[0] + glow_color[0] * combined * 0.8;
            let g = base_color[1] + glow_color[1] * combined * 0.8;
            let b = base_color[2] + glow_color[2] * combined * 0.8;

            albedo.set(x, y, Color::rgba(r.min(1.0), g.min(1.0), b.min(1.0), 1.0));

            // Low roughness for glow areas
            let rough = roughness_range[1] - combined * (roughness_range[1] - roughness_range[0]);
            roughness.set(x, y, rough);

            metallic.set(x, y, metallic_value);
            height_buf.set(x, y, combined * 0.2 + 0.4);
        }
    }

    (albedo, roughness, metallic, height_buf)
}

/// Generate CeramicGlaze preset: smooth, high-gloss look.
#[allow(clippy::too_many_arguments)]
pub fn generate_ceramic_glaze(
    width: u32,
    height: u32,
    base_color: [f64; 3],
    roughness_range: [f64; 2],
    metallic_value: f64,
    noise_scale: f64,
    _pattern_scale: f64,
    tileable: bool,
    rng: &mut DeterministicRng,
) -> (
    TextureBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
) {
    let mut albedo = TextureBuffer::new(
        width,
        height,
        Color::rgba(base_color[0], base_color[1], base_color[2], 1.0),
    );
    let mut roughness = GrayscaleBuffer::new(width, height, roughness_range[0]);
    let mut metallic = GrayscaleBuffer::new(width, height, metallic_value);
    let mut height_buf = GrayscaleBuffer::new(width, height, 0.5);

    let noise = PerlinNoise::new(rng.gen_u32());

    for y in 0..height {
        for x in 0..width {
            let (nx, ny) = if tileable {
                tileable_coord(x, y, width, height, noise_scale)
            } else {
                (x as f64 * noise_scale, y as f64 * noise_scale)
            };

            // Very subtle variation for ceramic
            let n = (noise.sample(nx, ny) + 1.0) * 0.5;
            let subtle = n * 0.05;

            albedo.set(
                x,
                y,
                Color::rgba(
                    (base_color[0] + subtle).min(1.0),
                    (base_color[1] + subtle).min(1.0),
                    (base_color[2] + subtle).min(1.0),
                    1.0,
                ),
            );

            // Very low, consistent roughness for glaze
            let rough = roughness_range[0] + n * (roughness_range[1] - roughness_range[0]) * 0.3;
            roughness.set(x, y, rough);

            metallic.set(x, y, metallic_value);
            height_buf.set(x, y, 0.5 + subtle);
        }
    }

    (albedo, roughness, metallic, height_buf)
}

/// Generate SciFiPanel preset: geometric patterns, metallic with panel lines.
#[allow(clippy::too_many_arguments)]
pub fn generate_scifi_panel(
    width: u32,
    height: u32,
    base_color: [f64; 3],
    roughness_range: [f64; 2],
    metallic_value: f64,
    noise_scale: f64,
    pattern_scale: f64,
    _tileable: bool,
    rng: &mut DeterministicRng,
) -> (
    TextureBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
) {
    let mut albedo = TextureBuffer::new(
        width,
        height,
        Color::rgba(base_color[0], base_color[1], base_color[2], 1.0),
    );
    let mut roughness = GrayscaleBuffer::new(width, height, roughness_range[0]);
    let mut metallic = GrayscaleBuffer::new(width, height, metallic_value);
    let mut height_buf = GrayscaleBuffer::new(width, height, 0.5);

    let detail_noise = PerlinNoise::new(rng.gen_u32());
    let panel_size = (width as f64 * pattern_scale * 0.5).max(16.0) as u32;

    for y in 0..height {
        for x in 0..width {
            // Panel grid
            let panel_x = x % panel_size;
            let panel_y = y % panel_size;
            let edge_x = panel_x < 2 || panel_x >= panel_size - 2;
            let edge_y = panel_y < 2 || panel_y >= panel_size - 2;
            let is_edge = edge_x || edge_y;

            // Detail noise
            let nx = x as f64 * noise_scale;
            let ny = y as f64 * noise_scale;
            let detail = (detail_noise.sample(nx, ny) + 1.0) * 0.5;

            if is_edge {
                // Dark panel lines/grooves
                albedo.set(
                    x,
                    y,
                    Color::rgba(
                        base_color[0] * 0.3,
                        base_color[1] * 0.3,
                        base_color[2] * 0.3,
                        1.0,
                    ),
                );
                roughness.set(x, y, roughness_range[1]);
                height_buf.set(x, y, 0.2);
            } else {
                // Panel surface with subtle variation
                let brightness = 0.9 + detail * 0.2;
                albedo.set(
                    x,
                    y,
                    Color::rgba(
                        (base_color[0] * brightness).min(1.0),
                        (base_color[1] * brightness).min(1.0),
                        (base_color[2] * brightness).min(1.0),
                        1.0,
                    ),
                );
                let rough =
                    roughness_range[0] + detail * (roughness_range[1] - roughness_range[0]) * 0.5;
                roughness.set(x, y, rough);
                height_buf.set(x, y, 0.6 + detail * 0.1);
            }

            metallic.set(x, y, metallic_value);
        }
    }

    (albedo, roughness, metallic, height_buf)
}

/// Generate CleanPlastic preset: uniform albedo with medium roughness.
#[allow(clippy::too_many_arguments)]
pub fn generate_clean_plastic(
    width: u32,
    height: u32,
    base_color: [f64; 3],
    roughness_range: [f64; 2],
    metallic_value: f64,
    noise_scale: f64,
    _pattern_scale: f64,
    tileable: bool,
    rng: &mut DeterministicRng,
) -> (
    TextureBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
) {
    let mut albedo = TextureBuffer::new(
        width,
        height,
        Color::rgba(base_color[0], base_color[1], base_color[2], 1.0),
    );
    let mut roughness = GrayscaleBuffer::new(
        width,
        height,
        (roughness_range[0] + roughness_range[1]) * 0.5,
    );
    let mut metallic = GrayscaleBuffer::new(width, height, metallic_value);
    let mut height_buf = GrayscaleBuffer::new(width, height, 0.5);

    let noise = PerlinNoise::new(rng.gen_u32());

    for y in 0..height {
        for x in 0..width {
            let (nx, ny) = if tileable {
                tileable_coord(x, y, width, height, noise_scale)
            } else {
                (x as f64 * noise_scale, y as f64 * noise_scale)
            };

            // Very subtle micro-detail
            let n = (noise.sample(nx * 5.0, ny * 5.0) + 1.0) * 0.5;
            let subtle = n * 0.02;

            albedo.set(
                x,
                y,
                Color::rgba(
                    (base_color[0] + subtle).min(1.0),
                    (base_color[1] + subtle).min(1.0),
                    (base_color[2] + subtle).min(1.0),
                    1.0,
                ),
            );

            // Consistent medium roughness with micro variation
            let rough = roughness_range[0]
                + n * (roughness_range[1] - roughness_range[0]) * 0.2
                + (roughness_range[1] - roughness_range[0]) * 0.4;
            roughness.set(x, y, rough);

            metallic.set(x, y, metallic_value);
            height_buf.set(x, y, 0.5 + subtle);
        }
    }

    (albedo, roughness, metallic, height_buf)
}

/// Generate RoughStone preset: rocky noise patterns, high roughness.
#[allow(clippy::too_many_arguments)]
pub fn generate_rough_stone(
    width: u32,
    height: u32,
    base_color: [f64; 3],
    roughness_range: [f64; 2],
    metallic_value: f64,
    noise_scale: f64,
    pattern_scale: f64,
    tileable: bool,
    rng: &mut DeterministicRng,
) -> (
    TextureBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
) {
    let mut albedo = TextureBuffer::new(
        width,
        height,
        Color::rgba(base_color[0], base_color[1], base_color[2], 1.0),
    );
    let mut roughness = GrayscaleBuffer::new(width, height, roughness_range[1]);
    let mut metallic = GrayscaleBuffer::new(width, height, metallic_value);
    let mut height_buf = GrayscaleBuffer::new(width, height, 0.5);

    let large_noise = Fbm::new(PerlinNoise::new(rng.gen_u32()))
        .with_octaves(4)
        .with_persistence(0.5)
        .with_lacunarity(2.0);
    let detail_noise = PerlinNoise::new(rng.gen_u32());
    let crack_noise = WorleyNoise::new(rng.gen_u32());

    for y in 0..height {
        for x in 0..width {
            let (nx, ny) = if tileable {
                tileable_coord(x, y, width, height, noise_scale)
            } else {
                (x as f64 * noise_scale, y as f64 * noise_scale)
            };

            // Large-scale rock variation
            let large = (large_noise.sample(nx * pattern_scale, ny * pattern_scale) + 1.0) * 0.5;
            // Fine detail
            let detail = (detail_noise.sample(nx * 3.0, ny * 3.0) + 1.0) * 0.5;
            // Cracks
            let crack = crack_noise.sample(nx * 2.0, ny * 2.0);

            let combined = large * 0.6 + detail * 0.3 + (1.0 - crack) * 0.1;

            // Color variation
            let color_var = combined * 0.3;
            let r = (base_color[0] * (0.7 + color_var)).min(1.0);
            let g = (base_color[1] * (0.7 + color_var)).min(1.0);
            let b = (base_color[2] * (0.7 + color_var)).min(1.0);

            albedo.set(x, y, Color::rgba(r, g, b, 1.0));

            // High roughness with some variation
            let rough = roughness_range[0]
                + combined * (roughness_range[1] - roughness_range[0]) * 0.5
                + (roughness_range[1] - roughness_range[0]) * 0.5;
            roughness.set(x, y, rough.min(1.0));

            metallic.set(x, y, metallic_value);
            height_buf.set(x, y, combined * 0.4 + 0.3);
        }
    }

    (albedo, roughness, metallic, height_buf)
}

/// Generate BrushedMetal preset: directional anisotropic streaks.
#[allow(clippy::too_many_arguments)]
pub fn generate_brushed_metal(
    width: u32,
    height: u32,
    base_color: [f64; 3],
    roughness_range: [f64; 2],
    metallic_value: f64,
    noise_scale: f64,
    _pattern_scale: f64,
    tileable: bool,
    rng: &mut DeterministicRng,
) -> (
    TextureBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
    GrayscaleBuffer,
) {
    let mut albedo = TextureBuffer::new(
        width,
        height,
        Color::rgba(base_color[0], base_color[1], base_color[2], 1.0),
    );
    let mut roughness = GrayscaleBuffer::new(width, height, roughness_range[0]);
    let mut metallic = GrayscaleBuffer::new(width, height, metallic_value);
    let mut height_buf = GrayscaleBuffer::new(width, height, 0.5);

    let streak_noise = PerlinNoise::new(rng.gen_u32());
    let micro_noise = PerlinNoise::new(rng.gen_u32());

    for y in 0..height {
        for x in 0..width {
            let (nx, ny) = if tileable {
                tileable_coord(x, y, width, height, noise_scale)
            } else {
                (x as f64 * noise_scale, y as f64 * noise_scale)
            };

            // Directional brushing (horizontal streaks)
            let streak = (streak_noise.sample(nx * 0.5, ny * 10.0) + 1.0) * 0.5;
            // Micro detail
            let micro = (micro_noise.sample(nx * 20.0, ny * 2.0) + 1.0) * 0.5;

            let combined = streak * 0.7 + micro * 0.3;

            // Subtle color variation along streaks
            let brightness = 0.85 + combined * 0.15;
            albedo.set(
                x,
                y,
                Color::rgba(
                    (base_color[0] * brightness).min(1.0),
                    (base_color[1] * brightness).min(1.0),
                    (base_color[2] * brightness).min(1.0),
                    1.0,
                ),
            );

            // Roughness varies with streak direction
            let rough = roughness_range[0] + combined * (roughness_range[1] - roughness_range[0]);
            roughness.set(x, y, rough);

            metallic.set(x, y, metallic_value);
            // Anisotropic height pattern
            height_buf.set(x, y, 0.4 + streak * 0.2);
        }
    }

    (albedo, roughness, metallic, height_buf)
}
