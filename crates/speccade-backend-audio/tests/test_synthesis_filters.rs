//! Filter configuration integration tests.

use speccade_backend_audio::generate::generate_from_params;
use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params as AudioSfxLayeredSynthV1Params, Envelope, Filter, NoiseType,
    Synthesis, Waveform,
};

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
            filter: None,
            lfo: None,
        }],
        master_filter: Some(Filter::Lowpass {
            cutoff: 2000.0,
            resonance: 0.707,
            cutoff_end: None,
        }),
        effects: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: Some(Filter::Lowpass {
            cutoff: 5000.0,
            resonance: 1.0,
            cutoff_end: Some(500.0),
        }),
        effects: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: Some(Filter::Highpass {
            cutoff: 1000.0,
            resonance: 0.5,
            cutoff_end: None,
        }),
        effects: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: Some(Filter::Highpass {
            cutoff: 100.0,
            resonance: 0.8,
            cutoff_end: Some(3000.0),
        }),
        effects: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: Some(Filter::Bandpass {
            center: 1000.0,
            resonance: 0.707,
            center_end: None,
        }),
        effects: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: Some(Filter::Bandpass {
            center: 2000.0,
            resonance: 1.2,
            center_end: Some(500.0),
        }),
        effects: vec![],
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}
