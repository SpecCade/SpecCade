//! SSIM (Structural Similarity Index) computation.
//!
//! Provides deterministic implementation of SSIM for image comparison.

use super::{round_f64, FLOAT_PRECISION};

/// SSIM constants (from the original paper).
/// c1 = (K1 * L)^2 where K1 = 0.01, L = 255 (8-bit range)
pub(super) const SSIM_C1: f64 = 6.5025; // (0.01 * 255)^2
/// c2 = (K2 * L)^2 where K2 = 0.03, L = 255
pub(super) const SSIM_C2: f64 = 58.5225; // (0.03 * 255)^2

/// Window size for SSIM calculation.
const SSIM_WINDOW_SIZE: usize = 8;

/// Calculate SSIM between two images.
///
/// Both images must be the same dimensions. Returns average SSIM across all windows.
/// Uses 8x8 non-overlapping windows.
pub fn calculate_ssim(
    pixels_a: &[u8],
    pixels_b: &[u8],
    width: u32,
    height: u32,
    channels: u8,
) -> f64 {
    if pixels_a.len() != pixels_b.len() {
        return 0.0;
    }

    let w = width as usize;
    let h = height as usize;
    let ch = channels as usize;

    // Convert to luminance (single channel for comparison)
    let lum_a = to_luminance(pixels_a, w, h, ch);
    let lum_b = to_luminance(pixels_b, w, h, ch);

    // Calculate SSIM using 8x8 windows
    let mut ssim_sum = 0.0;
    let mut window_count = 0;

    let win = SSIM_WINDOW_SIZE;
    let num_windows_x = w / win;
    let num_windows_y = h / win;

    if num_windows_x == 0 || num_windows_y == 0 {
        // Image too small for windowed SSIM, compute global
        return compute_ssim_global(&lum_a, &lum_b);
    }

    for wy in 0..num_windows_y {
        for wx in 0..num_windows_x {
            let x_start = wx * win;
            let y_start = wy * win;

            // Extract window pixels
            let mut win_a = Vec::with_capacity(win * win);
            let mut win_b = Vec::with_capacity(win * win);

            for dy in 0..win {
                for dx in 0..win {
                    let idx = (y_start + dy) * w + (x_start + dx);
                    win_a.push(lum_a[idx]);
                    win_b.push(lum_b[idx]);
                }
            }

            let ssim = compute_ssim_window(&win_a, &win_b);
            ssim_sum += ssim;
            window_count += 1;
        }
    }

    if window_count > 0 {
        round_f64(ssim_sum / window_count as f64, FLOAT_PRECISION)
    } else {
        0.0
    }
}

/// Convert pixel data to luminance values (0-255 range as f64).
fn to_luminance(pixels: &[u8], width: usize, height: usize, channels: usize) -> Vec<f64> {
    let mut lum = Vec::with_capacity(width * height);

    for i in 0..(width * height) {
        let offset = i * channels;
        let l = match channels {
            1 => pixels[offset] as f64,
            2 => pixels[offset] as f64, // Grayscale + alpha
            3 | 4 => {
                // RGB or RGBA - use ITU-R BT.601 coefficients
                let r = pixels[offset] as f64;
                let g = pixels[offset + 1] as f64;
                let b = pixels[offset + 2] as f64;
                0.299 * r + 0.587 * g + 0.114 * b
            }
            _ => 0.0,
        };
        lum.push(l);
    }

    lum
}

/// Compute SSIM for a single window.
fn compute_ssim_window(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    if n == 0.0 {
        return 0.0;
    }

    // Mean
    let mean_a: f64 = a.iter().sum::<f64>() / n;
    let mean_b: f64 = b.iter().sum::<f64>() / n;

    // Variance and covariance
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    let mut cov_ab = 0.0;

    for i in 0..a.len() {
        let diff_a = a[i] - mean_a;
        let diff_b = b[i] - mean_b;
        var_a += diff_a * diff_a;
        var_b += diff_b * diff_b;
        cov_ab += diff_a * diff_b;
    }

    var_a /= n;
    var_b /= n;
    cov_ab /= n;

    // SSIM formula
    let numerator = (2.0 * mean_a * mean_b + SSIM_C1) * (2.0 * cov_ab + SSIM_C2);
    let denominator = (mean_a * mean_a + mean_b * mean_b + SSIM_C1) * (var_a + var_b + SSIM_C2);

    if denominator > 0.0 {
        numerator / denominator
    } else {
        1.0 // Identical black images
    }
}

/// Compute global SSIM for small images.
fn compute_ssim_global(a: &[f64], b: &[f64]) -> f64 {
    round_f64(compute_ssim_window(a, b), FLOAT_PRECISION)
}
