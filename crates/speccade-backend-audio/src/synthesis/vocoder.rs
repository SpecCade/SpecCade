//! Vocoder synthesis implementation.
//!
//! A vocoder transfers the spectral envelope from a modulator signal to a carrier signal.
//! Since we're generating from scratch (not processing existing audio), we create:
//! - A carrier signal (sawtooth, pulse, or noise)
//! - Modulator envelope patterns (simulated speech formants or rhythmic patterns)
//!
//! The vocoder works by:
//! 1. Splitting both modulator and carrier signals into frequency bands (filter bank)
//! 2. Extracting the amplitude envelope of each modulator band
//! 3. Applying those envelopes to the corresponding carrier bands
//! 4. Summing all bands to create the output

use std::f64::consts::PI;

use rand::Rng;
use rand_pcg::Pcg32;

use crate::filter::{BiquadCoeffs, BiquadFilter};
use crate::oscillator::PhaseAccumulator;
use super::Synthesizer;

/// Band spacing mode for the vocoder filter bank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandSpacing {
    /// Linear spacing between bands (equal Hz between centers).
    Linear,
    /// Logarithmic spacing (equal ratio between bands, more perceptually uniform).
    Logarithmic,
}

/// Carrier waveform type for the vocoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarrierType {
    /// Sawtooth wave - rich in harmonics, classic vocoder sound.
    Sawtooth,
    /// Pulse wave - hollow, more synthetic sound.
    Pulse,
    /// White noise - whispery, unvoiced consonant-like sound.
    Noise,
}

/// A single vocoder band configuration.
#[derive(Debug, Clone)]
pub struct VocoderBand {
    /// Center frequency of the band in Hz.
    pub center_freq: f64,
    /// Bandwidth (Q factor) of the band filter.
    pub bandwidth: f64,
    /// Envelope pattern for this band (amplitude values over time, 0.0-1.0).
    /// If empty, a default formant animation is used.
    pub envelope_pattern: Vec<f64>,
}

impl VocoderBand {
    /// Creates a new vocoder band.
    ///
    /// # Arguments
    /// * `center_freq` - Center frequency in Hz
    /// * `bandwidth` - Q factor for the band filter
    /// * `envelope_pattern` - Optional envelope pattern (empty for default)
    pub fn new(center_freq: f64, bandwidth: f64, envelope_pattern: Vec<f64>) -> Self {
        Self {
            center_freq: center_freq.clamp(20.0, 20000.0),
            bandwidth: bandwidth.clamp(0.5, 20.0),
            envelope_pattern,
        }
    }
}

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
    fn generate_band_frequencies(&self, sample_rate: f64) -> Vec<f64> {
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
    fn generate_formant_patterns(
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
    fn generate_carrier(
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
fn interpolate_pattern(pattern: &[f64], target_len: usize) -> Vec<f64> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_vocoder_basic() {
        let synth = VocoderSynth::new(
            110.0,
            CarrierType::Sawtooth,
            16,
            BandSpacing::Logarithmic,
            0.01,
            0.05,
        );
        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s), "Sample {} out of range", s);
        }
    }

    #[test]
    fn test_vocoder_robot_voice_preset() {
        let synth = VocoderSynth::robot_voice(110.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        assert_eq!(synth.carrier_type, CarrierType::Sawtooth);
        assert_eq!(synth.num_bands, 16);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_vocoder_choir_preset() {
        let synth = VocoderSynth::choir(220.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        assert_eq!(synth.carrier_type, CarrierType::Noise);
        assert_eq!(synth.num_bands, 24);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_vocoder_strings_preset() {
        let synth = VocoderSynth::strings_through_vocoder(165.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        assert_eq!(synth.carrier_type, CarrierType::Pulse);
        assert_eq!(synth.num_bands, 20);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_vocoder_determinism() {
        let synth = VocoderSynth::robot_voice(110.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_vocoder_different_seeds() {
        let synth = VocoderSynth::robot_voice(110.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(43);

        let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

        // Should be different due to random formant patterns
        assert_ne!(samples1, samples2);
    }

    #[test]
    fn test_vocoder_linear_spacing() {
        let synth = VocoderSynth::new(
            110.0,
            CarrierType::Sawtooth,
            8,
            BandSpacing::Linear,
            0.01,
            0.05,
        );
        let mut rng = create_rng(42);
        let samples = synth.synthesize(2000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 2000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_vocoder_noise_carrier() {
        let synth = VocoderSynth::new(
            110.0,
            CarrierType::Noise,
            12,
            BandSpacing::Logarithmic,
            0.01,
            0.05,
        );
        let mut rng = create_rng(42);
        let samples = synth.synthesize(2000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 2000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_vocoder_pulse_carrier() {
        let synth = VocoderSynth::new(
            110.0,
            CarrierType::Pulse,
            12,
            BandSpacing::Logarithmic,
            0.01,
            0.05,
        );
        let mut rng = create_rng(42);
        let samples = synth.synthesize(2000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 2000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_vocoder_empty_samples() {
        let synth = VocoderSynth::robot_voice(110.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(0, 44100.0, &mut rng);

        assert!(samples.is_empty());
    }

    #[test]
    fn test_vocoder_custom_bands() {
        let bands = vec![
            VocoderBand::new(200.0, 4.0, vec![]),
            VocoderBand::new(500.0, 4.0, vec![]),
            VocoderBand::new(1000.0, 4.0, vec![]),
            VocoderBand::new(2000.0, 4.0, vec![]),
        ];

        let synth = VocoderSynth::new(
            110.0,
            CarrierType::Sawtooth,
            4,
            BandSpacing::Logarithmic,
            0.01,
            0.05,
        )
        .with_bands(bands);

        let mut rng = create_rng(42);
        let samples = synth.synthesize(2000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 2000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_vocoder_band_clamping() {
        let band = VocoderBand::new(10.0, 0.1, vec![]); // Below min
        assert!(band.center_freq >= 20.0);
        assert!(band.bandwidth >= 0.5);

        let band2 = VocoderBand::new(30000.0, 50.0, vec![]); // Above max
        assert!(band2.center_freq <= 20000.0);
        assert!(band2.bandwidth <= 20.0);
    }

    #[test]
    fn test_interpolate_pattern() {
        let pattern = vec![0.0, 1.0];
        let result = interpolate_pattern(&pattern, 5);

        assert_eq!(result.len(), 5);
        assert!((result[0] - 0.0).abs() < 0.01);
        assert!((result[2] - 0.5).abs() < 0.01);
        assert!((result[4] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_empty_pattern() {
        let pattern: Vec<f64> = vec![];
        let result = interpolate_pattern(&pattern, 10);

        assert_eq!(result.len(), 10);
        for &v in &result {
            assert!((v - 0.5).abs() < 0.01);
        }
    }
}
