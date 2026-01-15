//! Tests for vector synthesis.

use crate::rng::create_rng;
use crate::synthesis::Synthesizer;

use super::presets::{evolving_pad, morph_texture, sweep_corners};
use super::synth::VectorSynth;
use super::types::{VectorPath, VectorPathPoint, VectorPosition, VectorSource};

#[test]
fn test_vector_position_new() {
    let pos = VectorPosition::new(0.5, 0.5);
    assert!((pos.x - 0.5).abs() < 0.001);
    assert!((pos.y - 0.5).abs() < 0.001);
}

#[test]
fn test_vector_position_clamp() {
    let pos = VectorPosition::new(-0.5, 1.5);
    assert!((pos.x - 0.0).abs() < 0.001);
    assert!((pos.y - 1.0).abs() < 0.001);
}

#[test]
fn test_vector_mix_weights_center() {
    let pos = VectorPosition::center();
    let weights = VectorSynth::calculate_mix_weights(&pos);

    // At center (0.5, 0.5), all corners should have equal weight (0.25)
    for weight in &weights {
        assert!((*weight - 0.25).abs() < 0.001);
    }
}

#[test]
fn test_vector_mix_weights_corners() {
    // Corner A (top-left)
    let weights_a = VectorSynth::calculate_mix_weights(&VectorPosition::corner_a());
    assert!((weights_a[0] - 1.0).abs() < 0.001); // A = 1.0
    assert!(weights_a[1].abs() < 0.001); // B = 0.0
    assert!(weights_a[2].abs() < 0.001); // C = 0.0
    assert!(weights_a[3].abs() < 0.001); // D = 0.0

    // Corner D (bottom-right)
    let weights_d = VectorSynth::calculate_mix_weights(&VectorPosition::corner_d());
    assert!(weights_d[0].abs() < 0.001); // A = 0.0
    assert!(weights_d[1].abs() < 0.001); // B = 0.0
    assert!(weights_d[2].abs() < 0.001); // C = 0.0
    assert!((weights_d[3] - 1.0).abs() < 0.001); // D = 1.0
}

#[test]
fn test_vector_synth_basic() {
    let sources = [
        VectorSource::sine(1.0),
        VectorSource::saw(1.0),
        VectorSource::triangle(1.0),
        VectorSource::square(1.0),
    ];
    let synth = VectorSynth::new(440.0, sources).with_position(VectorPosition::center());

    let mut rng = create_rng(42);
    let samples = synth.synthesize(1000, 44100.0, &mut rng);

    assert_eq!(samples.len(), 1000);
    for &s in &samples {
        assert!(s.is_finite());
        assert!(s.abs() <= 1.0);
    }
}

#[test]
fn test_vector_synth_with_path() {
    let sources = [
        VectorSource::sine(1.0),
        VectorSource::saw(1.0),
        VectorSource::triangle(1.0),
        VectorSource::square(1.0),
    ];

    let path = VectorPath::new(vec![
        VectorPathPoint::new(VectorPosition::corner_a(), 0.1),
        VectorPathPoint::new(VectorPosition::corner_d(), 0.1),
    ]);

    let synth = VectorSynth::new(440.0, sources).with_path(path, false);

    let mut rng = create_rng(42);
    let samples = synth.synthesize(1000, 44100.0, &mut rng);

    assert_eq!(samples.len(), 1000);
    for &s in &samples {
        assert!(s.is_finite());
        assert!(s.abs() <= 1.0);
    }
}

#[test]
fn test_vector_synth_determinism() {
    let sources = [
        VectorSource::sine(1.0),
        VectorSource::noise(),
        VectorSource::triangle(1.0),
        VectorSource::wavetable(1.0),
    ];
    let synth = VectorSynth::new(440.0, sources).with_position(VectorPosition::new(0.3, 0.7));

    let mut rng1 = create_rng(42);
    let mut rng2 = create_rng(42);

    let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
    let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

    assert_eq!(samples1, samples2);
}

#[test]
fn test_evolving_pad_preset() {
    let synth = evolving_pad(220.0, 1.0);

    let mut rng = create_rng(42);
    let samples = synth.synthesize(1000, 44100.0, &mut rng);

    assert_eq!(samples.len(), 1000);
    for &s in &samples {
        assert!(s.is_finite());
        assert!(s.abs() <= 1.0);
    }
}

#[test]
fn test_morph_texture_preset() {
    let synth = morph_texture(330.0, 0.5);

    let mut rng = create_rng(42);
    let samples = synth.synthesize(1000, 44100.0, &mut rng);

    assert_eq!(samples.len(), 1000);
    for &s in &samples {
        assert!(s.is_finite());
        assert!(s.abs() <= 1.0);
    }
}

#[test]
fn test_sweep_corners_preset() {
    let synth = sweep_corners(440.0, 2.0);

    let mut rng = create_rng(42);
    let samples = synth.synthesize(1000, 44100.0, &mut rng);

    assert_eq!(samples.len(), 1000);
    for &s in &samples {
        assert!(s.is_finite());
        assert!(s.abs() <= 1.0);
    }
}

#[test]
fn test_vector_path_position_at() {
    let path = VectorPath::new(vec![
        VectorPathPoint::new(VectorPosition::new(0.0, 0.0), 0.0),
        VectorPathPoint::new(VectorPosition::new(1.0, 1.0), 1.0),
    ]);

    // At start
    let pos_start = path.position_at(0.0, false);
    assert!((pos_start.x - 0.0).abs() < 0.001);
    assert!((pos_start.y - 0.0).abs() < 0.001);

    // At middle
    let pos_mid = path.position_at(0.5, false);
    assert!((pos_mid.x - 0.5).abs() < 0.001);
    assert!((pos_mid.y - 0.5).abs() < 0.001);

    // At end
    let pos_end = path.position_at(1.0, false);
    assert!((pos_end.x - 1.0).abs() < 0.001);
    assert!((pos_end.y - 1.0).abs() < 0.001);
}

#[test]
fn test_vector_path_looping() {
    let path = VectorPath::new(vec![
        VectorPathPoint::new(VectorPosition::new(0.0, 0.0), 0.0),
        VectorPathPoint::new(VectorPosition::new(1.0, 0.0), 1.0),
    ]);

    // With looping, time 1.5 should be equivalent to time 0.5
    let pos_loop = path.position_at(1.5, true);
    let pos_half = path.position_at(0.5, false);
    assert!((pos_loop.x - pos_half.x).abs() < 0.001);
    assert!((pos_loop.y - pos_half.y).abs() < 0.001);
}
