//! Spritesheet generation with deterministic shelf packing.
//!
//! This module implements a deterministic spritesheet generator that packs
//! sprite frames into an atlas. It reuses the shelf packing algorithm from
//! the trimsheet module for consistent, byte-identical output.
//!
//! Features:
//! - Deterministic shelf packing (frames sorted by height, then width, then id)
//! - Mip-safe gutters (edge pixels are replicated into padding)
//! - Frame metadata output with UVs and pivot points

use std::collections::HashMap;

use speccade_spec::recipe::sprite::{
    SpriteFrame, SpriteFrameSource, SpriteFrameUv, SpriteSheetMetadata, SpriteSheetV1Params,
};
use thiserror::Error;

use crate::color::Color;
use crate::maps::TextureBuffer;
use crate::png::{write_rgba_to_vec_with_hash, PngConfig};

/// Errors that can occur during spritesheet generation.
#[derive(Debug, Error)]
pub enum SpriteSheetError {
    /// Frame is too large to fit in the atlas.
    #[error("Frame '{0}' ({1}x{2}) with padding is too large for atlas ({3}x{4})")]
    FrameTooLarge(String, u32, u32, u32, u32),

    /// Not enough space to pack all frames.
    #[error(
        "Cannot fit all frames into atlas. Consider increasing resolution or reducing frame sizes"
    )]
    PackingFailed,

    /// Duplicate frame id.
    #[error("Duplicate frame id: '{0}'")]
    DuplicateFrameId(String),

    /// PNG encoding error.
    #[error("PNG encoding error: {0}")]
    PngError(#[from] crate::png::PngError),

    /// Invalid parameters.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Invalid pivot value.
    #[error("Invalid pivot for frame '{0}': values must be in [0, 1] range")]
    InvalidPivot(String),
}

/// Result of spritesheet generation.
#[derive(Debug)]
pub struct SpriteSheetResult {
    /// PNG-encoded atlas image data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
    /// Metadata with UV coordinates and pivot points for each frame.
    pub metadata: SpriteSheetMetadata,
}

/// Internal representation of a packed frame placement.
#[derive(Debug, Clone)]
struct PackedFrame {
    /// Frame id.
    id: String,
    /// X position in atlas (excluding padding).
    x: u32,
    /// Y position in atlas (excluding padding).
    y: u32,
    /// Frame width (without padding).
    width: u32,
    /// Frame height (without padding).
    height: u32,
    /// Pivot point in normalized coordinates.
    pivot: [f64; 2],
}

/// Shelf for shelf packing algorithm.
#[derive(Debug)]
struct Shelf {
    /// Y position of this shelf.
    y: u32,
    /// Height of this shelf (tallest frame + padding).
    height: u32,
    /// Current X position (next free spot).
    current_x: u32,
}

/// Generate a spritesheet atlas from parameters.
///
/// # Arguments
/// * `params` - Spritesheet parameters including resolution, padding, and frames
/// * `_seed` - Deterministic seed (reserved for future procedural frame sources)
///
/// # Returns
/// A `SpriteSheetResult` containing the PNG data, hash, and frame metadata.
pub fn generate_sprite_sheet(
    params: &SpriteSheetV1Params,
    _seed: u32,
) -> Result<SpriteSheetResult, SpriteSheetError> {
    let width = params.resolution[0];
    let height = params.resolution[1];
    let padding = params.padding;

    // Validate parameters
    if width == 0 || height == 0 {
        return Err(SpriteSheetError::InvalidParameter(
            "Atlas resolution must be non-zero".to_string(),
        ));
    }

    // Check for duplicate frame ids
    let mut seen_ids: HashMap<&str, bool> = HashMap::new();
    for frame in &params.frames {
        if seen_ids.contains_key(frame.id.as_str()) {
            return Err(SpriteSheetError::DuplicateFrameId(frame.id.clone()));
        }
        seen_ids.insert(&frame.id, true);
    }

    // Validate pivots and frame sizes
    for frame in &params.frames {
        // Validate pivot is in [0, 1] range
        if frame.pivot[0] < 0.0
            || frame.pivot[0] > 1.0
            || frame.pivot[1] < 0.0
            || frame.pivot[1] > 1.0
        {
            return Err(SpriteSheetError::InvalidPivot(frame.id.clone()));
        }

        // Validate frame fits in atlas
        let padded_width = frame.width + padding * 2;
        let padded_height = frame.height + padding * 2;
        if padded_width > width || padded_height > height {
            return Err(SpriteSheetError::FrameTooLarge(
                frame.id.clone(),
                frame.width,
                frame.height,
                width,
                height,
            ));
        }
    }

    // Pack frames using shelf algorithm
    let packed = pack_frames_shelf(&params.frames, width, height, padding)?;

    // Create atlas buffer
    let mut atlas = TextureBuffer::new(width, height, Color::rgba(0.0, 0.0, 0.0, 0.0));

    // Render each frame into the atlas
    for (frame, placement) in params.frames.iter().zip(packed.iter()) {
        render_frame_with_gutter(&mut atlas, frame, placement, padding);
    }

    // Encode to PNG
    let config = PngConfig::default();
    let (png_data, hash) = write_rgba_to_vec_with_hash(&atlas, &config)?;

    // Build metadata
    let frame_uvs: Vec<SpriteFrameUv> = packed
        .iter()
        .map(|p| {
            // UV coordinates are normalized [0, 1]
            // Note: UVs point to the inner frame content, not the gutter
            SpriteFrameUv {
                id: p.id.clone(),
                u_min: p.x as f64 / width as f64,
                v_min: p.y as f64 / height as f64,
                u_max: (p.x + p.width) as f64 / width as f64,
                v_max: (p.y + p.height) as f64 / height as f64,
                width: p.width,
                height: p.height,
                pivot: p.pivot,
            }
        })
        .collect();

    let metadata = SpriteSheetMetadata {
        atlas_width: width,
        atlas_height: height,
        padding,
        frames: frame_uvs,
    };

    Ok(SpriteSheetResult {
        png_data,
        hash,
        metadata,
    })
}

