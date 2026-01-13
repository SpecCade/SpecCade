//! Legacy packed texture map generation for `texture.packed_v1`.

use std::collections::HashMap;

use speccade_spec::recipe::texture::{MapDefinition, TexturePackedV1Params};

use crate::maps::{AoGenerator, GrayscaleBuffer, TextureBuffer};
use crate::noise::{
    tile_coord, DistanceFunction, Fbm, Noise2D, PerlinNoise, SimplexNoise, WorleyNoise,
    WorleyReturn,
};
use crate::rng::DeterministicRng;

use super::helpers::{validate_resolution, validate_unit_interval};
use super::GenerateError;

const DEFAULT_NOISE_SCALE: f64 = 0.05;
const DEFAULT_FBM_OCTAVES: u8 = 4;
const DEFAULT_NOISE_PERSISTENCE: f64 = 0.5;
const DEFAULT_NOISE_LACUNARITY: f64 = 2.0;

/// Generate all packed texture maps for legacy `texture.packed_v1`.
pub fn generate_packed_maps(
    params: &TexturePackedV1Params,
    seed: u32,
) -> Result<HashMap<String, TextureBuffer>, GenerateError> {
    let [width, height] = params.resolution;
    validate_resolution(width, height)?;

    let needs_height = params.maps.values().any(|def| {
        matches!(
            def,
            MapDefinition::Grayscale {
                from_height: Some(true),
                ..
            }
        )
    });

    if needs_height && !params.maps.contains_key("height") {
        return Err(GenerateError::InvalidParameter(
            "packed maps use from_height but no 'height' map is defined".to_string(),
        ));
    }

    let height_map = if let Some(def) = params.maps.get("height") {
        Some(generate_height_map(def, width, height, seed, params.tileable)?)
    } else {
        None
    };

    let mut buffers = HashMap::with_capacity(params.maps.len());

    for (key, def) in &params.maps {
        let buffer = generate_map_buffer(
            key,
            def,
            width,
            height,
            seed,
            params.tileable,
            height_map.as_ref(),
        )?;
        buffers.insert(key.clone(), buffer);
    }

    Ok(buffers)
}

fn generate_height_map(
    def: &MapDefinition,
    width: u32,
    height: u32,
    seed: u32,
    tileable: bool,
) -> Result<GrayscaleBuffer, GenerateError> {
    let grayscale = generate_grayscale_map("height", def, width, height, seed, tileable, None)?;
    Ok(grayscale)
}

fn generate_map_buffer(
    key: &str,
    def: &MapDefinition,
    width: u32,
    height: u32,
    seed: u32,
    tileable: bool,
    height_map: Option<&GrayscaleBuffer>,
) -> Result<TextureBuffer, GenerateError> {
    let grayscale = generate_grayscale_map(key, def, width, height, seed, tileable, height_map)?;
    Ok(grayscale.to_texture_buffer())
}

