//! Oscillator synthesis integration tests.

use speccade_backend_audio::generate::generate_from_params;
use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params as AudioSfxLayeredSynthV1Params, Envelope, FreqSweep, SweepCurve,
    Synthesis, Waveform,
};

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
