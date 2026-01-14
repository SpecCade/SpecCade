//! Tests for vocoder synthesis.

use super::*;
use crate::rng::create_rng;
use crate::synthesis::Synthesizer;

#[test]
fn test_vocoder_basic() {
    let synth = VocoderSynth::new(
        110.0,
        CarrierType::Sawtooth,
        16,
        BandSpacing::Logarithmic,
        0.01,
        0.05,
    );
    let mut rng = create_rng(42);
    let samples = synth.synthesize(4410, 44100.0, &mut rng);

    assert_eq!(samples.len(), 4410);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s), "Sample {} out of range", s);
    }
}

#[test]
fn test_vocoder_robot_voice_preset() {
    let synth = VocoderSynth::robot_voice(110.0);
    let mut rng = create_rng(42);
    let samples = synth.synthesize(4410, 44100.0, &mut rng);

    assert_eq!(samples.len(), 4410);
    assert_eq!(synth.carrier_type, CarrierType::Sawtooth);
    assert_eq!(synth.num_bands, 16);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_vocoder_choir_preset() {
    let synth = VocoderSynth::choir(220.0);
    let mut rng = create_rng(42);
    let samples = synth.synthesize(4410, 44100.0, &mut rng);

    assert_eq!(samples.len(), 4410);
    assert_eq!(synth.carrier_type, CarrierType::Noise);
    assert_eq!(synth.num_bands, 24);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_vocoder_strings_preset() {
    let synth = VocoderSynth::strings_through_vocoder(165.0);
    let mut rng = create_rng(42);
    let samples = synth.synthesize(4410, 44100.0, &mut rng);

    assert_eq!(samples.len(), 4410);
    assert_eq!(synth.carrier_type, CarrierType::Pulse);
    assert_eq!(synth.num_bands, 20);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_vocoder_determinism() {
    let synth = VocoderSynth::robot_voice(110.0);

    let mut rng1 = create_rng(42);
    let mut rng2 = create_rng(42);

    let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
    let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

    assert_eq!(samples1, samples2);
}

#[test]
fn test_vocoder_different_seeds() {
    let synth = VocoderSynth::robot_voice(110.0);

    let mut rng1 = create_rng(42);
    let mut rng2 = create_rng(43);

    let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
    let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

    // Should be different due to random formant patterns
    assert_ne!(samples1, samples2);
}

#[test]
fn test_vocoder_linear_spacing() {
    let synth = VocoderSynth::new(
        110.0,
        CarrierType::Sawtooth,
        8,
        BandSpacing::Linear,
        0.01,
        0.05,
    );
    let mut rng = create_rng(42);
    let samples = synth.synthesize(2000, 44100.0, &mut rng);

    assert_eq!(samples.len(), 2000);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_vocoder_noise_carrier() {
    let synth = VocoderSynth::new(
        110.0,
        CarrierType::Noise,
        12,
        BandSpacing::Logarithmic,
        0.01,
        0.05,
    );
    let mut rng = create_rng(42);
    let samples = synth.synthesize(2000, 44100.0, &mut rng);

    assert_eq!(samples.len(), 2000);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_vocoder_pulse_carrier() {
    let synth = VocoderSynth::new(
        110.0,
        CarrierType::Pulse,
        12,
        BandSpacing::Logarithmic,
        0.01,
        0.05,
    );
    let mut rng = create_rng(42);
    let samples = synth.synthesize(2000, 44100.0, &mut rng);

    assert_eq!(samples.len(), 2000);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_vocoder_empty_samples() {
    let synth = VocoderSynth::robot_voice(110.0);
    let mut rng = create_rng(42);
    let samples = synth.synthesize(0, 44100.0, &mut rng);

    assert!(samples.is_empty());
}

#[test]
fn test_vocoder_custom_bands() {
    let bands = vec![
        VocoderBand::new(200.0, 4.0, vec![]),
        VocoderBand::new(500.0, 4.0, vec![]),
        VocoderBand::new(1000.0, 4.0, vec![]),
        VocoderBand::new(2000.0, 4.0, vec![]),
    ];

    let synth = VocoderSynth::new(
        110.0,
        CarrierType::Sawtooth,
        4,
        BandSpacing::Logarithmic,
        0.01,
        0.05,
    )
    .with_bands(bands);

    let mut rng = create_rng(42);
    let samples = synth.synthesize(2000, 44100.0, &mut rng);

    assert_eq!(samples.len(), 2000);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_vocoder_band_clamping() {
    let band = VocoderBand::new(10.0, 0.1, vec![]); // Below min
    assert!(band.center_freq >= 20.0);
    assert!(band.bandwidth >= 0.5);

    let band2 = VocoderBand::new(30000.0, 50.0, vec![]); // Above max
    assert!(band2.center_freq <= 20000.0);
    assert!(band2.bandwidth <= 20.0);
}

#[test]
fn test_interpolate_pattern() {
    let pattern = vec![0.0, 1.0];
    let result = processor::interpolate_pattern(&pattern, 5);

    assert_eq!(result.len(), 5);
    assert!((result[0] - 0.0).abs() < 0.01);
    assert!((result[2] - 0.5).abs() < 0.01);
    assert!((result[4] - 1.0).abs() < 0.01);
}

#[test]
fn test_interpolate_empty_pattern() {
    let pattern: Vec<f64> = vec![];
    let result = processor::interpolate_pattern(&pattern, 10);

    assert_eq!(result.len(), 10);
    for &v in &result {
        assert!((v - 0.5).abs() < 0.01);
    }
}
