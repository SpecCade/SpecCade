//! Envelope conversion utilities for tracker formats.
//!
//! This module handles converting SpecCade ADSR envelopes to format-specific
//! envelope representations (XM and IT).
//!
//! The conversion uses a generic approach with the `EnvelopePoint` trait,
//! allowing shared ADSR calculation logic for both XM and IT formats.

use speccade_spec::recipe::audio::Envelope;

use crate::it::{env_flags, ItEnvelope, ItEnvelopePoint};
use crate::xm::{XmEnvelope, XmEnvelopePoint};

/// Ticks per second for envelope timing calculations.
///
/// This value approximates typical tracker timing at default tempo.
const TICKS_PER_SEC: f64 = 50.0;

fn seconds_to_ticks(seconds: f64) -> u16 {
    if seconds <= 0.0 {
        return 0;
    }
    let ticks = (seconds * TICKS_PER_SEC).round();
    ticks.clamp(0.0, u16::MAX as f64) as u16
}

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
        XmEnvelopePoint { frame: tick, value }
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
    // Trackers operate on relatively coarse time units (frames/ticks). If we directly truncate
    // (or allow 0-length segments), very short envelopes can collapse to duplicate points, which
    // some players interpret as silence. We round to the nearest tick and ensure that point times
    // are strictly increasing.
    //
    // Special case: if `attack` rounds to 0 ticks, starting at 0 volume would effectively mute the
    // first tick in many players. For percussive instruments this can make the sound inaudible, so
    // we start at full volume at tick 0 when attack is 0 ticks.
    let attack_ticks = seconds_to_ticks(envelope.attack);
    let decay_ticks = seconds_to_ticks(envelope.decay);
    let release_ticks = seconds_to_ticks(envelope.release);
    let sustain_value = (envelope.sustain * 64.0).round().clamp(0.0, 64.0) as u16;

    let mut points = Vec::new();

    let mut current_tick: u16 = 0;

    if attack_ticks == 0 {
        // Immediate attack: start at full volume at tick 0.
        points.push(P::new(0, 64));
    } else {
        // Attack: 0 -> 64
        points.push(P::new(0, 0));
        points.push(P::new(attack_ticks, 64));
        current_tick = attack_ticks;
    }

    // If sustain is 0, treat this as a one-shot envelope (AD) where `release` extends the tail.
    // This matches common expectations for percussive instruments in trackers (no explicit note-off).
    if envelope.sustain <= 0.0 {
        let tail_ticks = decay_ticks.saturating_add(release_ticks);
        let end = current_tick
            .saturating_add(tail_ticks.max(1))
            .max(current_tick.saturating_add(1));
        points.push(P::new(end, 0));
        return points;
    }

    // Decay: 64 -> sustain
    let decay_end = current_tick
        .saturating_add(decay_ticks.max(1))
        .max(current_tick.saturating_add(1));
    points.push(P::new(decay_end, sustain_value));

    // Release: sustain -> 0 (after note-off)
    let release_end = decay_end
        .saturating_add(release_ticks.max(1))
        .max(decay_end.saturating_add(1));
    points.push(P::new(release_end, 0));

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
    let sustain_enabled = envelope.sustain > 0.0;
    let points: Vec<XmEnvelopePoint> = calculate_adsr_points(envelope);
    let sustain_point = if sustain_enabled {
        // Sustain at the last non-zero point (the decay end).
        (points.len().saturating_sub(2)) as u8
    } else {
        0
    };
    XmEnvelope {
        points,
        sustain_point,
        loop_start: 0,
        loop_end: 0,
        enabled: true,
        sustain_enabled,
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
    let sustain_enabled = envelope.sustain > 0.0;
    let flags = if sustain_enabled {
        env_flags::ENABLED | env_flags::SUSTAIN_LOOP
    } else {
        env_flags::ENABLED
    };

    let points: Vec<ItEnvelopePoint> = calculate_adsr_points(envelope);
    let sustain_point = if sustain_enabled {
        (points.len().saturating_sub(2)) as u8
    } else {
        0
    };

    ItEnvelope {
        flags,
        points,
        loop_begin: 0,
        loop_end: 0,
        sustain_begin: sustain_point,
        sustain_end: sustain_point,
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
        assert_eq!(xm_env.points.len(), 4);
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
        assert_eq!(it_env.points.len(), 4);
        assert_eq!(it_env.sustain_begin, 2);
        assert_eq!(it_env.sustain_end, 2);

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
        // Immediate attack should start at full volume
        assert_eq!(xm_env.points[0].frame, 0);
        assert_eq!(xm_env.points[0].value, 64);

        let it_env = convert_envelope_to_it(&env);
        assert_eq!(it_env.points[0].tick, 0);
        assert_eq!(it_env.points[0].value, 64);
    }

    #[test]
    fn test_one_shot_envelope_disables_sustain_and_is_monotonic() {
        let env = Envelope {
            attack: 0.001,
            decay: 0.02,
            sustain: 0.0,
            release: 0.015,
        };

        let xm_env = convert_envelope_to_xm(&env);
        assert!(xm_env.enabled);
        assert!(!xm_env.sustain_enabled);
        assert!(xm_env.points.len() >= 2);
        for w in xm_env.points.windows(2) {
            assert!(w[0].frame < w[1].frame);
        }

        let it_env = convert_envelope_to_it(&env);
        assert_eq!(it_env.flags, env_flags::ENABLED);
        assert!(it_env.points.len() >= 2);
        for w in it_env.points.windows(2) {
            assert!(w[0].tick < w[1].tick);
        }
    }
}
