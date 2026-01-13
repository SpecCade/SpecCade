//! Audio effects for post-processing.
//!
//! This module provides deterministic audio effects that can be chained together
//! after synthesis and mixing.

pub mod chorus;
pub mod delay;
pub mod distortion;
pub mod dynamics;
pub mod reverb;

use speccade_spec::recipe::audio::Effect;

use crate::error::AudioResult;
use crate::mixer::{MixerOutput, StereoOutput};

/// Applies a chain of effects to mixed audio.
///
/// # Arguments
/// * `mixed` - Mixed audio (mono or stereo)
/// * `effects` - Effect chain to apply
/// * `sample_rate` - Sample rate in Hz
///
/// # Returns
/// Processed audio with effects applied
pub fn apply_effect_chain(
    mixed: MixerOutput,
    effects: &[Effect],
    sample_rate: f64,
) -> AudioResult<MixerOutput> {
    if effects.is_empty() {
        return Ok(mixed);
    }

    // Convert to stereo for effect processing
    let mut stereo = match mixed {
        MixerOutput::Mono(samples) => StereoOutput {
            left: samples.clone(),
            right: samples,
        },
        MixerOutput::Stereo(stereo) => stereo,
    };

    // Apply each effect in order
    for effect in effects {
        match effect {
            Effect::Reverb {
                room_size,
                damping,
                wet,
                width,
            } => {
                reverb::apply(&mut stereo, *room_size, *damping, *wet, *width, sample_rate)?;
            }
            Effect::Delay {
                time_ms,
                feedback,
                wet,
                ping_pong,
            } => {
                delay::apply(&mut stereo, *time_ms, *feedback, *wet, *ping_pong, sample_rate)?;
            }
            Effect::Chorus {
                rate,
                depth,
                wet,
                voices,
            } => {
                chorus::apply(&mut stereo, *rate, *depth, *wet, *voices, sample_rate)?;
            }
            Effect::Phaser {
                rate,
                depth,
                stages,
                wet,
            } => {
                // Phaser uses chorus module with allpass configuration
                chorus::apply_phaser(&mut stereo, *rate, *depth, *stages, *wet, sample_rate)?;
            }
            Effect::Bitcrush {
                bits,
                sample_rate_reduction,
            } => {
                distortion::apply_bitcrush(&mut stereo, *bits, *sample_rate_reduction);
            }
            Effect::Waveshaper { drive, curve, wet } => {
                distortion::apply_waveshaper(&mut stereo, *drive, curve, *wet);
            }
            Effect::Compressor {
                threshold_db,
                ratio,
                attack_ms,
                release_ms,
                makeup_db,
            } => {
                dynamics::apply_compressor(
                    &mut stereo,
                    *threshold_db,
                    *ratio,
                    *attack_ms,
                    *release_ms,
                    *makeup_db,
                    sample_rate,
                )?;
            }
        }
    }

    Ok(MixerOutput::Stereo(stereo))
}
