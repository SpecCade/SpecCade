//! Tests for mono and stereo mixing operations.

use super::*;

// ============================================================================
// Mono Mixing Tests
// ============================================================================

#[test]
fn test_mix_mono_single_layer() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![0.5; 100], 1.0);

    let output = mixer.mix_mono();
    assert_eq!(output.len(), 100);
    assert!(output.iter().all(|&s| (s - 0.5).abs() < 0.001));
}

#[test]
fn test_mix_mono_single_layer_with_volume() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![1.0; 100], 0.5);

    let output = mixer.mix_mono();
    assert!(output.iter().all(|&s| (s - 0.5).abs() < 0.001));
}

#[test]
fn test_mix_mono_multiple_layers() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![0.3; 100], 1.0);
    mixer.add_mono(vec![0.2; 100], 1.0);
    mixer.add_mono(vec![0.1; 100], 1.0);

    let output = mixer.mix_mono();
    // Sum should be 0.3 + 0.2 + 0.1 = 0.6
    assert!(output.iter().all(|&s| (s - 0.6).abs() < 0.001));
}

#[test]
fn test_mix_mono_multiple_layers_with_volumes() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![1.0; 100], 0.5); // 1.0 * 0.5 = 0.5
    mixer.add_mono(vec![1.0; 100], 0.25); // 1.0 * 0.25 = 0.25

    let output = mixer.mix_mono();
    assert!(output.iter().all(|&s| (s - 0.75).abs() < 0.001));
}

#[test]
fn test_mix_mono_partial_overlap() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![0.5; 50], 1.0); // Samples 0-49
    mixer.add_layer(Layer::centered(vec![0.3; 50], 1.0).with_delay(25)); // Samples 25-74

    let output = mixer.mix_mono();

    // Samples 0-24: only first layer (0.5)
    assert!((output[10] - 0.5).abs() < 0.001);
    // Samples 25-49: both layers overlap (0.5 + 0.3 = 0.8)
    assert!((output[30] - 0.8).abs() < 0.001);
    // Samples 50-74: only second layer (0.3)
    assert!((output[60] - 0.3).abs() < 0.001);
    // Samples 75-99: silent
    assert!((output[80] - 0.0).abs() < 0.001);
}

#[test]
fn test_mix_mono_empty_layers() {
    let mixer = Mixer::new(100, 44100.0);
    let output = mixer.mix_mono();
    assert!(output.iter().all(|&s| s == 0.0));
}

#[test]
fn test_mix_mono_layer_exceeds_output_length() {
    let mut mixer = Mixer::new(50, 44100.0);
    mixer.add_mono(vec![1.0; 100], 1.0); // Layer is longer than output

    let output = mixer.mix_mono();
    assert_eq!(output.len(), 50);
    assert!(output.iter().all(|&s| (s - 1.0).abs() < 0.001));
}

#[test]
fn test_mono_mixing() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_mono(vec![0.5; 50], 1.0);
    mixer.add_mono(vec![0.3; 50], 1.0);

    let output = mixer.mix_mono();
    assert_eq!(output.len(), 100);
    // First 50 samples should have both layers
    assert!((output[25] - 0.8).abs() < 0.01);
}

// ============================================================================
// Stereo Panning Tests
// ============================================================================

#[test]
fn test_mix_stereo_panning_left() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_panned(vec![1.0; 100], 1.0, -1.0); // Hard left (pan = -1.0)

    let output = mixer.mix_stereo();

    // At hard left, pan_angle = 0, so cos(0) = 1.0 and sin(0) = 0.0
    assert!((output.left[50] - 1.0).abs() < 0.001);
    assert!(output.right[50].abs() < 0.001);
}

#[test]
fn test_mix_stereo_panning_right() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_panned(vec![1.0; 100], 1.0, 1.0); // Hard right (pan = 1.0)

    let output = mixer.mix_stereo();

    // At hard right, pan_angle = PI/2, so cos(PI/2) = 0.0 and sin(PI/2) = 1.0
    assert!(output.left[50].abs() < 0.001);
    assert!((output.right[50] - 1.0).abs() < 0.001);
}

