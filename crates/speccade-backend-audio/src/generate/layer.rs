//! Layer generation and synthesis dispatching.

use speccade_spec::recipe::audio::{AudioLayer, Filter, ModulationTarget, Synthesis};

use crate::error::{AudioError, AudioResult};
use crate::rng::create_rng;

/// Output from layer generation, supporting both mono and stereo sources.
#[derive(Debug, Clone)]
pub enum LayerOutput {
    /// Mono output (single channel).
    Mono(Vec<f64>),
    /// Stereo output (separate left/right channels).
    Stereo { left: Vec<f64>, right: Vec<f64> },
}

impl LayerOutput {
    /// Convert to mono by averaging stereo channels if needed.
    pub fn to_mono(self) -> Vec<f64> {
        match self {
            LayerOutput::Mono(samples) => samples,
            LayerOutput::Stereo { left, right } => left
                .into_iter()
                .zip(right)
                .map(|(l, r)| (l + r) * 0.5)
                .collect(),
        }
    }

    /// Returns true if this is stereo output.
    pub fn is_stereo(&self) -> bool {
        matches!(self, LayerOutput::Stereo { .. })
    }

    /// Get the number of samples (per channel for stereo).
    pub fn len(&self) -> usize {
        match self {
            LayerOutput::Mono(samples) => samples.len(),
            LayerOutput::Stereo { left, .. } => left.len(),
        }
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
use crate::synthesis::am::AmSynth;
use crate::synthesis::bowed_string::BowedStringSynth;
use crate::synthesis::comb_synth::CombFilterSynth;
use crate::synthesis::feedback_fm::FeedbackFmSynth;
use crate::synthesis::fm::FmSynth;
use crate::synthesis::formant::{Formant as FormantImpl, FormantSynth};
use crate::synthesis::granular::GranularSynth;
use crate::synthesis::harmonics::HarmonicSynth;
use crate::synthesis::karplus::KarplusStrong;
use crate::synthesis::membrane::MembraneDrumSynth;
use crate::synthesis::metallic::MetallicSynth;
use crate::synthesis::modal::{ModalSynth, Mode};
use crate::synthesis::noise::NoiseSynth;
use crate::synthesis::phase_distortion::PhaseDistortionSynth;
use crate::synthesis::pitched_body::PitchedBody;
use crate::synthesis::pulsar::PulsarSynth;
use crate::synthesis::ring_mod::RingModSynth;
use crate::synthesis::spectral::SpectralFreezeSynth;
use crate::synthesis::vector::{
    VectorPath, VectorPathPoint as VectorPathPointImpl, VectorPosition,
    VectorSource as VectorSourceImpl, VectorSynth,
};
use crate::synthesis::vocoder::{VocoderBand as VocoderBandImpl, VocoderSynth};
use crate::synthesis::vosim::VosimSynth;
use crate::synthesis::waveguide::WaveguideSynth;
use crate::synthesis::wavetable::{PositionSweep as WavetablePositionSweep, WavetableSynth};
use crate::synthesis::{FrequencySweep, Synthesizer};

use super::converters::*;
use super::filters;
use super::modulation;
use super::oscillators;

/// Generates a single audio layer, returning mono or stereo output.
pub fn generate_layer(
    layer: &AudioLayer,
    layer_idx: usize,
    num_samples: usize,
    sample_rate: f64,
    seed: u32,
) -> AudioResult<LayerOutput> {
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

    // Track if we're generating stereo (only granular with pan_spread > 0 for now)
    let is_stereo_granular = matches!(
        &layer.synthesis,
        Synthesis::Granular { pan_spread, .. } if *pan_spread > 0.0
    );

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
                    Filter::Notch { center_end, .. } => center_end.is_some(),
                    Filter::Allpass { frequency_end, .. } => frequency_end.is_some(),
                    Filter::Comb { .. } => false, // Comb filter has no sweep support
                    Filter::Formant { .. } => false, // Formant filter has no sweep support
                    Filter::Ladder { cutoff_end, .. } => cutoff_end.is_some(),
                    Filter::ShelfLow { .. } => false, // Shelf filters have no sweep support
                    Filter::ShelfHigh { .. } => false, // Shelf filters have no sweep support
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
            // If pan_spread > 0, granular synthesis returns interleaved stereo [L, R, L, R, ...]
            // We keep it interleaved for now and will split at the end
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
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
            let mut synth = PhaseDistortionSynth::new(*frequency, *distortion, pd_waveform)
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
                        VocoderBandImpl::new(b.center_freq, b.bandwidth, b.envelope_pattern.clone())
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
                    .map(|p| VectorPathPointImpl::new(VectorPosition::new(p.x, p.y), p.duration))
                    .collect();
                let path_impl =
                    VectorPath::new(path_points).with_curve(convert_sweep_curve(path_curve));
                synth = synth.with_path(path_impl, *path_loop);
            }

            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::SupersawUnison { .. } => {
            // SupersawUnison is handled by virtual layer expansion in generate_from_unified_params.
            // This branch should never be reached.
            return Err(AudioError::invalid_param(
                format!("layers[{}].synthesis", layer_idx),
                "SupersawUnison should be expanded to virtual layers, not processed directly",
            ));
        }

        Synthesis::Waveguide {
            frequency,
            breath,
            noise,
            damping,
            resonance,
        } => {
            let synth = WaveguideSynth::new(*frequency, *breath, *noise, *damping, *resonance);
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::BowedString {
            frequency,
            bow_pressure,
            bow_position,
            damping,
        } => {
            let synth = BowedStringSynth::new(*frequency, *bow_pressure, *bow_position, *damping);
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::MembraneDrum {
            frequency,
            decay,
            tone,
            strike,
        } => {
            let synth = MembraneDrumSynth::new(*frequency, *decay, *tone, *strike);
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::FeedbackFm {
            frequency,
            feedback,
            modulation_index,
            freq_sweep,
        } => {
            let mut synth = FeedbackFmSynth::new(*frequency, *feedback, *modulation_index);

            if let Some(sweep) = freq_sweep {
                let curve = convert_sweep_curve(&sweep.curve);
                synth = synth.with_sweep(FrequencySweep::new(*frequency, sweep.end_freq, curve));
            }

            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::CombFilterSynth {
            frequency,
            decay,
            excitation,
        } => {
            let synth = CombFilterSynth::new(*frequency, *decay, *excitation);
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::Pulsar {
            frequency,
            pulse_rate,
            grain_size_ms,
            shape,
        } => {
            let synth = PulsarSynth::new(*frequency, *pulse_rate, *grain_size_ms, *shape);
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::Vosim {
            frequency,
            formant_freq,
            pulses,
            breathiness,
        } => {
            let synth = VosimSynth::new(*frequency, *formant_freq, *pulses, *breathiness);
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::SpectralFreeze { source } => {
            let synth = SpectralFreezeSynth::new(source.clone());
            synth.synthesize(synthesis_samples, sample_rate, &mut rng)
        }
    };

    // Handle stereo granular separately - it has interleaved samples [L, R, L, R, ...]
    if is_stereo_granular {
        // De-interleave stereo samples
        assert_eq!(
            samples.len(),
            synthesis_samples * 2,
            "Granular stereo synthesis must return exactly {} samples, got {}",
            synthesis_samples * 2,
            samples.len()
        );
        let mut left = Vec::with_capacity(synthesis_samples);
        let mut right = Vec::with_capacity(synthesis_samples);
        for i in 0..synthesis_samples {
            left.push(samples[i * 2]);
            right.push(samples[i * 2 + 1]);
        }

        // Apply envelope to both channels
        let envelope =
            modulation::generate_envelope(&layer.envelope, sample_rate, synthesis_samples);
        for (i, env) in envelope.iter().enumerate() {
            left[i] *= env;
            right[i] *= env;
        }

        // Pad with delay at the start if needed
        if delay_samples > 0 {
            let mut padded_left = vec![0.0; delay_samples];
            let mut padded_right = vec![0.0; delay_samples];
            padded_left.extend(left);
            padded_right.extend(right);
            left = padded_left;
            right = padded_right;
        }

        return Ok(LayerOutput::Stereo { left, right });
    }

    // Standard mono processing path
    // Apply LFO modulation if specified
    if let Some(ref lfo_mod) = layer.lfo {
        use crate::modulation::lfo::{apply_volume_modulation, Lfo};

        let initial_phase = lfo_mod.config.phase.unwrap_or(0.0);
        let mut lfo = Lfo::new(
            lfo_mod.config.waveform,
            lfo_mod.config.rate,
            sample_rate,
            initial_phase,
        );

        match &lfo_mod.target {
            ModulationTarget::Pitch { semitones } => {
                if matches!(
                    layer.synthesis,
                    Synthesis::Oscillator { .. } | Synthesis::MultiOscillator { .. }
                ) {
                    // Regenerate with per-sample frequency modulation for oscillator-based synthesis.
                    samples = modulation::apply_lfo_pitch_modulation(modulation::LfoPitchParams {
                        layer,
                        layer_idx,
                        num_samples: synthesis_samples,
                        sample_rate,
                        seed,
                        lfo: &mut lfo,
                        semitones: *semitones,
                        depth: lfo_mod.config.depth,
                        rng: &mut rng,
                    })?;
                } else {
                    // Fallback: apply pitch modulation via deterministic time-warp (variable-rate resampling).
                    samples = modulation::apply_lfo_pitch_warp(
                        &samples,
                        &mut lfo,
                        *semitones,
                        lfo_mod.config.depth,
                        &mut rng,
                    );
                }
            }
            ModulationTarget::Volume { amount } => {
                // Apply volume modulation per-sample
                for sample in samples.iter_mut() {
                    let lfo_value = lfo.next_sample(&mut rng);
                    *sample =
                        apply_volume_modulation(*sample, lfo_value, *amount, lfo_mod.config.depth);
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
                        lfo_mod.config.depth,
                        sample_rate,
                        &mut rng,
                    );
                }
            }
            ModulationTarget::Pan { .. } => {
                // Pan modulation is applied during mixing (not during layer synthesis).
            }
            ModulationTarget::PulseWidth { amount } => {
                // Pulse width modulation requires per-sample duty cycle changes.
                // Only valid for oscillator-based synthesis with square/pulse waveforms.
                samples =
                    modulation::apply_lfo_pulse_width_modulation(modulation::LfoPulseWidthParams {
                        layer,
                        num_samples: synthesis_samples,
                        sample_rate,
                        lfo: &mut lfo,
                        amount: *amount,
                        depth: lfo_mod.config.depth,
                        rng: &mut rng,
                    });
            }
            ModulationTarget::FmIndex { amount } => {
                // FM index modulation requires per-sample modulation index changes.
                // Only valid for FmSynth synthesis.
                samples = modulation::apply_lfo_fm_index_modulation(modulation::LfoFmIndexParams {
                    layer,
                    num_samples: synthesis_samples,
                    sample_rate,
                    lfo: &mut lfo,
                    amount: *amount,
                    depth: lfo_mod.config.depth,
                    rng: &mut rng,
                });
            }
            ModulationTarget::GrainSize { amount_ms } => {
                // Grain size modulation for granular synthesis.
                // Modulates grain size per-grain by sampling LFO at grain start.
                samples =
                    modulation::apply_lfo_grain_size_modulation(modulation::LfoGrainSizeParams {
                        layer,
                        num_samples: synthesis_samples,
                        sample_rate,
                        lfo: &mut lfo,
                        amount_ms: *amount_ms,
                        depth: lfo_mod.config.depth,
                        rng: &mut rng,
                    });
            }
            ModulationTarget::GrainDensity { amount } => {
                // Grain density modulation for granular synthesis.
                // Modulates grain density per-grain by sampling LFO at grain start.
                samples = modulation::apply_lfo_grain_density_modulation(
                    modulation::LfoGrainDensityParams {
                        layer,
                        num_samples: synthesis_samples,
                        sample_rate,
                        lfo: &mut lfo,
                        amount: *amount,
                        depth: lfo_mod.config.depth,
                        rng: &mut rng,
                    },
                );
            }
            ModulationTarget::DelayTime { .. }
            | ModulationTarget::ReverbSize { .. }
            | ModulationTarget::DistortionDrive { .. } => {
                // Post-FX only targets - no-op at layer level.
                // Validation should reject these on layer LFOs, but we handle them gracefully.
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

    Ok(LayerOutput::Mono(samples))
}
