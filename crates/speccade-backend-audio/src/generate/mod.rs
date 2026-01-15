//! Main entry point for audio generation.
//!
//! This module takes a spec and generates a WAV file deterministically.

mod converters;
mod filters;
mod layer;
mod modulation;
mod oscillators;

#[cfg(test)]
mod tests;

use speccade_spec::recipe::audio::AudioV1Params;
use speccade_spec::Spec;

use crate::error::{AudioError, AudioResult};
use crate::mixer::{Layer, Mixer, MixerOutput};
use crate::wav::WavResult;

pub use layer::generate_layer;
pub use modulation::{calculate_loop_point, generate_envelope};

/// Result of audio generation.
#[derive(Debug)]
pub struct GenerateResult {
    /// WAV file data.
    pub wav: WavResult,
    /// Number of layers processed.
    pub num_layers: usize,
    /// Base note (MIDI note number) if this is an instrument sample.
    pub base_note: Option<u8>,
    /// Loop point in samples (if generated).
    pub loop_point: Option<usize>,
}

/// Generates audio from a spec.
///
/// # Arguments
/// * `spec` - The specification containing audio parameters
///
/// # Returns
/// Generated WAV file and metadata
pub fn generate(spec: &Spec) -> AudioResult<GenerateResult> {
    let recipe = spec.recipe.as_ref().ok_or(AudioError::MissingRecipe)?;

    match recipe.kind.as_str() {
        "audio_v1" => {
            let params: AudioV1Params =
                serde_json::from_value(recipe.params.clone()).map_err(|e| {
                    AudioError::InvalidRecipeType {
                        expected: "audio_v1".to_string(),
                        found: format!("{}: {}", recipe.kind, e),
                    }
                })?;
            generate_from_params(&params, spec.seed)
        }
        _ => Err(AudioError::InvalidRecipeType {
            expected: "audio_v1".to_string(),
            found: recipe.kind.clone(),
        }),
    }
}

/// Generates audio directly from parameters (without a full `Spec`).
///
/// # Arguments
/// * `params` - Unified audio parameters
/// * `seed` - RNG seed for deterministic generation
///
/// # Returns
/// Generated WAV file and metadata
pub fn generate_from_params(params: &AudioV1Params, seed: u32) -> AudioResult<GenerateResult> {
    generate_from_unified_params(params, seed)
}

