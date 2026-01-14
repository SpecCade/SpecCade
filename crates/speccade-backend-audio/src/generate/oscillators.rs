//! Oscillator generation helpers.

use speccade_spec::recipe::audio::{FreqSweep, OscillatorConfig, Waveform};

use crate::synthesis::oscillators::{SawSynth, SineSynth, SquareSynth, TriangleSynth};
use crate::synthesis::{FrequencySweep, Synthesizer};

use super::converters::convert_sweep_curve;

/// Generates oscillator samples based on waveform type.
pub fn generate_oscillator_samples(
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
pub fn generate_multi_oscillator(
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
