//! Tests for the WAV writer module.

use crate::mixer::StereoOutput;

use super::builder::WavWriter;
use super::format::WavFormat;
use super::pcm::{compute_pcm_hash, extract_pcm_data};
use super::result::WavResult;
use super::writer::{samples_to_pcm16, stereo_to_pcm16, write_wav, write_wav_to_vec};

// =========================================================================
// WavFormat construction tests
// =========================================================================

#[test]
fn test_wav_format_mono() {
    let format = WavFormat::mono(44100);
    assert_eq!(format.channels, 1);
    assert_eq!(format.sample_rate, 44100);
    assert_eq!(format.bits_per_sample, 16);
}

#[test]
fn test_wav_format_stereo() {
    let format = WavFormat::stereo(48000);
    assert_eq!(format.channels, 2);
    assert_eq!(format.sample_rate, 48000);
    assert_eq!(format.bits_per_sample, 16);
}

#[test]
fn test_wav_format_various_sample_rates() {
    // Test common sample rates
    for &rate in &[8000, 11025, 22050, 44100, 48000, 96000, 192000] {
        let mono = WavFormat::mono(rate);
        assert_eq!(mono.sample_rate, rate);

        let stereo = WavFormat::stereo(rate);
        assert_eq!(stereo.sample_rate, rate);
    }
}

// =========================================================================
// Bytes calculation tests
// =========================================================================

#[test]
fn test_bytes_per_sample() {
    let mono = WavFormat::mono(44100);
    assert_eq!(mono.bytes_per_sample(), 2); // 16 bits / 8 = 2 bytes

    let stereo = WavFormat::stereo(44100);
    assert_eq!(stereo.bytes_per_sample(), 2); // Still 2 bytes per sample per channel
}

#[test]
fn test_block_align() {
    let mono = WavFormat::mono(44100);
    assert_eq!(mono.block_align(), 2); // 1 channel * 2 bytes

    let stereo = WavFormat::stereo(44100);
    assert_eq!(stereo.block_align(), 4); // 2 channels * 2 bytes
}

#[test]
fn test_byte_rate() {
    let mono = WavFormat::mono(44100);
    // 44100 samples/sec * 1 channel * 2 bytes/sample = 88200 bytes/sec
    assert_eq!(mono.byte_rate(), 88200);

    let stereo = WavFormat::stereo(44100);
    // 44100 samples/sec * 2 channels * 2 bytes/sample = 176400 bytes/sec
    assert_eq!(stereo.byte_rate(), 176400);

    // Test with 48kHz
    let stereo_48k = WavFormat::stereo(48000);
    // 48000 * 2 * 2 = 192000
    assert_eq!(stereo_48k.byte_rate(), 192000);
}

#[test]
fn test_wav_format_combined() {
    let mono = WavFormat::mono(44100);
    assert_eq!(mono.channels, 1);
    assert_eq!(mono.sample_rate, 44100);
    assert_eq!(mono.byte_rate(), 88200);
    assert_eq!(mono.block_align(), 2);

    let stereo = WavFormat::stereo(44100);
    assert_eq!(stereo.channels, 2);
    assert_eq!(stereo.byte_rate(), 176400);
    assert_eq!(stereo.block_align(), 4);
}

// =========================================================================
// PCM conversion tests
// =========================================================================

#[test]
fn test_samples_to_pcm16_normal_range() {
    let samples = vec![0.0, 0.5, -0.5, 0.25, -0.25];
    let pcm = samples_to_pcm16(&samples);

    assert_eq!(pcm.len(), 10); // 5 samples * 2 bytes

    // 0.0 should be 0
    assert_eq!(i16::from_le_bytes([pcm[0], pcm[1]]), 0);
    // 0.5 should be approximately 16384 (32767 * 0.5)
    let val_0_5 = i16::from_le_bytes([pcm[2], pcm[3]]);
    assert_eq!(val_0_5, 16384); // (0.5 * 32767).round() = 16384
                                // -0.5 should be approximately -16384
    let val_neg_0_5 = i16::from_le_bytes([pcm[4], pcm[5]]);
    assert_eq!(val_neg_0_5, -16384);
}

