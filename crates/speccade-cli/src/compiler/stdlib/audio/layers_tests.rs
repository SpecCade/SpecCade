//! Tests for audio layer composition functions

use crate::compiler::stdlib::tests::eval_to_json;

// ========================================================================
// oneshot_envelope Tests
// ========================================================================

#[test]
fn test_oneshot_envelope_basic() {
    let result = eval_to_json(r#"oneshot_envelope(10, 200, 0.5)"#).unwrap();
    // 10ms attack = 0.01 seconds
    assert!((result["attack"].as_f64().unwrap() - 0.01).abs() < 0.0001);
    // 200ms decay = 0.2 seconds
    assert!((result["decay"].as_f64().unwrap() - 0.2).abs() < 0.0001);
    // Sustain level as provided
    assert!((result["sustain"].as_f64().unwrap() - 0.5).abs() < 0.0001);
    // Default release for one-shots is 50ms = 0.05 seconds
    assert!((result["release"].as_f64().unwrap() - 0.05).abs() < 0.0001);
}

#[test]
fn test_oneshot_envelope_drum_hit() {
    // Fast attack, medium decay, no sustain (typical drum)
    let result = eval_to_json(r#"oneshot_envelope(5, 150, 0.0)"#).unwrap();
    assert!((result["attack"].as_f64().unwrap() - 0.005).abs() < 0.0001);
    assert!((result["decay"].as_f64().unwrap() - 0.15).abs() < 0.0001);
    assert!((result["sustain"].as_f64().unwrap()).abs() < 0.0001);
}

#[test]
fn test_oneshot_envelope_zero_attack() {
    // Zero attack is valid (instant sound)
    let result = eval_to_json(r#"oneshot_envelope(0, 100, 0.3)"#).unwrap();
    assert!((result["attack"].as_f64().unwrap()).abs() < 0.0001);
}

#[test]
fn test_oneshot_envelope_negative_attack_fails() {
    let result = eval_to_json(r#"oneshot_envelope(-5, 100, 0.5)"#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("attack_ms"));
}

#[test]
fn test_oneshot_envelope_negative_decay_fails() {
    let result = eval_to_json(r#"oneshot_envelope(10, -100, 0.5)"#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("decay_ms"));
}

#[test]
fn test_oneshot_envelope_invalid_sustain_fails() {
    // Sustain > 1.0
    let result = eval_to_json(r#"oneshot_envelope(10, 100, 1.5)"#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("sustain_level"));

    // Sustain < 0.0
    let result = eval_to_json(r#"oneshot_envelope(10, 100, -0.5)"#);
    assert!(result.is_err());
}

// ========================================================================
// loop_envelope Tests
// ========================================================================

#[test]
fn test_loop_envelope_basic() {
    let result = eval_to_json(r#"loop_envelope(10, 1.0, 50)"#).unwrap();
    // 10ms attack = 0.01 seconds
    assert!((result["attack"].as_f64().unwrap() - 0.01).abs() < 0.0001);
    // Loop envelopes use minimal decay (0.01)
    assert!((result["decay"].as_f64().unwrap() - 0.01).abs() < 0.0001);
    // Sustain level as provided
    assert!((result["sustain"].as_f64().unwrap() - 1.0).abs() < 0.0001);
    // 50ms release = 0.05 seconds
    assert!((result["release"].as_f64().unwrap() - 0.05).abs() < 0.0001);
}

#[test]
fn test_loop_envelope_pad() {
    // Slow attack, high sustain, long release (typical pad)
    let result = eval_to_json(r#"loop_envelope(500, 0.9, 1000)"#).unwrap();
    assert!((result["attack"].as_f64().unwrap() - 0.5).abs() < 0.0001);
    assert!((result["sustain"].as_f64().unwrap() - 0.9).abs() < 0.0001);
    assert!((result["release"].as_f64().unwrap() - 1.0).abs() < 0.0001);
}

#[test]
fn test_loop_envelope_organ() {
    // Instant attack, full sustain, quick release (typical organ)
    let result = eval_to_json(r#"loop_envelope(5, 1.0, 30)"#).unwrap();
    assert!((result["attack"].as_f64().unwrap() - 0.005).abs() < 0.0001);
    assert!((result["sustain"].as_f64().unwrap() - 1.0).abs() < 0.0001);
    assert!((result["release"].as_f64().unwrap() - 0.03).abs() < 0.0001);
}

#[test]
fn test_loop_envelope_zero_attack() {
    let result = eval_to_json(r#"loop_envelope(0, 1.0, 100)"#).unwrap();
    assert!((result["attack"].as_f64().unwrap()).abs() < 0.0001);
}

#[test]
fn test_loop_envelope_negative_attack_fails() {
    let result = eval_to_json(r#"loop_envelope(-5, 1.0, 50)"#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("attack_ms"));
}

#[test]
fn test_loop_envelope_invalid_sustain_fails() {
    let result = eval_to_json(r#"loop_envelope(10, 1.5, 50)"#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("sustain_level"));
}

#[test]
fn test_loop_envelope_negative_release_fails() {
    let result = eval_to_json(r#"loop_envelope(10, 1.0, -50)"#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("release_ms"));
}

// ========================================================================
// with_loop_config Tests
// ========================================================================

#[test]
fn test_with_loop_config_basic() {
    let result = eval_to_json(
        r#"with_loop_config(audio_layer(oscillator(440), envelope = loop_envelope(10, 1.0, 50)))"#,
    )
    .unwrap();

    // Should have all original layer fields
    assert!(result.get("synthesis").is_some());
    assert!(result.get("envelope").is_some());

    // Should have loop_config added
    let loop_config = result.get("loop_config").unwrap();
    assert_eq!(loop_config["enabled"], true);
    assert_eq!(loop_config["snap_to_zero_crossing"], true);
    // Default crossfade_samples (441) / 44.1 = ~10ms
    assert!((loop_config["crossfade_ms"].as_f64().unwrap() - 10.0).abs() < 0.1);
}

#[test]
fn test_with_loop_config_custom_crossfade() {
    let result = eval_to_json(
        r#"with_loop_config(
            audio_layer(oscillator(440), envelope = loop_envelope(10, 1.0, 50)),
            crossfade_samples = 882
        )"#,
    )
    .unwrap();

    let loop_config = result.get("loop_config").unwrap();
    // 882 samples / 44.1 = ~20ms
    assert!((loop_config["crossfade_ms"].as_f64().unwrap() - 20.0).abs() < 0.1);
}

#[test]
fn test_with_loop_config_explicit_loop_points() {
    let result = eval_to_json(
        r#"with_loop_config(
            audio_layer(oscillator(440), envelope = loop_envelope(10, 1.0, 50)),
            loop_start = 4410,
            loop_end = 22050
        )"#,
    )
    .unwrap();

    let loop_config = result.get("loop_config").unwrap();
    assert_eq!(loop_config["start_sample"], 4410);
    assert_eq!(loop_config["end_sample"], 22050);
}

#[test]
fn test_with_loop_config_preserves_layer_fields() {
    let result = eval_to_json(
        r#"with_loop_config(
            audio_layer(oscillator(440), envelope = loop_envelope(10, 1.0, 50), volume = 0.7, pan = -0.5)
        )"#,
    )
    .unwrap();

    // All original fields should be preserved
    assert_eq!(result["volume"], 0.7);
    assert_eq!(result["pan"], -0.5);
    assert!(result.get("synthesis").is_some());
    assert!(result.get("envelope").is_some());
    assert!(result.get("loop_config").is_some());
}

#[test]
fn test_with_loop_config_negative_crossfade_fails() {
    let result = eval_to_json(
        r#"with_loop_config(
            audio_layer(oscillator(440)),
            crossfade_samples = -100
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("crossfade_samples"));
}

#[test]
fn test_with_loop_config_negative_loop_start_fails() {
    let result = eval_to_json(
        r#"with_loop_config(
            audio_layer(oscillator(440)),
            loop_start = -100
        )"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S103"));
    assert!(err.contains("loop_start"));
}

#[test]
fn test_with_loop_config_invalid_layer_type_fails() {
    let result = eval_to_json(r#"with_loop_config("not a dict")"#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("S102"));
    assert!(err.contains("expected dict"));
}

// ========================================================================
// Integration Tests: One-Shot vs Loop Pairing
// ========================================================================

#[test]
fn test_oneshot_vs_loop_pairing() {
    // This test demonstrates the workflow for creating paired samples
    // from the same synthesis

    // One-shot version: punchy attack, full decay
    let oneshot = eval_to_json(
        r#"audio_layer(
            oscillator(440, "sawtooth"),
            envelope = oneshot_envelope(5, 200, 0.3),
            volume = 0.8
        )"#,
    )
    .unwrap();

    // Loop version: quick attack, sustained, with loop config
    let looped = eval_to_json(
        r#"with_loop_config(
            audio_layer(
                oscillator(440, "sawtooth"),
                envelope = loop_envelope(5, 0.9, 100),
                volume = 0.8
            ),
            crossfade_samples = 512
        )"#,
    )
    .unwrap();

    // Verify both use the same synthesis
    assert_eq!(oneshot["synthesis"]["type"], looped["synthesis"]["type"]);
    assert_eq!(
        oneshot["synthesis"]["frequency"],
        looped["synthesis"]["frequency"]
    );
    assert_eq!(
        oneshot["synthesis"]["waveform"],
        looped["synthesis"]["waveform"]
    );

    // Verify both have same volume
    assert_eq!(oneshot["volume"], looped["volume"]);

    // Verify envelope differences
    // One-shot: longer decay, lower sustain
    assert!(
        oneshot["envelope"]["decay"].as_f64().unwrap()
            > looped["envelope"]["decay"].as_f64().unwrap()
    );
    assert!(
        oneshot["envelope"]["sustain"].as_f64().unwrap()
            < looped["envelope"]["sustain"].as_f64().unwrap()
    );

    // One-shot should NOT have loop_config
    assert!(oneshot.get("loop_config").is_none());

    // Loop version SHOULD have loop_config
    assert!(looped.get("loop_config").is_some());
    assert_eq!(looped["loop_config"]["enabled"], true);
}

#[test]
fn test_complete_instrument_sample_workflow() {
    // Demonstrates creating both one-shot and loop versions for a tracker instrument

    // Attack sample (one-shot): sharp transient for note-on
    let attack_sample = eval_to_json(
        r#"audio_spec(
            asset_id = "synth-lead-attack",
            seed = 42,
            output_path = "instruments/synth_lead_attack.wav",
            format = "wav",
            duration_seconds = 0.5,
            sample_rate = 44100,
            layers = [
                audio_layer(
                    oscillator(440, "sawtooth"),
                    envelope = oneshot_envelope(5, 100, 0.0),
                    volume = 0.9
                )
            ]
        )"#,
    )
    .unwrap();

    // Sustain sample (loop): for held notes
    let sustain_sample = eval_to_json(
        r#"audio_spec(
            asset_id = "synth-lead-sustain",
            seed = 42,
            output_path = "instruments/synth_lead_sustain.wav",
            format = "wav",
            duration_seconds = 1.0,
            sample_rate = 44100,
            layers = [
                with_loop_config(
                    audio_layer(
                        oscillator(440, "sawtooth"),
                        envelope = loop_envelope(10, 1.0, 50),
                        volume = 0.9
                    ),
                    crossfade_samples = 441
                )
            ]
        )"#,
    )
    .unwrap();

    // Verify both are valid audio specs
    assert_eq!(attack_sample["asset_type"], "audio");
    assert_eq!(sustain_sample["asset_type"], "audio");
    assert_eq!(attack_sample["recipe"]["kind"], "audio_v1");
    assert_eq!(sustain_sample["recipe"]["kind"], "audio_v1");

    // Attack sample is shorter (transient only)
    assert!(
        attack_sample["recipe"]["params"]["duration_seconds"]
            .as_f64()
            .unwrap()
            < sustain_sample["recipe"]["params"]["duration_seconds"]
                .as_f64()
                .unwrap()
    );

    // Sustain sample has loop config
    let sustain_layer = &sustain_sample["recipe"]["params"]["layers"][0];
    assert!(sustain_layer.get("loop_config").is_some());
}
