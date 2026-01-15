//! Parametric EQ effect implementation.
//!
//! Implements a multi-band parametric equalizer using cascaded biquad filters.

use speccade_spec::recipe::audio::{EqBand, EqBandType};

use crate::filter::{BiquadCoeffs, BiquadFilter};
use crate::mixer::StereoOutput;

/// Applies parametric EQ to stereo audio.
///
/// Each band is processed as a cascaded biquad filter. Bands are applied
/// in the order they appear in the list.
///
/// # Arguments
/// * `stereo` - Stereo audio buffer to process in place
/// * `bands` - EQ bands to apply
/// * `sample_rate` - Sample rate in Hz
pub fn apply(stereo: &mut StereoOutput, bands: &[EqBand], sample_rate: f64) {
    if bands.is_empty() {
        return;
    }

    // Create filter pairs (left/right) for each band
    let mut filters: Vec<(BiquadFilter, BiquadFilter)> = bands
        .iter()
        .map(|band| {
            let coeffs = compute_band_coefficients(band, sample_rate);
            (BiquadFilter::new(coeffs), BiquadFilter::new(coeffs))
        })
        .collect();

    // Process all samples through all bands
    for i in 0..stereo.left.len() {
        let mut left = stereo.left[i];
        let mut right = stereo.right[i];

        // Apply each band in sequence
        for (filter_l, filter_r) in &mut filters {
            left = filter_l.process(left);
            right = filter_r.process(right);
        }

        stereo.left[i] = left;
        stereo.right[i] = right;
    }
}

/// Computes biquad coefficients for an EQ band.
fn compute_band_coefficients(band: &EqBand, sample_rate: f64) -> BiquadCoeffs {
    // Clamp parameters to safe ranges
    let frequency = band.frequency.clamp(20.0, sample_rate * 0.45);
    let q = band.q.clamp(0.1, 10.0);
    let gain_db = band.gain_db.clamp(-24.0, 24.0);

    match band.band_type {
        EqBandType::Lowshelf => BiquadCoeffs::low_shelf(frequency, gain_db, sample_rate),
        EqBandType::Highshelf => BiquadCoeffs::high_shelf(frequency, gain_db, sample_rate),
        EqBandType::Peak => BiquadCoeffs::peaking_eq(frequency, q, gain_db, sample_rate),
        EqBandType::Notch => BiquadCoeffs::notch(frequency, q, sample_rate),
    }
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
    fn test_empty_bands_passthrough() {
        let mut stereo = make_stereo_buffer(100, 0.5);
        apply(&mut stereo, &[], 44100.0);

        // Should be unchanged
        assert!((stereo.left[50] - 0.5).abs() < 1e-10);
        assert!((stereo.right[50] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_peak_band_processes() {
        let mut stereo = make_stereo_buffer(1000, 1.0);
        let bands = vec![EqBand {
            frequency: 1000.0,
            gain_db: 6.0,
            q: 2.0,
            band_type: EqBandType::Peak,
        }];

        apply(&mut stereo, &bands, 44100.0);

        // DC input through a peak filter should pass unchanged (peak only affects center freq)
        // After settling, output should approach 1.0
        assert!((stereo.left[999] - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_notch_band_processes() {
        let mut stereo = make_stereo_buffer(1000, 1.0);
        let bands = vec![EqBand {
            frequency: 500.0,
            gain_db: 0.0, // gain_db ignored for notch
            q: 2.0,
            band_type: EqBandType::Notch,
        }];

        apply(&mut stereo, &bands, 44100.0);

        // DC input through notch should pass (notch only removes center freq)
        assert!((stereo.left[999] - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_lowshelf_band_processes() {
        let mut stereo = make_stereo_buffer(1000, 1.0);
        let bands = vec![EqBand {
            frequency: 200.0,
            gain_db: 6.0, // +6dB boost
            q: 1.0,       // Q ignored for shelf
            band_type: EqBandType::Lowshelf,
        }];

        apply(&mut stereo, &bands, 44100.0);

        // DC (0 Hz) should be boosted by the low shelf
        // 6 dB = ~2x amplitude
        let output = stereo.left[999];
        assert!(output > 1.5, "Low shelf should boost DC, got {}", output);
    }

    #[test]
    fn test_highshelf_band_processes() {
        let mut stereo = make_stereo_buffer(1000, 1.0);
        let bands = vec![EqBand {
            frequency: 4000.0,
            gain_db: -6.0, // -6dB cut
            q: 1.0,        // Q ignored for shelf
            band_type: EqBandType::Highshelf,
        }];

        apply(&mut stereo, &bands, 44100.0);

        // DC input should pass unchanged through high shelf cut
        assert!((stereo.left[999] - 1.0).abs() < 0.2);
    }

    #[test]
    fn test_multiple_bands_cascade() {
        let mut stereo = make_stereo_buffer(1000, 1.0);
        let bands = vec![
            EqBand {
                frequency: 100.0,
                gain_db: 3.0,
                q: 1.0,
                band_type: EqBandType::Lowshelf,
            },
            EqBand {
                frequency: 1000.0,
                gain_db: 0.0,
                q: 2.0,
                band_type: EqBandType::Peak,
            },
            EqBand {
                frequency: 8000.0,
                gain_db: -3.0,
                q: 1.0,
                band_type: EqBandType::Highshelf,
            },
        ];

        apply(&mut stereo, &bands, 44100.0);

        // Should process without error/panic
        assert!(stereo.left[999].is_finite());
        assert!(stereo.right[999].is_finite());
    }

    #[test]
    fn test_extreme_parameters_clamped() {
        let mut stereo = make_stereo_buffer(100, 0.5);

        // Extreme values that should be clamped
        let bands = vec![EqBand {
            frequency: 50000.0, // Will be clamped to safe range
            gain_db: 100.0,     // Will be clamped to 24
            q: 100.0,           // Will be clamped to 10
            band_type: EqBandType::Peak,
        }];

        apply(&mut stereo, &bands, 44100.0);

        // Should not panic or produce NaN/Inf
        assert!(stereo.left[99].is_finite());
    }

    #[test]
    fn test_stereo_independence() {
        // Left and right channels with different values
        let mut stereo = StereoOutput {
            left: vec![1.0; 1000],
            right: vec![0.5; 1000],
        };

        let bands = vec![EqBand {
            frequency: 200.0,
            gain_db: 6.0,
            q: 1.0,
            band_type: EqBandType::Lowshelf,
        }];

        apply(&mut stereo, &bands, 44100.0);

        // Both channels should be processed but maintain their ratio relationship
        let left_out = stereo.left[999];
        let right_out = stereo.right[999];

        // Both should be boosted
        assert!(left_out > 1.0);
        assert!(right_out > 0.5);
        // Ratio should be approximately maintained
        assert!((left_out / right_out - 2.0).abs() < 0.2);
    }
}
