//! Deterministic WAV file writer.
//!
//! This module writes 16-bit PCM WAV files with no timestamps or variable metadata
//! to ensure deterministic output. The hash of the PCM data can be used for
//! Tier 1 validation.

use std::io::{self, Write};

use crate::mixer::StereoOutput;

/// WAV file format parameters.
#[derive(Debug, Clone, Copy)]
pub struct WavFormat {
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Bits per sample (always 16 for this implementation).
    pub bits_per_sample: u16,
}

impl WavFormat {
    /// Creates a mono WAV format.
    pub fn mono(sample_rate: u32) -> Self {
        Self {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
        }
    }

    /// Creates a stereo WAV format.
    pub fn stereo(sample_rate: u32) -> Self {
        Self {
            channels: 2,
            sample_rate,
            bits_per_sample: 16,
        }
    }

    /// Calculates bytes per sample (per channel).
    fn bytes_per_sample(&self) -> u16 {
        self.bits_per_sample / 8
    }

    /// Calculates block align (bytes per sample frame).
    fn block_align(&self) -> u16 {
        self.channels * self.bytes_per_sample()
    }

    /// Calculates byte rate (bytes per second).
    fn byte_rate(&self) -> u32 {
        self.sample_rate * self.block_align() as u32
    }
}

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

/// WAV file writer builder.
#[derive(Debug)]
pub struct WavWriter {
    format: WavFormat,
}

impl WavWriter {
    /// Creates a new WAV writer with mono format.
    pub fn mono(sample_rate: u32) -> Self {
        Self {
            format: WavFormat::mono(sample_rate),
        }
    }

    /// Creates a new WAV writer with stereo format.
    pub fn stereo(sample_rate: u32) -> Self {
        Self {
            format: WavFormat::stereo(sample_rate),
        }
    }

    /// Writes mono samples to a byte vector.
    pub fn write_mono(&self, samples: &[f64]) -> Vec<u8> {
        let pcm = samples_to_pcm16(samples);
        write_wav_to_vec(&self.format, &pcm)
    }

    /// Writes stereo samples to a byte vector.
    pub fn write_stereo(&self, left: &[f64], right: &[f64]) -> Vec<u8> {
        let pcm = stereo_to_pcm16(left, right);
        write_wav_to_vec(&self.format, &pcm)
    }

    /// Writes a StereoOutput to a byte vector.
    pub fn write_stereo_output(&self, stereo: &StereoOutput) -> Vec<u8> {
        self.write_stereo(&stereo.left, &stereo.right)
    }

    /// Returns the PCM data hash for Tier 1 validation.
    ///
    /// # Arguments
    /// * `samples` - Mono audio samples
    ///
    /// # Returns
    /// BLAKE3 hash of the PCM data (not the full WAV file)
    pub fn pcm_hash_mono(&self, samples: &[f64]) -> String {
        let pcm = samples_to_pcm16(samples);
        blake3::hash(&pcm).to_hex().to_string()
    }

    /// Returns the PCM data hash for Tier 1 validation (stereo).
    ///
    /// # Arguments
    /// * `left` - Left channel samples
    /// * `right` - Right channel samples
    ///
    /// # Returns
    /// BLAKE3 hash of the PCM data (not the full WAV file)
    pub fn pcm_hash_stereo(&self, left: &[f64], right: &[f64]) -> String {
        let pcm = stereo_to_pcm16(left, right);
        blake3::hash(&pcm).to_hex().to_string()
    }
}

/// Result of WAV file generation.
#[derive(Debug)]
pub struct WavResult {
    /// Complete WAV file bytes.
    pub wav_data: Vec<u8>,
    /// BLAKE3 hash of PCM data only (for Tier 1 validation).
    pub pcm_hash: String,
    /// Whether the output is stereo.
    pub is_stereo: bool,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of samples per channel.
    pub num_samples: usize,
}

impl WavResult {
    /// Creates a WavResult from mono samples.
    pub fn from_mono(samples: &[f64], sample_rate: u32) -> Self {
        let pcm = samples_to_pcm16(samples);
        let pcm_hash = blake3::hash(&pcm).to_hex().to_string();
        let format = WavFormat::mono(sample_rate);
        let wav_data = write_wav_to_vec(&format, &pcm);

        Self {
            wav_data,
            pcm_hash,
            is_stereo: false,
            sample_rate,
            num_samples: samples.len(),
        }
    }

    /// Creates a WavResult from stereo samples.
    pub fn from_stereo(left: &[f64], right: &[f64], sample_rate: u32) -> Self {
        let pcm = stereo_to_pcm16(left, right);
        let pcm_hash = blake3::hash(&pcm).to_hex().to_string();
        let format = WavFormat::stereo(sample_rate);
        let wav_data = write_wav_to_vec(&format, &pcm);

        Self {
            wav_data,
            pcm_hash,
            is_stereo: true,
            sample_rate,
            num_samples: left.len().min(right.len()),
        }
    }

