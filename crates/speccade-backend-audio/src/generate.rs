//! Main entry point for audio SFX generation.
//!
//! This module takes a spec and generates a WAV file deterministically.

use speccade_spec::recipe::audio_sfx::{
    AudioLayer, AudioSfxLayeredSynthV1Params, Envelope, Filter, NoiseType, Synthesis, Waveform,
};
use speccade_spec::Spec;

use crate::envelope::{AdsrEnvelope, AdsrParams};
use crate::error::{AudioError, AudioResult};
use crate::mixer::{Layer, Mixer, MixerOutput};
use crate::rng::create_rng;
use crate::synthesis::fm::FmSynth;
use crate::synthesis::harmonics::HarmonicSynth;
use crate::synthesis::karplus::KarplusStrong;
use crate::synthesis::metallic::MetallicSynth;
use crate::synthesis::noise::{NoiseColor, NoiseSynth};
use crate::synthesis::oscillators::{SawSynth, SineSynth, SquareSynth, TriangleSynth};
use crate::synthesis::pitched_body::PitchedBody;
use crate::synthesis::{FrequencySweep, SweepCurve, Synthesizer};
use crate::wav::WavResult;

/// Result of audio generation.
#[derive(Debug)]
pub struct GenerateResult {
    /// WAV file data.
    pub wav: WavResult,
    /// Number of layers processed.
    pub num_layers: usize,
}

/// Generates audio from a spec.
///
/// # Arguments
/// * `spec` - The specification containing audio parameters
///
/// # Returns
/// Generated WAV file and metadata
pub fn generate(spec: &Spec) -> AudioResult<GenerateResult> {
    // Extract recipe params
    let params = extract_params(spec)?;

    // Generate audio
    generate_from_params(&params, spec.seed)
}

/// Extracts audio parameters from a spec.
fn extract_params(spec: &Spec) -> AudioResult<AudioSfxLayeredSynthV1Params> {
    let recipe = spec
        .recipe
        .as_ref()
        .ok_or(AudioError::MissingRecipe)?;

    // Use the Recipe's helper method to parse params
    recipe
        .as_audio_sfx_layered_synth()
        .map_err(|e| AudioError::InvalidRecipeType {
            expected: "audio_sfx.layered_synth_v1".to_string(),
            found: format!("{}: {}", recipe.kind, e),
        })
}

/// Generates audio from parameters directly.
///
/// # Arguments
/// * `params` - Audio synthesis parameters
/// * `seed` - RNG seed for deterministic generation
///
/// # Returns
/// Generated WAV file and metadata
pub fn generate_from_params(params: &AudioSfxLayeredSynthV1Params, seed: u32) -> AudioResult<GenerateResult> {
    let sample_rate = params.sample_rate as f64;
    let num_samples = (params.duration_seconds * sample_rate).ceil() as usize;

    let mut mixer = Mixer::new(num_samples, sample_rate);

    // Process each layer
    for (layer_idx, layer) in params.layers.iter().enumerate() {
        let layer_seed = crate::rng::derive_layer_seed(seed, layer_idx as u32);
        let layer_samples = generate_layer(layer, num_samples, sample_rate, layer_seed)?;
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
    })
}

/// Generates a single audio layer.
fn generate_layer(
    layer: &AudioLayer,
    num_samples: usize,
    sample_rate: f64,
    seed: u32,
) -> AudioResult<Vec<f64>> {
    let mut rng = create_rng(seed);

    // Calculate delay padding
    let delay_samples = layer.delay.map(|d| (d * sample_rate) as usize).unwrap_or(0);
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
            let has_sweep = filter.as_ref().map(|f| match f {
                Filter::Lowpass { cutoff_end, .. } => cutoff_end.is_some(),
                Filter::Highpass { cutoff_end, .. } => cutoff_end.is_some(),
                Filter::Bandpass { center_end, .. } => center_end.is_some(),
            }).unwrap_or(false);

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
        } => {
            generate_oscillator_samples(waveform, *frequency, freq_sweep.as_ref(), *duty, synthesis_samples, sample_rate, &mut rng)
        }

        Synthesis::MultiOscillator {
            frequency,
            oscillators,
            freq_sweep,
        } => {
            generate_multi_oscillator(*frequency, oscillators, freq_sweep.as_ref(), synthesis_samples, sample_rate, &mut rng)
        }

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
    };

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
    freq_sweep: Option<&speccade_spec::recipe::audio_sfx::FreqSweep>,
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
                SineSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(num_samples, sample_rate, rng)
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
                SawSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(num_samples, sample_rate, rng)
            } else {
                SawSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
        Waveform::Triangle => {
            if let Some(s) = sweep {
                TriangleSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(num_samples, sample_rate, rng)
            } else {
                TriangleSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
        }
    }
}

