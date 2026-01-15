//! Audio effects for post-processing.
//!
//! This module provides deterministic audio effects that can be chained together
//! after synthesis and mixing.

pub mod auto_filter;
pub mod cabinet;
mod chain;
pub mod chorus;
pub mod delay;
pub mod delay_line;
pub mod distortion;
pub mod dynamics;
pub mod eq;
pub mod flanger;
pub mod granular_delay;
pub mod multi_tap_delay;
pub mod reverb;
pub mod ring_mod;
pub mod rotary;
pub mod stereo;
pub mod tape;
pub mod transient;

use speccade_spec::recipe::audio::{Effect, LfoModulation, ModulationTarget};

use crate::error::AudioResult;
use crate::mixer::{MixerOutput, StereoOutput};
use crate::modulation::lfo::Lfo;
use crate::rng::create_rng;

// Re-export the main entry points
pub use chain::{apply_effect_chain, to_stereo};

/// Applies a chain of effects with post-FX LFO modulation.
///
/// # Arguments
/// * `mixed` - Mixed audio (mono or stereo)
/// * `effects` - Effect chain to apply
/// * `post_fx_lfos` - Post-FX LFO modulations
/// * `sample_rate` - Sample rate in Hz
/// * `seed` - RNG seed for deterministic LFO generation
///
/// # Returns
/// Processed audio with effects and modulation applied
pub fn apply_effect_chain_with_lfos(
    mixed: MixerOutput,
    effects: &[Effect],
    post_fx_lfos: &[LfoModulation],
    sample_rate: f64,
    seed: u32,
) -> AudioResult<MixerOutput> {
    if effects.is_empty() {
        return Ok(mixed);
    }

    let mut stereo = to_stereo(mixed);
    let num_samples = stereo.left.len();

    // Pre-generate LFO curves for all post-FX LFOs
    let delay_time_curve = generate_lfo_curve(
        post_fx_lfos,
        |target| matches!(target, ModulationTarget::DelayTime { .. }),
        |target| match target {
            ModulationTarget::DelayTime { amount_ms } => *amount_ms,
            _ => 0.0,
        },
        "post_fx_delay_time_lfo",
        num_samples,
        sample_rate,
        seed,
    );

    let reverb_size_curve = generate_lfo_curve(
        post_fx_lfos,
        |target| matches!(target, ModulationTarget::ReverbSize { .. }),
        |target| match target {
            ModulationTarget::ReverbSize { amount } => *amount,
            _ => 0.0,
        },
        "post_fx_reverb_size_lfo",
        num_samples,
        sample_rate,
        seed,
    );

    let distortion_drive_curve = generate_lfo_curve(
        post_fx_lfos,
        |target| matches!(target, ModulationTarget::DistortionDrive { .. }),
        |target| match target {
            ModulationTarget::DistortionDrive { amount } => *amount,
            _ => 0.0,
        },
        "post_fx_distortion_drive_lfo",
        num_samples,
        sample_rate,
        seed,
    );

    // Apply each effect in order
    for effect in effects {
        apply_effect_with_lfo(
            &mut stereo,
            effect,
            &delay_time_curve,
            &reverb_size_curve,
            &distortion_drive_curve,
            sample_rate,
            seed,
        )?;
    }

    Ok(MixerOutput::Stereo(stereo))
}

/// Generates an LFO curve for a specific modulation target.
fn generate_lfo_curve<F, G>(
    post_fx_lfos: &[LfoModulation],
    matches_target: F,
    get_amount: G,
    seed_salt: &str,
    num_samples: usize,
    sample_rate: f64,
    seed: u32,
) -> Option<(Vec<f64>, f64)>
where
    F: Fn(&ModulationTarget) -> bool,
    G: Fn(&ModulationTarget) -> f64,
{
    post_fx_lfos
        .iter()
        .find(|lfo| matches_target(&lfo.target))
        .map(|lfo| {
            let lfo_seed = crate::rng::derive_component_seed(seed, seed_salt);
            let mut rng = create_rng(lfo_seed);
            let initial_phase = lfo.config.phase.unwrap_or(0.0);
            let mut lfo_gen = Lfo::new(
                lfo.config.waveform,
                lfo.config.rate,
                sample_rate,
                initial_phase,
            );
            let curve = lfo_gen.generate(num_samples, &mut rng);
            let amount = get_amount(&lfo.target);
            (curve, amount * lfo.config.depth)
        })
}

