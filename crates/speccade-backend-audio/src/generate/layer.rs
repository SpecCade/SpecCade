//! Layer generation and synthesis dispatching.

use speccade_spec::recipe::audio::{AudioLayer, Filter, ModulationTarget, Synthesis};

use crate::error::{AudioError, AudioResult};
use crate::rng::create_rng;
use crate::synthesis::am::AmSynth;
use crate::synthesis::fm::FmSynth;
use crate::synthesis::formant::{Formant as FormantImpl, FormantSynth};
use crate::synthesis::granular::GranularSynth;
use crate::synthesis::harmonics::HarmonicSynth;
use crate::synthesis::karplus::KarplusStrong;
use crate::synthesis::metallic::MetallicSynth;
use crate::synthesis::modal::{Mode, ModalSynth};
use crate::synthesis::noise::NoiseSynth;
use crate::synthesis::phase_distortion::PhaseDistortionSynth;
use crate::synthesis::pitched_body::PitchedBody;
use crate::synthesis::ring_mod::RingModSynth;
use crate::synthesis::vector::{
    VectorPath, VectorPathPoint as VectorPathPointImpl, VectorPosition,
    VectorSource as VectorSourceImpl, VectorSynth,
};
use crate::synthesis::vocoder::{VocoderBand as VocoderBandImpl, VocoderSynth};
use crate::synthesis::wavetable::{PositionSweep as WavetablePositionSweep, WavetableSynth};
use crate::synthesis::{FrequencySweep, Synthesizer};

use super::converters::*;
use super::filters;
use super::modulation;
use super::oscillators;

/// Generates a single audio layer.
pub fn generate_layer(
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
                    synth = filters::apply_noise_filter(synth, f);
                }
                synth.synthesize(synthesis_samples, sample_rate, &mut rng)
            };

            // Apply swept filter if needed
            if has_sweep {
                if let Some(f) = filter {
                    filters::apply_swept_filter(&mut samples, f, sample_rate);
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
        } => oscillators::generate_oscillator_samples(
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
            oscillators: oscs,
            freq_sweep,
        } => oscillators::generate_multi_oscillator(
            *frequency,
            oscs,
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
                    samples = modulation::apply_lfo_pitch_modulation(
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
                    modulation::apply_lfo_filter_modulation(
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
            filters::apply_swept_filter(&mut samples, filter, sample_rate);
        }
    }

    // Apply envelope
    let envelope = modulation::generate_envelope(&layer.envelope, sample_rate, synthesis_samples);
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
