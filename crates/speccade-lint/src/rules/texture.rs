//! Texture quality lint rules.
//!
//! Rules for detecting perceptual problems in generated texture assets.

use crate::report::{AssetType, LintIssue, Severity};
use crate::rules::{AssetData, LintRule};
use speccade_spec::Spec;

/// Returns all texture lint rules.
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        // Error-level rules
        Box::new(AllBlackRule),
        Box::new(AllWhiteRule),
        Box::new(CorruptAlphaRule),
        // Warning-level rules
        Box::new(LowContrastRule),
        Box::new(BandingRule),
        Box::new(TileSeamRule),
        Box::new(NoisyRule),
        Box::new(ColorCastRule),
        // Info-level rules
        Box::new(PowerOfTwoRule),
        Box::new(LargeSolidRegionsRule),
    ]
}

// ============================================================================
// PNG Decoding and Image Data
// ============================================================================

/// Decoded PNG image data.
struct ImageData {
    /// Width in pixels.
    width: u32,
    /// Height in pixels.
    height: u32,
    /// Color type (RGB, RGBA, Grayscale, etc.).
    color_type: png::ColorType,
    /// Raw pixel data (interleaved channels).
    pixels: Vec<u8>,
    /// Bytes per pixel.
    bytes_per_pixel: usize,
}

impl ImageData {
    /// Returns the number of channels in the image.
    fn channels(&self) -> usize {
        match self.color_type {
            png::ColorType::Grayscale => 1,
            png::ColorType::GrayscaleAlpha => 2,
            png::ColorType::Rgb => 3,
            png::ColorType::Rgba => 4,
            png::ColorType::Indexed => 1,
        }
    }

    /// Returns true if the image has an alpha channel.
    fn has_alpha(&self) -> bool {
        matches!(
            self.color_type,
            png::ColorType::GrayscaleAlpha | png::ColorType::Rgba
        )
    }

    /// Returns the pixel at (x, y) as a slice of bytes.
    fn pixel_at(&self, x: u32, y: u32) -> &[u8] {
        let idx = (y as usize * self.width as usize + x as usize) * self.bytes_per_pixel;
        &self.pixels[idx..idx + self.bytes_per_pixel]
    }

    /// Computes the luminance of a pixel (0-255).
    fn pixel_luminance(&self, x: u32, y: u32) -> u8 {
        let pixel = self.pixel_at(x, y);
        match self.color_type {
            png::ColorType::Grayscale | png::ColorType::GrayscaleAlpha => pixel[0],
            png::ColorType::Rgb | png::ColorType::Rgba => {
                // Standard luminance formula
                let r = pixel[0] as f32;
                let g = pixel[1] as f32;
                let b = pixel[2] as f32;
                (0.299 * r + 0.587 * g + 0.114 * b) as u8
            }
            png::ColorType::Indexed => pixel[0],
        }
    }

    /// Returns the RGB values at (x, y), or grayscale repeated if not RGB.
    fn rgb_at(&self, x: u32, y: u32) -> (u8, u8, u8) {
        let pixel = self.pixel_at(x, y);
        match self.color_type {
            png::ColorType::Grayscale | png::ColorType::GrayscaleAlpha => {
                (pixel[0], pixel[0], pixel[0])
            }
            png::ColorType::Rgb | png::ColorType::Rgba => (pixel[0], pixel[1], pixel[2]),
            png::ColorType::Indexed => (pixel[0], pixel[0], pixel[0]),
        }
    }

    /// Returns the alpha value at (x, y), or 255 if no alpha channel.
    fn alpha_at(&self, x: u32, y: u32) -> u8 {
        if !self.has_alpha() {
            return 255;
        }
        let pixel = self.pixel_at(x, y);
        match self.color_type {
            png::ColorType::GrayscaleAlpha => pixel[1],
            png::ColorType::Rgba => pixel[3],
            _ => 255,
        }
    }
}

