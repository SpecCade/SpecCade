//! Icon set recipe types for `ui.icon_set_v1`.
//!
//! Icon sets pack multiple icon frames into a sprite atlas with labeled entries,
//! ideal for UI icon libraries, button sets, and status indicators.

use serde::{Deserialize, Serialize};

use super::DEFAULT_UI_PADDING;

/// Parameters for the `ui.icon_set_v1` recipe.
///
/// Packs icon frames into a sprite atlas with deterministic shelf packing.
/// Icons are rendered as solid color shapes (placeholders in v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiIconSetV1Params {
    /// Atlas resolution [width, height] in pixels.
    pub resolution: [u32; 2],

    /// Padding/gutter in pixels between icons (for mip-safe borders).
    #[serde(default = "default_ui_padding")]
    pub padding: u32,

    /// List of icon entries to pack into the atlas.
    pub icons: Vec<IconEntry>,
}

fn default_ui_padding() -> u32 {
    DEFAULT_UI_PADDING
}

/// An icon entry definition for atlas packing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IconEntry {
    /// Unique identifier for this icon (e.g., "close", "settings", "heart").
    pub id: String,

    /// Icon width in pixels.
    pub width: u32,

    /// Icon height in pixels.
    pub height: u32,

    /// Icon fill color (RGBA, 0.0-1.0).
    /// In v1, all icons are solid color rectangles (placeholders).
    pub color: [f64; 4],

    /// Optional semantic category (e.g., "action", "status", "social").
    /// Helps organize large icon sets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// UV rectangle for a packed icon in normalized [0, 1] coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IconUv {
    /// Icon identifier.
    pub id: String,

    /// Left edge U coordinate (0-1).
    pub u_min: f64,

    /// Top edge V coordinate (0-1).
    pub v_min: f64,

    /// Right edge U coordinate (0-1).
    pub u_max: f64,

    /// Bottom edge V coordinate (0-1).
    pub v_max: f64,

    /// Icon width in pixels.
    pub width: u32,

    /// Icon height in pixels.
    pub height: u32,

    /// Optional semantic category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Metadata output for an icon set atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IconSetMetadata {
    /// Atlas width in pixels.
    pub atlas_width: u32,

    /// Atlas height in pixels.
    pub atlas_height: u32,

    /// Padding/gutter in pixels.
    pub padding: u32,

    /// UV rectangles and metadata for each packed icon.
    pub icons: Vec<IconUv>,
}

impl UiIconSetV1Params {
    /// Creates a new icon set params with the given resolution.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            resolution: [width, height],
            padding: DEFAULT_UI_PADDING,
            icons: Vec::new(),
        }
    }

    /// Sets the padding between icons.
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Adds an icon to the set.
    pub fn with_icon(mut self, icon: IconEntry) -> Self {
        self.icons.push(icon);
        self
    }
}

impl IconEntry {
    /// Creates a new icon entry.
    pub fn new(id: impl Into<String>, width: u32, height: u32, color: [f64; 4]) -> Self {
        Self {
            id: id.into(),
            width,
            height,
            color,
            category: None,
        }
    }

    /// Sets the category for this icon.
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}
