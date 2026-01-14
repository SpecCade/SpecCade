//! Basic tests for Mixer construction and layer management.

use super::*;

#[test]
fn test_mixer_new() {
    let mixer = Mixer::new(1000, 48000.0);
    assert_eq!(mixer.num_samples(), 1000);
    assert_eq!(mixer.sample_rate(), 48000.0);
    assert!(!mixer.is_stereo());
}

#[test]
fn test_mixer_add_layer_mono() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_layer(Layer::centered(vec![0.5; 50], 1.0));
    assert!(!mixer.is_stereo()); // Centered layer doesn't create stereo content
}

#[test]
fn test_mixer_add_layer_stereo() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_layer(Layer::new(vec![0.5; 50], 1.0, 0.5)); // Non-zero pan
    assert!(mixer.is_stereo());
}

#[test]
fn test_mixer_add_mono() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![0.5; 50], 0.8);
    assert!(!mixer.is_stereo());
}

#[test]
fn test_mixer_add_panned() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_panned(vec![0.5; 50], 0.8, -0.5);
    assert!(mixer.is_stereo());
}

#[test]
fn test_empty_mixer() {
    let mixer = Mixer::new(0, 44100.0);
    let output = mixer.mix_mono();
    assert!(output.is_empty());
}

#[test]
fn test_zero_volume_layer() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![1.0; 100], 0.0);

    let output = mixer.mix_mono();
    assert!(output.iter().all(|&s| s == 0.0));
}

#[test]
fn test_very_small_pan() {
    let mut mixer = Mixer::new(100, 44100.0);
    // Pan value just above the stereo detection threshold
    mixer.add_panned(vec![1.0; 100], 1.0, 0.0000001);

    // Should not be detected as stereo (threshold is 1e-6)
    assert!(!mixer.is_stereo());
}

#[test]
fn test_pan_at_stereo_threshold() {
    let mut mixer = Mixer::new(100, 44100.0);
    // Pan value just above the stereo detection threshold
    mixer.add_panned(vec![1.0; 100], 1.0, 0.00001);

    assert!(mixer.is_stereo());
}
