//! Audio layer types for layered synthesis.

use serde::{Deserialize, Serialize};

use super::envelope::Envelope;
use super::synthesis::Synthesis;

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
    /// Layer start delay in seconds (default: 0.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::audio_sfx::common::Waveform;

    // ========================================================================
    // AudioLayer Tests (amplitude, delay, envelope, volume, pan)
    // ========================================================================

    #[test]
    fn test_audio_layer_serde() {
        let layer = AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Square,
                frequency: 880.0,
                freq_sweep: None,
                detune: None,
                duty: Some(0.5),
            },
            envelope: Envelope {
                attack: 0.02,
                decay: 0.15,
                sustain: 0.6,
                release: 0.3,
            },
            volume: 0.75,
            pan: -0.5,
            delay: Some(0.25),
        };

        let json = serde_json::to_string(&layer).unwrap();
        let parsed: AudioLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layer);
    }

    #[test]
    fn test_audio_layer_no_delay() {
        let layer = AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 1.0,
            pan: 0.0,
            delay: None,
        };

        let json = serde_json::to_string(&layer).unwrap();
        assert!(!json.contains("delay"));
        let parsed: AudioLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layer);
    }
}
