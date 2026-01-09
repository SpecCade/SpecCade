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
use crate::synthesis::noise::{NoiseColor, NoiseSynth};
use crate::synthesis::oscillators::{SawSynth, SineSynth, SquareSynth, TriangleSynth};
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
    let mixed = mixer.mix();

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

            synth.synthesize(num_samples, sample_rate, &mut rng)
        }

        Synthesis::KarplusStrong {
            frequency,
            decay,
            blend,
        } => {
            let synth = KarplusStrong::new(*frequency, *decay, *blend);
            synth.synthesize(num_samples, sample_rate, &mut rng)
        }

        Synthesis::NoiseBurst { noise_type, filter } => {
            let color = convert_noise_type(noise_type);
            let mut synth = NoiseSynth::new(color);

            if let Some(f) = filter {
                synth = apply_noise_filter(synth, f);
            }

            synth.synthesize(num_samples, sample_rate, &mut rng)
        }

        Synthesis::Additive {
            base_freq,
            harmonics,
        } => {
            let synth = HarmonicSynth::new(*base_freq, harmonics.clone());
            synth.synthesize(num_samples, sample_rate, &mut rng)
        }

        Synthesis::Oscillator {
            waveform,
            frequency,
            freq_sweep,
        } => {
            generate_oscillator_samples(waveform, *frequency, freq_sweep.as_ref(), num_samples, sample_rate, &mut rng)
        }
    };

    // Apply envelope
    let envelope = generate_envelope(&layer.envelope, sample_rate, num_samples);
    for (sample, env) in samples.iter_mut().zip(envelope.iter()) {
        *sample *= env;
    }

    Ok(samples)
}

/// Generates oscillator samples based on waveform type.
fn generate_oscillator_samples(
    waveform: &Waveform,
    frequency: f64,
    freq_sweep: Option<&speccade_spec::recipe::audio_sfx::FreqSweep>,
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
            if let Some(s) = sweep {
                SquareSynth::with_sweep(frequency, s.end_freq, s.curve).synthesize(num_samples, sample_rate, rng)
            } else {
                SquareSynth::new(frequency).synthesize(num_samples, sample_rate, rng)
            }
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
        Filter::Lowpass { cutoff, resonance } => {
            synth = synth.with_lowpass(*cutoff, *resonance);
        }
        Filter::Highpass { cutoff, resonance } => {
            synth = synth.with_highpass(*cutoff, *resonance);
        }
        Filter::Bandpass {
            center,
            bandwidth,
            resonance,
        } => {
            synth = synth.with_bandpass(*center, *bandwidth, *resonance);
        }
    }
    synth
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
            layers: vec![AudioLayer {
                synthesis: Synthesis::NoiseBurst {
                    noise_type: NoiseType::White,
                    filter: None,
                },
                envelope: Envelope::default(),
                volume: 1.0,
                pan: 0.0,
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
            layers: vec![
                AudioLayer {
                    synthesis: Synthesis::Oscillator {
                        waveform: Waveform::Sine,
                        frequency: 440.0,
                        freq_sweep: None,
                    },
                    envelope: Envelope::default(),
                    volume: 0.5,
                    pan: -0.8, // Left
                },
                AudioLayer {
                    synthesis: Synthesis::Oscillator {
                        waveform: Waveform::Sine,
                        frequency: 550.0,
                        freq_sweep: None,
                    },
                    envelope: Envelope::default(),
                    volume: 0.5,
                    pan: 0.8, // Right
                },
            ],
        };

        let result = generate_from_params(&params, 42).expect("should generate");
        assert!(result.wav.is_stereo);
    }
}
