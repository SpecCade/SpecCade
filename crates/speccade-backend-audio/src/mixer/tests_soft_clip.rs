//! Tests for soft clipping functions.

use super::*;

#[test]
fn test_soft_clip_below_threshold() {
    assert!((soft_clip(0.5, 0.8) - 0.5).abs() < 0.001);
    assert!((soft_clip(-0.5, 0.8) - (-0.5)).abs() < 0.001);
    assert!((soft_clip(0.0, 0.8) - 0.0).abs() < 0.001);
    assert!((soft_clip(0.79, 0.8) - 0.79).abs() < 0.001);
}

#[test]
fn test_soft_clip_at_threshold() {
    let result = soft_clip(0.8, 0.8);
    assert!((result - 0.8).abs() < 0.001);
}

#[test]
fn test_soft_clip_above_threshold() {
    let clipped = soft_clip(2.0, 0.8);
    assert!(clipped < 2.0, "Should be compressed");
    assert!(clipped > 0.8, "Should be above threshold");
    assert!(clipped < 1.0, "Should approach but not exceed 1.0");
}

#[test]
fn test_soft_clip_very_high_values() {
    let hard = soft_clip(10.0, 0.8);
    assert!(hard < 1.0);
    assert!(hard > 0.95); // Should be very close to 1.0
}

#[test]
fn test_soft_clip_preserves_sign() {
    let positive = soft_clip(2.0, 0.8);
    let negative = soft_clip(-2.0, 0.8);
    assert!(positive > 0.0);
    assert!(negative < 0.0);
    assert!((positive + negative).abs() < 0.001); // Symmetric
}

#[test]
fn test_soft_clip_buffer() {
    let mut samples = vec![0.5, 1.5, -0.3, 2.0, -1.0];
    soft_clip_buffer(&mut samples, 0.8);

    assert!((samples[0] - 0.5).abs() < 0.001); // Below threshold
    assert!(samples[1] > 0.8 && samples[1] < 1.0); // Compressed
    assert!((samples[2] - (-0.3)).abs() < 0.001); // Below threshold
    assert!(samples[3] > 0.8 && samples[3] < 1.0); // Compressed
    assert!(samples[4] > -1.0 && samples[4] < -0.8); // Compressed (negative)
}

#[test]
fn test_soft_clip() {
    // Below threshold: unchanged
    assert!((soft_clip(0.5, 0.8) - 0.5).abs() < 0.001);

    // Above threshold: compressed
    let clipped = soft_clip(2.0, 0.8);
    assert!(clipped < 2.0);
    assert!(clipped > 0.8);

    // Very high values should approach but not exceed 1.0
    let hard = soft_clip(10.0, 0.8);
    assert!(hard < 1.0);
}
