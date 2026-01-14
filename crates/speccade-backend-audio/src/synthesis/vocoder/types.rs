//! Vocoder types and band configuration.

/// Band spacing mode for the vocoder filter bank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandSpacing {
    /// Linear spacing between bands (equal Hz between centers).
    Linear,
    /// Logarithmic spacing (equal ratio between bands, more perceptually uniform).
    Logarithmic,
}

/// Carrier waveform type for the vocoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarrierType {
    /// Sawtooth wave - rich in harmonics, classic vocoder sound.
    Sawtooth,
    /// Pulse wave - hollow, more synthetic sound.
    Pulse,
    /// White noise - whispery, unvoiced consonant-like sound.
    Noise,
}

/// A single vocoder band configuration.
#[derive(Debug, Clone)]
pub struct VocoderBand {
    /// Center frequency of the band in Hz.
    pub center_freq: f64,
    /// Bandwidth (Q factor) of the band filter.
    pub bandwidth: f64,
    /// Envelope pattern for this band (amplitude values over time, 0.0-1.0).
    /// If empty, a default formant animation is used.
    pub envelope_pattern: Vec<f64>,
}

impl VocoderBand {
    /// Creates a new vocoder band.
    ///
    /// # Arguments
    /// * `center_freq` - Center frequency in Hz
    /// * `bandwidth` - Q factor for the band filter
    /// * `envelope_pattern` - Optional envelope pattern (empty for default)
    pub fn new(center_freq: f64, bandwidth: f64, envelope_pattern: Vec<f64>) -> Self {
        Self {
            center_freq: center_freq.clamp(20.0, 20000.0),
            bandwidth: bandwidth.clamp(0.5, 20.0),
            envelope_pattern,
        }
    }
}
