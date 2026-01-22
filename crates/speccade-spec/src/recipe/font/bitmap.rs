//! Bitmap font recipe types for `font.bitmap_v1`.
//!
//! Renders ASCII characters as pixel bitmaps, packs into an atlas,
//! and outputs glyph metrics JSON for runtime text rendering.

use serde::{Deserialize, Serialize};

/// Parameters for the `font.bitmap_v1` recipe.
///
/// Renders glyphs using hardcoded pixel patterns (5x7, 8x8, etc.),
/// packs them into an atlas, and outputs glyph metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontBitmapV1Params {
    /// Character set to include (ASCII range).
    /// Format: [start, end] inclusive (e.g., [32, 126] for printable ASCII).
    pub charset: [u32; 2],

    /// Glyph size in pixels [width, height].
    /// Supported sizes: [5, 7], [8, 8], [6, 9].
    pub glyph_size: [u32; 2],

    /// Padding in pixels between glyphs (for mip-safe borders).
    #[serde(default = "default_padding")]
    pub padding: u32,

    /// Font style (monospace or proportional).
    #[serde(default)]
    pub font_style: FontStyle,

    /// Glyph color in RGBA (0.0-1.0).
    #[serde(default = "default_color")]
    pub color: [f64; 4],
}

fn default_padding() -> u32 {
    2
}

fn default_color() -> [f64; 4] {
    [1.0, 1.0, 1.0, 1.0]
}

/// Font style (monospace or proportional spacing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FontStyle {
    /// Monospace: all glyphs have the same advance width.
    #[default]
    Monospace,
    /// Proportional: glyphs have variable advance widths based on actual pixel data.
    Proportional,
}

/// Metadata for a single glyph in the atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphMetadata {
    /// Character code (e.g., 65 for 'A').
    pub char_code: u32,

    /// Character as a string (e.g., "A").
    pub character: String,

    /// UV min (top-left) in normalized [0, 1] coordinates.
    pub uv_min: [f64; 2],

    /// UV max (bottom-right) in normalized [0, 1] coordinates.
    pub uv_max: [f64; 2],

    /// Glyph width in pixels.
    pub width: u32,

    /// Glyph height in pixels.
    pub height: u32,

    /// Horizontal advance in pixels (distance to next glyph).
    pub advance: u32,

    /// Baseline offset in pixels from top (distance from top to baseline).
    pub baseline: u32,
}

/// Metadata output for a bitmap font atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontBitmapMetadata {
    /// Atlas width in pixels.
    pub atlas_width: u32,

    /// Atlas height in pixels.
    pub atlas_height: u32,

    /// Glyph size [width, height] in pixels.
    pub glyph_size: [u32; 2],

    /// Padding in pixels.
    pub padding: u32,

    /// Font style.
    pub font_style: FontStyle,

    /// Line height in pixels (recommended spacing between lines).
    pub line_height: u32,

    /// List of glyphs in the atlas.
    pub glyphs: Vec<GlyphMetadata>,
}

impl FontBitmapV1Params {
    /// Creates a new bitmap font params with default ASCII printable charset (32-126).
    pub fn new(glyph_width: u32, glyph_height: u32) -> Self {
        Self {
            charset: [32, 126], // ASCII printable characters
            glyph_size: [glyph_width, glyph_height],
            padding: default_padding(),
            font_style: FontStyle::default(),
            color: default_color(),
        }
    }

    /// Sets the character set range.
    pub fn with_charset(mut self, start: u32, end: u32) -> Self {
        self.charset = [start, end];
        self
    }

    /// Sets the padding between glyphs.
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the font style.
    pub fn with_style(mut self, style: FontStyle) -> Self {
        self.font_style = style;
        self
    }

    /// Sets the glyph color.
    pub fn with_color(mut self, color: [f64; 4]) -> Self {
        self.color = color;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap_params_defaults() {
        let params = FontBitmapV1Params::new(8, 8);
        assert_eq!(params.charset, [32, 126]);
        assert_eq!(params.glyph_size, [8, 8]);
        assert_eq!(params.padding, 2);
        assert_eq!(params.font_style, FontStyle::Monospace);
        assert_eq!(params.color, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_bitmap_params_builder() {
        let params = FontBitmapV1Params::new(5, 7)
            .with_charset(65, 90)
            .with_padding(1)
            .with_style(FontStyle::Proportional)
            .with_color([0.8, 0.8, 0.8, 1.0]);

        assert_eq!(params.charset, [65, 90]);
        assert_eq!(params.padding, 1);
        assert_eq!(params.font_style, FontStyle::Proportional);
        assert_eq!(params.color, [0.8, 0.8, 0.8, 1.0]);
    }

    #[test]
    fn test_font_style_serde() {
        let style = FontStyle::Monospace;
        let json = serde_json::to_string(&style).unwrap();
        assert_eq!(json, "\"monospace\"");

        let parsed: FontStyle = serde_json::from_str("\"proportional\"").unwrap();
        assert_eq!(parsed, FontStyle::Proportional);
    }
}
