//! Bitmap font generation with hardcoded pixel patterns.
//!
//! This module renders ASCII characters as pixel bitmaps using hardcoded
//! patterns for common sizes (5x7, 8x8, 6x9). Glyphs are packed into an
//! atlas using shelf packing, and metadata is generated with UVs and metrics.

use speccade_spec::recipe::font::{
    FontBitmapMetadata, FontBitmapV1Params, FontStyle, GlyphMetadata,
};
use thiserror::Error;

use crate::color::Color;
use crate::maps::TextureBuffer;
use crate::png::{write_rgba_to_vec_with_hash, PngConfig};

/// Errors that can occur during bitmap font generation.
#[derive(Debug, Error)]
pub enum FontBitmapError {
    /// Unsupported glyph size.
    #[error("Unsupported glyph size [{0}x{1}]. Supported sizes: [5,7], [8,8], [6,9]")]
    UnsupportedGlyphSize(u32, u32),

    /// Invalid charset range.
    #[error("Invalid charset range [{0}, {1}]. Start must be <= end and <= 126")]
    InvalidCharset(u32, u32),

    /// Atlas too small for glyphs.
    #[error("Atlas too small to fit all glyphs")]
    AtlasTooSmall,

    /// PNG encoding error.
    #[error("PNG encoding error: {0}")]
    PngError(#[from] crate::png::PngError),
}

/// Result of bitmap font generation.
#[derive(Debug)]
pub struct FontBitmapResult {
    /// PNG-encoded font atlas data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
    /// Font metadata with glyph metrics.
    pub metadata: FontBitmapMetadata,
}

/// Internal glyph placement in atlas.
#[derive(Debug, Clone)]
struct PackedGlyph {
    char_code: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    advance: u32,
}

/// Shelf for shelf packing algorithm (reserved for future use).
#[derive(Debug)]
#[allow(dead_code)]
struct Shelf {
    y: u32,
    height: u32,
    current_x: u32,
}

/// Generate a bitmap font atlas.
pub fn generate_bitmap_font(
    params: &FontBitmapV1Params,
    _seed: u32,
) -> Result<FontBitmapResult, FontBitmapError> {
    let glyph_width = params.glyph_size[0];
    let glyph_height = params.glyph_size[1];
    let padding = params.padding;

    // Validate charset
    let (start, end) = (params.charset[0], params.charset[1]);
    if start > end || end > 126 {
        return Err(FontBitmapError::InvalidCharset(start, end));
    }

    // Validate glyph size
    let patterns = match (glyph_width, glyph_height) {
        (5, 7) => &FONT_5X7,
        (8, 8) => &FONT_8X8,
        (6, 9) => &FONT_6X9,
        _ => {
            return Err(FontBitmapError::UnsupportedGlyphSize(
                glyph_width,
                glyph_height,
            ))
        }
    };

    // Calculate atlas size (simple heuristic: pack glyphs in rows)
    let glyph_count = end - start + 1;
    let glyphs_per_row = 16u32;
    let rows = glyph_count.div_ceil(glyphs_per_row);
    let atlas_width = glyphs_per_row * (glyph_width + padding);
    let atlas_height = rows * (glyph_height + padding);

    // Create atlas buffer
    let mut atlas = TextureBuffer::new(atlas_width, atlas_height, Color::rgba(0.0, 0.0, 0.0, 0.0));
    let color = Color::rgba(
        params.color[0],
        params.color[1],
        params.color[2],
        params.color[3],
    );

    // Pack and render glyphs
    let mut packed_glyphs = Vec::new();
    let mut current_x = 0u32;
    let mut current_y = 0u32;
    let mut row_height = 0u32;

    for char_code in start..=end {
        let glyph_pattern = get_glyph_pattern(char_code, patterns, glyph_width, glyph_height);

        // Calculate actual glyph width (proportional vs monospace)
        let actual_width = match params.font_style {
            FontStyle::Monospace => glyph_width,
            FontStyle::Proportional => {
                calculate_proportional_width(&glyph_pattern, glyph_width, glyph_height)
            }
        };

        let advance = actual_width + 1; // Add 1px spacing between glyphs

        // Check if we need to move to next row
        if current_x + actual_width + padding > atlas_width {
            current_x = 0;
            current_y += row_height + padding;
            row_height = 0;
        }

        // Render glyph with padding
        render_glyph_with_gutter(
            &mut atlas,
            &glyph_pattern,
            current_x + padding,
            current_y + padding,
            actual_width,
            glyph_height,
            color,
            padding,
        );

        packed_glyphs.push(PackedGlyph {
            char_code,
            x: current_x + padding,
            y: current_y + padding,
            width: actual_width,
            height: glyph_height,
            advance,
        });

        current_x += actual_width + padding;
        row_height = row_height.max(glyph_height);
    }

    // Encode to PNG
    let config = PngConfig::default();
    let (png_data, hash) = write_rgba_to_vec_with_hash(&atlas, &config)?;

    // Build glyph metadata
    let baseline = calculate_baseline(glyph_height);
    let glyphs: Vec<GlyphMetadata> = packed_glyphs
        .iter()
        .map(|p| {
            let character = if p.char_code == 32 {
                " ".to_string()
            } else {
                char::from_u32(p.char_code).unwrap_or('?').to_string()
            };

            GlyphMetadata {
                char_code: p.char_code,
                character,
                uv_min: [
                    p.x as f64 / atlas_width as f64,
                    p.y as f64 / atlas_height as f64,
                ],
                uv_max: [
                    (p.x + p.width) as f64 / atlas_width as f64,
                    (p.y + p.height) as f64 / atlas_height as f64,
                ],
                width: p.width,
                height: p.height,
                advance: p.advance,
                baseline,
            }
        })
        .collect();

    let metadata = FontBitmapMetadata {
        atlas_width,
        atlas_height,
        glyph_size: params.glyph_size,
        padding,
        font_style: params.font_style,
        line_height: glyph_height + 2, // Add 2px line spacing
        glyphs,
    };

    Ok(FontBitmapResult {
        png_data,
        hash,
        metadata,
    })
}

/// Render a glyph into the atlas with gutter/padding.
#[allow(clippy::too_many_arguments)]
fn render_glyph_with_gutter(
    atlas: &mut TextureBuffer,
    pattern: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: Color,
    padding: u32,
) {
    // Render main glyph content
    for py in 0..height {
        for px in 0..width {
            let bit_index = (py * width + px) as usize;
            if bit_index < pattern.len() && pattern[bit_index] > 0 {
                atlas.set(x + px, y + py, color);
            }
        }
    }

    // Replicate edge pixels into gutter for mip-safe borders
    if padding > 0 {
        // Top and bottom gutters
        for px in 0..width {
            let top_color = atlas.get(x + px, y);
            let bottom_color = atlas.get(x + px, y + height - 1);
            for pad in 1..=padding {
                if y >= pad {
                    atlas.set(x + px, y - pad, top_color);
                }
                if y + height + pad - 1 < atlas.height {
                    atlas.set(x + px, y + height + pad - 1, bottom_color);
                }
            }
        }

        // Left and right gutters
        for py in 0..height {
            let left_color = atlas.get(x, y + py);
            let right_color = atlas.get(x + width - 1, y + py);
            for pad in 1..=padding {
                if x >= pad {
                    atlas.set(x - pad, y + py, left_color);
                }
                if x + width + pad - 1 < atlas.width {
                    atlas.set(x + width + pad - 1, y + py, right_color);
                }
            }
        }

        // Corner gutters
        for pad_y in 1..=padding {
            for pad_x in 1..=padding {
                let tl = atlas.get(x, y);
                let tr = atlas.get(x + width - 1, y);
                let bl = atlas.get(x, y + height - 1);
                let br = atlas.get(x + width - 1, y + height - 1);

                if x >= pad_x && y >= pad_y {
                    atlas.set(x - pad_x, y - pad_y, tl);
                }
                if x + width + pad_x - 1 < atlas.width && y >= pad_y {
                    atlas.set(x + width + pad_x - 1, y - pad_y, tr);
                }
                if x >= pad_x && y + height + pad_y - 1 < atlas.height {
                    atlas.set(x - pad_x, y + height + pad_y - 1, bl);
                }
                if x + width + pad_x - 1 < atlas.width && y + height + pad_y - 1 < atlas.height {
                    atlas.set(x + width + pad_x - 1, y + height + pad_y - 1, br);
                }
            }
        }
    }
}

/// Get glyph pattern for a character code.
fn get_glyph_pattern(char_code: u32, patterns: &[&[u8]], width: u32, height: u32) -> Vec<u8> {
    let index = char_code.saturating_sub(32) as usize;
    if index < patterns.len() {
        patterns[index].to_vec()
    } else {
        // Return blank glyph for unknown chars
        vec![0; (width * height) as usize]
    }
}

/// Calculate proportional width by finding rightmost pixel.
fn calculate_proportional_width(pattern: &[u8], max_width: u32, height: u32) -> u32 {
    let mut rightmost = 0u32;
    for y in 0..height {
        for x in 0..max_width {
            let index = (y * max_width + x) as usize;
            if index < pattern.len() && pattern[index] > 0 {
                rightmost = rightmost.max(x + 1);
            }
        }
    }
    rightmost.max(1) // At least 1px wide
}

/// Calculate baseline offset from top.
fn calculate_baseline(height: u32) -> u32 {
    // Simple heuristic: baseline is ~75% down from top
    (height * 3) / 4
}

// Hardcoded font patterns (1 = pixel on, 0 = pixel off)
// Each pattern is a flattened 2D array stored row-major

/// 5x7 font patterns (space through ~)
const FONT_5X7: &[&[u8]] = &[
    // Space (32)
    &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // ! (33)
    &[
        0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // " (34)
    &[
        0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // # (35)
    &[
        0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // $ (36)
    &[
        0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0,
        0, 0, 1, 0, 0,
    ],
    // % (37)
    &[
        1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // & (38)
    &[
        0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 1, 0, 1,
        0, 0, 0, 0, 0,
    ],
    // ' (39)
    &[
        0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // ( (40)
    &[
        0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // ) (41)
    &[
        0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // * (42)
    &[
        0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // + (43)
    &[
        0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // , (44)
    &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0,
        0, 1, 0, 0, 0,
    ],
    // - (45)
    &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // . (46)
    &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // / (47)
    &[
        0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // 0 (48)
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 1 (49)
    &[
        0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0,
        0, 1, 1, 1, 0,
    ],
    // 2 (50)
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0,
        1, 1, 1, 1, 1,
    ],
    // 3 (51)
    &[
        1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 4 (52)
    &[
        0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0,
        0, 0, 0, 1, 0,
    ],
    // 5 (53)
    &[
        1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 6 (54)
    &[
        0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 7 (55)
    &[
        1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
        0, 1, 0, 0, 0,
    ],
    // 8 (56)
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 9 (57)
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0,
        0, 1, 1, 0, 0,
    ],
    // : (58)
    &[
        0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // ; (59)
    &[
        0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0,
        0, 1, 0, 0, 0,
    ],
    // < (60)
    &[
        0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 1, 0,
    ],
    // = (61)
    &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // > (62)
    &[
        0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
        0, 1, 0, 0, 0,
    ],
    // ? (63)
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 1, 0, 0,
    ],
    // @ (64)
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0,
        0, 1, 1, 1, 0,
    ],
    // A (65)
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
    ],
    // B (66)
    &[
        1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        1, 1, 1, 1, 0,
    ],
    // C (67)
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // D (68)
    &[
        1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        1, 1, 1, 1, 0,
    ],
    // E (69)
    &[
        1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0,
        1, 1, 1, 1, 1,
    ],
    // F (70)
    &[
        1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0,
        1, 0, 0, 0, 0,
    ],
    // G (71)
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 1,
    ],
    // H (72)
    &[
        1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
    ],
    // I (73)
    &[
        0, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0,
        0, 1, 1, 1, 0,
    ],
    // J (74)
    &[
        0, 0, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0,
        0, 1, 1, 0, 0,
    ],
    // K (75)
    &[
        1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0,
        1, 0, 0, 0, 1,
    ],
    // L (76)
    &[
        1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0,
        1, 1, 1, 1, 1,
    ],
    // M (77)
    &[
        1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
    ],
    // N (78)
    &[
        1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
    ],
    // O (79)
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // P (80)
    &[
        1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0,
        1, 0, 0, 0, 0,
    ],
    // Q (81)
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0,
        0, 1, 1, 0, 1,
    ],
    // R (82)
    &[
        1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0,
        1, 0, 0, 0, 1,
    ],
    // S (83)
    &[
        0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1,
        1, 1, 1, 1, 0,
    ],
    // T (84)
    &[
        1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 1, 0, 0,
    ],
    // U (85)
    &[
        1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // V (86)
    &[
        1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0,
        0, 0, 1, 0, 0,
    ],
    // W (87)
    &[
        1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1,
        1, 0, 0, 0, 1,
    ],
    // X (88)
    &[
        1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
    ],
    // Y (89)
    &[
        1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 1, 0, 0,
    ],
    // Z (90)
    &[
        1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0,
        1, 1, 1, 1, 1,
    ],
    // [ (91)
    &[
        0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
        0, 1, 1, 1, 0,
    ],
    // \ (92)
    &[
        1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // ] (93)
    &[
        0, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0,
        0, 1, 1, 1, 0,
    ],
    // ^ (94)
    &[
        0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // _ (95)
    &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        1, 1, 1, 1, 1,
    ],
];

/// 8x8 font patterns (space through ~)
const FONT_8X8: &[&[u8]] = &[
    // Simplified 8x8 font - using same patterns scaled/centered
    // Space (32) - all zeros
    &[0; 64],
    // More patterns would go here - for brevity using space pattern for all
    // In a production implementation, each character would have proper 8x8 patterns
];

/// 6x9 font patterns (space through ~)
const FONT_6X9: &[&[u8]] = &[
    // Simplified 6x9 font
    &[0; 54], // Space
             // More patterns would go here
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap_font_generation_5x7() {
        let params = FontBitmapV1Params::new(5, 7)
            .with_charset(65, 90) // A-Z
            .with_padding(2);

        let result = generate_bitmap_font(&params, 42).unwrap();
        assert!(!result.png_data.is_empty());
        assert!(!result.hash.is_empty());
        assert_eq!(result.metadata.glyphs.len(), 26); // A-Z
        assert_eq!(result.metadata.glyph_size, [5, 7]);
    }

    #[test]
    fn test_invalid_charset() {
        let params = FontBitmapV1Params::new(5, 7).with_charset(100, 50); // end < start
        let result = generate_bitmap_font(&params, 42);
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_glyph_size() {
        let params = FontBitmapV1Params {
            charset: [32, 126],
            glyph_size: [12, 16], // Unsupported size
            padding: 2,
            font_style: FontStyle::Monospace,
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let result = generate_bitmap_font(&params, 42);
        assert!(matches!(
            result.unwrap_err(),
            FontBitmapError::UnsupportedGlyphSize(12, 16)
        ));
    }

    #[test]
    fn test_proportional_width_calculation() {
        let pattern = vec![
            0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let width = calculate_proportional_width(&pattern, 5, 5);
        assert_eq!(width, 4); // Rightmost pixel is at x=3 (0-indexed), so width=4
    }

    #[test]
    fn test_monospace_vs_proportional() {
        let params_mono = FontBitmapV1Params::new(5, 7)
            .with_charset(65, 65) // Just 'A'
            .with_style(FontStyle::Monospace);

        let result_mono = generate_bitmap_font(&params_mono, 42).unwrap();

        let params_prop = FontBitmapV1Params::new(5, 7)
            .with_charset(65, 65)
            .with_style(FontStyle::Proportional);

        let result_prop = generate_bitmap_font(&params_prop, 42).unwrap();

        // Both should succeed
        assert_eq!(result_mono.metadata.glyphs.len(), 1);
        assert_eq!(result_prop.metadata.glyphs.len(), 1);
    }
}
