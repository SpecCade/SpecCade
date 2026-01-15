//! Tests for audio effect types.

use super::*;

#[test]
fn test_parametric_eq_serde_roundtrip() {
    let effect = Effect::ParametricEq {
        bands: vec![
            EqBand {
                frequency: 100.0,
                gain_db: 3.0,
                q: 1.0,
                band_type: EqBandType::Lowshelf,
            },
            EqBand {
                frequency: 1000.0,
                gain_db: -2.0,
                q: 2.0,
                band_type: EqBandType::Peak,
            },
            EqBand {
                frequency: 8000.0,
                gain_db: 2.0,
                q: 1.0,
                band_type: EqBandType::Highshelf,
            },
        ],
    };

    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("\"type\":\"parametric_eq\""));
    assert!(json.contains("\"band_type\":\"lowshelf\""));
    assert!(json.contains("\"band_type\":\"peak\""));
    assert!(json.contains("\"band_type\":\"highshelf\""));

    let parsed: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_parametric_eq_from_json() {
    let json = r#"{
        "type": "parametric_eq",
        "bands": [
            { "frequency": 200.0, "gain_db": -6.0, "q": 4.0, "band_type": "notch" }
        ]
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::ParametricEq { bands } => {
            assert_eq!(bands.len(), 1);
            assert_eq!(bands[0].frequency, 200.0);
            assert_eq!(bands[0].gain_db, -6.0);
            assert_eq!(bands[0].q, 4.0);
            assert_eq!(bands[0].band_type, EqBandType::Notch);
        }
        _ => panic!("Expected ParametricEq variant"),
    }
}

#[test]
fn test_eq_band_type_serde() {
    assert_eq!(
        serde_json::to_string(&EqBandType::Lowshelf).unwrap(),
        "\"lowshelf\""
    );
    assert_eq!(
        serde_json::to_string(&EqBandType::Highshelf).unwrap(),
        "\"highshelf\""
    );
    assert_eq!(
        serde_json::to_string(&EqBandType::Peak).unwrap(),
        "\"peak\""
    );
    assert_eq!(
        serde_json::to_string(&EqBandType::Notch).unwrap(),
        "\"notch\""
    );

    assert_eq!(
        serde_json::from_str::<EqBandType>("\"lowshelf\"").unwrap(),
        EqBandType::Lowshelf
    );
    assert_eq!(
        serde_json::from_str::<EqBandType>("\"highshelf\"").unwrap(),
        EqBandType::Highshelf
    );
    assert_eq!(
        serde_json::from_str::<EqBandType>("\"peak\"").unwrap(),
        EqBandType::Peak
    );
    assert_eq!(
        serde_json::from_str::<EqBandType>("\"notch\"").unwrap(),
        EqBandType::Notch
    );
}

#[test]
fn test_eq_band_deny_unknown_fields() {
    let json = r#"{
        "frequency": 1000.0,
        "gain_db": 3.0,
        "q": 2.0,
        "band_type": "peak",
        "unknown_field": true
    }"#;

    let result: Result<EqBand, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_limiter_serde_roundtrip() {
    let effect = Effect::Limiter {
        threshold_db: -6.0,
        release_ms: 100.0,
        lookahead_ms: 5.0,
        ceiling_db: -0.3,
    };

    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("\"type\":\"limiter\""));
    assert!(json.contains("\"threshold_db\":-6.0"));
    assert!(json.contains("\"release_ms\":100.0"));
    assert!(json.contains("\"lookahead_ms\":5.0"));
    assert!(json.contains("\"ceiling_db\":-0.3"));

    let parsed: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_limiter_from_json() {
    let json = r#"{
        "type": "limiter",
        "threshold_db": -12.0,
        "release_ms": 200.0,
        "lookahead_ms": 3.0,
        "ceiling_db": -1.0
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::Limiter {
            threshold_db,
            release_ms,
            lookahead_ms,
            ceiling_db,
        } => {
            assert_eq!(threshold_db, -12.0);
            assert_eq!(release_ms, 200.0);
            assert_eq!(lookahead_ms, 3.0);
            assert_eq!(ceiling_db, -1.0);
        }
        _ => panic!("Expected Limiter variant"),
    }
}

