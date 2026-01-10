//! Envelope types for audio synthesis.

use serde::{Deserialize, Serialize};

/// ADSR envelope parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    /// Attack time in seconds.
    pub attack: f64,
    /// Decay time in seconds.
    pub decay: f64,
    /// Sustain level (0.0 to 1.0).
    pub sustain: f64,
    /// Release time in seconds.
    pub release: f64,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
        }
    }
}

/// Pitch envelope for modulating frequency over time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PitchEnvelope {
    /// Attack time in seconds.
    pub attack: f64,
    /// Decay time in seconds.
    pub decay: f64,
    /// Sustain level (0.0 to 1.0).
    pub sustain: f64,
    /// Release time in seconds.
    pub release: f64,
    /// Pitch depth in semitones (can be positive or negative).
    pub depth: f64,
}

impl Default for PitchEnvelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
            depth: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Envelope Tests (ADSR)
    // ========================================================================

    #[test]
    fn test_envelope_default() {
        let env = Envelope::default();
        assert_eq!(env.attack, 0.01);
        assert_eq!(env.decay, 0.1);
        assert_eq!(env.sustain, 0.5);
        assert_eq!(env.release, 0.2);
    }

    #[test]
    fn test_envelope_custom_serde() {
        let env = Envelope {
            attack: 0.05,
            decay: 0.2,
            sustain: 0.7,
            release: 0.3,
        };

        let json = serde_json::to_string(&env).unwrap();
        let parsed: Envelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, env);
    }

    // ========================================================================
    // PitchEnvelope Tests
    // ========================================================================

    #[test]
    fn test_pitch_envelope_default() {
        let env = PitchEnvelope::default();
        assert_eq!(env.attack, 0.01);
        assert_eq!(env.decay, 0.1);
        assert_eq!(env.sustain, 0.5);
        assert_eq!(env.release, 0.2);
        assert_eq!(env.depth, 0.0);
    }

    #[test]
    fn test_pitch_envelope_custom_serde() {
        let env = PitchEnvelope {
            attack: 0.02,
            decay: 0.15,
            sustain: 0.7,
            release: 0.25,
            depth: 12.0,
        };

        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("depth"));

        let parsed: PitchEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, env);
    }
}
