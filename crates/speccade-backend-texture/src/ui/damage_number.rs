//! Damage number sprite generation with deterministic atlas packing.
//!
//! This module implements a damage number generator that creates sprite atlases
//! for combat feedback UI (damage, healing, critical hits). Each style variant
//! gets its own row in the atlas with all digit glyphs.

use speccade_spec::recipe::ui::{
    DamageNumberGlyph, DamageNumberMetadata, DamageNumberStyle, DamageNumberStyleMetadata,
    UiDamageNumberV1Params,
};
use thiserror::Error;

use super::gutter::validate_color;
use crate::color::Color;
use crate::maps::TextureBuffer;
use crate::png::{write_rgba_to_vec_with_hash, PngConfig};

/// Errors that can occur during damage number generation.
#[derive(Debug, Error)]
pub enum DamageNumberError {
    /// No styles defined.
    #[error("At least one style must be defined")]
    NoStyles,

    /// Invalid glyph size.
    #[error("Invalid glyph size [{0}x{1}]: must be at least 8x8 and at most 128x128")]
    InvalidGlyphSize(u32, u32),

    /// Invalid outline width.
    #[error("Invalid outline width {0}: must be between 1 and 8")]
    InvalidOutlineWidth(u32),

    /// Invalid color value.
    #[error("{0}")]
    InvalidColor(String),

    /// Duplicate style type.
    #[error("Duplicate style type: '{0}'")]
    DuplicateStyleType(String),

    /// Invalid scale value.
    #[error("Invalid scale {0}: must be between 0.5 and 2.0")]
    InvalidScale(f64),

    /// PNG encoding error.
    #[error("PNG encoding error: {0}")]
    PngError(#[from] crate::png::PngError),
}

/// Result of damage number generation.
#[derive(Debug)]
pub struct DamageNumberResult {
    /// PNG-encoded atlas image data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
    /// Metadata with style and glyph information.
    pub metadata: DamageNumberMetadata,
}

/// The charset for damage numbers: digits 0-9 plus +, -, .
const DAMAGE_NUMBER_CHARSET: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '-', '.',
];

