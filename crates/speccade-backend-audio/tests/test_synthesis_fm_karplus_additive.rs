//! FM, Karplus-Strong, and Additive synthesis integration tests.

use speccade_backend_audio::generate::generate_from_params;
use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params as AudioSfxLayeredSynthV1Params, Envelope, FreqSweep, Synthesis,
    SweepCurve,
};

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
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}
