//! XM sample utilities.
//!
//! This module provides helper functions for sample conversion and manipulation
//! specific to the XM format.

use crate::note::calculate_pitch_correction;

/// Convert 8-bit unsigned samples to 16-bit signed samples.
pub fn convert_8bit_to_16bit(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len() * 2);

    for &sample in data {
        // Convert unsigned 8-bit (0-255) to signed 16-bit (-32768 to 32767)
        let s16 = (sample as i16 - 128) * 256;
        output.extend_from_slice(&s16.to_le_bytes());
    }

    output
}

/// Convert f32 samples to 16-bit signed PCM bytes.
pub fn convert_f32_to_i16_bytes(samples: &[f32]) -> Vec<u8> {
    let mut output = Vec::with_capacity(samples.len() * 2);

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_sample = (clamped * 32767.0) as i16;
        output.extend_from_slice(&i16_sample.to_le_bytes());
    }

    output
}

/// Convert f64 samples to 16-bit signed PCM bytes.
pub fn convert_f64_to_i16_bytes(samples: &[f64]) -> Vec<u8> {
    let mut output = Vec::with_capacity(samples.len() * 2);

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_sample = (clamped * 32767.0) as i16;
        output.extend_from_slice(&i16_sample.to_le_bytes());
    }

    output
}

/// Resample audio data using linear interpolation.
///
/// # Arguments
/// * `data` - 16-bit signed PCM bytes (little-endian)
/// * `src_rate` - Source sample rate
/// * `dst_rate` - Destination sample rate
///
/// # Returns
/// Resampled 16-bit signed PCM bytes
pub fn resample_linear(data: &[u8], src_rate: u32, dst_rate: u32) -> Vec<u8> {
    if src_rate == dst_rate {
        return data.to_vec();
    }

    let num_src_samples = data.len() / 2;
    let ratio = src_rate as f64 / dst_rate as f64;
    let num_dst_samples = (num_src_samples as f64 / ratio).ceil() as usize;

    let mut output = Vec::with_capacity(num_dst_samples * 2);

    for i in 0..num_dst_samples {
        let src_pos = i as f64 * ratio;
        let src_idx = src_pos.floor() as usize;
        let frac = src_pos - src_idx as f64;

        let sample = if src_idx + 1 < num_src_samples {
            let s0 = i16::from_le_bytes([data[src_idx * 2], data[src_idx * 2 + 1]]) as f64;
            let s1 =
                i16::from_le_bytes([data[(src_idx + 1) * 2], data[(src_idx + 1) * 2 + 1]]) as f64;
            s0 + (s1 - s0) * frac
        } else if src_idx < num_src_samples {
            i16::from_le_bytes([data[src_idx * 2], data[src_idx * 2 + 1]]) as f64
        } else {
            0.0
        };

        let i16_sample = sample.clamp(-32768.0, 32767.0) as i16;
        output.extend_from_slice(&i16_sample.to_le_bytes());
    }

    output
}

/// Calculate loop points for a sample that should loop seamlessly.
///
/// # Arguments
/// * `num_samples` - Total number of samples
/// * `frequency` - Frequency of the waveform
/// * `sample_rate` - Sample rate
///
/// # Returns
/// Tuple of (loop_start, loop_length) in samples
pub fn calculate_loop_points(num_samples: u32, frequency: f64, sample_rate: u32) -> (u32, u32) {
    // Calculate samples per cycle
    let samples_per_cycle = (sample_rate as f64 / frequency).round() as u32;

    if samples_per_cycle >= num_samples {
        // Sample is shorter than one cycle, loop the whole thing
        (0, num_samples)
    } else {
        // Find the largest number of complete cycles that fit
        let num_cycles = num_samples / samples_per_cycle;
        let loop_length = num_cycles * samples_per_cycle;
        (0, loop_length)
    }
}

/// Get pitch correction values for a sample at a given sample rate.
///
/// # Arguments
/// * `sample_rate` - The sample rate of the audio
///
/// # Returns
/// Tuple of (finetune, relative_note) for XM sample header
pub fn get_xm_pitch_correction(sample_rate: u32) -> (i8, i8) {
    calculate_pitch_correction(sample_rate)
}

/// Normalize audio samples to a target peak level.
///
/// # Arguments
/// * `data` - 16-bit signed PCM bytes (little-endian)
/// * `target_peak` - Target peak level (0.0 to 1.0)
///
/// # Returns
/// Normalized 16-bit signed PCM bytes
pub fn normalize_samples(data: &[u8], target_peak: f64) -> Vec<u8> {
    let num_samples = data.len() / 2;

    // Find current peak
    let mut max_abs: i16 = 0;
    for i in 0..num_samples {
        let sample = i16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
        max_abs = max_abs.max(sample.abs());
    }

    if max_abs == 0 {
        return data.to_vec();
    }

    // Calculate scaling factor
    let target_max = (target_peak * 32767.0) as i16;
    let scale = target_max as f64 / max_abs as f64;

    // Apply scaling
    let mut output = Vec::with_capacity(data.len());
    for i in 0..num_samples {
        let sample = i16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
        let scaled = (sample as f64 * scale).clamp(-32768.0, 32767.0) as i16;
        output.extend_from_slice(&scaled.to_le_bytes());
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_8bit_to_16bit() {
        let input = vec![0, 128, 255];
        let output = convert_8bit_to_16bit(&input);

        assert_eq!(output.len(), 6);

        // 0 -> -32768
        let s0 = i16::from_le_bytes([output[0], output[1]]);
        assert_eq!(s0, -32768);

        // 128 -> 0
        let s1 = i16::from_le_bytes([output[2], output[3]]);
        assert_eq!(s1, 0);

        // 255 -> 32512 (close to max)
        let s2 = i16::from_le_bytes([output[4], output[5]]);
        assert_eq!(s2, 32512);
    }

    #[test]
    fn test_convert_f32_to_i16() {
        let input = vec![0.0f32, 0.5, 1.0, -1.0];
        let output = convert_f32_to_i16_bytes(&input);

        assert_eq!(output.len(), 8);

        let s0 = i16::from_le_bytes([output[0], output[1]]);
        assert_eq!(s0, 0);

        let s2 = i16::from_le_bytes([output[4], output[5]]);
        assert_eq!(s2, 32767);

        let s3 = i16::from_le_bytes([output[6], output[7]]);
        assert_eq!(s3, -32767);
    }

    #[test]
    fn test_calculate_loop_points() {
        let (start, length) = calculate_loop_points(22050, 440.0, 22050);
        // At 440 Hz and 22050 Hz sample rate, samples per cycle = 50.11
        // Should find complete cycles
        assert_eq!(start, 0);
        assert!(length > 0);
        assert!(length <= 22050);
    }

    #[test]
    fn test_normalize_samples() {
        // Create samples with peak at 0.5
        let mut data = Vec::new();
        data.extend_from_slice(&16383i16.to_le_bytes()); // 0.5 * 32767
        data.extend_from_slice(&(-16383i16).to_le_bytes());
        data.extend_from_slice(&0i16.to_le_bytes());

        let normalized = normalize_samples(&data, 1.0);
        let peak = i16::from_le_bytes([normalized[0], normalized[1]]);
        assert_eq!(peak, 32767);
    }
}
