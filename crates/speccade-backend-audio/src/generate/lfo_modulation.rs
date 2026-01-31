//! LFO modulation application for synthesis layers.

use std::f64::consts::PI;

use speccade_spec::recipe::audio::{AudioLayer, Filter, Synthesis, Waveform};

use crate::error::AudioResult;
use crate::filter::{BiquadCoeffs, BiquadFilter, CombFilter, FormantFilter, LadderFilter};
use crate::modulation::lfo::{
    apply_filter_cutoff_modulation, apply_fm_index_modulation, apply_pitch_modulation_with_depth,
    apply_pulse_width_modulation, Lfo,
};
use crate::oscillator::{PhaseAccumulator, TWO_PI};
use crate::synthesis::FrequencySweep;

use super::converters::convert_sweep_curve;
use super::layer::generate_layer;

/// Parameters for LFO pitch modulation.
pub struct LfoPitchParams<'a> {
    /// The audio layer to modulate
    pub layer: &'a AudioLayer,
    /// Layer index
    pub layer_idx: usize,
    /// Number of samples to generate
    pub num_samples: usize,
    /// Sample rate in Hz
    pub sample_rate: f64,
    /// RNG seed for determinism
    pub seed: u32,
    /// LFO instance
    pub lfo: &'a mut Lfo,
    /// Pitch modulation amount in semitones
    pub semitones: f64,
    /// Modulation depth (0.0 to 1.0)
    pub depth: f64,
    /// RNG for LFO
    pub rng: &'a mut rand_pcg::Pcg32,
}

/// Applies LFO pitch modulation to a layer by regenerating with modulated frequency.
pub fn apply_lfo_pitch_modulation(params: LfoPitchParams<'_>) -> AudioResult<Vec<f64>> {
    let LfoPitchParams {
        layer,
        layer_idx,
        num_samples,
        sample_rate,
        seed,
        lfo,
        semitones,
        depth,
        rng,
    } = params;
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

            for out_sample in output.iter_mut().take(num_samples) {
                let lfo_value = lfo.next_sample(rng);
                let freq = apply_pitch_modulation_with_depth(
                    base_frequency * detune_mult,
                    lfo_value,
                    semitones,
                    depth,
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
                *out_sample = sample;
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

                for out_sample in output.iter_mut().take(num_samples) {
                    let lfo_value = lfo_clone.next_sample(rng);
                    let freq = apply_pitch_modulation_with_depth(
                        base_frequency * detune_mult,
                        lfo_value,
                        semitones,
                        depth,
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
                    *out_sample += sample * volume;
                }
            }
            let count = oscillators.len().max(1) as f64;
            for sample in &mut output {
                *sample /= count;
            }
        }
        _ => {
            // For other synthesis types, generate without pitch modulation
            // Convert LayerOutput to mono (stereo will be downmixed)
            return Ok(generate_layer(layer, layer_idx, num_samples, sample_rate, seed)?.to_mono());
        }
    }

    Ok(output)
}

/// Applies LFO pitch modulation to an existing sample buffer by time-warping (variable-rate resampling).
///
/// This makes pitch LFO usable for synthesis modes that don't support per-sample frequency modulation
/// directly. It preserves the buffer length by normalizing the cumulative playback position so the
/// last output sample maps to the last input sample.
pub fn apply_lfo_pitch_warp(
    input: &[f64],
    lfo: &mut Lfo,
    semitones: f64,
    depth: f64,
    rng: &mut rand_pcg::Pcg32,
) -> Vec<f64> {
    let n = input.len();
    if n <= 1 {
        return input.to_vec();
    }

    // Generate per-interval pitch multipliers (N-1 intervals for N samples).
    let mut multipliers = Vec::with_capacity(n - 1);
    for _ in 0..(n - 1) {
        let lfo_value = lfo.next_sample(rng);
        // Using a base frequency of 1.0 gives a pure multiplier.
        multipliers.push(apply_pitch_modulation_with_depth(
            1.0, lfo_value, semitones, depth,
        ));
    }

    let total: f64 = multipliers.iter().sum();
    let scale = if total.is_finite() && total > 0.0 {
        (n as f64 - 1.0) / total
    } else {
        1.0
    };

    let mut output = Vec::with_capacity(n);
    output.push(input[0]);

    let mut pos = 0.0_f64;
    for m in multipliers {
        pos += m * scale;
        let pos = pos.clamp(0.0, n as f64 - 1.0);

        let i0 = pos.floor() as usize;
        let i1 = (i0 + 1).min(n - 1);
        let frac = pos - i0 as f64;

        let sample = input[i0] * (1.0 - frac) + input[i1] * frac;
        output.push(sample);
    }

    output
}

