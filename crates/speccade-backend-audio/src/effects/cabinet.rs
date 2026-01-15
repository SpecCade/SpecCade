//! Cabinet simulation effect implementation.
//!
//! Implements cabinet simulation using cascaded biquad filters to approximate
//! the frequency response of various speaker cabinets. This is a procedural
//! approach without convolution/IR for deterministic output.

use speccade_spec::recipe::audio::CabinetType;

use crate::error::AudioResult;
use crate::filter::{BiquadCoeffs, BiquadFilter};
use crate::mixer::StereoOutput;

/// Applies cabinet simulation to stereo audio.
///
/// # Arguments
/// * `stereo` - Stereo audio buffer to process in place
/// * `cabinet_type` - Cabinet type defining the filter curve
/// * `mic_position` - Mic position (0.0 = close/bright, 1.0 = far/dark)
/// * `sample_rate` - Sample rate in Hz
pub fn apply(
    stereo: &mut StereoOutput,
    cabinet_type: CabinetType,
    mic_position: f64,
    sample_rate: f64,
) -> AudioResult<()> {
    let mic_position = mic_position.clamp(0.0, 1.0);

    // Create filters for the cabinet type
    let mut filters = create_cabinet_filters(cabinet_type, mic_position, sample_rate);

    // Process all samples through all filters (stereo pairs)
    for i in 0..stereo.left.len() {
        let mut left = stereo.left[i];
        let mut right = stereo.right[i];

        // Apply each filter in sequence
        for (filter_l, filter_r) in &mut filters {
            left = filter_l.process(left);
            right = filter_r.process(right);
        }

        stereo.left[i] = left;
        stereo.right[i] = right;
    }

    Ok(())
}

