//! Decal texture recipe types.
//!
//! `texture.decal_v1` produces RGBA decal textures with optional normal map and
//! roughness outputs, plus placement metadata for projection.

use serde::{Deserialize, Serialize};

use super::procedural::TextureProceduralNode;

/// Parameters for the `texture.decal_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureDecalV1Params {
    /// Texture resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Procedural graph nodes (a DAG) shared across all outputs.
    pub nodes: Vec<TextureProceduralNode>,
    /// Node id to use as the albedo/diffuse output (RGBA).
    pub albedo_output: String,
    /// Node id to use as the alpha output (grayscale). The alpha is composited
    /// into the RGBA albedo output.
    pub alpha_output: String,
    /// Optional node id to use for normal map output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal_output: Option<String>,
    /// Optional node id to use for roughness output (grayscale).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness_output: Option<String>,
    /// Placement metadata for decal projection.
    pub metadata: DecalMetadata,
}

/// Placement metadata for decal projection.
///
/// This metadata is emitted as a JSON sidecar and can be used by game engines
/// to correctly project and place decals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecalMetadata {
    /// Aspect ratio (width / height) for maintaining correct proportions.
    #[serde(default = "default_aspect_ratio")]
    pub aspect_ratio: f64,
    /// Anchor point in normalized [0, 1] coordinates. Default is center [0.5, 0.5].
    #[serde(default = "default_anchor")]
    pub anchor: [f64; 2],
    /// Fade distance from edges in normalized [0, 1] range for soft falloff.
    /// 0.0 = hard edges, higher values = softer fade from edges.
    #[serde(default = "default_fade_distance")]
    pub fade_distance: f64,
    /// Optional world-space projection bounds [width, height] in meters.
    /// If not specified, the engine should use its own default size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection_size: Option<[f64; 2]>,
    /// Optional depth range [near, far] for projection clipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth_range: Option<[f64; 2]>,
}

fn default_aspect_ratio() -> f64 {
    1.0
}

fn default_anchor() -> [f64; 2] {
    [0.5, 0.5]
}

fn default_fade_distance() -> f64 {
    0.0
}

impl Default for DecalMetadata {
    fn default() -> Self {
        Self {
            aspect_ratio: default_aspect_ratio(),
            anchor: default_anchor(),
            fade_distance: default_fade_distance(),
            projection_size: None,
            depth_range: None,
        }
    }
}

/// Metadata output for a generated decal.
///
/// This is written to `{asset_id}.decal.json` alongside the texture outputs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecalOutputMetadata {
    /// Texture resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Aspect ratio (width / height).
    pub aspect_ratio: f64,
    /// Anchor point in normalized [0, 1] coordinates.
    pub anchor: [f64; 2],
    /// Fade distance from edges.
    pub fade_distance: f64,
    /// Optional world-space projection bounds [width, height] in meters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection_size: Option<[f64; 2]>,
    /// Optional depth range [near, far] for projection clipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth_range: Option<[f64; 2]>,
    /// Whether a normal map output was generated.
    pub has_normal_map: bool,
    /// Whether a roughness output was generated.
    pub has_roughness_map: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::texture::{NoiseAlgorithm, NoiseConfig, TextureProceduralOp};

    #[test]
    fn decal_params_roundtrip() {
        let params = TextureDecalV1Params {
            resolution: [512, 512],
            nodes: vec![
                TextureProceduralNode {
                    id: "base".to_string(),
                    op: TextureProceduralOp::Noise {
                        noise: NoiseConfig {
                            algorithm: NoiseAlgorithm::Perlin,
                            scale: 0.05,
                            octaves: 2,
                            persistence: 0.5,
                            lacunarity: 2.0,
                        },
                    },
                },
                TextureProceduralNode {
                    id: "alpha".to_string(),
                    op: TextureProceduralOp::Threshold {
                        input: "base".to_string(),
                        threshold: 0.3,
                    },
                },
            ],
            albedo_output: "base".to_string(),
            alpha_output: "alpha".to_string(),
            normal_output: None,
            roughness_output: None,
            metadata: DecalMetadata {
                aspect_ratio: 1.0,
                anchor: [0.5, 0.5],
                fade_distance: 0.1,
                projection_size: None,
                depth_range: None,
            },
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: TextureDecalV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn decal_params_from_json() {
        let json = r#"
        {
          "resolution": [512, 512],
          "nodes": [
            { "id": "base", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.05 } },
            { "id": "alpha", "type": "threshold", "input": "base", "threshold": 0.3 }
          ],
          "albedo_output": "base",
          "alpha_output": "alpha",
          "metadata": {
            "aspect_ratio": 1.0,
            "anchor": [0.5, 0.5],
            "fade_distance": 0.1
          }
        }
        "#;

        let params: TextureDecalV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.resolution, [512, 512]);
        assert_eq!(params.albedo_output, "base");
        assert_eq!(params.alpha_output, "alpha");
        assert!(params.normal_output.is_none());
        assert!(params.roughness_output.is_none());
    }

    #[test]
    fn decal_params_with_optional_outputs() {
        let json = r#"
        {
          "resolution": [256, 256],
          "nodes": [
            { "id": "base", "type": "constant", "value": 0.5 },
            { "id": "alpha", "type": "constant", "value": 1.0 },
            { "id": "normal", "type": "normal_from_height", "input": "base", "strength": 1.0 },
            { "id": "rough", "type": "constant", "value": 0.7 }
          ],
          "albedo_output": "base",
          "alpha_output": "alpha",
          "normal_output": "normal",
          "roughness_output": "rough",
          "metadata": {
            "aspect_ratio": 2.0,
            "anchor": [0.5, 1.0],
            "fade_distance": 0.2,
            "projection_size": [1.0, 0.5],
            "depth_range": [0.0, 0.1]
          }
        }
        "#;

        let params: TextureDecalV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.normal_output, Some("normal".to_string()));
        assert_eq!(params.roughness_output, Some("rough".to_string()));
        assert_eq!(params.metadata.aspect_ratio, 2.0);
        assert_eq!(params.metadata.anchor, [0.5, 1.0]);
        assert_eq!(params.metadata.projection_size, Some([1.0, 0.5]));
        assert_eq!(params.metadata.depth_range, Some([0.0, 0.1]));
    }

    #[test]
    fn decal_metadata_defaults() {
        let json = r#"
        {
          "resolution": [64, 64],
          "nodes": [
            { "id": "c", "type": "constant", "value": 0.5 }
          ],
          "albedo_output": "c",
          "alpha_output": "c",
          "metadata": {}
        }
        "#;

        let params: TextureDecalV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.metadata.aspect_ratio, 1.0);
        assert_eq!(params.metadata.anchor, [0.5, 0.5]);
        assert_eq!(params.metadata.fade_distance, 0.0);
    }

    #[test]
    fn decal_output_metadata_roundtrip() {
        let metadata = DecalOutputMetadata {
            resolution: [512, 512],
            aspect_ratio: 1.0,
            anchor: [0.5, 0.5],
            fade_distance: 0.1,
            projection_size: Some([1.0, 1.0]),
            depth_range: None,
            has_normal_map: true,
            has_roughness_map: false,
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        let parsed: DecalOutputMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, metadata);
    }
}
