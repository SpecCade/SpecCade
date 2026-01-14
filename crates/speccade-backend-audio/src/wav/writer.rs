//! Core WAV writing and PCM conversion functions.

use std::io::{self, Write};

use super::format::WavFormat;

/// Writes a complete WAV file to a writer.
///
/// # Arguments
/// * `writer` - Output writer
/// * `format` - WAV format parameters
/// * `pcm_data` - Raw PCM samples as bytes
///
/// # Returns
/// Result indicating success or I/O error
pub fn write_wav<W: Write>(writer: &mut W, format: &WavFormat, pcm_data: &[u8]) -> io::Result<()> {
    let data_size = pcm_data.len() as u32;
    let file_size = 36 + data_size; // Total file size minus 8 bytes for RIFF header

    // RIFF header
    writer.write_all(b"RIFF")?;
    writer.write_all(&file_size.to_le_bytes())?;
    writer.write_all(b"WAVE")?;

    // fmt chunk
    writer.write_all(b"fmt ")?;
    writer.write_all(&16u32.to_le_bytes())?; // Chunk size (16 for PCM)
    writer.write_all(&1u16.to_le_bytes())?; // Audio format (1 = PCM)
    writer.write_all(&format.channels.to_le_bytes())?;
    writer.write_all(&format.sample_rate.to_le_bytes())?;
    writer.write_all(&format.byte_rate().to_le_bytes())?;
    writer.write_all(&format.block_align().to_le_bytes())?;
    writer.write_all(&format.bits_per_sample.to_le_bytes())?;

    // data chunk
    writer.write_all(b"data")?;
    writer.write_all(&data_size.to_le_bytes())?;
    writer.write_all(pcm_data)?;

    Ok(())
}

/// Writes a WAV file to a byte vector.
///
/// # Arguments
/// * `format` - WAV format parameters
/// * `pcm_data` - Raw PCM samples as bytes
///
/// # Returns
/// Complete WAV file as bytes
pub fn write_wav_to_vec(format: &WavFormat, pcm_data: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(44 + pcm_data.len());
    write_wav(&mut buffer, format, pcm_data).expect("writing to Vec should not fail");
    buffer
}

/// Converts f64 samples to 16-bit PCM bytes.
///
/// Samples are expected to be in range [-1.0, 1.0]. Values outside this range
/// will be clipped.
///
/// # Arguments
/// * `samples` - Audio samples in f64 format
///
/// # Returns
/// PCM data as little-endian 16-bit samples
pub fn samples_to_pcm16(samples: &[f64]) -> Vec<u8> {
    let mut pcm = Vec::with_capacity(samples.len() * 2);

    for &sample in samples {
        // Clip to [-1, 1]
        let clipped = sample.clamp(-1.0, 1.0);
        // Convert to 16-bit signed integer
        let pcm_value = (clipped * 32767.0).round() as i16;
        pcm.extend_from_slice(&pcm_value.to_le_bytes());
    }

    pcm
}

/// Converts interleaved stereo f64 samples to 16-bit PCM bytes.
///
/// # Arguments
/// * `left` - Left channel samples
/// * `right` - Right channel samples
///
/// # Returns
/// Interleaved PCM data as little-endian 16-bit samples
pub fn stereo_to_pcm16(left: &[f64], right: &[f64]) -> Vec<u8> {
    let len = left.len().min(right.len());
    let mut pcm = Vec::with_capacity(len * 4); // 2 channels * 2 bytes per sample

    for i in 0..len {
        // Left channel
        let left_clipped = left[i].clamp(-1.0, 1.0);
        let left_pcm = (left_clipped * 32767.0).round() as i16;
        pcm.extend_from_slice(&left_pcm.to_le_bytes());

        // Right channel
        let right_clipped = right[i].clamp(-1.0, 1.0);
        let right_pcm = (right_clipped * 32767.0).round() as i16;
        pcm.extend_from_slice(&right_pcm.to_le_bytes());
    }

    pcm
}
