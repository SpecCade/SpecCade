//! Texture embedding computation.
//!
//! Provides deterministic feature vector computation for texture assets.

/// Precision for floating point values in output (6 decimal places).
const FLOAT_PRECISION: i32 = 6;

/// Number of histogram bins for texture embedding.
const HISTOGRAM_BINS: usize = 16;

/// Number of spatial features for texture embedding.
const SPATIAL_FEATURES: usize = 16;

/// Number of channel features for texture embedding.
const CHANNEL_FEATURES: usize = 16;

/// Total texture embedding dimension.
pub const EMBEDDING_DIM: usize = HISTOGRAM_BINS + SPATIAL_FEATURES + CHANNEL_FEATURES;

/// Round a float to the specified number of decimal places.
fn round_f64(value: f64, decimals: i32) -> f64 {
    let multiplier = 10_f64.powi(decimals);
    (value * multiplier).round() / multiplier
}

/// Compute texture embedding from pixels.
///
/// Returns a 48-dimension feature vector capturing:
/// - Luminance histogram (16 bins)
/// - Spatial features (16 values): edge density, contrast, texture measures
/// - Channel features (16 values): per-channel stats and correlations
pub fn compute(pixels: &[u8], width: u32, height: u32, channels: u8) -> Vec<f64> {
    let mut embedding = Vec::with_capacity(EMBEDDING_DIM);

    let histogram = compute_luminance_histogram(pixels, channels);
    embedding.extend(histogram);

    let spatial = compute_spatial_features(pixels, width, height, channels);
    embedding.extend(spatial);

    let channel_features = compute_channel_features(pixels, channels);
    embedding.extend(channel_features);

    embedding
        .iter()
        .map(|&v| round_f64(v, FLOAT_PRECISION))
        .collect()
}

/// Compute luminance histogram with 16 bins.
fn compute_luminance_histogram(pixels: &[u8], channels: u8) -> Vec<f64> {
    let step = channels as usize;
    let pixel_count = pixels.len() / step;

    if pixel_count == 0 {
        return vec![0.0; HISTOGRAM_BINS];
    }

    let mut histogram = [0u64; HISTOGRAM_BINS];

    for i in 0..pixel_count {
        let offset = i * step;
        let luminance = match channels {
            1 | 2 => pixels[offset] as f64,
            3 | 4 => rgb_to_luminance(pixels[offset], pixels[offset + 1], pixels[offset + 2]),
            _ => 0.0,
        };
        let bin = ((luminance / 256.0) * HISTOGRAM_BINS as f64) as usize;
        let bin = bin.min(HISTOGRAM_BINS - 1);
        histogram[bin] += 1;
    }

    let max_count = *histogram.iter().max().unwrap_or(&1) as f64;
    histogram
        .iter()
        .map(|&c| c as f64 / max_count.max(1.0))
        .collect()
}

/// Calculate luminance from RGB values (ITU-R BT.601).
fn rgb_to_luminance(r: u8, g: u8, b: u8) -> f64 {
    0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64
}