/// Attempts to decode a PNG from the given bytes.
fn decode_png(bytes: &[u8]) -> Result<ImageData, String> {
    let decoder = png::Decoder::new(bytes);
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("failed to read PNG header: {}", e))?;

    let mut pixels = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut pixels)
        .map_err(|e| format!("failed to decode PNG frame: {}", e))?;

    pixels.truncate(info.buffer_size());

    let bytes_per_pixel = match info.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        png::ColorType::Indexed => 1,
    };

    Ok(ImageData {
        width: info.width,
        height: info.height,
        color_type: info.color_type,
        pixels,
        bytes_per_pixel,
    })
}

// ============================================================================
// Statistical Analysis Helpers
// ============================================================================

/// Computes the minimum and maximum pixel values across all channels.
fn compute_min_max(image: &ImageData) -> (u8, u8) {
    let mut min_val = 255u8;
    let mut max_val = 0u8;

    for y in 0..image.height {
        for x in 0..image.width {
            let lum = image.pixel_luminance(x, y);
            min_val = min_val.min(lum);
            max_val = max_val.max(lum);
        }
    }

    (min_val, max_val)
}

/// Computes the standard deviation of pixel luminance values.
fn compute_std_dev(image: &ImageData) -> f64 {
    let total_pixels = (image.width * image.height) as f64;
    if total_pixels == 0.0 {
        return 0.0;
    }

    // Compute mean
    let mut sum = 0.0f64;
    for y in 0..image.height {
        for x in 0..image.width {
            sum += image.pixel_luminance(x, y) as f64;
        }
    }
    let mean = sum / total_pixels;

    // Compute variance
    let mut variance_sum = 0.0f64;
    for y in 0..image.height {
        for x in 0..image.width {
            let diff = image.pixel_luminance(x, y) as f64 - mean;
            variance_sum += diff * diff;
        }
    }

    (variance_sum / total_pixels).sqrt()
}

/// Counts the number of unique values per channel (R, G, B).
fn count_unique_values_per_channel(image: &ImageData) -> (usize, usize, usize) {
    let mut r_values = [false; 256];
    let mut g_values = [false; 256];
    let mut b_values = [false; 256];

    for y in 0..image.height {
        for x in 0..image.width {
            let (r, g, b) = image.rgb_at(x, y);
            r_values[r as usize] = true;
            g_values[g as usize] = true;
            b_values[b as usize] = true;
        }
    }

    let r_count = r_values.iter().filter(|&&v| v).count();
    let g_count = g_values.iter().filter(|&&v| v).count();
    let b_count = b_values.iter().filter(|&&v| v).count();

    (r_count, g_count, b_count)
}

/// Computes the average value of each RGB channel.
fn compute_channel_averages(image: &ImageData) -> (f64, f64, f64) {
    let total_pixels = (image.width * image.height) as f64;
    if total_pixels == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let mut r_sum = 0.0f64;
    let mut g_sum = 0.0f64;
    let mut b_sum = 0.0f64;

    for y in 0..image.height {
        for x in 0..image.width {
            let (r, g, b) = image.rgb_at(x, y);
            r_sum += r as f64;
            g_sum += g as f64;
            b_sum += b as f64;
        }
    }

    (
        r_sum / total_pixels,
        g_sum / total_pixels,
        b_sum / total_pixels,
    )
}

/// Computes a histogram of pixel values (luminance).
fn compute_histogram(image: &ImageData) -> [u32; 256] {
    let mut histogram = [0u32; 256];

    for y in 0..image.height {
        for x in 0..image.width {
            let lum = image.pixel_luminance(x, y);
            histogram[lum as usize] += 1;
        }
    }

    histogram
}

