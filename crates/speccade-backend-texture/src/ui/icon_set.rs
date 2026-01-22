//! Icon set generation with deterministic shelf packing.
//!
//! This module implements an icon set generator that packs icon frames
//! into an atlas using the same deterministic shelf packing algorithm
//! as the spritesheet generator.

use std::collections::HashMap;

use speccade_spec::recipe::ui::{IconEntry, IconSetMetadata, IconUv, UiIconSetV1Params};
use thiserror::Error;

use super::gutter::{render_region_with_gutter, validate_color};
use crate::color::Color;
use crate::maps::TextureBuffer;
use crate::png::{write_rgba_to_vec_with_hash, PngConfig};

/// Errors that can occur during icon set generation.
#[derive(Debug, Error)]
pub enum IconSetError {
    /// Icon is too large to fit in the atlas.
    #[error("Icon '{0}' ({1}x{2}) with padding is too large for atlas ({3}x{4})")]
    IconTooLarge(String, u32, u32, u32, u32),

    /// Not enough space to pack all icons.
    #[error(
        "Cannot fit all icons into atlas. Consider increasing resolution or reducing icon sizes"
    )]
    PackingFailed,

    /// Duplicate icon id.
    #[error("Duplicate icon id: '{0}'")]
    DuplicateIconId(String),

    /// PNG encoding error.
    #[error("PNG encoding error: {0}")]
    PngError(#[from] crate::png::PngError),

    /// Invalid parameters.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Invalid color value.
    #[error("{0}")]
    InvalidColor(String),
}

/// Result of icon set generation.
#[derive(Debug)]
pub struct IconSetResult {
    /// PNG-encoded atlas image data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
    /// Metadata with UV coordinates for each icon.
    pub metadata: IconSetMetadata,
}

/// Internal representation of a packed icon placement.
#[derive(Debug, Clone)]
struct PackedIcon {
    /// Icon id.
    id: String,
    /// X position in atlas (excluding padding).
    x: u32,
    /// Y position in atlas (excluding padding).
    y: u32,
    /// Icon width (without padding).
    width: u32,
    /// Icon height (without padding).
    height: u32,
    /// Optional category.
    category: Option<String>,
}

/// Shelf for shelf packing algorithm.
#[derive(Debug)]
struct Shelf {
    /// Y position of this shelf.
    y: u32,
    /// Height of this shelf (tallest icon + padding).
    height: u32,
    /// Current X position (next free spot).
    current_x: u32,
}