/// Compute spatial features (edge density, contrast, texture).
fn compute_spatial_features(pixels: &[u8], width: u32, height: u32, channels: u8) -> Vec<f64> {
    let step = channels as usize;
    let pixel_count = pixels.len() / step;

    if pixel_count == 0 || width < 2 || height < 2 {
        return vec![0.0; SPATIAL_FEATURES];
    }

    let mut features = vec![0.0; SPATIAL_FEATURES];

    let luminance: Vec<f64> = (0..pixel_count)
        .map(|i| {
            let offset = i * step;
            match channels {
                1 | 2 => pixels[offset] as f64,
                3 | 4 => rgb_to_luminance(pixels[offset], pixels[offset + 1], pixels[offset + 2]),
                _ => 0.0,
            }
        })
        .collect();

    // 0: Global luminance mean (normalized)
    let lum_mean: f64 = luminance.iter().sum::<f64>() / luminance.len() as f64;
    features[0] = lum_mean / 255.0;

    // 1: Global luminance std (normalized)
    let lum_variance: f64 = luminance
        .iter()
        .map(|l| (l - lum_mean).powi(2))
        .sum::<f64>()
        / luminance.len() as f64;
    features[1] = lum_variance.sqrt() / 127.5;

    // 2: Luminance range (max - min, normalized)
    let lum_min = luminance.iter().cloned().fold(f64::INFINITY, f64::min);
    let lum_max = luminance.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    features[2] = (lum_max - lum_min) / 255.0;

    // 3-4: Horizontal and vertical edge density
    let (h_edges, v_edges) = compute_edge_density(&luminance, width, height);
    features[3] = h_edges.min(1.0);
    features[4] = v_edges.min(1.0);

    // 5: Total edge density
    features[5] = ((h_edges + v_edges) / 2.0).min(1.0);

    // 6: Contrast ratio
    features[6] = if lum_min > 0.0 {
        ((lum_max / lum_min).min(255.0)) / 255.0
    } else {
        (lum_max - lum_min) / 255.0
    };

    // 7: Weber contrast
    features[7] = if lum_mean > 0.0 {
        ((lum_max - lum_min) / lum_mean).min(2.0) / 2.0
    } else {
        0.0
    };

    // 8-11: Quadrant mean luminance (2x2 grid)
    let half_w = (width / 2) as usize;
    let half_h = (height / 2) as usize;
    let w = width as usize;
    let h = height as usize;

    for (qi, (qy, qx)) in [(0, 0), (0, 1), (1, 0), (1, 1)].iter().enumerate() {
        let start_y = qy * half_h;
        let end_y = if *qy == 1 { h } else { half_h };
        let start_x = qx * half_w;
        let end_x = if *qx == 1 { w } else { half_w };

        let mut sum = 0.0;
        let mut count = 0;
        for y in start_y..end_y {
            for x in start_x..end_x {
                sum += luminance[y * w + x];
                count += 1;
            }
        }
        features[8 + qi] = if count > 0 {
            (sum / count as f64) / 255.0
        } else {
            0.0
        };
    }

    // 12: Luminance entropy
    let mut hist = vec![0u64; 256];
    for &l in &luminance {
        let bin = (l as usize).min(255);
        hist[bin] += 1;
    }
    let n = luminance.len() as f64;
    let entropy: f64 = hist
        .iter()
        .map(|&c| {
            if c > 0 {
                let p = c as f64 / n;
                -p * p.ln()
            } else {
                0.0
            }
        })
        .sum();
    features[12] = (entropy / 8.0f64.ln()).min(1.0);

    // 13: Local variance (texture measure)
    features[13] = compute_local_variance(&luminance, width, height);

    // 14: Smoothness measure
    features[14] = 1.0 - features[1].min(1.0);

    // 15: Uniformity
    let uniformity: f64 = hist.iter().map(|&c| (c as f64 / n).powi(2)).sum();
    features[15] = uniformity.sqrt();

    features
}

/// Compute horizontal and vertical edge density using Sobel-like operator.
fn compute_edge_density(luminance: &[f64], width: u32, height: u32) -> (f64, f64) {
    let w = width as usize;
    let h = height as usize;

    if w < 3 || h < 3 {
        return (0.0, 0.0);
    }

    let mut h_edges = 0.0;
    let mut v_edges = 0.0;
    let mut count = 0;

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let idx = y * w + x;
            let gx = luminance[idx + 1] - luminance[idx - 1];
            h_edges += gx.abs();
            let gy = luminance[idx + w] - luminance[idx - w];
            v_edges += gy.abs();
            count += 1;
        }
    }

    if count > 0 {
        let max_gradient = 255.0 * 2.0;
        (
            h_edges / (count as f64 * max_gradient),
            v_edges / (count as f64 * max_gradient),
        )
    } else {
        (0.0, 0.0)
    }
}

