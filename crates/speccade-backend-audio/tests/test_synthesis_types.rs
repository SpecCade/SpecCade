//! Integration tests covering all synthesis types and filter configurations.

use speccade_backend_audio::generate::generate_from_params;
use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params as AudioSfxLayeredSynthV1Params, Envelope, Filter, FreqSweep,
    NoiseType, OscillatorConfig, SweepCurve, Synthesis, Waveform,
};

// ============================================================================
// Oscillator Synthesis Tests
// ============================================================================

#[test]
fn test_oscillator_sine() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(!result.wav.wav_data.is_empty());
}

#[test]
fn test_oscillator_square() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Square,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_oscillator_sawtooth() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sawtooth,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_oscillator_triangle() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Triangle,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_oscillator_pulse_with_duty() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Pulse,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: Some(0.25),
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_oscillator_with_detune() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0,
                freq_sweep: None,
                detune: Some(50.0), // 50 cents up
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_oscillator_with_freq_sweep_linear() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 880.0,
                freq_sweep: Some(FreqSweep {
                    end_freq: 220.0,
                    curve: SweepCurve::Linear,
                }),
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_oscillator_with_freq_sweep_exponential() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 880.0,
                freq_sweep: Some(FreqSweep {
                    end_freq: 220.0,
                    curve: SweepCurve::Exponential,
                }),
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_oscillator_with_freq_sweep_logarithmic() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 880.0,
                freq_sweep: Some(FreqSweep {
                    end_freq: 220.0,
                    curve: SweepCurve::Logarithmic,
                }),
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

// ============================================================================
// FM Synthesis Tests
// ============================================================================

#[test]
fn test_fm_synth_basic() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::FmSynth {
                carrier_freq: 440.0,
                modulator_freq: 880.0,
                modulation_index: 2.0,
                freq_sweep: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_fm_synth_with_sweep() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::FmSynth {
                carrier_freq: 440.0,
                modulator_freq: 880.0,
                modulation_index: 3.0,
                freq_sweep: Some(FreqSweep {
                    end_freq: 110.0,
                    curve: SweepCurve::Exponential,
                }),
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

// ============================================================================
// Karplus-Strong Synthesis Tests
// ============================================================================

#[test]
fn test_karplus_strong() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_karplus_strong_high_damping() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::KarplusStrong {
                frequency: 220.0,
                decay: 0.99,
                blend: 0.9,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

// ============================================================================
// Additive Synthesis Tests
// ============================================================================

#[test]
fn test_additive_synthesis() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Additive {
                base_freq: 440.0,
                harmonics: vec![1.0, 0.5, 0.25, 0.125, 0.0625],
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_additive_synthesis_single_harmonic() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Additive {
                base_freq: 440.0,
                harmonics: vec![1.0],
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
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
                oscillators: vec![
                    OscillatorConfig {
                        waveform: Waveform::Sawtooth,
                        volume: 1.0,
                        detune: None,
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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

// ============================================================================
// Pitched Body Synthesis Tests
// ============================================================================

#[test]
fn test_pitched_body() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::PitchedBody {
                start_freq: 200.0,
                end_freq: 50.0,
            },
            envelope: Envelope {
                attack: 0.001,
                decay: 0.5,
                sustain: 0.0,
                release: 0.5,
            },
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

// ============================================================================
// Metallic Synthesis Tests
// ============================================================================

#[test]
fn test_metallic_synthesis() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Metallic {
                base_freq: 800.0,
                num_partials: 6,
                inharmonicity: 1.414,
            },
            envelope: Envelope {
                attack: 0.001,
                decay: 0.3,
                sustain: 0.1,
                release: 0.5,
            },
            volume: 0.6,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_metallic_synthesis_many_partials() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Metallic {
                base_freq: 500.0,
                num_partials: 12,
                inharmonicity: 2.0,
            },
            envelope: Envelope {
                attack: 0.001,
                decay: 0.4,
                sustain: 0.0,
                release: 0.6,
            },
            volume: 0.5,
            pan: 0.0,
            delay: None,
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

// ============================================================================
// Filter Tests
// ============================================================================

#[test]
fn test_filter_lowpass() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sawtooth,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: Some(Filter::Lowpass {
            cutoff: 2000.0,
            resonance: 0.707,
            cutoff_end: None,
        }),
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_filter_lowpass_with_sweep() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sawtooth,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 0.8,
            pan: 0.0,
            delay: None,
        }],
        master_filter: Some(Filter::Lowpass {
            cutoff: 5000.0,
            resonance: 1.0,
            cutoff_end: Some(500.0),
        }),
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_filter_highpass() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
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
        }],
        master_filter: Some(Filter::Highpass {
            cutoff: 1000.0,
            resonance: 0.5,
            cutoff_end: None,
        }),
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_filter_highpass_with_sweep() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
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
        }],
        master_filter: Some(Filter::Highpass {
            cutoff: 100.0,
            resonance: 0.8,
            cutoff_end: Some(3000.0),
        }),
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_filter_bandpass() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 0.5,
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
        }],
        master_filter: Some(Filter::Bandpass {
            center: 1000.0,
            bandwidth: 500.0,
            resonance: 0.707,
            center_end: None,
        }),
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

#[test]
fn test_filter_bandpass_with_sweep() {
    let params = AudioSfxLayeredSynthV1Params {
        duration_seconds: 1.0,
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
        }],
        master_filter: Some(Filter::Bandpass {
            center: 2000.0,
            bandwidth: 300.0,
            resonance: 1.2,
            center_end: Some(500.0),
        }),
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}

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
            },
        ],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
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
            },
        ],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
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
        }],
        master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
    };

    let result1 = generate_from_params(&params, 42).expect("first generation");
    let result2 = generate_from_params(&params, 42).expect("second generation");

    // Should produce identical output with same seed
    assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);
}
