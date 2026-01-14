//! Channel conversion utilities for stereo/mono audio.

/// Convert stereo samples to mono by averaging channels.
///
/// # Arguments
///
/// * `samples` - Interleaved stereo samples (L, R, L, R, ...).
///
/// # Returns
///
/// Mono samples where each output sample is the average of the
/// corresponding left and right input samples.
///
/// # Example
///
/// ```rust
/// use speccade_tests::audio_analysis::stereo_to_mono;
///
/// let stereo = vec![0.5f32, 0.3, 0.8, 0.2]; // L=0.5, R=0.3, L=0.8, R=0.2
/// let mono = stereo_to_mono(&stereo);
/// assert_eq!(mono.len(), 2);
/// assert!((mono[0] - 0.4).abs() < 0.001); // (0.5 + 0.3) / 2
/// assert!((mono[1] - 0.5).abs() < 0.001); // (0.8 + 0.2) / 2
/// ```
pub fn stereo_to_mono(samples: &[f32]) -> Vec<f32> {
    samples
        .chunks_exact(2)
        .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
        .collect()
}

/// Get only the left channel from stereo samples.
///
/// # Arguments
///
/// * `samples` - Interleaved stereo samples (L, R, L, R, ...).
///
/// # Returns
///
/// Only the left channel samples.
pub fn left_channel(samples: &[f32]) -> Vec<f32> {
    samples.chunks_exact(2).map(|chunk| chunk[0]).collect()
}

/// Get only the right channel from stereo samples.
///
/// # Arguments
///
/// * `samples` - Interleaved stereo samples (L, R, L, R, ...).
///
/// # Returns
///
/// Only the right channel samples.
pub fn right_channel(samples: &[f32]) -> Vec<f32> {
    samples.chunks_exact(2).map(|chunk| chunk[1]).collect()
}
