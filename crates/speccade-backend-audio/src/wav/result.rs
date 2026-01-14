//! WAV file generation result type.

use crate::mixer::StereoOutput;

use super::format::WavFormat;
use super::writer::{samples_to_pcm16, stereo_to_pcm16, write_wav_to_vec};

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
