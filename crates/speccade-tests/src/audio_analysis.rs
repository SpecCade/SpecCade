//! Audio signal analysis utilities for testing speccade audio generation.
//!
//! This module provides functions to verify audio output quality beyond simple
//! "not empty" checks. It includes signal analysis functions and WAV parsing helpers.
//!
//! ## Example
//!
//! ```rust,no_run
//! use speccade_tests::audio_analysis::{parse_wav_samples, calculate_rms, is_silent};
//!
//! let wav_data = std::fs::read("output.wav").unwrap();
//! let samples = parse_wav_samples(&wav_data).unwrap();
//!
//! let rms = calculate_rms(&samples);
//! assert!(!is_silent(&samples, 0.001));
//! ```

use std::fmt;

/// Error type for audio analysis operations.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioAnalysisError {
    /// WAV data is too short to contain required headers.
    DataTooShort { expected: usize, actual: usize },
    /// Missing or invalid RIFF header.
    InvalidRiffHeader,
    /// Missing or invalid WAVE format identifier.
    InvalidWaveFormat,
    /// Missing fmt chunk.
    MissingFmtChunk,
    /// Missing data chunk.
    MissingDataChunk,
    /// Unsupported audio format (only PCM supported).
    UnsupportedAudioFormat { format_code: u16 },
    /// Unsupported bits per sample.
    UnsupportedBitsPerSample { bits: u16 },
    /// Invalid or corrupted chunk.
    InvalidChunk { message: String },
}

impl fmt::Display for AudioAnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioAnalysisError::DataTooShort { expected, actual } => {
                write!(f, "WAV data too short: expected at least {} bytes, got {}", expected, actual)
            }
            AudioAnalysisError::InvalidRiffHeader => {
                write!(f, "Invalid or missing RIFF header")
            }
            AudioAnalysisError::InvalidWaveFormat => {
                write!(f, "Invalid or missing WAVE format identifier")
            }
            AudioAnalysisError::MissingFmtChunk => {
                write!(f, "Missing fmt chunk in WAV file")
            }
            AudioAnalysisError::MissingDataChunk => {
                write!(f, "Missing data chunk in WAV file")
            }
            AudioAnalysisError::UnsupportedAudioFormat { format_code } => {
                write!(f, "Unsupported audio format code: {} (only PCM/1 supported)", format_code)
            }
            AudioAnalysisError::UnsupportedBitsPerSample { bits } => {
                write!(f, "Unsupported bits per sample: {} (only 8, 16, 24, 32 supported)", bits)
            }
            AudioAnalysisError::InvalidChunk { message } => {
                write!(f, "Invalid chunk: {}", message)
            }
        }
    }
}

impl std::error::Error for AudioAnalysisError {}

/// WAV file header information.
#[derive(Debug, Clone, PartialEq)]
pub struct WavHeader {
    /// Number of audio channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Sample rate in Hz (e.g., 44100, 48000).
    pub sample_rate: u32,
    /// Bits per sample (8, 16, 24, or 32).
    pub bits_per_sample: u16,
    /// Total number of samples per channel.
    pub num_samples: usize,
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Byte rate (bytes per second).
    pub byte_rate: u32,
    /// Block alignment (bytes per sample frame).
    pub block_align: u16,
}

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

    let sum_of_squares: f64 = samples.iter()
        .map(|&s| (s as f64) * (s as f64))
        .sum();

    ((sum_of_squares / samples.len() as f64).sqrt()) as f32
}

/// Check if audio is silent (all samples near zero).
///
/// # Arguments
///
/// * `samples` - Audio samples normalized to the range [-1.0, 1.0].
/// * `threshold` - Maximum absolute value for a sample to be considered silent.
///                 Typical values: 0.001 for strict silence, 0.01 for near-silence.
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
    samples.iter()
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

    let crossings: usize = samples.windows(2)
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

