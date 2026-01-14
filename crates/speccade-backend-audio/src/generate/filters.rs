//! Filter application and processing.

use speccade_spec::recipe::audio::Filter;

use crate::filter::{generate_cutoff_sweep, BiquadCoeffs, BiquadFilter, SweepMode};
use crate::synthesis::noise::NoiseSynth;

/// Applies filter configuration to noise synthesizer.
pub fn apply_noise_filter(mut synth: NoiseSynth, filter: &Filter) -> NoiseSynth {
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
pub fn apply_swept_filter(samples: &mut [f64], filter: &Filter, sample_rate: f64) {
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
