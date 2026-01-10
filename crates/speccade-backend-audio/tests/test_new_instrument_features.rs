//! Integration tests for new instrument features.

use speccade_backend_audio::instrument::generate_instrument;
use speccade_spec::recipe::audio_instrument::{
    AudioInstrumentSynthPatchV1Params, InstrumentSynthesis, NoteSpec,
};
use speccade_spec::recipe::audio_sfx::{
    Envelope, OscillatorConfig, PitchEnvelope, Synthesis, Waveform,
};

#[test]
fn test_instrument_with_detune() {
    let params = AudioInstrumentSynthPatchV1Params {
        note_duration_seconds: 0.5,
        sample_rate: 22050,
        synthesis: InstrumentSynthesis::Simple {
            synthesis: Synthesis::Oscillator {
            waveform: Waveform::Sine,
            frequency: 440.0,
            freq_sweep: None,
            detune: Some(50.0), // 50 cents up
            duty: None,
            },
        },
        envelope: Envelope::default(),
        pitch_envelope: None,
        notes: Some(vec![NoteSpec::MidiNote(69)]), // A4
        generate_loop_points: false,
    };

    let result = generate_instrument(&params, 42);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.notes, vec![69]);
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_instrument_with_duty_cycle() {
    let params = AudioInstrumentSynthPatchV1Params {
        note_duration_seconds: 0.5,
        sample_rate: 22050,
        synthesis: InstrumentSynthesis::Simple {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Square,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: Some(0.25), // 25% duty cycle
            },
        },
        envelope: Envelope::default(),
        pitch_envelope: None,
        notes: Some(vec![NoteSpec::MidiNote(60)]), // C4
        generate_loop_points: false,
    };

    let result = generate_instrument(&params, 42);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.notes, vec![60]);
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_instrument_with_pitch_envelope() {
    let params = AudioInstrumentSynthPatchV1Params {
        note_duration_seconds: 1.0,
        sample_rate: 44100,
        synthesis: InstrumentSynthesis::Simple {
            synthesis: Synthesis::Oscillator {
            waveform: Waveform::Sine,
            frequency: 440.0,
            freq_sweep: None,
            detune: None,
            duty: None,
            },
        },
        envelope: Envelope::default(),
        pitch_envelope: Some(PitchEnvelope {
            attack: 0.1,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
            depth: 12.0, // 1 octave up
        }),
        notes: Some(vec![NoteSpec::MidiNote(69)]),
        generate_loop_points: false,
    };

    let result = generate_instrument(&params, 42);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.notes, vec![69]);
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_instrument_multi_oscillator() {
    let params = AudioInstrumentSynthPatchV1Params {
        note_duration_seconds: 0.5,
        sample_rate: 22050,
        synthesis: InstrumentSynthesis::Simple {
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
        },
        envelope: Envelope::default(),
        pitch_envelope: None,
        notes: Some(vec![NoteSpec::MidiNote(60)]), // C4
        generate_loop_points: false,
    };

    let result = generate_instrument(&params, 42);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.notes, vec![60]);
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_instrument_multi_oscillator_with_pitch_envelope() {
    let params = AudioInstrumentSynthPatchV1Params {
        note_duration_seconds: 0.5,
        sample_rate: 22050,
        synthesis: InstrumentSynthesis::Simple {
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
        },
        envelope: Envelope::default(),
        pitch_envelope: Some(PitchEnvelope {
            attack: 0.05,
            decay: 0.1,
            sustain: 0.8,
            release: 0.1,
            depth: -12.0, // Octave down
        }),
        notes: Some(vec![NoteSpec::MidiNote(69)]),
        generate_loop_points: false,
    };

    let result = generate_instrument(&params, 42);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.notes, vec![69]);
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_instrument_determinism_with_new_features() {
    let params = AudioInstrumentSynthPatchV1Params {
        note_duration_seconds: 0.3,
        sample_rate: 22050,
        synthesis: InstrumentSynthesis::Simple {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Square,
                frequency: 440.0,
                freq_sweep: None,
                detune: Some(10.0),
                duty: Some(0.3),
            },
        },
        envelope: Envelope::default(),
        pitch_envelope: Some(PitchEnvelope {
            attack: 0.05,
            decay: 0.05,
            sustain: 0.7,
            release: 0.1,
            depth: 5.0,
        }),
        notes: Some(vec![NoteSpec::MidiNote(60)]),
        generate_loop_points: false,
    };

    let result1 = generate_instrument(&params, 42).expect("first generation");
    let result2 = generate_instrument(&params, 42).expect("second generation");

    // Should produce identical output with same seed
    assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);
}
