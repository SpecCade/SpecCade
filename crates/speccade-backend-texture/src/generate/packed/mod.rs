//! Packed texture map generation for `texture.packed_v1`.

use std::collections::HashMap;

use speccade_spec::recipe::texture::{MapDefinition, TexturePackedV1Params};

use crate::maps::{AoGenerator, GrayscaleBuffer, TextureBuffer};
use crate::rng::DeterministicRng;

use super::helpers::{validate_resolution, validate_unit_interval};
use super::GenerateError;

mod parsers;
mod pattern_generators;

#[cfg(test)]
mod tests;

use parsers::parse_noise_type;
use pattern_generators::{
    build_noise, generate_gradient_map, generate_grid_map, generate_noise_map, generate_stripes_map,
};

const DEFAULT_NOISE_SCALE: f64 = 0.05;
const DEFAULT_FBM_OCTAVES: u8 = 4;

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
        Some(generate_height_map(
            def,
            width,
            height,
            seed,
            params.tileable,
        )?)
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
    use parsers::{parse_axis, parse_worley_distance_fn};
    use pattern_generators::NoiseKind;

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
        } => match pattern.as_str() {
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
                use crate::noise::{WorleyNoise, WorleyReturn};

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
                Ok(generate_stripes_map(
                    width, height, axis, frequency, duty, phase,
                ))
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
                if !(0.0..=0.5).contains(&line_width) {
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
                    width, height, axis, start, end, phase, tileable,
                ))
            }
            other => Err(GenerateError::InvalidParameter(format!(
                "map '{}' has unsupported pattern '{}'",
                key, other
            ))),
        },
    }
}
