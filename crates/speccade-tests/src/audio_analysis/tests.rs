//! Tests for audio analysis functionality.

use super::channel::*;
use super::error::AudioAnalysisError;
use super::signal::*;
use super::wav::*;

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
    let alternating: Vec<f32> = (0..100)
        .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
        .collect();
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
    let sine: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
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
    let quiet: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.1).sin() * 0.0001).collect();
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
fn create_test_wav(
    channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
    samples: &[i16],
) -> Vec<u8> {
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
    assert!(matches!(
        result,
        Err(AudioAnalysisError::DataTooShort { .. })
    ));
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
    assert!(matches!(
        result,
        Err(AudioAnalysisError::DataTooShort { .. })
    ));
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
    let err = AudioAnalysisError::DataTooShort {
        expected: 44,
        actual: 10,
    };
    assert!(err.to_string().contains("44"));
    assert!(err.to_string().contains("10"));

    let err = AudioAnalysisError::UnsupportedAudioFormat { format_code: 3 };
    assert!(err.to_string().contains("3"));

    let err = AudioAnalysisError::UnsupportedBitsPerSample { bits: 12 };
    assert!(err.to_string().contains("12"));
}