#[test]
fn test_samples_to_pcm16_boundary_values() {
    let samples = vec![1.0, -1.0];
    let pcm = samples_to_pcm16(&samples);

    // 1.0 should be 32767 (i16::MAX)
    assert_eq!(i16::from_le_bytes([pcm[0], pcm[1]]), 32767);
    // -1.0 should be -32767 (not i16::MIN because -1.0 * 32767 = -32767)
    assert_eq!(i16::from_le_bytes([pcm[2], pcm[3]]), -32767);
}

#[test]
fn test_samples_to_pcm16_clipping_positive() {
    // Values > 1.0 should clip to i16::MAX (32767)
    let samples = vec![1.5, 2.0, 10.0, 100.0, f64::MAX];
    let pcm = samples_to_pcm16(&samples);

    for i in 0..5 {
        let val = i16::from_le_bytes([pcm[i * 2], pcm[i * 2 + 1]]);
        assert_eq!(val, 32767, "Sample {} should be clipped to 32767", i);
    }
}

#[test]
fn test_samples_to_pcm16_clipping_negative() {
    // Values < -1.0 should clip to -32767
    let samples = vec![-1.5, -2.0, -10.0, -100.0, f64::MIN];
    let pcm = samples_to_pcm16(&samples);

    for i in 0..5 {
        let val = i16::from_le_bytes([pcm[i * 2], pcm[i * 2 + 1]]);
        assert_eq!(val, -32767, "Sample {} should be clipped to -32767", i);
    }
}

#[test]
fn test_samples_to_pcm16_precision() {
    // Test specific values to verify rounding behavior
    let samples = vec![0.0001, -0.0001, 0.9999, -0.9999];
    let pcm = samples_to_pcm16(&samples);

    // 0.0001 * 32767 = 3.2767 -> rounds to 3
    assert_eq!(i16::from_le_bytes([pcm[0], pcm[1]]), 3);
    // -0.0001 * 32767 = -3.2767 -> rounds to -3
    assert_eq!(i16::from_le_bytes([pcm[2], pcm[3]]), -3);
    // 0.9999 * 32767 = 32763.7233 -> rounds to 32764
    assert_eq!(i16::from_le_bytes([pcm[4], pcm[5]]), 32764);
    // -0.9999 * 32767 = -32763.7233 -> rounds to -32764
    assert_eq!(i16::from_le_bytes([pcm[6], pcm[7]]), -32764);
}

#[test]
fn test_stereo_to_pcm16() {
    let left = vec![0.5, -0.5];
    let right = vec![-0.5, 0.5];
    let pcm = stereo_to_pcm16(&left, &right);

    assert_eq!(pcm.len(), 8); // 2 samples * 2 channels * 2 bytes

    // First frame: left=0.5, right=-0.5
    assert_eq!(i16::from_le_bytes([pcm[0], pcm[1]]), 16384); // left
    assert_eq!(i16::from_le_bytes([pcm[2], pcm[3]]), -16384); // right

    // Second frame: left=-0.5, right=0.5
    assert_eq!(i16::from_le_bytes([pcm[4], pcm[5]]), -16384); // left
    assert_eq!(i16::from_le_bytes([pcm[6], pcm[7]]), 16384); // right
}

#[test]
fn test_stereo_to_pcm16_mismatched_lengths() {
    // When lengths don't match, should use minimum length
    let left = vec![0.5, 0.3, 0.1];
    let right = vec![-0.5, -0.3];
    let pcm = stereo_to_pcm16(&left, &right);

    // Should only produce 2 frames (8 bytes), not 3
    assert_eq!(pcm.len(), 8);
}

#[test]
fn test_stereo_to_pcm16_clipping() {
    let left = vec![2.0, -2.0];
    let right = vec![-2.0, 2.0];
    let pcm = stereo_to_pcm16(&left, &right);

    // All values should be clipped
    assert_eq!(i16::from_le_bytes([pcm[0], pcm[1]]), 32767);
    assert_eq!(i16::from_le_bytes([pcm[2], pcm[3]]), -32767);
    assert_eq!(i16::from_le_bytes([pcm[4], pcm[5]]), -32767);
    assert_eq!(i16::from_le_bytes([pcm[6], pcm[7]]), 32767);
}

// =========================================================================
// WAV header correctness tests
// =========================================================================

