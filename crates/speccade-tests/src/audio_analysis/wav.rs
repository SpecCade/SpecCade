//! WAV file parsing and sample extraction.

use super::error::{AudioAnalysisError, WavHeader};

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
    let (fmt_offset, _fmt_size) =
        find_chunk(wav_data, b"fmt ")?.ok_or(AudioAnalysisError::MissingFmtChunk)?;

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
    let (data_offset, data_size) =
        find_chunk(wav_data, b"data")?.ok_or(AudioAnalysisError::MissingDataChunk)?;

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
                data_offset,
                data_size,
                wav_data.len()
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
    let (data_offset, data_size) =
        find_chunk(wav_data, b"data")?.ok_or(AudioAnalysisError::MissingDataChunk)?;

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
fn find_chunk(
    wav_data: &[u8],
    chunk_id: &[u8; 4],
) -> Result<Option<(usize, usize)>, AudioAnalysisError> {
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
    data.iter().map(|&b| (b as f32 - 128.0) / 128.0).collect()
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