/// Compute local variance as a texture measure.
fn compute_local_variance(luminance: &[f64], width: u32, height: u32) -> f64 {
    let w = width as usize;
    let h = height as usize;

    if w < 3 || h < 3 {
        return 0.0;
    }

    let mut total_variance = 0.0;
    let mut count = 0;

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let mut window_sum = 0.0;
            let mut window_sq_sum = 0.0;

            for dy in 0..3 {
                for dx in 0..3 {
                    let idx = (y - 1 + dy) * w + (x - 1 + dx);
                    let val = luminance[idx];
                    window_sum += val;
                    window_sq_sum += val * val;
                }
            }

            let mean = window_sum / 9.0;
            let variance = (window_sq_sum / 9.0) - (mean * mean);
            total_variance += variance.max(0.0);
            count += 1;
        }
    }

    if count > 0 {
        let avg_var = total_variance / count as f64;
        (avg_var / 16256.25).min(1.0)
    } else {
        0.0
    }
}

/// Compute per-channel features.
fn compute_channel_features(pixels: &[u8], channels: u8) -> Vec<f64> {
    let step = channels as usize;
    let pixel_count = pixels.len() / step;

    if pixel_count == 0 {
        return vec![0.0; CHANNEL_FEATURES];
    }

    let mut features = vec![0.0; CHANNEL_FEATURES];

    let mut red_vals = Vec::with_capacity(pixel_count);
    let mut green_vals = Vec::with_capacity(pixel_count);
    let mut blue_vals = Vec::with_capacity(pixel_count);
    let mut alpha_vals = Vec::with_capacity(pixel_count);

    for i in 0..pixel_count {
        let offset = i * step;
        red_vals.push(pixels[offset] as f64);
        if channels >= 3 {
            green_vals.push(pixels[offset + 1] as f64);
            blue_vals.push(pixels[offset + 2] as f64);
        }
        if channels == 2 {
            alpha_vals.push(pixels[offset + 1] as f64);
        } else if channels == 4 {
            alpha_vals.push(pixels[offset + 3] as f64);
        }
    }

    let (r_mean, r_std) = compute_mean_std(&red_vals);
    features[0] = r_mean / 255.0;
    features[1] = r_std / 127.5;

    if !green_vals.is_empty() {
        let (g_mean, g_std) = compute_mean_std(&green_vals);
        features[2] = g_mean / 255.0;
        features[3] = g_std / 127.5;
    } else {
        features[2] = features[0];
        features[3] = features[1];
    }

    if !blue_vals.is_empty() {
        let (b_mean, b_std) = compute_mean_std(&blue_vals);
        features[4] = b_mean / 255.0;
        features[5] = b_std / 127.5;
    } else {
        features[4] = features[0];
        features[5] = features[1];
    }

    if !alpha_vals.is_empty() {
        let (a_mean, a_std) = compute_mean_std(&alpha_vals);
        features[6] = a_mean / 255.0;
        features[7] = a_std / 127.5;
    } else {
        features[6] = 1.0;
        features[7] = 0.0;
    }

    if !green_vals.is_empty() && !blue_vals.is_empty() {
        features[8] = compute_correlation(&red_vals, &green_vals).abs();
        features[9] = compute_correlation(&red_vals, &blue_vals).abs();
        features[10] = compute_correlation(&green_vals, &blue_vals).abs();
    } else {
        features[8] = 1.0;
        features[9] = 1.0;
        features[10] = 1.0;
    }

    // 11: Color saturation
    if !green_vals.is_empty() && !blue_vals.is_empty() {
        let mut sat_sum = 0.0;
        for i in 0..pixel_count {
            let r = red_vals[i];
            let g = green_vals[i];
            let b = blue_vals[i];
            let max_c = r.max(g).max(b);
            let min_c = r.min(g).min(b);
            let sat = if max_c > 0.0 {
                (max_c - min_c) / max_c
            } else {
                0.0
            };
            sat_sum += sat;
        }
        features[11] = sat_sum / pixel_count as f64;
    }

    // 12-13: Color range
    let r_range = red_vals.iter().fold(0.0f64, |m, &v| m.max(v))
        - red_vals.iter().fold(255.0f64, |m, &v| m.min(v));
    features[12] = r_range / 255.0;

    if !green_vals.is_empty() && !blue_vals.is_empty() {
        let g_range = green_vals.iter().fold(0.0f64, |m, &v| m.max(v))
            - green_vals.iter().fold(255.0f64, |m, &v| m.min(v));
        let b_range = blue_vals.iter().fold(0.0f64, |m, &v| m.max(v))
            - blue_vals.iter().fold(255.0f64, |m, &v| m.min(v));
        features[13] = ((r_range + g_range + b_range) / 3.0) / 255.0;
    } else {
        features[13] = features[12];
    }

    // 14: Grayscale similarity
    if !green_vals.is_empty() && !blue_vals.is_empty() {
        let mut diff_sum = 0.0;
        for i in 0..pixel_count {
            let r = red_vals[i];
            let g = green_vals[i];
            let b = blue_vals[i];
            diff_sum += (r - g).abs() + (r - b).abs() + (g - b).abs();
        }
        features[14] = 1.0 - (diff_sum / (pixel_count as f64 * 765.0));
    } else {
        features[14] = 1.0;
    }

    // 15: Alpha coverage
    if !alpha_vals.is_empty() {
        let opaque_count = alpha_vals.iter().filter(|&&a| a > 0.0).count();
        features[15] = opaque_count as f64 / pixel_count as f64;
    } else {
        features[15] = 1.0;
    }

    features
}