/// Computes local variance across the image using a window-based approach.
/// Returns the average local variance.
fn compute_average_local_variance(image: &ImageData, window_size: u32) -> f64 {
    if image.width < window_size || image.height < window_size {
        return 0.0;
    }

    let step = window_size.max(1);
    let mut total_variance = 0.0f64;
    let mut window_count = 0u32;

    let mut y = 0;
    while y + window_size <= image.height {
        let mut x = 0;
        while x + window_size <= image.width {
            // Compute mean in window
            let mut sum = 0.0f64;
            let window_pixels = (window_size * window_size) as f64;

            for wy in 0..window_size {
                for wx in 0..window_size {
                    sum += image.pixel_luminance(x + wx, y + wy) as f64;
                }
            }
            let mean = sum / window_pixels;

            // Compute variance in window
            let mut variance_sum = 0.0f64;
            for wy in 0..window_size {
                for wx in 0..window_size {
                    let diff = image.pixel_luminance(x + wx, y + wy) as f64 - mean;
                    variance_sum += diff * diff;
                }
            }

            total_variance += variance_sum / window_pixels;
            window_count += 1;

            x += step;
        }
        y += step;
    }

    if window_count > 0 {
        total_variance / window_count as f64
    } else {
        0.0
    }
}

/// Computes the maximum edge discontinuity (for tileability checking).
/// Compares left edge with right edge and top edge with bottom edge.
fn compute_edge_discontinuity(image: &ImageData) -> f64 {
    if image.width == 0 || image.height == 0 {
        return 0.0;
    }

    // Compare left and right edges
    let mut lr_max_diff = 0.0f64;
    for y in 0..image.height {
        let left_lum = image.pixel_luminance(0, y) as f64;
        let right_lum = image.pixel_luminance(image.width - 1, y) as f64;
        let diff = (left_lum - right_lum).abs();
        lr_max_diff = lr_max_diff.max(diff);
    }

    // Compare top and bottom edges
    let mut tb_max_diff = 0.0f64;
    for x in 0..image.width {
        let top_lum = image.pixel_luminance(x, 0) as f64;
        let bottom_lum = image.pixel_luminance(x, image.height - 1) as f64;
        let diff = (top_lum - bottom_lum).abs();
        tb_max_diff = tb_max_diff.max(diff);
    }

    lr_max_diff.max(tb_max_diff)
}

/// Returns true if n is a power of two.
fn is_power_of_two(n: u32) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

// ============================================================================
// Error-Level Rules
// ============================================================================

/// Rule: texture/all-black
/// Detects images that are entirely black (max pixel value < 5).
pub struct AllBlackRule;

impl LintRule for AllBlackRule {
    fn id(&self) -> &'static str {
        "texture/all-black"
    }

    fn description(&self) -> &'static str {
        "Image is entirely black"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Texture]
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let image = match decode_png(asset.bytes) {
            Ok(img) => img,
            Err(_) => return vec![],
        };

        let (_min_val, max_val) = compute_min_max(&image);

        if max_val < 5 {
            vec![LintIssue::new(
                self.id(),
                self.default_severity(),
                "Image is entirely black",
                "Check noise scale or color ramp",
            )
            .with_actual_value(format!("max_value={}", max_val))
            .with_expected_range(">= 5")]
        } else {
            vec![]
        }
    }
}

/// Rule: texture/all-white
/// Detects images that are entirely white (min pixel value > 250).
pub struct AllWhiteRule;

impl LintRule for AllWhiteRule {
    fn id(&self) -> &'static str {
        "texture/all-white"
    }

    fn description(&self) -> &'static str {
        "Image is entirely white"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Texture]
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let image = match decode_png(asset.bytes) {
            Ok(img) => img,
            Err(_) => return vec![],
        };

        let (min_val, _max_val) = compute_min_max(&image);

        if min_val > 250 {
            vec![LintIssue::new(
                self.id(),
                self.default_severity(),
                "Image is entirely white",
                "Check threshold or invert node",
            )
            .with_actual_value(format!("min_value={}", min_val))
            .with_expected_range("<= 250")]
        } else {
            vec![]
        }
    }
}

/// Rule: texture/corrupt-alpha
/// Detects images with alpha channel that is all 0 or all 255 (uniform).
pub struct CorruptAlphaRule;

