//! Instrument specifications for music.

use serde::{Deserialize, Serialize};

use crate::recipe::audio::{AudioV1Params, Envelope};

/// Instrument definition for tracker modules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TrackerInstrument {
    /// Instrument name.
    #[serde(default)]
    pub name: String,
    /// Optional comment for documentation purposes (ignored by generator).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Reference to external spec file (mutually exclusive with synthesis and wav).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    /// Inline `audio_v1` synthesis params (mutually exclusive with `ref`, `wav`, and legacy
    /// `synthesis`).
    ///
    /// When set, this instrument is baked to a tracker sample by running the unified audio
    /// backend. Use this for advanced synthesis types (FM, Karplus-Strong, additive,
    /// filters/sweeps, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synthesis_audio_v1: Option<AudioV1Params>,
    /// Synthesis configuration (mutually exclusive with ref and wav).
    ///
    /// Legacy field: prefer `synthesis_audio_v1` (or `ref` to an `audio_v1` spec) for new
    /// content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synthesis: Option<InstrumentSynthesis>,
    /// Path to WAV sample file (mutually exclusive with ref and synthesis).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wav: Option<String>,
    /// Base note for the instrument (e.g., "C4", "A#3").
    /// Used for pitch correction when synthesis or wav sample is at a specific pitch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_note: Option<String>,
    /// Sample rate for synthesized instruments (default: 22050).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,
    /// ADSR envelope.
    #[serde(default = "default_envelope")]
    pub envelope: Envelope,
    /// Optional tracker sample loop override.
    ///
    /// By default, Speccade loops sustained instruments (envelope `sustain > 0`) and leaves
    /// one-shots unlooped. This field can override that behavior per instrument.
    ///
    /// - `auto` (default): choose the best loop mode automatically.
    /// - `forward`: force a forward loop (Speccade may bake a crossfade into the sample tail).
    /// - `pingpong`: force a ping-pong loop.
    /// - `none`: disable looping even for sustained instruments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_mode: Option<TrackerLoopMode>,
    /// Optional volume (0-64).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_volume: Option<u8>,
}

/// Loop mode override for tracker samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackerLoopMode {
    /// Default behavior: loop sustained instruments and choose the best loop mode.
    Auto,
    /// Disable looping.
    None,
    /// Forward loop.
    Forward,
    /// Ping-pong loop.
    #[serde(rename = "pingpong")]
    PingPong,
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
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum InstrumentSynthesis {
    /// Pulse/square wave with variable duty cycle.
    Pulse {
        /// Duty cycle (0.0 to 1.0, 0.5 = square).
        #[serde(default = "default_duty_cycle")]
        duty_cycle: f64,
    },
    /// Square wave (50% duty cycle pulse).
    Square,
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

fn default_duty_cycle() -> f64 {
    0.5
}
