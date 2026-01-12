//! Packed texture map generation for `texture.packed_v1`.

use std::collections::HashMap;

use speccade_spec::recipe::texture::{MapDefinition, TexturePackedV1Params};

use crate::maps::{AoGenerator, GrayscaleBuffer, TextureBuffer};
use crate::noise::{tile_coord, Fbm, Noise2D, PerlinNoise, SimplexNoise, WorleyNoise};
use crate::rng::DeterministicRng;

use super::helpers::{validate_resolution, validate_unit_interval};
use super::GenerateError;

const DEFAULT_NOISE_SCALE: f64 = 0.05;
const DEFAULT_FBM_OCTAVES: u8 = 4;
const DEFAULT_NOISE_PERSISTENCE: f64 = 0.5;
const DEFAULT_NOISE_LACUNARITY: f64 = 2.0;

/// Generate all packed texture maps for `texture.packed_v1`.
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
        } => {
            if pattern != "noise" {
                return Err(GenerateError::InvalidParameter(format!(
                    "map '{}' has unsupported pattern '{}'",
                    key, pattern
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
    }
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
}
