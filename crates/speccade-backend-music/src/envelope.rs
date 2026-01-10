//! Envelope conversion utilities for tracker formats.
//!
//! This module handles converting SpecCade ADSR envelopes to format-specific
//! envelope representations (XM and IT).
//!
//! The conversion uses a generic approach with the [`EnvelopePoint`] trait,
//! allowing shared ADSR calculation logic for both XM and IT formats.

use speccade_spec::recipe::audio_sfx::Envelope;

use crate::it::{env_flags, ItEnvelope, ItEnvelopePoint};
use crate::xm::{XmEnvelope, XmEnvelopePoint};

/// Ticks per second for envelope timing calculations.
///
/// This value approximates typical tracker timing at default tempo.
const TICKS_PER_SEC: f64 = 50.0;

/// Trait for envelope point types that can be constructed from tick and value.
///
/// This enables generic ADSR envelope calculation across different tracker formats.
trait EnvelopePoint {
    /// Create a new envelope point at the given tick position with the given value.
    ///
    /// # Arguments
    /// * `tick` - The tick/frame position in the envelope
    /// * `value` - The envelope value (0-64 range, will be converted to format-specific type)
    fn new(tick: u16, value: u16) -> Self;
}

impl EnvelopePoint for XmEnvelopePoint {
    fn new(tick: u16, value: u16) -> Self {
        XmEnvelopePoint {
            frame: tick,
            value,
        }
    }
}

impl EnvelopePoint for ItEnvelopePoint {
    fn new(tick: u16, value: u16) -> Self {
        ItEnvelopePoint {
            tick,
            value: value as i8,
        }
    }
}

/// Calculate ADSR envelope points in a format-agnostic way.
///
/// This is the core envelope calculation logic shared by both XM and IT converters.
/// The `sustain_value` is passed separately to allow format-specific type handling.
///
/// # Arguments
/// * `envelope` - Source ADSR envelope with times in seconds
///
/// # Returns
/// A tuple of (points, sustain_value) where points is a Vec of envelope points
fn calculate_adsr_points<P: EnvelopePoint>(envelope: &Envelope) -> Vec<P> {
    let attack_ticks = (envelope.attack * TICKS_PER_SEC) as u16;
    let decay_ticks = (envelope.decay * TICKS_PER_SEC) as u16;
    let release_ticks = (envelope.release * TICKS_PER_SEC) as u16;
    let sustain_value = (envelope.sustain * 64.0) as u16;

    let mut points = Vec::new();

    // Attack: 0 -> 64
    points.push(P::new(0, 0));
    points.push(P::new(attack_ticks.max(1), 64));

    // Decay: 64 -> sustain
    let decay_end = attack_ticks + decay_ticks;
    points.push(P::new(decay_end.max(attack_ticks + 1), sustain_value));

    // Sustain hold point
    let sustain_end = decay_end + 100;
    points.push(P::new(sustain_end, sustain_value));

    // Release: sustain -> 0
    points.push(P::new(sustain_end + release_ticks, 0));

    points
}

/// Convert ADSR envelope to XM envelope format.
///
/// XM envelopes use frame-based timing with 16-bit values.
/// The sustain point is set at the decay end for proper note-off behavior.
///
/// # Arguments
/// * `envelope` - Source ADSR envelope with times in seconds
///
/// # Returns
/// XM-formatted envelope with points, sustain, and loop settings
pub fn convert_envelope_to_xm(envelope: &Envelope) -> XmEnvelope {
    XmEnvelope {
        points: calculate_adsr_points(envelope),
        sustain_point: 2,
        loop_start: 0,
        loop_end: 0,
        enabled: true,
        sustain_enabled: true,
        loop_enabled: false,
    }
}

/// Convert ADSR envelope to IT envelope format.
///
/// IT envelopes use tick-based timing with signed 8-bit values.
/// Sustain loop is set between decay end and sustain hold points.
///
/// # Arguments
/// * `envelope` - Source ADSR envelope with times in seconds
///
/// # Returns
/// IT-formatted envelope with points, sustain loop, and flags
pub fn convert_envelope_to_it(envelope: &Envelope) -> ItEnvelope {
    ItEnvelope {
        flags: env_flags::ENABLED | env_flags::SUSTAIN_LOOP,
        points: calculate_adsr_points(envelope),
        loop_begin: 0,
        loop_end: 0,
        sustain_begin: 2,
        sustain_end: 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_envelope() -> Envelope {
        Envelope {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
        }
    }

    #[test]
    fn test_xm_envelope_conversion() {
        let env = test_envelope();
        let xm_env = convert_envelope_to_xm(&env);

        assert!(xm_env.enabled);
        assert!(xm_env.sustain_enabled);
        assert!(!xm_env.loop_enabled);
        assert_eq!(xm_env.points.len(), 5);
        assert_eq!(xm_env.sustain_point, 2);

        // Check attack starts at 0
        assert_eq!(xm_env.points[0].frame, 0);
        assert_eq!(xm_env.points[0].value, 0);

        // Check attack peak
        assert_eq!(xm_env.points[1].value, 64);

        // Check sustain value (0.5 * 64 = 32)
        assert_eq!(xm_env.points[2].value, 32);
    }

    #[test]
    fn test_it_envelope_conversion() {
        let env = test_envelope();
        let it_env = convert_envelope_to_it(&env);

        assert_eq!(it_env.flags, env_flags::ENABLED | env_flags::SUSTAIN_LOOP);
        assert_eq!(it_env.points.len(), 5);
        assert_eq!(it_env.sustain_begin, 2);
        assert_eq!(it_env.sustain_end, 3);

        // Check attack starts at 0
        assert_eq!(it_env.points[0].tick, 0);
        assert_eq!(it_env.points[0].value, 0);

        // Check attack peak
        assert_eq!(it_env.points[1].value, 64);

        // Check sustain value (0.5 * 64 = 32)
        assert_eq!(it_env.points[2].value, 32);
    }

    #[test]
    fn test_zero_attack_envelope() {
        let env = Envelope {
            attack: 0.0,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
        };

        let xm_env = convert_envelope_to_xm(&env);
        // Attack should be clamped to at least 1 tick
        assert!(xm_env.points[1].frame >= 1);

        let it_env = convert_envelope_to_it(&env);
        assert!(it_env.points[1].tick >= 1);
    }
}
