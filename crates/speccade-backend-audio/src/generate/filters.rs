//! Filter application and processing.

use speccade_spec::recipe::audio::Filter;

use crate::filter::{
    generate_cutoff_sweep, BiquadCoeffs, BiquadFilter, CombFilter, FormantFilter, LadderFilter,
    SweepMode,
};
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
        Filter::Notch {
            center, resonance, ..
        } => {
            synth = synth.with_notch(*center, *resonance);
        }
        Filter::Allpass {
            frequency,
            resonance,
            ..
        } => {
            synth = synth.with_allpass(*frequency, *resonance);
        }
        Filter::Comb {
            delay_ms,
            feedback,
            wet,
        } => {
            synth = synth.with_comb(*delay_ms, *feedback, *wet);
        }
        Filter::Formant { vowel, intensity } => {
            synth = synth.with_formant(*vowel, *intensity);
        }
        Filter::Ladder {
            cutoff, resonance, ..
        } => {
            synth = synth.with_ladder(*cutoff, *resonance);
        }
        Filter::ShelfLow { frequency, gain_db } => {
            synth = synth.with_shelf_low(*frequency, *gain_db);
        }
        Filter::ShelfHigh { frequency, gain_db } => {
            synth = synth.with_shelf_high(*frequency, *gain_db);
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
        Filter::Notch {
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
                let mut filter_state = BiquadFilter::notch(*center, q, sample_rate);
                for (i, sample) in samples.iter_mut().enumerate() {
                    let coeffs = BiquadCoeffs::notch(centers[i], q, sample_rate);
                    filter_state.set_coeffs(coeffs);
                    *sample = filter_state.process(*sample);
                }
            } else {
                // Static filter
                let q = *resonance;
                let mut filter = BiquadFilter::notch(*center, q, sample_rate);
                filter.process_buffer(samples);
            }
        }
        Filter::Allpass {
            frequency,
            resonance,
            frequency_end,
        } => {
            if let Some(end_frequency) = frequency_end {
                // Generate frequency sweep
                let frequencies = generate_cutoff_sweep(
                    *frequency,
                    *end_frequency,
                    num_samples,
                    SweepMode::Exponential,
                );

                // Apply time-varying filter
                let q = *resonance;
                let mut filter_state =
                    BiquadFilter::new(BiquadCoeffs::allpass(*frequency, q, sample_rate));
                for (i, sample) in samples.iter_mut().enumerate() {
                    let coeffs = BiquadCoeffs::allpass(frequencies[i], q, sample_rate);
                    filter_state.set_coeffs(coeffs);
                    *sample = filter_state.process(*sample);
                }
            } else {
                // Static filter
                let q = *resonance;
                let mut filter =
                    BiquadFilter::new(BiquadCoeffs::allpass(*frequency, q, sample_rate));
                filter.process_buffer(samples);
            }
        }
        Filter::Comb {
            delay_ms,
            feedback,
            wet,
        } => {
            // Comb filter has no sweep support (static only)
            let mut filter = CombFilter::new(*delay_ms, *feedback, *wet, sample_rate);
            filter.process_buffer(samples);
        }
        Filter::Formant { vowel, intensity } => {
            // Formant filter has no sweep support (static only)
            let mut filter = FormantFilter::new(*vowel, *intensity, sample_rate);
            filter.process_buffer(samples);
        }
        Filter::Ladder {
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

                // Apply time-varying ladder filter
                let mut filter_state = LadderFilter::new(*cutoff, *resonance, sample_rate);
                for (i, sample) in samples.iter_mut().enumerate() {
                    filter_state.set_cutoff(cutoffs[i]);
                    *sample = filter_state.process(*sample);
                }
            } else {
                // Static filter
                let mut filter = LadderFilter::new(*cutoff, *resonance, sample_rate);
                filter.process_buffer(samples);
            }
        }
        Filter::ShelfLow { frequency, gain_db } => {
            // Shelf filters are static only (no sweep support)
            let mut filter =
                BiquadFilter::new(BiquadCoeffs::low_shelf(*frequency, *gain_db, sample_rate));
            filter.process_buffer(samples);
        }
        Filter::ShelfHigh { frequency, gain_db } => {
            // Shelf filters are static only (no sweep support)
            let mut filter =
                BiquadFilter::new(BiquadCoeffs::high_shelf(*frequency, *gain_db, sample_rate));
            filter.process_buffer(samples);
        }
    }
}
