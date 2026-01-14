//! Envelope generation and modulation utilities.

use speccade_spec::recipe::audio::{AudioLayer, Envelope, Filter, PitchEnvelope, Synthesis, Waveform};

use crate::envelope::{AdsrEnvelope, AdsrParams};
use crate::error::AudioResult;
use crate::filter::{BiquadCoeffs, BiquadFilter};
use crate::modulation::lfo::{apply_filter_cutoff_modulation, apply_pitch_modulation, Lfo};
use crate::oscillator::{PhaseAccumulator, TWO_PI};

use super::layer::generate_layer;

/// Generates an ADSR envelope for the given duration.
pub fn generate_envelope(env: &Envelope, sample_rate: f64, num_samples: usize) -> Vec<f64> {
    let params = AdsrParams::new(env.attack, env.decay, env.sustain, env.release);
    let duration = num_samples as f64 / sample_rate;
    AdsrEnvelope::generate_fixed_duration(&params, sample_rate, duration)
}

/// Calculates the loop point based on the envelope.
///
/// The loop point is set after the attack + decay phases.
pub fn calculate_loop_point(env: &Envelope, sample_rate: f64) -> usize {
    let loop_time = env.attack + env.decay;
    (loop_time * sample_rate) as usize
}

/// Generates a pitch envelope curve.
///
/// Returns a vector of frequency multipliers (1.0 = no change).
pub fn generate_pitch_envelope_curve(
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
pub fn apply_pitch_envelope_to_layer_samples(
    layer: &AudioLayer,
    layer_idx: usize,
    pitch_curve: &[f64],
    num_samples: usize,
    sample_rate: f64,
    seed: u32,
) -> AudioResult<Vec<f64>> {
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
pub fn apply_lfo_pitch_modulation(
    layer: &AudioLayer,
    layer_idx: usize,
    num_samples: usize,
    sample_rate: f64,
    seed: u32,
    lfo: &mut Lfo,
    semitones: f64,
    rng: &mut rand_pcg::Pcg32,
) -> AudioResult<Vec<f64>> {
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
pub fn apply_lfo_filter_modulation(
    samples: &mut [f64],
    filter: &Filter,
    lfo: &mut Lfo,
    amount: f64,
    sample_rate: f64,
    rng: &mut rand_pcg::Pcg32,
) {
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