/// Parameters for LFO pulse width modulation.
pub struct LfoPulseWidthParams<'a> {
    /// The audio layer to modulate
    pub layer: &'a AudioLayer,
    /// Number of samples to generate
    pub num_samples: usize,
    /// Sample rate in Hz
    pub sample_rate: f64,
    /// LFO instance
    pub lfo: &'a mut Lfo,
    /// Pulse width modulation amount (0.0-0.49)
    pub amount: f64,
    /// Modulation depth (0.0-1.0)
    pub depth: f64,
    /// RNG for LFO
    pub rng: &'a mut rand_pcg::Pcg32,
}

/// Applies LFO pulse width modulation to a layer.
///
/// Only valid for `Synthesis::Oscillator` with `waveform: Square|Pulse` or
/// `Synthesis::MultiOscillator` with at least one oscillator using `waveform: Square|Pulse`.
/// Other oscillators in a MultiOscillator are rendered with their static duty.
pub fn apply_lfo_pulse_width_modulation(params: LfoPulseWidthParams<'_>) -> Vec<f64> {
    let LfoPulseWidthParams {
        layer,
        num_samples,
        sample_rate,
        lfo,
        amount,
        depth,
        rng,
    } = params;

    let mut output = vec![0.0; num_samples];

    match &layer.synthesis {
        Synthesis::Oscillator {
            waveform,
            frequency,
            detune,
            duty,
            ..
        } => {
            // Only apply PWM if waveform is square or pulse
            if !matches!(waveform, Waveform::Square | Waveform::Pulse) {
                // Return silence if called on incompatible waveform
                // (validation should have caught this, but be defensive)
                return output;
            }

            let base_frequency = *frequency;
            let detune_mult = if let Some(detune_cents) = detune {
                2.0_f64.powf(*detune_cents / 1200.0)
            } else {
                1.0
            };
            let base_duty = duty.unwrap_or(0.5);
            let freq = base_frequency * detune_mult;

            let mut phase_acc = PhaseAccumulator::new(sample_rate);

            for out_sample in output.iter_mut().take(num_samples) {
                let lfo_value = lfo.next_sample(rng);
                let modulated_duty =
                    apply_pulse_width_modulation(base_duty, lfo_value, amount, depth);
                let phase = phase_acc.advance(freq);
                *out_sample = crate::oscillator::square(phase, modulated_duty);
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
                let base_duty = osc_config.duty.unwrap_or(0.5);
                let phase_offset = osc_config.phase.unwrap_or(0.0);
                let volume = osc_config.volume;
                let freq = base_frequency * detune_mult;

                let mut phase_acc = PhaseAccumulator::new(sample_rate);

                // Clone LFO for each oscillator to maintain determinism across oscillators
                let mut lfo_clone = lfo.clone();

                let is_pulse = matches!(osc_config.waveform, Waveform::Square | Waveform::Pulse);

                for out_sample in output.iter_mut().take(num_samples) {
                    let lfo_value = lfo_clone.next_sample(rng);
                    let duty = if is_pulse {
                        apply_pulse_width_modulation(base_duty, lfo_value, amount, depth)
                    } else {
                        base_duty
                    };

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
                    *out_sample += sample * volume;
                }
            }

            // Normalize by oscillator count
            let count = oscillators.len().max(1) as f64;
            for sample in &mut output {
                *sample /= count;
            }
        }
        _ => {
            // Unsupported synthesis type - return silence
            // (validation should have caught this)
        }
    }

    output
}

