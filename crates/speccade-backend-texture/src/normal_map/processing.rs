//! Height map processing functions for normal map generation.

use speccade_spec::recipe::texture::NormalMapProcessing;

use crate::maps::GrayscaleBuffer;

/// Apply post-processing to height map.
pub(crate) fn apply_processing(height_map: &mut GrayscaleBuffer, processing: &NormalMapProcessing) {
    // Apply blur if specified
    if let Some(sigma) = processing.blur {
        if sigma > 0.0 {
            apply_gaussian_blur(height_map, sigma);
        }
    }

    // Apply invert if specified
    if processing.invert {
        for value in &mut height_map.data {
            *value = 1.0 - *value;
        }
    }
}

/// Apply Gaussian blur to a height map.
fn apply_gaussian_blur(height_map: &mut GrayscaleBuffer, sigma: f64) {
    let width = height_map.width;
    let height = height_map.height;

    // Calculate kernel size (3 sigma on each side)
    let kernel_size = ((sigma * 3.0).ceil() as usize * 2 + 1).max(3);
    let half_kernel = kernel_size / 2;

    // Generate Gaussian kernel
    let mut kernel = vec![0.0; kernel_size];
    let mut sum = 0.0;
    for (i, kernel_value) in kernel.iter_mut().enumerate() {
        let x = i as f64 - half_kernel as f64;
        let value = (-x * x / (2.0 * sigma * sigma)).exp();
        *kernel_value = value;
        sum += value;
    }
    // Normalize kernel
    for value in &mut kernel {
        *value /= sum;
    }

    // Horizontal pass
    let mut temp = vec![0.0; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            for (i, kernel_value) in kernel.iter().enumerate() {
                let offset = i as i32 - half_kernel as i32;
                let sample_x = (x as i32 + offset).rem_euclid(width as i32) as u32;
                sum += height_map.get(sample_x, y) * kernel_value;
            }
            temp[(y * width + x) as usize] = sum;
        }
    }

    // Vertical pass
    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            for (i, kernel_value) in kernel.iter().enumerate() {
                let offset = i as i32 - half_kernel as i32;
                let sample_y = (y as i32 + offset).rem_euclid(height as i32) as u32;
                sum += temp[(sample_y * width + x) as usize] * kernel_value;
            }
            height_map.set(x, y, sum);
        }
    }
}