/// Parse WAV file header and return metadata.
///
/// # Arguments
///
/// * `wav_data` - Raw bytes of a WAV file.
///
/// # Returns
///
/// A `WavHeader` containing the audio format information,
/// or an error if the WAV file is invalid.
///
/// # Example
///
/// ```rust,no_run
/// use speccade_tests::audio_analysis::parse_wav_header;
///
/// let wav_data = std::fs::read("audio.wav").unwrap();
/// let header = parse_wav_header(&wav_data).unwrap();
/// println!("Sample rate: {} Hz", header.sample_rate);
/// println!("Channels: {}", header.channels);
/// println!("Duration: {:.2} seconds", header.duration_secs);
/// ```
pub fn parse_wav_header(wav_data: &[u8]) -> Result<WavHeader, AudioAnalysisError> {
    // Minimum WAV file size: RIFF header (12) + fmt chunk (24) + data header (8)
    if wav_data.len() < 44 {
        return Err(AudioAnalysisError::DataTooShort {
            expected: 44,
            actual: wav_data.len(),
        });
    }

    // Check RIFF header
    if &wav_data[0..4] != b"RIFF" {
        return Err(AudioAnalysisError::InvalidRiffHeader);
    }

    // Check WAVE format
    if &wav_data[8..12] != b"WAVE" {
        return Err(AudioAnalysisError::InvalidWaveFormat);
    }

    // Find fmt chunk
    let (fmt_offset, _fmt_size) = find_chunk(wav_data, b"fmt ")?
        .ok_or(AudioAnalysisError::MissingFmtChunk)?;

    // Parse fmt chunk
    let fmt_data = &wav_data[fmt_offset..];
    if fmt_data.len() < 16 {
        return Err(AudioAnalysisError::InvalidChunk {
            message: "fmt chunk too short".to_string(),
        });
    }

    let audio_format = u16::from_le_bytes([fmt_data[0], fmt_data[1]]);
    let channels = u16::from_le_bytes([fmt_data[2], fmt_data[3]]);
    let sample_rate = u32::from_le_bytes([fmt_data[4], fmt_data[5], fmt_data[6], fmt_data[7]]);
    let byte_rate = u32::from_le_bytes([fmt_data[8], fmt_data[9], fmt_data[10], fmt_data[11]]);
    let block_align = u16::from_le_bytes([fmt_data[12], fmt_data[13]]);
    let bits_per_sample = u16::from_le_bytes([fmt_data[14], fmt_data[15]]);

    // Only support PCM format (1)
    if audio_format != 1 {
        return Err(AudioAnalysisError::UnsupportedAudioFormat {
            format_code: audio_format,
        });
    }

    // Find data chunk
    let (data_offset, data_size) = find_chunk(wav_data, b"data")?
        .ok_or(AudioAnalysisError::MissingDataChunk)?;

    // Calculate number of samples
    let bytes_per_sample = (bits_per_sample / 8) as usize;
    let num_samples = if bytes_per_sample > 0 && channels > 0 {
        data_size / (bytes_per_sample * channels as usize)
    } else {
        0
    };

    let duration_secs = if sample_rate > 0 {
        num_samples as f64 / sample_rate as f64
    } else {
        0.0
    };

    // Verify data chunk has expected size
    if data_offset + data_size > wav_data.len() {
        return Err(AudioAnalysisError::InvalidChunk {
            message: format!(
                "data chunk extends beyond file: offset {} + size {} > file length {}",
                data_offset, data_size, wav_data.len()
            ),
        });
    }

    Ok(WavHeader {
        channels,
        sample_rate,
        bits_per_sample,
        num_samples,
        duration_secs,
        byte_rate,
        block_align,
    })
}