#[test]
fn test_wav_header_riff_magic() {
    let format = WavFormat::mono(44100);
    let samples = vec![0.0; 10];
    let pcm = samples_to_pcm16(&samples);
    let wav = write_wav_to_vec(&format, &pcm);

    assert_eq!(&wav[0..4], b"RIFF", "RIFF magic number");
    assert_eq!(&wav[8..12], b"WAVE", "WAVE format identifier");
}

#[test]
fn test_wav_header_fmt_chunk() {
    let format = WavFormat::mono(44100);
    let samples = vec![0.0; 10];
    let pcm = samples_to_pcm16(&samples);
    let wav = write_wav_to_vec(&format, &pcm);

    // fmt chunk identifier
    assert_eq!(&wav[12..16], b"fmt ");

    // fmt chunk size (16 for PCM)
    let fmt_size = u32::from_le_bytes([wav[16], wav[17], wav[18], wav[19]]);
    assert_eq!(fmt_size, 16);

    // Audio format (1 = PCM)
    let audio_format = u16::from_le_bytes([wav[20], wav[21]]);
    assert_eq!(audio_format, 1);

    // Channels
    let channels = u16::from_le_bytes([wav[22], wav[23]]);
    assert_eq!(channels, 1);

    // Sample rate
    let sample_rate = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
    assert_eq!(sample_rate, 44100);

    // Byte rate
    let byte_rate = u32::from_le_bytes([wav[28], wav[29], wav[30], wav[31]]);
    assert_eq!(byte_rate, 88200);

    // Block align
    let block_align = u16::from_le_bytes([wav[32], wav[33]]);
    assert_eq!(block_align, 2);

    // Bits per sample
    let bits_per_sample = u16::from_le_bytes([wav[34], wav[35]]);
    assert_eq!(bits_per_sample, 16);
}

#[test]
fn test_wav_header_data_chunk() {
    let format = WavFormat::mono(44100);
    let samples = vec![0.0; 10];
    let pcm = samples_to_pcm16(&samples);
    let wav = write_wav_to_vec(&format, &pcm);

    // data chunk identifier
    assert_eq!(&wav[36..40], b"data");

    // data chunk size
    let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
    assert_eq!(data_size, 20); // 10 samples * 2 bytes
}

#[test]
fn test_wav_header_file_size() {
    let format = WavFormat::mono(44100);
    let samples = vec![0.0; 100];
    let pcm = samples_to_pcm16(&samples);
    let wav = write_wav_to_vec(&format, &pcm);

    // File size field (bytes 4-7) = total size - 8
    let file_size = u32::from_le_bytes([wav[4], wav[5], wav[6], wav[7]]);
    assert_eq!(file_size, wav.len() as u32 - 8);

    // Total file size should be 44 (header) + 200 (data) = 244
    assert_eq!(wav.len(), 244);
}

#[test]
fn test_wav_header_stereo_format() {
    let format = WavFormat::stereo(48000);
    let pcm = stereo_to_pcm16(&[0.5; 50], &[-0.5; 50]);
    let wav = write_wav_to_vec(&format, &pcm);

    // Channels should be 2
    let channels = u16::from_le_bytes([wav[22], wav[23]]);
    assert_eq!(channels, 2);

    // Sample rate should be 48000
    let sample_rate = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
    assert_eq!(sample_rate, 48000);

    // Byte rate should be 48000 * 2 * 2 = 192000
    let byte_rate = u32::from_le_bytes([wav[28], wav[29], wav[30], wav[31]]);
    assert_eq!(byte_rate, 192000);

    // Block align should be 4 (2 channels * 2 bytes)
    let block_align = u16::from_le_bytes([wav[32], wav[33]]);
    assert_eq!(block_align, 4);
}

// =========================================================================
// Determinism tests
// =========================================================================

#[test]
fn test_wav_determinism() {
    let samples = vec![0.5, -0.5, 0.0, 0.25, -0.25];
    let format = WavFormat::mono(44100);
    let pcm = samples_to_pcm16(&samples);

    let wav1 = write_wav_to_vec(&format, &pcm);
    let wav2 = write_wav_to_vec(&format, &pcm);

    assert_eq!(wav1, wav2, "WAV output should be deterministic");
}

#[test]
fn test_wav_determinism_stereo() {
    let left = vec![0.5, -0.5, 0.3];
    let right = vec![-0.5, 0.5, -0.3];
    let format = WavFormat::stereo(44100);
    let pcm = stereo_to_pcm16(&left, &right);

    let wav1 = write_wav_to_vec(&format, &pcm);
    let wav2 = write_wav_to_vec(&format, &pcm);

    assert_eq!(wav1, wav2, "Stereo WAV output should be deterministic");
}

