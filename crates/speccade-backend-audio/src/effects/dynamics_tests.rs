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

// True-peak limiter tests

#[test]
fn test_true_peak_limiter_basic() {
    // Create a test signal with peaks exceeding the ceiling
    let mut stereo = StereoOutput {
        left: vec![0.1, 0.3, 0.8, 1.2, 0.9, 0.4, 0.1],
        right: vec![0.1, 0.3, 0.8, 1.2, 0.9, 0.4, 0.1],
    };

    // Apply limiter with ceiling at -1 dB (~0.89)
    let result = apply_true_peak_limiter(&mut stereo, -1.0, 100.0, 44100.0);

    assert!(result.is_ok());
    assert_eq!(stereo.left.len(), 7);
}

#[test]
fn test_true_peak_limiter_empty_buffer() {
    let mut stereo = StereoOutput {
        left: vec![],
        right: vec![],
    };

    let result = apply_true_peak_limiter(&mut stereo, -1.0, 100.0, 44100.0);
    assert!(result.is_ok());
}

#[test]
fn test_true_peak_limiter_invalid_ceiling() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // ceiling_db too low (< -6)
    let result = apply_true_peak_limiter(&mut stereo, -10.0, 100.0, 44100.0);
    assert!(result.is_err());

    // ceiling_db too high (> 0)
    let result = apply_true_peak_limiter(&mut stereo, 1.0, 100.0, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_true_peak_limiter_invalid_release() {
    let mut stereo = StereoOutput {
        left: vec![0.5],
        right: vec![0.5],
    };

    // release_ms too low (< 10)
    let result = apply_true_peak_limiter(&mut stereo, -1.0, 5.0, 44100.0);
    assert!(result.is_err());

    // release_ms too high (> 500)
    let result = apply_true_peak_limiter(&mut stereo, -1.0, 600.0, 44100.0);
    assert!(result.is_err());
}

#[test]
fn test_true_peak_limiter_determinism() {
    // Verify true-peak limiter produces identical output for identical input
    let create_stereo = || StereoOutput {
        left: vec![0.1, 0.3, 0.8, 1.2, 0.9, 0.4, 0.1],
        right: vec![0.1, 0.3, 0.8, 1.2, 0.9, 0.4, 0.1],
    };

    let mut stereo1 = create_stereo();
    let mut stereo2 = create_stereo();

    apply_true_peak_limiter(&mut stereo1, -1.0, 100.0, 44100.0).unwrap();
    apply_true_peak_limiter(&mut stereo2, -1.0, 100.0, 44100.0).unwrap();

    assert_eq!(stereo1.left, stereo2.left);
    assert_eq!(stereo1.right, stereo2.right);
}

#[test]
fn test_true_peak_limiter_limits_peaks() {
    // Create a signal with a clear peak that should be limited
    let sample_rate = 44100.0;
    let num_samples = 4410; // 100ms

    let mut stereo = StereoOutput {
        left: vec![0.0; num_samples],
        right: vec![0.0; num_samples],
    };

    // Create a burst at 0.5 amplitude, then a loud peak at 1.5 (should be limited)
    for i in 0..num_samples {
        let t = i as f64 / sample_rate;
        let signal = if i > num_samples / 2 && i < num_samples / 2 + 100 {
            1.5 // Loud burst
        } else {
            0.3 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()
        };
        stereo.left[i] = signal;
        stereo.right[i] = signal;
    }

    let ceiling_linear = 10.0_f64.powf(-1.0 / 20.0); // -1 dB

    apply_true_peak_limiter(&mut stereo, -1.0, 100.0, sample_rate).unwrap();

    // Check that no sample exceeds the ceiling significantly
    // (Due to lookahead there may be slight overshoots at the very start)
    let tolerance = 0.05; // 5% tolerance for edge effects
    for (i, &s) in stereo.left.iter().enumerate().skip(50) {
        assert!(
            s.abs() <= ceiling_linear + tolerance,
            "Sample {} exceeded ceiling: {} > {}",
            i,
            s.abs(),
            ceiling_linear + tolerance
        );
    }
}

#[test]
fn test_true_peak_limiter_broadcast_settings() {
    // Test typical broadcast settings (-2 dBTP ceiling)
    let mut stereo = StereoOutput {
        left: vec![0.5, 0.8, 1.0, 0.8, 0.5],
        right: vec![0.5, 0.8, 1.0, 0.8, 0.5],
    };

    let result = apply_true_peak_limiter(&mut stereo, -2.0, 200.0, 48000.0);
    assert!(result.is_ok());
}

#[test]
fn test_true_peak_limiter_streaming_settings() {
    // Test typical streaming settings (-1 dBTP ceiling)
    let mut stereo = StereoOutput {
        left: vec![0.5, 0.8, 1.0, 0.8, 0.5],
        right: vec![0.5, 0.8, 1.0, 0.8, 0.5],
    };

    let result = apply_true_peak_limiter(&mut stereo, -1.0, 100.0, 44100.0);
    assert!(result.is_ok());
}
