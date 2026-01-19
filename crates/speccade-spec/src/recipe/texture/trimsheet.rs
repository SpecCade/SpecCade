//! Trimsheet / atlas texture recipe types.
//!
//! `texture.trimsheet_v1` packs multiple tile definitions into a single atlas
//! texture with deterministic shelf packing, mip-safe gutters, and UV metadata output.

use serde::{Deserialize, Serialize};

/// Parameters for the `texture.trimsheet_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureTrimsheetV1Params {
    /// Atlas resolution [width, height] in pixels.
    pub resolution: [u32; 2],
    /// Padding/gutter in pixels between tiles (for mip-safe borders).
    #[serde(default = "default_padding")]
    pub padding: u32,
    /// List of tiles to pack into the atlas.
    pub tiles: Vec<TrimsheetTile>,
}

fn default_padding() -> u32 {
    2
}

/// A tile definition for trimsheet packing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrimsheetTile {
    /// Unique identifier for this tile.
    pub id: String,
    /// Tile width in pixels.
    pub width: u32,
    /// Tile height in pixels.
    pub height: u32,
    /// Tile content source.
    #[serde(flatten)]
    pub source: TileSource,
}

/// Source for tile content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TileSource {
    /// Solid color fill.
    Color {
        /// RGBA color as [r, g, b, a] with values in 0.0-1.0 range.
        color: [f64; 4],
    },
    /// Reference to a procedural texture node (for future extension).
    NodeRef {
        /// Node id to reference.
        node_ref: String,
    },
}

/// UV rectangle for a packed tile in normalized [0, 1] coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileUvRect {
    /// Tile identifier.
    pub id: String,
    /// Left edge U coordinate (0-1).
    pub u_min: f64,
    /// Bottom edge V coordinate (0-1).
    pub v_min: f64,
    /// Right edge U coordinate (0-1).
    pub u_max: f64,
    /// Top edge V coordinate (0-1).
    pub v_max: f64,
    /// Tile width in pixels.
    pub width: u32,
    /// Tile height in pixels.
    pub height: u32,
}

/// Metadata output for a packed trimsheet atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrimsheetMetadata {
    /// Atlas width in pixels.
    pub atlas_width: u32,
    /// Atlas height in pixels.
    pub atlas_height: u32,
    /// Padding/gutter in pixels.
    pub padding: u32,
    /// UV rectangles for each packed tile.
    pub tiles: Vec<TileUvRect>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trimsheet_params_roundtrip() {
        let params = TextureTrimsheetV1Params {
            resolution: [1024, 1024],
            padding: 2,
            tiles: vec![
                TrimsheetTile {
                    id: "grass".to_string(),
                    width: 128,
                    height: 128,
                    source: TileSource::Color {
                        color: [0.2, 0.6, 0.2, 1.0],
                    },
                },
                TrimsheetTile {
                    id: "stone".to_string(),
                    width: 64,
                    height: 64,
                    source: TileSource::Color {
                        color: [0.5, 0.5, 0.5, 1.0],
                    },
                },
            ],
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: TextureTrimsheetV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn trimsheet_params_from_json() {
        let json = r#"
        {
          "resolution": [1024, 1024],
          "padding": 2,
          "tiles": [
            { "id": "grass", "width": 128, "height": 128, "color": [0.2, 0.6, 0.2, 1.0] },
            { "id": "stone", "width": 64, "height": 64, "color": [0.5, 0.5, 0.5, 1.0] }
          ]
        }
        "#;

        let params: TextureTrimsheetV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.resolution, [1024, 1024]);
        assert_eq!(params.padding, 2);
        assert_eq!(params.tiles.len(), 2);
        assert_eq!(params.tiles[0].id, "grass");
        assert_eq!(params.tiles[0].width, 128);
    }

    #[test]
    fn trimsheet_default_padding() {
        let json = r#"
        {
          "resolution": [512, 512],
          "tiles": []
        }
        "#;

        let params: TextureTrimsheetV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.padding, 2);
    }

    #[test]
    fn tile_uv_rect_serialization() {
        let uv = TileUvRect {
            id: "test".to_string(),
            u_min: 0.0,
            v_min: 0.0,
            u_max: 0.25,
            v_max: 0.25,
            width: 128,
            height: 128,
        };

        let json = serde_json::to_string(&uv).unwrap();
        let parsed: TileUvRect = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, uv);
    }

    #[test]
    fn trimsheet_metadata_serialization() {
        let metadata = TrimsheetMetadata {
            atlas_width: 1024,
            atlas_height: 1024,
            padding: 2,
            tiles: vec![TileUvRect {
                id: "grass".to_string(),
                u_min: 0.0,
                v_min: 0.0,
                u_max: 0.125,
                v_max: 0.125,
                width: 128,
                height: 128,
            }],
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        let parsed: TrimsheetMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, metadata);
    }

    #[test]
    fn tile_source_node_ref() {
        let json = r#"
        {
          "resolution": [512, 512],
          "tiles": [
            { "id": "procedural", "width": 256, "height": 256, "node_ref": "noise_output" }
          ]
        }
        "#;

        let params: TextureTrimsheetV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(params.tiles.len(), 1);
        assert!(matches!(
            &params.tiles[0].source,
            TileSource::NodeRef { node_ref } if node_ref == "noise_output"
        ));
    }
}
