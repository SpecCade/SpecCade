//! Common types shared across audio synthesis modules.

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Waveform Tests
    // ========================================================================

    #[test]
    fn test_waveform_serde() {
        let waveforms = vec![
            Waveform::Sine,
            Waveform::Square,
            Waveform::Sawtooth,
            Waveform::Triangle,
            Waveform::Pulse,
        ];

        for waveform in waveforms {
            let json = serde_json::to_string(&waveform).unwrap();
            let parsed: Waveform = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, waveform);
        }
    }

    // ========================================================================
    // FreqSweep Tests
    // ========================================================================

    #[test]
    fn test_freq_sweep_linear() {
        let sweep = FreqSweep {
            end_freq: 220.0,
            curve: SweepCurve::Linear,
        };

        let json = serde_json::to_string(&sweep).unwrap();
        assert!(json.contains("linear"));

        let parsed: FreqSweep = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, sweep);
    }

    #[test]
    fn test_freq_sweep_exponential() {
        let sweep = FreqSweep {
            end_freq: 880.0,
            curve: SweepCurve::Exponential,
        };

        let json = serde_json::to_string(&sweep).unwrap();
        assert!(json.contains("exponential"));

        let parsed: FreqSweep = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, sweep);
    }

    #[test]
    fn test_freq_sweep_logarithmic() {
        let sweep = FreqSweep {
            end_freq: 110.0,
            curve: SweepCurve::Logarithmic,
        };

        let json = serde_json::to_string(&sweep).unwrap();
        assert!(json.contains("logarithmic"));

        let parsed: FreqSweep = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, sweep);
    }
}