#[test]
fn test_pcm_hash_determinism() {
    let writer = WavWriter::mono(44100);
    let samples = vec![0.5, -0.5, 0.3, -0.3, 0.0];

    let hash1 = writer.pcm_hash_mono(&samples);
    let hash2 = writer.pcm_hash_mono(&samples);

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64); // BLAKE3 produces 64 hex chars
}

#[test]
fn test_pcm_hash_determinism_stereo() {
    let writer = WavWriter::stereo(44100);
    let left = vec![0.5, -0.5, 0.3];
    let right = vec![-0.5, 0.5, -0.3];

    let hash1 = writer.pcm_hash_stereo(&left, &right);
    let hash2 = writer.pcm_hash_stereo(&left, &right);

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64);
}

#[test]
fn test_pcm_hash_different_for_different_samples() {
    let writer = WavWriter::mono(44100);
    let samples1 = vec![0.5, -0.5, 0.3];
    let samples2 = vec![0.5, -0.5, 0.31]; // Slightly different

    let hash1 = writer.pcm_hash_mono(&samples1);
    let hash2 = writer.pcm_hash_mono(&samples2);

    assert_ne!(
        hash1, hash2,
        "Different samples should produce different hashes"
    );
}

#[test]
fn test_compute_pcm_hash_matches_direct_hash() {
    let writer = WavWriter::mono(44100);
    let samples = vec![0.5, -0.5, 0.3, -0.3, 0.0];
    let wav = writer.write_mono(&samples);

    let hash_from_wav = compute_pcm_hash(&wav).expect("should compute hash");
    let hash_direct = writer.pcm_hash_mono(&samples);

    assert_eq!(hash_from_wav, hash_direct);
}

// =========================================================================
// Edge case tests
// =========================================================================

#[test]
fn test_empty_audio() {
    let samples: Vec<f64> = vec![];
    let format = WavFormat::mono(44100);
    let pcm = samples_to_pcm16(&samples);
    let wav = write_wav_to_vec(&format, &pcm);

    // Header should still be valid
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");

    // Data size should be 0
    let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
    assert_eq!(data_size, 0);

    // Total size should be 44 bytes (header only)
    assert_eq!(wav.len(), 44);
}

#[test]
fn test_single_sample() {
    let samples = vec![0.5];
    let format = WavFormat::mono(44100);
    let pcm = samples_to_pcm16(&samples);
    let wav = write_wav_to_vec(&format, &pcm);

    // Data size should be 2 bytes
    let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
    assert_eq!(data_size, 2);

    // Verify the single sample value
    let sample_value = i16::from_le_bytes([wav[44], wav[45]]);
    assert_eq!(sample_value, 16384); // 0.5 * 32767 rounded
}

#[test]
fn test_very_long_audio() {
    // Test with 10 seconds of audio at 44100Hz = 441000 samples
    let num_samples = 441000;
    let samples: Vec<f64> = (0..num_samples)
        .map(|i| (i as f64 * 0.001).sin()) // Simple sine-like pattern
        .collect();

    let format = WavFormat::mono(44100);
    let pcm = samples_to_pcm16(&samples);
    let wav = write_wav_to_vec(&format, &pcm);

    // Data size should be 441000 * 2 = 882000 bytes
    let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
    assert_eq!(data_size, 882000);

    // Total size should be 44 + 882000 = 882044 bytes
    assert_eq!(wav.len(), 882044);

    // Verify header is still correct
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
}

#[test]
fn test_empty_stereo_audio() {
    let left: Vec<f64> = vec![];
    let right: Vec<f64> = vec![];
    let pcm = stereo_to_pcm16(&left, &right);

    assert_eq!(pcm.len(), 0);

    let format = WavFormat::stereo(44100);
    let wav = write_wav_to_vec(&format, &pcm);

    // Data size should be 0
    let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
    assert_eq!(data_size, 0);
}

