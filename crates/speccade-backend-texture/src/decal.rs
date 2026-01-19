//! Decal texture generation.
//!
//! Generates RGBA decal textures with optional normal and roughness outputs,
//! plus placement metadata.

use speccade_spec::recipe::texture::{DecalOutputMetadata, TextureDecalV1Params};
use thiserror::Error;

use crate::color::Color;
use crate::generate::{generate_graph, GenerateError, GraphValue};
use crate::maps::TextureBuffer;
use crate::png::{write_rgba_to_vec_with_hash, PngConfig, PngError};

/// Errors that can occur during decal generation.
#[derive(Debug, Error)]
pub enum DecalError {
    /// Graph evaluation failed.
    #[error("Graph evaluation failed: {0}")]
    GenerateError(#[from] GenerateError),

    /// PNG encoding failed.
    #[error("PNG encoding failed: {0}")]
    PngError(#[from] PngError),

    /// Invalid node reference.
    #[error("Node '{0}' not found in graph")]
    NodeNotFound(String),

    /// Invalid output type.
    #[error("Node '{0}' has wrong output type for {1}")]
    InvalidOutputType(String, String),

    /// Invalid parameter.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result of a single decal output (PNG).
#[derive(Debug)]
pub struct DecalTextureResult {
    /// PNG-encoded image data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
}

/// Result of decal generation.
#[derive(Debug)]
pub struct DecalResult {
    /// RGBA albedo texture with alpha composited.
    pub albedo: DecalTextureResult,
    /// Optional normal map.
    pub normal: Option<DecalTextureResult>,
    /// Optional roughness map.
    pub roughness: Option<DecalTextureResult>,
    /// Metadata for decal placement.
    pub metadata: DecalOutputMetadata,
}

/// Generate a decal from parameters.
///
/// # Arguments
/// * `params` - Decal parameters including nodes, outputs, and metadata.
/// * `seed` - Deterministic seed for noise operations.
///
/// # Returns
/// A `DecalResult` containing PNG outputs and metadata.
pub fn generate_decal(params: &TextureDecalV1Params, seed: u32) -> Result<DecalResult, DecalError> {
    let width = params.resolution[0];
    let height = params.resolution[1];

    // Validate parameters
    if width == 0 || height == 0 {
        return Err(DecalError::InvalidParameter(
            "Resolution must be non-zero".to_string(),
        ));
    }

    // Evaluate the procedural graph
    // Create a minimal procedural params struct to reuse the graph evaluator
    let procedural_params = speccade_spec::recipe::texture::TextureProceduralV1Params {
        resolution: params.resolution,
        tileable: false, // Decals typically don't tile
        nodes: params.nodes.clone(),
    };

    let nodes = generate_graph(&procedural_params, seed)?;

    // Get albedo output
    let albedo_value = nodes.get(&params.albedo_output).ok_or_else(|| {
        DecalError::NodeNotFound(params.albedo_output.clone())
    })?;

    // Get alpha output
    let alpha_value = nodes.get(&params.alpha_output).ok_or_else(|| {
        DecalError::NodeNotFound(params.alpha_output.clone())
    })?;

    // Composite albedo and alpha into RGBA texture
    let albedo_buffer = composite_albedo_with_alpha(albedo_value, alpha_value, width, height)?;

    // Encode albedo PNG
    let config = PngConfig::default();
    let (albedo_png_data, albedo_hash) = write_rgba_to_vec_with_hash(&albedo_buffer, &config)?;

    // Generate optional normal map
    let normal = if let Some(ref normal_id) = params.normal_output {
        let normal_value = nodes.get(normal_id).ok_or_else(|| {
            DecalError::NodeNotFound(normal_id.clone())
        })?;
        let normal_buffer = graph_value_to_rgba(normal_value, width, height)?;
        let (png_data, hash) = write_rgba_to_vec_with_hash(&normal_buffer, &config)?;
        Some(DecalTextureResult { png_data, hash })
    } else {
        None
    };

    // Generate optional roughness map
    let roughness = if let Some(ref roughness_id) = params.roughness_output {
        let roughness_value = nodes.get(roughness_id).ok_or_else(|| {
            DecalError::NodeNotFound(roughness_id.clone())
        })?;
        let roughness_buffer = graph_value_to_grayscale_rgba(roughness_value, width, height)?;
        let (png_data, hash) = write_rgba_to_vec_with_hash(&roughness_buffer, &config)?;
        Some(DecalTextureResult { png_data, hash })
    } else {
        None
    };

    // Build output metadata
    let metadata = DecalOutputMetadata {
        resolution: params.resolution,
        aspect_ratio: params.metadata.aspect_ratio,
        anchor: params.metadata.anchor,
        fade_distance: params.metadata.fade_distance,
        projection_size: params.metadata.projection_size,
        depth_range: params.metadata.depth_range,
        has_normal_map: normal.is_some(),
        has_roughness_map: roughness.is_some(),
    };

    Ok(DecalResult {
        albedo: DecalTextureResult {
            png_data: albedo_png_data,
            hash: albedo_hash,
        },
        normal,
        roughness,
        metadata,
    })
}

/// Composite albedo and alpha into a single RGBA buffer.
fn composite_albedo_with_alpha(
    albedo: &GraphValue,
    alpha: &GraphValue,
    width: u32,
    height: u32,
) -> Result<TextureBuffer, DecalError> {
    let mut buffer = TextureBuffer::new(width, height, Color::rgba(0.0, 0.0, 0.0, 0.0));

    match (albedo, alpha) {
        // Albedo is RGBA, alpha is grayscale
        (GraphValue::Color(albedo_buf), GraphValue::Grayscale(alpha_buf)) => {
            for y in 0..height {
                for x in 0..width {
                    let color = albedo_buf.get(x, y);
                    let a = alpha_buf.get(x, y);
                    buffer.set(x, y, Color::rgba(color.r, color.g, color.b, a));
                }
            }
        }
        // Albedo is grayscale, alpha is grayscale
        (GraphValue::Grayscale(albedo_buf), GraphValue::Grayscale(alpha_buf)) => {
            for y in 0..height {
                for x in 0..width {
                    let v = albedo_buf.get(x, y);
                    let a = alpha_buf.get(x, y);
                    buffer.set(x, y, Color::rgba(v, v, v, a));
                }
            }
        }
        // Albedo is RGBA, alpha is RGBA (use alpha channel)
        (GraphValue::Color(albedo_buf), GraphValue::Color(alpha_buf)) => {
            for y in 0..height {
                for x in 0..width {
                    let color = albedo_buf.get(x, y);
                    let alpha_color = alpha_buf.get(x, y);
                    // Use the alpha channel of the alpha source, or luminance if opaque
                    let a = if alpha_color.a < 0.999 {
                        alpha_color.a
                    } else {
                        // Use luminance as alpha
                        0.299 * alpha_color.r + 0.587 * alpha_color.g + 0.114 * alpha_color.b
                    };
                    buffer.set(x, y, Color::rgba(color.r, color.g, color.b, a));
                }
            }
        }
        // Albedo is grayscale, alpha is RGBA
        (GraphValue::Grayscale(albedo_buf), GraphValue::Color(alpha_buf)) => {
            for y in 0..height {
                for x in 0..width {
                    let v = albedo_buf.get(x, y);
                    let alpha_color = alpha_buf.get(x, y);
                    let a = if alpha_color.a < 0.999 {
                        alpha_color.a
                    } else {
                        0.299 * alpha_color.r + 0.587 * alpha_color.g + 0.114 * alpha_color.b
                    };
                    buffer.set(x, y, Color::rgba(v, v, v, a));
                }
            }
        }
    }

    Ok(buffer)
}

/// Convert a graph value to RGBA (for normal maps).
fn graph_value_to_rgba(
    value: &GraphValue,
    width: u32,
    height: u32,
) -> Result<TextureBuffer, DecalError> {
    let mut buffer = TextureBuffer::new(width, height, Color::rgba(0.5, 0.5, 1.0, 1.0));

    match value {
        GraphValue::Color(src) => {
            for y in 0..height {
                for x in 0..width {
                    buffer.set(x, y, src.get(x, y));
                }
            }
        }
        GraphValue::Grayscale(src) => {
            // For grayscale, create a flat normal (0.5, 0.5, 1.0)
            for y in 0..height {
                for x in 0..width {
                    let v = src.get(x, y);
                    buffer.set(x, y, Color::rgba(v, v, v, 1.0));
                }
            }
        }
    }

    Ok(buffer)
}

/// Convert a graph value to grayscale RGBA (for roughness maps).
fn graph_value_to_grayscale_rgba(
    value: &GraphValue,
    width: u32,
    height: u32,
) -> Result<TextureBuffer, DecalError> {
    let mut buffer = TextureBuffer::new(width, height, Color::rgba(0.5, 0.5, 0.5, 1.0));

    match value {
        GraphValue::Grayscale(src) => {
            for y in 0..height {
                for x in 0..width {
                    let v = src.get(x, y);
                    buffer.set(x, y, Color::rgba(v, v, v, 1.0));
                }
            }
        }
        GraphValue::Color(src) => {
            // Convert to grayscale using luminance
            for y in 0..height {
                for x in 0..width {
                    let color = src.get(x, y);
                    let v = 0.299 * color.r + 0.587 * color.g + 0.114 * color.b;
                    buffer.set(x, y, Color::rgba(v, v, v, 1.0));
                }
            }
        }
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::texture::{
        DecalMetadata, NoiseAlgorithm, NoiseConfig, TextureProceduralNode, TextureProceduralOp,
    };

    fn make_constant_node(id: &str, value: f64) -> TextureProceduralNode {
        TextureProceduralNode {
            id: id.to_string(),
            op: TextureProceduralOp::Constant { value },
        }
    }

    #[test]
    fn test_generate_basic_decal() {
        let params = TextureDecalV1Params {
            resolution: [64, 64],
            nodes: vec![
                make_constant_node("base", 0.5),
                make_constant_node("alpha", 1.0),
            ],
            albedo_output: "base".to_string(),
            alpha_output: "alpha".to_string(),
            normal_output: None,
            roughness_output: None,
            metadata: DecalMetadata::default(),
        };

        let result = generate_decal(&params, 42).unwrap();
        assert!(!result.albedo.png_data.is_empty());
        assert!(!result.albedo.hash.is_empty());
        assert!(result.normal.is_none());
        assert!(result.roughness.is_none());
        assert!(!result.metadata.has_normal_map);
        assert!(!result.metadata.has_roughness_map);
    }

    #[test]
    fn test_generate_decal_with_normal_and_roughness() {
        let params = TextureDecalV1Params {
            resolution: [32, 32],
            nodes: vec![
                make_constant_node("base", 0.5),
                make_constant_node("alpha", 0.8),
                make_constant_node("normal", 0.5),
                make_constant_node("rough", 0.7),
            ],
            albedo_output: "base".to_string(),
            alpha_output: "alpha".to_string(),
            normal_output: Some("normal".to_string()),
            roughness_output: Some("rough".to_string()),
            metadata: DecalMetadata {
                aspect_ratio: 2.0,
                anchor: [0.5, 1.0],
                fade_distance: 0.1,
                projection_size: Some([1.0, 0.5]),
                depth_range: None,
            },
        };

        let result = generate_decal(&params, 42).unwrap();
        assert!(!result.albedo.png_data.is_empty());
        assert!(result.normal.is_some());
        assert!(result.roughness.is_some());
        assert!(result.metadata.has_normal_map);
        assert!(result.metadata.has_roughness_map);
        assert_eq!(result.metadata.aspect_ratio, 2.0);
        assert_eq!(result.metadata.anchor, [0.5, 1.0]);
    }

    #[test]
    fn test_generate_decal_determinism() {
        let params = TextureDecalV1Params {
            resolution: [32, 32],
            nodes: vec![
                TextureProceduralNode {
                    id: "noise".to_string(),
                    op: TextureProceduralOp::Noise {
                        noise: NoiseConfig {
                            algorithm: NoiseAlgorithm::Perlin,
                            scale: 0.1,
                            octaves: 2,
                            persistence: 0.5,
                            lacunarity: 2.0,
                        },
                    },
                },
                TextureProceduralNode {
                    id: "alpha".to_string(),
                    op: TextureProceduralOp::Threshold {
                        input: "noise".to_string(),
                        threshold: 0.5,
                    },
                },
            ],
            albedo_output: "noise".to_string(),
            alpha_output: "alpha".to_string(),
            normal_output: None,
            roughness_output: None,
            metadata: DecalMetadata::default(),
        };

        let result1 = generate_decal(&params, 42).unwrap();
        let result2 = generate_decal(&params, 42).unwrap();

        assert_eq!(result1.albedo.png_data, result2.albedo.png_data);
        assert_eq!(result1.albedo.hash, result2.albedo.hash);
    }

    #[test]
    fn test_generate_decal_node_not_found() {
        let params = TextureDecalV1Params {
            resolution: [32, 32],
            nodes: vec![make_constant_node("base", 0.5)],
            albedo_output: "base".to_string(),
            alpha_output: "missing".to_string(),
            normal_output: None,
            roughness_output: None,
            metadata: DecalMetadata::default(),
        };

        let result = generate_decal(&params, 42);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, DecalError::NodeNotFound(ref id) if id == "missing"));
    }

    #[test]
    fn test_generate_decal_zero_resolution() {
        let params = TextureDecalV1Params {
            resolution: [0, 64],
            nodes: vec![make_constant_node("c", 0.5)],
            albedo_output: "c".to_string(),
            alpha_output: "c".to_string(),
            normal_output: None,
            roughness_output: None,
            metadata: DecalMetadata::default(),
        };

        let result = generate_decal(&params, 42);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DecalError::InvalidParameter(_)));
    }

    #[test]
    fn test_metadata_serialization() {
        let params = TextureDecalV1Params {
            resolution: [64, 64],
            nodes: vec![make_constant_node("c", 0.5)],
            albedo_output: "c".to_string(),
            alpha_output: "c".to_string(),
            normal_output: None,
            roughness_output: None,
            metadata: DecalMetadata {
                aspect_ratio: 1.5,
                anchor: [0.0, 0.5],
                fade_distance: 0.2,
                projection_size: Some([2.0, 1.0]),
                depth_range: Some([0.0, 0.05]),
            },
        };

        let result = generate_decal(&params, 42).unwrap();
        let json = serde_json::to_string_pretty(&result.metadata).unwrap();
        let parsed: DecalOutputMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, result.metadata);
    }
}