/// Pack frames using a deterministic shelf packing algorithm.
///
/// Frames are sorted by height (descending), then width (descending), then id
/// to ensure deterministic output. The algorithm places frames on shelves,
/// creating new shelves as needed.
fn pack_frames_shelf(
    frames: &[SpriteFrame],
    atlas_width: u32,
    atlas_height: u32,
    padding: u32,
) -> Result<Vec<PackedFrame>, SpriteSheetError> {
    if frames.is_empty() {
        return Ok(Vec::new());
    }

    // Create sorted indices for deterministic ordering
    // Sort by: height (desc), width (desc), id (asc)
    let mut indices: Vec<usize> = (0..frames.len()).collect();
    indices.sort_by(|&a, &b| {
        let frame_a = &frames[a];
        let frame_b = &frames[b];

        // Primary: height descending
        match frame_b.height.cmp(&frame_a.height) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // Secondary: width descending
        match frame_b.width.cmp(&frame_a.width) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // Tertiary: id ascending (for stable determinism)
        frame_a.id.cmp(&frame_b.id)
    });

    let mut shelves: Vec<Shelf> = Vec::new();
    let mut placements: Vec<Option<PackedFrame>> = vec![None; frames.len()];

    for &idx in &indices {
        let frame = &frames[idx];
        let padded_width = frame.width + padding * 2;
        let padded_height = frame.height + padding * 2;

        // Try to find a shelf that can accommodate this frame
        let mut placed = false;
        for shelf in &mut shelves {
            if shelf.current_x + padded_width <= atlas_width && padded_height <= shelf.height {
                // Place on this shelf
                placements[idx] = Some(PackedFrame {
                    id: frame.id.clone(),
                    x: shelf.current_x + padding,
                    y: shelf.y + padding,
                    width: frame.width,
                    height: frame.height,
                    pivot: frame.pivot,
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
                return Err(SpriteSheetError::PackingFailed);
            }

            let new_shelf = Shelf {
                y: shelf_y,
                height: padded_height,
                current_x: padded_width,
            };

            placements[idx] = Some(PackedFrame {
                id: frame.id.clone(),
                x: padding,
                y: shelf_y + padding,
                width: frame.width,
                height: frame.height,
                pivot: frame.pivot,
            });

            shelves.push(new_shelf);
        }
    }

    // Return placements in original order
    Ok(placements.into_iter().map(|p| p.unwrap()).collect())
}

/// Render a frame into the atlas with mip-safe gutter.
///
/// The gutter pixels are filled by replicating the edge pixels of the frame,
/// which prevents color bleeding during mipmap generation.
fn render_frame_with_gutter(
    atlas: &mut TextureBuffer,
    frame: &SpriteFrame,
    placement: &PackedFrame,
    padding: u32,
) {
    // Get frame color
    let color = match &frame.source {
        SpriteFrameSource::Color { color } => Color::rgba(color[0], color[1], color[2], color[3]),
        SpriteFrameSource::NodeRef { .. } => {
            // NodeRef is for future extension - use magenta placeholder
            Color::rgba(1.0, 0.0, 1.0, 1.0)
        }
    };

    // Fill the frame content area
    for y in 0..frame.height {
        for x in 0..frame.width {
            atlas.set(placement.x + x, placement.y + y, color);
        }
    }

    // Fill gutters by replicating edge pixels
    if padding > 0 {
        // For solid color frames, the edge color is the same everywhere
        // so we just fill the gutter regions with the same color

        // Top gutter
        for gy in 0..padding {
            for x in 0..frame.width {
                let gutter_x = placement.x + x;
                let gutter_y = placement.y.saturating_sub(padding) + gy;
                if gutter_y < atlas.height {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }

        // Bottom gutter
        for gy in 0..padding {
            for x in 0..frame.width {
                let gutter_x = placement.x + x;
                let gutter_y = placement.y + frame.height + gy;
                if gutter_y < atlas.height {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }

        // Left gutter
        for y in 0..frame.height {
            for gx in 0..padding {
                let gutter_x = placement.x.saturating_sub(padding) + gx;
                let gutter_y = placement.y + y;
                if gutter_x < atlas.width {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }

        // Right gutter
        for y in 0..frame.height {
            for gx in 0..padding {
                let gutter_x = placement.x + frame.width + gx;
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
                let gutter_x = placement.x + frame.width + gx;
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
                let gutter_y = placement.y + frame.height + gy;
                if gutter_x < atlas.width && gutter_y < atlas.height {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }

        // Bottom-right
        for gy in 0..padding {
            for gx in 0..padding {
                let gutter_x = placement.x + frame.width + gx;
                let gutter_y = placement.y + frame.height + gy;
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

    fn make_frame(id: &str, width: u32, height: u32, color: [f64; 4]) -> SpriteFrame {
        SpriteFrame {
            id: id.to_string(),
            width,
            height,
            pivot: [0.5, 0.5],
            source: SpriteFrameSource::Color { color },
        }
    }

    fn make_frame_with_pivot(
        id: &str,
        width: u32,
        height: u32,
        color: [f64; 4],
        pivot: [f64; 2],
    ) -> SpriteFrame {
        SpriteFrame {
            id: id.to_string(),
            width,
            height,
            pivot,
            source: SpriteFrameSource::Color { color },
        }
    }

    #[test]
    fn test_empty_frames() {
        let params = SpriteSheetV1Params {
            resolution: [256, 256],
            padding: 2,
            frames: vec![],
        };

        let result = generate_sprite_sheet(&params, 42).unwrap();
        assert_eq!(result.metadata.frames.len(), 0);
        assert!(!result.png_data.is_empty());
    }

    #[test]
    fn test_single_frame() {
        let params = SpriteSheetV1Params {
            resolution: [256, 256],
            padding: 2,
            frames: vec![make_frame("idle_0", 64, 64, [0.5, 0.5, 0.5, 1.0])],
        };

        let result = generate_sprite_sheet(&params, 42).unwrap();
        assert_eq!(result.metadata.frames.len(), 1);
        assert_eq!(result.metadata.frames[0].id, "idle_0");
        assert_eq!(result.metadata.frames[0].width, 64);
        assert_eq!(result.metadata.frames[0].height, 64);
        assert_eq!(result.metadata.frames[0].pivot, [0.5, 0.5]);
    }

    #[test]
    fn test_frame_with_custom_pivot() {
        let params = SpriteSheetV1Params {
            resolution: [256, 256],
            padding: 2,
            frames: vec![make_frame_with_pivot(
                "character",
                64,
                64,
                [1.0, 0.0, 0.0, 1.0],
                [0.5, 0.0], // Bottom-center pivot
            )],
        };

        let result = generate_sprite_sheet(&params, 42).unwrap();
        assert_eq!(result.metadata.frames[0].pivot, [0.5, 0.0]);
    }

    #[test]
    fn test_multiple_frames() {
        let params = SpriteSheetV1Params {
            resolution: [512, 512],
            padding: 2,
            frames: vec![
                make_frame("idle_0", 64, 64, [0.5, 0.5, 0.5, 1.0]),
                make_frame("idle_1", 64, 64, [0.6, 0.6, 0.6, 1.0]),
                make_frame("walk_0", 64, 96, [0.7, 0.7, 0.7, 1.0]),
            ],
        };

        let result = generate_sprite_sheet(&params, 42).unwrap();
        assert_eq!(result.metadata.frames.len(), 3);

        // Verify all frames have unique positions
        let positions: Vec<_> = result
            .metadata
            .frames
            .iter()
            .map(|f| (f.u_min, f.v_min))
            .collect();
        for (i, p1) in positions.iter().enumerate() {
            for (j, p2) in positions.iter().enumerate() {
                if i != j {
                    assert!(
                        p1 != p2,
                        "Frames {} and {} have same position",
                        result.metadata.frames[i].id,
                        result.metadata.frames[j].id
                    );
                }
            }
        }
    }

    #[test]
    fn test_determinism() {
        let params = SpriteSheetV1Params {
            resolution: [256, 256],
            padding: 2,
            frames: vec![
                make_frame("a", 64, 64, [1.0, 0.0, 0.0, 1.0]),
                make_frame("b", 32, 32, [0.0, 1.0, 0.0, 1.0]),
                make_frame("c", 48, 48, [0.0, 0.0, 1.0, 1.0]),
            ],
        };

        let result1 = generate_sprite_sheet(&params, 42).unwrap();
        let result2 = generate_sprite_sheet(&params, 42).unwrap();

        // PNG data should be byte-identical
        assert_eq!(result1.png_data, result2.png_data);
        assert_eq!(result1.hash, result2.hash);

        // Metadata should be identical
        assert_eq!(result1.metadata.frames.len(), result2.metadata.frames.len());
        for (f1, f2) in result1
            .metadata
            .frames
            .iter()
            .zip(result2.metadata.frames.iter())
        {
            assert_eq!(f1.id, f2.id);
            assert!((f1.u_min - f2.u_min).abs() < 1e-10);
            assert!((f1.v_min - f2.v_min).abs() < 1e-10);
            assert!((f1.u_max - f2.u_max).abs() < 1e-10);
            assert!((f1.v_max - f2.v_max).abs() < 1e-10);
            assert_eq!(f1.pivot, f2.pivot);
        }
    }

    #[test]
    fn test_frame_too_large() {
        let params = SpriteSheetV1Params {
            resolution: [64, 64],
            padding: 2,
            frames: vec![make_frame("huge", 128, 128, [1.0, 1.0, 1.0, 1.0])],
        };

        let err = generate_sprite_sheet(&params, 42).unwrap_err();
        assert!(matches!(err, SpriteSheetError::FrameTooLarge(..)));
    }

    #[test]
    fn test_duplicate_frame_id() {
        let params = SpriteSheetV1Params {
            resolution: [256, 256],
            padding: 2,
            frames: vec![
                make_frame("same", 32, 32, [1.0, 0.0, 0.0, 1.0]),
                make_frame("same", 64, 64, [0.0, 1.0, 0.0, 1.0]),
            ],
        };

        let err = generate_sprite_sheet(&params, 42).unwrap_err();
        assert!(matches!(err, SpriteSheetError::DuplicateFrameId(..)));
    }

    #[test]
    fn test_invalid_pivot_negative() {
        let params = SpriteSheetV1Params {
            resolution: [256, 256],
            padding: 2,
            frames: vec![make_frame_with_pivot(
                "bad",
                32,
                32,
                [1.0, 0.0, 0.0, 1.0],
                [-0.1, 0.5],
            )],
        };

        let err = generate_sprite_sheet(&params, 42).unwrap_err();
        assert!(matches!(err, SpriteSheetError::InvalidPivot(..)));
    }

    #[test]
    fn test_invalid_pivot_too_large() {
        let params = SpriteSheetV1Params {
            resolution: [256, 256],
            padding: 2,
            frames: vec![make_frame_with_pivot(
                "bad",
                32,
                32,
                [1.0, 0.0, 0.0, 1.0],
                [0.5, 1.5],
            )],
        };

        let err = generate_sprite_sheet(&params, 42).unwrap_err();
        assert!(matches!(err, SpriteSheetError::InvalidPivot(..)));
    }

    #[test]
    fn test_packing_failed() {
        // Try to pack too many frames in a small atlas
        let params = SpriteSheetV1Params {
            resolution: [64, 64],
            padding: 0,
            frames: vec![
                make_frame("a", 32, 32, [1.0, 0.0, 0.0, 1.0]),
                make_frame("b", 32, 32, [0.0, 1.0, 0.0, 1.0]),
                make_frame("c", 32, 32, [0.0, 0.0, 1.0, 1.0]),
                make_frame("d", 32, 32, [1.0, 1.0, 0.0, 1.0]),
                make_frame("e", 32, 32, [1.0, 0.0, 1.0, 1.0]), // This won't fit
            ],
        };

        let err = generate_sprite_sheet(&params, 42).unwrap_err();
        assert!(matches!(err, SpriteSheetError::PackingFailed));
    }

    #[test]
    fn test_uv_coordinates_normalized() {
        let params = SpriteSheetV1Params {
            resolution: [256, 256],
            padding: 0,
            frames: vec![make_frame("test", 128, 128, [1.0, 1.0, 1.0, 1.0])],
        };

        let result = generate_sprite_sheet(&params, 42).unwrap();
        let uv = &result.metadata.frames[0];

        // UVs should be in [0, 1] range
        assert!(uv.u_min >= 0.0 && uv.u_min <= 1.0);
        assert!(uv.v_min >= 0.0 && uv.v_min <= 1.0);
        assert!(uv.u_max >= 0.0 && uv.u_max <= 1.0);
        assert!(uv.v_max >= 0.0 && uv.v_max <= 1.0);

        // For 128x128 frame in 256x256 atlas, UV range should be 0.5
        assert!((uv.u_max - uv.u_min - 0.5).abs() < 1e-10);
        assert!((uv.v_max - uv.v_min - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_shelf_packing_order() {
        // Verify that frames are sorted by height for shelf packing
        let params = SpriteSheetV1Params {
            resolution: [512, 512],
            padding: 0,
            frames: vec![
                make_frame("small", 32, 32, [1.0, 0.0, 0.0, 1.0]),
                make_frame("tall", 32, 128, [0.0, 1.0, 0.0, 1.0]),
                make_frame("medium", 32, 64, [0.0, 0.0, 1.0, 1.0]),
            ],
        };

        let result = generate_sprite_sheet(&params, 42).unwrap();

        // The tall frame should be on the first shelf (lowest y)
        let tall = result
            .metadata
            .frames
            .iter()
            .find(|f| f.id == "tall")
            .unwrap();
        let medium = result
            .metadata
            .frames
            .iter()
            .find(|f| f.id == "medium")
            .unwrap();
        let small = result
            .metadata
            .frames
            .iter()
            .find(|f| f.id == "small")
            .unwrap();

        // Tall should have the smallest v_min (placed first)
        assert!(tall.v_min <= medium.v_min);
        assert!(tall.v_min <= small.v_min);
    }

    #[test]
    fn test_zero_padding() {
        let params = SpriteSheetV1Params {
            resolution: [128, 128],
            padding: 0,
            frames: vec![make_frame("test", 64, 64, [0.5, 0.5, 0.5, 1.0])],
        };

        let result = generate_sprite_sheet(&params, 42).unwrap();
        assert_eq!(result.metadata.padding, 0);

        // UV should start at (0, 0) with no padding
        let uv = &result.metadata.frames[0];
        assert!((uv.u_min).abs() < 1e-10);
        assert!((uv.v_min).abs() < 1e-10);
    }

    #[test]
    fn test_metadata_serialization() {
        let params = SpriteSheetV1Params {
            resolution: [256, 256],
            padding: 2,
            frames: vec![make_frame_with_pivot(
                "test",
                64,
                64,
                [1.0, 0.0, 0.0, 1.0],
                [0.5, 0.0],
            )],
        };

        let result = generate_sprite_sheet(&params, 42).unwrap();
        let json = serde_json::to_string_pretty(&result.metadata).unwrap();

        // Should be valid JSON
        let parsed: SpriteSheetMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.atlas_width, result.metadata.atlas_width);
        assert_eq!(parsed.frames.len(), result.metadata.frames.len());
        assert_eq!(parsed.frames[0].pivot, [0.5, 0.0]);
    }

    #[test]
    fn test_pivot_edge_values() {
        // Test that pivots at edges (0, 0), (1, 1) are valid
        let params = SpriteSheetV1Params {
            resolution: [256, 256],
            padding: 2,
            frames: vec![
                make_frame_with_pivot("tl", 32, 32, [1.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
                make_frame_with_pivot("br", 32, 32, [0.0, 1.0, 0.0, 1.0], [1.0, 1.0]),
                make_frame_with_pivot("center", 32, 32, [0.0, 0.0, 1.0, 1.0], [0.5, 0.5]),
            ],
        };

        let result = generate_sprite_sheet(&params, 42).unwrap();
        assert_eq!(result.metadata.frames.len(), 3);

        let tl = result
            .metadata
            .frames
            .iter()
            .find(|f| f.id == "tl")
            .unwrap();
        let br = result
            .metadata
            .frames
            .iter()
            .find(|f| f.id == "br")
            .unwrap();
        let center = result
            .metadata
            .frames
            .iter()
            .find(|f| f.id == "center")
            .unwrap();

        assert_eq!(tl.pivot, [0.0, 0.0]);
        assert_eq!(br.pivot, [1.0, 1.0]);
        assert_eq!(center.pivot, [0.5, 0.5]);
    }
}
