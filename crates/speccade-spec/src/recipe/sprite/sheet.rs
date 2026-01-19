//! Spritesheet recipe types for `sprite.sheet_v1`.
//!
//! Packs sprite frames into an atlas with deterministic shelf packing,
//! mip-safe gutters, and metadata output (UVs, pivots, dimensions).

use serde::{Deserialize, Serialize};

/// Parameters for the `sprite.sheet_v1` recipe.
///
/// Packs multiple sprite frames into a single atlas texture with
/// deterministic shelf packing and mip-safe gutters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteSheetV1Params {
    /// Atlas resolution [width, height] in pixels.
    pub resolution: [u32; 2],

    /// Padding/gutter in pixels between frames (for mip-safe borders).
    #[serde(default = "default_padding")]
    pub padding: u32,

    /// List of frames to pack into the atlas.
    pub frames: Vec<SpriteFrame>,
}

fn default_padding() -> u32 {
    2
}

/// A sprite frame definition for atlas packing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteFrame {
    /// Unique identifier for this frame.
    pub id: String,

    /// Frame width in pixels.
    pub width: u32,

    /// Frame height in pixels.
    pub height: u32,

    /// Pivot point in normalized coordinates [0-1].
    /// [0,0] = top-left, [0.5,0.5] = center, [1,1] = bottom-right.
    #[serde(default = "default_pivot")]
    pub pivot: [f64; 2],

    /// Frame content source.
    #[serde(flatten)]
    pub source: SpriteFrameSource,
}

fn default_pivot() -> [f64; 2] {
    [0.5, 0.5]
}

/// Source for sprite frame content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpriteFrameSource {
    /// Solid color fill (v1 only supports solid colors).
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

/// UV rectangle for a packed frame in normalized [0, 1] coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteFrameUv {
    /// Frame identifier.
    pub id: String,

    /// Left edge U coordinate (0-1).
    pub u_min: f64,

    /// Top edge V coordinate (0-1).
    pub v_min: f64,

    /// Right edge U coordinate (0-1).
    pub u_max: f64,

    /// Bottom edge V coordinate (0-1).
    pub v_max: f64,

    /// Frame width in pixels.
    pub width: u32,

    /// Frame height in pixels.
    pub height: u32,

    /// Pivot point in normalized coordinates [0-1].
    pub pivot: [f64; 2],
}

/// Metadata output for a packed spritesheet atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheetMetadata {
    /// Atlas width in pixels.
    pub atlas_width: u32,

    /// Atlas height in pixels.
    pub atlas_height: u32,

    /// Padding/gutter in pixels.
    pub padding: u32,

    /// UV rectangles and metadata for each packed frame.
    pub frames: Vec<SpriteFrameUv>,
}

impl SpriteSheetV1Params {
    /// Creates a new spritesheet params with the given resolution.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            resolution: [width, height],
            padding: default_padding(),
            frames: Vec::new(),
        }
    }

    /// Sets the padding/gutter between frames.
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Adds a frame to the spritesheet.
    pub fn with_frame(mut self, frame: SpriteFrame) -> Self {
        self.frames.push(frame);
        self
    }
}

impl SpriteFrame {
    /// Creates a new solid color frame.
    pub fn solid(id: impl Into<String>, width: u32, height: u32, color: [f64; 4]) -> Self {
        Self {
            id: id.into(),
            width,
            height,
            pivot: default_pivot(),
            source: SpriteFrameSource::Color { color },
        }
    }

    /// Sets the pivot point for this frame.
    pub fn with_pivot(mut self, pivot: [f64; 2]) -> Self {
        self.pivot = pivot;
        self
    }
}
