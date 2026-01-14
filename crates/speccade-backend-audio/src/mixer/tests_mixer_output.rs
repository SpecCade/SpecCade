//! Tests for MixerOutput conversion and handling.

use super::*;

#[test]
fn test_mixer_output_mono() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![0.5; 100], 1.0);

    let output = mixer.mix();
    assert!(!output.is_stereo());
    assert_eq!(output.len(), 100);
}

#[test]
fn test_mixer_output_stereo() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_panned(vec![0.5; 100], 1.0, 0.5);

    let output = mixer.mix();
    assert!(output.is_stereo());
    assert_eq!(output.len(), 100);
}

#[test]
fn test_mixer_output_to_mono_from_mono() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![0.5; 100], 1.0);

    let output = mixer.mix();
    let mono = output.to_mono();
    assert_eq!(mono.len(), 100);
    assert!(mono.iter().all(|&s| (s - 0.5).abs() < 0.001));
}

#[test]
fn test_mixer_output_to_mono_from_stereo() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_panned(vec![1.0; 100], 1.0, -1.0); // Hard left

    let output = mixer.mix();
    let mono = output.to_mono();

    // Hard left has L=1.0, R=0.0, so mono = (1.0 + 0.0) / 2 = 0.5
    assert!(mono.iter().all(|&s| (s - 0.5).abs() < 0.001));
}

#[test]
fn test_mixer_output_to_stereo_from_mono() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![0.5; 100], 1.0);

    let output = mixer.mix();
    let stereo = output.to_stereo();

    assert_eq!(stereo.left.len(), 100);
    assert_eq!(stereo.right.len(), 100);
    assert_eq!(stereo.left, stereo.right);
}

#[test]
fn test_mixer_output_to_stereo_from_stereo() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_panned(vec![1.0; 100], 1.0, 0.5);

    let output = mixer.mix();
    let stereo = output.to_stereo();

    assert_eq!(stereo.left.len(), 100);
    assert_eq!(stereo.right.len(), 100);
}

#[test]
fn test_mixer_output_is_empty() {
    let mixer = Mixer::new(0, 44100.0);
    let output = mixer.mix();
    assert!(output.is_empty());
}

#[test]
fn test_mixer_output() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![0.5; 100], 1.0);

    let output = mixer.mix();
    assert!(!output.is_stereo()); // Should be mono since no panning

    let mut mixer_stereo = Mixer::new(100, 44100.0);
    mixer_stereo.add_panned(vec![0.5; 100], 1.0, 0.5);

    let output_stereo = mixer_stereo.mix();
    assert!(output_stereo.is_stereo()); // Should be stereo with panning
}