#[test]
fn test_mix_stereo_panning_center() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_panned(vec![1.0; 100], 1.0, 0.0); // Center (pan = 0.0)

    let output = mixer.mix_stereo();

    // At center, pan_angle = PI/4, so cos(PI/4) = sin(PI/4) = ~0.707
    let expected = std::f64::consts::FRAC_PI_4.cos();
    assert!((output.left[50] - expected).abs() < 0.001);
    assert!((output.right[50] - expected).abs() < 0.001);
    assert!((output.left[50] - output.right[50]).abs() < 0.001);
}

#[test]
fn test_mix_stereo_equal_power_preservation() {
    // Equal power panning should preserve total power across all pan positions
    let sample_value = 1.0;
    let num_samples = 100;

    // Test multiple pan positions
    for pan in [-1.0, -0.5, 0.0, 0.5, 1.0] {
        let mut mixer = Mixer::new(num_samples, 44100.0);
        mixer.add_panned(vec![sample_value; num_samples], 1.0, pan);

        let output = mixer.mix_stereo();

        // Calculate power (L^2 + R^2)
        let power = output.left[50].powi(2) + output.right[50].powi(2);

        // Power should be constant (approximately 1.0 for unit input)
        assert!(
            (power - 1.0).abs() < 0.01,
            "Power at pan={} is {}, expected ~1.0",
            pan,
            power
        );
    }
}

#[test]
fn test_mix_stereo_multiple_layers() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_panned(vec![0.5; 100], 1.0, -1.0); // Hard left
    mixer.add_panned(vec![0.5; 100], 1.0, 1.0); // Hard right

    let output = mixer.mix_stereo();

    // Left channel gets the left-panned signal
    assert!((output.left[50] - 0.5).abs() < 0.001);
    // Right channel gets the right-panned signal
    assert!((output.right[50] - 0.5).abs() < 0.001);
}

#[test]
fn test_mix_stereo_with_volume() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_panned(vec![1.0; 100], 0.5, -1.0); // Hard left with 50% volume

    let output = mixer.mix_stereo();

    assert!((output.left[50] - 0.5).abs() < 0.001);
    assert!(output.right[50].abs() < 0.001);
}

#[test]
fn test_mix_stereo_with_delay() {
    let mut mixer = Mixer::new(100, 44100.0);
    let layer = Layer::new(vec![1.0; 20], 1.0, 0.5).with_delay(30);
    mixer.add_layer(layer);

    let output = mixer.mix_stereo();

    // Before delay: silent
    assert!(output.left[20].abs() < 0.001);
    assert!(output.right[20].abs() < 0.001);

    // After delay: has signal
    assert!(output.left[35].abs() > 0.1);
    assert!(output.right[35].abs() > 0.1);
}

#[test]
fn test_stereo_panning() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_panned(vec![1.0; 100], 1.0, -1.0); // Hard left
    mixer.add_panned(vec![1.0; 100], 1.0, 1.0); // Hard right

    let output = mixer.mix_stereo();

    // Left channel should be louder from left-panned signal
    // Right channel should be louder from right-panned signal
    assert!(output.left[50] > 0.5);
    assert!(output.right[50] > 0.5);
}

#[test]
fn test_center_pan_equal_power() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_panned(vec![1.0; 100], 1.0, 0.0); // Center

    let output = mixer.mix_stereo();

    // Center pan should have equal power in both channels
    // At center, each channel gets cos(45deg) = ~0.707 of the signal
    let expected = std::f64::consts::FRAC_PI_4.cos();
    assert!((output.left[50] - expected).abs() < 0.01);
    assert!((output.right[50] - expected).abs() < 0.01);
}

// ============================================================================
// Stereo Layer Tests (LayerSamples::Stereo)
// ============================================================================

#[test]
fn test_stereo_layer_basic() {
    let mut mixer = Mixer::new(100, 44100.0);
    // Stereo layer: left channel has 0.8, right channel has 0.3
    mixer.add_layer(Layer::new_stereo(vec![0.8; 100], vec![0.3; 100], 1.0, 0.0));

    let output = mixer.mix_stereo();

    // At pan=0, stereo image is preserved
    // pan_normalized = 0.5, so:
    // left_to_left = 0.5, left_to_right = 0.5
    // right_to_left = 0.5, right_to_right = 0.5
    // output.left = 0.8 * 0.5 + 0.3 * 0.5 = 0.55
    // output.right = 0.8 * 0.5 + 0.3 * 0.5 = 0.55
    assert!((output.left[50] - 0.55).abs() < 0.01);
    assert!((output.right[50] - 0.55).abs() < 0.01);
}

