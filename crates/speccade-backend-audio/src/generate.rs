//! Main entry point for audio generation.
//!
//! This module takes a spec and generates a WAV file deterministically.

use speccade_spec::recipe::audio::{
    AudioLayer, AudioV1Params, Envelope, Filter, FormantVowel, FreqSweep, ModalExcitation,
    ModulationTarget, NoiseType, OscillatorConfig, PdWaveform, PitchEnvelope, SweepCurve,
    Synthesis, VectorSourceType, VocoderBandSpacing, VocoderCarrierType, Waveform,
};
use speccade_spec::Spec;

use crate::envelope::{AdsrEnvelope, AdsrParams};
use crate::error::{AudioError, AudioResult};
use crate::mixer::{Layer, Mixer, MixerOutput};
use crate::rng::create_rng;
use crate::synthesis::am::AmSynth;
use crate::synthesis::fm::FmSynth;
use crate::synthesis::formant::{
    Formant as FormantImpl, FormantSynth, VowelPreset as FormantVowelPresetImpl,
};
use crate::synthesis::granular::GranularSynth;
use crate::synthesis::harmonics::HarmonicSynth;
use crate::synthesis::karplus::KarplusStrong;
use crate::synthesis::metallic::MetallicSynth;
use crate::synthesis::modal::{Excitation as ModalExcitationImpl, Mode, ModalSynth};
use crate::synthesis::noise::{NoiseColor, NoiseSynth};
use crate::synthesis::oscillators::{SawSynth, SineSynth, SquareSynth, TriangleSynth};
use crate::synthesis::phase_distortion::{PdWaveform as PdWaveformImpl, PhaseDistortionSynth};
use crate::synthesis::pitched_body::PitchedBody;
use crate::synthesis::ring_mod::RingModSynth;
use crate::synthesis::vector::{
    VectorPath, VectorPathPoint as VectorPathPointImpl, VectorPosition,
    VectorSource as VectorSourceImpl, VectorSourceType as VectorSourceTypeImpl, VectorSynth,
};
use crate::synthesis::vocoder::{
    BandSpacing as VocoderBandSpacingImpl, CarrierType as VocoderCarrierTypeImpl,
    VocoderBand as VocoderBandImpl, VocoderSynth,
};
use crate::synthesis::wavetable::{PositionSweep as WavetablePositionSweep, WavetableSynth};
use crate::synthesis::{FrequencySweep, Synthesizer};
use crate::wav::WavResult;

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
            let pitch_curve = generate_pitch_envelope_curve(pitch_env, sample_rate, num_samples);
            layer_samples = apply_pitch_envelope_to_layer_samples(
                layer,
                layer_idx,
                &pitch_curve,
                num_samples,
                sample_rate,
                layer_seed,
            )?;
        }

        mixer.add_layer(Layer::new(layer_samples, layer.volume, layer.pan));
    }

    // Mix layers
    let mut mixed = mixer.mix();

    // Apply master filter if specified
    if let Some(ref master_filter) = params.master_filter {
        mixed = match mixed {
            MixerOutput::Mono(mut samples) => {
                apply_swept_filter(&mut samples, master_filter, sample_rate);
                MixerOutput::Mono(samples)
            }
            MixerOutput::Stereo(mut stereo) => {
                apply_swept_filter(&mut stereo.left, master_filter, sample_rate);
                apply_swept_filter(&mut stereo.right, master_filter, sample_rate);
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

/// Generates a single audio layer.
fn generate_layer(
    layer: &AudioLayer,
    layer_idx: usize,
    num_samples: usize,
    sample_rate: f64,
    seed: u32,
) -> AudioResult<Vec<f64>> {
    let mut rng = create_rng(seed);

    // Calculate delay padding
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

    // Generate base synthesis
    let mut samples = match &layer.synthesis {
        Synthesis::FmSynth {
            carrier_freq,
            modulator_freq,
            modulation_index,
            freq_sweep,
        } => {
            let mut synth = FmSynth::new(*carrier_freq, *modulator_freq, *modulation_index);

            if let Some(sweep) = freq_sweep {
                let curve = convert_sweep_curve(&sweep.curve);
                synth = synth.with_sweep(FrequencySweep::new(*carrier_freq, sweep.end_freq, curve));
            }

            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::AmSynth {
            carrier_freq,
            modulator_freq,
            modulation_depth,
            freq_sweep,
        } => {
            let mut synth = AmSynth::new(*carrier_freq, *modulator_freq, *modulation_depth);

            if let Some(sweep) = freq_sweep {
                let curve = convert_sweep_curve(&sweep.curve);
                synth = synth.with_sweep(FrequencySweep::new(*carrier_freq, sweep.end_freq, curve));
            }

            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::RingModSynth {
            carrier_freq,
            modulator_freq,
            mix,
            freq_sweep,
        } => {
            let mut synth = RingModSynth::new(*carrier_freq, *modulator_freq, *mix);

            if let Some(sweep) = freq_sweep {
                let curve = convert_sweep_curve(&sweep.curve);
                synth = synth.with_sweep(FrequencySweep::new(*carrier_freq, sweep.end_freq, curve));
            }

            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::KarplusStrong {
            frequency,
            decay,
            blend,
        } => {
            let synth = KarplusStrong::new(*frequency, *decay, *blend);
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::NoiseBurst { noise_type, filter } => {
            let color = convert_noise_type(noise_type);

            // Check if filter has sweep parameter
            let has_sweep = filter
                .as_ref()
                .map(|f| match f {
                    Filter::Lowpass { cutoff_end, .. } => cutoff_end.is_some(),
                    Filter::Highpass { cutoff_end, .. } => cutoff_end.is_some(),
                    Filter::Bandpass { center_end, .. } => center_end.is_some(),
                })
                .unwrap_or(false);

            let mut samples = if has_sweep {
                // Generate raw noise without filter, then apply swept filter
                NoiseSynth::new(color).synthesize(synthesis_samples, sample_rate, &mut rng)
            } else {
                // Use static filter in the synth
                let mut synth = NoiseSynth::new(color);
                if let Some(f) = filter {
                    synth = apply_noise_filter(synth, f);
                }
                synth.synthesize(synthesis_samples, sample_rate, &mut rng)
            };

            // Apply swept filter if needed
            if has_sweep {
                if let Some(f) = filter {
                    apply_swept_filter(&mut samples, f, sample_rate);
                }
            }

            samples
        }

        Synthesis::Additive {
            base_freq,
            harmonics,
        } => {
            let synth = HarmonicSynth::new(*base_freq, harmonics.clone());
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::Oscillator {
            waveform,
            frequency,
            freq_sweep,
            duty,
            ..
        } => generate_oscillator_samples(
            waveform,
            *frequency,
            freq_sweep.as_ref(),
            *duty,
            synthesis_samples,
            sample_rate,
            &mut rng,
        ),

        Synthesis::MultiOscillator {
            frequency,
            oscillators,
            freq_sweep,
        } => generate_multi_oscillator(
            *frequency,
            oscillators,
            freq_sweep.as_ref(),
            synthesis_samples,
            sample_rate,
            &mut rng,
        ),

        Synthesis::PitchedBody {
            start_freq,
            end_freq,
        } => {
            let synth = PitchedBody::new(*start_freq, *end_freq);
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::Metallic {
            base_freq,
            num_partials,
            inharmonicity,
        } => {
            let synth = MetallicSynth::new(*base_freq, *num_partials, *inharmonicity);
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::Granular {
            source,
            grain_size_ms,
            grain_density,
            pitch_spread,
            position_spread,
            pan_spread,
        } => {
            let synth = GranularSynth::new(
                source.clone(),
                *grain_size_ms,
                *grain_density,
                *pitch_spread,
                *position_spread,
                *pan_spread,
            );
            let raw_samples = synth.synthesize(synthesis_samples, sample_rate, &mut rng);

            // If pan_spread > 0, granular synthesis returns interleaved stereo [L, R, L, R, ...]
            // We need to de-interleave and mix to mono for now, or handle stereo properly
            if *pan_spread > 0.0 {
                // De-interleave and mix to mono
                // TODO: In the future, could support per-layer stereo
                let mut mono_samples = Vec::with_capacity(synthesis_samples);
                for i in 0..synthesis_samples {
                    let left = raw_samples[i * 2];
                    let right = raw_samples[i * 2 + 1];
                    mono_samples.push((left + right) * 0.5);
                }
                mono_samples
            } else {
                raw_samples
            }
        }

        Synthesis::Wavetable {
            table,
            frequency,
            position,
            position_sweep,
            voices,
            detune,
        } => {
            let sweep = position_sweep.as_ref().map(|ps| WavetablePositionSweep {
                start_position: *position,
                end_position: ps.end_position,
                curve: convert_sweep_curve(&ps.curve),
            });

            let synth = WavetableSynth::new(*table, *frequency, *position, sweep, *voices, *detune);
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::PdSynth {
            frequency,
            distortion,
            distortion_decay,
            waveform,
            freq_sweep,
        } => {
            let pd_waveform = convert_pd_waveform(waveform);
            let mut synth =
                PhaseDistortionSynth::new(*frequency, *distortion, pd_waveform)
                    .with_distortion_decay(*distortion_decay);

            if let Some(sweep) = freq_sweep {
                let curve = convert_sweep_curve(&sweep.curve);
                synth = synth.with_sweep(FrequencySweep::new(*frequency, sweep.end_freq, curve));
            }

            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::Modal {
            frequency,
            modes,
            excitation,
            freq_sweep,
        } => {
            let modal_modes: Vec<Mode> = modes
                .iter()
                .map(|m| Mode::new(m.freq_ratio, m.amplitude, m.decay_time))
                .collect();
            let modal_excitation = convert_modal_excitation(excitation);
            let mut synth = ModalSynth::new(*frequency, modal_modes, modal_excitation);

            if let Some(sweep) = freq_sweep {
                let curve = convert_sweep_curve(&sweep.curve);
                synth = synth.with_sweep(FrequencySweep::new(*frequency, sweep.end_freq, curve));
            }

            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::Vocoder {
            carrier_freq,
            carrier_type,
            num_bands,
            band_spacing,
            envelope_attack,
            envelope_release,
            formant_rate,
            bands,
        } => {
            let carrier_type_impl = convert_vocoder_carrier_type(carrier_type);
            let band_spacing_impl = convert_vocoder_band_spacing(band_spacing);

            let mut synth = VocoderSynth::new(
                *carrier_freq,
                carrier_type_impl,
                *num_bands,
                band_spacing_impl,
                *envelope_attack,
                *envelope_release,
            )
            .with_formant_rate(*formant_rate);

            if !bands.is_empty() {
                let bands_impl: Vec<VocoderBandImpl> = bands
                    .iter()
                    .map(|b| {
                        VocoderBandImpl::new(
                            b.center_freq,
                            b.bandwidth,
                            b.envelope_pattern.clone(),
                        )
                    })
                    .collect();
                synth = synth.with_bands(bands_impl);
            }

            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::Formant {
            frequency,
            formants,
            vowel,
            vowel_morph,
            morph_amount,
            breathiness,
        } => {
            let synth = if !formants.is_empty() {
                // Use custom formants
                let formants_impl: Vec<FormantImpl> = formants
                    .iter()
                    .map(|f| FormantImpl::new(f.frequency, f.amplitude, f.bandwidth))
                    .collect();
                FormantSynth::new(*frequency, formants_impl).with_breathiness(*breathiness)
            } else if let Some(vowel_preset) = vowel {
                // Use vowel preset
                let vowel_impl = convert_formant_vowel(vowel_preset);
                let mut synth =
                    FormantSynth::with_vowel(*frequency, vowel_impl).with_breathiness(*breathiness);

                // Apply vowel morph if specified
                if let Some(morph_vowel) = vowel_morph {
                    let morph_impl = convert_formant_vowel(morph_vowel);
                    synth = synth.with_vowel_morph(morph_impl, *morph_amount);
                }

                synth
            } else {
                // Default to vowel A
                FormantSynth::vowel_a(*frequency).with_breathiness(*breathiness)
            };

            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::Vector {
            frequency,
            sources,
            position_x,
            position_y,
            path,
            path_loop,
            path_curve,
        } => {
            // Convert sources to internal representation
            let sources_impl: [VectorSourceImpl; 4] = [
                convert_vector_source(&sources[0]),
                convert_vector_source(&sources[1]),
                convert_vector_source(&sources[2]),
                convert_vector_source(&sources[3]),
            ];

            let mut synth = VectorSynth::new(*frequency, sources_impl)
                .with_position(VectorPosition::new(*position_x, *position_y));

            // Add path if specified
            if !path.is_empty() {
                let path_points: Vec<VectorPathPointImpl> = path
                    .iter()
                    .map(|p| {
                        VectorPathPointImpl::new(VectorPosition::new(p.x, p.y), p.duration)
                    })
                    .collect();
                let path_impl = VectorPath::new(path_points)
                    .with_curve(convert_sweep_curve(path_curve));
                synth = synth.with_path(path_impl, *path_loop);
            }

            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }
    };

    // Apply LFO modulation if specified
    if let Some(ref lfo_mod) = layer.lfo {
        use crate::modulation::lfo::{apply_volume_modulation, Lfo};

        let initial_phase = lfo_mod.config.phase.unwrap_or(0.0);
        let mut lfo = Lfo::new(
            lfo_mod.config.waveform,
            lfo_mod.config.rate,
            lfo_mod.config.depth,
            sample_rate,
            initial_phase,
        );

        match &lfo_mod.target {
            ModulationTarget::Pitch { semitones } => {
                // Regenerate with pitch modulation (only works for oscillator-based synthesis)
                if matches!(
                    layer.synthesis,
                    Synthesis::Oscillator { .. } | Synthesis::MultiOscillator { .. }
                ) {
                    samples = apply_lfo_pitch_modulation(
                        layer,
                        layer_idx,
                        synthesis_samples,
                        sample_rate,
                        seed,
                        &mut lfo,
                        *semitones,
                        &mut rng,
                    )?;
                }
            }
            ModulationTarget::Volume => {
                // Apply volume modulation per-sample
                for sample in samples.iter_mut() {
                    let lfo_value = lfo.next_sample(&mut rng);
                    *sample = apply_volume_modulation(*sample, lfo_value, lfo_mod.config.depth);
                }
            }
            ModulationTarget::FilterCutoff { amount } => {
                // Apply filter with LFO-modulated cutoff
                // This requires the layer to have a filter
                if let Some(ref filter) = layer.filter {
                    apply_lfo_filter_modulation(
                        &mut samples,
                        filter,
                        &mut lfo,
                        *amount,
                        sample_rate,
                        &mut rng,
                    );
                }
            }
            ModulationTarget::Pan => {
                // Pan modulation would need stereo processing, skip for now
                // as layers are mono at this stage
            }
        }
    }

    // Apply layer filter if specified (and not already applied by LFO)
    if let Some(ref filter) = layer.filter {
        // Only apply if there's no filter cutoff LFO modulation
        let has_filter_lfo = layer
            .lfo
            .as_ref()
            .map(|lfo_mod| matches!(lfo_mod.target, ModulationTarget::FilterCutoff { .. }))
            .unwrap_or(false);

        if !has_filter_lfo {
            apply_swept_filter(&mut samples, filter, sample_rate);
        }
    }

    // Apply envelope
    let envelope = generate_envelope(&layer.envelope, sample_rate, synthesis_samples);
    for (sample, env) in samples.iter_mut().zip(envelope.iter()) {
        *sample *= env;
    }

    // Pad with delay at the start if needed
    if delay_samples > 0 {
        let mut padded = vec![0.0; delay_samples];
        padded.extend(samples);
        samples = padded;
    }

    Ok(samples)
}

/// Generates oscillator samples based on waveform type.
fn generate_oscillator_samples(
    waveform: &Waveform,
    frequency: f64,
    freq_sweep: Option<&FreqSweep>,
    duty: Option<f64>,
    num_samples: usize,
    sample_rate: f64,
    rng: &mut rand_pcg::Pcg32,
) -> Vec<f64> {
    let sweep = freq_sweep.map(|s| {
        let curve = convert_sweep_curve(&s.curve);
        FrequencySweep::new(frequency, s.end_freq, curve)
    });

    match waveform {
        Waveform::Sine => {
            if let Some(s) = sweep {
                SineSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(
                    num_samples,
                    sample_rate,
                    rng,
                )
            } else {
                SineSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
        Waveform::Square | Waveform::Pulse => {
            let duty_cycle = duty.unwrap_or(0.5);
            let mut synth = if let Some(s) = sweep {
                SquareSynth::with_sweep(frequency, s.end_freq, s.curve)
            } else {
                SquareSynth::pulse(frequency, duty_cycle)
            };
            // Set duty cycle even for sweep case
            synth.duty = duty_cycle;
            synth.synthesize(num_samples, sample_rate, rng)
        }
        Waveform::Sawtooth => {
            if let Some(s) = sweep {
                SawSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(
                    num_samples,
                    sample_rate,
                    rng,
                )
            } else {
                SawSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
        Waveform::Triangle => {
            if let Some(s) = sweep {
                TriangleSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(
                    num_samples,
                    sample_rate,
                    rng,
                )
            } else {
                TriangleSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
    }
}

/// Generates multi-oscillator stack samples.
fn generate_multi_oscillator(
    base_frequency: f64,
    oscillators: &[OscillatorConfig],
    freq_sweep: Option<&FreqSweep>,
    num_samples: usize,
    sample_rate: f64,
    _rng: &mut rand_pcg::Pcg32,
) -> Vec<f64> {
    use crate::oscillator::{PhaseAccumulator, TWO_PI};

    let mut output = vec![0.0; num_samples];

    // Sweep applied to all oscillators
    let sweep_curve = freq_sweep.map(|s| {
        let curve = convert_sweep_curve(&s.curve);
        FrequencySweep::new(base_frequency, s.end_freq, curve)
    });

    for osc_config in oscillators {
        // Calculate oscillator frequency with detune
        let detune_mult = if let Some(detune_cents) = osc_config.detune {
            2.0_f64.powf(detune_cents / 1200.0)
        } else {
            1.0
        };

        let duty = osc_config.duty.unwrap_or(0.5);
        let phase_offset = osc_config.phase.unwrap_or(0.0);
        let volume = osc_config.volume;

        // Generate oscillator samples
        let mut phase_acc = PhaseAccumulator::new(sample_rate);

        for (i, out_sample) in output.iter_mut().enumerate() {
            let base_freq = if let Some(ref sweep) = sweep_curve {
                sweep.at(i as f64 / num_samples as f64)
            } else {
                base_frequency
            };

            let freq = base_freq * detune_mult;
            let mut phase = phase_acc.advance(freq);
            phase += phase_offset;

            // Wrap phase
            while phase >= TWO_PI {
                phase -= TWO_PI;
            }

            let sample = match osc_config.waveform {
                Waveform::Sine => crate::oscillator::sine(phase),
                Waveform::Square | Waveform::Pulse => crate::oscillator::square(phase, duty),
                Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                Waveform::Triangle => crate::oscillator::triangle(phase),
            };

            *out_sample += sample * volume;
        }
    }

    // Normalize by oscillator count to prevent clipping
    let count = oscillators.len().max(1) as f64;
    for sample in &mut output {
        *sample /= count;
    }

    output
}

/// Converts spec sweep curve to internal representation.
fn convert_sweep_curve(curve: &SweepCurve) -> crate::synthesis::SweepCurve {
    match curve {
        SweepCurve::Linear => crate::synthesis::SweepCurve::Linear,
        SweepCurve::Exponential => crate::synthesis::SweepCurve::Exponential,
        SweepCurve::Logarithmic => crate::synthesis::SweepCurve::Logarithmic,
    }
}

/// Converts spec noise type to internal representation.
fn convert_noise_type(noise_type: &NoiseType) -> NoiseColor {
    match noise_type {
        NoiseType::White => NoiseColor::White,
        NoiseType::Pink => NoiseColor::Pink,
        NoiseType::Brown => NoiseColor::Brown,
    }
}

/// Converts spec PD waveform to internal representation.
fn convert_pd_waveform(waveform: &PdWaveform) -> PdWaveformImpl {
    match waveform {
        PdWaveform::Resonant => PdWaveformImpl::Resonant,
        PdWaveform::Sawtooth => PdWaveformImpl::Sawtooth,
        PdWaveform::Pulse => PdWaveformImpl::Pulse,
    }
}

/// Converts spec modal excitation to internal representation.
fn convert_modal_excitation(excitation: &ModalExcitation) -> ModalExcitationImpl {
    match excitation {
        ModalExcitation::Impulse => ModalExcitationImpl::Impulse,
        ModalExcitation::Noise => ModalExcitationImpl::Noise,
        ModalExcitation::Pluck => ModalExcitationImpl::Pluck,
    }
}

/// Converts spec vocoder carrier type to internal representation.
fn convert_vocoder_carrier_type(carrier_type: &VocoderCarrierType) -> VocoderCarrierTypeImpl {
    match carrier_type {
        VocoderCarrierType::Sawtooth => VocoderCarrierTypeImpl::Sawtooth,
        VocoderCarrierType::Pulse => VocoderCarrierTypeImpl::Pulse,
        VocoderCarrierType::Noise => VocoderCarrierTypeImpl::Noise,
    }
}

/// Converts spec vocoder band spacing to internal representation.
fn convert_vocoder_band_spacing(band_spacing: &VocoderBandSpacing) -> VocoderBandSpacingImpl {
    match band_spacing {
        VocoderBandSpacing::Linear => VocoderBandSpacingImpl::Linear,
        VocoderBandSpacing::Logarithmic => VocoderBandSpacingImpl::Logarithmic,
    }
}

/// Converts spec formant vowel to internal representation.
fn convert_formant_vowel(vowel: &FormantVowel) -> FormantVowelPresetImpl {
    match vowel {
        FormantVowel::A => FormantVowelPresetImpl::A,
        FormantVowel::I => FormantVowelPresetImpl::I,
        FormantVowel::U => FormantVowelPresetImpl::U,
        FormantVowel::E => FormantVowelPresetImpl::E,
        FormantVowel::O => FormantVowelPresetImpl::O,
    }
}

/// Converts spec vector source type to internal representation.
fn convert_vector_source_type(source_type: &VectorSourceType) -> VectorSourceTypeImpl {
    match source_type {
        VectorSourceType::Sine => VectorSourceTypeImpl::Sine,
        VectorSourceType::Saw => VectorSourceTypeImpl::Saw,
        VectorSourceType::Square => VectorSourceTypeImpl::Square,
        VectorSourceType::Triangle => VectorSourceTypeImpl::Triangle,
        VectorSourceType::Noise => VectorSourceTypeImpl::Noise,
        VectorSourceType::Wavetable => VectorSourceTypeImpl::Wavetable,
    }
}

/// Converts spec vector source to internal representation.
fn convert_vector_source(source: &speccade_spec::recipe::audio::VectorSource) -> VectorSourceImpl {
    VectorSourceImpl::new(
        convert_vector_source_type(&source.source_type),
        source.frequency_ratio,
    )
}

/// Applies filter configuration to noise synthesizer.
fn apply_noise_filter(mut synth: NoiseSynth, filter: &Filter) -> NoiseSynth {
    match filter {
        Filter::Lowpass {
            cutoff, resonance, ..
        } => {
            synth = synth.with_lowpass(*cutoff, *resonance);
        }
        Filter::Highpass {
            cutoff, resonance, ..
        } => {
            synth = synth.with_highpass(*cutoff, *resonance);
        }
        Filter::Bandpass {
            center, resonance, ..
        } => {
            synth = synth.with_bandpass(*center, *resonance);
        }
    }
    synth
}

/// Applies a swept filter to a buffer of samples.
fn apply_swept_filter(samples: &mut [f64], filter: &Filter, sample_rate: f64) {
    use crate::filter::{generate_cutoff_sweep, BiquadCoeffs, BiquadFilter, SweepMode};

    let num_samples = samples.len();

    match filter {
        Filter::Lowpass {
            cutoff,
            resonance,
            cutoff_end,
        } => {
            if let Some(end_cutoff) = cutoff_end {
                // Generate cutoff sweep
                let cutoffs = generate_cutoff_sweep(
                    *cutoff,
                    *end_cutoff,
                    num_samples,
                    SweepMode::Exponential,
                );

                // Apply time-varying filter
                let mut filter_state = BiquadFilter::lowpass(*cutoff, *resonance, sample_rate);
                for (i, sample) in samples.iter_mut().enumerate() {
                    // Update filter coefficients for this sample
                    let coeffs = BiquadCoeffs::lowpass(cutoffs[i], *resonance, sample_rate);
                    filter_state.set_coeffs(coeffs);
                    *sample = filter_state.process(*sample);
                }
            } else {
                // Static filter
                let mut filter = BiquadFilter::lowpass(*cutoff, *resonance, sample_rate);
                filter.process_buffer(samples);
            }
        }
        Filter::Highpass {
            cutoff,
            resonance,
            cutoff_end,
        } => {
            if let Some(end_cutoff) = cutoff_end {
                // Generate cutoff sweep
                let cutoffs = generate_cutoff_sweep(
                    *cutoff,
                    *end_cutoff,
                    num_samples,
                    SweepMode::Exponential,
                );

                // Apply time-varying filter
                let mut filter_state = BiquadFilter::highpass(*cutoff, *resonance, sample_rate);
                for (i, sample) in samples.iter_mut().enumerate() {
                    // Update filter coefficients for this sample
                    let coeffs = BiquadCoeffs::highpass(cutoffs[i], *resonance, sample_rate);
                    filter_state.set_coeffs(coeffs);
                    *sample = filter_state.process(*sample);
                }
            } else {
                // Static filter
                let mut filter = BiquadFilter::highpass(*cutoff, *resonance, sample_rate);
                filter.process_buffer(samples);
            }
        }
        Filter::Bandpass {
            center,
            resonance,
            center_end,
        } => {
            if let Some(end_center) = center_end {
                // Generate center frequency sweep
                let centers = generate_cutoff_sweep(
                    *center,
                    *end_center,
                    num_samples,
                    SweepMode::Exponential,
                );

                // Apply time-varying filter
                let q = *resonance;
                let mut filter_state = BiquadFilter::bandpass(*center, q, sample_rate);
                for (i, sample) in samples.iter_mut().enumerate() {
                    let coeffs = BiquadCoeffs::bandpass(centers[i], q, sample_rate);
                    filter_state.set_coeffs(coeffs);
                    *sample = filter_state.process(*sample);
                }
            } else {
                // Static filter
                let q = *resonance;
                let mut filter = BiquadFilter::bandpass(*center, q, sample_rate);
                filter.process_buffer(samples);
            }
        }
    }
}

/// Generates an ADSR envelope for the given duration.
fn generate_envelope(env: &Envelope, sample_rate: f64, num_samples: usize) -> Vec<f64> {
    let params = AdsrParams::new(env.attack, env.decay, env.sustain, env.release);
    let duration = num_samples as f64 / sample_rate;
    AdsrEnvelope::generate_fixed_duration(&params, sample_rate, duration)
}

/// Calculates the loop point based on the envelope.
///
/// The loop point is set after the attack + decay phases.
fn calculate_loop_point(env: &Envelope, sample_rate: f64) -> usize {
    let loop_time = env.attack + env.decay;
    (loop_time * sample_rate) as usize
}

/// Generates a pitch envelope curve.
///
/// Returns a vector of frequency multipliers (1.0 = no change).
fn generate_pitch_envelope_curve(
    pitch_env: &PitchEnvelope,
    sample_rate: f64,
    num_samples: usize,
) -> Vec<f64> {
    let attack_samples = (pitch_env.attack * sample_rate) as usize;
    let decay_samples = (pitch_env.decay * sample_rate) as usize;
    let release_samples = (pitch_env.release * sample_rate) as usize;
    let sustain_samples =
        num_samples.saturating_sub(attack_samples + decay_samples + release_samples);

    let mut curve = Vec::with_capacity(num_samples);

    // Convert depth from semitones to frequency multiplier
    let depth_multiplier = 2.0_f64.powf(pitch_env.depth / 12.0);

    // Attack phase: 1.0 -> depth_multiplier
    for i in 0..attack_samples {
        let t = i as f64 / attack_samples.max(1) as f64;
        let multiplier = 1.0 + (depth_multiplier - 1.0) * t;
        curve.push(multiplier);
    }

    // Decay phase: depth_multiplier -> sustain_level * depth_multiplier
    for i in 0..decay_samples {
        let t = i as f64 / decay_samples.max(1) as f64;
        let start = depth_multiplier;
        let end = 1.0 + (depth_multiplier - 1.0) * pitch_env.sustain;
        let multiplier = start + (end - start) * t;
        curve.push(multiplier);
    }

    // Sustain phase: hold at sustain_level * depth_multiplier
    let sustain_multiplier = 1.0 + (depth_multiplier - 1.0) * pitch_env.sustain;
    for _ in 0..sustain_samples {
        curve.push(sustain_multiplier);
    }

    // Release phase: sustain_level * depth_multiplier -> 1.0
    for i in 0..release_samples {
        let t = i as f64 / release_samples.max(1) as f64;
        let start = sustain_multiplier;
        let multiplier = start + (1.0 - start) * t;
        curve.push(multiplier);
    }

    // Ensure exact length
    curve.resize(num_samples, 1.0);
    curve
}

/// Applies pitch envelope modulation to a layer's samples.
///
/// This regenerates the layer with pitch modulation applied.
fn apply_pitch_envelope_to_layer_samples(
    layer: &AudioLayer,
    layer_idx: usize,
    pitch_curve: &[f64],
    num_samples: usize,
    sample_rate: f64,
    seed: u32,
) -> AudioResult<Vec<f64>> {
    use crate::oscillator::{PhaseAccumulator, TWO_PI};

    let mut output = vec![0.0; num_samples];

    // Only oscillator-based synthesis can be pitch-modulated per-sample
    match &layer.synthesis {
        Synthesis::Oscillator {
            waveform,
            frequency,
            detune,
            duty,
            ..
        } => {
            let base_frequency = *frequency;
            let detune_mult = if let Some(detune_cents) = detune {
                2.0_f64.powf(*detune_cents / 1200.0)
            } else {
                1.0
            };
            let duty_cycle = duty.unwrap_or(0.5);
            let mut phase_acc = PhaseAccumulator::new(sample_rate);

            for i in 0..num_samples {
                let freq = base_frequency * detune_mult * pitch_curve[i];
                let phase = phase_acc.advance(freq);
                let sample = match waveform {
                    Waveform::Sine => crate::oscillator::sine(phase),
                    Waveform::Square | Waveform::Pulse => {
                        crate::oscillator::square(phase, duty_cycle)
                    }
                    Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                    Waveform::Triangle => crate::oscillator::triangle(phase),
                };
                output[i] = sample;
            }
        }
        Synthesis::MultiOscillator {
            frequency,
            oscillators,
            ..
        } => {
            let base_frequency = *frequency;
            for osc_config in oscillators {
                let detune_mult = if let Some(detune_cents) = osc_config.detune {
                    2.0_f64.powf(detune_cents / 1200.0)
                } else {
                    1.0
                };
                let duty = osc_config.duty.unwrap_or(0.5);
                let phase_offset = osc_config.phase.unwrap_or(0.0);
                let volume = osc_config.volume;
                let mut phase_acc = PhaseAccumulator::new(sample_rate);

                for i in 0..num_samples {
                    let freq = base_frequency * detune_mult * pitch_curve[i];
                    let mut phase = phase_acc.advance(freq);
                    phase += phase_offset;
                    while phase >= TWO_PI {
                        phase -= TWO_PI;
                    }
                    let sample = match osc_config.waveform {
                        Waveform::Sine => crate::oscillator::sine(phase),
                        Waveform::Square | Waveform::Pulse => {
                            crate::oscillator::square(phase, duty)
                        }
                        Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                        Waveform::Triangle => crate::oscillator::triangle(phase),
                    };
                    output[i] += sample * volume;
                }
            }
            let count = oscillators.len().max(1) as f64;
            for sample in &mut output {
                *sample /= count;
            }
        }
        _ => {
            // For other synthesis types, regenerate without pitch modulation
            return generate_layer(layer, layer_idx, num_samples, sample_rate, seed);
        }
    }

    Ok(output)
}

/// Applies LFO pitch modulation to a layer by regenerating with modulated frequency.
fn apply_lfo_pitch_modulation(
    layer: &AudioLayer,
    layer_idx: usize,
    num_samples: usize,
    sample_rate: f64,
    seed: u32,
    lfo: &mut crate::modulation::lfo::Lfo,
    semitones: f64,
    rng: &mut rand_pcg::Pcg32,
) -> AudioResult<Vec<f64>> {
    use crate::modulation::lfo::apply_pitch_modulation;
    use crate::oscillator::{PhaseAccumulator, TWO_PI};

    let mut output = vec![0.0; num_samples];

    // Only oscillator-based synthesis can be pitch-modulated per-sample
    match &layer.synthesis {
        Synthesis::Oscillator {
            waveform,
            frequency,
            detune,
            duty,
            ..
        } => {
            let base_frequency = *frequency;
            let detune_mult = if let Some(detune_cents) = detune {
                2.0_f64.powf(*detune_cents / 1200.0)
            } else {
                1.0
            };
            let duty_cycle = duty.unwrap_or(0.5);
            let mut phase_acc = PhaseAccumulator::new(sample_rate);

            for i in 0..num_samples {
                let lfo_value = lfo.next_sample(rng);
                let freq = apply_pitch_modulation(
                    base_frequency * detune_mult,
                    lfo_value,
                    semitones,
                );
                let phase = phase_acc.advance(freq);
                let sample = match waveform {
                    Waveform::Sine => crate::oscillator::sine(phase),
                    Waveform::Square | Waveform::Pulse => {
                        crate::oscillator::square(phase, duty_cycle)
                    }
                    Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                    Waveform::Triangle => crate::oscillator::triangle(phase),
                };
                output[i] = sample;
            }
        }
        Synthesis::MultiOscillator {
            frequency,
            oscillators,
            ..
        } => {
            let base_frequency = *frequency;
            for osc_config in oscillators {
                let detune_mult = if let Some(detune_cents) = osc_config.detune {
                    2.0_f64.powf(detune_cents / 1200.0)
                } else {
                    1.0
                };
                let duty = osc_config.duty.unwrap_or(0.5);
                let phase_offset = osc_config.phase.unwrap_or(0.0);
                let volume = osc_config.volume;
                let mut phase_acc = PhaseAccumulator::new(sample_rate);

                // Reset LFO for each oscillator
                let mut lfo_clone = lfo.clone();

                for i in 0..num_samples {
                    let lfo_value = lfo_clone.next_sample(rng);
                    let freq = apply_pitch_modulation(
                        base_frequency * detune_mult,
                        lfo_value,
                        semitones,
                    );
                    let mut phase = phase_acc.advance(freq);
                    phase += phase_offset;
                    while phase >= TWO_PI {
                        phase -= TWO_PI;
                    }
                    let sample = match osc_config.waveform {
                        Waveform::Sine => crate::oscillator::sine(phase),
                        Waveform::Square | Waveform::Pulse => {
                            crate::oscillator::square(phase, duty)
                        }
                        Waveform::Sawtooth => crate::oscillator::sawtooth(phase),
                        Waveform::Triangle => crate::oscillator::triangle(phase),
                    };
                    output[i] += sample * volume;
                }
            }
            let count = oscillators.len().max(1) as f64;
            for sample in &mut output {
                *sample /= count;
            }
        }
        _ => {
            // For other synthesis types, generate without pitch modulation
            return generate_layer(layer, layer_idx, num_samples, sample_rate, seed);
        }
    }

    Ok(output)
}

/// Applies LFO-modulated filter to a sample buffer.
fn apply_lfo_filter_modulation(
    samples: &mut [f64],
    filter: &Filter,
    lfo: &mut crate::modulation::lfo::Lfo,
    amount: f64,
    sample_rate: f64,
    rng: &mut rand_pcg::Pcg32,
) {
    use crate::filter::{BiquadCoeffs, BiquadFilter};
    use crate::modulation::lfo::apply_filter_cutoff_modulation;

    match filter {
        Filter::Lowpass {
            cutoff, resonance, ..
        } => {
            let mut filter_state = BiquadFilter::lowpass(*cutoff, *resonance, sample_rate);
            for sample in samples.iter_mut() {
                let lfo_value = lfo.next_sample(rng);
                let modulated_cutoff =
                    apply_filter_cutoff_modulation(*cutoff, lfo_value, amount);
                let coeffs = BiquadCoeffs::lowpass(modulated_cutoff, *resonance, sample_rate);
                filter_state.set_coeffs(coeffs);
                *sample = filter_state.process(*sample);
            }
        }
        Filter::Highpass {
            cutoff, resonance, ..
        } => {
            let mut filter_state = BiquadFilter::highpass(*cutoff, *resonance, sample_rate);
            for sample in samples.iter_mut() {
                let lfo_value = lfo.next_sample(rng);
                let modulated_cutoff =
                    apply_filter_cutoff_modulation(*cutoff, lfo_value, amount);
                let coeffs = BiquadCoeffs::highpass(modulated_cutoff, *resonance, sample_rate);
                filter_state.set_coeffs(coeffs);
                *sample = filter_state.process(*sample);
            }
        }
        Filter::Bandpass {
            center, resonance, ..
        } => {
            let q = *resonance;
            let mut filter_state = BiquadFilter::bandpass(*center, q, sample_rate);
            for sample in samples.iter_mut() {
                let lfo_value = lfo.next_sample(rng);
                let modulated_center = apply_filter_cutoff_modulation(*center, lfo_value, amount);
                let coeffs = BiquadCoeffs::bandpass(modulated_center, q, sample_rate);
                filter_state.set_coeffs(coeffs);
                *sample = filter_state.process(*sample);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::Recipe;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Spec};

    fn create_test_spec() -> Spec {
        let params = AudioV1Params {
            duration_seconds: 0.5,
            sample_rate: 44100,
            master_filter: None,
            layers: vec![AudioLayer {
                synthesis: Synthesis::FmSynth {
                    carrier_freq: 440.0,
                    modulator_freq: 880.0,
                    modulation_index: 2.0,
                    freq_sweep: None,
                },
                envelope: Envelope {
                    attack: 0.01,
                    decay: 0.1,
                    sustain: 0.5,
                    release: 0.2,
                },
                volume: 0.8,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
            effects: vec![],
        };

        Spec::builder("test-sfx", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_v1",
                serde_json::to_value(&params).unwrap(),
            ))
            .build()
    }

    #[test]
    fn test_generate_basic() {
        let spec = create_test_spec();
        let result = generate(&spec).expect("should generate");

        assert_eq!(result.num_layers, 1);
        assert!(!result.wav.wav_data.is_empty());
        assert_eq!(result.wav.sample_rate, 44100);
    }

    #[test]
    fn test_generate_determinism() {
        let spec = create_test_spec();

        let result1 = generate(&spec).expect("should generate");
        let result2 = generate(&spec).expect("should generate");

        assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);
    }

    #[test]
    fn test_generate_different_seeds() {
        let params = AudioV1Params {
            duration_seconds: 0.1,
            sample_rate: 22050,
            master_filter: None,
            layers: vec![AudioLayer {
                synthesis: Synthesis::NoiseBurst {
                    noise_type: NoiseType::White,
                    filter: None,
                },
                envelope: Envelope::default(),
                volume: 1.0,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
            effects: vec![],
        };

        let spec1 = Spec::builder("test-sfx", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_v1",
                serde_json::to_value(&params).unwrap(),
            ))
            .build();

        let mut spec2 = spec1.clone();
        spec2.seed = 43;

        let result1 = generate(&spec1).expect("should generate");
        let result2 = generate(&spec2).expect("should generate");

        assert_ne!(result1.wav.pcm_hash, result2.wav.pcm_hash);
    }

    #[test]
    fn test_generate_stereo() {
        let params = AudioV1Params {
            duration_seconds: 0.1,
            sample_rate: 44100,
            master_filter: None,
            pitch_envelope: None,
            base_note: None,
            generate_loop_points: false,
            layers: vec![
                AudioLayer {
                    synthesis: Synthesis::Oscillator {
                        waveform: Waveform::Sine,
                        frequency: 440.0,
                        freq_sweep: None,
                        detune: None,
                        duty: None,
                    },
                    envelope: Envelope::default(),
                    volume: 0.5,
                    pan: -0.8, // Left
                    delay: None,
                    filter: None,
                    lfo: None,
                },
                AudioLayer {
                    synthesis: Synthesis::Oscillator {
                        waveform: Waveform::Sine,
                        frequency: 550.0,
                        freq_sweep: None,
                        detune: None,
                        duty: None,
                    },
                    envelope: Envelope::default(),
                    volume: 0.5,
                    pan: 0.8, // Right
                    delay: None,
                    filter: None,
                    lfo: None,
                },
            ],
            effects: vec![],
        };

        let result = generate_from_params(&params, 42).expect("should generate");
        assert!(result.wav.is_stereo);
    }
}
