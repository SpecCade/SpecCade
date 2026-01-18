//! Noise and Multi-Oscillator synthesis integration tests.

use speccade_backend_audio::generate::generate_from_params;
use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params as AudioSfxLayeredSynthV1Params, Envelope, Filter, NoiseType,
    OscillatorConfig, Synthesis, Waveform,
};

// ============================================================================
// Noise Synthesis Tests
// ============================================================================

#[test]
fn test_noise_white() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::NoiseBurst {
                noise_type: NoiseType::White,
                filter: None,
            },
            envelope: Envelope {
                attack: 0.001,
                decay: 0.05,
                sustain: 0.0,
                release: 0.1,
            },
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

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_noise_pink() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::NoiseBurst {
                noise_type: NoiseType::Pink,
                filter: None,
            },
            envelope: Envelope {
                attack: 0.001,
                decay: 0.05,
                sustain: 0.0,
                release: 0.1,
            },
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

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_noise_brown() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::NoiseBurst {
                noise_type: NoiseType::Brown,
                filter: None,
            },
            envelope: Envelope {
                attack: 0.001,
                decay: 0.05,
                sustain: 0.0,
                release: 0.1,
            },
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

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_noise_with_filter() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::NoiseBurst {
                noise_type: NoiseType::White,
                filter: Some(Filter::Lowpass {
                    cutoff: 2000.0,
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

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

// ============================================================================
// Multi-Oscillator Synthesis Tests
// ============================================================================

#[test]
fn test_multi_oscillator_basic() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::MultiOscillator {
                frequency: 440.0,
                oscillators: vec![OscillatorConfig {
                    waveform: Waveform::Sawtooth,
                    volume: 1.0,
                    detune: None,
                    phase: None,
                    duty: None,
                }],
                freq_sweep: None,
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

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_multi_oscillator_detuned() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::MultiOscillator {
                frequency: 440.0,
                oscillators: vec![
                    OscillatorConfig {
                        waveform: Waveform::Sawtooth,
                        volume: 0.5,
                        detune: Some(0.0),
                        phase: None,
                        duty: None,
                    },
                    OscillatorConfig {
                        waveform: Waveform::Sawtooth,
                        volume: 0.5,
                        detune: Some(5.0),
                        phase: None,
                        duty: None,
                    },
                ],
                freq_sweep: None,
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

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_multi_oscillator_with_phase() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::MultiOscillator {
                frequency: 440.0,
                oscillators: vec![
                    OscillatorConfig {
                        waveform: Waveform::Sine,
                        volume: 0.5,
                        detune: None,
                        phase: Some(0.0),
                        duty: None,
                    },
                    OscillatorConfig {
                        waveform: Waveform::Sine,
                        volume: 0.5,
                        detune: None,
                        phase: Some(1.57), // PI/2
                        duty: None,
                    },
                ],
                freq_sweep: None,
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

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_multi_oscillator_with_duty() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::MultiOscillator {
                frequency: 440.0,
                oscillators: vec![
                    OscillatorConfig {
                        waveform: Waveform::Square,
                        volume: 0.5,
                        detune: None,
                        phase: None,
                        duty: Some(0.5),
                    },
                    OscillatorConfig {
                        waveform: Waveform::Square,
                        volume: 0.5,
                        detune: None,
                        phase: None,
                        duty: Some(0.25),
                    },
                ],
                freq_sweep: None,
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

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}