/// Creates the filter chain for a cabinet type.
fn create_cabinet_filters(
    cabinet_type: CabinetType,
    mic_position: f64,
    sample_rate: f64,
) -> Vec<(BiquadFilter, BiquadFilter)> {
    let mut filters = Vec::new();

    match cabinet_type {
        CabinetType::Guitar1x12 => {
            // Classic combo amp: bright, focused
            // Highpass: 80 Hz
            let hp = BiquadCoeffs::highpass(80.0, 0.707, sample_rate);
            filters.push((BiquadFilter::new(hp), BiquadFilter::new(hp)));

            // Peak boost: 3 kHz, +3 dB, Q=1.0
            let peak = BiquadCoeffs::peaking_eq(3000.0, 1.0, 3.0, sample_rate);
            filters.push((BiquadFilter::new(peak), BiquadFilter::new(peak)));

            // Lowpass: 6 kHz (adjusted by mic position)
            let lp_cutoff = apply_mic_rolloff(6000.0, mic_position);
            let lp = BiquadCoeffs::lowpass(lp_cutoff, 0.707, sample_rate);
            filters.push((BiquadFilter::new(lp), BiquadFilter::new(lp)));
        }
        CabinetType::Guitar4x12 => {
            // Big stack sound: full, warm
            // Highpass: 60 Hz
            let hp = BiquadCoeffs::highpass(60.0, 0.707, sample_rate);
            filters.push((BiquadFilter::new(hp), BiquadFilter::new(hp)));

            // Low shelf boost: 120 Hz, +2 dB
            let low_shelf = BiquadCoeffs::low_shelf(120.0, 2.0, sample_rate);
            filters.push((BiquadFilter::new(low_shelf), BiquadFilter::new(low_shelf)));

            // Peak cut: 400 Hz, -2 dB (less mud)
            let mid_cut = BiquadCoeffs::peaking_eq(400.0, 1.0, -2.0, sample_rate);
            filters.push((BiquadFilter::new(mid_cut), BiquadFilter::new(mid_cut)));

            // Peak boost: 2.5 kHz, +3 dB
            let presence = BiquadCoeffs::peaking_eq(2500.0, 1.0, 3.0, sample_rate);
            filters.push((BiquadFilter::new(presence), BiquadFilter::new(presence)));

            // Lowpass: 5 kHz (adjusted by mic position)
            let lp_cutoff = apply_mic_rolloff(5000.0, mic_position);
            let lp = BiquadCoeffs::lowpass(lp_cutoff, 0.707, sample_rate);
            filters.push((BiquadFilter::new(lp), BiquadFilter::new(lp)));
        }
        CabinetType::Bass1x15 => {
            // Bass cabinet: deep, punchy
            // Highpass: 40 Hz
            let hp = BiquadCoeffs::highpass(40.0, 0.707, sample_rate);
            filters.push((BiquadFilter::new(hp), BiquadFilter::new(hp)));

            // Low shelf boost: 80 Hz, +4 dB
            let low_shelf = BiquadCoeffs::low_shelf(80.0, 4.0, sample_rate);
            filters.push((BiquadFilter::new(low_shelf), BiquadFilter::new(low_shelf)));

            // Peak cut: 600 Hz, -3 dB
            let mid_cut = BiquadCoeffs::peaking_eq(600.0, 1.0, -3.0, sample_rate);
            filters.push((BiquadFilter::new(mid_cut), BiquadFilter::new(mid_cut)));

            // Lowpass: 4 kHz (adjusted by mic position)
            let lp_cutoff = apply_mic_rolloff(4000.0, mic_position);
            let lp = BiquadCoeffs::lowpass(lp_cutoff, 0.707, sample_rate);
            filters.push((BiquadFilter::new(lp), BiquadFilter::new(lp)));
        }
        CabinetType::Radio => {
            // AM radio lo-fi: bandlimited
            // Highpass: 300 Hz
            let hp = BiquadCoeffs::highpass(300.0, 0.707, sample_rate);
            filters.push((BiquadFilter::new(hp), BiquadFilter::new(hp)));

            // Bandpass centered: 1.5 kHz (using peak boost with moderate Q)
            let bp = BiquadCoeffs::peaking_eq(1500.0, 1.5, 4.0, sample_rate);
            filters.push((BiquadFilter::new(bp), BiquadFilter::new(bp)));

            // Lowpass: 3 kHz (adjusted by mic position)
            let lp_cutoff = apply_mic_rolloff(3000.0, mic_position);
            let lp = BiquadCoeffs::lowpass(lp_cutoff, 0.707, sample_rate);
            filters.push((BiquadFilter::new(lp), BiquadFilter::new(lp)));
        }
        CabinetType::Telephone => {
            // Telephone line quality: narrow bandwidth
            // Highpass: 300 Hz
            let hp = BiquadCoeffs::highpass(300.0, 0.707, sample_rate);
            filters.push((BiquadFilter::new(hp), BiquadFilter::new(hp)));

            // Bandpass centered: 1.5 kHz, narrow Q (using peak boost with high Q)
            let bp = BiquadCoeffs::peaking_eq(1500.0, 3.0, 6.0, sample_rate);
            filters.push((BiquadFilter::new(bp), BiquadFilter::new(bp)));

            // Lowpass: 3.4 kHz (adjusted by mic position)
            let lp_cutoff = apply_mic_rolloff(3400.0, mic_position);
            let lp = BiquadCoeffs::lowpass(lp_cutoff, 0.707, sample_rate);
            filters.push((BiquadFilter::new(lp), BiquadFilter::new(lp)));
        }
    }

    filters
}

