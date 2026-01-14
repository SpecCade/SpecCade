//! Vocoder synthesizer implementation.

use std::f64::consts::PI;

use rand::Rng;
use rand_pcg::Pcg32;

use crate::oscillator::PhaseAccumulator;
use super::types::{BandSpacing, CarrierType, VocoderBand};

/// Vocoder synthesizer.
///
/// Generates vocoder-like sounds by applying time-varying amplitude envelopes
/// (simulating a modulator signal's spectral envelope) to a harmonically rich
/// carrier signal through a filter bank.
#[derive(Debug, Clone)]
pub struct VocoderSynth {
    /// Base frequency of the carrier in Hz.
    pub carrier_freq: f64,
    /// Type of carrier waveform.
    pub carrier_type: CarrierType,
    /// Number of filter bands (8-32 typical).
    pub num_bands: usize,
    /// Band spacing mode.
    pub band_spacing: BandSpacing,
    /// Envelope attack time in seconds (how fast bands respond).
    pub envelope_attack: f64,
    /// Envelope release time in seconds (how fast bands decay).
    pub envelope_release: f64,
    /// Custom band configurations (if empty, auto-generated).
    pub bands: Vec<VocoderBand>,
    /// Formant animation speed (cycles per second for envelope patterns).
    pub formant_rate: f64,
}

impl VocoderSynth {
    /// Creates a new vocoder synthesizer.
    ///
    /// # Arguments
    /// * `carrier_freq` - Base frequency of carrier in Hz
    /// * `carrier_type` - Type of carrier waveform
    /// * `num_bands` - Number of filter bands (8-32 typical)
    /// * `band_spacing` - Linear or Logarithmic band spacing
    /// * `envelope_attack` - Envelope follower attack time in seconds
    /// * `envelope_release` - Envelope follower release time in seconds
    pub fn new(
        carrier_freq: f64,
        carrier_type: CarrierType,
        num_bands: usize,
        band_spacing: BandSpacing,
        envelope_attack: f64,
        envelope_release: f64,
    ) -> Self {
        Self {
            carrier_freq: carrier_freq.max(20.0),
            carrier_type,
            num_bands: num_bands.clamp(4, 64),
            band_spacing,
            envelope_attack: envelope_attack.max(0.001),
            envelope_release: envelope_release.max(0.001),
            bands: Vec::new(),
            formant_rate: 2.0,
        }
    }

    /// Sets the formant animation rate.
    pub fn with_formant_rate(mut self, rate: f64) -> Self {
        self.formant_rate = rate.max(0.1);
        self
    }

    /// Sets custom band configurations.
    pub fn with_bands(mut self, bands: Vec<VocoderBand>) -> Self {
        self.bands = bands;
        self
    }

    /// Creates a robot voice preset.
    ///
    /// Classic vocoder robot voice with sawtooth carrier and
    /// speech-like formant animation.
    pub fn robot_voice(carrier_freq: f64) -> Self {
        Self::new(
            carrier_freq,
            CarrierType::Sawtooth,
            16,
            BandSpacing::Logarithmic,
            0.005,
            0.020,
        )
        .with_formant_rate(3.0)
    }

    /// Creates a choir preset.
    ///
    /// Breathy, choir-like sound with noise carrier and
    /// slow formant movements.
    pub fn choir(carrier_freq: f64) -> Self {
        Self::new(
            carrier_freq,
            CarrierType::Noise,
            24,
            BandSpacing::Logarithmic,
            0.050,
            0.100,
        )
        .with_formant_rate(0.5)
    }

    /// Creates a strings-through-vocoder preset.
    ///
    /// Rich, evolving pad-like sound with pulse carrier.
    pub fn strings_through_vocoder(carrier_freq: f64) -> Self {
        Self::new(
            carrier_freq,
            CarrierType::Pulse,
            20,
            BandSpacing::Logarithmic,
            0.030,
            0.080,
        )
        .with_formant_rate(1.0)
    }

    /// Generates filter bank center frequencies.
    pub(super) fn generate_band_frequencies(&self, sample_rate: f64) -> Vec<f64> {
        let num_bands = if self.bands.is_empty() {
            self.num_bands
        } else {
            self.bands.len()
        };

        if !self.bands.is_empty() {
            return self.bands.iter().map(|b| b.center_freq).collect();
        }

        let min_freq = 80.0;
        let max_freq = (sample_rate / 2.0 - 100.0).min(16000.0);

        match self.band_spacing {
            BandSpacing::Linear => {
                let step = (max_freq - min_freq) / (num_bands as f64);
                (0..num_bands)
                    .map(|i| min_freq + (i as f64 + 0.5) * step)
                    .collect()
            }
            BandSpacing::Logarithmic => {
                let log_min = min_freq.ln();
                let log_max = max_freq.ln();
                let log_step = (log_max - log_min) / (num_bands as f64);
                (0..num_bands)
                    .map(|i| (log_min + (i as f64 + 0.5) * log_step).exp())
                    .collect()
            }
        }
    }