/// Generate an icon set atlas from parameters.
///
/// # Arguments
/// * `params` - Icon set parameters including resolution, padding, and icon entries
/// * `_seed` - Deterministic seed (reserved for future procedural icon sources)
///
/// # Returns
/// An `IconSetResult` containing the PNG data, hash, and icon metadata.
pub fn generate_icon_set(
    params: &UiIconSetV1Params,
    _seed: u32,
) -> Result<IconSetResult, IconSetError> {
    let width = params.resolution[0];
    let height = params.resolution[1];
    let padding = params.padding;

    // Validate parameters
    if width == 0 || height == 0 {
        return Err(IconSetError::InvalidParameter(
            "Atlas resolution must be non-zero".to_string(),
        ));
    }

    // Check for duplicate icon ids
    let mut seen_ids: HashMap<&str, bool> = HashMap::new();
    for icon in &params.icons {
        if seen_ids.contains_key(icon.id.as_str()) {
            return Err(IconSetError::DuplicateIconId(icon.id.clone()));
        }
        seen_ids.insert(&icon.id, true);
    }

    // Validate icon sizes and colors
    for icon in &params.icons {
        // Validate color is in [0, 1] range
        validate_color(&icon.color, &format!("icon '{}'", icon.id))
            .map_err(IconSetError::InvalidColor)?;

        // Validate icon fits in atlas
        let padded_width = icon.width + padding * 2;
        let padded_height = icon.height + padding * 2;
        if padded_width > width || padded_height > height {
            return Err(IconSetError::IconTooLarge(
                icon.id.clone(),
                icon.width,
                icon.height,
                width,
                height,
            ));
        }
    }

    // Pack icons using shelf algorithm
    let packed = pack_icons_shelf(&params.icons, width, height, padding)?;

    // Create atlas buffer
    let mut atlas = TextureBuffer::new(width, height, Color::rgba(0.0, 0.0, 0.0, 0.0));

    // Render each icon into the atlas
    for (icon, placement) in params.icons.iter().zip(packed.iter()) {
        let color = Color::rgba(icon.color[0], icon.color[1], icon.color[2], icon.color[3]);
        render_region_with_gutter(
            &mut atlas,
            placement.x,
            placement.y,
            icon.width,
            icon.height,
            color,
            padding,
        );
    }

    // Encode to PNG
    let config = PngConfig::default();
    let (png_data, hash) = write_rgba_to_vec_with_hash(&atlas, &config)?;

    // Build metadata
    let icon_uvs: Vec<IconUv> = packed
        .iter()
        .map(|p| IconUv {
            id: p.id.clone(),
            u_min: p.x as f64 / width as f64,
            v_min: p.y as f64 / height as f64,
            u_max: (p.x + p.width) as f64 / width as f64,
            v_max: (p.y + p.height) as f64 / height as f64,
            width: p.width,
            height: p.height,
            category: p.category.clone(),
        })
        .collect();

    let metadata = IconSetMetadata {
        atlas_width: width,
        atlas_height: height,
        padding,
        icons: icon_uvs,
    };

    Ok(IconSetResult {
        png_data,
        hash,
        metadata,
    })
}

