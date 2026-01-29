//! Tests for music generation module.

use super::*;
use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params, Envelope, NoiseType, NoteSpec as AudioNoteSpec,
    Synthesis as AudioSynthesis, Waveform,
};
use speccade_spec::recipe::music::{
    ArrangementEntry, InstrumentSynthesis, PatternNote, TrackerFormat, TrackerInstrument,
    TrackerPattern,
};
use std::collections::HashMap;

fn create_test_params() -> MusicTrackerSongV1Params {
    let envelope = Envelope {
        attack: 0.01,
        decay: 0.1,
        sustain: 0.5,
        release: 0.2,
    };

    let instrument = TrackerInstrument {
        name: "Test Lead".to_string(),
        synthesis: Some(InstrumentSynthesis::Pulse { duty_cycle: 0.5 }),
        envelope: envelope.clone(),
        default_volume: Some(64),
        ..Default::default()
    };

    let mut notes = HashMap::new();
    notes.insert(
        "0".to_string(),
        vec![
            PatternNote {
                row: 0,
                note: "C4".to_string(),
                inst: 0,
                vol: Some(64),
                ..Default::default()
            },
            PatternNote {
                row: 4,
                note: "E4".to_string(),
                inst: 0,
                vol: Some(64),
                ..Default::default()
            },
            PatternNote {
                row: 8,
                note: "G4".to_string(),
                inst: 0,
                vol: Some(64),
                ..Default::default()
            },
            PatternNote {
                row: 12,
                note: "OFF".to_string(),
                inst: 0,
                ..Default::default()
            },
        ],
    );
    let pattern = TrackerPattern {
        rows: 16,
        notes: Some(notes),
        data: None,
    };

    let mut patterns = HashMap::new();
    patterns.insert("intro".to_string(), pattern);

    MusicTrackerSongV1Params {
        format: TrackerFormat::Xm,
        bpm: 120,
        speed: 6,
        channels: 4,
        r#loop: true,
        instruments: vec![instrument],
        patterns,
        arrangement: vec![ArrangementEntry {
            pattern: "intro".to_string(),
            repeat: 1,
        }],
        ..Default::default()
    }
}

#[test]
fn test_generate_xm() {
    let params = create_test_params();
    let spec_dir = Path::new(".");
    let result = generate_music(&params, 42, spec_dir).unwrap();

    assert_eq!(result.extension, "xm");
    assert!(!result.data.is_empty());
    assert_eq!(result.hash.len(), 64);
}

#[test]
fn test_generate_it() {
    let mut params = create_test_params();
    params.format = TrackerFormat::It;

    let spec_dir = Path::new(".");
    let result = generate_music(&params, 42, spec_dir).unwrap();

    assert_eq!(result.extension, "it");
    assert!(!result.data.is_empty());
    assert_eq!(result.hash.len(), 64);
}

#[test]
fn test_determinism() {
    let params = create_test_params();
    let spec_dir = Path::new(".");

    let result1 = generate_music(&params, 42, spec_dir).unwrap();
    let result2 = generate_music(&params, 42, spec_dir).unwrap();

    assert_eq!(result1.hash, result2.hash);
    assert_eq!(result1.data, result2.data);
}

#[test]
fn test_different_seeds_different_output() {
    // Use noise synthesis which uses the seed
    let mut params = create_test_params();
    params.instruments[0].synthesis = Some(InstrumentSynthesis::Noise { periodic: false });

    let spec_dir = Path::new(".");
    let result1 = generate_music(&params, 42, spec_dir).unwrap();
    let result2 = generate_music(&params, 43, spec_dir).unwrap();

    // Different seeds should produce different hashes (due to noise synthesis)
    assert_ne!(result1.hash, result2.hash);
}

#[test]
fn test_different_seeds_same_output_for_pure_oscillator() {
    // Oscillators are deterministic and should not use RNG; seeds should not affect output.
    let params = create_test_params();
    let spec_dir = Path::new(".");

    let result1 = generate_music(&params, 42, spec_dir).unwrap();
    let result2 = generate_music(&params, 43, spec_dir).unwrap();

    assert_eq!(result1.hash, result2.hash);
    assert_eq!(result1.data, result2.data);
}

