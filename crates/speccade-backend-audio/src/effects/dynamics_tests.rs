//! Tests for dynamics processing (compressor, limiter, gate/expander).

use super::*;
use crate::mixer::StereoOutput;

#[test]
fn test_limiter_basic() {
    // Create a simple test signal with a loud peak
    let mut stereo = StereoOutput {
        left: vec![0.1, 0.2, 0.5, 1.0, 0.5, 0.2, 0.1], // 1.0 is above threshold
        right: vec![0.1, 0.2, 0.5, 1.0, 0.5, 0.2, 0.1],
    };

    // Apply limiter with threshold at -6dB (0.5 linear) and ceiling at -3dB
    let result = apply_limiter(
        &mut stereo,
        -6.0,  // threshold_db
        100.0, // release_ms
        1.0,   // lookahead_ms
        -3.0,  // ceiling_db
        44100.0,
    );

    assert!(result.is_ok());
    // After limiting, the output should be processed
    // The exact values depend on the algorithm, but we can verify it ran
    assert_eq!(stereo.left.len(), 7);
}

#[test]
fn test_limiter_empty_buffer() {
    let mut stereo = StereoOutput {
        left: vec![],
        right: vec![],
    };

    let result = apply_limiter(&mut stereo, -6.0, 100.0, 5.0, -0.3, 44100.0);
    assert!(result.is_ok());
}

