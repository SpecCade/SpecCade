//! Audio SFX recipe types.

use serde::{Deserialize, Serialize};

/// Parameters for the `audio_sfx.layered_synth_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSfxLayeredSynthV1Params {
    /// Duration of the sound effect in seconds.
    pub duration_seconds: f64,
    /// Sample rate in Hz (22050, 44100, or 48000).
    pub sample_rate: u32,
    /// Synthesis layers to combine.
    pub layers: Vec<AudioLayer>,
}

/// A single synthesis layer in an audio SFX.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioLayer {
    /// Synthesis parameters.
    pub synthesis: Synthesis,
    /// ADSR envelope.
    pub envelope: Envelope,
    /// Volume level (0.0 to 1.0).
    pub volume: f64,
    /// Stereo pan (-1.0 = left, 0.0 = center, 1.0 = right).
    pub pan: f64,
}

/// Synthesis type configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Synthesis {
    /// FM synthesis.
    FmSynth {
        /// Carrier frequency in Hz.
        carrier_freq: f64,
        /// Modulator frequency in Hz.
        modulator_freq: f64,
        /// Modulation index.
        modulation_index: f64,
        /// Optional frequency sweep.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
    },
    /// Karplus-Strong plucked string synthesis.
    KarplusStrong {
        /// Base frequency in Hz.
        frequency: f64,
        /// Decay factor (0.0 to 1.0).
        decay: f64,
        /// Blend factor for the lowpass filter.
        blend: f64,
    },
    /// Noise burst.
    NoiseBurst {
        /// Type of noise.
        noise_type: NoiseType,
        /// Optional filter.
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<Filter>,
    },
    /// Additive synthesis with multiple harmonics.
    Additive {
        /// Base frequency in Hz.
        base_freq: f64,
        /// Harmonic amplitudes (index 0 = fundamental).
        harmonics: Vec<f64>,
    },
    /// Simple waveform oscillator.
    Oscillator {
        /// Waveform type.
        waveform: Waveform,
        /// Frequency in Hz.
        frequency: f64,
        /// Optional frequency sweep.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
    },
}

/// Frequency sweep parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreqSweep {
    /// Target frequency at end of sweep.
    pub end_freq: f64,
    /// Sweep curve type.
    pub curve: SweepCurve,
}

/// Sweep curve type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SweepCurve {
    /// Linear interpolation.
    Linear,
    /// Exponential interpolation.
    Exponential,
    /// Logarithmic interpolation.
    Logarithmic,
}

/// Noise type for noise-based synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseType {
    /// White noise (equal energy per frequency).
    White,
    /// Pink noise (1/f spectrum).
    Pink,
    /// Brown noise (1/f^2 spectrum).
    Brown,
}

/// Basic waveform types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Waveform {
    /// Sine wave.
    Sine,
    /// Square wave.
    Square,
    /// Sawtooth wave.
    Sawtooth,
    /// Triangle wave.
    Triangle,
    /// Pulse wave with variable duty cycle.
    Pulse,
}

/// Filter configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Filter {
    /// Low-pass filter.
    Lowpass {
        /// Cutoff frequency in Hz.
        cutoff: f64,
        /// Resonance (Q factor).
        resonance: f64,
    },
    /// High-pass filter.
    Highpass {
        /// Cutoff frequency in Hz.
        cutoff: f64,
        /// Resonance (Q factor).
        resonance: f64,
    },
    /// Band-pass filter.
    Bandpass {
        /// Center frequency in Hz.
        center: f64,
        /// Bandwidth in Hz.
        bandwidth: f64,
        /// Resonance (Q factor).
        resonance: f64,
    },
}

/// ADSR envelope parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    /// Attack time in seconds.
    pub attack: f64,
    /// Decay time in seconds.
    pub decay: f64,
    /// Sustain level (0.0 to 1.0).
    pub sustain: f64,
    /// Release time in seconds.
    pub release: f64,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesis_fm_serde() {
        let fm = Synthesis::FmSynth {
            carrier_freq: 440.0,
            modulator_freq: 880.0,
            modulation_index: 2.5,
            freq_sweep: Some(FreqSweep {
                end_freq: 110.0,
                curve: SweepCurve::Exponential,
            }),
        };

        let json = serde_json::to_string(&fm).unwrap();
        assert!(json.contains("fm_synth"));
        assert!(json.contains("carrier_freq"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, fm);
    }

    #[test]
    fn test_filter_serde() {
        let filter = Filter::Highpass {
            cutoff: 2000.0,
            resonance: 0.5,
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("highpass"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }
}
