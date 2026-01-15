//! Parsing utilities for packed texture parameters.

use crate::noise::DistanceFunction;

use super::super::GenerateError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

pub fn parse_axis(axis: Option<&str>) -> Result<Axis, &'static str> {
    match axis.unwrap_or("x") {
        "x" => Ok(Axis::X),
        "y" => Ok(Axis::Y),
        _ => Err("has invalid axis (expected 'x' or 'y')"),
    }
}

pub fn parse_worley_distance_fn(value: Option<&str>) -> Result<DistanceFunction, GenerateError> {
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

pub fn parse_noise_type(
    value: Option<&str>,
) -> Result<super::pattern_generators::NoiseKind, GenerateError> {
    use super::pattern_generators::NoiseKind;

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
