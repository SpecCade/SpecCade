//! Effect chain application without LFO modulation.
//!
//! This module provides the basic effect chain application logic.

use speccade_spec::recipe::audio::Effect;

use super::{
    auto_filter, cabinet, chorus, delay, distortion, dynamics, eq, flanger, multi_tap_delay,
    reverb, ring_mod, rotary, stereo, tape, transient,
};
use crate::error::AudioResult;
use crate::mixer::{MixerOutput, StereoOutput};

/// Converts mixer output to stereo for effect processing.
pub fn to_stereo(mixed: MixerOutput) -> StereoOutput {
    match mixed {
        MixerOutput::Mono(samples) => StereoOutput {
            left: samples.clone(),
            right: samples,
        },
        MixerOutput::Stereo(stereo) => stereo,
    }
}

/// Applies a chain of effects to mixed audio.
///
/// # Arguments
/// * `mixed` - Mixed audio (mono or stereo)
/// * `effects` - Effect chain to apply
/// * `sample_rate` - Sample rate in Hz
/// * `seed` - RNG seed for deterministic effect generation (e.g., tape hiss)
///
/// # Returns
/// Processed audio with effects applied
pub fn apply_effect_chain(
    mixed: MixerOutput,
    effects: &[Effect],
    sample_rate: f64,
    seed: u32,
) -> AudioResult<MixerOutput> {
    if effects.is_empty() {
        return Ok(mixed);
    }

    let mut stereo = to_stereo(mixed);
    for effect in effects {
        apply_single_effect(&mut stereo, effect, sample_rate, seed)?;
    }

    Ok(MixerOutput::Stereo(stereo))
}

/// Applies a single effect to stereo audio.
///
/// # Arguments
/// * `stereo` - Stereo audio buffer to process in place
/// * `effect` - Effect to apply
/// * `sample_rate` - Sample rate in Hz
/// * `seed` - RNG seed for deterministic effect generation
pub fn apply_single_effect(
    stereo: &mut StereoOutput,
    effect: &Effect,
    sample_rate: f64,
    seed: u32,
) -> AudioResult<()> {
    match effect {
        Effect::ParametricEq { bands } => {
            eq::apply(stereo, bands, sample_rate);
        }
        Effect::Reverb {
            room_size,
            damping,
            wet,
            width,
        } => {
            reverb::apply(stereo, *room_size, *damping, *wet, *width, sample_rate)?;
        }
        Effect::Delay {
            time_ms,
            feedback,
            wet,
            ping_pong,
        } => {
            delay::apply(stereo, *time_ms, *feedback, *wet, *ping_pong, sample_rate)?;
        }
        Effect::Chorus {
            rate,
            depth,
            wet,
            voices,
        } => {
            chorus::apply(stereo, *rate, *depth, *wet, *voices, sample_rate)?;
        }
        Effect::Phaser {
            rate,
            depth,
            stages,
            wet,
        } => {
            chorus::apply_phaser(stereo, *rate, *depth, *stages, *wet, sample_rate)?;
        }
        Effect::Bitcrush {
            bits,
            sample_rate_reduction,
        } => {
            distortion::apply_bitcrush(stereo, *bits, *sample_rate_reduction);
        }
        Effect::Waveshaper { drive, curve, wet } => {
            distortion::apply_waveshaper(stereo, *drive, curve, *wet);
        }
        Effect::Compressor {
            threshold_db,
            ratio,
            attack_ms,
            release_ms,
            makeup_db,
        } => {
            dynamics::apply_compressor(
                stereo,
                *threshold_db,
                *ratio,
                *attack_ms,
                *release_ms,
                *makeup_db,
                sample_rate,
            )?;
        }
        Effect::Flanger {
            rate,
            depth,
            feedback,
            delay_ms,
            wet,
        } => {
            flanger::apply(
                stereo,
                *rate,
                *depth,
                *feedback,
                *delay_ms,
                *wet,
                sample_rate,
            )?;
        }
        Effect::Limiter {
            threshold_db,
            release_ms,
            lookahead_ms,
            ceiling_db,
        } => {
            dynamics::apply_limiter(
                stereo,
                *threshold_db,
                *release_ms,
                *lookahead_ms,
                *ceiling_db,
                sample_rate,
            )?;
        }
        Effect::GateExpander {
            threshold_db,
            ratio,
            attack_ms,
            hold_ms,
            release_ms,
            range_db,
        } => {
            dynamics::apply_gate_expander(
                stereo,
                *threshold_db,
                *ratio,
                *attack_ms,
                *hold_ms,
                *release_ms,
                *range_db,
                sample_rate,
            )?;
        }
        Effect::StereoWidener {
            width,
            mode,
            delay_ms,
        } => {
            stereo::apply(stereo, *width, mode, *delay_ms, sample_rate)?;
        }
        Effect::MultiTapDelay { taps } => {
            multi_tap_delay::apply(stereo, taps, sample_rate)?;
        }
        Effect::TapeSaturation {
            drive,
            bias,
            wow_rate,
            flutter_rate,
            hiss_level,
        } => {
            tape::apply(
                stereo,
                *drive,
                *bias,
                *wow_rate,
                *flutter_rate,
                *hiss_level,
                sample_rate,
                seed,
            )?;
        }
        Effect::TransientShaper {
            attack,
            sustain,
            output_gain_db,
        } => {
            transient::apply(stereo, *attack, *sustain, *output_gain_db, sample_rate)?;
        }
        Effect::AutoFilter {
            sensitivity,
            attack_ms,
            release_ms,
            depth,
            base_frequency,
        } => {
            auto_filter::apply(
                stereo,
                *sensitivity,
                *attack_ms,
                *release_ms,
                *depth,
                *base_frequency,
                sample_rate,
            )?;
        }
        Effect::CabinetSim {
            cabinet_type,
            mic_position,
        } => {
            cabinet::apply(stereo, *cabinet_type, *mic_position, sample_rate)?;
        }
        Effect::RotarySpeaker { rate, depth, wet } => {
            rotary::apply(stereo, *rate, *depth, *wet, sample_rate)?;
        }
        Effect::RingModulator { frequency, mix } => {
            ring_mod::apply(stereo, *frequency, *mix, sample_rate)?;
        }
        Effect::GranularDelay {
            time_ms,
            feedback,
            grain_size_ms,
            pitch_semitones,
            wet,
        } => {
            super::granular_delay::apply(
                stereo,
                *time_ms,
                *feedback,
                *grain_size_ms,
                *pitch_semitones,
                *wet,
                sample_rate,
                seed,
            )?;
        }
    }
    Ok(())
}
