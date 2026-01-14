//! WAV file writer builder pattern.

use crate::mixer::StereoOutput;

use super::format::WavFormat;
use super::writer::{samples_to_pcm16, stereo_to_pcm16, write_wav_to_vec};

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