#[test]
fn test_invalid_channels() {
    let mut params = create_test_params();
    params.channels = 100; // Invalid for XM (max 32)

    let spec_dir = Path::new(".");
    let result = generate_music(&params, 42, spec_dir);
    assert!(result.is_err());
}

// =========================================================================
// External instrument reference tests
// =========================================================================

#[test]
fn test_external_ref_file_not_found() {
    // Test that missing external reference returns a clear error
    let instr = TrackerInstrument {
        name: "Missing".to_string(),
        r#ref: Some("nonexistent/file.json".to_string()),
        ..Default::default()
    };

    let err = bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap_err();
    assert!(err
        .to_string()
        .contains("Failed to read external instrument spec"));
}

#[test]
fn test_bake_instrument_sample_rejects_multiple_sources() {
    let instr = TrackerInstrument {
        name: "Bad".to_string(),
        wav: Some("samples/kick.wav".to_string()),
        synthesis: Some(InstrumentSynthesis::Sine),
        ..Default::default()
    };

    let err = bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap_err();
    assert!(err.to_string().contains("exactly one of"));
}

#[test]
fn test_bake_instrument_sample_from_ref_supports_advanced_audio_v1() {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../specs/music");

    let instr = TrackerInstrument {
        name: "FM Ref".to_string(),
        r#ref: Some("../audio/audio_instrument_fm_advanced.spec.json".to_string()),
        // Sustain > 0 => loopable sample.
        envelope: Envelope {
            attack: 0.01,
            decay: 0.05,
            sustain: 0.7,
            release: 0.2,
        },
        ..Default::default()
    };

    let (baked, _) = bake_instrument_sample(&instr, 42, 0, &spec_dir, TrackerFormat::Xm).unwrap();
    assert!(!baked.pcm16_mono.is_empty());
    assert!(
        baked.loop_region.is_some(),
        "sustained instruments should loop"
    );
}

#[test]
fn test_bake_instrument_sample_inline_audio_v1_downmixes_stereo() {
    let instr = TrackerInstrument {
        name: "Stereo Inline".to_string(),
        synthesis_audio_v1: Some(AudioV1Params {
            base_note: Some(AudioNoteSpec::NoteName("A4".to_string())),
            duration_seconds: 0.1,
            sample_rate: 22050,
            layers: vec![
                AudioLayer {
                    synthesis: AudioSynthesis::Oscillator {
                        waveform: Waveform::Sine,
                        frequency: 440.0,
                        freq_sweep: None,
                        detune: None,
                        duty: None,
                    },
                    envelope: Envelope::default(),
                    volume: 0.5,
                    pan: -1.0,
                    delay: None,
                    filter: None,
                    lfo: None,
                },
                AudioLayer {
                    synthesis: AudioSynthesis::Oscillator {
                        waveform: Waveform::Sine,
                        frequency: 440.0,
                        freq_sweep: None,
                        detune: None,
                        duty: None,
                    },
                    envelope: Envelope::default(),
                    volume: 0.5,
                    pan: 1.0,
                    delay: None,
                    filter: None,
                    lfo: None,
                },
            ],
            pitch_envelope: None,
            loop_config: None,
            generate_loop_points: false,
            master_filter: None,
            effects: vec![],
            post_fx_lfos: vec![],
        }),
        envelope: Envelope {
            attack: 0.01,
            decay: 0.05,
            sustain: 0.0,
            release: 0.05,
        },
        ..Default::default()
    };

    let (baked, _) =
        bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
    assert!(!baked.pcm16_mono.is_empty());

    // Must be mono 16-bit PCM: duration 0.1s @ 22050 Hz => 2205 samples => 4410 bytes.
    assert_eq!(baked.pcm16_mono.len(), 2205 * 2);
}

