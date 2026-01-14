//! Signal analysis functions for audio testing.

/// Calculate RMS (Root Mean Square) of audio samples.
///
/// RMS is a measure of the average power of the audio signal.
/// A higher RMS indicates louder audio.
///
/// # Arguments
///
/// * `samples` - Audio samples normalized to the range [-1.0, 1.0].
///
/// # Returns
///
/// The RMS value, typically in the range [0.0, 1.0] for normalized audio.
/// Returns 0.0 for empty input.
///
/// # Example
///
/// ```rust
/// use speccade_tests::audio_analysis::calculate_rms;
///
/// let silence = vec![0.0f32; 100];
/// assert_eq!(calculate_rms(&silence), 0.0);
///
/// let loud = vec![1.0f32; 100];
/// assert_eq!(calculate_rms(&loud), 1.0);
/// ```
pub fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_of_squares: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();

    ((sum_of_squares / samples.len() as f64).sqrt()) as f32
}

/// Check if audio is silent (all samples near zero).
///
/// # Arguments
///
/// * `samples` - Audio samples normalized to the range [-1.0, 1.0].
/// * `threshold` - Maximum absolute value for a sample to be considered silent.
///   Typical values: 0.001 for strict silence, 0.01 for near-silence.
///
/// # Returns
///
/// `true` if all samples are below the threshold, `false` otherwise.
/// Returns `true` for empty input.
///
/// # Example
///
/// ```rust
/// use speccade_tests::audio_analysis::is_silent;
///
/// let silence = vec![0.0001f32; 100];
/// assert!(is_silent(&silence, 0.001));
///
/// let audio = vec![0.5f32; 100];
/// assert!(!is_silent(&audio, 0.001));
/// ```
pub fn is_silent(samples: &[f32], threshold: f32) -> bool {
    samples.iter().all(|&s| s.abs() <= threshold)
}

/// Calculate peak amplitude (maximum absolute sample value).
///
/// # Arguments
///
/// * `samples` - Audio samples normalized to the range [-1.0, 1.0].
///
/// # Returns
///
/// The maximum absolute sample value. Returns 0.0 for empty input.
///
/// # Example
///
/// ```rust
/// use speccade_tests::audio_analysis::peak_amplitude;
///
/// let samples = vec![-0.8f32, 0.5, -0.3, 0.9];
/// assert_eq!(peak_amplitude(&samples), 0.9);
/// ```
pub fn peak_amplitude(samples: &[f32]) -> f32 {
    samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0f32, |max, s| max.max(s))
}

/// Detect if audio is clipped (samples at max/min values).
///
/// Clipping occurs when audio exceeds the maximum representable value,
/// resulting in distortion. This function checks for samples at or very
/// close to the [-1.0, 1.0] boundaries.
///
/// # Arguments
///
/// * `samples` - Audio samples normalized to the range [-1.0, 1.0].
///
/// # Returns
///
/// `true` if any samples are clipped (|sample| >= 0.999), `false` otherwise.
///
/// # Example
///
/// ```rust
/// use speccade_tests::audio_analysis::detect_clipping;
///
/// let clean = vec![0.5f32, -0.3, 0.8];
/// assert!(!detect_clipping(&clean));
///
/// let clipped = vec![0.5f32, 1.0, -1.0];
/// assert!(detect_clipping(&clipped));
/// ```
pub fn detect_clipping(samples: &[f32]) -> bool {
    const CLIPPING_THRESHOLD: f32 = 0.999;
    samples.iter().any(|&s| s.abs() >= CLIPPING_THRESHOLD)
}

/// Count zero crossings and return the rate per sample.
///
/// Zero crossing rate is useful for basic pitch estimation and
/// distinguishing between different types of audio content.
/// Higher rates typically indicate higher frequency content.
///
/// # Arguments
///
/// * `samples` - Audio samples normalized to the range [-1.0, 1.0].
///
/// # Returns
///
/// The zero crossing rate as crossings per sample (range [0.0, 1.0]).
/// Returns 0.0 for empty or single-sample input.
///
/// # Example
///
/// ```rust
/// use speccade_tests::audio_analysis::zero_crossing_rate;
///
/// // Alternating samples have maximum zero crossing rate
/// let alternating = vec![1.0f32, -1.0, 1.0, -1.0];
/// assert!(zero_crossing_rate(&alternating) > 0.9);
///
/// // Constant signal has no zero crossings
/// let constant = vec![1.0f32; 100];
/// assert_eq!(zero_crossing_rate(&constant), 0.0);
/// ```
pub fn zero_crossing_rate(samples: &[f32]) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }

    let crossings: usize = samples
        .windows(2)
        .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
        .count();

    crossings as f32 / (samples.len() - 1) as f32
}

/// Simple check for whether audio has meaningful content.
///
/// This combines multiple metrics to determine if the audio
/// contains actual sound rather than silence or noise.
///
/// # Arguments
///
/// * `samples` - Audio samples normalized to the range [-1.0, 1.0].
///
/// # Returns
///
/// `true` if the audio appears to have meaningful content:
/// - RMS above 0.001 (not effectively silent)
/// - Peak amplitude above 0.01 (has some signal)
/// - Has at least some zero crossings (not DC offset)
///
/// # Example
///
/// ```rust
/// use speccade_tests::audio_analysis::has_audio_content;
///
/// let silence = vec![0.0f32; 1000];
/// assert!(!has_audio_content(&silence));
///
/// // Generate a simple sine wave
/// let sine: Vec<f32> = (0..1000)
///     .map(|i| (i as f32 * 0.1).sin() * 0.5)
///     .collect();
/// assert!(has_audio_content(&sine));
/// ```
pub fn has_audio_content(samples: &[f32]) -> bool {
    if samples.is_empty() {
        return false;
    }

    let rms = calculate_rms(samples);
    let peak = peak_amplitude(samples);
    let zcr = zero_crossing_rate(samples);

    // Audio should have:
    // 1. Some RMS level (not silence)
    // 2. Some peak amplitude
    // 3. Some zero crossings (indicating actual oscillation, not DC)
    rms > 0.001 && peak > 0.01 && zcr > 0.0001
}
