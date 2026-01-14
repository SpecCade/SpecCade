//! WAV file loading and processing utilities.

use std::path::Path;

use super::utils::samples_to_bytes;

/// Load a WAV file and return 16-bit PCM bytes (little-endian i16), mono.
///
/// This function:
/// - Loads a WAV file from the specified path
/// - Converts multi-channel audio to mono by averaging channels
/// - Preserves the original sample rate (returned alongside data)
/// - Returns 16-bit PCM data as bytes (little-endian)
///
/// # Arguments
/// * `sample_path` - Absolute path to the WAV file
///
/// # Returns
/// Result containing tuple of (16-bit PCM bytes, original sample rate)
///
/// # Errors
/// Returns an error if:
/// - The file cannot be read
/// - The WAV format is unsupported (non-PCM formats)
/// - The bit depth is not 8, 16, 24, or 32 bits
pub fn load_wav_sample(sample_path: &Path) -> Result<(Vec<u8>, u32), String> {
    // Read the WAV file
    let mut reader = hound::WavReader::open(sample_path)
        .map_err(|e| format!("Failed to open WAV file '{}': {}", sample_path.display(), e))?;

    let spec = reader.spec();

    // Validate format
    if spec.sample_format != hound::SampleFormat::Int {
        return Err(format!(
            "Unsupported WAV format in '{}': only PCM (integer) format is supported, got {:?}",
            sample_path.display(),
            spec.sample_format
        ));
    }

    // Load samples based on bit depth
    let mono_samples: Vec<f64> = match spec.bits_per_sample {
        8 => {
            let samples: Result<Vec<i8>, _> = reader.samples::<i8>().collect();
            let samples = samples.map_err(|e| format!("Failed to read 8-bit samples: {}", e))?;
            convert_to_mono_f64(&samples, spec.channels, spec.bits_per_sample)
        }
        16 => {
            let samples: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
            let samples = samples.map_err(|e| format!("Failed to read 16-bit samples: {}", e))?;
            convert_to_mono_f64(&samples, spec.channels, spec.bits_per_sample)
        }
        24 | 32 => {
            let samples: Result<Vec<i32>, _> = reader.samples::<i32>().collect();
            let samples = samples.map_err(|e| format!("Failed to read 32-bit samples: {}", e))?;
            convert_to_mono_f64(&samples, spec.channels, spec.bits_per_sample)
        }
        _ => {
            return Err(format!(
                "Unsupported bit depth in '{}': {} bits (supported: 8, 16, 24, 32)",
                sample_path.display(),
                spec.bits_per_sample
            ));
        }
    };

    // Convert to 16-bit PCM bytes, preserving original sample rate
    Ok((samples_to_bytes(&mono_samples), spec.sample_rate))
}

/// Convert interleaved multi-channel samples to mono by averaging channels.
///
/// This is a deterministic conversion that averages all channels together.
fn convert_to_mono_f64<T>(samples: &[T], channels: u16, bits_per_sample: u16) -> Vec<f64>
where
    T: Copy + Into<i32>,
{
    if channels == 1 {
        // Already mono, just convert
        return samples
            .iter()
            .map(|&s| normalize_sample(s.into(), bits_per_sample))
            .collect();
    }

    let channels = channels as usize;
    let frame_count = samples.len() / channels;
    let mut mono = Vec::with_capacity(frame_count);

    for frame_idx in 0..frame_count {
        let mut sum = 0i64;
        for ch in 0..channels {
            sum += samples[frame_idx * channels + ch].into() as i64;
        }
        // Average the channels
        let avg = (sum / channels as i64) as i32;
        mono.push(normalize_sample(avg, bits_per_sample));
    }

    mono
}

/// Normalize a sample value to [-1.0, 1.0] range.
///
/// Uses the bit depth to determine the correct normalization range.
fn normalize_sample(sample: i32, bits_per_sample: u16) -> f64 {
    let max_value = match bits_per_sample {
        8 => 128.0,
        16 => 32768.0,
        24 => 8388608.0,
        32 => 2147483648.0,
        _ => 32768.0, // Default to 16-bit for safety
    };

    sample as f64 / max_value
}
