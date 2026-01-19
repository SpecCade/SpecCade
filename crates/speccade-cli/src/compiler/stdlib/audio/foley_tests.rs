//! Tests for foley sound effect helpers

use crate::compiler::stdlib::tests::eval_to_json;

// ========================================================================
// impact_builder Tests
// ========================================================================

#[test]
fn test_impact_builder_basic() {
    let result = eval_to_json(
        r#"impact_builder(
            transient = {
                "synthesis": noise_burst("white"),
                "envelope": oneshot_envelope(1, 20, 0.0),
                "volume": 0.7,
                "filter": highpass(2000, 1.5)
            },
            body = {
                "synthesis": oscillator(100, "sine"),
                "envelope": oneshot_envelope(5, 80, 0.1),
                "volume": 0.9,
                "filter": lowpass(500, 1.0)
            },
            tail = {
                "synthesis": noise_burst("brown"),
                "envelope": oneshot_envelope(10, 300, 0.0),
                "volume": 0.4
            }
        )"#,
    )
    .unwrap();

    // Should return a list of 3 layers
    assert!(result.is_array());
    let layers = result.as_array().unwrap();
    assert_eq!(layers.len(), 3);

    // Verify transient layer
    let transient = &layers[0];
    assert_eq!(transient["synthesis"]["type"], "noise_burst");
    assert_eq!(transient["synthesis"]["noise_type"], "white");
    assert_eq!(transient["volume"], 0.7);
    assert_eq!(transient["filter"]["type"], "highpass");
    assert_eq!(transient["filter"]["cutoff"], 2000.0);

    // Verify body layer
    let body = &layers[1];
    assert_eq!(body["synthesis"]["type"], "oscillator");
    assert_eq!(body["synthesis"]["frequency"], 100.0);
    assert_eq!(body["volume"], 0.9);
    assert_eq!(body["filter"]["type"], "lowpass");

    // Verify tail layer
    let tail = &layers[2];
    assert_eq!(tail["synthesis"]["type"], "noise_burst");
    assert_eq!(tail["synthesis"]["noise_type"], "brown");
    assert_eq!(tail["volume"], 0.4);
    // Tail has no filter
    assert!(tail.get("filter").is_none());
}

#[test]
fn test_impact_builder_without_filters() {
    let result = eval_to_json(
        r#"impact_builder(
            transient = {
                "synthesis": noise_burst("white"),
                "envelope": oneshot_envelope(1, 10, 0.0),
                "volume": 0.5
            },
            body = {
                "synthesis": oscillator(80, "sine"),
                "envelope": oneshot_envelope(3, 50, 0.0),
                "volume": 0.8
            },
            tail = {
                "synthesis": noise_burst("pink"),
                "envelope": oneshot_envelope(5, 200, 0.0),
                "volume": 0.3
            }
        )"#,
    )
    .unwrap();

    let layers = result.as_array().unwrap();
    assert_eq!(layers.len(), 3);

    // All layers should have no filter
    for layer in layers {
        assert!(layer.get("filter").is_none());
    }
}

#[test]
fn test_impact_builder_fm_synthesis() {
    // Test using FM synthesis for body layer (metallic impact)
    let result = eval_to_json(
        r#"impact_builder(
            transient = {
                "synthesis": noise_burst("white"),
                "envelope": oneshot_envelope(0, 5, 0.0),
                "volume": 0.6
            },
            body = {
                "synthesis": fm_synth(440, 880, 3.0),
                "envelope": oneshot_envelope(2, 100, 0.2),
                "volume": 0.7,
                "filter": bandpass(1000, 2.0)
            },
            tail = {
                "synthesis": noise_burst("brown"),
                "envelope": oneshot_envelope(10, 500, 0.0),
                "volume": 0.25
            }
        )"#,
    )
    .unwrap();

    let layers = result.as_array().unwrap();
    let body = &layers[1];
    assert_eq!(body["synthesis"]["type"], "fm_synth");
    assert_eq!(body["synthesis"]["carrier_freq"], 440.0);
    assert_eq!(body["synthesis"]["modulator_freq"], 880.0);
}

