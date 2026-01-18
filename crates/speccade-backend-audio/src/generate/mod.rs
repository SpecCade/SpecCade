//! Main entry point for audio generation.
//!
//! This module takes a spec and generates a WAV file deterministically.

mod converters;
mod filters;
mod layer;
mod lfo_granular;
mod lfo_modulation;
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

/// Generates a preview of audio from a spec with truncated duration.
///
/// # Arguments
/// * `spec` - The specification containing audio parameters
/// * `preview_duration` - Maximum duration in seconds for the preview
///
/// # Returns
/// Generated WAV file (truncated) and metadata
pub fn generate_preview(spec: &Spec, preview_duration: f64) -> AudioResult<GenerateResult> {
    let recipe = spec.recipe.as_ref().ok_or(AudioError::MissingRecipe)?;

    match recipe.kind.as_str() {
        "audio_v1" => {
            let mut params: AudioV1Params =
                serde_json::from_value(recipe.params.clone()).map_err(|e| {
                    AudioError::InvalidRecipeType {
                        expected: "audio_v1".to_string(),
                        found: format!("{}: {}", recipe.kind, e),
                    }
                })?;

            // Truncate duration to preview_duration
            if params.duration_seconds > preview_duration {
                params.duration_seconds = preview_duration;
            }

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

        // Check if this is a SupersawUnison layer that needs expansion
        if let speccade_spec::recipe::audio::Synthesis::SupersawUnison {
            frequency,
            voices,
            detune_cents,
            spread,
            detune_curve,
        } = &layer.synthesis
        {
            // Expand SupersawUnison into N virtual layers
            let supersaw_params = SupersawParams {
                frequency: *frequency,
                voices: *voices,
                detune_cents: *detune_cents,
                spread: *spread,
                detune_curve: *detune_curve,
            };
            let virtual_layers = generate_supersaw_virtual_layers(
                layer,
                layer_idx,
                num_samples,
                sample_rate,
                layer_seed,
                supersaw_params,
            )?;

            for virtual_layer in virtual_layers {
                mixer.add_layer(virtual_layer);
            }
            continue;
        }

        let mut layer_samples =
            generate_layer(layer, layer_idx, num_samples, sample_rate, layer_seed)?;

        // Apply pitch envelope if specified
        if let Some(ref pitch_env) = params.pitch_envelope {
            let pitch_curve =
                modulation::generate_pitch_envelope_curve(pitch_env, sample_rate, num_samples);
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
                for pan_sample in pan_curve.iter_mut().take(num_samples).skip(delay_samples) {
                    let lfo_value = lfo.next_sample(&mut lfo_rng);
                    *pan_sample =
                        apply_pan_modulation(layer.pan, lfo_value, *amount, lfo_mod.config.depth);
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
        if params.post_fx_lfos.is_empty() {
            mixed = crate::effects::apply_effect_chain(mixed, &params.effects, sample_rate, seed)?;
        } else {
            mixed = crate::effects::apply_effect_chain_with_lfos(
                mixed,
                &params.effects,
                &params.post_fx_lfos,
                sample_rate,
                seed,
            )?;
        }
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

/// Parameters for supersaw voice expansion.
struct SupersawParams {
    frequency: f64,
    voices: u8,
    detune_cents: f64,
    spread: f64,
    detune_curve: speccade_spec::recipe::audio::DetuneCurve,
}

/// Generates virtual layers for SupersawUnison synthesis.
///
/// Expands a single SupersawUnison layer into N virtual layers (one per voice),
/// each with appropriate detuning and panning.
fn generate_supersaw_virtual_layers(
    layer: &speccade_spec::recipe::audio::AudioLayer,
    layer_idx: usize,
    num_samples: usize,
    sample_rate: f64,
    layer_seed: u32,
    params: SupersawParams,
) -> AudioResult<Vec<Layer>> {
    use crate::synthesis::oscillators::SawSynth;
    use crate::synthesis::Synthesizer;
    use speccade_spec::recipe::audio::DetuneCurve;

    let n = (params.voices as usize).max(1);
    let voice_volume = layer.volume / (n as f64);

    let mut virtual_layers = Vec::with_capacity(n);

    // Calculate delay samples once
    let delay_samples = match layer.delay {
        Some(delay) => {
            if !delay.is_finite() || delay < 0.0 {
                return Err(AudioError::invalid_param(
                    format!("layers[{}].delay", layer_idx),
                    format!("must be finite and non-negative, got {}", delay),
                ));
            }
            let delay_samples_f = delay * sample_rate;
            if !delay_samples_f.is_finite() || delay_samples_f < 0.0 {
                return Err(AudioError::invalid_param(
                    format!("layers[{}].delay", layer_idx),
                    "produced an invalid sample count",
                ));
            }
            let delay_samples = delay_samples_f.floor() as usize;
            if delay_samples > num_samples {
                return Err(AudioError::invalid_param(
                    format!("layers[{}].delay", layer_idx),
                    format!(
                        "delay exceeds duration ({} samples > {} total)",
                        delay_samples, num_samples
                    ),
                ));
            }
            delay_samples
        }
        None => 0,
    };
    let synthesis_samples = num_samples.saturating_sub(delay_samples);

    // Generate envelope once for reuse
    let envelope = modulation::generate_envelope(&layer.envelope, sample_rate, synthesis_samples);

    for voice_idx in 0..n {
        // Derive unique seed for this voice
        let voice_seed =
            crate::rng::derive_component_seed(layer_seed, &format!("supersaw_voice_{}", voice_idx));
        let mut rng = crate::rng::create_rng(voice_seed);

        // Calculate normalized position x in [-1, 1]
        let x = if n == 1 {
            0.0
        } else {
            -1.0 + 2.0 * (voice_idx as f64) / ((n - 1) as f64)
        };

        // Calculate detune offset based on curve
        let detune_offset_cents = match params.detune_curve {
            DetuneCurve::Linear => x * params.detune_cents,
            DetuneCurve::Exp2 => x.signum() * (x.abs() * x.abs()) * params.detune_cents,
        };

        // Calculate voice frequency with detune
        let voice_freq = params.frequency * 2.0_f64.powf(detune_offset_cents / 1200.0);

        // Calculate voice pan
        let voice_pan = (layer.pan + x * params.spread).clamp(-1.0, 1.0);

        // Synthesize sawtooth oscillator
        let synth = SawSynth::new(voice_freq);
        let mut samples = synth.synthesize(synthesis_samples, sample_rate, &mut rng);

        // Apply envelope
        for (sample, env) in samples.iter_mut().zip(envelope.iter()) {
            *sample *= env;
        }

        // Pad with delay at the start if needed
        if delay_samples > 0 {
            let mut padded = vec![0.0; delay_samples];
            padded.extend(samples);
            samples = padded;
        }

        let mix_layer = Layer::new(samples, voice_volume, voice_pan);
        virtual_layers.push(mix_layer);
    }

    Ok(virtual_layers)
}