/// Generates multi-oscillator stack samples.
fn generate_multi_oscillator(
    base_frequency: f64,
    oscillators: &[speccade_spec::recipe::audio_sfx::OscillatorConfig],
    freq_sweep: Option<&speccade_spec::recipe::audio_sfx::FreqSweep>,
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

        for i in 0..num_samples {
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

            output[i] += sample * volume;
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
fn convert_sweep_curve(curve: &speccade_spec::recipe::audio_sfx::SweepCurve) -> SweepCurve {
    match curve {
        speccade_spec::recipe::audio_sfx::SweepCurve::Linear => SweepCurve::Linear,
        speccade_spec::recipe::audio_sfx::SweepCurve::Exponential => SweepCurve::Exponential,
        speccade_spec::recipe::audio_sfx::SweepCurve::Logarithmic => SweepCurve::Logarithmic,
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

/// Applies filter configuration to noise synthesizer.
fn apply_noise_filter(mut synth: NoiseSynth, filter: &Filter) -> NoiseSynth {
    match filter {
        Filter::Lowpass { cutoff, resonance, .. } => {
            synth = synth.with_lowpass(*cutoff, *resonance);
        }
        Filter::Highpass { cutoff, resonance, .. } => {
            synth = synth.with_highpass(*cutoff, *resonance);
        }
        Filter::Bandpass {
            center,
            bandwidth,
            resonance,
            ..
        } => {
            synth = synth.with_bandpass(*center, *bandwidth, *resonance);
        }
    }
    synth
}

/// Applies a swept filter to a buffer of samples.
fn apply_swept_filter(samples: &mut [f64], filter: &Filter, sample_rate: f64) {
    use crate::filter::{BiquadCoeffs, BiquadFilter, SweepMode, generate_cutoff_sweep};

    let num_samples = samples.len();

    match filter {
        Filter::Lowpass { cutoff, resonance, cutoff_end } => {
            if let Some(end_cutoff) = cutoff_end {
                // Generate cutoff sweep
                let cutoffs = generate_cutoff_sweep(*cutoff, *end_cutoff, num_samples, SweepMode::Exponential);

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
        Filter::Highpass { cutoff, resonance, cutoff_end } => {
            if let Some(end_cutoff) = cutoff_end {
                // Generate cutoff sweep
                let cutoffs = generate_cutoff_sweep(*cutoff, *end_cutoff, num_samples, SweepMode::Exponential);

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
        Filter::Bandpass { center, bandwidth, resonance, center_end } => {
            if let Some(end_center) = center_end {
                // Generate center frequency sweep
                let centers = generate_cutoff_sweep(*center, *end_center, num_samples, SweepMode::Exponential);

                // Apply time-varying filter
                // Q = center / bandwidth for bandpass
                let q = *center / bandwidth;
                let mut filter_state = BiquadFilter::bandpass(*center, q, sample_rate);
                for (i, sample) in samples.iter_mut().enumerate() {
                    // Keep Q constant, which means bandwidth scales with center
                    let q_at_i = centers[i] / bandwidth;
                    let coeffs = BiquadCoeffs::bandpass(centers[i], q_at_i, sample_rate);
                    filter_state.set_coeffs(coeffs);
                    *sample = filter_state.process(*sample);
                }
            } else {
                // Static filter
                let q = *center / bandwidth;
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

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::audio_sfx::{AudioLayer, Envelope, Synthesis};
    use speccade_spec::recipe::Recipe;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Spec};

    fn create_test_spec() -> Spec {
        let params = AudioSfxLayeredSynthV1Params {
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
            }],
        };

        Spec::builder("test-sfx", AssetType::AudioSfx)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_sfx.layered_synth_v1",
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
        let params = AudioSfxLayeredSynthV1Params {
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
            }],
        };

        let spec1 = Spec::builder("test-sfx", AssetType::AudioSfx)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_sfx.layered_synth_v1",
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
        let params = AudioSfxLayeredSynthV1Params {
            duration_seconds: 0.1,
            sample_rate: 44100,
            master_filter: None,
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
                },
            ],
        };

        let result = generate_from_params(&params, 42).expect("should generate");
        assert!(result.wav.is_stereo);
    }
}
