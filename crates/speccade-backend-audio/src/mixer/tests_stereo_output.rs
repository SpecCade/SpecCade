//! Tests for StereoOutput type.

use super::*;

#[test]
fn test_stereo_output_new() {
    let stereo = StereoOutput::new(100);
    assert_eq!(stereo.left.len(), 100);
    assert_eq!(stereo.right.len(), 100);
    assert!(stereo.left.iter().all(|&s| s == 0.0));
    assert!(stereo.right.iter().all(|&s| s == 0.0));
}

#[test]
fn test_stereo_from_mono() {
    let mono = vec![1.0, 0.5, 0.0, -0.5, -1.0];
    let stereo = StereoOutput::from_mono(mono.clone());
    assert_eq!(stereo.left, mono);
    assert_eq!(stereo.right, mono);
    assert_eq!(stereo.left, stereo.right);
}

#[test]
fn test_stereo_from_mono_empty() {
    let mono: Vec<f64> = vec![];
    let stereo = StereoOutput::from_mono(mono);
    assert!(stereo.left.is_empty());
    assert!(stereo.right.is_empty());
}

#[test]
fn test_stereo_interleave() {
    let stereo = StereoOutput {
        left: vec![1.0, 2.0, 3.0],
        right: vec![4.0, 5.0, 6.0],
    };

    let interleaved = stereo.interleave();
    assert_eq!(interleaved, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
}

#[test]
fn test_stereo_interleave_empty() {
    let stereo = StereoOutput::new(0);
    let interleaved = stereo.interleave();
    assert!(interleaved.is_empty());
}

#[test]
fn test_stereo_interleave_single_sample() {
    let stereo = StereoOutput {
        left: vec![0.5],
        right: vec![-0.5],
    };
    let interleaved = stereo.interleave();
    assert_eq!(interleaved, vec![0.5, -0.5]);
}

#[test]
fn test_stereo_is_mono_true() {
    let stereo = StereoOutput::from_mono(vec![1.0, 0.5, 0.0]);
    assert!(stereo.is_mono());
}

#[test]
fn test_stereo_is_mono_false() {
    let stereo = StereoOutput {
        left: vec![1.0, 0.5, 0.0],
        right: vec![0.5, 0.25, 0.0],
    };
    assert!(!stereo.is_mono());
}

#[test]
fn test_stereo_to_mono() {
    let stereo = StereoOutput {
        left: vec![1.0, 0.5, 0.0],
        right: vec![0.0, 0.5, 1.0],
    };
    let mono = stereo.to_mono();
    assert_eq!(mono, vec![0.5, 0.5, 0.5]); // Average of each pair
}

#[test]
fn test_stereo_to_mono_preserves_mono_content() {
    let original = vec![1.0, 0.5, -0.5];
    let stereo = StereoOutput::from_mono(original.clone());
    let mono = stereo.to_mono();
    assert_eq!(mono, original);
}

#[test]
fn test_stereo_len() {
    let stereo = StereoOutput::new(50);
    assert_eq!(stereo.len(), 50);
}

#[test]
fn test_stereo_is_empty() {
    let empty = StereoOutput::new(0);
    assert!(empty.is_empty());

    let non_empty = StereoOutput::new(1);
    assert!(!non_empty.is_empty());
}

#[test]
fn test_stereo_interleave_original() {
    let stereo = StereoOutput {
        left: vec![1.0, 2.0, 3.0],
        right: vec![4.0, 5.0, 6.0],
    };

    let interleaved = stereo.interleave();
    assert_eq!(interleaved, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
}
