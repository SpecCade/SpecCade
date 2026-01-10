//! Helper functions for texture generation.
//!
//! This module contains validation utilities, noise generator creation,
//! pattern application helpers, and default value getters used throughout
//! the texture generation pipeline.

use std::collections::HashSet;

use speccade_spec::recipe::texture::{
    MaterialType, TextureMapType, Texture2dMaterialMapsV1Params,
};
use speccade_spec::validation::common as shared_validation;

use super::GenerateError;

// Re-export shared utilities with consistent naming for this module
pub use crate::shared::create_noise_generator;
pub use crate::shared::sample_pattern_to_buffer as apply_pattern_to_buffer;
pub use crate::shared::sample_pattern_blended as apply_pattern_blended;
pub use crate::shared::apply_buffer_transform as apply_transform;
pub use crate::shared::PatternBlendMode as BlendMode;

/// Validate that resolution is positive and doesn't overflow.
///
/// This delegates to the shared validation in speccade-spec and converts
/// the error to a GenerateError for consistency with this backend's API.
pub fn validate_resolution(width: u32, height: u32) -> Result<(), GenerateError> {
    shared_validation::validate_resolution(width, height)
        .map_err(|e| GenerateError::InvalidParameter(e.message))
}

/// Validate that the map list is non-empty and has no duplicates.
pub fn validate_map_list(map_types: &[TextureMapType]) -> Result<(), GenerateError> {
    if map_types.is_empty() {
        return Err(GenerateError::InvalidParameter(
            "maps must contain at least one map type".to_string(),
        ));
    }

    let mut seen: HashSet<TextureMapType> = HashSet::new();
    for map_type in map_types {
        if !seen.insert(*map_type) {
            return Err(GenerateError::InvalidParameter(format!(
                "duplicate map type: {:?}",
                map_type
            )));
        }
    }

    Ok(())
}

/// Validate that a value is in [0, 1].
///
/// This delegates to the shared validation in speccade-spec and converts
/// the error to a GenerateError for consistency with this backend's API.
pub fn validate_unit_interval(name: &str, value: f64) -> Result<(), GenerateError> {
    shared_validation::validate_unit_interval(name, value)
        .map_err(|e| GenerateError::InvalidParameter(e.message))
}

/// Validate base material parameters.
pub fn validate_base_material(params: &Texture2dMaterialMapsV1Params) -> Result<(), GenerateError> {
    let Some(mat) = &params.base_material else {
        return Ok(());
    };

    for (i, c) in mat.base_color.iter().enumerate() {
        validate_unit_interval(&format!("base_material.base_color[{}]", i), *c)?;
    }

    if let Some([min, max]) = mat.roughness_range {
        validate_unit_interval("base_material.roughness_range[0]", min)?;
        validate_unit_interval("base_material.roughness_range[1]", max)?;
        if min > max {
            return Err(GenerateError::InvalidParameter(format!(
                "base_material.roughness_range must have min <= max, got [{}, {}]",
                min, max
            )));
        }
    }

    if let Some(metallic) = mat.metallic {
        validate_unit_interval("base_material.metallic", metallic)?;
    }

    Ok(())
}

/// Get default roughness range for a material type.
pub fn get_default_roughness_range(material_type: &MaterialType) -> [f64; 2] {
    match material_type {
        MaterialType::Metal => [0.2, 0.5],
        MaterialType::Wood => [0.5, 0.8],
        MaterialType::Stone => [0.5, 0.9],
        MaterialType::Fabric => [0.8, 1.0],
        MaterialType::Plastic => [0.3, 0.5],
        MaterialType::Concrete => [0.6, 0.95],
        MaterialType::Brick => [0.6, 0.9],
        MaterialType::Procedural => [0.3, 0.7],
    }
}

/// Get default metallic value for a material type.
pub fn get_default_metallic(material_type: &MaterialType) -> f64 {
    match material_type {
        MaterialType::Metal => 1.0,
        _ => 0.0,
    }
}
