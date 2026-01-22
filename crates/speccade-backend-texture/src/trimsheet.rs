//! Trimsheet/atlas generation with deterministic shelf packing.
//!
//! This module implements a deterministic shelf packing algorithm for creating
//! texture atlases from a list of tile definitions. The algorithm produces
//! byte-identical output for the same input (Tier 1 determinism).
//!
//! Features:
//! - Deterministic shelf packing (tiles sorted by height, then width, then id)
//! - Mip-safe gutters (edge pixels are replicated into padding)
//! - UV metadata output for each packed tile

use std::collections::HashMap;

use speccade_spec::recipe::texture::{
    TextureTrimsheetV1Params, TileSource, TileUvRect, TrimsheetMetadata, TrimsheetTile,
};
use thiserror::Error;

use crate::color::Color;
use crate::maps::TextureBuffer;
use crate::png::{write_rgba_to_vec_with_hash, PngConfig};

/// Errors that can occur during trimsheet generation.
#[derive(Debug, Error)]
pub enum TrimsheetError {
    /// Tile is too large to fit in the atlas.
    #[error("Tile '{0}' ({1}x{2}) with padding is too large for atlas ({3}x{4})")]
    TileTooLarge(String, u32, u32, u32, u32),

    /// Not enough space to pack all tiles.
    #[error(
        "Cannot fit all tiles into atlas. Consider increasing resolution or reducing tile sizes"
    )]
    PackingFailed,

    /// Duplicate tile id.
    #[error("Duplicate tile id: '{0}'")]
    DuplicateTileId(String),

    /// PNG encoding error.
    #[error("PNG encoding error: {0}")]
    PngError(#[from] crate::png::PngError),

    /// Invalid parameters.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result of trimsheet generation.
#[derive(Debug)]
pub struct TrimsheetResult {
    /// PNG-encoded atlas image data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
    /// Metadata with UV coordinates for each tile.
    pub metadata: TrimsheetMetadata,
}

/// Internal representation of a packed tile placement.
#[derive(Debug, Clone)]
struct PackedTile {
    /// Tile id.
    id: String,
    /// X position in atlas (excluding padding).
    x: u32,
    /// Y position in atlas (excluding padding).
    y: u32,
    /// Tile width (without padding).
    width: u32,
    /// Tile height (without padding).
    height: u32,
}

/// Shelf for shelf packing algorithm.
#[derive(Debug)]
struct Shelf {
    /// Y position of this shelf.
    y: u32,
    /// Height of this shelf (tallest tile + padding).
    height: u32,
    /// Current X position (next free spot).
    current_x: u32,
}

/// Generate a trimsheet atlas from parameters.
///
/// # Arguments
/// * `params` - Trimsheet parameters including resolution, padding, and tiles
/// * `_seed` - Deterministic seed (reserved for future procedural tile sources)
///
/// # Returns
/// A `TrimsheetResult` containing the PNG data, hash, and UV metadata.
pub fn generate_trimsheet(
    params: &TextureTrimsheetV1Params,
    _seed: u32,
) -> Result<TrimsheetResult, TrimsheetError> {
    let width = params.resolution[0];
    let height = params.resolution[1];
    let padding = params.padding;

    // Validate parameters
    if width == 0 || height == 0 {
        return Err(TrimsheetError::InvalidParameter(
            "Atlas resolution must be non-zero".to_string(),
        ));
    }

    // Check for duplicate tile ids
    let mut seen_ids: HashMap<&str, bool> = HashMap::new();
    for tile in &params.tiles {
        if seen_ids.contains_key(tile.id.as_str()) {
            return Err(TrimsheetError::DuplicateTileId(tile.id.clone()));
        }
        seen_ids.insert(&tile.id, true);
    }

    // Validate all tiles can fit
    for tile in &params.tiles {
        let padded_width = tile.width + padding * 2;
        let padded_height = tile.height + padding * 2;
        if padded_width > width || padded_height > height {
            return Err(TrimsheetError::TileTooLarge(
                tile.id.clone(),
                tile.width,
                tile.height,
                width,
                height,
            ));
        }
    }

    // Pack tiles using shelf algorithm
    let packed = pack_tiles_shelf(&params.tiles, width, height, padding)?;

    // Create atlas buffer
    let mut atlas = TextureBuffer::new(width, height, Color::rgba(0.0, 0.0, 0.0, 0.0));

    // Render each tile into the atlas
    for (tile, placement) in params.tiles.iter().zip(packed.iter()) {
        render_tile_with_gutter(&mut atlas, tile, placement, padding);
    }

    // Encode to PNG
    let config = PngConfig::default();
    let (png_data, hash) = write_rgba_to_vec_with_hash(&atlas, &config)?;

    // Build metadata
    let tile_uvs: Vec<TileUvRect> = packed
        .iter()
        .map(|p| {
            // UV coordinates are normalized [0, 1]
            // Note: UVs point to the inner tile content, not the gutter
            TileUvRect {
                id: p.id.clone(),
                u_min: p.x as f64 / width as f64,
                v_min: p.y as f64 / height as f64,
                u_max: (p.x + p.width) as f64 / width as f64,
                v_max: (p.y + p.height) as f64 / height as f64,
                width: p.width,
                height: p.height,
            }
        })
        .collect();

    let metadata = TrimsheetMetadata {
        atlas_width: width,
        atlas_height: height,
        padding,
        tiles: tile_uvs,
    };

    Ok(TrimsheetResult {
        png_data,
        hash,
        metadata,
    })
}

/// Pack tiles using a deterministic shelf packing algorithm.
///
/// Tiles are sorted by height (descending), then width (descending), then id
/// to ensure deterministic output. The algorithm places tiles on shelves,
/// creating new shelves as needed.
fn pack_tiles_shelf(
    tiles: &[TrimsheetTile],
    atlas_width: u32,
    atlas_height: u32,
    padding: u32,
) -> Result<Vec<PackedTile>, TrimsheetError> {
    if tiles.is_empty() {
        return Ok(Vec::new());
    }

    // Create sorted indices for deterministic ordering
    // Sort by: height (desc), width (desc), id (asc)
    let mut indices: Vec<usize> = (0..tiles.len()).collect();
    indices.sort_by(|&a, &b| {
        let tile_a = &tiles[a];
        let tile_b = &tiles[b];

        // Primary: height descending
        match tile_b.height.cmp(&tile_a.height) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // Secondary: width descending
        match tile_b.width.cmp(&tile_a.width) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // Tertiary: id ascending (for stable determinism)
        tile_a.id.cmp(&tile_b.id)
    });

    let mut shelves: Vec<Shelf> = Vec::new();
    let mut placements: Vec<Option<PackedTile>> = vec![None; tiles.len()];

    for &idx in &indices {
        let tile = &tiles[idx];
        let padded_width = tile.width + padding * 2;
        let padded_height = tile.height + padding * 2;

        // Try to find a shelf that can accommodate this tile
        let mut placed = false;
        for shelf in &mut shelves {
            if shelf.current_x + padded_width <= atlas_width && padded_height <= shelf.height {
                // Place on this shelf
                placements[idx] = Some(PackedTile {
                    id: tile.id.clone(),
                    x: shelf.current_x + padding,
                    y: shelf.y + padding,
                    width: tile.width,
                    height: tile.height,
                });
                shelf.current_x += padded_width;
                placed = true;
                break;
            }
        }

        if !placed {
            // Create a new shelf
            let shelf_y = shelves.last().map_or(0, |s| s.y + s.height);

            if shelf_y + padded_height > atlas_height {
                return Err(TrimsheetError::PackingFailed);
            }

            let new_shelf = Shelf {
                y: shelf_y,
                height: padded_height,
                current_x: padded_width,
            };

            placements[idx] = Some(PackedTile {
                id: tile.id.clone(),
                x: padding,
                y: shelf_y + padding,
                width: tile.width,
                height: tile.height,
            });

            shelves.push(new_shelf);
        }
    }

    // Return placements in original order
    Ok(placements.into_iter().map(|p| p.unwrap()).collect())
}

/// Render a tile into the atlas with mip-safe gutter.
///
/// The gutter pixels are filled by replicating the edge pixels of the tile,
/// which prevents color bleeding during mipmap generation.
fn render_tile_with_gutter(
    atlas: &mut TextureBuffer,
    tile: &TrimsheetTile,
    placement: &PackedTile,
    padding: u32,
) {
    // Get tile color
    let color = match &tile.source {
        TileSource::Color { color } => Color::rgba(color[0], color[1], color[2], color[3]),
        TileSource::NodeRef { .. } => {
            // NodeRef is for future extension - use magenta placeholder
            Color::rgba(1.0, 0.0, 1.0, 1.0)
        }
    };

    // Fill the tile content area
    for y in 0..tile.height {
        for x in 0..tile.width {
            atlas.set(placement.x + x, placement.y + y, color);
        }
    }

    // Fill gutters by replicating edge pixels
    if padding > 0 {
        // For solid color tiles, the edge color is the same everywhere
        // so we just fill the gutter regions with the same color

        // Top gutter
        for gy in 0..padding {
            for x in 0..tile.width {
                let gutter_x = placement.x + x;
                let gutter_y = placement.y.saturating_sub(padding) + gy;
                if gutter_y < atlas.height {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }

        // Bottom gutter
        for gy in 0..padding {
            for x in 0..tile.width {
                let gutter_x = placement.x + x;
                let gutter_y = placement.y + tile.height + gy;
                if gutter_y < atlas.height {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }

        // Left gutter
        for y in 0..tile.height {
            for gx in 0..padding {
                let gutter_x = placement.x.saturating_sub(padding) + gx;
                let gutter_y = placement.y + y;
                if gutter_x < atlas.width {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }

        // Right gutter
        for y in 0..tile.height {
            for gx in 0..padding {
                let gutter_x = placement.x + tile.width + gx;
                let gutter_y = placement.y + y;
                if gutter_x < atlas.width {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }

        // Corner gutters
        // Top-left
        for gy in 0..padding {
            for gx in 0..padding {
                let gutter_x = placement.x.saturating_sub(padding) + gx;
                let gutter_y = placement.y.saturating_sub(padding) + gy;
                if gutter_x < atlas.width && gutter_y < atlas.height {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }

        // Top-right
        for gy in 0..padding {
            for gx in 0..padding {
                let gutter_x = placement.x + tile.width + gx;
                let gutter_y = placement.y.saturating_sub(padding) + gy;
                if gutter_x < atlas.width && gutter_y < atlas.height {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }

        // Bottom-left
        for gy in 0..padding {
            for gx in 0..padding {
                let gutter_x = placement.x.saturating_sub(padding) + gx;
                let gutter_y = placement.y + tile.height + gy;
                if gutter_x < atlas.width && gutter_y < atlas.height {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }

        // Bottom-right
        for gy in 0..padding {
            for gx in 0..padding {
                let gutter_x = placement.x + tile.width + gx;
                let gutter_y = placement.y + tile.height + gy;
                if gutter_x < atlas.width && gutter_y < atlas.height {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tile(id: &str, width: u32, height: u32, color: [f64; 4]) -> TrimsheetTile {
        TrimsheetTile {
            id: id.to_string(),
            width,
            height,
            source: TileSource::Color { color },
        }
    }

    #[test]
    fn test_empty_tiles() {
        let params = TextureTrimsheetV1Params {
            resolution: [256, 256],
            padding: 2,
            tiles: vec![],
        };

        let result = generate_trimsheet(&params, 42).unwrap();
        assert_eq!(result.metadata.tiles.len(), 0);
        assert!(!result.png_data.is_empty());
    }

    #[test]
    fn test_single_tile() {
        let params = TextureTrimsheetV1Params {
            resolution: [256, 256],
            padding: 2,
            tiles: vec![make_tile("grass", 64, 64, [0.2, 0.6, 0.2, 1.0])],
        };

        let result = generate_trimsheet(&params, 42).unwrap();
        assert_eq!(result.metadata.tiles.len(), 1);
        assert_eq!(result.metadata.tiles[0].id, "grass");
        assert_eq!(result.metadata.tiles[0].width, 64);
        assert_eq!(result.metadata.tiles[0].height, 64);
    }

    #[test]
    fn test_multiple_tiles() {
        let params = TextureTrimsheetV1Params {
            resolution: [512, 512],
            padding: 2,
            tiles: vec![
                make_tile("grass", 128, 128, [0.2, 0.6, 0.2, 1.0]),
                make_tile("stone", 64, 64, [0.5, 0.5, 0.5, 1.0]),
                make_tile("water", 128, 64, [0.1, 0.3, 0.8, 1.0]),
            ],
        };

        let result = generate_trimsheet(&params, 42).unwrap();
        assert_eq!(result.metadata.tiles.len(), 3);

        // Verify all tiles have unique positions
        let positions: Vec<_> = result
            .metadata
            .tiles
            .iter()
            .map(|t| (t.u_min, t.v_min))
            .collect();
        for (i, p1) in positions.iter().enumerate() {
            for (j, p2) in positions.iter().enumerate() {
                if i != j {
                    assert!(
                        p1 != p2,
                        "Tiles {} and {} have same position",
                        result.metadata.tiles[i].id,
                        result.metadata.tiles[j].id
                    );
                }
            }
        }
    }

    #[test]
    fn test_determinism() {
        let params = TextureTrimsheetV1Params {
            resolution: [256, 256],
            padding: 2,
            tiles: vec![
                make_tile("a", 64, 64, [1.0, 0.0, 0.0, 1.0]),
                make_tile("b", 32, 32, [0.0, 1.0, 0.0, 1.0]),
                make_tile("c", 48, 48, [0.0, 0.0, 1.0, 1.0]),
            ],
        };

        let result1 = generate_trimsheet(&params, 42).unwrap();
        let result2 = generate_trimsheet(&params, 42).unwrap();

        // PNG data should be byte-identical
        assert_eq!(result1.png_data, result2.png_data);
        assert_eq!(result1.hash, result2.hash);

        // Metadata should be identical
        assert_eq!(result1.metadata.tiles.len(), result2.metadata.tiles.len());
        for (t1, t2) in result1
            .metadata
            .tiles
            .iter()
            .zip(result2.metadata.tiles.iter())
        {
            assert_eq!(t1.id, t2.id);
            assert!((t1.u_min - t2.u_min).abs() < 1e-10);
            assert!((t1.v_min - t2.v_min).abs() < 1e-10);
            assert!((t1.u_max - t2.u_max).abs() < 1e-10);
            assert!((t1.v_max - t2.v_max).abs() < 1e-10);
        }
    }

    #[test]
    fn test_tile_too_large() {
        let params = TextureTrimsheetV1Params {
            resolution: [64, 64],
            padding: 2,
            tiles: vec![make_tile("huge", 128, 128, [1.0, 1.0, 1.0, 1.0])],
        };

        let err = generate_trimsheet(&params, 42).unwrap_err();
        assert!(matches!(err, TrimsheetError::TileTooLarge(..)));
    }

    #[test]
    fn test_duplicate_tile_id() {
        let params = TextureTrimsheetV1Params {
            resolution: [256, 256],
            padding: 2,
            tiles: vec![
                make_tile("same", 32, 32, [1.0, 0.0, 0.0, 1.0]),
                make_tile("same", 64, 64, [0.0, 1.0, 0.0, 1.0]),
            ],
        };

        let err = generate_trimsheet(&params, 42).unwrap_err();
        assert!(matches!(err, TrimsheetError::DuplicateTileId(..)));
    }

    #[test]
    fn test_packing_failed() {
        // Try to pack too many tiles in a small atlas
        let params = TextureTrimsheetV1Params {
            resolution: [64, 64],
            padding: 0,
            tiles: vec![
                make_tile("a", 32, 32, [1.0, 0.0, 0.0, 1.0]),
                make_tile("b", 32, 32, [0.0, 1.0, 0.0, 1.0]),
                make_tile("c", 32, 32, [0.0, 0.0, 1.0, 1.0]),
                make_tile("d", 32, 32, [1.0, 1.0, 0.0, 1.0]),
                make_tile("e", 32, 32, [1.0, 0.0, 1.0, 1.0]), // This won't fit
            ],
        };

        let err = generate_trimsheet(&params, 42).unwrap_err();
        assert!(matches!(err, TrimsheetError::PackingFailed));
    }

    #[test]
    fn test_uv_coordinates_normalized() {
        let params = TextureTrimsheetV1Params {
            resolution: [256, 256],
            padding: 0,
            tiles: vec![make_tile("test", 128, 128, [1.0, 1.0, 1.0, 1.0])],
        };

        let result = generate_trimsheet(&params, 42).unwrap();
        let uv = &result.metadata.tiles[0];

        // UVs should be in [0, 1] range
        assert!(uv.u_min >= 0.0 && uv.u_min <= 1.0);
        assert!(uv.v_min >= 0.0 && uv.v_min <= 1.0);
        assert!(uv.u_max >= 0.0 && uv.u_max <= 1.0);
        assert!(uv.v_max >= 0.0 && uv.v_max <= 1.0);

        // For 128x128 tile in 256x256 atlas, UV range should be 0.5
        assert!((uv.u_max - uv.u_min - 0.5).abs() < 1e-10);
        assert!((uv.v_max - uv.v_min - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_shelf_packing_order() {
        // Verify that tiles are sorted by height for shelf packing
        let params = TextureTrimsheetV1Params {
            resolution: [512, 512],
            padding: 0,
            tiles: vec![
                make_tile("small", 32, 32, [1.0, 0.0, 0.0, 1.0]),
                make_tile("tall", 32, 128, [0.0, 1.0, 0.0, 1.0]),
                make_tile("medium", 32, 64, [0.0, 0.0, 1.0, 1.0]),
            ],
        };

        let result = generate_trimsheet(&params, 42).unwrap();

        // The tall tile should be on the first shelf (lowest y)
        let tall = result
            .metadata
            .tiles
            .iter()
            .find(|t| t.id == "tall")
            .unwrap();
        let medium = result
            .metadata
            .tiles
            .iter()
            .find(|t| t.id == "medium")
            .unwrap();
        let small = result
            .metadata
            .tiles
            .iter()
            .find(|t| t.id == "small")
            .unwrap();

        // Tall should have the smallest v_min (placed first)
        assert!(tall.v_min <= medium.v_min);
        assert!(tall.v_min <= small.v_min);
    }

    #[test]
    fn test_zero_padding() {
        let params = TextureTrimsheetV1Params {
            resolution: [128, 128],
            padding: 0,
            tiles: vec![make_tile("test", 64, 64, [0.5, 0.5, 0.5, 1.0])],
        };

        let result = generate_trimsheet(&params, 42).unwrap();
        assert_eq!(result.metadata.padding, 0);

        // UV should start at (0, 0) with no padding
        let uv = &result.metadata.tiles[0];
        assert!((uv.u_min).abs() < 1e-10);
        assert!((uv.v_min).abs() < 1e-10);
    }

    #[test]
    fn test_metadata_serialization() {
        let params = TextureTrimsheetV1Params {
            resolution: [256, 256],
            padding: 2,
            tiles: vec![make_tile("test", 64, 64, [1.0, 0.0, 0.0, 1.0])],
        };

        let result = generate_trimsheet(&params, 42).unwrap();
        let json = serde_json::to_string_pretty(&result.metadata).unwrap();

        // Should be valid JSON
        let parsed: TrimsheetMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.atlas_width, result.metadata.atlas_width);
        assert_eq!(parsed.tiles.len(), result.metadata.tiles.len());
    }
}
