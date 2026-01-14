//! WAV file format parameters.

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
    pub(crate) fn bytes_per_sample(&self) -> u16 {
        self.bits_per_sample / 8
    }

    /// Calculates block align (bytes per sample frame).
    pub(crate) fn block_align(&self) -> u16 {
        self.channels * self.bytes_per_sample()
    }

    /// Calculates byte rate (bytes per second).
    pub(crate) fn byte_rate(&self) -> u32 {
        self.sample_rate * self.block_align() as u32
    }
}
