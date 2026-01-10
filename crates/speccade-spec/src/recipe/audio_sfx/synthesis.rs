//! Synthesis type configuration.

use serde::{Deserialize, Serialize};

use super::common::{FreqSweep, Waveform};
use super::filter::Filter;
use super::noise::NoiseType;
use super::oscillators::OscillatorConfig;

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
        /// Detune amount in cents (100 cents = 1 semitone).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detune: Option<f64>,
        /// Duty cycle for square/pulse waves (0.0 to 1.0, default 0.5).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duty: Option<f64>,
    },
    /// Multi-oscillator stack (subtractive synthesis).
    MultiOscillator {
        /// Base frequency in Hz.
        frequency: f64,
        /// Stack of oscillators to mix additively.
        oscillators: Vec<OscillatorConfig>,
        /// Optional frequency sweep applied to all oscillators.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
    },
    /// Pitched body synthesis (impact sounds with frequency sweep).
    PitchedBody {
        /// Starting frequency in Hz.
        start_freq: f64,
        /// Ending frequency in Hz.
        end_freq: f64,
    },
    /// Metallic synthesis with inharmonic partials.
    Metallic {
        /// Base frequency in Hz.
        base_freq: f64,
        /// Number of inharmonic partials.
        num_partials: usize,
        /// Inharmonicity factor (1.0 = harmonic, >1.0 = increasingly inharmonic).
        inharmonicity: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::audio_sfx::common::SweepCurve;

    // ========================================================================
    // Synthesis Type Tests
    // ========================================================================

    #[test]
    fn test_synthesis_oscillator_sine() {
        let osc = Synthesis::Oscillator {
            waveform: Waveform::Sine,
            frequency: 440.0,
            freq_sweep: None,
            detune: None,
            duty: None,
        };

        let json = serde_json::to_string(&osc).unwrap();
        assert!(json.contains("oscillator"));
        assert!(json.contains("sine"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, osc);
    }

    #[test]
    fn test_synthesis_oscillator_square_with_duty() {
        let osc = Synthesis::Oscillator {
            waveform: Waveform::Square,
            frequency: 220.0,
            freq_sweep: None,
            detune: None,
            duty: Some(0.25),
        };

        let json = serde_json::to_string(&osc).unwrap();
        assert!(json.contains("duty"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, osc);
    }

    #[test]
    fn test_synthesis_oscillator_with_freq_sweep() {
        let osc = Synthesis::Oscillator {
            waveform: Waveform::Sawtooth,
            frequency: 880.0,
            freq_sweep: Some(FreqSweep {
                end_freq: 110.0,
                curve: SweepCurve::Linear,
            }),
            detune: Some(5.0),
            duty: None,
        };

        let json = serde_json::to_string(&osc).unwrap();
        assert!(json.contains("freq_sweep"));
        assert!(json.contains("detune"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, osc);
    }

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
    fn test_synthesis_karplus_strong() {
        let ks = Synthesis::KarplusStrong {
            frequency: 220.0,
            decay: 0.996,
            blend: 0.7,
        };

        let json = serde_json::to_string(&ks).unwrap();
        assert!(json.contains("karplus_strong"));
        assert!(json.contains("decay"));
        assert!(json.contains("blend"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ks);
    }

    #[test]
    fn test_synthesis_noise_burst_white() {
        let noise = Synthesis::NoiseBurst {
            noise_type: NoiseType::White,
            filter: None,
        };

        let json = serde_json::to_string(&noise).unwrap();
        assert!(json.contains("noise_burst"));
        assert!(json.contains("white"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, noise);
    }

    #[test]
    fn test_synthesis_noise_burst_pink_with_filter() {
        let noise = Synthesis::NoiseBurst {
            noise_type: NoiseType::Pink,
            filter: Some(Filter::Lowpass {
                cutoff: 1000.0,
                resonance: 0.5,
                cutoff_end: None,
            }),
        };

        let json = serde_json::to_string(&noise).unwrap();
        assert!(json.contains("pink"));
        assert!(json.contains("filter"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, noise);
    }

    #[test]
    fn test_synthesis_noise_burst_brown() {
        let noise = Synthesis::NoiseBurst {
            noise_type: NoiseType::Brown,
            filter: None,
        };

        let json = serde_json::to_string(&noise).unwrap();
        assert!(json.contains("brown"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, noise);
    }

    #[test]
    fn test_synthesis_additive() {
        let additive = Synthesis::Additive {
            base_freq: 440.0,
            harmonics: vec![1.0, 0.5, 0.25, 0.125],
        };

        let json = serde_json::to_string(&additive).unwrap();
        assert!(json.contains("additive"));
        assert!(json.contains("harmonics"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, additive);
    }

    #[test]
    fn test_synthesis_multi_oscillator() {
        let multi = Synthesis::MultiOscillator {
            frequency: 440.0,
            oscillators: vec![
                OscillatorConfig {
                    waveform: Waveform::Sawtooth,
                    volume: 1.0,
                    detune: None,
                    phase: None,
                    duty: None,
                },
                OscillatorConfig {
                    waveform: Waveform::Square,
                    volume: 0.5,
                    detune: Some(5.0),
                    phase: Some(1.57),
                    duty: Some(0.5),
                },
            ],
            freq_sweep: None,
        };

        let json = serde_json::to_string(&multi).unwrap();
        assert!(json.contains("multi_oscillator"));
        assert!(json.contains("oscillators"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, multi);
    }

    #[test]
    fn test_synthesis_pitched_body() {
        let pitched = Synthesis::PitchedBody {
            start_freq: 200.0,
            end_freq: 50.0,
        };

        let json = serde_json::to_string(&pitched).unwrap();
        assert!(json.contains("pitched_body"));
        assert!(json.contains("start_freq"));
        assert!(json.contains("end_freq"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, pitched);
    }

    #[test]
    fn test_synthesis_metallic() {
        let metallic = Synthesis::Metallic {
            base_freq: 800.0,
            num_partials: 6,
            inharmonicity: 1.414,
        };

        let json = serde_json::to_string(&metallic).unwrap();
        assert!(json.contains("metallic"));
        assert!(json.contains("base_freq"));
        assert!(json.contains("num_partials"));
        assert!(json.contains("inharmonicity"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, metallic);
    }
}