impl LintRule for CorruptAlphaRule {
    fn id(&self) -> &'static str {
        "texture/corrupt-alpha"
    }

    fn description(&self) -> &'static str {
        "Alpha channel is uniform (all 0 or all 255)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Texture]
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let image = match decode_png(asset.bytes) {
            Ok(img) => img,
            Err(_) => return vec![],
        };

        // Only check if image has alpha channel
        if !image.has_alpha() {
            return vec![];
        }

        // Check if alpha is uniform (all same value)
        let mut min_alpha = 255u8;
        let mut max_alpha = 0u8;

        for y in 0..image.height {
            for x in 0..image.width {
                let alpha = image.alpha_at(x, y);
                min_alpha = min_alpha.min(alpha);
                max_alpha = max_alpha.max(alpha);
            }
        }

        // Uniform alpha is suspicious - either all 0 (invisible) or all 255 (why have alpha?)
        if min_alpha == max_alpha && (min_alpha == 0 || min_alpha == 255) {
            vec![LintIssue::new(
                self.id(),
                self.default_severity(),
                format!(
                    "Alpha channel is uniform (all {})",
                    if min_alpha == 0 {
                        "transparent"
                    } else {
                        "opaque"
                    }
                ),
                "Check alpha source node",
            )
            .with_actual_value(format!("alpha={}", min_alpha))
            .with_expected_range("variable alpha values")]
        } else {
            vec![]
        }
    }
}

// ============================================================================
// Warning-Level Rules
// ============================================================================

/// Rule: texture/low-contrast
/// Detects images with a narrow value range (std dev < 20).
pub struct LowContrastRule;

impl LintRule for LowContrastRule {
    fn id(&self) -> &'static str {
        "texture/low-contrast"
    }

    fn description(&self) -> &'static str {
        "Image has low contrast (narrow value range)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Texture]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let image = match decode_png(asset.bytes) {
            Ok(img) => img,
            Err(_) => return vec![],
        };

        let std_dev = compute_std_dev(&image);

        if std_dev < 20.0 {
            vec![LintIssue::new(
                self.id(),
                self.default_severity(),
                "Image has low contrast",
                "Increase color_ramp spread",
            )
            .with_actual_value(format!("std_dev={:.2}", std_dev))
            .with_expected_range(">= 20")]
        } else {
            vec![]
        }
    }
}

/// Rule: texture/banding
/// Detects visible color stepping (< 32 unique values per channel).
pub struct BandingRule;

impl LintRule for BandingRule {
    fn id(&self) -> &'static str {
        "texture/banding"
    }

    fn description(&self) -> &'static str {
        "Image shows color banding (too few unique values)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Texture]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let image = match decode_png(asset.bytes) {
            Ok(img) => img,
            Err(_) => return vec![],
        };

        let (r_count, g_count, b_count) = count_unique_values_per_channel(&image);
        let min_count = r_count.min(g_count).min(b_count);

        if min_count < 32 {
            vec![LintIssue::new(
                self.id(),
                self.default_severity(),
                format!(
                    "Image shows color banding (R:{}, G:{}, B:{} unique values)",
                    r_count, g_count, b_count
                ),
                "Add dithering noise",
            )
            .with_actual_value(format!(
                "unique_values: R={}, G={}, B={}",
                r_count, g_count, b_count
            ))
            .with_expected_range(">= 32 per channel")]
        } else {
            vec![]
        }
    }
}

/// Rule: texture/tile-seam
/// Detects edge discontinuity for tileable textures.
pub struct TileSeamRule;

impl LintRule for TileSeamRule {
    fn id(&self) -> &'static str {
        "texture/tile-seam"
    }

    fn description(&self) -> &'static str {
        "Image has visible seams at tile edges"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Texture]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let image = match decode_png(asset.bytes) {
            Ok(img) => img,
            Err(_) => return vec![],
        };

        let max_discontinuity = compute_edge_discontinuity(&image);
        let threshold = 50.0; // Threshold for noticeable seam

        if max_discontinuity > threshold {
            vec![LintIssue::new(
                self.id(),
                self.default_severity(),
                format!(
                    "Image has visible tile seams (max edge diff: {:.1})",
                    max_discontinuity
                ),
                "Enable seamless mode",
            )
            .with_actual_value(format!("edge_diff={:.1}", max_discontinuity))
            .with_expected_range("<= 50")]
        } else {
            vec![]
        }
    }
}

