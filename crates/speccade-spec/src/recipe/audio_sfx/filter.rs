//! Filter types for audio synthesis.

use serde::{Deserialize, Serialize};

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
        /// Optional target cutoff frequency for sweep.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cutoff_end: Option<f64>,
    },
    /// High-pass filter.
    Highpass {
        /// Cutoff frequency in Hz.
        cutoff: f64,
        /// Resonance (Q factor).
        resonance: f64,
        /// Optional target cutoff frequency for sweep.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cutoff_end: Option<f64>,
    },
    /// Band-pass filter.
    Bandpass {
        /// Center frequency in Hz.
        center: f64,
        /// Bandwidth in Hz.
        bandwidth: f64,
        /// Resonance (Q factor).
        resonance: f64,
        /// Optional target center frequency for sweep.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        center_end: Option<f64>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Filter Tests (lowpass, highpass, bandpass, cutoff sweeps)
    // ========================================================================

    #[test]
    fn test_filter_lowpass_basic() {
        let filter = Filter::Lowpass {
            cutoff: 2000.0,
            resonance: 0.707,
            cutoff_end: None,
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("lowpass"));
        assert!(json.contains("2000"));
        assert!(json.contains("0.707"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_lowpass_with_sweep() {
        let filter = Filter::Lowpass {
            cutoff: 5000.0,
            resonance: 1.0,
            cutoff_end: Some(500.0),
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("cutoff_end"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_highpass() {
        let filter = Filter::Highpass {
            cutoff: 2000.0,
            resonance: 0.5,
            cutoff_end: None,
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("highpass"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_highpass_with_sweep() {
        let filter = Filter::Highpass {
            cutoff: 100.0,
            resonance: 0.8,
            cutoff_end: Some(2000.0),
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("cutoff_end"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_bandpass() {
        let filter = Filter::Bandpass {
            center: 1000.0,
            bandwidth: 500.0,
            resonance: 0.707,
            center_end: None,
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("bandpass"));
        assert!(json.contains("center"));
        assert!(json.contains("bandwidth"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_bandpass_with_sweep() {
        let filter = Filter::Bandpass {
            center: 2000.0,
            bandwidth: 300.0,
            resonance: 1.2,
            center_end: Some(500.0),
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("center_end"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }
}
