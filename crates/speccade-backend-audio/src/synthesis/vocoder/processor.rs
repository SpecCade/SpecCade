//! Vocoder signal processing and synthesis implementation.

use rand_pcg::Pcg32;

use super::synth::VocoderSynth;
use crate::filter::{BiquadCoeffs, BiquadFilter};
use crate::synthesis::Synthesizer;

impl Synthesizer for VocoderSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        if num_samples == 0 {
            return Vec::new();
        }

        // Generate band center frequencies
        let band_frequencies = self.generate_band_frequencies(sample_rate);
        let num_bands = band_frequencies.len();

        if num_bands == 0 {
            return vec![0.0; num_samples];
        }

        // Generate carrier signal
        let carrier = self.generate_carrier(num_samples, sample_rate, rng);

        // Generate or use custom formant patterns
        let formant_patterns = if !self.bands.is_empty()
            && self.bands.iter().all(|b| !b.envelope_pattern.is_empty())
        {
            // Use custom patterns from bands
            self.bands
                .iter()
                .map(|b| {
                    // Interpolate pattern to match num_samples
                    interpolate_pattern(&b.envelope_pattern, num_samples)
                })
                .collect()
        } else {
            // Generate procedural formant patterns
            self.generate_formant_patterns(num_samples, sample_rate, &band_frequencies, rng)
        };

        // Calculate envelope follower coefficients
        let attack_coeff = (-1.0 / (self.envelope_attack * sample_rate)).exp();
        let release_coeff = (-1.0 / (self.envelope_release * sample_rate)).exp();

        // Process each band
        let mut output = vec![0.0; num_samples];
        let q_factor = 2.0; // Moderate Q for vocoder bands

        for (band_idx, &center_freq) in band_frequencies.iter().enumerate() {
            // Skip bands outside valid frequency range
            if center_freq >= sample_rate / 2.0 || center_freq < 20.0 {
                continue;
            }

            // Create bandpass filter for this band
            let coeffs = BiquadCoeffs::bandpass(center_freq, q_factor, sample_rate);
            let mut carrier_filter = BiquadFilter::new(coeffs);

            // Filter the carrier signal for this band
            let mut band_signal = Vec::with_capacity(num_samples);
            for &sample in &carrier {
                band_signal.push(carrier_filter.process(sample));
            }

            // Get the envelope pattern for this band
            let envelope_pattern = &formant_patterns[band_idx];

            // Apply envelope follower to smooth the pattern
            let mut envelope_follower = 0.0;
            let mut smoothed_envelope = Vec::with_capacity(num_samples);

            for (i, &target) in envelope_pattern.iter().enumerate() {
                // Envelope follower with separate attack/release
                let coeff = if target > envelope_follower {
                    attack_coeff
                } else {
                    release_coeff
                };
                envelope_follower = target + coeff * (envelope_follower - target);
                smoothed_envelope.push(envelope_follower);

                // Apply envelope to band signal
                output[i] += band_signal[i] * smoothed_envelope[i];
            }
        }

        // Normalize output to [-1.0, 1.0]
        let max_val = output
            .iter()
            .map(|s| s.abs())
            .fold(0.0_f64, |a, b| a.max(b));

        if max_val > 0.0 {
            let scale = 1.0 / max_val;
            for sample in &mut output {
                *sample *= scale;
            }
        }

        output
    }
}

/// Interpolates a pattern to a target length.
pub(super) fn interpolate_pattern(pattern: &[f64], target_len: usize) -> Vec<f64> {
    if pattern.is_empty() {
        return vec![0.5; target_len];
    }

    if pattern.len() == target_len {
        return pattern.to_vec();
    }

    let mut result = Vec::with_capacity(target_len);
    let scale = (pattern.len() - 1) as f64 / (target_len - 1).max(1) as f64;

    for i in 0..target_len {
        let pos = i as f64 * scale;
        let idx_low = pos.floor() as usize;
        let idx_high = (idx_low + 1).min(pattern.len() - 1);
        let frac = pos - idx_low as f64;

        let value = pattern[idx_low] * (1.0 - frac) + pattern[idx_high] * frac;
        result.push(value);
    }

    result
}
