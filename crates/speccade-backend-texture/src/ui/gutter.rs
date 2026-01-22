//! Shared gutter rendering utilities for UI atlases.
//!
//! Gutters are padding regions around atlas entries that replicate edge pixels
//! to prevent sampling artifacts during mipmapping.

use crate::color::Color;
use crate::maps::TextureBuffer;

/// Render a rectangular region into the atlas with mip-safe gutter.
///
/// The gutter pixels are filled by replicating the edge pixels of the region.
///
/// # Arguments
/// * `atlas` - The texture buffer to render into
/// * `x` - X position of the region content area
/// * `y` - Y position of the region content area
/// * `w` - Width of the region content area
/// * `h` - Height of the region content area
/// * `color` - Color to fill the region with
/// * `padding` - Padding/gutter width in pixels
pub fn render_region_with_gutter(
    atlas: &mut TextureBuffer,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    color: Color,
    padding: u32,
) {
    // Fill the region content area
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            if px < atlas.width && py < atlas.height {
                atlas.set(px, py, color);
            }
        }
    }

    if padding == 0 {
        return;
    }

    // Fill gutters by replicating edge pixels
    fill_horizontal_gutters(atlas, x, y, w, h, color, padding);
    fill_vertical_gutters(atlas, x, y, w, h, color, padding);
    fill_corner_gutters(atlas, x, y, w, h, color, padding);
}

/// Fill top and bottom horizontal gutters.
fn fill_horizontal_gutters(
    atlas: &mut TextureBuffer,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    color: Color,
    padding: u32,
) {
    // Top gutter
    for gy in 0..padding {
        for dx in 0..w {
            let gutter_x = x + dx;
            let gutter_y = y.saturating_sub(padding) + gy;
            if gutter_x < atlas.width && gutter_y < atlas.height {
                atlas.set(gutter_x, gutter_y, color);
            }
        }
    }

    // Bottom gutter
    for gy in 0..padding {
        for dx in 0..w {
            let gutter_x = x + dx;
            let gutter_y = y + h + gy;
            if gutter_x < atlas.width && gutter_y < atlas.height {
                atlas.set(gutter_x, gutter_y, color);
            }
        }
    }
}

/// Fill left and right vertical gutters.
fn fill_vertical_gutters(
    atlas: &mut TextureBuffer,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    color: Color,
    padding: u32,
) {
    // Left gutter
    for dy in 0..h {
        for gx in 0..padding {
            let gutter_x = x.saturating_sub(padding) + gx;
            let gutter_y = y + dy;
            if gutter_x < atlas.width && gutter_y < atlas.height {
                atlas.set(gutter_x, gutter_y, color);
            }
        }
    }

    // Right gutter
    for dy in 0..h {
        for gx in 0..padding {
            let gutter_x = x + w + gx;
            let gutter_y = y + dy;
            if gutter_x < atlas.width && gutter_y < atlas.height {
                atlas.set(gutter_x, gutter_y, color);
            }
        }
    }
}

/// Fill corner gutters (top-left, top-right, bottom-left, bottom-right).
fn fill_corner_gutters(
    atlas: &mut TextureBuffer,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    color: Color,
    padding: u32,
) {
    // Top-left
    for gy in 0..padding {
        for gx in 0..padding {
            let gutter_x = x.saturating_sub(padding) + gx;
            let gutter_y = y.saturating_sub(padding) + gy;
            if gutter_x < atlas.width && gutter_y < atlas.height {
                atlas.set(gutter_x, gutter_y, color);
            }
        }
    }

    // Top-right
    for gy in 0..padding {
        for gx in 0..padding {
            let gutter_x = x + w + gx;
            let gutter_y = y.saturating_sub(padding) + gy;
            if gutter_x < atlas.width && gutter_y < atlas.height {
                atlas.set(gutter_x, gutter_y, color);
            }
        }
    }

    // Bottom-left
    for gy in 0..padding {
        for gx in 0..padding {
            let gutter_x = x.saturating_sub(padding) + gx;
            let gutter_y = y + h + gy;
            if gutter_x < atlas.width && gutter_y < atlas.height {
                atlas.set(gutter_x, gutter_y, color);
            }
        }
    }

    // Bottom-right
    for gy in 0..padding {
        for gx in 0..padding {
            let gutter_x = x + w + gx;
            let gutter_y = y + h + gy;
            if gutter_x < atlas.width && gutter_y < atlas.height {
                atlas.set(gutter_x, gutter_y, color);
            }
        }
    }
}

/// Validate that RGBA color values are in [0, 1] range.
pub fn validate_color(color: &[f64; 4], name: &str) -> Result<(), String> {
    for &val in color {
        if !(0.0..=1.0).contains(&val) {
            return Err(format!(
                "Invalid color for '{}': RGBA values must be in [0, 1] range",
                name
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_region_without_gutter() {
        let mut atlas = TextureBuffer::new(64, 64, Color::rgba(0.0, 0.0, 0.0, 0.0));
        let color = Color::rgba(1.0, 0.0, 0.0, 1.0);

        render_region_with_gutter(&mut atlas, 10, 10, 8, 8, color, 0);

        // Check that the region is filled
        assert_eq!(atlas.get(10, 10), color);
        assert_eq!(atlas.get(17, 17), color);

        // Check that outside the region is still background
        assert_eq!(atlas.get(9, 10), Color::rgba(0.0, 0.0, 0.0, 0.0));
        assert_eq!(atlas.get(18, 10), Color::rgba(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_render_region_with_gutter() {
        let mut atlas = TextureBuffer::new(64, 64, Color::rgba(0.0, 0.0, 0.0, 0.0));
        let color = Color::rgba(1.0, 0.0, 0.0, 1.0);

        render_region_with_gutter(&mut atlas, 10, 10, 8, 8, color, 2);

        // Check content area
        assert_eq!(atlas.get(10, 10), color);
        assert_eq!(atlas.get(17, 17), color);

        // Check top gutter
        assert_eq!(atlas.get(10, 8), color);
        assert_eq!(atlas.get(10, 9), color);

        // Check bottom gutter
        assert_eq!(atlas.get(10, 18), color);
        assert_eq!(atlas.get(10, 19), color);

        // Check left gutter
        assert_eq!(atlas.get(8, 10), color);
        assert_eq!(atlas.get(9, 10), color);

        // Check right gutter
        assert_eq!(atlas.get(18, 10), color);
        assert_eq!(atlas.get(19, 10), color);

        // Check corner gutters
        assert_eq!(atlas.get(8, 8), color); // top-left
        assert_eq!(atlas.get(19, 8), color); // top-right
        assert_eq!(atlas.get(8, 19), color); // bottom-left
        assert_eq!(atlas.get(19, 19), color); // bottom-right
    }

    #[test]
    fn test_validate_color_valid() {
        assert!(validate_color(&[0.0, 0.5, 1.0, 1.0], "test").is_ok());
        assert!(validate_color(&[0.0, 0.0, 0.0, 0.0], "test").is_ok());
        assert!(validate_color(&[1.0, 1.0, 1.0, 1.0], "test").is_ok());
    }

    #[test]
    fn test_validate_color_invalid() {
        assert!(validate_color(&[1.5, 0.0, 0.0, 1.0], "test").is_err());
        assert!(validate_color(&[0.0, -0.1, 0.0, 1.0], "test").is_err());
        assert!(validate_color(&[0.0, 0.0, 0.0, 2.0], "test").is_err());
    }
}