/// Generates audio from unified AudioV1Params.
///
/// # Arguments
/// * `params` - Unified audio parameters
/// * `seed` - RNG seed for deterministic generation
///
/// # Returns
/// Generated WAV file and metadata
fn generate_from_unified_params(params: &AudioV1Params, seed: u32) -> AudioResult<GenerateResult> {
    const MAX_AUDIO_DURATION_SECONDS: f64 = 30.0;
    const MAX_AUDIO_LAYERS: usize = 32;
    const MAX_NUM_SAMPLES: usize = (MAX_AUDIO_DURATION_SECONDS as usize) * 48_000;

    match params.sample_rate {
        22050 | 44100 | 48000 => {}
        other => return Err(AudioError::InvalidSampleRate { rate: other }),
    }
    if !params.duration_seconds.is_finite() || params.duration_seconds <= 0.0 {
        return Err(AudioError::InvalidDuration {
            duration: params.duration_seconds,
        });
    }
    if params.duration_seconds > MAX_AUDIO_DURATION_SECONDS {
        return Err(AudioError::invalid_param(
            "duration_seconds",
            format!(
                "must be <= {} seconds, got {}",
                MAX_AUDIO_DURATION_SECONDS, params.duration_seconds
            ),
        ));
    }
    if params.layers.len() > MAX_AUDIO_LAYERS {
        return Err(AudioError::invalid_param(
            "layers",
            format!(
                "must have at most {} entries, got {}",
                MAX_AUDIO_LAYERS,
                params.layers.len()
            ),
        ));
    }

    let sample_rate = params.sample_rate as f64;
    let num_samples_f = params.duration_seconds * sample_rate;
    if !num_samples_f.is_finite() || num_samples_f <= 0.0 {
        return Err(AudioError::InvalidDuration {
            duration: params.duration_seconds,
        });
    }
    let num_samples = num_samples_f.ceil() as usize;
    if num_samples == 0 || num_samples > MAX_NUM_SAMPLES {
        return Err(AudioError::invalid_param(
            "duration_seconds",
            format!(
                "produces too many samples ({} > max {})",
                num_samples, MAX_NUM_SAMPLES
            ),
        ));
    }

    // Get base note as MIDI note number (None means tracker uses a format default)
    use speccade_spec::recipe::audio::NoteSpec as UnifiedNoteSpec;
    let base_note_midi: Option<u8> = match &params.base_note {
        Some(UnifiedNoteSpec::MidiNote(n)) => Some(*n),
        Some(UnifiedNoteSpec::NoteName(name)) => {
            speccade_spec::recipe::audio::parse_note_name(name)
        }
        None => None, // Tracker uses native default (IT: C5, XM: C4)
    };

    let mut mixer = Mixer::new(num_samples, sample_rate);

    // Process each layer
    for (layer_idx, layer) in params.layers.iter().enumerate() {
        let layer_seed = crate::rng::derive_layer_seed(seed, layer_idx as u32);
        let mut layer_samples =
            generate_layer(layer, layer_idx, num_samples, sample_rate, layer_seed)?;

        // Apply pitch envelope if specified
        if let Some(ref pitch_env) = params.pitch_envelope {
            let pitch_curve = modulation::generate_pitch_envelope_curve(pitch_env, sample_rate, num_samples);
            layer_samples = modulation::apply_pitch_envelope_to_layer_samples(
                layer,
                layer_idx,
                &pitch_curve,
                num_samples,
                sample_rate,
                layer_seed,
            )?;
        }

        let mut mix_layer = Layer::new(layer_samples, layer.volume, layer.pan);

        // Pan LFO is applied during mixing. Keep it deterministic and aligned to layer start:
        // delay time does not advance LFO phase.
        if let Some(lfo_mod) = &layer.lfo {
            if let speccade_spec::recipe::audio::ModulationTarget::Pan { amount } = &lfo_mod.target
            {
                use crate::modulation::lfo::{apply_pan_modulation, Lfo};

                let initial_phase = lfo_mod.config.phase.unwrap_or(0.0);
                let mut lfo = Lfo::new(
                    lfo_mod.config.waveform,
                    lfo_mod.config.rate,
                    sample_rate,
                    initial_phase,
                );
                let lfo_seed = crate::rng::derive_component_seed(layer_seed, "layer_pan_lfo");
                let mut lfo_rng = crate::rng::create_rng(lfo_seed);

                let delay_samples = layer
                    .delay
                    .map(|delay| (delay.max(0.0) * sample_rate).floor() as usize)
                    .unwrap_or(0)
                    .min(num_samples);

                let mut pan_curve = vec![layer.pan.clamp(-1.0, 1.0); num_samples];
                for i in delay_samples..num_samples {
                    let lfo_value = lfo.next_sample(&mut lfo_rng);
                    pan_curve[i] = apply_pan_modulation(
                        layer.pan,
                        lfo_value,
                        *amount,
                        lfo_mod.config.depth,
                    );
                }

                mix_layer = mix_layer.with_pan_curve(pan_curve);
            }
        }

        mixer.add_layer(mix_layer);
    }

    // Mix layers
    let mut mixed = mixer.mix();

    // Apply master filter if specified
    if let Some(ref master_filter) = params.master_filter {
        mixed = match mixed {
            MixerOutput::Mono(mut samples) => {
                filters::apply_swept_filter(&mut samples, master_filter, sample_rate);
                MixerOutput::Mono(samples)
            }
            MixerOutput::Stereo(mut stereo) => {
                filters::apply_swept_filter(&mut stereo.left, master_filter, sample_rate);
                filters::apply_swept_filter(&mut stereo.right, master_filter, sample_rate);
                MixerOutput::Stereo(stereo)
            }
        };
    }

    // Apply effect chain if specified
    if !params.effects.is_empty() {
        mixed = crate::effects::apply_effect_chain(mixed, &params.effects, sample_rate)?;
    }

    // Determine loop point if requested
    let loop_point = if params.generate_loop_points && !params.layers.is_empty() {
        // Use first layer's envelope for loop point calculation
        let first_layer = &params.layers[0];
        Some(calculate_loop_point(&first_layer.envelope, sample_rate))
    } else {
        None
    };

    // Convert to WAV
    let wav = match mixed {
        MixerOutput::Mono(mut samples) => {
            crate::mixer::normalize(&mut samples, -3.0);
            WavResult::from_mono(&samples, params.sample_rate)
        }
        MixerOutput::Stereo(mut stereo) => {
            crate::mixer::normalize_stereo(&mut stereo, -3.0);
            WavResult::from_stereo_output(&stereo, params.sample_rate)
        }
    };

    Ok(GenerateResult {
        wav,
        num_layers: params.layers.len(),
        base_note: base_note_midi,
        loop_point,
    })
}