#[test]
fn test_single_stereo_frame() {
    let left = vec![0.5];
    let right = vec![-0.5];
    let pcm = stereo_to_pcm16(&left, &right);

    assert_eq!(pcm.len(), 4); // 1 frame * 2 channels * 2 bytes

    let format = WavFormat::stereo(44100);
    let wav = write_wav_to_vec(&format, &pcm);

    // Data size should be 4 bytes
    let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
    assert_eq!(data_size, 4);
}

// =========================================================================
// WavWriter tests
// =========================================================================

#[test]
fn test_wav_writer_mono() {
    let writer = WavWriter::mono(44100);
    let samples = vec![0.0; 100];
    let wav = writer.write_mono(&samples);

    // Check RIFF header
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert_eq!(&wav[12..16], b"fmt ");
    assert_eq!(&wav[36..40], b"data");

    // Data should be 200 bytes (100 samples * 2 bytes)
    let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
    assert_eq!(data_size, 200);
}

#[test]
fn test_wav_writer_stereo() {
    let writer = WavWriter::stereo(44100);
    let left = vec![0.5; 100];
    let right = vec![-0.5; 100];
    let wav = writer.write_stereo(&left, &right);

    // Data should be 400 bytes (100 samples * 2 channels * 2 bytes)
    let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
    assert_eq!(data_size, 400);

    // Check channel count
    let channels = u16::from_le_bytes([wav[22], wav[23]]);
    assert_eq!(channels, 2);
}

#[test]
fn test_wav_writer_pcm_hash() {
    let writer = WavWriter::mono(44100);
    let samples = vec![0.5, -0.5, 0.3, -0.3, 0.0];
    let hash = writer.pcm_hash_mono(&samples);

    assert_eq!(hash.len(), 64); // BLAKE3 produces 64 hex chars
}

#[test]
fn test_wav_writer_stereo_output() {
    let writer = WavWriter::stereo(44100);
    let stereo = StereoOutput {
        left: vec![0.5, 0.3, 0.1],
        right: vec![-0.5, -0.3, -0.1],
    };
    let wav = writer.write_stereo_output(&stereo);

    // Data should be 12 bytes (3 samples * 2 channels * 2 bytes)
    let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
    assert_eq!(data_size, 12);
}

// =========================================================================
// WavResult tests
// =========================================================================

#[test]
fn test_wav_result_mono() {
    let samples = vec![0.5, -0.5, 0.3, -0.3];
    let result = WavResult::from_mono(&samples, 44100);

    assert!(!result.is_stereo);
    assert_eq!(result.sample_rate, 44100);
    assert_eq!(result.num_samples, 4);
    assert_eq!(result.pcm_hash.len(), 64);
    assert_eq!(result.wav_data.len(), 44 + 8); // Header + 4 samples * 2 bytes
}

#[test]
fn test_wav_result_stereo() {
    let left = vec![0.5, -0.5, 0.3];
    let right = vec![-0.5, 0.5, -0.3];
    let result = WavResult::from_stereo(&left, &right, 48000);

    assert!(result.is_stereo);
    assert_eq!(result.sample_rate, 48000);
    assert_eq!(result.num_samples, 3);
    assert_eq!(result.pcm_hash.len(), 64);
    assert_eq!(result.wav_data.len(), 44 + 12); // Header + 3 frames * 2 channels * 2 bytes
}

#[test]
fn test_wav_result_from_stereo_output() {
    let stereo = StereoOutput {
        left: vec![0.5, 0.3],
        right: vec![-0.5, -0.3],
    };
    let result = WavResult::from_stereo_output(&stereo, 44100);

    assert!(result.is_stereo);
    assert_eq!(result.num_samples, 2);
}

#[test]
fn test_wav_result_duration_seconds() {
    let samples = vec![0.0; 44100]; // 1 second at 44100Hz
    let result = WavResult::from_mono(&samples, 44100);

    assert!((result.duration_seconds() - 1.0).abs() < 0.0001);

    let samples_2sec = vec![0.0; 88200]; // 2 seconds
    let result_2sec = WavResult::from_mono(&samples_2sec, 44100);

    assert!((result_2sec.duration_seconds() - 2.0).abs() < 0.0001);
}

#[test]
fn test_wav_result_duration_fractional() {
    // 22050 samples at 44100Hz = 0.5 seconds
    let samples = vec![0.0; 22050];
    let result = WavResult::from_mono(&samples, 44100);

    assert!((result.duration_seconds() - 0.5).abs() < 0.0001);
}