#[test]
fn test_impact_builder_missing_synthesis_fails() {
    let result = eval_to_json(
        r#"impact_builder(
            transient = {
                "envelope": oneshot_envelope(1, 20, 0.0),
                "volume": 0.7
            },
            body = {
                "synthesis": oscillator(100),
                "envelope": oneshot_envelope(5, 80, 0.1),
                "volume": 0.9
            },
            tail = {
                "synthesis": noise_burst("brown"),
                "envelope": oneshot_envelope(10, 300, 0.0),
                "volume": 0.4
            }
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S101"));
    assert!(err.contains("synthesis"));
}

#[test]
fn test_impact_builder_missing_envelope_fails() {
    let result = eval_to_json(
        r#"impact_builder(
            transient = {
                "synthesis": noise_burst("white"),
                "volume": 0.7
            },
            body = {
                "synthesis": oscillator(100),
                "envelope": oneshot_envelope(5, 80, 0.1),
                "volume": 0.9
            },
            tail = {
                "synthesis": noise_burst("brown"),
                "envelope": oneshot_envelope(10, 300, 0.0),
                "volume": 0.4
            }
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S101"));
    assert!(err.contains("envelope"));
}

#[test]
fn test_impact_builder_missing_volume_fails() {
    let result = eval_to_json(
        r#"impact_builder(
            transient = {
                "synthesis": noise_burst("white"),
                "envelope": oneshot_envelope(1, 20, 0.0)
            },
            body = {
                "synthesis": oscillator(100),
                "envelope": oneshot_envelope(5, 80, 0.1),
                "volume": 0.9
            },
            tail = {
                "synthesis": noise_burst("brown"),
                "envelope": oneshot_envelope(10, 300, 0.0),
                "volume": 0.4
            }
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S101"));
    assert!(err.contains("volume"));
}

#[test]
fn test_impact_builder_invalid_volume_fails() {
    let result = eval_to_json(
        r#"impact_builder(
            transient = {
                "synthesis": noise_burst("white"),
                "envelope": oneshot_envelope(1, 20, 0.0),
                "volume": 1.5
            },
            body = {
                "synthesis": oscillator(100),
                "envelope": oneshot_envelope(5, 80, 0.1),
                "volume": 0.9
            },
            tail = {
                "synthesis": noise_burst("brown"),
                "envelope": oneshot_envelope(10, 300, 0.0),
                "volume": 0.4
            }
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("volume"));
}

#[test]
fn test_impact_builder_invalid_transient_type_fails() {
    let result = eval_to_json(
        r#"impact_builder(
            transient = "not a dict",
            body = {
                "synthesis": oscillator(100),
                "envelope": oneshot_envelope(5, 80, 0.1),
                "volume": 0.9
            },
            tail = {
                "synthesis": noise_burst("brown"),
                "envelope": oneshot_envelope(10, 300, 0.0),
                "volume": 0.4
            }
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S102"));
    assert!(err.contains("transient"));
}

// ========================================================================
// whoosh_builder Tests
// ========================================================================

#[test]
fn test_whoosh_builder_rising() {
    let result = eval_to_json(
        r#"whoosh_builder(
            direction = "rising",
            duration_ms = 200,
            start_freq = 400,
            end_freq = 2000
        )"#,
    )
    .unwrap();

    // Should return a single layer dict
    assert!(result.is_object());

    // Check synthesis
    assert_eq!(result["synthesis"]["type"], "noise_burst");
    assert_eq!(result["synthesis"]["noise_type"], "white"); // default

    // Check filter is bandpass with sweep
    assert_eq!(result["filter"]["type"], "bandpass");
    assert_eq!(result["filter"]["center"], 400.0);
    assert_eq!(result["filter"]["center_end"], 2000.0);
    assert_eq!(result["filter"]["resonance"], 2.0);

    // Check envelope timing (200ms total)
    // Attack = 30% = 60ms = 0.06s
    // Decay = 10% = 20ms = 0.02s
    // Release = 60% = 120ms = 0.12s
    assert!((result["envelope"]["attack"].as_f64().unwrap() - 0.06).abs() < 0.001);
    assert!((result["envelope"]["decay"].as_f64().unwrap() - 0.02).abs() < 0.001);
    assert!((result["envelope"]["release"].as_f64().unwrap() - 0.12).abs() < 0.001);
    assert_eq!(result["envelope"]["sustain"], 0.8);

    // Check volume and pan defaults
    assert_eq!(result["volume"], 0.8);
    assert_eq!(result["pan"], 0.0);
}

#[test]
fn test_whoosh_builder_falling() {
    let result = eval_to_json(
        r#"whoosh_builder(
            direction = "falling",
            duration_ms = 300,
            start_freq = 2000,
            end_freq = 300
        )"#,
    )
    .unwrap();

    // Check filter sweeps from high to low
    assert_eq!(result["filter"]["center"], 2000.0);
    assert_eq!(result["filter"]["center_end"], 300.0);
}

#[test]
fn test_whoosh_builder_pink_noise() {
    let result = eval_to_json(
        r#"whoosh_builder(
            direction = "rising",
            duration_ms = 150,
            start_freq = 500,
            end_freq = 1500,
            noise_type = "pink"
        )"#,
    )
    .unwrap();

    assert_eq!(result["synthesis"]["noise_type"], "pink");
}

#[test]
fn test_whoosh_builder_brown_noise() {
    let result = eval_to_json(
        r#"whoosh_builder(
            direction = "falling",
            duration_ms = 500,
            start_freq = 1000,
            end_freq = 100,
            noise_type = "brown"
        )"#,
    )
    .unwrap();

    assert_eq!(result["synthesis"]["noise_type"], "brown");
}