#[test]
fn test_stereo_layer_pan_left() {
    let mut mixer = Mixer::new(100, 44100.0);
    // Stereo layer with distinct L/R content, panned hard left
    mixer.add_layer(Layer::new_stereo(vec![1.0; 100], vec![0.5; 100], 1.0, -1.0));

    let output = mixer.mix_stereo();

    // At pan=-1 (hard left), both channels go to left output
    // pan_normalized = 0, so:
    // left_to_left = 1.0, left_to_right = 0.0
    // right_to_left = 1.0, right_to_right = 0.0
    // output.left = 1.0 * 1.0 + 0.5 * 1.0 = 1.5
    // output.right = 1.0 * 0.0 + 0.5 * 0.0 = 0.0
    assert!((output.left[50] - 1.5).abs() < 0.01);
    assert!(output.right[50].abs() < 0.01);
}

#[test]
fn test_stereo_layer_pan_right() {
    let mut mixer = Mixer::new(100, 44100.0);
    // Stereo layer with distinct L/R content, panned hard right
    mixer.add_layer(Layer::new_stereo(vec![1.0; 100], vec![0.5; 100], 1.0, 1.0));

    let output = mixer.mix_stereo();

    // At pan=1 (hard right), both channels go to right output
    // pan_normalized = 1, so:
    // left_to_left = 0.0, left_to_right = 1.0
    // right_to_left = 0.0, right_to_right = 1.0
    // output.left = 1.0 * 0.0 + 0.5 * 0.0 = 0.0
    // output.right = 1.0 * 1.0 + 0.5 * 1.0 = 1.5
    assert!(output.left[50].abs() < 0.01);
    assert!((output.right[50] - 1.5).abs() < 0.01);
}

#[test]
fn test_stereo_layer_with_volume() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_layer(Layer::new_stereo(vec![1.0; 100], vec![1.0; 100], 0.5, 0.0));

    let output = mixer.mix_stereo();

    // Volume 0.5 should halve the output
    // At center pan with equal L/R, output should be 0.5 in both channels
    assert!((output.left[50] - 0.5).abs() < 0.01);
    assert!((output.right[50] - 0.5).abs() < 0.01);
}

#[test]
fn test_stereo_layer_mixer_detects_stereo() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_layer(Layer::new_stereo(vec![1.0; 100], vec![1.0; 100], 1.0, 0.0));

    // Mixer should detect stereo content from stereo layer
    assert!(mixer.is_stereo());
}

#[test]
fn test_stereo_layer_mix_mono_downmixes() {
    let mut mixer = Mixer::new(100, 44100.0);
    // Stereo layer: left=1.0, right=0.5
    mixer.add_layer(Layer::new_stereo(vec![1.0; 100], vec![0.5; 100], 1.0, 0.0));

    let output = mixer.mix_mono();

    // Mono mixing should average the channels
    // (1.0 + 0.5) * 0.5 = 0.75
    assert!((output[50] - 0.75).abs() < 0.01);
}

#[test]
fn test_stereo_layer_centered_constructor() {
    let mut mixer = Mixer::new(100, 44100.0);
    mixer.add_layer(Layer::centered_stereo(vec![1.0; 100], vec![0.5; 100], 1.0));

    let output = mixer.mix_stereo();

    // centered_stereo should have pan=0
    // Output should blend L and R to both channels equally
    assert!((output.left[50] - 0.75).abs() < 0.01);
    assert!((output.right[50] - 0.75).abs() < 0.01);
}

#[test]
fn test_mixed_mono_and_stereo_layers() {
    let mut mixer = Mixer::new(100, 44100.0);
    // Add a mono layer panned left
    mixer.add_panned(vec![0.5; 100], 1.0, -1.0);
    // Add a stereo layer centered
    mixer.add_layer(Layer::centered_stereo(vec![0.3; 100], vec![0.3; 100], 1.0));

    let output = mixer.mix_stereo();

    // Mono layer: hard left, so only contributes to left channel (0.5)
    // Stereo layer: center, both channels get 0.3 * 0.5 + 0.3 * 0.5 = 0.3
    // Total left: 0.5 + 0.3 = 0.8
    // Total right: 0.0 + 0.3 = 0.3
    assert!((output.left[50] - 0.8).abs() < 0.01);
    assert!((output.right[50] - 0.3).abs() < 0.01);
}