/// Rule: texture/noisy
/// Detects excessive high-frequency noise.
pub struct NoisyRule;

impl LintRule for NoisyRule {
    fn id(&self) -> &'static str {
        "texture/noisy"
    }

    fn description(&self) -> &'static str {
        "Image has excessive high-frequency noise"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Texture]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let image = match decode_png(asset.bytes) {
            Ok(img) => img,
            Err(_) => return vec![],
        };

        // Use small window for local variance to detect high-frequency noise
        let local_variance = compute_average_local_variance(&image, 4);
        let threshold = 2500.0; // High local variance indicates noise

        if local_variance > threshold {
            vec![LintIssue::new(
                self.id(),
                self.default_severity(),
                format!(
                    "Image has excessive noise (local variance: {:.1})",
                    local_variance
                ),
                "Reduce octaves or add blur",
            )
            .with_actual_value(format!("local_variance={:.1}", local_variance))
            .with_expected_range("<= 2500")]
        } else {
            vec![]
        }
    }
}

/// Rule: texture/color-cast
/// Detects strong single-channel dominance.
pub struct ColorCastRule;

impl LintRule for ColorCastRule {
    fn id(&self) -> &'static str {
        "texture/color-cast"
    }

    fn description(&self) -> &'static str {
        "Image has a strong color cast (one channel dominates)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Texture]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let image = match decode_png(asset.bytes) {
            Ok(img) => img,
            Err(_) => return vec![],
        };

        // Skip grayscale images
        if image.channels() < 3 {
            return vec![];
        }

        let (r_avg, g_avg, b_avg) = compute_channel_averages(&image);

        // Find dominant channel and compare to others
        let min_avg = r_avg.min(g_avg).min(b_avg);

        // Avoid division by zero for very dark images
        if min_avg < 10.0 {
            return vec![];
        }

        let r_ratio = r_avg / min_avg;
        let g_ratio = g_avg / min_avg;
        let b_ratio = b_avg / min_avg;

        let threshold = 1.5;

        let (dominant, ratio) = if r_ratio > g_ratio && r_ratio > b_ratio && r_ratio > threshold {
            ("red", r_ratio)
        } else if g_ratio > r_ratio && g_ratio > b_ratio && g_ratio > threshold {
            ("green", g_ratio)
        } else if b_ratio > r_ratio && b_ratio > g_ratio && b_ratio > threshold {
            ("blue", b_ratio)
        } else {
            return vec![];
        };

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Image has {} color cast ({:.2}x other channels)",
                dominant, ratio
            ),
            "Balance color ramp",
        )
        .with_actual_value(format!(
            "R={:.1}, G={:.1}, B={:.1}",
            r_avg, g_avg, b_avg
        ))
        .with_expected_range("channels within 1.5x of each other")]
    }
}

// ============================================================================
// Info-Level Rules
// ============================================================================

/// Rule: texture/power-of-two
/// Detects non-power-of-two dimensions.
pub struct PowerOfTwoRule;

impl LintRule for PowerOfTwoRule {
    fn id(&self) -> &'static str {
        "texture/power-of-two"
    }

    fn description(&self) -> &'static str {
        "Image dimensions are not power-of-two"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Texture]
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let image = match decode_png(asset.bytes) {
            Ok(img) => img,
            Err(_) => return vec![],
        };

        let width_pot = is_power_of_two(image.width);
        let height_pot = is_power_of_two(image.height);

        if !width_pot || !height_pot {
            let mut issues = Vec::new();

            if !width_pot {
                issues.push(format!("width={}", image.width));
            }
            if !height_pot {
                issues.push(format!("height={}", image.height));
            }

            vec![LintIssue::new(
                self.id(),
                self.default_severity(),
                format!("Image has non-power-of-two dimensions ({})", issues.join(", ")),
                "Use 256/512/1024",
            )
            .with_actual_value(format!("{}x{}", image.width, image.height))
            .with_expected_range("power-of-two (e.g., 256, 512, 1024)")]
        } else {
            vec![]
        }
    }
}

