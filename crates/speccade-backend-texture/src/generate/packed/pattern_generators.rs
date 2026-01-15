//! Pattern generation functions for packed textures.

use crate::maps::GrayscaleBuffer;
use crate::noise::{tile_coord, Fbm, Noise2D, PerlinNoise, SimplexNoise, WorleyNoise};

use super::parsers::Axis;

const DEFAULT_NOISE_PERSISTENCE: f64 = 0.5;
const DEFAULT_NOISE_LACUNARITY: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseKind {
    Perlin,
    Simplex,
    Worley,
    Fbm,
}

pub fn build_noise(kind: NoiseKind, seed: u32, octaves: u8) -> Box<dyn Noise2D> {
    match kind {
        NoiseKind::Perlin => Box::new(PerlinNoise::new(seed)),
        NoiseKind::Simplex => Box::new(SimplexNoise::new(seed)),
        NoiseKind::Worley => Box::new(WorleyNoise::new(seed)),
        NoiseKind::Fbm => Box::new(
            Fbm::new(PerlinNoise::new(seed))
                .with_octaves(octaves)
                .with_persistence(DEFAULT_NOISE_PERSISTENCE)
                .with_lacunarity(DEFAULT_NOISE_LACUNARITY),
        ),
    }
}

pub fn generate_noise_map(
    width: u32,
    height: u32,
    tileable: bool,
    scale: f64,
    noise: &dyn Noise2D,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 0.0);

    let period_x = (width as f64) * scale;
    let period_y = (height as f64) * scale;

    for y in 0..height {
        for x in 0..width {
            let mut nx = x as f64 * scale;
            let mut ny = y as f64 * scale;

            if tileable && period_x > 0.0 && period_y > 0.0 {
                nx = tile_coord(nx, period_x);
                ny = tile_coord(ny, period_y);
            }

            let value = noise.sample_01(nx, ny);
            buffer.set(x, y, value);
        }
    }

    buffer
}

#[inline]
fn fract01(value: f64) -> f64 {
    value.rem_euclid(1.0)
}

pub fn generate_stripes_map(
    width: u32,
    height: u32,
    axis: Axis,
    frequency: u32,
    duty_cycle: f64,
    phase: f64,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 0.0);
    let freq = frequency as f64;

    for y in 0..height {
        for x in 0..width {
            let coord = match axis {
                Axis::X => x as f64 / width as f64,
                Axis::Y => y as f64 / height as f64,
            };
            let t = coord * freq + phase;
            let stripe = fract01(t) < duty_cycle;
            buffer.set(x, y, if stripe { 1.0 } else { 0.0 });
        }
    }

    buffer
}

pub fn generate_grid_map(
    width: u32,
    height: u32,
    cells: [u32; 2],
    line_width: f64,
    phase: f64,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 0.0);

    let cells_x = cells[0] as f64;
    let cells_y = cells[1] as f64;

    for y in 0..height {
        for x in 0..width {
            let u = x as f64 / width as f64;
            let v = y as f64 / height as f64;
            let fx = fract01(u * cells_x + phase);
            let fy = fract01(v * cells_y + phase);

            let is_line = fx < line_width
                || fx > 1.0 - line_width
                || fy < line_width
                || fy > 1.0 - line_width;
            buffer.set(x, y, if is_line { 1.0 } else { 0.0 });
        }
    }

    buffer
}

pub fn generate_gradient_map(
    width: u32,
    height: u32,
    axis: Axis,
    start: f64,
    end: f64,
    phase: f64,
    tileable: bool,
) -> GrayscaleBuffer {
    let mut buffer = GrayscaleBuffer::new(width, height, 0.0);

    for y in 0..height {
        for x in 0..width {
            let coord = match axis {
                Axis::X => x as f64 / width as f64,
                Axis::Y => y as f64 / height as f64,
            };

            let t = if tileable {
                let wrapped = fract01(coord + phase);
                1.0 - (wrapped * 2.0 - 1.0).abs()
            } else {
                (coord + phase).clamp(0.0, 1.0)
            };

            let value = start + (end - start) * t;
            buffer.set(x, y, value.clamp(0.0, 1.0));
        }
    }

    buffer
}