/// Pack icons using a deterministic shelf packing algorithm.
///
/// Icons are sorted by height (descending), then width (descending), then id
/// to ensure deterministic output.
fn pack_icons_shelf(
    icons: &[IconEntry],
    atlas_width: u32,
    atlas_height: u32,
    padding: u32,
) -> Result<Vec<PackedIcon>, IconSetError> {
    if icons.is_empty() {
        return Ok(Vec::new());
    }

    // Create sorted indices for deterministic ordering
    // Sort by: height (desc), width (desc), id (asc)
    let mut indices: Vec<usize> = (0..icons.len()).collect();
    indices.sort_by(|&a, &b| {
        let icon_a = &icons[a];
        let icon_b = &icons[b];

        // Primary: height descending
        match icon_b.height.cmp(&icon_a.height) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // Secondary: width descending
        match icon_b.width.cmp(&icon_a.width) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // Tertiary: id ascending (for stable determinism)
        icon_a.id.cmp(&icon_b.id)
    });

    let mut shelves: Vec<Shelf> = Vec::new();
    let mut placements: Vec<Option<PackedIcon>> = vec![None; icons.len()];

    for &idx in &indices {
        let icon = &icons[idx];
        let padded_width = icon.width + padding * 2;
        let padded_height = icon.height + padding * 2;

        // Try to find a shelf that can accommodate this icon
        let mut placed = false;
        for shelf in &mut shelves {
            if shelf.current_x + padded_width <= atlas_width && padded_height <= shelf.height {
                // Place on this shelf
                placements[idx] = Some(PackedIcon {
                    id: icon.id.clone(),
                    x: shelf.current_x + padding,
                    y: shelf.y + padding,
                    width: icon.width,
                    height: icon.height,
                    category: icon.category.clone(),
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
                return Err(IconSetError::PackingFailed);
            }

            let new_shelf = Shelf {
                y: shelf_y,
                height: padded_height,
                current_x: padded_width,
            };

            placements[idx] = Some(PackedIcon {
                id: icon.id.clone(),
                x: padding,
                y: shelf_y + padding,
                width: icon.width,
                height: icon.height,
                category: icon.category.clone(),
            });

            shelves.push(new_shelf);
        }
    }

    // Return placements in original order
    Ok(placements.into_iter().map(|p| p.unwrap()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_icons() {
        let params = UiIconSetV1Params::new(256, 256);
        let result = generate_icon_set(&params, 42).unwrap();

        assert_eq!(result.metadata.icons.len(), 0);
        assert!(!result.png_data.is_empty());
    }

    #[test]
    fn test_single_icon() {
        let params = UiIconSetV1Params::new(256, 256).with_icon(IconEntry::new(
            "test",
            64,
            64,
            [0.5, 0.5, 0.5, 1.0],
        ));

        let result = generate_icon_set(&params, 42).unwrap();
        assert_eq!(result.metadata.icons.len(), 1);
        assert_eq!(result.metadata.icons[0].id, "test");
        assert_eq!(result.metadata.icons[0].width, 64);
        assert_eq!(result.metadata.icons[0].height, 64);
    }

    #[test]
    fn test_multiple_icons() {
        let params = UiIconSetV1Params::new(512, 512)
            .with_icon(IconEntry::new("close", 32, 32, [1.0, 0.0, 0.0, 1.0]))
            .with_icon(IconEntry::new("settings", 48, 48, [0.5, 0.5, 0.5, 1.0]))
            .with_icon(IconEntry::new("heart", 24, 24, [1.0, 0.0, 0.0, 1.0]));

        let result = generate_icon_set(&params, 42).unwrap();
        assert_eq!(result.metadata.icons.len(), 3);

        // Verify all icons have unique positions
        let positions: Vec<_> = result
            .metadata
            .icons
            .iter()
            .map(|i| (i.u_min, i.v_min))
            .collect();
        for (i, p1) in positions.iter().enumerate() {
            for (j, p2) in positions.iter().enumerate() {
                if i != j {
                    assert!(
                        p1 != p2,
                        "Icons {} and {} have same position",
                        result.metadata.icons[i].id,
                        result.metadata.icons[j].id
                    );
                }
            }
        }
    }

    #[test]
    fn test_icon_with_category() {
        let params = UiIconSetV1Params::new(256, 256).with_icon(
            IconEntry::new("settings", 32, 32, [0.5, 0.5, 0.5, 1.0]).with_category("system"),
        );

        let result = generate_icon_set(&params, 42).unwrap();
        assert_eq!(result.metadata.icons[0].category.as_deref(), Some("system"));
    }

    #[test]
    fn test_determinism() {
        let params = UiIconSetV1Params::new(256, 256)
            .with_icon(IconEntry::new("a", 64, 64, [1.0, 0.0, 0.0, 1.0]))
            .with_icon(IconEntry::new("b", 32, 32, [0.0, 1.0, 0.0, 1.0]))
            .with_icon(IconEntry::new("c", 48, 48, [0.0, 0.0, 1.0, 1.0]));

        let result1 = generate_icon_set(&params, 42).unwrap();
        let result2 = generate_icon_set(&params, 42).unwrap();

        // PNG data should be byte-identical
        assert_eq!(result1.png_data, result2.png_data);
        assert_eq!(result1.hash, result2.hash);

        // Metadata should be identical
        assert_eq!(result1.metadata.icons.len(), result2.metadata.icons.len());
        for (i1, i2) in result1
            .metadata
            .icons
            .iter()
            .zip(result2.metadata.icons.iter())
        {
            assert_eq!(i1.id, i2.id);
            assert!((i1.u_min - i2.u_min).abs() < 1e-10);
            assert!((i1.v_min - i2.v_min).abs() < 1e-10);
            assert!((i1.u_max - i2.u_max).abs() < 1e-10);
            assert!((i1.v_max - i2.v_max).abs() < 1e-10);
        }
    }

    #[test]
    fn test_icon_too_large() {
        let params = UiIconSetV1Params::new(64, 64).with_icon(IconEntry::new(
            "huge",
            128,
            128,
            [1.0, 1.0, 1.0, 1.0],
        ));

        let err = generate_icon_set(&params, 42).unwrap_err();
        assert!(matches!(err, IconSetError::IconTooLarge(..)));
    }

    #[test]
    fn test_duplicate_icon_id() {
        let params = UiIconSetV1Params::new(256, 256)
            .with_icon(IconEntry::new("same", 32, 32, [1.0, 0.0, 0.0, 1.0]))
            .with_icon(IconEntry::new("same", 64, 64, [0.0, 1.0, 0.0, 1.0]));

        let err = generate_icon_set(&params, 42).unwrap_err();
        assert!(matches!(err, IconSetError::DuplicateIconId(..)));
    }

    #[test]
    fn test_invalid_color() {
        let params = UiIconSetV1Params::new(256, 256).with_icon(IconEntry::new(
            "bad",
            32,
            32,
            [1.5, 0.0, 0.0, 1.0],
        ));

        let err = generate_icon_set(&params, 42).unwrap_err();
        assert!(matches!(err, IconSetError::InvalidColor(..)));
    }

    #[test]
    fn test_packing_failed() {
        // Try to pack too many icons in a small atlas
        let params = UiIconSetV1Params::new(64, 64)
            .with_padding(0)
            .with_icon(IconEntry::new("a", 32, 32, [1.0, 0.0, 0.0, 1.0]))
            .with_icon(IconEntry::new("b", 32, 32, [0.0, 1.0, 0.0, 1.0]))
            .with_icon(IconEntry::new("c", 32, 32, [0.0, 0.0, 1.0, 1.0]))
            .with_icon(IconEntry::new("d", 32, 32, [1.0, 1.0, 0.0, 1.0]))
            .with_icon(IconEntry::new("e", 32, 32, [1.0, 0.0, 1.0, 1.0]));

        let err = generate_icon_set(&params, 42).unwrap_err();
        assert!(matches!(err, IconSetError::PackingFailed));
    }

    #[test]
    fn test_uv_coordinates_normalized() {
        let params = UiIconSetV1Params::new(256, 256).with_icon(IconEntry::new(
            "test",
            128,
            128,
            [1.0, 1.0, 1.0, 1.0],
        ));

        let result = generate_icon_set(&params, 42).unwrap();
        let uv = &result.metadata.icons[0];

        // UVs should be in [0, 1] range
        assert!(uv.u_min >= 0.0 && uv.u_min <= 1.0);
        assert!(uv.v_min >= 0.0 && uv.v_min <= 1.0);
        assert!(uv.u_max >= 0.0 && uv.u_max <= 1.0);
        assert!(uv.v_max >= 0.0 && uv.v_max <= 1.0);
    }

    #[test]
    fn test_shelf_packing_order() {
        // Verify that icons are sorted by height for shelf packing
        let params = UiIconSetV1Params::new(512, 512)
            .with_padding(0)
            .with_icon(IconEntry::new("small", 32, 32, [1.0, 0.0, 0.0, 1.0]))
            .with_icon(IconEntry::new("tall", 32, 128, [0.0, 1.0, 0.0, 1.0]))
            .with_icon(IconEntry::new("medium", 32, 64, [0.0, 0.0, 1.0, 1.0]));

        let result = generate_icon_set(&params, 42).unwrap();

        // The tall icon should be on the first shelf (lowest y)
        let tall = result
            .metadata
            .icons
            .iter()
            .find(|i| i.id == "tall")
            .unwrap();
        let medium = result
            .metadata
            .icons
            .iter()
            .find(|i| i.id == "medium")
            .unwrap();
        let small = result
            .metadata
            .icons
            .iter()
            .find(|i| i.id == "small")
            .unwrap();

        // Tall should have the smallest v_min (placed first)
        assert!(tall.v_min <= medium.v_min);
        assert!(tall.v_min <= small.v_min);
    }
}