#[test]
fn test_loop_policy_uses_tracker_envelope_sustain() {
    let audio = AudioV1Params {
        base_note: Some(AudioNoteSpec::NoteName("A4".to_string())),
        duration_seconds: 0.2,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: AudioSynthesis::Oscillator {
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
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        loop_config: None,
        // Intentionally opposite of the tracker envelope tests below.
        generate_loop_points: true,
        master_filter: None,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let one_shot = TrackerInstrument {
        name: "One Shot".to_string(),
        synthesis_audio_v1: Some(audio.clone()),
        envelope: Envelope {
            attack: 0.01,
            decay: 0.05,
            sustain: 0.0,
            release: 0.05,
        },
        ..Default::default()
    };
    let (baked, _) =
        bake_instrument_sample(&one_shot, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
    assert!(baked.loop_region.is_none());

    let sustained = TrackerInstrument {
        name: "Sustain".to_string(),
        synthesis_audio_v1: Some(AudioV1Params {
            loop_config: None,
            generate_loop_points: false,
            ..audio
        }),
        envelope: Envelope {
            attack: 0.01,
            decay: 0.05,
            sustain: 0.7,
            release: 0.2,
        },
        ..Default::default()
    };
    let (baked, _) =
        bake_instrument_sample(&sustained, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
    assert!(baked.loop_region.is_some());
}

#[test]
fn test_audio_v1_base_note_midi_note_is_used_for_pitch_mapping() {
    let instr = TrackerInstrument {
        name: "Midi Base".to_string(),
        synthesis_audio_v1: Some(AudioV1Params {
            base_note: Some(AudioNoteSpec::MidiNote(69)),
            duration_seconds: 0.1,
            sample_rate: 22050,
            layers: vec![AudioLayer {
                synthesis: AudioSynthesis::Oscillator {
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
                filter: None,
                lfo: None,
            }],
            pitch_envelope: None,
            loop_config: None,
            generate_loop_points: false,
            master_filter: None,
            effects: vec![],
            post_fx_lfos: vec![],
        }),
        envelope: Envelope {
            attack: 0.01,
            decay: 0.05,
            sustain: 0.0,
            release: 0.05,
        },
        ..Default::default()
    };

    let (baked, _) =
        bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
    assert_eq!(baked.base_midi, 69);
}

#[test]
fn test_sustained_sine_prefers_forward_loop_with_crossfade() {
    let instr = TrackerInstrument {
        name: "Sine Loop".to_string(),
        synthesis_audio_v1: Some(AudioV1Params {
            base_note: Some(AudioNoteSpec::NoteName("A4".to_string())),
            duration_seconds: 1.0,
            sample_rate: 44100,
            layers: vec![AudioLayer {
                synthesis: AudioSynthesis::Oscillator {
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
                filter: None,
                lfo: None,
            }],
            pitch_envelope: None,
            loop_config: None,
            generate_loop_points: false,
            master_filter: None,
            effects: vec![],
            post_fx_lfos: vec![],
        }),
        envelope: Envelope {
            attack: 0.05,
            decay: 0.2,
            sustain: 0.7,
            release: 0.2,
        },
        ..Default::default()
    };

    let (baked, _) =
        bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
    let loop_region = baked
        .loop_region
        .expect("sustained instruments should loop");
    assert_eq!(loop_region.mode, LoopMode::Forward);
    assert!(loop_region.end > loop_region.start);
}

#[test]
fn test_sustained_noise_falls_back_to_pingpong_loop() {
    let instr = TrackerInstrument {
        name: "Noise Loop".to_string(),
        synthesis_audio_v1: Some(AudioV1Params {
            base_note: Some(AudioNoteSpec::NoteName("A4".to_string())),
            duration_seconds: 1.0,
            sample_rate: 44100,
            layers: vec![AudioLayer {
                synthesis: AudioSynthesis::NoiseBurst {
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
            loop_config: None,
            generate_loop_points: false,
            master_filter: None,
            effects: vec![],
            post_fx_lfos: vec![],
        }),
        envelope: Envelope {
            attack: 0.05,
            decay: 0.2,
            sustain: 0.7,
            release: 0.2,
        },
        ..Default::default()
    };

    let (baked, _) =
        bake_instrument_sample(&instr, 42, 0, Path::new("."), TrackerFormat::Xm).unwrap();
    let loop_region = baked
        .loop_region
        .expect("sustained instruments should loop");
    assert_eq!(loop_region.mode, LoopMode::PingPong);
    assert!(loop_region.end > loop_region.start);
}
