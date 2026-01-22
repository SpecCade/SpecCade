//! Nine-slice panel generation with deterministic packing.
//!
//! This module implements a nine-slice panel generator that packs
//! corners, edges, and center regions into an atlas with UV metadata.

use speccade_spec::recipe::ui::{
    NineSliceMetadata, NineSliceUvRegions, UiNineSliceV1Params, UvRect,
};
use thiserror::Error;

use super::gutter::{render_region_with_gutter, validate_color};
use crate::color::Color;
use crate::maps::TextureBuffer;
use crate::png::{write_rgba_to_vec_with_hash, PngConfig};

/// Errors that can occur during nine-slice generation.
#[derive(Debug, Error)]
pub enum NineSliceError {
    /// Invalid parameters.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Region too large to fit in atlas.
    #[error("Nine-slice regions ({0}x{1}) with padding are too large for atlas ({2}x{3})")]
    RegionsTooLarge(u32, u32, u32, u32),

    /// PNG encoding error.
    #[error("PNG encoding error: {0}")]
    PngError(#[from] crate::png::PngError),

    /// Invalid color value.
    #[error("{0}")]
    InvalidColor(String),
}

/// Result of nine-slice generation.
#[derive(Debug)]
pub struct NineSliceResult {
    /// PNG-encoded atlas image data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
    /// Metadata with UV coordinates for each region.
    pub metadata: NineSliceMetadata,
}

/// Generate a nine-slice panel atlas from parameters.
///
/// # Arguments
/// * `params` - Nine-slice parameters including resolution, padding, and region colors
/// * `_seed` - Deterministic seed (reserved for future procedural region sources)
///
/// # Returns
/// A `NineSliceResult` containing the PNG data, hash, and region metadata.
pub fn generate_nine_slice(
    params: &UiNineSliceV1Params,
    _seed: u32,
) -> Result<NineSliceResult, NineSliceError> {
    let width = params.resolution[0];
    let height = params.resolution[1];
    let padding = params.padding;

    // Validate parameters
    if width == 0 || height == 0 {
        return Err(NineSliceError::InvalidParameter(
            "Atlas resolution must be non-zero".to_string(),
        ));
    }

    let corner_w = params.regions.corner_size[0];
    let corner_h = params.regions.corner_size[1];
    let edge_w = params.regions.get_edge_width();
    let edge_h = params.regions.get_edge_height();

    // Validate that regions fit in atlas
    let min_width = corner_w * 2 + edge_w + padding * 4;
    let min_height = corner_h * 2 + edge_h + padding * 4;

    if min_width > width || min_height > height {
        return Err(NineSliceError::RegionsTooLarge(
            min_width, min_height, width, height,
        ));
    }

    // Validate color ranges
    validate_color(&params.regions.top_left, "top_left").map_err(NineSliceError::InvalidColor)?;
    validate_color(&params.regions.top_right, "top_right").map_err(NineSliceError::InvalidColor)?;
    validate_color(&params.regions.bottom_left, "bottom_left")
        .map_err(NineSliceError::InvalidColor)?;
    validate_color(&params.regions.bottom_right, "bottom_right")
        .map_err(NineSliceError::InvalidColor)?;
    validate_color(&params.regions.top_edge, "top_edge").map_err(NineSliceError::InvalidColor)?;
    validate_color(&params.regions.bottom_edge, "bottom_edge")
        .map_err(NineSliceError::InvalidColor)?;
    validate_color(&params.regions.left_edge, "left_edge").map_err(NineSliceError::InvalidColor)?;
    validate_color(&params.regions.right_edge, "right_edge")
        .map_err(NineSliceError::InvalidColor)?;
    validate_color(&params.regions.center, "center").map_err(NineSliceError::InvalidColor)?;

    if let Some(bg) = params.background_color {
        validate_color(&bg, "background_color").map_err(NineSliceError::InvalidColor)?;
    }

    // Create atlas buffer
    let bg_color = params
        .background_color
        .map(|c| Color::rgba(c[0], c[1], c[2], c[3]))
        .unwrap_or_else(|| Color::rgba(0.0, 0.0, 0.0, 0.0));

    let mut atlas = TextureBuffer::new(width, height, bg_color);

    // Calculate region positions (deterministic shelf layout)
    // Layout: top-left, top-edge, top-right on first row
    //         left-edge, center, right-edge on second row
    //         bottom-left, bottom-edge, bottom-right on third row

    let row1_y = padding;
    let row2_y = padding + corner_h + padding;
    let row3_y = padding + corner_h + padding + edge_h + padding;

    let col1_x = padding;
    let col2_x = padding + corner_w + padding;
    let col3_x = padding + corner_w + padding + edge_w + padding;

    // Render regions with gutters
    let regions = [
        // Top-left corner
        (
            "top_left",
            col1_x,
            row1_y,
            corner_w,
            corner_h,
            &params.regions.top_left,
        ),
        // Top-right corner
        (
            "top_right",
            col3_x,
            row1_y,
            corner_w,
            corner_h,
            &params.regions.top_right,
        ),
        // Bottom-left corner
        (
            "bottom_left",
            col1_x,
            row3_y,
            corner_w,
            corner_h,
            &params.regions.bottom_left,
        ),
        // Bottom-right corner
        (
            "bottom_right",
            col3_x,
            row3_y,
            corner_w,
            corner_h,
            &params.regions.bottom_right,
        ),
        // Top edge
        (
            "top_edge",
            col2_x,
            row1_y,
            edge_w,
            edge_h,
            &params.regions.top_edge,
        ),
        // Bottom edge
        (
            "bottom_edge",
            col2_x,
            row3_y,
            edge_w,
            edge_h,
            &params.regions.bottom_edge,
        ),
        // Left edge
        (
            "left_edge",
            col1_x,
            row2_y,
            edge_w,
            edge_h,
            &params.regions.left_edge,
        ),
        // Right edge
        (
            "right_edge",
            col3_x,
            row2_y,
            edge_w,
            edge_h,
            &params.regions.right_edge,
        ),
        // Center
        (
            "center",
            col2_x,
            row2_y,
            edge_w,
            edge_h,
            &params.regions.center,
        ),
    ];

    for (_, x, y, w, h, color_rgba) in &regions {
        let color = Color::rgba(color_rgba[0], color_rgba[1], color_rgba[2], color_rgba[3]);
        render_region_with_gutter(&mut atlas, *x, *y, *w, *h, color, padding);
    }

    // Encode to PNG
    let config = PngConfig::default();
    let (png_data, hash) = write_rgba_to_vec_with_hash(&atlas, &config)?;

    // Build metadata with UV coordinates
    let uv_regions = NineSliceUvRegions {
        top_left: UvRect::from_pixels(col1_x, row1_y, corner_w, corner_h, width, height),
        top_right: UvRect::from_pixels(col3_x, row1_y, corner_w, corner_h, width, height),
        bottom_left: UvRect::from_pixels(col1_x, row3_y, corner_w, corner_h, width, height),
        bottom_right: UvRect::from_pixels(col3_x, row3_y, corner_w, corner_h, width, height),
        top_edge: UvRect::from_pixels(col2_x, row1_y, edge_w, edge_h, width, height),
        bottom_edge: UvRect::from_pixels(col2_x, row3_y, edge_w, edge_h, width, height),
        left_edge: UvRect::from_pixels(col1_x, row2_y, edge_w, edge_h, width, height),
        right_edge: UvRect::from_pixels(col3_x, row2_y, edge_w, edge_h, width, height),
        center: UvRect::from_pixels(col2_x, row2_y, edge_w, edge_h, width, height),
    };

    let metadata = NineSliceMetadata {
        atlas_width: width,
        atlas_height: height,
        padding,
        regions: uv_regions,
    };

    Ok(NineSliceResult {
        png_data,
        hash,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_nine_slice() {
        let params = UiNineSliceV1Params::new(256, 256, 16, 16);
        let result = generate_nine_slice(&params, 42).unwrap();

        assert!(!result.png_data.is_empty());
        assert_eq!(result.metadata.atlas_width, 256);
        assert_eq!(result.metadata.atlas_height, 256);
        assert_eq!(result.metadata.padding, 2);
    }

    #[test]
    fn test_nine_slice_with_colors() {
        let mut params = UiNineSliceV1Params::new(256, 256, 16, 16);
        params.regions.top_left = [1.0, 0.0, 0.0, 1.0];
        params.regions.center = [0.5, 0.5, 0.5, 1.0];

        let result = generate_nine_slice(&params, 42).unwrap();
        assert!(!result.png_data.is_empty());
    }

    #[test]
    fn test_nine_slice_determinism() {
        let params = UiNineSliceV1Params::new(256, 256, 16, 16);
        let result1 = generate_nine_slice(&params, 42).unwrap();
        let result2 = generate_nine_slice(&params, 42).unwrap();

        assert_eq!(result1.png_data, result2.png_data);
        assert_eq!(result1.hash, result2.hash);
    }

    #[test]
    fn test_nine_slice_regions_too_large() {
        let params = UiNineSliceV1Params::new(64, 64, 50, 50);
        let err = generate_nine_slice(&params, 42).unwrap_err();
        assert!(matches!(err, NineSliceError::RegionsTooLarge(..)));
    }

    #[test]
    fn test_nine_slice_invalid_color() {
        let mut params = UiNineSliceV1Params::new(256, 256, 16, 16);
        params.regions.top_left = [1.5, 0.0, 0.0, 1.0];

        let err = generate_nine_slice(&params, 42).unwrap_err();
        assert!(matches!(err, NineSliceError::InvalidColor(..)));
    }

    #[test]
    fn test_nine_slice_uv_coordinates() {
        let params = UiNineSliceV1Params::new(256, 256, 16, 16).with_padding(0);
        let result = generate_nine_slice(&params, 42).unwrap();

        let uv = &result.metadata.regions.top_left;
        assert!(uv.u_min >= 0.0 && uv.u_min <= 1.0);
        assert!(uv.v_min >= 0.0 && uv.v_min <= 1.0);
        assert!(uv.u_max >= 0.0 && uv.u_max <= 1.0);
        assert!(uv.v_max >= 0.0 && uv.v_max <= 1.0);
        assert!(uv.u_max > uv.u_min);
        assert!(uv.v_max > uv.v_min);
    }

    #[test]
    fn test_nine_slice_with_background() {
        let params =
            UiNineSliceV1Params::new(256, 256, 16, 16).with_background([0.1, 0.1, 0.1, 1.0]);

        let result = generate_nine_slice(&params, 42).unwrap();
        assert!(!result.png_data.is_empty());
    }

    #[test]
    fn test_nine_slice_custom_edge_sizes() {
        let mut params = UiNineSliceV1Params::new(256, 256, 16, 16);
        params.regions.edge_width = Some(8);
        params.regions.edge_height = Some(12);

        let result = generate_nine_slice(&params, 42).unwrap();
        assert!(!result.png_data.is_empty());

        assert_eq!(result.metadata.regions.left_edge.width, 8);
        assert_eq!(result.metadata.regions.top_edge.height, 12);
    }
}
