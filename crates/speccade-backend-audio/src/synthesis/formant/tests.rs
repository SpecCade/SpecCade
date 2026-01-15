//! Tests for formant synthesis.

use super::*;
use crate::rng::create_rng;
use crate::synthesis::Synthesizer;

#[test]
fn test_formant_basic() {
    let synth = FormantSynth::vowel_a(110.0);
    let mut rng = create_rng(42);
    let samples = synth.synthesize(4410, 44100.0, &mut rng);

    assert_eq!(samples.len(), 4410);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s), "Sample {} out of range", s);
    }
}

#[test]
fn test_formant_vowel_i() {
    let synth = FormantSynth::vowel_i(220.0);
    let mut rng = create_rng(42);
    let samples = synth.synthesize(4410, 44100.0, &mut rng);

    assert_eq!(samples.len(), 4410);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_formant_vowel_u() {
    let synth = FormantSynth::vowel_u(165.0);
    let mut rng = create_rng(42);
    let samples = synth.synthesize(4410, 44100.0, &mut rng);

    assert_eq!(samples.len(), 4410);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_formant_choir_ah() {
    let synth = FormantSynth::choir_ah(110.0);
    let mut rng = create_rng(42);
    let samples = synth.synthesize(4410, 44100.0, &mut rng);

    assert_eq!(samples.len(), 4410);
    assert!(synth.breathiness > 0.0);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_formant_creature_growl() {
    let synth = FormantSynth::creature_growl(55.0);
    let mut rng = create_rng(42);
    let samples = synth.synthesize(4410, 44100.0, &mut rng);

    assert_eq!(samples.len(), 4410);
    assert!(synth.breathiness > 0.3);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_formant_custom_formants() {
    let formants = vec![
        Formant::new(400.0, 1.0, 5.0),
        Formant::new(1000.0, 0.7, 6.0),
        Formant::new(2500.0, 0.5, 7.0),
    ];
    let synth = FormantSynth::new(110.0, formants);
    let mut rng = create_rng(42);
    let samples = synth.synthesize(4410, 44100.0, &mut rng);

    assert_eq!(samples.len(), 4410);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_formant_determinism() {
    let synth = FormantSynth::vowel_a(110.0);

    let mut rng1 = create_rng(42);
    let mut rng2 = create_rng(42);

    let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
    let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

    assert_eq!(samples1, samples2);
}

#[test]
fn test_formant_different_seeds() {
    let synth = FormantSynth::choir_ah(110.0);

    let mut rng1 = create_rng(42);
    let mut rng2 = create_rng(43);

    let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
    let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

    // Should be different due to noise in breathiness
    assert_ne!(samples1, samples2);
}

#[test]
fn test_formant_empty_samples() {
    let synth = FormantSynth::vowel_a(110.0);
    let mut rng = create_rng(42);
    let samples = synth.synthesize(0, 44100.0, &mut rng);

    assert!(samples.is_empty());
}

#[test]
fn test_formant_vowel_morph() {
    let synth =
        FormantSynth::with_vowel(110.0, VowelPreset::A).with_vowel_morph(VowelPreset::I, 0.5);

    let mut rng = create_rng(42);
    let samples = synth.synthesize(4410, 44100.0, &mut rng);

    assert_eq!(samples.len(), 4410);
    for &s in &samples {
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn test_formant_breathiness_range() {
    let synth1 = FormantSynth::vowel_a(110.0).with_breathiness(0.0);
    let synth2 = FormantSynth::vowel_a(110.0).with_breathiness(1.0);

    assert_eq!(synth1.breathiness, 0.0);
    assert_eq!(synth2.breathiness, 1.0);

    // Out of range should be clamped
    let synth3 = FormantSynth::vowel_a(110.0).with_breathiness(2.0);
    assert_eq!(synth3.breathiness, 1.0);
}

#[test]
fn test_vowel_target_formants() {
    let vowel = VowelTarget::vowel_a();
    let formants = vowel.to_formants();

    // Should have at least 3 formants, possibly more
    assert!(formants.len() >= 3);

    // F1 for /a/ should be around 800 Hz
    assert!((formants[0].frequency - 800.0).abs() < 1.0);
}

#[test]
fn test_formant_clamping() {
    let formant = Formant::new(10.0, 2.0, 0.1);
    assert!(formant.frequency >= 20.0);
    assert!(formant.amplitude <= 1.0);
    assert!(formant.bandwidth >= 0.5);

    let formant2 = Formant::new(30000.0, -0.5, 50.0);
    assert!(formant2.frequency <= 20000.0);
    assert!(formant2.amplitude >= 0.0);
    assert!(formant2.bandwidth <= 20.0);
}

#[test]
fn test_all_vowel_presets() {
    let presets = [
        VowelPreset::A,
        VowelPreset::I,
        VowelPreset::U,
        VowelPreset::E,
        VowelPreset::O,
    ];

    for preset in presets {
        let synth = FormantSynth::with_vowel(110.0, preset);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }
}