/// Parse WAV file and return samples as f32 normalized to [-1.0, 1.0].
///
/// For stereo audio, samples are interleaved (L, R, L, R, ...).
/// For mono audio, samples are sequential.
///
/// # Arguments
///
/// * `wav_data` - Raw bytes of a WAV file.
///
/// # Returns
///
/// A vector of f32 samples normalized to [-1.0, 1.0],
/// or an error if the WAV file is invalid.
///
/// # Supported Formats
///
/// - 8-bit unsigned PCM
/// - 16-bit signed PCM
/// - 24-bit signed PCM
/// - 32-bit signed PCM
///
/// # Example
///
/// ```rust,no_run
/// use speccade_tests::audio_analysis::{parse_wav_samples, calculate_rms};
///
/// let wav_data = std::fs::read("audio.wav").unwrap();
/// let samples = parse_wav_samples(&wav_data).unwrap();
/// let rms = calculate_rms(&samples);
/// println!("RMS level: {:.4}", rms);
/// ```
pub fn parse_wav_samples(wav_data: &[u8]) -> Result<Vec<f32>, AudioAnalysisError> {
    let header = parse_wav_header(wav_data)?;

    // Find data chunk
    let (data_offset, data_size) = find_chunk(wav_data, b"data")?
        .ok_or(AudioAnalysisError::MissingDataChunk)?;

    let data = &wav_data[data_offset..data_offset + data_size];

    match header.bits_per_sample {
        8 => Ok(parse_8bit_samples(data)),
        16 => Ok(parse_16bit_samples(data)),
        24 => Ok(parse_24bit_samples(data)),
        32 => Ok(parse_32bit_samples(data)),
        _ => Err(AudioAnalysisError::UnsupportedBitsPerSample {
            bits: header.bits_per_sample,
        }),
    }
}

/// Find a chunk in WAV data and return its data offset and size.
fn find_chunk(wav_data: &[u8], chunk_id: &[u8; 4]) -> Result<Option<(usize, usize)>, AudioAnalysisError> {
    let mut offset = 12; // Skip RIFF header

    while offset + 8 <= wav_data.len() {
        let id = &wav_data[offset..offset + 4];
        let size = u32::from_le_bytes([
            wav_data[offset + 4],
            wav_data[offset + 5],
            wav_data[offset + 6],
            wav_data[offset + 7],
        ]) as usize;

        if id == chunk_id {
            return Ok(Some((offset + 8, size)));
        }

        // Move to next chunk (size + 8 for header, aligned to 2 bytes)
        let chunk_total_size = 8 + size + (size % 2);
        offset += chunk_total_size;
    }

    Ok(None)
}

/// Parse 8-bit unsigned PCM samples to f32.
fn parse_8bit_samples(data: &[u8]) -> Vec<f32> {
    data.iter()
        .map(|&b| (b as f32 - 128.0) / 128.0)
        .collect()
}

/// Parse 16-bit signed PCM samples to f32.
fn parse_16bit_samples(data: &[u8]) -> Vec<f32> {
    data.chunks_exact(2)
        .map(|chunk| {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            sample as f32 / 32768.0
        })
        .collect()
}

/// Parse 24-bit signed PCM samples to f32.
fn parse_24bit_samples(data: &[u8]) -> Vec<f32> {
    data.chunks_exact(3)
        .map(|chunk| {
            // Sign-extend 24-bit to 32-bit
            let sample = if chunk[2] & 0x80 != 0 {
                // Negative: sign extend with 0xFF
                i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0xFF])
            } else {
                // Positive: zero extend
                i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0x00])
            };
            sample as f32 / 8388608.0 // 2^23
        })
        .collect()
}

