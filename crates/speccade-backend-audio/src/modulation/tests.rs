//! LFO modulation unit tests.

use super::lfo::*;
use crate::rng::create_rng;
use speccade_spec::recipe::audio::Waveform;

#[test]
fn test_lfo_sine_generation() {
    let mut rng = create_rng(42);
    let mut lfo = Lfo::new(Waveform::Sine, 1.0, 44100.0, 0.0);

    // Generate some samples
    let samples = lfo.generate(100, &mut rng);

    // All samples should be in range [0.0, 1.0]
    for sample in &samples {
        assert!(
            (0.0..=1.0).contains(sample),
            "Sample {} out of range",
            sample
        );
    }
}

#[test]
fn test_lfo_determinism() {
    let mut rng1 = create_rng(42);
    let mut rng2 = create_rng(42);

    let mut lfo1 = Lfo::new(Waveform::Sine, 2.0, 44100.0, 0.0);
    let mut lfo2 = Lfo::new(Waveform::Sine, 2.0, 44100.0, 0.0);

    let samples1 = lfo1.generate(100, &mut rng1);
    let samples2 = lfo2.generate(100, &mut rng2);

    assert_eq!(samples1, samples2);
}

#[test]
fn test_pitch_modulation() {
    let base_freq = 440.0; // A4
    let lfo_value = 0.5; // Center (no modulation)
    let semitones = 12.0; // One octave range

    let modulated = apply_pitch_modulation(base_freq, lfo_value, semitones);

    // At center, should be close to base frequency
    assert!((modulated - base_freq).abs() < 0.1);

    // At max (lfo_value = 1.0), should be one octave up
    let modulated_max = apply_pitch_modulation(base_freq, 1.0, semitones);
    assert!((modulated_max / base_freq - 2.0).abs() < 0.01);

    // At min (lfo_value = 0.0), should be one octave down
    let modulated_min = apply_pitch_modulation(base_freq, 0.0, semitones);
    assert!((modulated_min / base_freq - 0.5).abs() < 0.01);
}

#[test]
fn test_volume_modulation() {
    let amplitude = 1.0;
    let amount = 1.0;
    let depth = 1.0;

    // At LFO max, should be full amplitude
    let mod_max = apply_volume_modulation(amplitude, 1.0, amount, depth);
    assert!((mod_max - 1.0).abs() < 0.01);

    // At LFO min, should be zero with full depth
    let mod_min = apply_volume_modulation(amplitude, 0.0, amount, depth);
    assert!(mod_min < 0.01);

    // At LFO center with half depth, should be 0.75
    let mod_center = apply_volume_modulation(amplitude, 0.5, 1.0, 0.5);
    assert!((mod_center - 0.75).abs() < 0.01);
}

#[test]
fn test_filter_cutoff_modulation() {
    let base_cutoff = 1000.0;
    let amount = 500.0;
    let depth = 1.0;

    // At center, should be base cutoff
    let mod_center = apply_filter_cutoff_modulation(base_cutoff, 0.5, amount, depth);
    assert!((mod_center - base_cutoff).abs() < 0.1);

    // At max, should be base + amount
    let mod_max = apply_filter_cutoff_modulation(base_cutoff, 1.0, amount, depth);
    assert!((mod_max - (base_cutoff + amount)).abs() < 0.1);

    // At min, should be base - amount
    let mod_min = apply_filter_cutoff_modulation(base_cutoff, 0.0, amount, depth);
    assert!((mod_min - (base_cutoff - amount)).abs() < 0.1);
}

#[test]
fn test_pan_modulation() {
    let base_pan = 0.0; // Center
    let amount = 1.0;
    let depth = 1.0;

    // At center, should be close to base
    let mod_center = apply_pan_modulation(base_pan, 0.5, amount, depth);
    assert!(mod_center.abs() < 0.01);

    // At max, should be right
    let mod_max = apply_pan_modulation(base_pan, 1.0, amount, depth);
    assert!((mod_max - 1.0).abs() < 0.01);

    // At min, should be left
    let mod_min = apply_pan_modulation(base_pan, 0.0, amount, depth);
    assert!((mod_min + 1.0).abs() < 0.01);
}

