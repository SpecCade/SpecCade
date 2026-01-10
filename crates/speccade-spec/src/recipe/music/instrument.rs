//! Instrument specifications for music.

use serde::{Deserialize, Serialize};

use crate::recipe::audio_sfx::Envelope;

/// Instrument definition for tracker modules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackerInstrument {
    /// Instrument name.
    pub name: String,
    /// Reference to external spec file (mutually exclusive with synthesis).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    /// Synthesis configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synthesis: Option<InstrumentSynthesis>,
    /// ADSR envelope.
    #[serde(default = "default_envelope")]
    pub envelope: Envelope,
    /// Optional volume (0-64).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_volume: Option<u8>,
}

pub(crate) fn default_envelope() -> Envelope {
    Envelope {
        attack: 0.01,
        decay: 0.1,
        sustain: 0.7,
        release: 0.2,
    }
}

/// Synthesis type for tracker instruments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstrumentSynthesis {
    /// Pulse/square wave with variable duty cycle.
    Pulse {
        /// Duty cycle (0.0 to 1.0, 0.5 = square).
        duty_cycle: f64,
    },
    /// Triangle wave.
    Triangle,
    /// Sawtooth wave.
    Sawtooth,
    /// Sine wave.
    Sine,
    /// Noise generator.
    Noise {
        /// Whether to use periodic noise (more tonal).
        #[serde(default)]
        periodic: bool,
    },
    /// Sample-based instrument.
    Sample {
        /// Path to sample file (relative to spec).
        path: String,
        /// Base note for the sample.
        #[serde(skip_serializing_if = "Option::is_none")]
        base_note: Option<String>,
    },
}
