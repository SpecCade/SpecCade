//! Tests for Layer construction and manipulation.

use super::*;

#[test]
fn test_layer_new() {
    let samples = vec![0.1, 0.2, 0.3];
    let layer = Layer::new(samples.clone(), 0.8, 0.5);
    assert_eq!(layer.samples, samples);
    assert_eq!(layer.volume, 0.8);
    assert_eq!(layer.pan, 0.5);
    assert_eq!(layer.delay_samples, 0);
}

#[test]
fn test_layer_new_clamps_volume() {
    // Volume above 1.0 should be clamped
    let layer_high = Layer::new(vec![0.5], 1.5, 0.0);
    assert_eq!(layer_high.volume, 1.0);

    // Volume below 0.0 should be clamped
    let layer_low = Layer::new(vec![0.5], -0.5, 0.0);
    assert_eq!(layer_low.volume, 0.0);
}

#[test]
fn test_layer_new_clamps_pan() {
    // Pan above 1.0 should be clamped
    let layer_high = Layer::new(vec![0.5], 1.0, 2.0);
    assert_eq!(layer_high.pan, 1.0);

    // Pan below -1.0 should be clamped
    let layer_low = Layer::new(vec![0.5], 1.0, -2.0);
    assert_eq!(layer_low.pan, -1.0);
}

#[test]
fn test_layer_centered() {
    let samples = vec![0.1, 0.2, 0.3];
    let layer = Layer::centered(samples.clone(), 0.75);
    assert_eq!(layer.samples, samples);
    assert_eq!(layer.volume, 0.75);
    assert_eq!(layer.pan, 0.0); // Centered means pan = 0.0
    assert_eq!(layer.delay_samples, 0);
}

#[test]
fn test_layer_with_delay() {
    let samples = vec![1.0, 2.0, 3.0];
    let layer = Layer::new(samples, 1.0, 0.0).with_delay(100);
    assert_eq!(layer.delay_samples, 100);
}

#[test]
fn test_layer_with_delay_chainable() {
    let samples = vec![1.0, 2.0];
    let layer = Layer::new(samples, 0.5, 0.25)
        .with_delay(50)
        .with_delay(100); // Second call should override
    assert_eq!(layer.delay_samples, 100);
    assert_eq!(layer.volume, 0.5);
    assert_eq!(layer.pan, 0.25);
}

#[test]
fn test_layer_with_delay_seconds() {
    let samples = vec![1.0, 2.0, 3.0];
    let sample_rate = 44100.0;
    let delay_seconds = 0.5; // 500ms

    let layer = Layer::new(samples, 1.0, 0.0).with_delay_seconds(delay_seconds, sample_rate);

    // 0.5 seconds at 44100 Hz = 22050 samples
    let expected_delay = (delay_seconds * sample_rate).round() as usize;
    assert_eq!(layer.delay_samples, expected_delay);
    assert_eq!(layer.delay_samples, 22050);
}

#[test]
fn test_layer_with_delay_seconds_fractional() {
    let samples = vec![1.0];
    let sample_rate = 48000.0;
    let delay_seconds = 0.001; // 1ms

    let layer = Layer::new(samples, 1.0, 0.0).with_delay_seconds(delay_seconds, sample_rate);

    // 0.001 seconds at 48000 Hz = 48 samples
    assert_eq!(layer.delay_samples, 48);
}