#[test]
fn test_gate_expander_serde_roundtrip() {
    let effect = Effect::GateExpander {
        threshold_db: -30.0,
        ratio: 4.0,
        attack_ms: 1.0,
        hold_ms: 50.0,
        release_ms: 100.0,
        range_db: -60.0,
    };

    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("\"type\":\"gate_expander\""));
    assert!(json.contains("\"threshold_db\":-30.0"));
    assert!(json.contains("\"ratio\":4.0"));
    assert!(json.contains("\"attack_ms\":1.0"));
    assert!(json.contains("\"hold_ms\":50.0"));
    assert!(json.contains("\"release_ms\":100.0"));
    assert!(json.contains("\"range_db\":-60.0"));

    let parsed: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_gate_expander_from_json() {
    let json = r#"{
        "type": "gate_expander",
        "threshold_db": -40.0,
        "ratio": 10.0,
        "attack_ms": 0.5,
        "hold_ms": 100.0,
        "release_ms": 200.0,
        "range_db": -80.0
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::GateExpander {
            threshold_db,
            ratio,
            attack_ms,
            hold_ms,
            release_ms,
            range_db,
        } => {
            assert_eq!(threshold_db, -40.0);
            assert_eq!(ratio, 10.0);
            assert_eq!(attack_ms, 0.5);
            assert_eq!(hold_ms, 100.0);
            assert_eq!(release_ms, 200.0);
            assert_eq!(range_db, -80.0);
        }
        _ => panic!("Expected GateExpander variant"),
    }
}

#[test]
fn test_stereo_widener_serde_roundtrip() {
    let effect = Effect::StereoWidener {
        width: 1.5,
        mode: StereoWidenerMode::MidSide,
        delay_ms: 15.0,
    };

    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("\"type\":\"stereo_widener\""));
    assert!(json.contains("\"width\":1.5"));
    assert!(json.contains("\"mode\":\"mid_side\""));
    assert!(json.contains("\"delay_ms\":15.0"));

    let parsed: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_stereo_widener_from_json_simple() {
    let json = r#"{
        "type": "stereo_widener",
        "width": 1.2
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::StereoWidener {
            width,
            mode,
            delay_ms,
        } => {
            assert_eq!(width, 1.2);
            assert_eq!(mode, StereoWidenerMode::Simple);
            assert_eq!(delay_ms, 10.0); // default
        }
        _ => panic!("Expected StereoWidener variant"),
    }
}

#[test]
fn test_stereo_widener_from_json_haas() {
    let json = r#"{
        "type": "stereo_widener",
        "width": 0.8,
        "mode": "haas",
        "delay_ms": 20.0
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::StereoWidener {
            width,
            mode,
            delay_ms,
        } => {
            assert_eq!(width, 0.8);
            assert_eq!(mode, StereoWidenerMode::Haas);
            assert_eq!(delay_ms, 20.0);
        }
        _ => panic!("Expected StereoWidener variant"),
    }
}

#[test]
fn test_stereo_widener_mode_serde() {
    assert_eq!(
        serde_json::to_string(&StereoWidenerMode::Simple).unwrap(),
        "\"simple\""
    );
    assert_eq!(
        serde_json::to_string(&StereoWidenerMode::Haas).unwrap(),
        "\"haas\""
    );
    assert_eq!(
        serde_json::to_string(&StereoWidenerMode::MidSide).unwrap(),
        "\"mid_side\""
    );

    assert_eq!(
        serde_json::from_str::<StereoWidenerMode>("\"simple\"").unwrap(),
        StereoWidenerMode::Simple
    );
    assert_eq!(
        serde_json::from_str::<StereoWidenerMode>("\"haas\"").unwrap(),
        StereoWidenerMode::Haas
    );
    assert_eq!(
        serde_json::from_str::<StereoWidenerMode>("\"mid_side\"").unwrap(),
        StereoWidenerMode::MidSide
    );
}

#[test]
fn test_multi_tap_delay_serde_roundtrip() {
    let effect = Effect::MultiTapDelay {
        taps: vec![
            DelayTap {
                time_ms: 100.0,
                feedback: 0.3,
                pan: -0.5,
                level: 0.8,
                filter_cutoff: 2000.0,
            },
            DelayTap {
                time_ms: 200.0,
                feedback: 0.2,
                pan: 0.5,
                level: 0.6,
                filter_cutoff: 0.0,
            },
        ],
    };

    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("\"type\":\"multi_tap_delay\""));
    assert!(json.contains("\"time_ms\":100.0"));
    assert!(json.contains("\"feedback\":0.3"));
    assert!(json.contains("\"pan\":-0.5"));
    assert!(json.contains("\"level\":0.8"));
    assert!(json.contains("\"filter_cutoff\":2000.0"));

    let parsed: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_multi_tap_delay_from_json() {
    let json = r#"{
        "type": "multi_tap_delay",
        "taps": [
            {
                "time_ms": 150.0,
                "feedback": 0.4,
                "pan": 0.0,
                "level": 1.0,
                "filter_cutoff": 4000.0
            }
        ]
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::MultiTapDelay { taps } => {
            assert_eq!(taps.len(), 1);
            assert_eq!(taps[0].time_ms, 150.0);
            assert_eq!(taps[0].feedback, 0.4);
            assert_eq!(taps[0].pan, 0.0);
            assert_eq!(taps[0].level, 1.0);
            assert_eq!(taps[0].filter_cutoff, 4000.0);
        }
        _ => panic!("Expected MultiTapDelay variant"),
    }
}

