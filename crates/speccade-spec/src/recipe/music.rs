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
    /// Automation definitions (volume fades, tempo changes).
    #[serde(default)]
    pub automation: Vec<AutomationEntry>,
    /// IT-specific options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub it_options: Option<ItOptions>,
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

fn default_envelope() -> Envelope {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Effect parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<u8>,
    /// Effect X/Y nibbles (alternative to param).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_xy: Option<(u8, u8)>,
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

/// IT-specific module options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItOptions {
    /// Stereo output flag.
    #[serde(default = "default_stereo")]
    pub stereo: bool,
    /// Global volume (0-128).
    #[serde(default = "default_global_volume")]
    pub global_volume: u8,
    /// Mix volume (0-128).
    #[serde(default = "default_mix_volume")]
    pub mix_volume: u8,
}

fn default_stereo() -> bool {
    true
}

fn default_global_volume() -> u8 {
    128
}

fn default_mix_volume() -> u8 {
    48
}

impl Default for ItOptions {
    fn default() -> Self {
        Self {
            stereo: true,
            global_volume: 128,
            mix_volume: 48,
        }
    }
}

/// Automation entry for volume fades and tempo changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutomationEntry {
    /// Volume fade automation.
    VolumeFade {
        /// Target pattern name.
        pattern: String,
        /// Target channel (0-indexed).
        #[serde(default)]
        channel: u8,
        /// Start row.
        #[serde(default)]
        start_row: u16,
        /// End row.
        end_row: u16,
        /// Start volume (0-64).
        #[serde(default)]
        start_vol: u8,
        /// End volume (0-64).
        end_vol: u8,
    },
    /// Tempo change automation.
    TempoChange {
        /// Target pattern name.
        pattern: String,
        /// Row for tempo change.
        #[serde(default)]
        row: u16,
        /// New BPM (32-255).
        bpm: u8,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ==================== Top-Level Keys Tests ====================

    #[test]
    fn test_song_name_serialization() {
        let params = MusicTrackerSongV1Params {
            format: TrackerFormat::Xm,
            bpm: 125,
            speed: 6,
            channels: 8,
            r#loop: false,
            instruments: vec![],
            patterns: HashMap::new(),
            arrangement: vec![],
            automation: vec![],
            it_options: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: MusicTrackerSongV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.format, TrackerFormat::Xm);
    }

    #[test]
    fn test_format_xm_serialization() {
        let format = TrackerFormat::Xm;
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, r#""xm""#);
        let parsed: TrackerFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, TrackerFormat::Xm);
    }

    #[test]
    fn test_format_it_serialization() {
        let format = TrackerFormat::It;
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, r#""it""#);
        let parsed: TrackerFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, TrackerFormat::It);
    }

    #[test]
    fn test_tracker_format_extension() {
        assert_eq!(TrackerFormat::Xm.extension(), "xm");
        assert_eq!(TrackerFormat::It.extension(), "it");
    }

    #[test]
    fn test_bpm_serialization() {
        let json = r#"{"format":"xm","bpm":140,"speed":6,"channels":8,"loop":false,"instruments":[],"patterns":{},"arrangement":[]}"#;
        let parsed: MusicTrackerSongV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.bpm, 140);
    }

    #[test]
    fn test_bpm_default_value() {
        // Default value should be 125 according to parity matrix
        let json = r#"{"format":"xm","speed":6,"channels":8,"loop":false,"instruments":[],"patterns":{},"arrangement":[]}"#;
        let result: Result<MusicTrackerSongV1Params, _> = serde_json::from_str(json);
        // bpm is required, so this should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_speed_serialization() {
        let json = r#"{"format":"xm","bpm":125,"speed":8,"channels":8,"loop":false,"instruments":[],"patterns":{},"arrangement":[]}"#;
        let parsed: MusicTrackerSongV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.speed, 8);
    }

    #[test]
    fn test_channels_serialization() {
        let json = r#"{"format":"xm","bpm":125,"speed":6,"channels":16,"loop":false,"instruments":[],"patterns":{},"arrangement":[]}"#;
        let parsed: MusicTrackerSongV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.channels, 16);
    }

    #[test]
    fn test_loop_serialization() {
        let json = r#"{"format":"xm","bpm":125,"speed":6,"channels":8,"loop":true,"instruments":[],"patterns":{},"arrangement":[]}"#;
        let parsed: MusicTrackerSongV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.r#loop, true);
    }

    #[test]
    fn test_loop_default_value() {
        let json = r#"{"format":"xm","bpm":125,"speed":6,"channels":8,"instruments":[],"patterns":{},"arrangement":[]}"#;
        let parsed: MusicTrackerSongV1Params = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.r#loop, false);
    }

    #[test]
    fn test_instruments_serialization() {
        let instr = TrackerInstrument {
            name: "Lead".to_string(),
            r#ref: None,
            synthesis: Some(InstrumentSynthesis::Sine),
            envelope: Envelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.2,
            },
            default_volume: Some(64),
        };

        let json = serde_json::to_string(&instr).unwrap();
        let parsed: TrackerInstrument = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Lead");
    }

    #[test]
    fn test_patterns_serialization() {
        let mut patterns = HashMap::new();
        patterns.insert(
            "intro".to_string(),
            TrackerPattern {
                rows: 64,
                data: vec![],
            },
        );

        let json = serde_json::to_string(&patterns).unwrap();
        let parsed: HashMap<String, TrackerPattern> = serde_json::from_str(&json).unwrap();
        assert!(parsed.contains_key("intro"));
    }

    #[test]
    fn test_arrangement_serialization() {
        let arrangement = vec![
            ArrangementEntry {
                pattern: "intro".to_string(),
                repeat: 2,
            },
            ArrangementEntry {
                pattern: "verse".to_string(),
                repeat: 1,
            },
        ];

        let json = serde_json::to_string(&arrangement).unwrap();
        let parsed: Vec<ArrangementEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_automation_serialization() {
        let automation = vec![
            AutomationEntry::VolumeFade {
                pattern: "intro".to_string(),
                channel: 0,
                start_row: 0,
                end_row: 63,
                start_vol: 0,
                end_vol: 64,
            },
            AutomationEntry::TempoChange {
                pattern: "chorus".to_string(),
                row: 0,
                bpm: 140,
            },
        ];

        let json = serde_json::to_string(&automation).unwrap();
        let parsed: Vec<AutomationEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_it_options_serialization() {
        let it_options = ItOptions {
            stereo: true,
            global_volume: 128,
            mix_volume: 48,
        };

        let json = serde_json::to_string(&it_options).unwrap();
        let parsed: ItOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stereo, true);
        assert_eq!(parsed.global_volume, 128);
        assert_eq!(parsed.mix_volume, 48);
    }

    // ==================== Instrument Keys Tests ====================

    #[test]
    fn test_instrument_name_serialization() {
        let instr = TrackerInstrument {
            name: "Bass".to_string(),
            r#ref: None,
            synthesis: Some(InstrumentSynthesis::Sawtooth),
            envelope: default_envelope(),
            default_volume: None,
        };

        let json = serde_json::to_string(&instr).unwrap();
        assert!(json.contains("Bass"));
    }

    #[test]
    fn test_instrument_ref_serialization() {
        let instr = TrackerInstrument {
            name: "External".to_string(),
            r#ref: Some("instruments/lead.spec.py".to_string()),
            synthesis: None,
            envelope: default_envelope(),
            default_volume: None,
        };

        let json = serde_json::to_string(&instr).unwrap();
        let parsed: TrackerInstrument = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.r#ref, Some("instruments/lead.spec.py".to_string()));
    }

    #[test]
    fn test_instrument_synthesis_pulse() {
        let pulse = InstrumentSynthesis::Pulse { duty_cycle: 0.5 };
        let json = serde_json::to_string(&pulse).unwrap();
        assert!(json.contains("pulse"));
        assert!(json.contains("duty_cycle"));

        let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, pulse);
    }

    #[test]
    fn test_instrument_synthesis_triangle() {
        let triangle = InstrumentSynthesis::Triangle;
        let json = serde_json::to_string(&triangle).unwrap();
        assert!(json.contains("triangle"));

        let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, triangle);
    }

    #[test]
    fn test_instrument_synthesis_sawtooth() {
        let sawtooth = InstrumentSynthesis::Sawtooth;
        let json = serde_json::to_string(&sawtooth).unwrap();
        assert!(json.contains("sawtooth"));

        let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, sawtooth);
    }

    #[test]
    fn test_instrument_synthesis_sine() {
        let sine = InstrumentSynthesis::Sine;
        let json = serde_json::to_string(&sine).unwrap();
        assert!(json.contains("sine"));

        let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, sine);
    }

    #[test]
    fn test_instrument_synthesis_noise() {
        let noise = InstrumentSynthesis::Noise { periodic: true };
        let json = serde_json::to_string(&noise).unwrap();
        assert!(json.contains("noise"));

        let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, noise);
    }

    #[test]
    fn test_instrument_synthesis_sample() {
        let sample = InstrumentSynthesis::Sample {
            path: "samples/kick.wav".to_string(),
            base_note: Some("C4".to_string()),
        };
        let json = serde_json::to_string(&sample).unwrap();
        assert!(json.contains("sample"));
        assert!(json.contains("samples/kick.wav"));

        let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, sample);
    }

    #[test]
    fn test_instrument_envelope_serialization() {
        let envelope = Envelope {
            attack: 0.05,
            decay: 0.15,
            sustain: 0.6,
            release: 0.3,
        };

        let json = serde_json::to_string(&envelope).unwrap();
        let parsed: Envelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.attack, 0.05);
        assert_eq!(parsed.decay, 0.15);
        assert_eq!(parsed.sustain, 0.6);
        assert_eq!(parsed.release, 0.3);
    }

    #[test]
    fn test_instrument_envelope_default() {
        let envelope = default_envelope();
        assert_eq!(envelope.attack, 0.01);
        assert_eq!(envelope.decay, 0.1);
        assert_eq!(envelope.sustain, 0.7);
        assert_eq!(envelope.release, 0.2);
    }

    // ==================== Pattern Keys Tests ====================

    #[test]
    fn test_pattern_rows_serialization() {
        let pattern = TrackerPattern {
            rows: 128,
            data: vec![],
        };

        let json = serde_json::to_string(&pattern).unwrap();
        let parsed: TrackerPattern = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.rows, 128);
    }

    #[test]
    fn test_pattern_notes_serialization() {
        let note = PatternNote {
            row: 0,
            channel: 0,
            note: "C4".to_string(),
            instrument: 1,
            volume: Some(64),
            effect: None,
        };

        let pattern = TrackerPattern {
            rows: 64,
            data: vec![note],
        };

        let json = serde_json::to_string(&pattern).unwrap();
        let parsed: TrackerPattern = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.data.len(), 1);
    }

    // ==================== Note Keys Tests ====================

    #[test]
    fn test_note_row_serialization() {
        let note = PatternNote {
            row: 16,
            channel: 0,
            note: "C4".to_string(),
            instrument: 0,
            volume: None,
            effect: None,
        };

        let json = serde_json::to_string(&note).unwrap();
        let parsed: PatternNote = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.row, 16);
    }

    #[test]
    fn test_note_channel_serialization() {
        let note = PatternNote {
            row: 0,
            channel: 3,
            note: "E4".to_string(),
            instrument: 0,
            volume: None,
            effect: None,
        };

        let json = serde_json::to_string(&note).unwrap();
        let parsed: PatternNote = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.channel, 3);
    }

    #[test]
    fn test_note_note_serialization() {
        let note = PatternNote {
            row: 0,
            channel: 0,
            note: "A#5".to_string(),
            instrument: 0,
            volume: None,
            effect: None,
        };

        let json = serde_json::to_string(&note).unwrap();
        let parsed: PatternNote = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.note, "A#5");
    }

    #[test]
    fn test_note_instrument_serialization() {
        let note = PatternNote {
            row: 0,
            channel: 0,
            note: "C4".to_string(),
            instrument: 5,
            volume: None,
            effect: None,
        };

        let json = serde_json::to_string(&note).unwrap();
        let parsed: PatternNote = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.instrument, 5);
    }

    #[test]
    fn test_note_volume_serialization() {
        let note = PatternNote {
            row: 0,
            channel: 0,
            note: "C4".to_string(),
            instrument: 0,
            volume: Some(48),
            effect: None,
        };

        let json = serde_json::to_string(&note).unwrap();
        let parsed: PatternNote = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.volume, Some(48));
    }

    #[test]
    fn test_note_effect_serialization() {
        let effect = PatternEffect {
            r#type: Some("vibrato".to_string()),
            param: Some(0x44),
            effect_xy: None,
        };

        let note = PatternNote {
            row: 0,
            channel: 0,
            note: "C4".to_string(),
            instrument: 0,
            volume: None,
            effect: Some(effect),
        };

        let json = serde_json::to_string(&note).unwrap();
        let parsed: PatternNote = serde_json::from_str(&json).unwrap();
        assert!(parsed.effect.is_some());
    }

    #[test]
    fn test_note_effect_param_serialization() {
        let effect = PatternEffect {
            r#type: Some("arpeggio".to_string()),
            param: Some(0x37),
            effect_xy: None,
        };

        let json = serde_json::to_string(&effect).unwrap();
        let parsed: PatternEffect = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.param, Some(0x37));
    }

    #[test]
    fn test_note_effect_xy_serialization() {
        let effect = PatternEffect {
            r#type: Some("portamento".to_string()),
            param: None,
            effect_xy: Some((0x3, 0x7)),
        };

        let json = serde_json::to_string(&effect).unwrap();
        let parsed: PatternEffect = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.effect_xy, Some((0x3, 0x7)));
    }

    // ==================== Automation Keys Tests ====================

    #[test]
    fn test_automation_volume_fade_type() {
        let auto = AutomationEntry::VolumeFade {
            pattern: "intro".to_string(),
            channel: 0,
            start_row: 0,
            end_row: 63,
            start_vol: 0,
            end_vol: 64,
        };

        let json = serde_json::to_string(&auto).unwrap();
        assert!(json.contains("volume_fade"));
    }

    #[test]
    fn test_automation_tempo_change_type() {
        let auto = AutomationEntry::TempoChange {
            pattern: "chorus".to_string(),
            row: 0,
            bpm: 140,
        };

        let json = serde_json::to_string(&auto).unwrap();
        assert!(json.contains("tempo_change"));
    }

    #[test]
    fn test_automation_pattern_serialization() {
        let auto = AutomationEntry::VolumeFade {
            pattern: "verse".to_string(),
            channel: 0,
            start_row: 0,
            end_row: 63,
            start_vol: 64,
            end_vol: 0,
        };

        let json = serde_json::to_string(&auto).unwrap();
        assert!(json.contains("verse"));
    }

    #[test]
    fn test_automation_channel_serialization() {
        let auto = AutomationEntry::VolumeFade {
            pattern: "intro".to_string(),
            channel: 2,
            start_row: 0,
            end_row: 63,
            start_vol: 0,
            end_vol: 64,
        };

        let json = serde_json::to_string(&auto).unwrap();
        let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
        match parsed {
            AutomationEntry::VolumeFade { channel, .. } => assert_eq!(channel, 2),
            _ => panic!("Wrong automation type"),
        }
    }

    #[test]
    fn test_automation_start_row_serialization() {
        let auto = AutomationEntry::VolumeFade {
            pattern: "intro".to_string(),
            channel: 0,
            start_row: 16,
            end_row: 63,
            start_vol: 0,
            end_vol: 64,
        };

        let json = serde_json::to_string(&auto).unwrap();
        let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
        match parsed {
            AutomationEntry::VolumeFade { start_row, .. } => assert_eq!(start_row, 16),
            _ => panic!("Wrong automation type"),
        }
    }

    #[test]
    fn test_automation_end_row_serialization() {
        let auto = AutomationEntry::VolumeFade {
            pattern: "intro".to_string(),
            channel: 0,
            start_row: 0,
            end_row: 48,
            start_vol: 0,
            end_vol: 64,
        };

        let json = serde_json::to_string(&auto).unwrap();
        let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
        match parsed {
            AutomationEntry::VolumeFade { end_row, .. } => assert_eq!(end_row, 48),
            _ => panic!("Wrong automation type"),
        }
    }

    #[test]
    fn test_automation_start_vol_serialization() {
        let auto = AutomationEntry::VolumeFade {
            pattern: "intro".to_string(),
            channel: 0,
            start_row: 0,
            end_row: 63,
            start_vol: 32,
            end_vol: 64,
        };

        let json = serde_json::to_string(&auto).unwrap();
        let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
        match parsed {
            AutomationEntry::VolumeFade { start_vol, .. } => assert_eq!(start_vol, 32),
            _ => panic!("Wrong automation type"),
        }
    }

    #[test]
    fn test_automation_end_vol_serialization() {
        let auto = AutomationEntry::VolumeFade {
            pattern: "intro".to_string(),
            channel: 0,
            start_row: 0,
            end_row: 63,
            start_vol: 0,
            end_vol: 48,
        };

        let json = serde_json::to_string(&auto).unwrap();
        let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
        match parsed {
            AutomationEntry::VolumeFade { end_vol, .. } => assert_eq!(end_vol, 48),
            _ => panic!("Wrong automation type"),
        }
    }

    #[test]
    fn test_automation_tempo_row_serialization() {
        let auto = AutomationEntry::TempoChange {
            pattern: "bridge".to_string(),
            row: 32,
            bpm: 150,
        };

        let json = serde_json::to_string(&auto).unwrap();
        let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
        match parsed {
            AutomationEntry::TempoChange { row, .. } => assert_eq!(row, 32),
            _ => panic!("Wrong automation type"),
        }
    }

    #[test]
    fn test_automation_tempo_bpm_serialization() {
        let auto = AutomationEntry::TempoChange {
            pattern: "outro".to_string(),
            row: 0,
            bpm: 180,
        };

        let json = serde_json::to_string(&auto).unwrap();
        let parsed: AutomationEntry = serde_json::from_str(&json).unwrap();
        match parsed {
            AutomationEntry::TempoChange { bpm, .. } => assert_eq!(bpm, 180),
            _ => panic!("Wrong automation type"),
        }
    }

    // ==================== IT Options Keys Tests ====================

    #[test]
    fn test_it_options_stereo_serialization() {
        let opts = ItOptions {
            stereo: false,
            global_volume: 128,
            mix_volume: 48,
        };

        let json = serde_json::to_string(&opts).unwrap();
        let parsed: ItOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stereo, false);
    }

    #[test]
    fn test_it_options_stereo_default() {
        let opts = ItOptions::default();
        assert_eq!(opts.stereo, true);
    }

    #[test]
    fn test_it_options_global_volume_serialization() {
        let opts = ItOptions {
            stereo: true,
            global_volume: 96,
            mix_volume: 48,
        };

        let json = serde_json::to_string(&opts).unwrap();
        let parsed: ItOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.global_volume, 96);
    }

    #[test]
    fn test_it_options_global_volume_default() {
        let opts = ItOptions::default();
        assert_eq!(opts.global_volume, 128);
    }

    #[test]
    fn test_it_options_mix_volume_serialization() {
        let opts = ItOptions {
            stereo: true,
            global_volume: 128,
            mix_volume: 64,
        };

        let json = serde_json::to_string(&opts).unwrap();
        let parsed: ItOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mix_volume, 64);
    }

    #[test]
    fn test_it_options_mix_volume_default() {
        let opts = ItOptions::default();
        assert_eq!(opts.mix_volume, 48);
    }

    // ==================== Arrangement Entry Tests ====================

    #[test]
    fn test_arrangement_pattern_serialization() {
        let entry = ArrangementEntry {
            pattern: "chorus".to_string(),
            repeat: 4,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: ArrangementEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.pattern, "chorus");
    }

    #[test]
    fn test_arrangement_repeat_serialization() {
        let entry = ArrangementEntry {
            pattern: "verse".to_string(),
            repeat: 3,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: ArrangementEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.repeat, 3);
    }

    #[test]
    fn test_arrangement_default_repeat() {
        let json = r#"{"pattern": "intro"}"#;
        let entry: ArrangementEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.pattern, "intro");
        assert_eq!(entry.repeat, 1);
    }

    // ==================== Full Integration Tests ====================

    #[test]
    fn test_full_song_serialization_roundtrip() {
        let envelope = Envelope {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
        };

        let instrument = TrackerInstrument {
            name: "Lead".to_string(),
            r#ref: None,
            synthesis: Some(InstrumentSynthesis::Pulse { duty_cycle: 0.5 }),
            envelope: envelope.clone(),
            default_volume: Some(64),
        };

        let pattern = TrackerPattern {
            rows: 64,
            data: vec![
                PatternNote {
                    row: 0,
                    channel: 0,
                    note: "C4".to_string(),
                    instrument: 0,
                    volume: Some(64),
                    effect: Some(PatternEffect {
                        r#type: Some("vibrato".to_string()),
                        param: Some(0x44),
                        effect_xy: None,
                    }),
                },
            ],
        };

        let mut patterns = HashMap::new();
        patterns.insert("intro".to_string(), pattern);

        let params = MusicTrackerSongV1Params {
            format: TrackerFormat::Xm,
            bpm: 125,
            speed: 6,
            channels: 8,
            r#loop: true,
            instruments: vec![instrument],
            patterns,
            arrangement: vec![ArrangementEntry {
                pattern: "intro".to_string(),
                repeat: 2,
            }],
            automation: vec![AutomationEntry::VolumeFade {
                pattern: "intro".to_string(),
                channel: 0,
                start_row: 0,
                end_row: 63,
                start_vol: 0,
                end_vol: 64,
            }],
            it_options: None,
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: MusicTrackerSongV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.format, TrackerFormat::Xm);
        assert_eq!(parsed.bpm, 125);
        assert_eq!(parsed.speed, 6);
        assert_eq!(parsed.channels, 8);
        assert_eq!(parsed.r#loop, true);
        assert_eq!(parsed.instruments.len(), 1);
        assert_eq!(parsed.patterns.len(), 1);
        assert_eq!(parsed.arrangement.len(), 1);
        assert_eq!(parsed.automation.len(), 1);
    }

    #[test]
    fn test_full_it_song_with_it_options() {
        let params = MusicTrackerSongV1Params {
            format: TrackerFormat::It,
            bpm: 140,
            speed: 8,
            channels: 16,
            r#loop: false,
            instruments: vec![],
            patterns: HashMap::new(),
            arrangement: vec![],
            automation: vec![],
            it_options: Some(ItOptions {
                stereo: false,
                global_volume: 96,
                mix_volume: 64,
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: MusicTrackerSongV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.format, TrackerFormat::It);
        assert!(parsed.it_options.is_some());
        let opts = parsed.it_options.unwrap();
        assert_eq!(opts.stereo, false);
        assert_eq!(opts.global_volume, 96);
        assert_eq!(opts.mix_volume, 64);
    }
}
