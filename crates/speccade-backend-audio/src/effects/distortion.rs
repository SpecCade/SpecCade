//! Distortion effects: bitcrusher and waveshaper.

use crate::mixer::StereoOutput;
use speccade_spec::recipe::audio::WaveshaperCurve;

/// Applies bitcrush effect to stereo audio.
pub fn apply_bitcrush(stereo: &mut StereoOutput, bits: u8, sample_rate_reduction: f64) {
    let bits = bits.clamp(1, 16);
    let sr_reduction = sample_rate_reduction.max(1.0);

    // Calculate quantization step
    let levels = (1 << bits) as f64; // 2^bits
    let step = 2.0 / levels; // Range is -1.0 to 1.0

    let mut phase = 0.0;

    for i in 0..stereo.left.len() {
        // Sample rate reduction
        if phase >= 1.0 {
            phase -= 1.0;

            // Bit depth reduction (quantization)
            stereo.left[i] = quantize(stereo.left[i], step);
            stereo.right[i] = quantize(stereo.right[i], step);
        } else if i > 0 {
            // Hold previous sample
            stereo.left[i] = stereo.left[i - 1];
            stereo.right[i] = stereo.right[i - 1];
        }

        phase += 1.0 / sr_reduction;
    }
}

/// Quantizes a sample to a specific bit depth.
fn quantize(sample: f64, step: f64) -> f64 {
    (sample / step).round() * step
}

/// Applies waveshaper distortion to stereo audio.
pub fn apply_waveshaper(stereo: &mut StereoOutput, drive: f64, curve: &WaveshaperCurve, wet: f64) {
    // Create a constant drive curve for the non-modulated case
    let num_samples = stereo.left.len();
    let drive_curve = vec![drive; num_samples];
    apply_waveshaper_with_modulation(stereo, &drive_curve, curve, wet);
}

/// Applies waveshaper distortion to stereo audio with per-sample drive modulation.
///
/// # Arguments
/// * `stereo` - Stereo audio to process
/// * `drive_curve` - Per-sample drive values (clamped to 1.0-100.0)
/// * `curve` - Waveshaping curve type
/// * `wet` - Wet/dry mix (0.0-1.0)
pub fn apply_waveshaper_with_modulation(
    stereo: &mut StereoOutput,
    drive_curve: &[f64],
    curve: &WaveshaperCurve,
    wet: f64,
) {
    let wet = wet.clamp(0.0, 1.0);
    let dry = 1.0 - wet;

    for i in 0..stereo.left.len() {
        let in_left = stereo.left[i];
        let in_right = stereo.right[i];

        // Get modulated drive for this sample
        let drive = drive_curve.get(i).copied().unwrap_or(1.0).clamp(1.0, 100.0);

        // Apply drive
        let driven_left = in_left * drive;
        let driven_right = in_right * drive;

        // Apply waveshaping curve
        let shaped_left = apply_curve(driven_left, curve);
        let shaped_right = apply_curve(driven_right, curve);

        // Apply makeup gain (compensate for drive)
        let makeup = 1.0 / drive.sqrt();
        let out_left = shaped_left * makeup;
        let out_right = shaped_right * makeup;

        // Mix wet/dry
        stereo.left[i] = in_left * dry + out_left * wet;
        stereo.right[i] = in_right * dry + out_right * wet;
    }
}

/// Applies a waveshaping curve to a sample.
fn apply_curve(sample: f64, curve: &WaveshaperCurve) -> f64 {
    match curve {
        WaveshaperCurve::Tanh => sample.tanh(),
        WaveshaperCurve::SoftClip => {
            // Soft clipping using cubic polynomial
            if sample > 1.0 {
                2.0 / 3.0
            } else if sample < -1.0 {
                -2.0 / 3.0
            } else {
                sample - (sample * sample * sample) / 3.0
            }
        }
        WaveshaperCurve::HardClip => sample.clamp(-1.0, 1.0),
        WaveshaperCurve::Sine => {
            // Sine waveshaping (wraps around)
            use std::f64::consts::PI;
            (sample * PI / 2.0).sin()
        }
    }
}