/// Parameters for LFO FM index modulation.
pub struct LfoFmIndexParams<'a> {
    /// The audio layer to modulate
    pub layer: &'a AudioLayer,
    /// Number of samples to generate
    pub num_samples: usize,
    /// Sample rate in Hz
    pub sample_rate: f64,
    /// LFO instance
    pub lfo: &'a mut Lfo,
    /// FM index modulation amount
    pub amount: f64,
    /// Modulation depth (0.0-1.0)
    pub depth: f64,
    /// RNG for LFO
    pub rng: &'a mut rand_pcg::Pcg32,
}

/// Applies LFO FM index modulation to a layer.
///
/// Only valid for `Synthesis::FmSynth`. Re-synthesizes FM audio per-sample
/// with varying modulation index based on LFO.
pub fn apply_lfo_fm_index_modulation(params: LfoFmIndexParams<'_>) -> Vec<f64> {
    let LfoFmIndexParams {
        layer,
        num_samples,
        sample_rate,
        lfo,
        amount,
        depth,
        rng,
    } = params;

    let mut output = vec![0.0; num_samples];

    match &layer.synthesis {
        Synthesis::FmSynth {
            carrier_freq,
            modulator_freq,
            modulation_index,
            freq_sweep,
        } => {
            let base_carrier_freq = *carrier_freq;
            let base_modulator_freq = *modulator_freq;
            let base_index = *modulation_index;
            let mod_ratio = base_modulator_freq / base_carrier_freq;

            // Set up optional frequency sweep
            let sweep = freq_sweep.as_ref().map(|fs| {
                FrequencySweep::new(
                    base_carrier_freq,
                    fs.end_freq,
                    convert_sweep_curve(&fs.curve),
                )
            });

            let dt = 1.0 / sample_rate;
            let two_pi = 2.0 * PI;

            let mut carrier_phase: f64 = 0.0;
            let mut modulator_phase: f64 = 0.0;

            for (i, out_sample) in output.iter_mut().enumerate().take(num_samples) {
                let progress = i as f64 / num_samples as f64;

                // Get LFO value and apply FM index modulation
                let lfo_value = lfo.next_sample(rng);
                let modulated_index =
                    apply_fm_index_modulation(base_index, lfo_value, amount, depth);

                // Get carrier frequency (with optional sweep)
                let carrier_freq = if let Some(ref s) = sweep {
                    s.at(progress)
                } else {
                    base_carrier_freq
                };

                // Scale modulator frequency proportionally if sweeping
                let modulator_freq = carrier_freq * mod_ratio;

                // FM equation: carrier = sin(wc*t + index * sin(wm*t))
                let modulator = modulator_phase.sin();
                let carrier = (carrier_phase + modulated_index * modulator).sin();

                *out_sample = carrier;

                // Update phases
                carrier_phase += two_pi * carrier_freq * dt;
                modulator_phase += two_pi * modulator_freq * dt;

                // Wrap phases to prevent precision loss
                if carrier_phase >= two_pi {
                    carrier_phase -= two_pi;
                }
                if modulator_phase >= two_pi {
                    modulator_phase -= two_pi;
                }
            }
        }
        _ => {
            // Unsupported synthesis type - return silence
            // (validation should have caught this)
        }
    }

    output
}

