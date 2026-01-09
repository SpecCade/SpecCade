//! Music/tracker recipe types.

use serde::{Deserialize, Serialize};

use super::audio_sfx::Envelope;

/// Parameters for the `music.tracker_song_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicTrackerSongV1Params {
    /// Tracker format: "xm" or "it".
    pub format: TrackerFormat,
    /// Beats per minute (30-300).
    pub bpm: u16,
    /// Tracker speed (ticks per row, 1-31).
    pub speed: u8,
    /// Number of channels (1-32).
    pub channels: u8,
    /// Whether the song should loop.
    #[serde(default)]
    pub r#loop: bool,
    /// Instrument definitions.
    pub instruments: Vec<TrackerInstrument>,
    /// Pattern definitions.
    pub patterns: std::collections::HashMap<String, TrackerPattern>,
    /// Song arrangement (order of patterns).
    pub arrangement: Vec<ArrangementEntry>,
}

/// Tracker module format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackerFormat {
    /// FastTracker II Extended Module format.
    Xm,
    /// Impulse Tracker format.
    It,
}

impl TrackerFormat {
    /// Returns the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            TrackerFormat::Xm => "xm",
            TrackerFormat::It => "it",
        }
    }
}

/// Instrument definition for tracker modules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackerInstrument {
    /// Instrument name.
    pub name: String,
    /// Synthesis configuration.
    pub synthesis: InstrumentSynthesis,
    /// ADSR envelope.
    pub envelope: Envelope,
    /// Optional volume (0-64).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_volume: Option<u8>,
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

/// Pattern definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackerPattern {
    /// Number of rows in the pattern.
    pub rows: u16,
    /// Note data.
    pub data: Vec<PatternNote>,
}

/// A single note event in a pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternNote {
    /// Row number (0-indexed).
    pub row: u16,
    /// Channel number (0-indexed).
    pub channel: u8,
    /// Note name (e.g., "C4", "---" for note off, "..." for no note).
    pub note: String,
    /// Instrument index.
    pub instrument: u8,
    /// Volume (0-64, optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u8>,
    /// Effect command (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<PatternEffect>,
}

/// Pattern effect command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternEffect {
    /// Effect type (e.g., "vibrato", "volume_slide").
    pub r#type: String,
    /// Effect parameter.
    pub param: u8,
}

/// Entry in the song arrangement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrangementEntry {
    /// Pattern name.
    pub pattern: String,
    /// Number of times to repeat (default: 1).
    #[serde(default = "default_repeat")]
    pub repeat: u16,
}

fn default_repeat() -> u16 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_format_extension() {
        assert_eq!(TrackerFormat::Xm.extension(), "xm");
        assert_eq!(TrackerFormat::It.extension(), "it");
    }

    #[test]
    fn test_instrument_synthesis_serde() {
        let pulse = InstrumentSynthesis::Pulse { duty_cycle: 0.5 };
        let json = serde_json::to_string(&pulse).unwrap();
        assert!(json.contains("pulse"));
        assert!(json.contains("duty_cycle"));

        let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, pulse);
    }

    #[test]
    fn test_arrangement_default_repeat() {
        let json = r#"{"pattern": "intro"}"#;
        let entry: ArrangementEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.pattern, "intro");
        assert_eq!(entry.repeat, 1);
    }
}
