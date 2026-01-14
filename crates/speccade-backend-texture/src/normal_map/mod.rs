//! Dedicated normal map generation from patterns.
//!
//! This module generates normal maps directly from pattern specifications,
//! as opposed to deriving them from height maps. This provides more control
//! over normal map appearance and supports pattern-specific optimizations.

use std::path::Path;

use speccade_spec::recipe::texture::TextureNormalV1Params;
use speccade_spec::validation::common as shared_validation;
use speccade_spec::BackendError;

use crate::maps::GrayscaleBuffer;
use crate::normal_map_patterns::generate_height_from_pattern;
use crate::png::{self, PngConfig, PngError};

mod conversion;
mod processing;

#[cfg(test)]
mod tests;

use conversion::height_to_normal;
use processing::apply_processing;

/// Errors from normal map generation.
#[derive(Debug, thiserror::Error)]
pub enum NormalMapError {
    #[error("PNG error: {0}")]
    Png(#[from] PngError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

impl BackendError for NormalMapError {
    fn code(&self) -> &'static str {
        match self {
            NormalMapError::Png(_) => "NORMAL_001",
            NormalMapError::Io(_) => "NORMAL_002",
            NormalMapError::InvalidParameter(_) => "NORMAL_003",
        }
    }

    fn category(&self) -> &'static str {
        "texture"
    }
}

/// Result of generating a normal map.
#[derive(Debug)]
pub struct NormalMapResult {
    /// The generated normal map data (RGB PNG).
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// BLAKE3 hash of the PNG file.
    pub hash: String,
    /// File path if saved.
    pub file_path: Option<String>,
}

/// Generate a normal map from pattern specification.
pub fn generate_normal_map(
    params: &TextureNormalV1Params,
    seed: u32,
) -> Result<NormalMapResult, NormalMapError> {
    let width = params.resolution[0];
    let height = params.resolution[1];

    // Use shared validation for resolution
    shared_validation::validate_resolution(width, height)
        .map_err(|e| NormalMapError::InvalidParameter(e.message))?;

    // Use shared validation for bump_strength (must be non-negative)
    shared_validation::validate_non_negative("bump_strength", params.bump_strength)
        .map_err(|e| NormalMapError::InvalidParameter(e.message))?;

    // Generate height map from pattern
    let mut height_map = if let Some(pattern) = &params.pattern {
        generate_height_from_pattern(pattern, width, height, seed, params.tileable)
    } else {
        // No pattern: generate flat normal map
        GrayscaleBuffer::new(width, height, 0.5)
    };

    // Apply post-processing
    if let Some(processing) = &params.processing {
        apply_processing(&mut height_map, processing);
    }

    // Convert height map to normal map using Sobel operators
    let normal_buffer = height_to_normal(&height_map, params.bump_strength);

    // Encode to PNG with hash
    let config = PngConfig::default();
    let (data, hash) = png::write_rgb_to_vec_with_hash(&normal_buffer, &config)?;

    Ok(NormalMapResult {
        data,
        width,
        height,
        hash,
        file_path: None,
    })
}

/// Save normal map result to file.
pub fn save_normal_map(
    result: &NormalMapResult,
    output_path: &Path,
) -> Result<NormalMapResult, NormalMapError> {
    // Create parent directory if needed
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write PNG to file
    std::fs::write(output_path, &result.data)?;

    Ok(NormalMapResult {
        data: result.data.clone(),
        width: result.width,
        height: result.height,
        hash: result.hash.clone(),
        file_path: Some(output_path.to_string_lossy().to_string()),
    })
}