#[test]
fn test_whoosh_builder_short_duration() {
    // Very short UI whoosh
    let result = eval_to_json(
        r#"whoosh_builder(
            direction = "rising",
            duration_ms = 50,
            start_freq = 1000,
            end_freq = 3000
        )"#,
    )
    .unwrap();

    // 50ms duration
    // Attack = 15ms = 0.015s
    assert!((result["envelope"]["attack"].as_f64().unwrap() - 0.015).abs() < 0.001);
}

#[test]
fn test_whoosh_builder_long_duration() {
    // Long wind gust
    let result = eval_to_json(
        r#"whoosh_builder(
            direction = "falling",
            duration_ms = 2000,
            start_freq = 800,
            end_freq = 200
        )"#,
    )
    .unwrap();

    // 2000ms duration
    // Attack = 600ms = 0.6s
    // Release = 1200ms = 1.2s
    assert!((result["envelope"]["attack"].as_f64().unwrap() - 0.6).abs() < 0.001);
    assert!((result["envelope"]["release"].as_f64().unwrap() - 1.2).abs() < 0.001);
}

#[test]
fn test_whoosh_builder_invalid_direction_fails() {
    let result = eval_to_json(
        r#"whoosh_builder(
            direction = "sideways",
            duration_ms = 200,
            start_freq = 400,
            end_freq = 2000
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S104"));
    assert!(err.contains("direction"));
}

#[test]
fn test_whoosh_builder_invalid_noise_type_fails() {
    let result = eval_to_json(
        r#"whoosh_builder(
            direction = "rising",
            duration_ms = 200,
            start_freq = 400,
            end_freq = 2000,
            noise_type = "blue"
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S104"));
    assert!(err.contains("noise_type"));
}

#[test]
fn test_whoosh_builder_negative_duration_fails() {
    let result = eval_to_json(
        r#"whoosh_builder(
            direction = "rising",
            duration_ms = -100,
            start_freq = 400,
            end_freq = 2000
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("duration_ms"));
}

#[test]
fn test_whoosh_builder_negative_start_freq_fails() {
    let result = eval_to_json(
        r#"whoosh_builder(
            direction = "rising",
            duration_ms = 200,
            start_freq = -400,
            end_freq = 2000
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("start_freq"));
}

#[test]
fn test_whoosh_builder_negative_end_freq_fails() {
    let result = eval_to_json(
        r#"whoosh_builder(
            direction = "rising",
            duration_ms = 200,
            start_freq = 400,
            end_freq = -2000
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("end_freq"));
}

// ========================================================================
// Integration Tests
// ========================================================================

#[test]
fn test_impact_in_audio_spec() {
    let result = eval_to_json(
        r#"audio_spec(
            asset_id = "impact-test",
            seed = 42,
            output_path = "sfx/impact.wav",
            format = "wav",
            duration_seconds = 0.5,
            sample_rate = 44100,
            layers = impact_builder(
                transient = {
                    "synthesis": noise_burst("white"),
                    "envelope": oneshot_envelope(1, 15, 0.0),
                    "volume": 0.6,
                    "filter": highpass(3000)
                },
                body = {
                    "synthesis": oscillator(80, "sine"),
                    "envelope": oneshot_envelope(3, 60, 0.0),
                    "volume": 0.8
                },
                tail = {
                    "synthesis": noise_burst("brown"),
                    "envelope": oneshot_envelope(20, 300, 0.0),
                    "volume": 0.3
                }
            )
        )"#,
    )
    .unwrap();

    assert_eq!(result["asset_type"], "audio");
    assert_eq!(result["recipe"]["kind"], "audio_v1");

    let layers = result["recipe"]["params"]["layers"].as_array().unwrap();
    assert_eq!(layers.len(), 3);
}

#[test]
fn test_whoosh_in_audio_spec() {
    let result = eval_to_json(
        r#"audio_spec(
            asset_id = "whoosh-test",
            seed = 42,
            output_path = "sfx/whoosh.wav",
            format = "wav",
            duration_seconds = 0.3,
            sample_rate = 44100,
            layers = [
                whoosh_builder(
                    direction = "rising",
                    duration_ms = 200,
                    start_freq = 400,
                    end_freq = 2000
                )
            ]
        )"#,
    )
    .unwrap();

    assert_eq!(result["asset_type"], "audio");
    assert_eq!(result["recipe"]["kind"], "audio_v1");

    let layers = result["recipe"]["params"]["layers"].as_array().unwrap();
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0]["filter"]["type"], "bandpass");
}

