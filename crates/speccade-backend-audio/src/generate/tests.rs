//! Tests for audio generation.

use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params, DetuneCurve, Envelope, NoiseType, Synthesis, Waveform,
};
use speccade_spec::recipe::Recipe;
use speccade_spec::{AssetType, OutputFormat, OutputSpec, Spec};

use super::{generate, generate_from_params};

fn create_test_spec() -> Spec {
    let params = AudioV1Params {
        duration_seconds: 0.5,
        sample_rate: 44100,
        master_filter: None,
        layers: vec![AudioLayer {
            synthesis: Synthesis::FmSynth {
                carrier_freq: 440.0,
                modulator_freq: 880.0,
                modulation_index: 2.0,
                freq_sweep: None,
            },
            envelope: Envelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.5,
                release: 0.2,
            },
            volume: 0.8,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    Spec::builder("test-sfx", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::to_value(&params).unwrap(),
        ))
        .build()
}

#[test]
fn test_generate_basic() {
    let spec = create_test_spec();
    let result = generate(&spec).expect("should generate");

    assert_eq!(result.num_layers, 1);
    assert!(!result.wav.wav_data.is_empty());
    assert_eq!(result.wav.sample_rate, 44100);
}

#[test]
fn test_generate_determinism() {
    let spec = create_test_spec();

    let result1 = generate(&spec).expect("should generate");
    let result2 = generate(&spec).expect("should generate");

    assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);
}

#[test]
fn test_generate_different_seeds() {
    let params = AudioV1Params {
        duration_seconds: 0.1,
        sample_rate: 22050,
        master_filter: None,
        layers: vec![AudioLayer {
            synthesis: Synthesis::NoiseBurst {
                noise_type: NoiseType::White,
                filter: None,
            },
            envelope: Envelope::default(),
            volume: 1.0,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let spec1 = Spec::builder("test-sfx", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::to_value(&params).unwrap(),
        ))
        .build();

    let mut spec2 = spec1.clone();
    spec2.seed = 43;

    let result1 = generate(&spec1).expect("should generate");
    let result2 = generate(&spec2).expect("should generate");

    assert_ne!(result1.wav.pcm_hash, result2.wav.pcm_hash);
}

#[test]
fn test_generate_stereo() {
    let params = AudioV1Params {
        duration_seconds: 0.1,
        sample_rate: 44100,
        master_filter: None,
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
        layers: vec![
            AudioLayer {
                synthesis: Synthesis::Oscillator {
                    waveform: Waveform::Sine,
                    frequency: 440.0,
                    freq_sweep: None,
                    detune: None,
                    duty: None,
                },
                envelope: Envelope::default(),
                volume: 0.5,
                pan: -0.8, // Left
                delay: None,
                filter: None,
                lfo: None,
            },
            AudioLayer {
                synthesis: Synthesis::Oscillator {
                    waveform: Waveform::Sine,
                    frequency: 550.0,
                    freq_sweep: None,
                    detune: None,
                    duty: None,
                },
                envelope: Envelope::default(),
                volume: 0.5,
                pan: 0.8, // Right
                delay: None,
                filter: None,
                lfo: None,
            },
        ],
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let result = generate_from_params(&params, 42).expect("should generate");
    assert!(result.wav.is_stereo);
}

#[test]
fn test_generate_supersaw_unison() {
    let params = AudioV1Params {
        duration_seconds: 0.5,
        sample_rate: 44100,
        master_filter: None,
        layers: vec![AudioLayer {
            synthesis: Synthesis::SupersawUnison {
                frequency: 440.0,
                voices: 7,
                detune_cents: 25.0,
                spread: 0.8,
                detune_curve: DetuneCurve::Linear,
            },
            envelope: Envelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.2,
            },
            volume: 0.8,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let result = generate_from_params(&params, 42).expect("should generate");

    // Supersaw with spread should produce stereo output
    assert!(result.wav.is_stereo);
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_generate_supersaw_unison_determinism() {
    let params = AudioV1Params {
        duration_seconds: 0.2,
        sample_rate: 22050,
        master_filter: None,
        layers: vec![AudioLayer {
            synthesis: Synthesis::SupersawUnison {
                frequency: 220.0,
                voices: 5,
                detune_cents: 20.0,
                spread: 0.5,
                detune_curve: DetuneCurve::Exp2,
            },
            envelope: Envelope::default(),
            volume: 1.0,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let result1 = generate_from_params(&params, 42).expect("should generate");
    let result2 = generate_from_params(&params, 42).expect("should generate");

    // Same seed should produce identical output
    assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);
}

#[test]
fn test_generate_supersaw_unison_single_voice() {
    // A single voice should be mono (unless spread forces stereo, which it wont for 1 voice)
    let params = AudioV1Params {
        duration_seconds: 0.1,
        sample_rate: 44100,
        master_filter: None,
        layers: vec![AudioLayer {
            synthesis: Synthesis::SupersawUnison {
                frequency: 440.0,
                voices: 1,
                detune_cents: 25.0, // Ignored for single voice
                spread: 0.8,        // Ignored for single voice
                detune_curve: DetuneCurve::Linear,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0, // Center pan
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let result = generate_from_params(&params, 42).expect("should generate");
    assert!(!result.wav.wav_data.is_empty());
    // Single voice with center pan should be mono
    assert!(!result.wav.is_stereo);
}

#[test]
fn test_generate_supersaw_unison_with_delay() {
    let params = AudioV1Params {
        duration_seconds: 0.5,
        sample_rate: 44100,
        master_filter: None,
        layers: vec![AudioLayer {
            synthesis: Synthesis::SupersawUnison {
                frequency: 330.0,
                voices: 3,
                detune_cents: 15.0,
                spread: 0.6,
                detune_curve: DetuneCurve::Linear,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: Some(0.1), // 100ms delay
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let result = generate_from_params(&params, 42).expect("should generate");
    assert!(!result.wav.wav_data.is_empty());
}
