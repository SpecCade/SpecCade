//! Packed texture types for channel packing recipes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Map definition for the packed texture recipe.
///
/// Each map definition specifies how to generate a single texture map
/// that can then be packed into output channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MapDefinition {
    /// A solid grayscale value.
    Grayscale {
        /// The grayscale value (0.0 to 1.0).
        #[serde(default)]
        value: Option<f64>,
        /// Whether to generate from height map (for procedural variation).
        #[serde(default)]
        from_height: Option<bool>,
        /// AO strength when generating from height (0.0 to 1.0).
        #[serde(default)]
        ao_strength: Option<f64>,
    },
    /// A procedural pattern.
    Pattern {
        /// The pattern type (v1: only "noise").
        pattern: String,
        /// Noise type for noise patterns.
        #[serde(default)]
        noise_type: Option<String>,
        /// Number of octaves for FBM noise (only valid for noise_type="fbm").
        #[serde(default)]
        octaves: Option<u32>,
    },
}

impl MapDefinition {
    /// Creates a constant grayscale map definition.
    pub fn constant(value: f64) -> Self {
        MapDefinition::Grayscale {
            value: Some(value),
            from_height: None,
            ao_strength: None,
        }
    }

    /// Creates a grayscale map definition derived from height.
    pub fn from_height() -> Self {
        MapDefinition::Grayscale {
            value: None,
            from_height: Some(true),
            ao_strength: None,
        }
    }

    /// Creates an AO map definition with the given strength.
    pub fn ao(strength: f64) -> Self {
        MapDefinition::Grayscale {
            value: None,
            from_height: Some(true),
            ao_strength: Some(strength),
        }
    }
}

/// Parameters for the `texture.packed_v1` recipe.
///
/// This recipe generates multiple texture maps and packs them into
/// the output channels as specified by the output's `channels` field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TexturePackedV1Params {
    /// Texture resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Whether the texture should tile seamlessly.
    pub tileable: bool,
    /// Map definitions keyed by user-defined names.
    /// These keys are referenced in the output's `channels` field.
    pub maps: HashMap<String, MapDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_definition_grayscale_constant() {
        let def = MapDefinition::constant(0.5);
        let json = serde_json::to_string(&def).unwrap();
        assert!(json.contains("\"type\":\"grayscale\""));
        assert!(json.contains("\"value\":0.5"));
    }

    #[test]
    fn test_map_definition_grayscale_from_height() {
        let def = MapDefinition::from_height();
        let json = serde_json::to_string(&def).unwrap();
        assert!(json.contains("\"type\":\"grayscale\""));
        assert!(json.contains("\"from_height\":true"));
    }

    #[test]
    fn test_map_definition_grayscale_ao() {
        let def = MapDefinition::ao(0.5);
        let json = serde_json::to_string(&def).unwrap();
        assert!(json.contains("\"type\":\"grayscale\""));
        assert!(json.contains("\"ao_strength\":0.5"));
    }

    #[test]
    fn test_packed_params_roundtrip() {
        let mut maps = HashMap::new();
        maps.insert("metal".to_string(), MapDefinition::constant(1.0));
        maps.insert("rough".to_string(), MapDefinition::from_height());

        let params = TexturePackedV1Params {
            resolution: [512, 512],
            tileable: true,
            maps,
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: TexturePackedV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.resolution, params.resolution);
        assert_eq!(parsed.tileable, params.tileable);
        assert_eq!(parsed.maps.len(), 2);
    }

    #[test]
    fn test_packed_params_from_spec_json() {
        let json = r#"{
            "resolution": [512, 512],
            "tileable": true,
            "maps": {
                "ao": {
                    "type": "grayscale",
                    "from_height": true,
                    "ao_strength": 0.5
                },
                "roughness": {
                    "type": "grayscale",
                    "from_height": true
                },
                "metallic": {
                    "type": "grayscale",
                    "value": 1.0
                }
            }
        }"#;

        let params: TexturePackedV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.resolution, [512, 512]);
        assert!(params.tileable);
        assert_eq!(params.maps.len(), 3);
        assert!(params.maps.contains_key("ao"));
        assert!(params.maps.contains_key("roughness"));
        assert!(params.maps.contains_key("metallic"));
    }
}