/// Parse 32-bit signed PCM samples to f32.
fn parse_32bit_samples(data: &[u8]) -> Vec<f32> {
    data.chunks_exact(4)
        .map(|chunk| {
            let sample = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            sample as f32 / 2147483648.0 // 2^31
        })
        .collect()
}

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
    samples.chunks_exact(2)
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
    samples.chunks_exact(2)
        .map(|chunk| chunk[0])
        .collect()
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
    samples.chunks_exact(2)
        .map(|chunk| chunk[1])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // RMS Tests
    // ==========================================================================

    #[test]
    fn test_rms_silence() {
        let silence = vec![0.0f32; 100];
        assert_eq!(calculate_rms(&silence), 0.0);
    }

    #[test]
    fn test_rms_constant() {
        let constant = vec![0.5f32; 100];
        assert!((calculate_rms(&constant) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_rms_alternating() {
        // RMS of alternating +1/-1 is 1.0
        let alternating: Vec<f32> = (0..100).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
        assert!((calculate_rms(&alternating) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_rms_sine_wave() {
        // RMS of a sine wave is 1/sqrt(2) of the amplitude
        let sine: Vec<f32> = (0..10000)
            .map(|i| (i as f32 * 2.0 * std::f32::consts::PI / 100.0).sin())
            .collect();
        let expected_rms = 1.0 / std::f32::consts::SQRT_2;
        assert!((calculate_rms(&sine) - expected_rms).abs() < 0.01);
    }

    #[test]
    fn test_rms_empty() {
        let empty: Vec<f32> = vec![];
        assert_eq!(calculate_rms(&empty), 0.0);
    }

    // ==========================================================================
    // Silence Detection Tests
    // ==========================================================================

    #[test]
    fn test_is_silent_true() {
        let silence = vec![0.0001f32; 100];
        assert!(is_silent(&silence, 0.001));
    }

    #[test]
    fn test_is_silent_false() {
        let audio = vec![0.5f32; 100];
        assert!(!is_silent(&audio, 0.001));
    }

    #[test]
    fn test_is_silent_boundary() {
        let at_threshold = vec![0.001f32; 100];
        assert!(is_silent(&at_threshold, 0.001));

        let above_threshold = vec![0.0011f32; 100];
        assert!(!is_silent(&above_threshold, 0.001));
    }

    #[test]
    fn test_is_silent_empty() {
        let empty: Vec<f32> = vec![];
        assert!(is_silent(&empty, 0.001));
    }

    #[test]
    fn test_is_silent_negative() {
        let negative = vec![-0.0001f32; 100];
        assert!(is_silent(&negative, 0.001));
    }

    // ==========================================================================
    // Peak Amplitude Tests
    // ==========================================================================

    #[test]
    fn test_peak_amplitude_basic() {
        let samples = vec![-0.8f32, 0.5, -0.3, 0.9];
        assert_eq!(peak_amplitude(&samples), 0.9);
    }

    #[test]
    fn test_peak_amplitude_negative() {
        let samples = vec![-0.5f32, 0.3, -0.9, 0.2];
        assert_eq!(peak_amplitude(&samples), 0.9);
    }

    #[test]
    fn test_peak_amplitude_empty() {
        let empty: Vec<f32> = vec![];
        assert_eq!(peak_amplitude(&empty), 0.0);
    }

    #[test]
    fn test_peak_amplitude_constant() {
        let constant = vec![0.5f32; 100];
        assert_eq!(peak_amplitude(&constant), 0.5);
    }

    // ==========================================================================
    // Clipping Detection Tests
    // ==========================================================================

    #[test]
    fn test_detect_clipping_clean() {
        let clean = vec![0.5f32, -0.3, 0.8, -0.7];
        assert!(!detect_clipping(&clean));
    }

    #[test]
    fn test_detect_clipping_positive() {
        let clipped = vec![0.5f32, 1.0, 0.3];
        assert!(detect_clipping(&clipped));
    }

    #[test]
    fn test_detect_clipping_negative() {
        let clipped = vec![0.5f32, -1.0, 0.3];
        assert!(detect_clipping(&clipped));
    }

    #[test]
    fn test_detect_clipping_threshold() {
        let near_clip = vec![0.998f32; 100];
        assert!(!detect_clipping(&near_clip));

        let at_clip = vec![0.999f32; 100];
        assert!(detect_clipping(&at_clip));
    }

    #[test]
    fn test_detect_clipping_empty() {
        let empty: Vec<f32> = vec![];
        assert!(!detect_clipping(&empty));
    }

    // ==========================================================================
    // Zero Crossing Rate Tests
    // ==========================================================================

    #[test]
    fn test_zcr_alternating() {
        let alternating = vec![1.0f32, -1.0, 1.0, -1.0, 1.0];
        let zcr = zero_crossing_rate(&alternating);
        assert!((zcr - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_zcr_constant_positive() {
        let constant = vec![1.0f32; 100];
        assert_eq!(zero_crossing_rate(&constant), 0.0);
    }

    #[test]
    fn test_zcr_constant_negative() {
        let constant = vec![-1.0f32; 100];
        assert_eq!(zero_crossing_rate(&constant), 0.0);
    }

    #[test]
    fn test_zcr_sine_wave() {
        // A sine wave crosses zero twice per period
        let sample_rate = 1000.0;
        let freq = 100.0;
        let samples: Vec<f32> = (0..1000)
            .map(|i| (i as f32 / sample_rate * freq * 2.0 * std::f32::consts::PI).sin())
            .collect();

        // Expected: ~200 crossings per 1000 samples for 100Hz at 1000Hz sample rate
        let zcr = zero_crossing_rate(&samples);
        let expected_zcr = 2.0 * freq / sample_rate;
        assert!((zcr - expected_zcr).abs() < 0.05);
    }

    #[test]
    fn test_zcr_empty() {
        let empty: Vec<f32> = vec![];
        assert_eq!(zero_crossing_rate(&empty), 0.0);
    }

    #[test]
    fn test_zcr_single() {
        let single = vec![1.0f32];
        assert_eq!(zero_crossing_rate(&single), 0.0);
    }

    // ==========================================================================
    // Audio Content Detection Tests
    // ==========================================================================

    #[test]
    fn test_has_content_silence() {
        let silence = vec![0.0f32; 1000];
        assert!(!has_audio_content(&silence));
    }

    #[test]
    fn test_has_content_sine_wave() {
        let sine: Vec<f32> = (0..1000)
            .map(|i| (i as f32 * 0.1).sin() * 0.5)
            .collect();
        assert!(has_audio_content(&sine));
    }

    #[test]
    fn test_has_content_dc_offset() {
        // DC offset has no zero crossings
        let dc = vec![0.5f32; 1000];
        assert!(!has_audio_content(&dc));
    }

    #[test]
    fn test_has_content_very_quiet() {
        let quiet: Vec<f32> = (0..1000)
            .map(|i| (i as f32 * 0.1).sin() * 0.0001)
            .collect();
        assert!(!has_audio_content(&quiet));
    }

    #[test]
    fn test_has_content_empty() {
        let empty: Vec<f32> = vec![];
        assert!(!has_audio_content(&empty));
    }

    // ==========================================================================
    // WAV Parsing Tests
    // ==========================================================================

    /// Create a minimal valid WAV file for testing.
    fn create_test_wav(channels: u16, sample_rate: u32, bits_per_sample: u16, samples: &[i16]) -> Vec<u8> {
        let num_samples = samples.len();
        let bytes_per_sample = bits_per_sample as usize / 8;
        let data_size = num_samples * bytes_per_sample;
        let file_size = 36 + data_size;

        let mut wav = Vec::with_capacity(44 + data_size);

        // RIFF header
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(file_size as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        // fmt chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // audio format (PCM)
        wav.extend_from_slice(&channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        let byte_rate = sample_rate * channels as u32 * bytes_per_sample as u32;
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        let block_align = channels * bytes_per_sample as u16;
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data_size as u32).to_le_bytes());

        for &sample in samples {
            wav.extend_from_slice(&sample.to_le_bytes());
        }

        wav
    }

    #[test]
    fn test_parse_wav_header_mono() {
        let wav = create_test_wav(1, 44100, 16, &[0; 100]);
        let header = parse_wav_header(&wav).unwrap();

        assert_eq!(header.channels, 1);
        assert_eq!(header.sample_rate, 44100);
        assert_eq!(header.bits_per_sample, 16);
        assert_eq!(header.num_samples, 100);
    }

    #[test]
    fn test_parse_wav_header_stereo() {
        let wav = create_test_wav(2, 48000, 16, &[0; 200]);
        let header = parse_wav_header(&wav).unwrap();

        assert_eq!(header.channels, 2);
        assert_eq!(header.sample_rate, 48000);
        assert_eq!(header.bits_per_sample, 16);
        assert_eq!(header.num_samples, 100); // 200 bytes / 2 channels / 2 bytes per sample
    }

    #[test]
    fn test_parse_wav_samples_16bit() {
        let raw_samples: Vec<i16> = vec![0, 16384, 32767, -32768, -16384, 0];
        let wav = create_test_wav(1, 44100, 16, &raw_samples);

        let samples = parse_wav_samples(&wav).unwrap();

        assert_eq!(samples.len(), 6);
        assert!((samples[0] - 0.0).abs() < 0.001);
        assert!((samples[1] - 0.5).abs() < 0.001);
        assert!((samples[2] - 1.0).abs() < 0.001);
        assert!((samples[3] - (-1.0)).abs() < 0.001);
        assert!((samples[4] - (-0.5)).abs() < 0.001);
        assert!((samples[5] - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_wav_invalid_riff() {
        let invalid = b"NOTARIFF".to_vec();
        let result = parse_wav_header(&invalid);
        assert!(matches!(result, Err(AudioAnalysisError::DataTooShort { .. })));
    }

    #[test]
    fn test_parse_wav_invalid_wave() {
        let mut wav = create_test_wav(1, 44100, 16, &[0; 10]);
        wav[8..12].copy_from_slice(b"NOTW");

        let result = parse_wav_header(&wav);
        assert!(matches!(result, Err(AudioAnalysisError::InvalidWaveFormat)));
    }

    #[test]
    fn test_parse_wav_too_short() {
        let short = vec![0u8; 20];
        let result = parse_wav_header(&short);
        assert!(matches!(result, Err(AudioAnalysisError::DataTooShort { .. })));
    }

    // ==========================================================================
    // Stereo/Mono Conversion Tests
    // ==========================================================================

    #[test]
    fn test_stereo_to_mono_basic() {
        let stereo = vec![0.5f32, 0.3, 0.8, 0.2, -0.4, 0.4];
        let mono = stereo_to_mono(&stereo);

        assert_eq!(mono.len(), 3);
        assert!((mono[0] - 0.4).abs() < 0.001);
        assert!((mono[1] - 0.5).abs() < 0.001);
        assert!((mono[2] - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_stereo_to_mono_empty() {
        let stereo: Vec<f32> = vec![];
        let mono = stereo_to_mono(&stereo);
        assert!(mono.is_empty());
    }

    #[test]
    fn test_left_channel() {
        let stereo = vec![0.5f32, 0.3, 0.8, 0.2];
        let left = left_channel(&stereo);

        assert_eq!(left.len(), 2);
        assert_eq!(left[0], 0.5);
        assert_eq!(left[1], 0.8);
    }

    #[test]
    fn test_right_channel() {
        let stereo = vec![0.5f32, 0.3, 0.8, 0.2];
        let right = right_channel(&stereo);

        assert_eq!(right.len(), 2);
        assert_eq!(right[0], 0.3);
        assert_eq!(right[1], 0.2);
    }

    // ==========================================================================
    // Error Display Tests
    // ==========================================================================

    #[test]
    fn test_error_display() {
        let err = AudioAnalysisError::DataTooShort { expected: 44, actual: 10 };
        assert!(err.to_string().contains("44"));
        assert!(err.to_string().contains("10"));

        let err = AudioAnalysisError::UnsupportedAudioFormat { format_code: 3 };
        assert!(err.to_string().contains("3"));

        let err = AudioAnalysisError::UnsupportedBitsPerSample { bits: 12 };
        assert!(err.to_string().contains("12"));
    }
}