/// Compute mean and standard deviation.
fn compute_mean_std(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
    let variance: f64 =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    (mean, variance.sqrt())
}

/// Compute Pearson correlation coefficient.
fn compute_correlation(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let n = a.len() as f64;
    let mean_a: f64 = a.iter().sum::<f64>() / n;
    let mean_b: f64 = b.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;

    for i in 0..a.len() {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }

    let denom = (var_a * var_b).sqrt();
    if denom > 0.0 {
        (cov / denom).clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_dimension() {
        let pixels: Vec<u8> = vec![128; 64 * 64 * 4];
        let embedding = compute(&pixels, 64, 64, 4);
        assert_eq!(embedding.len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_embedding_range() {
        let pixels: Vec<u8> = (0..64 * 64 * 4).map(|i| (i % 256) as u8).collect();
        let embedding = compute(&pixels, 64, 64, 4);
        for (i, &v) in embedding.iter().enumerate() {
            assert!(
                (0.0..=1.0).contains(&v),
                "Embedding[{}] = {} out of range",
                i,
                v
            );
        }
    }

    #[test]
    fn test_embedding_deterministic() {
        let pixels: Vec<u8> = (0..64 * 64 * 4).map(|i| (i % 256) as u8).collect();
        let e1 = compute(&pixels, 64, 64, 4);
        let e2 = compute(&pixels, 64, 64, 4);
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_embedding_empty() {
        let embedding = compute(&[], 0, 0, 4);
        assert_eq!(embedding.len(), EMBEDDING_DIM);
        assert!(embedding.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_embedding_grayscale() {
        let pixels: Vec<u8> = vec![128; 64 * 64];
        let embedding = compute(&pixels, 64, 64, 1);
        assert_eq!(embedding.len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_luminance_histogram_normalization() {
        let pixels: Vec<u8> = (0u16..256).cycle().take(256 * 4).map(|v| v as u8).collect();
        let histogram = compute_luminance_histogram(&pixels, 4);
        assert_eq!(histogram.len(), HISTOGRAM_BINS);
        let max_bin = histogram.iter().cloned().fold(0.0f64, f64::max);
        assert!(max_bin <= 1.0);
    }
}