/// Applies mic position rolloff to a base lowpass cutoff.
///
/// mic_position 0.0 = close mic (brighter, base cutoff)
/// mic_position 1.0 = far mic (darker, 30% lower cutoff)
fn apply_mic_rolloff(base_cutoff: f64, mic_position: f64) -> f64 {
    base_cutoff * (1.0 - 0.3 * mic_position)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stereo_buffer(len: usize, value: f64) -> StereoOutput {
        StereoOutput {
            left: vec![value; len],
            right: vec![value; len],
        }
    }

    #[test]
    fn test_guitar_1x12_processes() {
        let mut stereo = make_stereo_buffer(1000, 0.5);
        apply(&mut stereo, CabinetType::Guitar1x12, 0.0, 44100.0).unwrap();

        // Should produce finite output
        assert!(stereo.left[999].is_finite());
        assert!(stereo.right[999].is_finite());
    }

    #[test]
    fn test_guitar_4x12_processes() {
        let mut stereo = make_stereo_buffer(1000, 0.5);
        apply(&mut stereo, CabinetType::Guitar4x12, 0.5, 44100.0).unwrap();

        // Should produce finite output
        assert!(stereo.left[999].is_finite());
        assert!(stereo.right[999].is_finite());
    }

    #[test]
    fn test_bass_1x15_processes() {
        let mut stereo = make_stereo_buffer(1000, 0.5);
        apply(&mut stereo, CabinetType::Bass1x15, 1.0, 44100.0).unwrap();

        // Should produce finite output
        assert!(stereo.left[999].is_finite());
        assert!(stereo.right[999].is_finite());
    }

    #[test]
    fn test_radio_processes() {
        let mut stereo = make_stereo_buffer(1000, 0.5);
        apply(&mut stereo, CabinetType::Radio, 0.3, 44100.0).unwrap();

        // Should produce finite output
        assert!(stereo.left[999].is_finite());
        assert!(stereo.right[999].is_finite());
    }

    #[test]
    fn test_telephone_processes() {
        let mut stereo = make_stereo_buffer(1000, 0.5);
        apply(&mut stereo, CabinetType::Telephone, 0.7, 44100.0).unwrap();

        // Should produce finite output
        assert!(stereo.left[999].is_finite());
        assert!(stereo.right[999].is_finite());
    }

    #[test]
    fn test_mic_position_clamped() {
        let mut stereo = make_stereo_buffer(1000, 0.5);

        // Negative mic position should be clamped to 0.0
        apply(&mut stereo, CabinetType::Guitar1x12, -1.0, 44100.0).unwrap();
        assert!(stereo.left[999].is_finite());

        // Mic position > 1.0 should be clamped to 1.0
        let mut stereo2 = make_stereo_buffer(1000, 0.5);
        apply(&mut stereo2, CabinetType::Guitar1x12, 2.0, 44100.0).unwrap();
        assert!(stereo2.left[999].is_finite());
    }

    #[test]
    fn test_mic_rolloff() {
        // Close mic (0.0) should use base cutoff
        assert!((apply_mic_rolloff(6000.0, 0.0) - 6000.0).abs() < 0.01);

        // Far mic (1.0) should reduce cutoff by 30%
        assert!((apply_mic_rolloff(6000.0, 1.0) - 4200.0).abs() < 0.01);

        // Mid position (0.5) should reduce cutoff by 15%
        assert!((apply_mic_rolloff(6000.0, 0.5) - 5100.0).abs() < 0.01);
    }

    #[test]
    fn test_deterministic_output() {
        let mut stereo1 = make_stereo_buffer(100, 0.5);
        let mut stereo2 = make_stereo_buffer(100, 0.5);

        apply(&mut stereo1, CabinetType::Guitar4x12, 0.5, 44100.0).unwrap();
        apply(&mut stereo2, CabinetType::Guitar4x12, 0.5, 44100.0).unwrap();

        // Output should be identical for identical input
        for i in 0..100 {
            assert_eq!(stereo1.left[i], stereo2.left[i]);
            assert_eq!(stereo1.right[i], stereo2.right[i]);
        }
    }

    #[test]
    fn test_stereo_independence() {
        // Left and right channels with different values
        let mut stereo = StereoOutput {
            left: vec![1.0; 1000],
            right: vec![0.5; 1000],
        };

        apply(&mut stereo, CabinetType::Guitar1x12, 0.0, 44100.0).unwrap();

        // Both should be processed but remain finite
        assert!(stereo.left[999].is_finite());
        assert!(stereo.right[999].is_finite());
    }
}