#[test]
fn test_delay_tap_filter_cutoff_default() {
    let json = r#"{
        "time_ms": 100.0,
        "feedback": 0.5,
        "pan": 0.0,
        "level": 0.7
    }"#;

    let tap: DelayTap = serde_json::from_str(json).unwrap();
    assert_eq!(tap.filter_cutoff, 0.0);
}

#[test]
fn test_delay_tap_deny_unknown_fields() {
    let json = r#"{
        "time_ms": 100.0,
        "feedback": 0.5,
        "pan": 0.0,
        "level": 0.7,
        "unknown_field": true
    }"#;

    let result: Result<DelayTap, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_tape_saturation_serde_roundtrip() {
    let effect = Effect::TapeSaturation {
        drive: 3.0,
        bias: 0.1,
        wow_rate: 1.0,
        flutter_rate: 8.0,
        hiss_level: 0.02,
    };

    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("\"type\":\"tape_saturation\""));
    assert!(json.contains("\"drive\":3.0"));
    assert!(json.contains("\"bias\":0.1"));
    assert!(json.contains("\"wow_rate\":1.0"));
    assert!(json.contains("\"flutter_rate\":8.0"));
    assert!(json.contains("\"hiss_level\":0.02"));

    let parsed: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_tape_saturation_from_json() {
    let json = r#"{
        "type": "tape_saturation",
        "drive": 5.0,
        "bias": -0.2,
        "wow_rate": 0.5,
        "flutter_rate": 12.0,
        "hiss_level": 0.05
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::TapeSaturation {
            drive,
            bias,
            wow_rate,
            flutter_rate,
            hiss_level,
        } => {
            assert_eq!(drive, 5.0);
            assert_eq!(bias, -0.2);
            assert_eq!(wow_rate, 0.5);
            assert_eq!(flutter_rate, 12.0);
            assert_eq!(hiss_level, 0.05);
        }
        _ => panic!("Expected TapeSaturation variant"),
    }
}

#[test]
fn test_transient_shaper_serde_roundtrip() {
    let effect = Effect::TransientShaper {
        attack: 0.5,
        sustain: -0.3,
        output_gain_db: 2.0,
    };

    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("\"type\":\"transient_shaper\""));
    assert!(json.contains("\"attack\":0.5"));
    assert!(json.contains("\"sustain\":-0.3"));
    assert!(json.contains("\"output_gain_db\":2.0"));

    let parsed: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_transient_shaper_from_json() {
    let json = r#"{
        "type": "transient_shaper",
        "attack": -0.7,
        "sustain": 0.4,
        "output_gain_db": -3.0
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::TransientShaper {
            attack,
            sustain,
            output_gain_db,
        } => {
            assert_eq!(attack, -0.7);
            assert_eq!(sustain, 0.4);
            assert_eq!(output_gain_db, -3.0);
        }
        _ => panic!("Expected TransientShaper variant"),
    }
}

#[test]
fn test_auto_filter_serde_roundtrip() {
    let effect = Effect::AutoFilter {
        sensitivity: 0.8,
        attack_ms: 5.0,
        release_ms: 100.0,
        depth: 0.7,
        base_frequency: 500.0,
    };

    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("\"type\":\"auto_filter\""));
    assert!(json.contains("\"sensitivity\":0.8"));
    assert!(json.contains("\"attack_ms\":5.0"));
    assert!(json.contains("\"release_ms\":100.0"));
    assert!(json.contains("\"depth\":0.7"));
    assert!(json.contains("\"base_frequency\":500.0"));

    let parsed: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_auto_filter_from_json() {
    let json = r#"{
        "type": "auto_filter",
        "sensitivity": 1.0,
        "attack_ms": 1.0,
        "release_ms": 50.0,
        "depth": 1.0,
        "base_frequency": 200.0
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::AutoFilter {
            sensitivity,
            attack_ms,
            release_ms,
            depth,
            base_frequency,
        } => {
            assert_eq!(sensitivity, 1.0);
            assert_eq!(attack_ms, 1.0);
            assert_eq!(release_ms, 50.0);
            assert_eq!(depth, 1.0);
            assert_eq!(base_frequency, 200.0);
        }
        _ => panic!("Expected AutoFilter variant"),
    }
}