#[test]
fn test_limiter_invalid_threshold() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // threshold_db too low (< -24)
    let result = apply_limiter(&mut stereo, -30.0, 100.0, 5.0, -0.3, 44100.0);
    assert!(result.is_err());

    // threshold_db too high (> 0)
    let result = apply_limiter(&mut stereo, 1.0, 100.0, 5.0, -0.3, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_limiter_invalid_release() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // release_ms too low (< 10)
    let result = apply_limiter(&mut stereo, -6.0, 5.0, 5.0, -0.3, 44100.0);
    assert!(result.is_err());

    // release_ms too high (> 500)
    let result = apply_limiter(&mut stereo, -6.0, 600.0, 5.0, -0.3, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_limiter_invalid_lookahead() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // lookahead_ms too low (< 1)
    let result = apply_limiter(&mut stereo, -6.0, 100.0, 0.5, -0.3, 44100.0);
    assert!(result.is_err());

    // lookahead_ms too high (> 10)
    let result = apply_limiter(&mut stereo, -6.0, 100.0, 15.0, -0.3, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_limiter_invalid_ceiling() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // ceiling_db too low (< -6)
    let result = apply_limiter(&mut stereo, -6.0, 100.0, 5.0, -10.0, 44100.0);
    assert!(result.is_err());

    // ceiling_db too high (> 0)
    let result = apply_limiter(&mut stereo, -6.0, 100.0, 5.0, 1.0, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_limiter_determinism() {
    // Verify limiter produces identical output for identical input
    let create_stereo = || StereoOutput {
        left: vec![0.1, 0.3, 0.8, 1.2, 0.9, 0.4, 0.2],
        right: vec![0.1, 0.3, 0.8, 1.2, 0.9, 0.4, 0.2],
    };

    let mut stereo1 = create_stereo();
    let mut stereo2 = create_stereo();

    apply_limiter(&mut stereo1, -6.0, 100.0, 5.0, -0.3, 44100.0).unwrap();
    apply_limiter(&mut stereo2, -6.0, 100.0, 5.0, -0.3, 44100.0).unwrap();

    assert_eq!(stereo1.left, stereo2.left);
    assert_eq!(stereo1.right, stereo2.right);
}

#[test]
fn test_db_conversions() {
    // Test round-trip conversion
    let db = -6.0;
    let linear = db_to_amp(db);
    let back_to_db = amp_to_db(linear);
    assert!((db - back_to_db).abs() < 0.001);

    // Test known values
    assert!((db_to_amp(0.0) - 1.0).abs() < 0.001);
    assert!((db_to_amp(-6.0) - 0.5011872).abs() < 0.001);
    assert!((db_to_amp(-20.0) - 0.1).abs() < 0.001);
}

#[test]
fn test_gate_expander_basic() {
    // Signal that goes above and below threshold
    let mut stereo = StereoOutput {
        left: vec![0.01, 0.02, 0.5, 0.8, 0.5, 0.02, 0.01],
        right: vec![0.01, 0.02, 0.5, 0.8, 0.5, 0.02, 0.01],
    };

    // Threshold at -12dB (0.25 linear), low samples should be attenuated
    let result = apply_gate_expander(
        &mut stereo,
        -12.0,   // threshold_db
        4.0,     // ratio
        1.0,     // attack_ms
        0.0,     // hold_ms
        100.0,   // release_ms
        -60.0,   // range_db
        44100.0, // sample_rate
    );

    assert!(result.is_ok());
    assert_eq!(stereo.left.len(), 7);
}

#[test]
fn test_gate_expander_empty_buffer() {
    let mut stereo = StereoOutput {
        left: vec![],
        right: vec![],
    };

    let result = apply_gate_expander(&mut stereo, -30.0, 4.0, 1.0, 50.0, 100.0, -60.0, 44100.0);
    assert!(result.is_ok());
}

#[test]
fn test_gate_expander_invalid_threshold() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // threshold_db too low (< -60)
    let result = apply_gate_expander(&mut stereo, -70.0, 4.0, 1.0, 50.0, 100.0, -60.0, 44100.0);
    assert!(result.is_err());

    // threshold_db too high (> 0)
    let result = apply_gate_expander(&mut stereo, 1.0, 4.0, 1.0, 50.0, 100.0, -60.0, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_gate_expander_invalid_ratio() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // ratio too low (< 1.0)
    let result = apply_gate_expander(&mut stereo, -30.0, 0.5, 1.0, 50.0, 100.0, -60.0, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_gate_expander_invalid_attack() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // attack_ms too low (< 0.1)
    let result = apply_gate_expander(&mut stereo, -30.0, 4.0, 0.05, 50.0, 100.0, -60.0, 44100.0);
    assert!(result.is_err());

    // attack_ms too high (> 50)
    let result = apply_gate_expander(&mut stereo, -30.0, 4.0, 60.0, 50.0, 100.0, -60.0, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_gate_expander_invalid_hold() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // hold_ms too low (< 0)
    let result = apply_gate_expander(&mut stereo, -30.0, 4.0, 1.0, -1.0, 100.0, -60.0, 44100.0);
    assert!(result.is_err());

    // hold_ms too high (> 500)
    let result = apply_gate_expander(&mut stereo, -30.0, 4.0, 1.0, 600.0, 100.0, -60.0, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_gate_expander_invalid_release() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // release_ms too low (< 10)
    let result = apply_gate_expander(&mut stereo, -30.0, 4.0, 1.0, 50.0, 5.0, -60.0, 44100.0);
    assert!(result.is_err());

    // release_ms too high (> 2000)
    let result = apply_gate_expander(&mut stereo, -30.0, 4.0, 1.0, 50.0, 2500.0, -60.0, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_gate_expander_invalid_range() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // range_db too low (< -80)
    let result = apply_gate_expander(&mut stereo, -30.0, 4.0, 1.0, 50.0, 100.0, -90.0, 44100.0);
    assert!(result.is_err());

    // range_db too high (> 0)
    let result = apply_gate_expander(&mut stereo, -30.0, 4.0, 1.0, 50.0, 100.0, 1.0, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_gate_expander_determinism() {
    // Verify gate produces identical output for identical input
    let create_stereo = || StereoOutput {
        left: vec![0.01, 0.1, 0.5, 0.8, 0.5, 0.1, 0.01],
        right: vec![0.01, 0.1, 0.5, 0.8, 0.5, 0.1, 0.01],
    };

    let mut stereo1 = create_stereo();
    let mut stereo2 = create_stereo();

    apply_gate_expander(&mut stereo1, -20.0, 4.0, 1.0, 50.0, 100.0, -60.0, 44100.0).unwrap();
    apply_gate_expander(&mut stereo2, -20.0, 4.0, 1.0, 50.0, 100.0, -60.0, 44100.0).unwrap();

    assert_eq!(stereo1.left, stereo2.left);
    assert_eq!(stereo1.right, stereo2.right);
}