// =========================================================================
// Extract PCM data tests
// =========================================================================

#[test]
fn test_extract_pcm_data() {
    let writer = WavWriter::mono(44100);
    let samples = vec![0.5; 100];
    let wav = writer.write_mono(&samples);

    let pcm = extract_pcm_data(&wav).expect("should extract PCM");
    assert_eq!(pcm.len(), 200); // 100 samples * 2 bytes
}

#[test]
fn test_extract_pcm_data_stereo() {
    let writer = WavWriter::stereo(44100);
    let left = vec![0.5; 50];
    let right = vec![-0.5; 50];
    let wav = writer.write_stereo(&left, &right);

    let pcm = extract_pcm_data(&wav).expect("should extract PCM");
    assert_eq!(pcm.len(), 200); // 50 frames * 2 channels * 2 bytes
}

#[test]
fn test_extract_pcm_data_invalid_too_short() {
    let short_data = vec![0u8; 30]; // Too short for WAV header
    assert!(extract_pcm_data(&short_data).is_none());
}

#[test]
fn test_extract_pcm_data_invalid_no_riff() {
    let mut invalid = vec![0u8; 100];
    invalid[0..4].copy_from_slice(b"XXXX"); // Invalid magic
    assert!(extract_pcm_data(&invalid).is_none());
}

#[test]
fn test_extract_pcm_data_invalid_no_wave() {
    let mut invalid = vec![0u8; 100];
    invalid[0..4].copy_from_slice(b"RIFF");
    invalid[8..12].copy_from_slice(b"XXXX"); // Invalid format
    assert!(extract_pcm_data(&invalid).is_none());
}

// =========================================================================
// Clipping tests
// =========================================================================

#[test]
fn test_clipping_positive() {
    let samples = vec![2.0, -2.0]; // Out of range
    let pcm = samples_to_pcm16(&samples);

    // Should be clipped to max/min values
    let val1 = i16::from_le_bytes([pcm[0], pcm[1]]);
    let val2 = i16::from_le_bytes([pcm[2], pcm[3]]);

    assert_eq!(val1, 32767);
    assert_eq!(val2, -32767);
}

#[test]
fn test_clipping_extreme_values() {
    let samples = vec![1000.0, -1000.0, f64::INFINITY, f64::NEG_INFINITY];
    let pcm = samples_to_pcm16(&samples);

    // All should be clipped
    assert_eq!(i16::from_le_bytes([pcm[0], pcm[1]]), 32767);
    assert_eq!(i16::from_le_bytes([pcm[2], pcm[3]]), -32767);
    // Note: Infinity.clamp(-1.0, 1.0) = 1.0, so these should also be 32767/-32767
    assert_eq!(i16::from_le_bytes([pcm[4], pcm[5]]), 32767);
    assert_eq!(i16::from_le_bytes([pcm[6], pcm[7]]), -32767);
}

#[test]
fn test_clipping_nan() {
    // NaN behavior - clamp returns NaN for NaN input, which then becomes 0 when cast
    let samples = vec![f64::NAN];
    let pcm = samples_to_pcm16(&samples);

    // NaN.clamp(-1.0, 1.0) is NaN, and (NaN * 32767.0).round() as i16 is 0
    let val = i16::from_le_bytes([pcm[0], pcm[1]]);
    assert_eq!(val, 0);
}

// =========================================================================
// write_wav function tests
// =========================================================================

#[test]
fn test_write_wav_to_writer() {
    let format = WavFormat::mono(44100);
    let samples = vec![0.5, -0.5];
    let pcm = samples_to_pcm16(&samples);

    let mut buffer = Vec::new();
    write_wav(&mut buffer, &format, &pcm).expect("should write successfully");

    assert_eq!(&buffer[0..4], b"RIFF");
    assert_eq!(buffer.len(), 44 + 4); // Header + 2 samples * 2 bytes
}

#[test]
fn test_write_wav_to_vec_matches_write_wav() {
    let format = WavFormat::stereo(48000);
    let pcm = stereo_to_pcm16(&[0.3; 10], &[-0.3; 10]);

    let wav_vec = write_wav_to_vec(&format, &pcm);

    let mut wav_writer = Vec::new();
    write_wav(&mut wav_writer, &format, &pcm).expect("should write");

    assert_eq!(wav_vec, wav_writer);
}