#[test]
fn test_cabinet_sim_serde_roundtrip() {
    let effect = Effect::CabinetSim {
        cabinet_type: CabinetType::Guitar4x12,
        mic_position: 0.5,
    };

    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("\"type\":\"cabinet_sim\""));
    assert!(json.contains("\"cabinet_type\":\"guitar_4x12\""));
    assert!(json.contains("\"mic_position\":0.5"));

    let parsed: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_cabinet_sim_from_json_all_types() {
    let types = [
        ("guitar_1x12", CabinetType::Guitar1x12),
        ("guitar_4x12", CabinetType::Guitar4x12),
        ("bass_1x15", CabinetType::Bass1x15),
        ("radio", CabinetType::Radio),
        ("telephone", CabinetType::Telephone),
    ];

    for (type_str, expected_type) in types {
        let json = format!(
            r#"{{
                "type": "cabinet_sim",
                "cabinet_type": "{}",
                "mic_position": 0.3
            }}"#,
            type_str
        );

        let effect: Effect = serde_json::from_str(&json).unwrap();
        match effect {
            Effect::CabinetSim {
                cabinet_type,
                mic_position,
            } => {
                assert_eq!(cabinet_type, expected_type);
                assert_eq!(mic_position, 0.3);
            }
            _ => panic!("Expected CabinetSim variant"),
        }
    }
}

#[test]
fn test_cabinet_sim_mic_position_default() {
    let json = r#"{
        "type": "cabinet_sim",
        "cabinet_type": "guitar_1x12"
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::CabinetSim {
            cabinet_type,
            mic_position,
        } => {
            assert_eq!(cabinet_type, CabinetType::Guitar1x12);
            assert_eq!(mic_position, 0.0); // default
        }
        _ => panic!("Expected CabinetSim variant"),
    }
}

#[test]
fn test_cabinet_type_serde() {
    assert_eq!(
        serde_json::to_string(&CabinetType::Guitar1x12).unwrap(),
        "\"guitar_1x12\""
    );
    assert_eq!(
        serde_json::to_string(&CabinetType::Guitar4x12).unwrap(),
        "\"guitar_4x12\""
    );
    assert_eq!(
        serde_json::to_string(&CabinetType::Bass1x15).unwrap(),
        "\"bass_1x15\""
    );
    assert_eq!(
        serde_json::to_string(&CabinetType::Radio).unwrap(),
        "\"radio\""
    );
    assert_eq!(
        serde_json::to_string(&CabinetType::Telephone).unwrap(),
        "\"telephone\""
    );

    assert_eq!(
        serde_json::from_str::<CabinetType>("\"guitar_1x12\"").unwrap(),
        CabinetType::Guitar1x12
    );
    assert_eq!(
        serde_json::from_str::<CabinetType>("\"guitar_4x12\"").unwrap(),
        CabinetType::Guitar4x12
    );
    assert_eq!(
        serde_json::from_str::<CabinetType>("\"bass_1x15\"").unwrap(),
        CabinetType::Bass1x15
    );
    assert_eq!(
        serde_json::from_str::<CabinetType>("\"radio\"").unwrap(),
        CabinetType::Radio
    );
    assert_eq!(
        serde_json::from_str::<CabinetType>("\"telephone\"").unwrap(),
        CabinetType::Telephone
    );
}

#[test]
fn test_rotary_speaker_serde_roundtrip() {
    let effect = Effect::RotarySpeaker {
        rate: 5.0,
        depth: 0.7,
        wet: 0.6,
    };

    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("\"type\":\"rotary_speaker\""));
    assert!(json.contains("\"rate\":5.0"));
    assert!(json.contains("\"depth\":0.7"));
    assert!(json.contains("\"wet\":0.6"));

    let parsed: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_rotary_speaker_from_json() {
    let json = r#"{
        "type": "rotary_speaker",
        "rate": 1.0,
        "depth": 0.5,
        "wet": 0.8
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::RotarySpeaker { rate, depth, wet } => {
            assert_eq!(rate, 1.0);
            assert_eq!(depth, 0.5);
            assert_eq!(wet, 0.8);
        }
        _ => panic!("Expected RotarySpeaker variant"),
    }
}

#[test]
fn test_ring_modulator_serde_roundtrip() {
    let effect = Effect::RingModulator {
        frequency: 150.0,
        mix: 0.8,
    };

    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("\"type\":\"ring_modulator\""));
    assert!(json.contains("\"frequency\":150.0"));
    assert!(json.contains("\"mix\":0.8"));

    let parsed: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, effect);
}

#[test]
fn test_ring_modulator_from_json() {
    let json = r#"{
        "type": "ring_modulator",
        "frequency": 200.0,
        "mix": 0.5
    }"#;

    let effect: Effect = serde_json::from_str(json).unwrap();
    match effect {
        Effect::RingModulator { frequency, mix } => {
            assert_eq!(frequency, 200.0);
            assert_eq!(mix, 0.5);
        }
        _ => panic!("Expected RingModulator variant"),
    }
}
