//! Pitched Body and Metallic synthesis integration tests.

use speccade_backend_audio::generate::generate_from_params;
use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params as AudioSfxLayeredSynthV1Params, Envelope, Synthesis,
};

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
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
        post_fx_lfos: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
        post_fx_lfos: vec![],
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
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
        pitch_envelope: None,
        base_note: None,
        generate_loop_points: false,
        post_fx_lfos: vec![],
    };

    let result = generate_from_params(&params, 42);
    assert!(result.is_ok());
}
