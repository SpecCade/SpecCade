//! LFO and modulation types for synthesis.

use serde::{Deserialize, Serialize};

use super::basic_types::Waveform;

/// LFO (Low Frequency Oscillator) configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LfoConfig {
    /// Waveform type for the LFO.
    pub waveform: Waveform,
    /// LFO rate in Hz (typically 0.1-20 Hz).
    pub rate: f64,
    /// Modulation depth (0.0-1.0).
    pub depth: f64,
    /// Initial phase offset (0.0-1.0, optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<f64>,
}

/// Modulation target specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "target", rename_all = "snake_case")]
pub enum ModulationTarget {
    /// Modulate pitch (vibrato).
    Pitch {
        /// Maximum pitch deviation in semitones.
        semitones: f64,
    },
    /// Modulate volume (tremolo).
    Volume,
    /// Modulate filter cutoff frequency.
    FilterCutoff {
        /// Maximum cutoff frequency change in Hz.
        amount: f64,
    },
    /// Modulate stereo pan.
    Pan,
}

/// LFO modulation configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LfoModulation {
    /// LFO configuration.
    pub config: LfoConfig,
    /// Modulation target.
    pub target: ModulationTarget,
}