/// Applies a single effect with optional LFO modulation.
#[allow(clippy::too_many_arguments)]
fn apply_effect_with_lfo(
    stereo: &mut StereoOutput,
    effect: &Effect,
    delay_time_curve: &Option<(Vec<f64>, f64)>,
    reverb_size_curve: &Option<(Vec<f64>, f64)>,
    distortion_drive_curve: &Option<(Vec<f64>, f64)>,
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
            if let Some((ref lfo_curve, amount)) = reverb_size_curve {
                let room_size_mod_curve: Vec<f64> = lfo_curve
                    .iter()
                    .map(|&lfo_value| {
                        let bipolar = (lfo_value - 0.5) * 2.0;
                        (*room_size + bipolar * amount).clamp(0.0, 1.0)
                    })
                    .collect();
                reverb::apply_with_modulation(
                    stereo,
                    &room_size_mod_curve,
                    *damping,
                    *wet,
                    *width,
                    sample_rate,
                )?;
            } else {
                reverb::apply(stereo, *room_size, *damping, *wet, *width, sample_rate)?;
            }
        }
        Effect::Delay {
            time_ms,
            feedback,
            wet,
            ping_pong,
        } => {
            if let Some((ref lfo_curve, amount_ms)) = delay_time_curve {
                let time_curve: Vec<f64> = lfo_curve
                    .iter()
                    .map(|&lfo_value| {
                        let bipolar = (lfo_value - 0.5) * 2.0;
                        (*time_ms + bipolar * amount_ms).clamp(1.0, 2000.0)
                    })
                    .collect();
                delay::apply_with_modulation(
                    stereo,
                    &time_curve,
                    *feedback,
                    *wet,
                    *ping_pong,
                    sample_rate,
                )?;
            } else {
                delay::apply(stereo, *time_ms, *feedback, *wet, *ping_pong, sample_rate)?;
            }
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
            if let Some((ref lfo_curve, amount)) = distortion_drive_curve {
                let drive_mod_curve: Vec<f64> = lfo_curve
                    .iter()
                    .map(|&lfo_value| {
                        let bipolar = (lfo_value - 0.5) * 2.0;
                        (*drive + bipolar * amount).clamp(1.0, 100.0)
                    })
                    .collect();
                distortion::apply_waveshaper_with_modulation(stereo, &drive_mod_curve, curve, *wet);
            } else {
                distortion::apply_waveshaper(stereo, *drive, curve, *wet);
            }
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
            if let Some((ref lfo_curve, amount_ms)) = delay_time_curve {
                flanger::apply_with_modulation(
                    stereo,
                    *rate,
                    *depth,
                    *feedback,
                    delay_ms,
                    lfo_curve,
                    *amount_ms,
                    *wet,
                    sample_rate,
                )?;
            } else {
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
            if matches!(mode, speccade_spec::recipe::audio::StereoWidenerMode::Haas) {
                if let Some((ref lfo_curve, amount_ms)) = delay_time_curve {
                    stereo::apply_with_modulation(
                        stereo,
                        *width,
                        mode,
                        *delay_ms,
                        lfo_curve,
                        *amount_ms,
                        sample_rate,
                    )?;
                } else {
                    stereo::apply(stereo, *width, mode, *delay_ms, sample_rate)?;
                }
            } else {
                stereo::apply(stereo, *width, mode, *delay_ms, sample_rate)?;
            }
        }
        Effect::MultiTapDelay { taps } => {
            if let Some((ref lfo_curve, amount_ms)) = delay_time_curve {
                let time_curves: Vec<Vec<f64>> = taps
                    .iter()
                    .map(|tap| {
                        lfo_curve
                            .iter()
                            .map(|&lfo_value| {
                                let bipolar = (lfo_value - 0.5) * 2.0;
                                (tap.time_ms + bipolar * amount_ms).clamp(1.0, 2000.0)
                            })
                            .collect()
                    })
                    .collect();
                multi_tap_delay::apply_with_modulation(stereo, taps, &time_curves, sample_rate)?;
            } else {
                multi_tap_delay::apply(stereo, taps, sample_rate)?;
            }
        }
        Effect::TapeSaturation {
            drive,
            bias,
            wow_rate,
            flutter_rate,
            hiss_level,
        } => {
            if let Some((ref lfo_curve, amount)) = distortion_drive_curve {
                let drive_mod_curve: Vec<f64> = lfo_curve
                    .iter()
                    .map(|&lfo_value| {
                        let bipolar = (lfo_value - 0.5) * 2.0;
                        (*drive + bipolar * amount).clamp(1.0, 20.0)
                    })
                    .collect();
                tape::apply_with_modulation(
                    stereo,
                    &drive_mod_curve,
                    *bias,
                    *wow_rate,
                    *flutter_rate,
                    *hiss_level,
                    sample_rate,
                    seed,
                )?;
            } else {
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
            if let Some((ref lfo_curve, amount_ms)) = delay_time_curve {
                let time_curve: Vec<f64> = lfo_curve
                    .iter()
                    .map(|&lfo_value| {
                        let bipolar = (lfo_value - 0.5) * 2.0;
                        (*time_ms + bipolar * amount_ms).clamp(10.0, 2000.0)
                    })
                    .collect();
                granular_delay::apply_with_modulation(
                    stereo,
                    &time_curve,
                    *feedback,
                    *grain_size_ms,
                    *pitch_semitones,
                    *wet,
                    sample_rate,
                    seed,
                )?;
            } else {
                granular_delay::apply(
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
    }
    Ok(())
}