#[test]
fn test_pulse_width_modulation() {
    let base_duty = 0.5; // 50% duty cycle
    let amount = 0.4; // +/- 40% delta
    let depth = 1.0;

    // At center (LFO = 0.5), should be close to base duty
    let mod_center = apply_pulse_width_modulation(base_duty, 0.5, amount, depth);
    assert!((mod_center - base_duty).abs() < 0.01);

    // At max (LFO = 1.0), should be base + amount = 0.9
    let mod_max = apply_pulse_width_modulation(base_duty, 1.0, amount, depth);
    assert!((mod_max - 0.9).abs() < 0.01);

    // At min (LFO = 0.0), should be base - amount = 0.1
    let mod_min = apply_pulse_width_modulation(base_duty, 0.0, amount, depth);
    assert!((mod_min - 0.1).abs() < 0.01);

    // Test clamping at extremes
    let mod_extreme_max = apply_pulse_width_modulation(0.95, 1.0, 0.49, 1.0);
    assert!(mod_extreme_max <= 0.99);

    let mod_extreme_min = apply_pulse_width_modulation(0.05, 0.0, 0.49, 1.0);
    assert!(mod_extreme_min >= 0.01);

    // Test depth scaling
    let mod_half_depth = apply_pulse_width_modulation(base_duty, 1.0, amount, 0.5);
    assert!((mod_half_depth - 0.7).abs() < 0.01); // 0.5 + 0.4 * 0.5 = 0.7
}

#[test]
fn test_fm_index_modulation() {
    let base_index = 4.0;
    let amount = 2.0;
    let depth = 1.0;

    // At center (LFO = 0.5), should be close to base index
    let mod_center = apply_fm_index_modulation(base_index, 0.5, amount, depth);
    assert!((mod_center - base_index).abs() < 0.01);

    // At max (LFO = 1.0), should be base + amount = 6.0
    let mod_max = apply_fm_index_modulation(base_index, 1.0, amount, depth);
    assert!((mod_max - 6.0).abs() < 0.01);

    // At min (LFO = 0.0), should be base - amount = 2.0
    let mod_min = apply_fm_index_modulation(base_index, 0.0, amount, depth);
    assert!((mod_min - 2.0).abs() < 0.01);

    // Test clamping at zero (index can't go negative)
    let mod_extreme_min = apply_fm_index_modulation(1.0, 0.0, 5.0, 1.0);
    assert!(mod_extreme_min >= 0.0);
    assert!((mod_extreme_min).abs() < 0.01); // 1.0 - 5.0 clamped to 0.0

    // Test depth scaling
    let mod_half_depth = apply_fm_index_modulation(base_index, 1.0, amount, 0.5);
    assert!((mod_half_depth - 5.0).abs() < 0.01); // 4.0 + 2.0 * 0.5 = 5.0
}

#[test]
fn test_grain_size_modulation() {
    let base_size_ms = 50.0;
    let amount_ms = 30.0;
    let depth = 1.0;

    // At center (LFO = 0.5), should be close to base size
    let mod_center = apply_grain_size_modulation(base_size_ms, 0.5, amount_ms, depth);
    assert!((mod_center - base_size_ms).abs() < 0.01);

    // At max (LFO = 1.0), should be base + amount = 80.0
    let mod_max = apply_grain_size_modulation(base_size_ms, 1.0, amount_ms, depth);
    assert!((mod_max - 80.0).abs() < 0.01);

    // At min (LFO = 0.0), should be base - amount = 20.0
    let mod_min = apply_grain_size_modulation(base_size_ms, 0.0, amount_ms, depth);
    assert!((mod_min - 20.0).abs() < 0.01);

    // Test clamping at minimum (10.0)
    let mod_extreme_min = apply_grain_size_modulation(15.0, 0.0, 50.0, 1.0);
    assert!((mod_extreme_min - 10.0).abs() < 0.01);

    // Test clamping at maximum (500.0)
    let mod_extreme_max = apply_grain_size_modulation(480.0, 1.0, 50.0, 1.0);
    assert!((mod_extreme_max - 500.0).abs() < 0.01);

    // Test depth scaling
    let mod_half_depth = apply_grain_size_modulation(base_size_ms, 1.0, amount_ms, 0.5);
    assert!((mod_half_depth - 65.0).abs() < 0.01); // 50.0 + 30.0 * 0.5 = 65.0
}

#[test]
fn test_grain_density_modulation() {
    let base_density = 20.0;
    let amount = 15.0;
    let depth = 1.0;

    // At center (LFO = 0.5), should be close to base density
    let mod_center = apply_grain_density_modulation(base_density, 0.5, amount, depth);
    assert!((mod_center - base_density).abs() < 0.01);

    // At max (LFO = 1.0), should be base + amount = 35.0
    let mod_max = apply_grain_density_modulation(base_density, 1.0, amount, depth);
    assert!((mod_max - 35.0).abs() < 0.01);

    // At min (LFO = 0.0), should be base - amount = 5.0
    let mod_min = apply_grain_density_modulation(base_density, 0.0, amount, depth);
    assert!((mod_min - 5.0).abs() < 0.01);

    // Test clamping at minimum (1.0)
    let mod_extreme_min = apply_grain_density_modulation(5.0, 0.0, 50.0, 1.0);
    assert!((mod_extreme_min - 1.0).abs() < 0.01);

    // Test clamping at maximum (100.0)
    let mod_extreme_max = apply_grain_density_modulation(90.0, 1.0, 50.0, 1.0);
    assert!((mod_extreme_max - 100.0).abs() < 0.01);

    // Test depth scaling
    let mod_half_depth = apply_grain_density_modulation(base_density, 1.0, amount, 0.5);
    assert!((mod_half_depth - 27.5).abs() < 0.01); // 20.0 + 15.0 * 0.5 = 27.5
}

