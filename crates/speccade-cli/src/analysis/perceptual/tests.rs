//! Unit tests for perceptual comparison metrics.

use super::audio::calculate_spectral_correlation;
use super::color::{calculate_delta_e, rgb_to_lab};
use super::ssim::calculate_ssim;

#[test]
fn test_ssim_identical_images() {
    let pixels: Vec<u8> = (0..64 * 64 * 4).map(|i| (i % 256) as u8).collect();
    let ssim = calculate_ssim(&pixels, &pixels, 64, 64, 4);
    assert!(
        (ssim - 1.0).abs() < 0.001,
        "SSIM for identical images should be 1.0, got {}",
        ssim
    );
}

#[test]
fn test_ssim_different_images() {
    let pixels_a: Vec<u8> = vec![0u8; 64 * 64 * 4];
    let pixels_b: Vec<u8> = vec![255u8; 64 * 64 * 4];
    let ssim = calculate_ssim(&pixels_a, &pixels_b, 64, 64, 4);
    assert!(
        ssim < 0.1,
        "SSIM for very different images should be low, got {}",
        ssim
    );
}

#[test]
fn test_ssim_grayscale() {
    let pixels: Vec<u8> = (0..32 * 32).map(|i| (i % 256) as u8).collect();
    let ssim = calculate_ssim(&pixels, &pixels, 32, 32, 1);
    assert!(
        (ssim - 1.0).abs() < 0.001,
        "SSIM for identical grayscale should be 1.0"
    );
}

#[test]
fn test_ssim_small_image() {
    // Image smaller than 8x8 window
    let pixels: Vec<u8> = vec![128u8; 4 * 4 * 4];
    let ssim = calculate_ssim(&pixels, &pixels, 4, 4, 4);
    assert!(
        (ssim - 1.0).abs() < 0.001,
        "SSIM for small identical images should be 1.0"
    );
}

#[test]
fn test_delta_e_identical() {
    let pixels: Vec<u8> = vec![128, 64, 32, 255, 100, 150, 200, 255];
    let (mean, max) = calculate_delta_e(&pixels, &pixels, 2, 1, 4);
    assert_eq!(mean, 0.0, "DeltaE for identical pixels should be 0");
    assert_eq!(max, 0.0, "Max DeltaE for identical pixels should be 0");
}

#[test]
fn test_delta_e_different() {
    let pixels_a: Vec<u8> = vec![255, 0, 0, 255]; // Red
    let pixels_b: Vec<u8> = vec![0, 255, 0, 255]; // Green
    let (mean, max) = calculate_delta_e(&pixels_a, &pixels_b, 1, 1, 4);
    assert!(
        mean > 50.0,
        "DeltaE between red and green should be significant"
    );
    assert!(
        max > 50.0,
        "Max DeltaE between red and green should be significant"
    );
}

#[test]
fn test_delta_e_black_white() {
    let pixels_a: Vec<u8> = vec![0, 0, 0, 255]; // Black
    let pixels_b: Vec<u8> = vec![255, 255, 255, 255]; // White
    let (mean, _max) = calculate_delta_e(&pixels_a, &pixels_b, 1, 1, 4);
    assert!(mean > 90.0, "DeltaE between black and white should be ~100");
}

#[test]
fn test_delta_e_grayscale() {
    let pixels_a: Vec<u8> = vec![0];
    let pixels_b: Vec<u8> = vec![255];
    let (mean, _max) = calculate_delta_e(&pixels_a, &pixels_b, 1, 1, 1);
    assert!(
        mean > 90.0,
        "DeltaE between black and white grayscale should be ~100"
    );
}

#[test]
fn test_rgb_to_lab_black() {
    let (l, _a, _b) = rgb_to_lab(0, 0, 0);
    assert!(l.abs() < 0.01, "Black should have L close to 0");
}

#[test]
fn test_rgb_to_lab_white() {
    let (l, _a, _b) = rgb_to_lab(255, 255, 255);
    assert!((l - 100.0).abs() < 1.0, "White should have L close to 100");
}

#[test]
fn test_spectral_correlation_identical() {
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin()).collect();
    let corr = calculate_spectral_correlation(&samples, &samples, 44100);
    assert!(
        (corr - 1.0).abs() < 0.001,
        "Correlation of identical signals should be 1.0"
    );
}

#[test]
fn test_spectral_correlation_different() {
    // Low frequency sine
    let samples_a: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.01).sin()).collect();
    // High frequency sine
    let samples_b: Vec<f32> = (0..4410).map(|i| (i as f32 * 1.0).sin()).collect();
    let corr = calculate_spectral_correlation(&samples_a, &samples_b, 44100);
    // Different frequencies should have lower correlation
    assert!(
        corr < 0.9,
        "Correlation of different frequency signals should be lower"
    );
}

#[test]
fn test_spectral_correlation_short_signal() {
    let samples: Vec<f32> = vec![0.0; 100]; // Too short for windowed analysis
    let corr = calculate_spectral_correlation(&samples, &samples, 44100);
    assert!(corr.is_finite(), "Should handle short signals gracefully");
}