/// Applies LFO-modulated filter to a sample buffer.
pub fn apply_lfo_filter_modulation(
    samples: &mut [f64],
    filter: &Filter,
    lfo: &mut Lfo,
    amount: f64,
    depth: f64,
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
                    apply_filter_cutoff_modulation(*cutoff, lfo_value, amount, depth);
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
                    apply_filter_cutoff_modulation(*cutoff, lfo_value, amount, depth);
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
                let modulated_center =
                    apply_filter_cutoff_modulation(*center, lfo_value, amount, depth);
                let coeffs = BiquadCoeffs::bandpass(modulated_center, q, sample_rate);
                filter_state.set_coeffs(coeffs);
                *sample = filter_state.process(*sample);
            }
        }
        Filter::Notch {
            center, resonance, ..
        } => {
            let q = *resonance;
            let mut filter_state = BiquadFilter::notch(*center, q, sample_rate);
            for sample in samples.iter_mut() {
                let lfo_value = lfo.next_sample(rng);
                let modulated_center =
                    apply_filter_cutoff_modulation(*center, lfo_value, amount, depth);
                let coeffs = BiquadCoeffs::notch(modulated_center, q, sample_rate);
                filter_state.set_coeffs(coeffs);
                *sample = filter_state.process(*sample);
            }
        }
        Filter::Allpass {
            frequency,
            resonance,
            ..
        } => {
            let q = *resonance;
            let mut filter_state =
                BiquadFilter::new(BiquadCoeffs::allpass(*frequency, q, sample_rate));
            for sample in samples.iter_mut() {
                let lfo_value = lfo.next_sample(rng);
                let modulated_frequency =
                    apply_filter_cutoff_modulation(*frequency, lfo_value, amount, depth);
                let coeffs = BiquadCoeffs::allpass(modulated_frequency, q, sample_rate);
                filter_state.set_coeffs(coeffs);
                *sample = filter_state.process(*sample);
            }
        }
        Filter::Comb {
            delay_ms,
            feedback,
            wet,
        } => {
            // Comb filter LFO modulation affects delay_ms.
            // We modulate delay as a ratio similar to cutoff modulation.
            // Recreating the filter each sample is expensive for comb (due to delay line),
            // so we apply static comb filtering with the base parameters.
            // LFO modulation on comb filters is not well-suited to per-sample changes.
            // Instead, we just apply a static comb filter here.
            let mut filter = CombFilter::new(*delay_ms, *feedback, *wet, sample_rate);
            filter.process_buffer(samples);
            // Advance LFO to maintain phase consistency
            for _ in 0..samples.len() {
                let _ = lfo.next_sample(rng);
            }
        }
        Filter::Formant { vowel, intensity } => {
            // Formant filter does not support LFO modulation (static only).
            // Apply the static formant filter and advance LFO to maintain phase consistency.
            let mut filter = FormantFilter::new(*vowel, *intensity, sample_rate);
            filter.process_buffer(samples);
            // Advance LFO to maintain phase consistency
            for _ in 0..samples.len() {
                let _ = lfo.next_sample(rng);
            }
        }
        Filter::Ladder {
            cutoff, resonance, ..
        } => {
            let mut filter_state = LadderFilter::new(*cutoff, *resonance, sample_rate);
            for sample in samples.iter_mut() {
                let lfo_value = lfo.next_sample(rng);
                let modulated_cutoff =
                    apply_filter_cutoff_modulation(*cutoff, lfo_value, amount, depth);
                filter_state.set_cutoff(modulated_cutoff);
                *sample = filter_state.process(*sample);
            }
        }
        Filter::ShelfLow { frequency, gain_db } => {
            // Shelf filters do not support LFO modulation (static only).
            // Apply the static shelf filter and advance LFO to maintain phase consistency.
            let mut filter =
                BiquadFilter::new(BiquadCoeffs::low_shelf(*frequency, *gain_db, sample_rate));
            filter.process_buffer(samples);
            // Advance LFO to maintain phase consistency
            for _ in 0..samples.len() {
                let _ = lfo.next_sample(rng);
            }
        }
        Filter::ShelfHigh { frequency, gain_db } => {
            // Shelf filters do not support LFO modulation (static only).
            // Apply the static shelf filter and advance LFO to maintain phase consistency.
            let mut filter =
                BiquadFilter::new(BiquadCoeffs::high_shelf(*frequency, *gain_db, sample_rate));
            filter.process_buffer(samples);
            // Advance LFO to maintain phase consistency
            for _ in 0..samples.len() {
                let _ = lfo.next_sample(rng);
            }
        }
    }
}

// Re-export granular LFO modulation from dedicated module
pub use super::lfo_granular::{
    apply_lfo_grain_density_modulation, apply_lfo_grain_size_modulation, LfoGrainDensityParams,
    LfoGrainSizeParams,
};