#[test]
fn test_delay_time_modulation() {
    let base_time_ms = 250.0;
    let amount_ms = 50.0;
    let depth = 1.0;

    // At center (LFO = 0.5), should be close to base time
    let mod_center = apply_delay_time_modulation(base_time_ms, 0.5, amount_ms, depth);
    assert!((mod_center - base_time_ms).abs() < 0.01);

    // At max (LFO = 1.0), should be base + amount = 300.0
    let mod_max = apply_delay_time_modulation(base_time_ms, 1.0, amount_ms, depth);
    assert!((mod_max - 300.0).abs() < 0.01);

    // At min (LFO = 0.0), should be base - amount = 200.0
    let mod_min = apply_delay_time_modulation(base_time_ms, 0.0, amount_ms, depth);
    assert!((mod_min - 200.0).abs() < 0.01);

    // Test clamping at minimum (1.0)
    let mod_extreme_min = apply_delay_time_modulation(10.0, 0.0, 50.0, 1.0);
    assert!((mod_extreme_min - 1.0).abs() < 0.01);

    // Test clamping at maximum (2000.0)
    let mod_extreme_max = apply_delay_time_modulation(1990.0, 1.0, 50.0, 1.0);
    assert!((mod_extreme_max - 2000.0).abs() < 0.01);

    // Test depth scaling
    let mod_half_depth = apply_delay_time_modulation(base_time_ms, 1.0, amount_ms, 0.5);
    assert!((mod_half_depth - 275.0).abs() < 0.01); // 250.0 + 50.0 * 0.5 = 275.0
}

#[test]
fn test_reverb_size_modulation() {
    let base_room_size = 0.5;
    let amount = 0.3;
    let depth = 1.0;

    // At center (LFO = 0.5), should be close to base room size
    let mod_center = apply_reverb_size_modulation(base_room_size, 0.5, amount, depth);
    assert!((mod_center - base_room_size).abs() < 0.01);

    // At max (LFO = 1.0), should be base + amount = 0.8
    let mod_max = apply_reverb_size_modulation(base_room_size, 1.0, amount, depth);
    assert!((mod_max - 0.8).abs() < 0.01);

    // At min (LFO = 0.0), should be base - amount = 0.2
    let mod_min = apply_reverb_size_modulation(base_room_size, 0.0, amount, depth);
    assert!((mod_min - 0.2).abs() < 0.01);

    // Test clamping at minimum (0.0)
    let mod_extreme_min = apply_reverb_size_modulation(0.1, 0.0, 0.5, 1.0);
    assert!((mod_extreme_min).abs() < 0.01);

    // Test clamping at maximum (1.0)
    let mod_extreme_max = apply_reverb_size_modulation(0.9, 1.0, 0.5, 1.0);
    assert!((mod_extreme_max - 1.0).abs() < 0.01);

    // Test depth scaling
    let mod_half_depth = apply_reverb_size_modulation(base_room_size, 1.0, amount, 0.5);
    assert!((mod_half_depth - 0.65).abs() < 0.01); // 0.5 + 0.3 * 0.5 = 0.65
}

#[test]
fn test_distortion_drive_modulation() {
    let base_drive = 10.0;
    let amount = 5.0;
    let depth = 1.0;

    // At center (LFO = 0.5), should be close to base drive
    let mod_center = apply_distortion_drive_modulation(base_drive, 0.5, amount, depth);
    assert!((mod_center - base_drive).abs() < 0.01);

    // At max (LFO = 1.0), should be base + amount = 15.0
    let mod_max = apply_distortion_drive_modulation(base_drive, 1.0, amount, depth);
    assert!((mod_max - 15.0).abs() < 0.01);

    // At min (LFO = 0.0), should be base - amount = 5.0
    let mod_min = apply_distortion_drive_modulation(base_drive, 0.0, amount, depth);
    assert!((mod_min - 5.0).abs() < 0.01);

    // Test clamping at minimum (1.0)
    let mod_extreme_min = apply_distortion_drive_modulation(3.0, 0.0, 10.0, 1.0);
    assert!((mod_extreme_min - 1.0).abs() < 0.01);

    // Test clamping at maximum (100.0)
    let mod_extreme_max = apply_distortion_drive_modulation(95.0, 1.0, 20.0, 1.0);
    assert!((mod_extreme_max - 100.0).abs() < 0.01);

    // Test depth scaling
    let mod_half_depth = apply_distortion_drive_modulation(base_drive, 1.0, amount, 0.5);
    assert!((mod_half_depth - 12.5).abs() < 0.01); // 10.0 + 5.0 * 0.5 = 12.5
}
