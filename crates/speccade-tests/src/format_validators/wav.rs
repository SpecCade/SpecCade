//! WAV file format validator.

use super::FormatError;

/// Information extracted from a WAV file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WavInfo {
    /// Number of audio channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Sample rate in Hz (e.g., 44100, 48000).
    pub sample_rate: u32,
    /// Bits per sample (e.g., 8, 16, 24, 32).
    pub bits_per_sample: u16,
    /// Total number of samples (per channel).
    pub num_samples: usize,
    /// Audio format code (1 = PCM, 3 = IEEE float).
    pub audio_format: u16,
    /// Byte rate (sample_rate * channels * bits_per_sample / 8).
    pub byte_rate: u32,
    /// Block alignment (channels * bits_per_sample / 8).
    pub block_align: u16,
}

/// Validate WAV file format and extract header information.
///
/// Parses the RIFF/WAVE header structure and validates:
/// - RIFF chunk identifier
/// - WAVE format identifier
/// - fmt sub-chunk with audio parameters
/// - data sub-chunk presence
///
/// # Arguments
/// * `data` - Raw bytes of the WAV file
///
/// # Returns
/// * `Ok(WavInfo)` - Successfully parsed WAV file information
/// * `Err(FormatError)` - Invalid or corrupted WAV file
pub fn validate_wav(data: &[u8]) -> Result<WavInfo, FormatError> {
    const MIN_HEADER_SIZE: usize = 44;

    if data.len() < MIN_HEADER_SIZE {
        return Err(FormatError::new(
            "WAV",
            format!(
                "File too short: {} bytes (minimum {} required)",
                data.len(),
                MIN_HEADER_SIZE
            ),
        ));
    }

    // Check RIFF header
    if &data[0..4] != b"RIFF" {
        return Err(FormatError::at_offset(
            "WAV",
            format!(
                "Invalid RIFF header: expected 'RIFF', got {:?}",
                &data[0..4]
            ),
            0,
        ));
    }

    // Check WAVE format
    if &data[8..12] != b"WAVE" {
        return Err(FormatError::at_offset(
            "WAV",
            format!(
                "Invalid WAVE format: expected 'WAVE', got {:?}",
                &data[8..12]
            ),
            8,
        ));
    }

    // Find and parse fmt chunk
    let mut offset = 12;
    let mut fmt_found = false;
    let mut audio_format = 0u16;
    let mut channels = 0u16;
    let mut sample_rate = 0u32;
    let mut byte_rate = 0u32;
    let mut block_align = 0u16;
    let mut bits_per_sample = 0u16;

    while offset + 8 <= data.len() {
        let chunk_id = &data[offset..offset + 4];
        let chunk_size = u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;

        if chunk_id == b"fmt " {
            if chunk_size < 16 {
                return Err(FormatError::at_offset(
                    "WAV",
                    format!("fmt chunk too small: {} bytes", chunk_size),
                    offset,
                ));
            }

            if offset + 8 + 16 > data.len() {
                return Err(FormatError::at_offset("WAV", "Truncated fmt chunk", offset));
            }

            let fmt_data = &data[offset + 8..];
            audio_format = u16::from_le_bytes([fmt_data[0], fmt_data[1]]);
            channels = u16::from_le_bytes([fmt_data[2], fmt_data[3]]);
            sample_rate = u32::from_le_bytes([fmt_data[4], fmt_data[5], fmt_data[6], fmt_data[7]]);
            byte_rate = u32::from_le_bytes([fmt_data[8], fmt_data[9], fmt_data[10], fmt_data[11]]);
            block_align = u16::from_le_bytes([fmt_data[12], fmt_data[13]]);
            bits_per_sample = u16::from_le_bytes([fmt_data[14], fmt_data[15]]);

            fmt_found = true;
        }

        if chunk_id == b"data" {
            if !fmt_found {
                return Err(FormatError::at_offset(
                    "WAV",
                    "data chunk found before fmt chunk",
                    offset,
                ));
            }

            let num_samples = if block_align > 0 {
                chunk_size / block_align as usize
            } else {
                0
            };

            return Ok(WavInfo {
                channels,
                sample_rate,
                bits_per_sample,
                num_samples,
                audio_format,
                byte_rate,
                block_align,
            });
        }

        // Move to next chunk (chunks are word-aligned)
        let padded_size = (chunk_size + 1) & !1;
        offset += 8 + padded_size;
    }

    if !fmt_found {
        return Err(FormatError::new("WAV", "Missing fmt chunk"));
    }

    Err(FormatError::new("WAV", "Missing data chunk"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_wav_valid_pcm16() {
        // Create a minimal valid WAV file (16-bit PCM, mono, 44100 Hz)
        let wav = create_test_wav(1, 44100, 16, 100);
        let info = validate_wav(&wav).expect("Should parse valid WAV");

        assert_eq!(info.channels, 1);
        assert_eq!(info.sample_rate, 44100);
        assert_eq!(info.bits_per_sample, 16);
        assert_eq!(info.audio_format, 1); // PCM
        assert_eq!(info.num_samples, 100);
    }

    #[test]
    fn test_validate_wav_stereo() {
        let wav = create_test_wav(2, 48000, 16, 50);
        let info = validate_wav(&wav).expect("Should parse stereo WAV");

        assert_eq!(info.channels, 2);
        assert_eq!(info.sample_rate, 48000);
        assert_eq!(info.num_samples, 50);
    }

    #[test]
    fn test_validate_wav_too_short() {
        let data = vec![0u8; 10];
        let err = validate_wav(&data).unwrap_err();
        assert_eq!(err.format, "WAV");
        assert!(err.message.contains("too short"));
    }

    #[test]
    fn test_validate_wav_invalid_riff() {
        let mut wav = create_test_wav(1, 44100, 16, 10);
        wav[0..4].copy_from_slice(b"XXXX");
        let err = validate_wav(&wav).unwrap_err();
        assert!(err.message.contains("RIFF"));
    }

    #[test]
    fn test_validate_wav_invalid_wave() {
        let mut wav = create_test_wav(1, 44100, 16, 10);
        wav[8..12].copy_from_slice(b"XXXX");
        let err = validate_wav(&wav).unwrap_err();
        assert!(err.message.contains("WAVE"));
    }

    fn create_test_wav(
        channels: u16,
        sample_rate: u32,
        bits_per_sample: u16,
        num_samples: usize,
    ) -> Vec<u8> {
        let block_align = channels * bits_per_sample / 8;
        let byte_rate = sample_rate * block_align as u32;
        let data_size = num_samples * block_align as usize;
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
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data_size as u32).to_le_bytes());
        wav.resize(wav.len() + data_size, 0);

        wav
    }
}
