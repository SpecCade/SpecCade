//! Tests for layer delay functionality.

use super::*;

#[test]
fn test_layer_delay_samples() {
    let mut mixer = Mixer::new(100, 44100.0);
    let layer = Layer::centered(vec![1.0; 20], 1.0).with_delay(50);
    mixer.add_layer(layer);

    let output = mixer.mix_mono();

    // First 50 samples should be silent
    for (i, sample) in output.iter().take(50).enumerate() {
        assert!(
            sample.abs() < 0.001,
            "Sample {} should be silent but is {}",
            i,
            sample
        );
    }
    // Samples 50-69 should have signal
    for (i, sample) in output.iter().take(70).skip(50).enumerate() {
        let i = i + 50;
        assert!(
            (*sample - 1.0).abs() < 0.001,
            "Sample {} should be 1.0 but is {}",
            i,
            sample
        );
    }
    // Samples 70-99 should be silent
    for (i, sample) in output.iter().take(100).skip(70).enumerate() {
        let i = i + 70;
        assert!(
            sample.abs() < 0.001,
            "Sample {} should be silent but is {}",
            i,
            sample
        );
    }
}

#[test]
fn test_layer_delay_seconds() {
    let sample_rate = 44100.0;
    let mut mixer = Mixer::new(88200, sample_rate); // 2 seconds
    let layer = Layer::centered(vec![1.0; 4410], 1.0).with_delay_seconds(0.5, sample_rate); // 500ms delay
    mixer.add_layer(layer);

    let output = mixer.mix_mono();

    // First 22050 samples (0.5 seconds) should be silent
    assert!(output[22049].abs() < 0.001);
    // Sample 22050 onwards should have signal
    assert!((output[22050] - 1.0).abs() < 0.001);
}

#[test]
fn test_layer_delay_truncates_at_output_boundary() {
    let mut mixer = Mixer::new(100, 44100.0);
    let layer = Layer::centered(vec![1.0; 50], 1.0).with_delay(80);
    mixer.add_layer(layer);

    let output = mixer.mix_mono();

    // Only 20 samples should fit (80 + 20 = 100)
    assert!(output[79].abs() < 0.001);
    assert!((output[80] - 1.0).abs() < 0.001);
    assert!((output[99] - 1.0).abs() < 0.001);
}

#[test]
fn test_layer_delay_beyond_output() {
    let mut mixer = Mixer::new(100, 44100.0);
    let layer = Layer::centered(vec![1.0; 50], 1.0).with_delay(150); // Beyond output length
    mixer.add_layer(layer);

    let output = mixer.mix_mono();

    // All samples should be silent
    assert!(output.iter().all(|&s| s == 0.0));
}

#[test]
fn test_multiple_layers_with_different_delays() {
    let mut mixer = Mixer::new(100, 44100.0);
    // Layer 1: samples 0-29 (delay 0, length 30)
    mixer.add_layer(Layer::centered(vec![0.3; 30], 1.0).with_delay(0));
    // Layer 2: samples 20-49 (delay 20, length 30)
    mixer.add_layer(Layer::centered(vec![0.3; 30], 1.0).with_delay(20));
    // Layer 3: samples 40-69 (delay 40, length 30)
    mixer.add_layer(Layer::centered(vec![0.3; 30], 1.0).with_delay(40));

    let output = mixer.mix_mono();

    // 0-19: one layer (layer 1 only) -> 0.3
    assert!((output[10] - 0.3).abs() < 0.001);
    // 20-29: two layers (layer 1 + layer 2) -> 0.6
    assert!((output[25] - 0.6).abs() < 0.001);
    // 30-39: one layer (layer 2 only, layer 1 ended, layer 3 not started) -> 0.3
    assert!((output[35] - 0.3).abs() < 0.001);
    // 40-49: two layers (layer 2 + layer 3) -> 0.6
    assert!((output[45] - 0.6).abs() < 0.001);
    // 50-69: one layer (layer 3 only) -> 0.3
    assert!((output[55] - 0.3).abs() < 0.001);
    // 70-99: silent (all layers ended)
    assert!((output[80] - 0.0).abs() < 0.001);
}

#[test]
fn test_layer_delay() {
    let mut mixer = Mixer::new(100, 44100.0);
    let layer = Layer::centered(vec![1.0; 20], 1.0).with_delay(50);
    mixer.add_layer(layer);

    let output = mixer.mix_mono();

    // First 50 samples should be silent
    assert!(output[49].abs() < 0.001);
    // Sample 50 onwards should have signal
    assert!((output[50] - 1.0).abs() < 0.001);
}
