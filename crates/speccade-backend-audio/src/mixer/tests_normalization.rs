//! Tests for audio normalization functions.

use super::*;

#[test]
fn test_normalize_basic() {
    let mut samples = vec![0.5, -0.3, 0.8, -0.2];
    normalize(&mut samples, -3.0);

    // Peak should be at -3dB
    let target = 10.0_f64.powf(-3.0 / 20.0);
    let peak = samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f64, |a, b| a.max(b));
    assert!((peak - target).abs() < 0.01);
}

#[test]
fn test_normalize_silent_audio() {
    let mut samples = vec![0.0, 0.0, 0.0, 0.0];
    normalize(&mut samples, -3.0);

    // Silent audio should remain silent (no division by zero)
    assert!(samples.iter().all(|&s| s == 0.0));
}

#[test]
fn test_normalize_loud_audio() {
    let mut samples = vec![2.0, -1.5, 3.0, -2.5];
    normalize(&mut samples, 0.0); // Normalize to 0dB (peak = 1.0)

    let peak = samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f64, |a, b| a.max(b));
    assert!((peak - 1.0).abs() < 0.001);
}

#[test]
fn test_normalize_quiet_audio() {
    let mut samples = vec![0.01, -0.005, 0.008, -0.003];
    normalize(&mut samples, 0.0); // Normalize to 0dB

    let peak = samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f64, |a, b| a.max(b));
    assert!((peak - 1.0).abs() < 0.001);
}

#[test]
fn test_normalize_peak_db() {
    // Test various headroom values
    for headroom_db in [-6.0, -3.0, 0.0, -12.0] {
        let mut samples = vec![1.0, -0.5, 0.75, -0.25];
        normalize(&mut samples, headroom_db);

        let target = 10.0_f64.powf(headroom_db / 20.0);
        let peak = samples
            .iter()
            .map(|s| s.abs())
            .fold(0.0_f64, |a, b| a.max(b));
        assert!(
            (peak - target).abs() < 0.001,
            "For {}dB, expected peak {}, got {}",
            headroom_db,
            target,
            peak
        );
    }
}

#[test]
fn test_normalize_preserves_relative_amplitudes() {
    let mut samples = vec![1.0, 0.5, 0.25];
    normalize(&mut samples, 0.0);

    // Ratios should be preserved
    assert!((samples[1] / samples[0] - 0.5).abs() < 0.001);
    assert!((samples[2] / samples[0] - 0.25).abs() < 0.001);
}

#[test]
fn test_normalize_stereo_basic() {
    let mut stereo = StereoOutput {
        left: vec![0.5, -0.3],
        right: vec![0.2, -0.1],
    };
    normalize_stereo(&mut stereo, 0.0);

    let peak = stereo
        .left
        .iter()
        .chain(stereo.right.iter())
        .map(|s| s.abs())
        .fold(0.0_f64, |a, b| a.max(b));
    assert!((peak - 1.0).abs() < 0.001);
}

#[test]
fn test_normalize_stereo_silent() {
    let mut stereo = StereoOutput {
        left: vec![0.0, 0.0],
        right: vec![0.0, 0.0],
    };
    normalize_stereo(&mut stereo, 0.0);

    // Silent audio should remain silent
    assert!(stereo.left.iter().all(|&s| s == 0.0));
    assert!(stereo.right.iter().all(|&s| s == 0.0));
}

#[test]
fn test_normalize_stereo_uses_global_peak() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![1.0], // Right has higher peak
    };
    normalize_stereo(&mut stereo, 0.0);

    // Right should be at 1.0, left should be at 0.5
    assert!((stereo.right[0] - 1.0).abs() < 0.001);
    assert!((stereo.left[0] - 0.5).abs() < 0.001);
}

#[test]
fn test_normalization() {
    let mut samples = vec![0.5, -0.3, 0.8, -0.2];
    normalize(&mut samples, -3.0);

    // Peak should be at -3dB
    let target = 10.0_f64.powf(-3.0 / 20.0);
    let peak = samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f64, |a, b| a.max(b));
    assert!((peak - target).abs() < 0.01);
}