/// Rule: texture/large-solid-regions
/// Detects images with >25% pixels being identical.
pub struct LargeSolidRegionsRule;

impl LintRule for LargeSolidRegionsRule {
    fn id(&self) -> &'static str {
        "texture/large-solid-regions"
    }

    fn description(&self) -> &'static str {
        "Image has large solid regions (>25% identical pixels)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Texture]
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let image = match decode_png(asset.bytes) {
            Ok(img) => img,
            Err(_) => return vec![],
        };

        let histogram = compute_histogram(&image);
        let total_pixels = image.width * image.height;

        if total_pixels == 0 {
            return vec![];
        }

        // Find the maximum histogram value (most common luminance)
        let max_count = *histogram.iter().max().unwrap_or(&0);
        let percentage = (max_count as f64 / total_pixels as f64) * 100.0;

        if percentage > 25.0 {
            vec![LintIssue::new(
                self.id(),
                self.default_severity(),
                format!("Image has large solid regions ({:.1}% identical pixels)", percentage),
                "Add subtle variation",
            )
            .with_actual_value(format!("{:.1}%", percentage))
            .with_expected_range("<= 25%")]
        } else {
            vec![]
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Creates a minimal PNG image with the given pixel data.
    fn create_test_png(width: u32, height: u32, pixels: &[u8], color_type: png::ColorType) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buffer, width, height);
            encoder.set_color(color_type);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(pixels).unwrap();
        }
        buffer
    }

    fn make_asset_data(bytes: &[u8]) -> AssetData<'_> {
        AssetData {
            path: Path::new("test.png"),
            bytes,
        }
    }

    // ============================================================================
    // Error-Level Rule Tests
    // ============================================================================

    #[test]
    fn test_all_black_rule_triggers() {
        // 2x2 black image
        let pixels = vec![0u8; 2 * 2 * 3]; // RGB
        let png_data = create_test_png(2, 2, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = AllBlackRule;
        let issues = rule.check(&asset, None);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "texture/all-black");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn test_all_black_rule_passes() {
        // 2x2 gray image (value 128)
        let pixels = vec![128u8; 2 * 2 * 3]; // RGB
        let png_data = create_test_png(2, 2, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = AllBlackRule;
        let issues = rule.check(&asset, None);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_all_white_rule_triggers() {
        // 2x2 white image
        let pixels = vec![255u8; 2 * 2 * 3]; // RGB
        let png_data = create_test_png(2, 2, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = AllWhiteRule;
        let issues = rule.check(&asset, None);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "texture/all-white");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn test_all_white_rule_passes() {
        // 2x2 gray image (value 128)
        let pixels = vec![128u8; 2 * 2 * 3]; // RGB
        let png_data = create_test_png(2, 2, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = AllWhiteRule;
        let issues = rule.check(&asset, None);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_corrupt_alpha_rule_triggers_all_zero() {
        // 2x2 RGBA image with all alpha = 0
        let mut pixels = Vec::new();
        for _ in 0..4 {
            pixels.extend_from_slice(&[128, 128, 128, 0]); // Gray with 0 alpha
        }
        let png_data = create_test_png(2, 2, &pixels, png::ColorType::Rgba);
        let asset = make_asset_data(&png_data);

        let rule = CorruptAlphaRule;
        let issues = rule.check(&asset, None);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "texture/corrupt-alpha");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn test_corrupt_alpha_rule_triggers_all_255() {
        // 2x2 RGBA image with all alpha = 255
        let mut pixels = Vec::new();
        for _ in 0..4 {
            pixels.extend_from_slice(&[128, 128, 128, 255]); // Gray with 255 alpha
        }
        let png_data = create_test_png(2, 2, &pixels, png::ColorType::Rgba);
        let asset = make_asset_data(&png_data);

        let rule = CorruptAlphaRule;
        let issues = rule.check(&asset, None);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "texture/corrupt-alpha");
    }

    #[test]
    fn test_corrupt_alpha_rule_passes_with_varied_alpha() {
        // 2x2 RGBA image with varied alpha
        let pixels = vec![
            128, 128, 128, 0,   // pixel 0: transparent
            128, 128, 128, 128, // pixel 1: semi-transparent
            128, 128, 128, 200, // pixel 2: mostly opaque
            128, 128, 128, 255, // pixel 3: opaque
        ];
        let png_data = create_test_png(2, 2, &pixels, png::ColorType::Rgba);
        let asset = make_asset_data(&png_data);

        let rule = CorruptAlphaRule;
        let issues = rule.check(&asset, None);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_corrupt_alpha_skips_rgb() {
        // RGB image (no alpha)
        let pixels = vec![128u8; 2 * 2 * 3];
        let png_data = create_test_png(2, 2, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = CorruptAlphaRule;
        let issues = rule.check(&asset, None);

        assert!(issues.is_empty());
    }

    // ============================================================================
    // Warning-Level Rule Tests
    // ============================================================================

    #[test]
    fn test_low_contrast_rule_triggers() {
        // 4x4 image with very similar gray values (low std dev)
        let mut pixels = Vec::new();
        for i in 0..16 {
            let val = 127 + (i % 3) as u8; // Values 127, 128, 129
            pixels.extend_from_slice(&[val, val, val]);
        }
        let png_data = create_test_png(4, 4, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = LowContrastRule;
        let issues = rule.check(&asset, None);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "texture/low-contrast");
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn test_low_contrast_rule_passes() {
        // 2x2 image with high contrast (black and white pixels)
        let pixels = vec![
            0, 0, 0,       // black
            255, 255, 255, // white
            0, 0, 0,       // black
            255, 255, 255, // white
        ];
        let png_data = create_test_png(2, 2, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = LowContrastRule;
        let issues = rule.check(&asset, None);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_banding_rule_triggers() {
        // 4x4 image with only 4 unique values per channel
        let values = [0u8, 85, 170, 255];
        let mut pixels = Vec::new();
        for i in 0..16 {
            let val = values[i % 4];
            pixels.extend_from_slice(&[val, val, val]);
        }
        let png_data = create_test_png(4, 4, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = BandingRule;
        let issues = rule.check(&asset, None);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "texture/banding");
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn test_banding_rule_passes() {
        // 8x8 image with many unique values
        let mut pixels = Vec::new();
        for i in 0..64 {
            let val = (i * 4) as u8; // 0, 4, 8, ... 252 (64 unique values)
            pixels.extend_from_slice(&[val, val, val]);
        }
        let png_data = create_test_png(8, 8, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = BandingRule;
        let issues = rule.check(&asset, None);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_tile_seam_rule_triggers() {
        // 4x4 image with big edge discontinuity
        // Left edge is black, right edge is white
        let mut pixels = vec![0u8; 4 * 4 * 3];
        // Set right column to white
        for y in 0..4 {
            let idx = (y * 4 + 3) * 3;
            pixels[idx] = 255;
            pixels[idx + 1] = 255;
            pixels[idx + 2] = 255;
        }
        let png_data = create_test_png(4, 4, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = TileSeamRule;
        let issues = rule.check(&asset, None);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "texture/tile-seam");
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn test_tile_seam_rule_passes() {
        // 4x4 uniform gray image (no seams)
        let pixels = vec![128u8; 4 * 4 * 3];
        let png_data = create_test_png(4, 4, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = TileSeamRule;
        let issues = rule.check(&asset, None);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_noisy_rule_triggers() {
        // 8x8 image with high local variance (random-like pattern)
        let mut pixels = Vec::new();
        for i in 0..64 {
            let val = if i % 2 == 0 { 0 } else { 255 };
            pixels.extend_from_slice(&[val, val, val]);
        }
        let png_data = create_test_png(8, 8, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = NoisyRule;
        let issues = rule.check(&asset, None);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "texture/noisy");
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn test_noisy_rule_passes() {
        // 8x8 uniform image (no noise)
        let pixels = vec![128u8; 8 * 8 * 3];
        let png_data = create_test_png(8, 8, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = NoisyRule;
        let issues = rule.check(&asset, None);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_color_cast_rule_triggers() {
        // 2x2 image with strong red cast
        let pixels = vec![
            200, 50, 50,  // red-heavy
            200, 50, 50,
            200, 50, 50,
            200, 50, 50,
        ];
        let png_data = create_test_png(2, 2, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = ColorCastRule;
        let issues = rule.check(&asset, None);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "texture/color-cast");
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn test_color_cast_rule_passes() {
        // 2x2 balanced gray image
        let pixels = vec![128u8; 2 * 2 * 3];
        let png_data = create_test_png(2, 2, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = ColorCastRule;
        let issues = rule.check(&asset, None);

        assert!(issues.is_empty());
    }

    // ============================================================================
    // Info-Level Rule Tests
    // ============================================================================

    #[test]
    fn test_power_of_two_rule_triggers() {
        // 3x3 image (not power of two)
        let pixels = vec![128u8; 3 * 3 * 3];
        let png_data = create_test_png(3, 3, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = PowerOfTwoRule;
        let issues = rule.check(&asset, None);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "texture/power-of-two");
        assert_eq!(issues[0].severity, Severity::Info);
    }

    #[test]
    fn test_power_of_two_rule_passes() {
        // 4x4 image (power of two)
        let pixels = vec![128u8; 4 * 4 * 3];
        let png_data = create_test_png(4, 4, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = PowerOfTwoRule;
        let issues = rule.check(&asset, None);

        assert!(issues.is_empty());
    }

    #[test]
    fn test_large_solid_regions_rule_triggers() {
        // 4x4 image where all pixels are the same gray
        let pixels = vec![128u8; 4 * 4 * 3];
        let png_data = create_test_png(4, 4, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = LargeSolidRegionsRule;
        let issues = rule.check(&asset, None);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "texture/large-solid-regions");
        assert_eq!(issues[0].severity, Severity::Info);
    }

    #[test]
    fn test_large_solid_regions_rule_passes() {
        // 8x8 image with varied luminance (no dominant value)
        let mut pixels = Vec::new();
        for i in 0..64 {
            let val = (i * 4) as u8;
            pixels.extend_from_slice(&[val, val, val]);
        }
        let png_data = create_test_png(8, 8, &pixels, png::ColorType::Rgb);
        let asset = make_asset_data(&png_data);

        let rule = LargeSolidRegionsRule;
        let issues = rule.check(&asset, None);

        assert!(issues.is_empty());
    }

    // ============================================================================
    // all_rules() test
    // ============================================================================

    #[test]
    fn test_all_rules_returns_10_rules() {
        let rules = all_rules();
        assert_eq!(rules.len(), 10);

        // Verify rule IDs
        let ids: Vec<_> = rules.iter().map(|r| r.id()).collect();
        assert!(ids.contains(&"texture/all-black"));
        assert!(ids.contains(&"texture/all-white"));
        assert!(ids.contains(&"texture/corrupt-alpha"));
        assert!(ids.contains(&"texture/low-contrast"));
        assert!(ids.contains(&"texture/banding"));
        assert!(ids.contains(&"texture/tile-seam"));
        assert!(ids.contains(&"texture/noisy"));
        assert!(ids.contains(&"texture/color-cast"));
        assert!(ids.contains(&"texture/power-of-two"));
        assert!(ids.contains(&"texture/large-solid-regions"));
    }

    #[test]
    fn test_all_rules_apply_to_texture() {
        let rules = all_rules();
        for rule in &rules {
            assert!(
                rule.applies_to().contains(&AssetType::Texture),
                "Rule {} should apply to Texture",
                rule.id()
            );
        }
    }

    // ============================================================================
    // Helper function tests
    // ============================================================================

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(256));
        assert!(is_power_of_two(512));
        assert!(is_power_of_two(1024));

        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(5));
        assert!(!is_power_of_two(100));
        assert!(!is_power_of_two(300));
    }
}