fn generate_grayscale_map(
    key: &str,
    def: &MapDefinition,
    width: u32,
    height: u32,
    seed: u32,
    tileable: bool,
    height_map: Option<&GrayscaleBuffer>,
) -> Result<GrayscaleBuffer, GenerateError> {
    match def {
        MapDefinition::Grayscale {
            value,
            from_height,
            ao_strength,
        } => {
            let uses_from_height = from_height.unwrap_or(false);

            if uses_from_height && value.is_some() {
                return Err(GenerateError::InvalidParameter(format!(
                    "map '{}' cannot set both value and from_height",
                    key
                )));
            }

            if !uses_from_height && ao_strength.is_some() {
                return Err(GenerateError::InvalidParameter(format!(
                    "map '{}' sets ao_strength without from_height",
                    key
                )));
            }

            if uses_from_height {
                let height_map = height_map.ok_or_else(|| {
                    GenerateError::InvalidParameter(format!(
                        "map '{}' requires a shared 'height' map (from_height = true)",
                        key
                    ))
                })?;

                if let Some(strength) = ao_strength {
                    validate_unit_interval(&format!("maps.{}.ao_strength", key), *strength)?;
                    let strength = strength.clamp(0.0, 1.0);
                    let generator = AoGenerator::new().with_strength(strength);
                    return Ok(generator.generate_from_height(height_map));
                }

                return Ok(height_map.clone());
            }

            if let Some(v) = value {
                validate_unit_interval(&format!("maps.{}.value", key), *v)?;
                let v = v.clamp(0.0, 1.0);
                return Ok(GrayscaleBuffer::new(width, height, v));
            }

            Ok(GrayscaleBuffer::new(width, height, 0.5))
        }
        MapDefinition::Pattern {
            pattern,
            noise_type,
            octaves,
            axis,
            frequency,
            duty_cycle,
            phase,
            cells,
            line_width,
            start,
            end,
            jitter,
            distance_fn,
        } => {
            match pattern.as_str() {
                "noise" => {
                    if axis.is_some()
                        || frequency.is_some()
                        || duty_cycle.is_some()
                        || phase.is_some()
                        || cells.is_some()
                        || line_width.is_some()
                        || start.is_some()
                        || end.is_some()
                        || jitter.is_some()
                        || distance_fn.is_some()
                    {
                        return Err(GenerateError::InvalidParameter(format!(
                            "map '{}' pattern 'noise' does not accept non-noise parameters",
                            key
                        )));
                    }

                    let noise_kind = parse_noise_type(noise_type.as_deref())?;
                    let octaves = match noise_kind {
                        NoiseKind::Fbm => match octaves {
                            Some(0) => {
                                return Err(GenerateError::InvalidParameter(format!(
                                    "map '{}' has invalid octaves=0",
                                    key
                                )))
                            }
                            Some(value) => u8::try_from(*value).map_err(|_| {
                                GenerateError::InvalidParameter(format!(
                                    "map '{}' has too many octaves ({})",
                                    key, value
                                ))
                            })?,
                            None => DEFAULT_FBM_OCTAVES,
                        },
                        _ => {
                            if octaves.is_some() {
                                return Err(GenerateError::InvalidParameter(format!(
                                    "map '{}' sets octaves for non-fbm noise",
                                    key
                                )));
                            }
                            DEFAULT_FBM_OCTAVES
                        }
                    };

                    let map_seed = DeterministicRng::derive_variant_seed(seed, key);
                    let noise = build_noise(noise_kind, map_seed, octaves);
                    Ok(generate_noise_map(
                        width,
                        height,
                        tileable,
                        DEFAULT_NOISE_SCALE,
                        noise.as_ref(),
                    ))
                }
                "worley_edges" => {
                    if noise_type.is_some()
                        || octaves.is_some()
                        || axis.is_some()
                        || frequency.is_some()
                        || duty_cycle.is_some()
                        || phase.is_some()
                        || cells.is_some()
                        || line_width.is_some()
                        || start.is_some()
                        || end.is_some()
                    {
                        return Err(GenerateError::InvalidParameter(format!(
                            "map '{}' pattern 'worley_edges' only supports jitter/distance_fn",
                            key
                        )));
                    }

                    if let Some(value) = *jitter {
                        validate_unit_interval(&format!("maps.{}.jitter", key), value)?;
                    }

                    let distance_fn = parse_worley_distance_fn(distance_fn.as_deref())?;

                    let map_seed = DeterministicRng::derive_variant_seed(seed, key);
                    let mut noise =
                        WorleyNoise::new(map_seed).with_return_type(WorleyReturn::F2MinusF1);
                    noise = noise.with_distance_function(distance_fn);
                    if let Some(value) = *jitter {
                        noise = noise.with_jitter(value);
                    }

                    Ok(generate_noise_map(
                        width,
                        height,
                        tileable,
                        DEFAULT_NOISE_SCALE,
                        &noise,
                    ))
                }
                "stripes" => {
                    if noise_type.is_some()
                        || octaves.is_some()
                        || cells.is_some()
                        || line_width.is_some()
                        || start.is_some()
                        || end.is_some()
                        || jitter.is_some()
                        || distance_fn.is_some()
                    {
                        return Err(GenerateError::InvalidParameter(format!(
                            "map '{}' pattern 'stripes' only supports axis/frequency/duty_cycle/phase",
                            key
                        )));
                    }

                    let axis = parse_axis(axis.as_deref()).map_err(|msg| {
                        GenerateError::InvalidParameter(format!("map '{}' {}", key, msg))
                    })?;
                    let frequency = frequency.unwrap_or(8);
                    if frequency == 0 {
                        return Err(GenerateError::InvalidParameter(format!(
                            "map '{}' has invalid frequency=0",
                            key
                        )));
                    }
                    let duty = duty_cycle.unwrap_or(0.5);
                    validate_unit_interval(&format!("maps.{}.duty_cycle", key), duty)?;

                    let phase = phase.unwrap_or(0.0);
                    Ok(generate_stripes_map(width, height, axis, frequency, duty, phase))
                }
                "grid" => {
                    if noise_type.is_some()
                        || octaves.is_some()
                        || axis.is_some()
                        || frequency.is_some()
                        || duty_cycle.is_some()
                        || start.is_some()
                        || end.is_some()
                        || jitter.is_some()
                        || distance_fn.is_some()
                    {
                        return Err(GenerateError::InvalidParameter(format!(
                            "map '{}' pattern 'grid' only supports cells/line_width/phase",
                            key
                        )));
                    }

                    let cells = cells.unwrap_or([8, 8]);
                    if cells[0] == 0 || cells[1] == 0 {
                        return Err(GenerateError::InvalidParameter(format!(
                            "map '{}' has invalid cells={:?}",
                            key, cells
                        )));
                    }

                    let line_width = line_width.unwrap_or(0.1);
                    if !(line_width >= 0.0 && line_width <= 0.5) {
                        return Err(GenerateError::InvalidParameter(format!(
                            "map '{}' has invalid line_width={}",
                            key, line_width
                        )));
                    }

                    let phase = phase.unwrap_or(0.0);
                    Ok(generate_grid_map(width, height, cells, line_width, phase))
                }
                "gradient" => {
                    if noise_type.is_some()
                        || octaves.is_some()
                        || frequency.is_some()
                        || duty_cycle.is_some()
                        || cells.is_some()
                        || line_width.is_some()
                        || jitter.is_some()
                        || distance_fn.is_some()
                    {
                        return Err(GenerateError::InvalidParameter(format!(
                            "map '{}' pattern 'gradient' only supports axis/start/end/phase",
                            key
                        )));
                    }

                    let axis = parse_axis(axis.as_deref()).map_err(|msg| {
                        GenerateError::InvalidParameter(format!("map '{}' {}", key, msg))
                    })?;
                    let start = start.unwrap_or(0.0);
                    let end = end.unwrap_or(1.0);
                    validate_unit_interval(&format!("maps.{}.start", key), start)?;
                    validate_unit_interval(&format!("maps.{}.end", key), end)?;

                    let phase = phase.unwrap_or(0.0);
                    Ok(generate_gradient_map(
                        width,
                        height,
                        axis,
                        start,
                        end,
                        phase,
                        tileable,
                    ))
                }
                other => Err(GenerateError::InvalidParameter(format!(
                    "map '{}' has unsupported pattern '{}'",
                    key, other
                ))),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
}

fn parse_axis(axis: Option<&str>) -> Result<Axis, &'static str> {
    match axis.unwrap_or("x") {
        "x" => Ok(Axis::X),
        "y" => Ok(Axis::Y),
        _ => Err("has invalid axis (expected 'x' or 'y')"),
    }
}

fn parse_worley_distance_fn(value: Option<&str>) -> Result<DistanceFunction, GenerateError> {
    let Some(raw) = value else {
        return Ok(DistanceFunction::Euclidean);
    };

    match raw.trim().to_lowercase().as_str() {
        "euclidean" => Ok(DistanceFunction::Euclidean),
        "manhattan" => Ok(DistanceFunction::Manhattan),
        "chebyshev" => Ok(DistanceFunction::Chebyshev),
        _ => Err(GenerateError::InvalidParameter(format!(
            "unsupported distance_fn '{}'",
            raw
        ))),
    }
}

#[inline]
fn fract01(value: f64) -> f64 {
    value.rem_euclid(1.0)
}

fn generate_stripes_map(
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

fn generate_grid_map(
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

fn generate_gradient_map(
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoiseKind {
    Perlin,
    Simplex,
    Worley,
    Fbm,
}

fn parse_noise_type(value: Option<&str>) -> Result<NoiseKind, GenerateError> {
    let Some(raw) = value else {
        return Ok(NoiseKind::Perlin);
    };

    match raw.trim().to_lowercase().as_str() {
        "perlin" => Ok(NoiseKind::Perlin),
        "simplex" => Ok(NoiseKind::Simplex),
        "worley" => Ok(NoiseKind::Worley),
        "fbm" => Ok(NoiseKind::Fbm),
        _ => Err(GenerateError::InvalidParameter(format!(
            "unsupported noise_type '{}'",
            raw
        ))),
    }
}

fn build_noise(kind: NoiseKind, seed: u32, octaves: u8) -> Box<dyn Noise2D> {
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

fn generate_noise_map(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packing::{pack_channels, ChannelSource, PackedChannels};
    use crate::png;

    fn make_orm_params(resolution: [u32; 2]) -> TexturePackedV1Params {
        let mut maps = HashMap::new();
        maps.insert(
            "height".to_string(),
            MapDefinition::Pattern {
                pattern: "noise".to_string(),
                noise_type: Some("fbm".to_string()),
                octaves: Some(4),
                axis: None,
                frequency: None,
                duty_cycle: None,
                phase: None,
                cells: None,
                line_width: None,
                start: None,
                end: None,
                jitter: None,
                distance_fn: None,
            },
        );
        maps.insert(
            "ao".to_string(),
            MapDefinition::Grayscale {
                value: None,
                from_height: Some(true),
                ao_strength: Some(0.5),
            },
        );
        maps.insert(
            "roughness".to_string(),
            MapDefinition::Grayscale {
                value: None,
                from_height: Some(true),
                ao_strength: None,
            },
        );
        maps.insert(
            "metallic".to_string(),
            MapDefinition::Grayscale {
                value: Some(1.0),
                from_height: None,
                ao_strength: None,
            },
        );

        TexturePackedV1Params {
            resolution,
            tileable: true,
            maps,
        }
    }

    #[test]
    fn packed_maps_are_deterministic() {
        let params = make_orm_params([16, 16]);
        let maps_a = generate_packed_maps(&params, 123).unwrap();
        let maps_b = generate_packed_maps(&params, 123).unwrap();

        let channels = PackedChannels::rgb(
            ChannelSource::key("ao"),
            ChannelSource::key("roughness"),
            ChannelSource::key("metallic"),
        );

        let packed_a = pack_channels(&channels, &maps_a, 16, 16).unwrap();
        let packed_b = pack_channels(&channels, &maps_b, 16, 16).unwrap();

        let config = crate::png::PngConfig::default();
        let (_, hash_a) = png::write_rgba_to_vec_with_hash(&packed_a, &config).unwrap();
        let (_, hash_b) = png::write_rgba_to_vec_with_hash(&packed_b, &config).unwrap();

        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn packed_channels_match_sources() {
        let params = make_orm_params([8, 8]);
        let maps = generate_packed_maps(&params, 77).unwrap();

        let channels = PackedChannels::rgb(
            ChannelSource::key("ao"),
            ChannelSource::key("roughness"),
            ChannelSource::key("metallic"),
        );

        let packed = pack_channels(&channels, &maps, 8, 8).unwrap();

        let ao = maps.get("ao").unwrap();
        let roughness = maps.get("roughness").unwrap();
        let metallic = maps.get("metallic").unwrap();

        for (x, y) in [(0, 0), (3, 5), (7, 2)] {
            let packed_pixel = packed.get(x, y);
            let ao_pixel = ao.get(x, y);
            let rough_pixel = roughness.get(x, y);
            let metal_pixel = metallic.get(x, y);

            assert!(
                (packed_pixel.r - ao_pixel.r).abs() < 1e-10,
                "AO channel mismatch at ({}, {})",
                x,
                y
            );
            assert!(
                (packed_pixel.g - rough_pixel.r).abs() < 1e-10,
                "Roughness channel mismatch at ({}, {})",
                x,
                y
            );
            assert!(
                (packed_pixel.b - metal_pixel.r).abs() < 1e-10,
                "Metallic channel mismatch at ({}, {})",
                x,
                y
            );
        }
    }

    #[test]
    fn packed_inversion_matches_expected() {
        let mut maps = HashMap::new();
        maps.insert(
            "height".to_string(),
            MapDefinition::Pattern {
                pattern: "noise".to_string(),
                noise_type: Some("fbm".to_string()),
                octaves: Some(4),
                axis: None,
                frequency: None,
                duty_cycle: None,
                phase: None,
                cells: None,
                line_width: None,
                start: None,
                end: None,
                jitter: None,
                distance_fn: None,
            },
        );
        maps.insert(
            "rough".to_string(),
            MapDefinition::Grayscale {
                value: None,
                from_height: Some(true),
                ao_strength: None,
            },
        );

        let params = TexturePackedV1Params {
            resolution: [8, 8],
            tileable: true,
            maps,
        };

        let maps = generate_packed_maps(&params, 456).unwrap();
        let rough = maps.get("rough").unwrap();

        let invert = ChannelSource::extended("rough").invert(true).build();
        let channels = PackedChannels::rgb(invert.clone(), invert.clone(), invert);

        let packed = pack_channels(&channels, &maps, 8, 8).unwrap();

        for (x, y) in [(1, 1), (4, 3), (6, 7)] {
            let packed_pixel = packed.get(x, y);
            let rough_pixel = rough.get(x, y);
            let expected = 1.0 - rough_pixel.r;

            assert!(
                (packed_pixel.r - expected).abs() < 1e-10,
                "Invert mismatch at ({}, {})",
                x,
                y
            );
            assert!(
                (packed_pixel.g - expected).abs() < 1e-10,
                "Invert mismatch at ({}, {})",
                x,
                y
            );
            assert!(
                (packed_pixel.b - expected).abs() < 1e-10,
                "Invert mismatch at ({}, {})",
                x,
                y
            );
        }
    }

    #[test]
    fn stripes_pattern_generates_expected_pixels() {
        let mut maps = HashMap::new();
        maps.insert(
            "stripes".to_string(),
            MapDefinition::Pattern {
                pattern: "stripes".to_string(),
                noise_type: None,
                octaves: None,
                axis: Some("x".to_string()),
                frequency: Some(2),
                duty_cycle: Some(0.5),
                phase: Some(0.0),
                cells: None,
                line_width: None,
                start: None,
                end: None,
                jitter: None,
                distance_fn: None,
            },
        );

        let params = TexturePackedV1Params {
            resolution: [8, 1],
            tileable: true,
            maps,
        };

        let maps = generate_packed_maps(&params, 0).unwrap();
        let stripes = maps.get("stripes").unwrap();

        let expected = [1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
        for (x, expected) in expected.into_iter().enumerate() {
            let actual = stripes.get(x as u32, 0).r;
            assert!(
                (actual - expected).abs() < 1e-12,
                "stripes mismatch at x={}: expected {}, got {}",
                x,
                expected,
                actual
            );
        }
    }

    #[test]
    fn grid_pattern_generates_expected_lines() {
        let mut maps = HashMap::new();
        maps.insert(
            "grid".to_string(),
            MapDefinition::Pattern {
                pattern: "grid".to_string(),
                noise_type: None,
                octaves: None,
                axis: None,
                frequency: None,
                duty_cycle: None,
                phase: Some(0.0),
                cells: Some([2, 2]),
                line_width: Some(0.25),
                start: None,
                end: None,
                jitter: None,
                distance_fn: None,
            },
        );

        let params = TexturePackedV1Params {
            resolution: [8, 8],
            tileable: true,
            maps,
        };

        let maps = generate_packed_maps(&params, 0).unwrap();
        let grid = maps.get("grid").unwrap();

        assert!((grid.get(0, 0).r - 1.0).abs() < 1e-12);
        assert!((grid.get(1, 0).r - 1.0).abs() < 1e-12);
        assert!((grid.get(0, 1).r - 1.0).abs() < 1e-12);
        assert!((grid.get(1, 1).r - 0.0).abs() < 1e-12);
        assert!((grid.get(2, 2).r - 0.0).abs() < 1e-12);
        assert!((grid.get(4, 4).r - 1.0).abs() < 1e-12);
    }

    #[test]
    fn gradient_pattern_generates_expected_ramp() {
        let mut maps = HashMap::new();
        maps.insert(
            "gradient".to_string(),
            MapDefinition::Pattern {
                pattern: "gradient".to_string(),
                noise_type: None,
                octaves: None,
                axis: Some("x".to_string()),
                frequency: None,
                duty_cycle: None,
                phase: Some(0.0),
                cells: None,
                line_width: None,
                start: Some(0.0),
                end: Some(1.0),
                jitter: None,
                distance_fn: None,
            },
        );

        let params = TexturePackedV1Params {
            resolution: [8, 1],
            tileable: true,
            maps,
        };

        let maps = generate_packed_maps(&params, 0).unwrap();
        let gradient = maps.get("gradient").unwrap();

        assert!((gradient.get(0, 0).r - 0.0).abs() < 1e-12);
        assert!((gradient.get(4, 0).r - 1.0).abs() < 1e-12);
        assert!((gradient.get(7, 0).r - 0.25).abs() < 1e-12);
    }

    #[test]
    fn worley_edges_pattern_is_deterministic() {
        let mut maps = HashMap::new();
        maps.insert(
            "edges".to_string(),
            MapDefinition::Pattern {
                pattern: "worley_edges".to_string(),
                noise_type: None,
                octaves: None,
                axis: None,
                frequency: None,
                duty_cycle: None,
                phase: None,
                cells: None,
                line_width: None,
                start: None,
                end: None,
                jitter: Some(1.0),
                distance_fn: Some("manhattan".to_string()),
            },
        );

        let params = TexturePackedV1Params {
            resolution: [16, 16],
            tileable: true,
            maps,
        };

        let maps_a = generate_packed_maps(&params, 123).unwrap();
        let maps_b = generate_packed_maps(&params, 123).unwrap();

        let config = crate::png::PngConfig::default();
        let (_, hash_a) = png::write_rgba_to_vec_with_hash(maps_a.get("edges").unwrap(), &config)
            .unwrap();
        let (_, hash_b) = png::write_rgba_to_vec_with_hash(maps_b.get("edges").unwrap(), &config)
            .unwrap();

        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn noise_pattern_rejects_non_noise_params() {
        let mut maps = HashMap::new();
        maps.insert(
            "height".to_string(),
            MapDefinition::Pattern {
                pattern: "noise".to_string(),
                noise_type: Some("perlin".to_string()),
                octaves: None,
                axis: None,
                frequency: Some(4),
                duty_cycle: None,
                phase: None,
                cells: None,
                line_width: None,
                start: None,
                end: None,
                jitter: None,
                distance_fn: None,
            },
        );

        let params = TexturePackedV1Params {
            resolution: [8, 8],
            tileable: true,
            maps,
        };

        let err = generate_packed_maps(&params, 0).unwrap_err();
        assert!(err.to_string().contains("does not accept"));
    }
}