    /// Creates a WavResult from StereoOutput.
    pub fn from_stereo_output(stereo: &StereoOutput, sample_rate: u32) -> Self {
        Self::from_stereo(&stereo.left, &stereo.right, sample_rate)
    }

    /// Returns the duration in seconds.
    pub fn duration_seconds(&self) -> f64 {
        self.num_samples as f64 / self.sample_rate as f64
    }
}

/// Extracts PCM data from a WAV file buffer.
///
/// Used for comparing WAV files by their audio content only.
///
/// # Arguments
/// * `wav_data` - Complete WAV file bytes
///
/// # Returns
/// PCM data if found, or None if the format is invalid
pub fn extract_pcm_data(wav_data: &[u8]) -> Option<&[u8]> {
    if wav_data.len() < 44 {
        return None;
    }

    // Verify RIFF header
    if &wav_data[0..4] != b"RIFF" || &wav_data[8..12] != b"WAVE" {
        return None;
    }

    // Find data chunk
    let mut pos = 12;
    while pos + 8 <= wav_data.len() {
        let chunk_id = &wav_data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            wav_data[pos + 4],
            wav_data[pos + 5],
            wav_data[pos + 6],
            wav_data[pos + 7],
        ]) as usize;

        if chunk_id == b"data" {
            let data_start = pos + 8;
            let data_end = data_start + chunk_size;
            if data_end <= wav_data.len() {
                return Some(&wav_data[data_start..data_end]);
            }
        }

        pos += 8 + chunk_size;
        // Align to word boundary
        if !chunk_size.is_multiple_of(2) {
            pos += 1;
        }
    }

    None
}

/// Computes the PCM hash of a WAV file.
///
/// # Arguments
/// * `wav_data` - Complete WAV file bytes
///
/// # Returns
/// BLAKE3 hash of PCM data, or None if format is invalid
pub fn compute_pcm_hash(wav_data: &[u8]) -> Option<String> {
    extract_pcm_data(wav_data).map(|pcm| blake3::hash(pcm).to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_format() {
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

    #[test]
    fn test_samples_to_pcm16() {
        let samples = vec![0.0, 1.0, -1.0, 0.5, -0.5];
        let pcm = samples_to_pcm16(&samples);

        assert_eq!(pcm.len(), 10); // 5 samples * 2 bytes

        // 0.0 should be 0
        assert_eq!(i16::from_le_bytes([pcm[0], pcm[1]]), 0);
        // 1.0 should be close to 32767
        assert_eq!(i16::from_le_bytes([pcm[2], pcm[3]]), 32767);
        // -1.0 should be close to -32767
        assert_eq!(i16::from_le_bytes([pcm[4], pcm[5]]), -32767);
    }

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
    fn test_pcm_hash_determinism() {
        let writer = WavWriter::mono(44100);
        let samples = vec![0.5, -0.5, 0.3, -0.3, 0.0];

        let hash1 = writer.pcm_hash_mono(&samples);
        let hash2 = writer.pcm_hash_mono(&samples);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // BLAKE3 produces 64 hex chars
    }

    #[test]
    fn test_extract_pcm_data() {
        let writer = WavWriter::mono(44100);
        let samples = vec![0.5; 100];
        let wav = writer.write_mono(&samples);

        let pcm = extract_pcm_data(&wav).expect("should extract PCM");
        assert_eq!(pcm.len(), 200); // 100 samples * 2 bytes
    }

    #[test]
    fn test_wav_result() {
        let samples = vec![0.5, -0.5, 0.3, -0.3];
        let result = WavResult::from_mono(&samples, 44100);

        assert!(!result.is_stereo);
        assert_eq!(result.sample_rate, 44100);
        assert_eq!(result.num_samples, 4);
        assert_eq!(result.pcm_hash.len(), 64);
    }

    #[test]
    fn test_wav_writer_pcm_hash() {
        let writer = WavWriter::mono(44100);
        let samples = vec![0.5, -0.5, 0.3, -0.3, 0.0];
        let hash = writer.pcm_hash_mono(&samples);

        assert_eq!(hash.len(), 64); // BLAKE3 produces 64 hex chars
    }

    #[test]
    fn test_clipping() {
        let samples = vec![2.0, -2.0]; // Out of range
        let pcm = samples_to_pcm16(&samples);

        // Should be clipped to max/min values
        let val1 = i16::from_le_bytes([pcm[0], pcm[1]]);
        let val2 = i16::from_le_bytes([pcm[2], pcm[3]]);

        assert_eq!(val1, 32767);
        assert_eq!(val2, -32767);
    }
}