    /// Generates formant animation patterns for each band.
    ///
    /// Creates time-varying envelopes that simulate speech formants.
    pub(super) fn generate_formant_patterns(
        &self,
        num_samples: usize,
        sample_rate: f64,
        band_frequencies: &[f64],
        rng: &mut Pcg32,
    ) -> Vec<Vec<f64>> {
        let num_bands = band_frequencies.len();
        let mut patterns = Vec::with_capacity(num_bands);

        // Define formant frequencies for vowel sounds (approximate)
        // F1: 250-900 Hz (jaw opening), F2: 700-2500 Hz (tongue position)
        // F3: 1800-3500 Hz (lip rounding)
        let formant_centers = [
            (300.0, 900.0),   // F1 range
            (900.0, 2500.0),  // F2 range
            (2000.0, 3500.0), // F3 range
        ];

        // Generate pattern cycle duration
        let cycle_samples = (sample_rate / self.formant_rate) as usize;
        let cycle_samples = cycle_samples.max(1);

        for (band_idx, &center_freq) in band_frequencies.iter().enumerate() {
            let mut pattern = Vec::with_capacity(num_samples);

            // Determine base amplitude based on proximity to formants
            let base_amp = self.calculate_formant_amplitude(center_freq, &formant_centers);

            // Add some band-specific phase offset for variety
            let phase_offset = (band_idx as f64 / num_bands as f64) * 2.0 * PI;
            let noise_factor = 0.1 + rng.gen::<f64>() * 0.1;

            for i in 0..num_samples {
                let t = (i as f64 / cycle_samples as f64) * 2.0 * PI + phase_offset;

                // Create complex envelope pattern
                let env1 = 0.5 + 0.5 * (t * 1.0).sin(); // Slow oscillation
                let env2 = 0.5 + 0.3 * (t * 2.3).sin(); // Faster overlay
                let env3 = 0.5 + 0.2 * (t * 3.7).sin(); // Even faster for texture

                // Combine envelopes with band-specific weighting
                let combined = (env1 * 0.5 + env2 * 0.3 + env3 * 0.2) * base_amp;

                // Add slight noise for natural variation
                let noise = (rng.gen::<f64>() * 2.0 - 1.0) * noise_factor;

                pattern.push((combined + noise).clamp(0.0, 1.0));
            }

            patterns.push(pattern);
        }

        patterns
    }

    /// Calculates the base amplitude for a band based on formant frequencies.
    fn calculate_formant_amplitude(&self, center_freq: f64, formant_centers: &[(f64, f64)]) -> f64 {
        let mut amplitude = 0.1; // Base level

        for &(low, high) in formant_centers {
            let center = (low + high) / 2.0;
            let width = high - low;

            // Gaussian-like weighting around formant center
            let distance = (center_freq - center).abs();
            let normalized_dist = distance / width;
            let contribution = (-normalized_dist * normalized_dist * 2.0).exp();

            amplitude += contribution * 0.6;
        }

        amplitude.min(1.0)
    }

    /// Generates carrier signal samples.
    pub(super) fn generate_carrier(
        &self,
        num_samples: usize,
        sample_rate: f64,
        rng: &mut Pcg32,
    ) -> Vec<f64> {
        match self.carrier_type {
            CarrierType::Sawtooth => {
                let mut phase_acc = PhaseAccumulator::new(sample_rate);
                (0..num_samples)
                    .map(|_| {
                        let phase = phase_acc.advance(self.carrier_freq);
                        crate::oscillator::sawtooth(phase)
                    })
                    .collect()
            }
            CarrierType::Pulse => {
                let mut phase_acc = PhaseAccumulator::new(sample_rate);
                (0..num_samples)
                    .map(|_| {
                        let phase = phase_acc.advance(self.carrier_freq);
                        crate::oscillator::square(phase, 0.3) // Narrow pulse
                    })
                    .collect()
            }
            CarrierType::Noise => crate::oscillator::white_noise(rng, num_samples),
        }
    }
}
