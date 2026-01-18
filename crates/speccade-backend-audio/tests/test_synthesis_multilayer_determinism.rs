//! Multi-layer and determinism integration tests.

use speccade_backend_audio::generate::generate_from_params;
use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params as AudioSfxLayeredSynthV1Params, Envelope, Filter, NoiseType,
    Synthesis, Waveform,
};

// ============================================================================
// Multi-Layer Tests
// ============================================================================

#[test]
fn test_multi_layer_with_delay() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
        sample_rate: 22050,
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
                volume: 0.8,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            },
            AudioLayer {
                synthesis: Synthesis::NoiseBurst {
                    noise_type: NoiseType::White,
                    filter: Some(Filter::Lowpass {
                        cutoff: 3000.0,
                        resonance: 0.707,
                        cutoff_end: None,
                    }),
                },
                envelope: Envelope {
                    attack: 0.001,
                    decay: 0.05,
                    sustain: 0.0,
                    release: 0.1,
                },
                volume: 0.3,
                pan: 0.0,
                delay: Some(0.05),
                filter: None,
                lfo: None,
            },
        ],
        master_filter: None,
        effects: vec![],
        pitch_envelope: None,
        base_note: None,
        loop_config: None,
        generate_loop_points: false,
        post_fx_lfos: vec![],
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_multi_layer_with_panning() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
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
                volume: 0.8,
                pan: -0.5, // Left
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
                volume: 0.8,
                pan: 0.5, // Right
                delay: None,
                filter: None,
                lfo: None,
            },
        ],
        master_filter: None,
        effects: vec![],
        pitch_envelope: None,
        base_note: None,
        loop_config: None,
        generate_loop_points: false,
        post_fx_lfos: vec![],
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

// ============================================================================
// Determinism Tests
// ============================================================================

#[test]
fn test_determinism_noise() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.3,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::NoiseBurst {
                noise_type: NoiseType::White,
                filter: None,
            },
            envelope: Envelope::default(),
            volume: 0.6,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
        pitch_envelope: None,
        base_note: None,
        loop_config: None,
        generate_loop_points: false,
        post_fx_lfos: vec![],
    };

    let result1 = generate_from_params(&params, 42).expect("first generation");
    let result2 = generate_from_params(&params, 42).expect("second generation");

    // Should produce identical output with same seed
    assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);
}

#[test]
fn test_determinism_karplus() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::KarplusStrong {
                frequency: 440.0,
                decay: 0.996,
                blend: 0.7,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
        pitch_envelope: None,
        base_note: None,
        loop_config: None,
        generate_loop_points: false,
        post_fx_lfos: vec![],
    };

    let result1 = generate_from_params(&params, 42).expect("first generation");
    let result2 = generate_from_params(&params, 42).expect("second generation");

    // Should produce identical output with same seed
    assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);
}
