//! Nine-slice panel recipe types for `ui.nine_slice_v1`.
//!
//! Nine-slice scaling divides a panel into 9 regions (corners, edges, center)
//! to enable scalable UI elements that preserve corner detail.

use serde::{Deserialize, Serialize};

use super::DEFAULT_UI_PADDING;

/// Parameters for the `ui.nine_slice_v1` recipe.
///
/// Generates a nine-slice panel texture with corner/edge/center regions.
/// The panel is packed into an atlas with region metadata for runtime scaling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiNineSliceV1Params {
    /// Atlas resolution [width, height] in pixels.
    /// Must be large enough to contain all nine regions with padding.
    pub resolution: [u32; 2],

    /// Padding/gutter in pixels between regions (for mip-safe borders).
    #[serde(default = "default_ui_padding")]
    pub padding: u32,

    /// Nine-slice region definitions.
    pub regions: NineSliceRegions,

    /// Optional background fill color (RGBA, 0.0-1.0).
    /// If specified, fills the atlas background before rendering regions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<[f64; 4]>,
}

fn default_ui_padding() -> u32 {
    DEFAULT_UI_PADDING
}

/// Nine-slice region definitions.
///
/// Defines the visual content for each of the 9 regions (corners, edges, center).
/// All regions use solid colors in v1 (future versions may support procedural textures).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NineSliceRegions {
    /// Corner dimensions in pixels [width, height].
    /// All four corners use the same dimensions.
    pub corner_size: [u32; 2],

    /// Top-left corner fill color (RGBA, 0.0-1.0).
    pub top_left: [f64; 4],

    /// Top-right corner fill color (RGBA, 0.0-1.0).
    pub top_right: [f64; 4],

    /// Bottom-left corner fill color (RGBA, 0.0-1.0).
    pub bottom_left: [f64; 4],

    /// Bottom-right corner fill color (RGBA, 0.0-1.0).
    pub bottom_right: [f64; 4],

    /// Top edge fill color (RGBA, 0.0-1.0).
    /// This region tiles horizontally between top-left and top-right corners.
    pub top_edge: [f64; 4],

    /// Bottom edge fill color (RGBA, 0.0-1.0).
    /// This region tiles horizontally between bottom-left and bottom-right corners.
    pub bottom_edge: [f64; 4],

    /// Left edge fill color (RGBA, 0.0-1.0).
    /// This region tiles vertically between top-left and bottom-left corners.
    pub left_edge: [f64; 4],

    /// Right edge fill color (RGBA, 0.0-1.0).
    /// This region tiles vertically between top-right and bottom-right corners.
    pub right_edge: [f64; 4],

    /// Center fill color (RGBA, 0.0-1.0).
    /// This region tiles in both directions to fill the interior.
    pub center: [f64; 4],

    /// Edge slice width in pixels (for left/right edges).
    /// If not specified, defaults to corner_size[0].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_width: Option<u32>,

    /// Edge slice height in pixels (for top/bottom edges).
    /// If not specified, defaults to corner_size[1].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_height: Option<u32>,
}

/// Metadata output for a nine-slice panel atlas.
///
/// Contains UV coordinates and dimensions for each of the 9 regions,
/// allowing runtime UI systems to scale panels correctly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NineSliceMetadata {
    /// Atlas width in pixels.
    pub atlas_width: u32,

    /// Atlas height in pixels.
    pub atlas_height: u32,

    /// Padding/gutter in pixels.
    pub padding: u32,

    /// UV regions for each nine-slice segment.
    pub regions: NineSliceUvRegions,
}

/// UV coordinates for nine-slice regions in normalized [0, 1] space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NineSliceUvRegions {
    /// Top-left corner UV rectangle.
    pub top_left: UvRect,

    /// Top-right corner UV rectangle.
    pub top_right: UvRect,

    /// Bottom-left corner UV rectangle.
    pub bottom_left: UvRect,

    /// Bottom-right corner UV rectangle.
    pub bottom_right: UvRect,

    /// Top edge UV rectangle.
    pub top_edge: UvRect,

    /// Bottom edge UV rectangle.
    pub bottom_edge: UvRect,

    /// Left edge UV rectangle.
    pub left_edge: UvRect,

    /// Right edge UV rectangle.
    pub right_edge: UvRect,

    /// Center UV rectangle.
    pub center: UvRect,
}

/// UV rectangle in normalized [0, 1] coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UvRect {
    /// Left edge U coordinate (0-1).
    pub u_min: f64,

    /// Top edge V coordinate (0-1).
    pub v_min: f64,

    /// Right edge U coordinate (0-1).
    pub u_max: f64,

    /// Bottom edge V coordinate (0-1).
    pub v_max: f64,

    /// Region width in pixels.
    pub width: u32,

    /// Region height in pixels.
    pub height: u32,
}

impl UiNineSliceV1Params {
    /// Creates a new nine-slice params with the given resolution and corner size.
    pub fn new(width: u32, height: u32, corner_width: u32, corner_height: u32) -> Self {
        Self {
            resolution: [width, height],
            padding: DEFAULT_UI_PADDING,
            regions: NineSliceRegions {
                corner_size: [corner_width, corner_height],
                top_left: [0.0, 0.0, 0.0, 1.0],
                top_right: [0.0, 0.0, 0.0, 1.0],
                bottom_left: [0.0, 0.0, 0.0, 1.0],
                bottom_right: [0.0, 0.0, 0.0, 1.0],
                top_edge: [0.0, 0.0, 0.0, 1.0],
                bottom_edge: [0.0, 0.0, 0.0, 1.0],
                left_edge: [0.0, 0.0, 0.0, 1.0],
                right_edge: [0.0, 0.0, 0.0, 1.0],
                center: [1.0, 1.0, 1.0, 1.0],
                edge_width: None,
                edge_height: None,
            },
            background_color: None,
        }
    }

    /// Sets the padding between regions.
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the background color.
    pub fn with_background(mut self, color: [f64; 4]) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Sets the nine-slice regions.
    pub fn with_regions(mut self, regions: NineSliceRegions) -> Self {
        self.regions = regions;
        self
    }
}

impl NineSliceRegions {
    /// Gets the effective edge width (uses edge_width if specified, otherwise corner width).
    pub fn get_edge_width(&self) -> u32 {
        self.edge_width.unwrap_or(self.corner_size[0])
    }

    /// Gets the effective edge height (uses edge_height if specified, otherwise corner height).
    pub fn get_edge_height(&self) -> u32 {
        self.edge_height.unwrap_or(self.corner_size[1])
    }
}

impl UvRect {
    /// Creates a new UV rectangle from pixel coordinates and atlas dimensions.
    pub fn from_pixels(
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        atlas_width: u32,
        atlas_height: u32,
    ) -> Self {
        Self {
            u_min: x as f64 / atlas_width as f64,
            v_min: y as f64 / atlas_height as f64,
            u_max: (x + width) as f64 / atlas_width as f64,
            v_max: (y + height) as f64 / atlas_height as f64,
            width,
            height,
        }
    }
}
