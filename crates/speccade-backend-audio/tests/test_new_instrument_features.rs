//! Integration tests for new instrument features.

use speccade_backend_audio::generate;
use speccade_spec::recipe::audio::{
    AudioV1Params, Envelope, NoteSpec, OscillatorConfig, PitchEnvelope, Synthesis, Waveform,
};
use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe, Spec};

fn create_instrument_spec(params: AudioV1Params, seed: u32, name: &str) -> Spec {
    Spec::builder(name, AssetType::Audio)
        .license("CC0-1.0")
        .seed(seed)
        .output(OutputSpec::primary(
            OutputFormat::Wav,
            format!("instruments/{}.wav", name),
        ))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::to_value(&params).unwrap(),
        ))
        .build()
}

#[test]
fn test_instrument_with_detune() {
    let params = AudioV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![speccade_spec::recipe::audio::AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0,
                freq_sweep: None,
                detune: Some(50.0), // 50 cents up
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 1.0,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
        pitch_envelope: None,
        base_note: Some(NoteSpec::MidiNote(69)), // A4
        generate_loop_points: false,
    };

    let spec = create_instrument_spec(params, 42, "test-detune");
    let result = generate(&spec);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.base_note, Some(69));
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_instrument_with_duty_cycle() {
    let params = AudioV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![speccade_spec::recipe::audio::AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Square,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: Some(0.25), // 25% duty cycle
            },
            envelope: Envelope::default(),
            volume: 1.0,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
        pitch_envelope: None,
        base_note: Some(NoteSpec::MidiNote(60)), // C4
        generate_loop_points: false,
    };

    let spec = create_instrument_spec(params, 42, "test-duty");
    let result = generate(&spec);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.base_note, Some(60));
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_instrument_with_pitch_envelope() {
    let params = AudioV1Params {
        duration_seconds: 1.0,
        sample_rate: 44100,
        layers: vec![speccade_spec::recipe::audio::AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 1.0,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
        pitch_envelope: Some(PitchEnvelope {
            attack: 0.1,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
            depth: 12.0, // 1 octave up
        }),
        base_note: Some(NoteSpec::MidiNote(69)),
        generate_loop_points: false,
    };

    let spec = create_instrument_spec(params, 42, "test-pitch-env");
    let result = generate(&spec);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.base_note, Some(69));
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_instrument_multi_oscillator() {
    let params = AudioV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![speccade_spec::recipe::audio::AudioLayer {
            synthesis: Synthesis::MultiOscillator {
                frequency: 440.0,
                oscillators: vec![
                    OscillatorConfig {
                        waveform: Waveform::Sawtooth,
                        volume: 0.5,
                        detune: Some(0.0), // Fundamental
                        phase: None,
                        duty: None,
                    },
                    OscillatorConfig {
                        waveform: Waveform::Sawtooth,
                        volume: 0.3,
                        detune: Some(7.0), // Perfect fifth (7 semitones = 700 cents)
                        phase: None,
                        duty: None,
                    },
                    OscillatorConfig {
                        waveform: Waveform::Sawtooth,
                        volume: 0.2,
                        detune: Some(-12.0), // Octave down (12 semitones = 1200 cents)
                        phase: None,
                        duty: None,
                    },
                ],
                freq_sweep: None,
            },
            envelope: Envelope::default(),
            volume: 1.0,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
        pitch_envelope: None,
        base_note: Some(NoteSpec::MidiNote(60)), // C4
        generate_loop_points: false,
    };

    let spec = create_instrument_spec(params, 42, "test-multi-osc");
    let result = generate(&spec);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.base_note, Some(60));
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_instrument_multi_oscillator_with_pitch_envelope() {
    let params = AudioV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![speccade_spec::recipe::audio::AudioLayer {
            synthesis: Synthesis::MultiOscillator {
                frequency: 440.0,
                oscillators: vec![
                    OscillatorConfig {
                        waveform: Waveform::Square,
                        volume: 0.5,
                        detune: Some(0.0),
                        phase: None,
                        duty: Some(0.5),
                    },
                    OscillatorConfig {
                        waveform: Waveform::Square,
                        volume: 0.5,
                        detune: Some(5.0), // Slightly detuned
                        phase: None,
                        duty: Some(0.25),
                    },
                ],
                freq_sweep: None,
            },
            envelope: Envelope::default(),
            volume: 1.0,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
        pitch_envelope: Some(PitchEnvelope {
            attack: 0.05,
            decay: 0.1,
            sustain: 0.8,
            release: 0.1,
            depth: -12.0, // Octave down
        }),
        base_note: Some(NoteSpec::MidiNote(69)),
        generate_loop_points: false,
    };

    let spec = create_instrument_spec(params, 42, "test-multi-osc-pitch");
    let result = generate(&spec);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.base_note, Some(69));
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_instrument_determinism_with_new_features() {
    let params = AudioV1Params {
        duration_seconds: 0.3,
        sample_rate: 22050,
        layers: vec![speccade_spec::recipe::audio::AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Square,
                frequency: 440.0,
                freq_sweep: None,
                detune: Some(10.0),
                duty: Some(0.3),
            },
            envelope: Envelope::default(),
            volume: 1.0,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
        pitch_envelope: Some(PitchEnvelope {
            attack: 0.05,
            decay: 0.05,
            sustain: 0.7,
            release: 0.1,
            depth: 5.0,
        }),
        base_note: Some(NoteSpec::MidiNote(60)),
        generate_loop_points: false,
    };

    let spec1 = create_instrument_spec(params.clone(), 42, "test-determinism-1");
    let spec2 = create_instrument_spec(params, 42, "test-determinism-2");

    let result1 = generate(&spec1).expect("first generation");
    let result2 = generate(&spec2).expect("second generation");

    // Should produce identical output with same seed
    assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);
}