#[test]
fn test_combined_impact_and_whoosh() {
    // Create a layered sound with both impact and whoosh elements
    let result = eval_to_json(
        r#"audio_spec(
            asset_id = "swing-impact",
            seed = 123,
            output_path = "sfx/swing_impact.wav",
            format = "wav",
            duration_seconds = 0.6,
            sample_rate = 44100,
            layers = impact_builder(
                transient = {
                    "synthesis": noise_burst("white"),
                    "envelope": oneshot_envelope(0, 10, 0.0),
                    "volume": 0.5,
                    "filter": highpass(4000)
                },
                body = {
                    "synthesis": oscillator(60, "sine", 40),
                    "envelope": oneshot_envelope(5, 100, 0.0),
                    "volume": 0.7
                },
                tail = {
                    "synthesis": noise_burst("brown"),
                    "envelope": oneshot_envelope(30, 400, 0.0),
                    "volume": 0.25
                }
            ) + [
                whoosh_builder(
                    direction = "falling",
                    duration_ms = 150,
                    start_freq = 1500,
                    end_freq = 400,
                    noise_type = "pink"
                )
            ]
        )"#,
    )
    .unwrap();

    // Should have 4 layers total (3 from impact + 1 whoosh)
    let layers = result["recipe"]["params"]["layers"].as_array().unwrap();
    assert_eq!(layers.len(), 4);

    // Last layer should be the whoosh
    let whoosh = &layers[3];
    assert_eq!(whoosh["synthesis"]["noise_type"], "pink");
    assert_eq!(whoosh["filter"]["type"], "bandpass");
}