/// Hardcoded 5x7 digit patterns (row-major, 1 = pixel on, 0 = pixel off).
/// These are the base patterns that get scaled up to the target glyph size.
const DIGIT_PATTERNS_5X7: &[&[u8]] = &[
    // 0
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 1
    &[
        0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0,
        0, 1, 1, 1, 0,
    ],
    // 2
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0,
        1, 1, 1, 1, 1,
    ],
    // 3
    &[
        1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 4
    &[
        0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0,
        0, 0, 0, 1, 0,
    ],
    // 5
    &[
        1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 6
    &[
        0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 7
    &[
        1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
        0, 1, 0, 0, 0,
    ],
    // 8
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 9
    &[
        0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0,
        0, 1, 1, 0, 0,
    ],
    // + (plus)
    &[
        0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // - (minus)
    &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ],
    // . (period)
    &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
        0, 1, 1, 0, 0,
    ],
];

const BASE_PATTERN_WIDTH: u32 = 5;
const BASE_PATTERN_HEIGHT: u32 = 7;

/// Generate a damage number atlas from parameters.
///
/// # Arguments
/// * `params` - Damage number parameters including glyph size, outline, and styles
/// * `_seed` - Deterministic seed (reserved for future procedural effects)
///
/// # Returns
/// A `DamageNumberResult` containing the PNG data, hash, and metadata.
pub fn generate_damage_number(
    params: &UiDamageNumberV1Params,
    _seed: u32,
) -> Result<DamageNumberResult, DamageNumberError> {
    let glyph_width = params.glyph_size[0];
    let glyph_height = params.glyph_size[1];
    let outline_width = params.outline_width;
    let padding = params.padding;

    // Validate glyph size
    if glyph_width < 8 || glyph_height < 8 || glyph_width > 128 || glyph_height > 128 {
        return Err(DamageNumberError::InvalidGlyphSize(
            glyph_width,
            glyph_height,
        ));
    }

    // Validate outline width
    if !(1..=8).contains(&outline_width) {
        return Err(DamageNumberError::InvalidOutlineWidth(outline_width));
    }

    // Validate at least one style
    if params.styles.is_empty() {
        return Err(DamageNumberError::NoStyles);
    }

    // Validate colors and check for duplicates
    let mut seen_types = std::collections::HashSet::new();
    for style in &params.styles {
        if !seen_types.insert(style.style_type) {
            return Err(DamageNumberError::DuplicateStyleType(
                style.style_type.as_str().to_string(),
            ));
        }

        validate_color(&style.text_color, &format!("{} text", style.style_type))
            .map_err(DamageNumberError::InvalidColor)?;
        validate_color(
            &style.outline_color,
            &format!("{} outline", style.style_type),
        )
        .map_err(DamageNumberError::InvalidColor)?;

        if let Some(glow) = &style.glow_color {
            validate_color(glow, &format!("{} glow", style.style_type))
                .map_err(DamageNumberError::InvalidColor)?;
        }

        if let Some(scale) = style.scale {
            if !(0.5..=2.0).contains(&scale) {
                return Err(DamageNumberError::InvalidScale(scale));
            }
        }
    }

    // Sort styles by type for deterministic output
    let mut sorted_styles = params.styles.clone();
    sorted_styles.sort_by_key(|s| s.style_type.sort_order());

    // Calculate scaled glyph dimensions (including outline and padding)
    let total_glyph_width = glyph_width + outline_width * 2;
    let total_glyph_height = glyph_height + outline_width * 2;

    // Calculate atlas dimensions
    // Each row contains all glyphs for one style
    let glyphs_per_row = DAMAGE_NUMBER_CHARSET.len() as u32;
    let atlas_width = glyphs_per_row * (total_glyph_width + padding) + padding;
    let atlas_height = (sorted_styles.len() as u32) * (total_glyph_height + padding) + padding;

    // Create atlas buffer
    let mut atlas = TextureBuffer::new(atlas_width, atlas_height, Color::rgba(0.0, 0.0, 0.0, 0.0));

    // Render each style row
    let mut style_metadata = Vec::with_capacity(sorted_styles.len());

    for (style_idx, style) in sorted_styles.iter().enumerate() {
        let row_y = (style_idx as u32) * (total_glyph_height + padding) + padding;
        let scale = style.scale.unwrap_or(1.0);

        let mut glyphs = Vec::with_capacity(DAMAGE_NUMBER_CHARSET.len());

        for (glyph_idx, &ch) in DAMAGE_NUMBER_CHARSET.iter().enumerate() {
            let glyph_x = (glyph_idx as u32) * (total_glyph_width + padding) + padding;

            // Get the pattern for this character
            let pattern = DIGIT_PATTERNS_5X7[glyph_idx];

            // Render the glyph with outline and optional glow
            render_glyph_with_outline(
                &mut atlas,
                glyph_x,
                row_y,
                glyph_width,
                glyph_height,
                outline_width,
                pattern,
                style,
                scale,
            );

            // Calculate UV coordinates
            let u_min = glyph_x as f64 / atlas_width as f64;
            let v_min = row_y as f64 / atlas_height as f64;
            let u_max = (glyph_x + total_glyph_width) as f64 / atlas_width as f64;
            let v_max = (row_y + total_glyph_height) as f64 / atlas_height as f64;

            glyphs.push(DamageNumberGlyph {
                char_code: ch.to_string(),
                uv: [u_min, v_min, u_max, v_max],
                width: total_glyph_width,
                height: total_glyph_height,
            });
        }

        let uv_offset = [
            padding as f64 / atlas_width as f64,
            row_y as f64 / atlas_height as f64,
        ];

        style_metadata.push(DamageNumberStyleMetadata {
            style: style.style_type.as_str().to_string(),
            uv_offset,
            scale,
            glyphs,
        });
    }

    // Encode to PNG
    let config = PngConfig::default();
    let (png_data, hash) = write_rgba_to_vec_with_hash(&atlas, &config)?;

    let metadata = DamageNumberMetadata {
        atlas_size: [atlas_width, atlas_height],
        glyph_size: params.glyph_size,
        outline_width,
        padding,
        styles: style_metadata,
    };

    Ok(DamageNumberResult {
        png_data,
        hash,
        metadata,
    })
}

/// Render a single glyph with outline and optional glow.
#[allow(clippy::too_many_arguments)]
fn render_glyph_with_outline(
    atlas: &mut TextureBuffer,
    x: u32,
    y: u32,
    glyph_width: u32,
    glyph_height: u32,
    outline_width: u32,
    pattern: &[u8],
    style: &DamageNumberStyle,
    _scale: f64,
) {
    let text_color = Color::rgba(
        style.text_color[0],
        style.text_color[1],
        style.text_color[2],
        style.text_color[3],
    );

    let outline_color = Color::rgba(
        style.outline_color[0],
        style.outline_color[1],
        style.outline_color[2],
        style.outline_color[3],
    );

    let glow_color = style
        .glow_color
        .map(|c| Color::rgba(c[0], c[1], c[2], c[3]));

    // Scale the pattern to the target glyph size
    let scaled_pattern = scale_pattern(
        pattern,
        BASE_PATTERN_WIDTH,
        BASE_PATTERN_HEIGHT,
        glyph_width,
        glyph_height,
    );

    // First pass: render glow (if present)
    if let Some(glow) = glow_color {
        let glow_radius = outline_width + 1;
        for gy in 0..glyph_height {
            for gx in 0..glyph_width {
                let pattern_idx = (gy * glyph_width + gx) as usize;
                if pattern_idx < scaled_pattern.len() && scaled_pattern[pattern_idx] > 0 {
                    // Draw glow around this pixel
                    for dy in 0..=(glow_radius * 2) {
                        for dx in 0..=(glow_radius * 2) {
                            let px = x + outline_width + gx + dx;
                            let py = y + outline_width + gy + dy;
                            if px >= glow_radius && py >= glow_radius {
                                let px = px - glow_radius;
                                let py = py - glow_radius;
                                if px < atlas.width && py < atlas.height {
                                    let dist_x = (dx as i32 - glow_radius as i32).abs() as f64;
                                    let dist_y = (dy as i32 - glow_radius as i32).abs() as f64;
                                    let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();
                                    if dist <= glow_radius as f64 {
                                        let alpha =
                                            (1.0 - dist / (glow_radius as f64 + 1.0)) * glow.a;
                                        let existing = atlas.get(px, py);
                                        let blended = blend_additive(glow, existing, alpha);
                                        atlas.set(px, py, blended);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Second pass: render outline
    for gy in 0..glyph_height {
        for gx in 0..glyph_width {
            let pattern_idx = (gy * glyph_width + gx) as usize;
            if pattern_idx < scaled_pattern.len() && scaled_pattern[pattern_idx] > 0 {
                // Draw outline pixels around this glyph pixel
                for dy in 0..=(outline_width * 2) {
                    for dx in 0..=(outline_width * 2) {
                        let px = x + gx + dx;
                        let py = y + gy + dy;
                        if px < atlas.width && py < atlas.height {
                            // Only draw if this is within outline distance from center
                            let dist_x = (dx as i32 - outline_width as i32).abs();
                            let dist_y = (dy as i32 - outline_width as i32).abs();
                            if dist_x <= outline_width as i32 && dist_y <= outline_width as i32 {
                                let existing = atlas.get(px, py);
                                if existing.a < outline_color.a {
                                    atlas.set(px, py, outline_color);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Third pass: render text (on top of outline)
    for gy in 0..glyph_height {
        for gx in 0..glyph_width {
            let pattern_idx = (gy * glyph_width + gx) as usize;
            if pattern_idx < scaled_pattern.len() && scaled_pattern[pattern_idx] > 0 {
                let px = x + outline_width + gx;
                let py = y + outline_width + gy;
                if px < atlas.width && py < atlas.height {
                    atlas.set(px, py, text_color);
                }
            }
        }
    }
}

/// Scale a pattern from source dimensions to target dimensions using nearest-neighbor.
fn scale_pattern(
    pattern: &[u8],
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> Vec<u8> {
    let mut scaled = vec![0u8; (dst_width * dst_height) as usize];

    for dy in 0..dst_height {
        for dx in 0..dst_width {
            // Map destination coordinate to source coordinate
            let sx = (dx * src_width / dst_width).min(src_width - 1);
            let sy = (dy * src_height / dst_height).min(src_height - 1);

            let src_idx = (sy * src_width + sx) as usize;
            let dst_idx = (dy * dst_width + dx) as usize;

            if src_idx < pattern.len() {
                scaled[dst_idx] = pattern[src_idx];
            }
        }
    }

    scaled
}

/// Additive blending for glow effect.
fn blend_additive(src: Color, dst: Color, src_alpha: f64) -> Color {
    let out_r = (dst.r + src.r * src_alpha).min(1.0);
    let out_g = (dst.g + src.g * src_alpha).min(1.0);
    let out_b = (dst.b + src.b * src_alpha).min(1.0);
    let out_a = (dst.a + src_alpha).min(1.0);

    Color::rgba(out_r, out_g, out_b, out_a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::ui::DamageNumberStyleType;

    fn make_basic_params() -> UiDamageNumberV1Params {
        UiDamageNumberV1Params::new(16, 24)
            .with_outline_width(2)
            .with_style(DamageNumberStyle::new(
                DamageNumberStyleType::Normal,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ))
    }

    #[test]
    fn test_generate_single_style() {
        let params = make_basic_params();
        let result = generate_damage_number(&params, 42).unwrap();

        assert!(!result.png_data.is_empty());
        assert_eq!(result.metadata.styles.len(), 1);
        assert_eq!(result.metadata.styles[0].style, "normal");
        assert_eq!(
            result.metadata.styles[0].glyphs.len(),
            DAMAGE_NUMBER_CHARSET.len()
        );
        assert_eq!(result.metadata.glyph_size, [16, 24]);
    }

    #[test]
    fn test_generate_multiple_styles() {
        let params = UiDamageNumberV1Params::new(16, 24)
            .with_outline_width(2)
            .with_style(DamageNumberStyle::new(
                DamageNumberStyleType::Normal,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ))
            .with_style(
                DamageNumberStyle::new(
                    DamageNumberStyleType::Critical,
                    [1.0, 0.9, 0.0, 1.0],
                    [1.0, 0.0, 0.0, 1.0],
                )
                .with_scale(1.25)
                .with_glow([1.0, 0.5, 0.0, 0.5]),
            )
            .with_style(DamageNumberStyle::new(
                DamageNumberStyleType::Healing,
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.3, 0.0, 1.0],
            ));

        let result = generate_damage_number(&params, 42).unwrap();

        assert_eq!(result.metadata.styles.len(), 3);
        // Styles should be sorted by type
        assert_eq!(result.metadata.styles[0].style, "normal");
        assert_eq!(result.metadata.styles[1].style, "critical");
        assert_eq!(result.metadata.styles[2].style, "healing");
    }

    #[test]
    fn test_determinism() {
        let params = make_basic_params();

        let result1 = generate_damage_number(&params, 42).unwrap();
        let result2 = generate_damage_number(&params, 42).unwrap();

        assert_eq!(result1.png_data, result2.png_data);
        assert_eq!(result1.hash, result2.hash);
    }

    #[test]
    fn test_no_styles() {
        let params = UiDamageNumberV1Params::new(16, 24).with_outline_width(2);

        let err = generate_damage_number(&params, 42).unwrap_err();
        assert!(matches!(err, DamageNumberError::NoStyles));
    }

    #[test]
    fn test_duplicate_style_type() {
        let params = UiDamageNumberV1Params::new(16, 24)
            .with_outline_width(2)
            .with_style(DamageNumberStyle::new(
                DamageNumberStyleType::Normal,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ))
            .with_style(DamageNumberStyle::new(
                DamageNumberStyleType::Normal,
                [0.9, 0.9, 0.9, 1.0],
                [0.1, 0.1, 0.1, 1.0],
            ));

        let err = generate_damage_number(&params, 42).unwrap_err();
        assert!(matches!(err, DamageNumberError::DuplicateStyleType(_)));
    }

    #[test]
    fn test_invalid_glyph_size_too_small() {
        let params = UiDamageNumberV1Params {
            glyph_size: [4, 4],
            outline_width: 1,
            padding: 2,
            styles: vec![DamageNumberStyle::new(
                DamageNumberStyleType::Normal,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            )],
        };

        let err = generate_damage_number(&params, 42).unwrap_err();
        assert!(matches!(err, DamageNumberError::InvalidGlyphSize(4, 4)));
    }

    #[test]
    fn test_invalid_glyph_size_too_large() {
        let params = UiDamageNumberV1Params {
            glyph_size: [256, 256],
            outline_width: 1,
            padding: 2,
            styles: vec![DamageNumberStyle::new(
                DamageNumberStyleType::Normal,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            )],
        };

        let err = generate_damage_number(&params, 42).unwrap_err();
        assert!(matches!(err, DamageNumberError::InvalidGlyphSize(256, 256)));
    }

    #[test]
    fn test_invalid_outline_width() {
        let params = UiDamageNumberV1Params {
            glyph_size: [16, 24],
            outline_width: 0,
            padding: 2,
            styles: vec![DamageNumberStyle::new(
                DamageNumberStyleType::Normal,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            )],
        };

        let err = generate_damage_number(&params, 42).unwrap_err();
        assert!(matches!(err, DamageNumberError::InvalidOutlineWidth(0)));
    }

    #[test]
    fn test_invalid_color() {
        let params = UiDamageNumberV1Params::new(16, 24)
            .with_outline_width(2)
            .with_style(DamageNumberStyle::new(
                DamageNumberStyleType::Normal,
                [1.5, 0.0, 0.0, 1.0], // Invalid: > 1.0
                [0.0, 0.0, 0.0, 1.0],
            ));

        let err = generate_damage_number(&params, 42).unwrap_err();
        assert!(matches!(err, DamageNumberError::InvalidColor(_)));
    }

    #[test]
    fn test_invalid_scale() {
        let params = UiDamageNumberV1Params::new(16, 24)
            .with_outline_width(2)
            .with_style(
                DamageNumberStyle::new(
                    DamageNumberStyleType::Normal,
                    [1.0, 1.0, 1.0, 1.0],
                    [0.0, 0.0, 0.0, 1.0],
                )
                .with_scale(3.0), // Invalid: > 2.0
            );

        let err = generate_damage_number(&params, 42).unwrap_err();
        assert!(matches!(err, DamageNumberError::InvalidScale(_)));
    }

    #[test]
    fn test_uv_coordinates_normalized() {
        let params = make_basic_params();
        let result = generate_damage_number(&params, 42).unwrap();

        for style in &result.metadata.styles {
            for glyph in &style.glyphs {
                assert!(glyph.uv[0] >= 0.0 && glyph.uv[0] <= 1.0);
                assert!(glyph.uv[1] >= 0.0 && glyph.uv[1] <= 1.0);
                assert!(glyph.uv[2] >= 0.0 && glyph.uv[2] <= 1.0);
                assert!(glyph.uv[3] >= 0.0 && glyph.uv[3] <= 1.0);
                assert!(glyph.uv[2] > glyph.uv[0]); // u_max > u_min
                assert!(glyph.uv[3] > glyph.uv[1]); // v_max > v_min
            }
        }
    }

    #[test]
    fn test_glow_effect() {
        let params = UiDamageNumberV1Params::new(16, 24)
            .with_outline_width(2)
            .with_style(
                DamageNumberStyle::new(
                    DamageNumberStyleType::Critical,
                    [1.0, 0.9, 0.0, 1.0],
                    [1.0, 0.0, 0.0, 1.0],
                )
                .with_glow([1.0, 0.5, 0.0, 0.5]),
            );

        let result = generate_damage_number(&params, 42).unwrap();
        assert!(!result.png_data.is_empty());
        assert_eq!(result.metadata.styles.len(), 1);
    }

    #[test]
    fn test_scale_pattern() {
        // Simple 2x2 pattern
        let pattern = vec![1, 0, 0, 1];
        let scaled = scale_pattern(&pattern, 2, 2, 4, 4);

        // Should scale to 4x4
        assert_eq!(scaled.len(), 16);
        // Top-left quadrant should be 1
        assert_eq!(scaled[0], 1);
        assert_eq!(scaled[1], 1);
        assert_eq!(scaled[4], 1);
        assert_eq!(scaled[5], 1);
    }
}
