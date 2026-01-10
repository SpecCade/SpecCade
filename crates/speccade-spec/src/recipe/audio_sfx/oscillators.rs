//! Oscillator configuration types.

use serde::{Deserialize, Serialize};

use super::common::Waveform;

/// Configuration for a single oscillator in a multi-oscillator stack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OscillatorConfig {
    /// Waveform type.
    pub waveform: Waveform,
    /// Volume/amplitude of this oscillator (0.0 to 1.0).
    #[serde(default = "default_oscillator_volume")]
    pub volume: f64,
    /// Detune amount in cents (100 cents = 1 semitone).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detune: Option<f64>,
    /// Phase offset in radians (0 to 2*PI).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<f64>,
    /// Duty cycle for square/pulse waves (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duty: Option<f64>,
}

fn default_oscillator_volume() -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // OscillatorConfig Tests (volume, detune, phase, duty)
    // ========================================================================

    #[test]
    fn test_oscillator_config_default_volume() {
        let osc = OscillatorConfig {
            waveform: Waveform::Sine,
            volume: default_oscillator_volume(),
            detune: None,
            phase: None,
            duty: None,
        };

        assert_eq!(osc.volume, 1.0);
    }

    #[test]
    fn test_oscillator_config_with_all_fields() {
        let osc = OscillatorConfig {
            waveform: Waveform::Square,
            volume: 0.75,
            detune: Some(10.0),
            phase: Some(3.14),
            duty: Some(0.3),
        };

        let json = serde_json::to_string(&osc).unwrap();
        assert!(json.contains("detune"));
        assert!(json.contains("phase"));
        assert!(json.contains("duty"));

        let parsed: OscillatorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, osc);
    }
}
