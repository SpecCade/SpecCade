//! Waveguide synthesis for wind/brass physical modeling.
//!
//! Implements a simple digital waveguide model using a delay line with
//! filtered noise excitation. This approach simulates the acoustics of
//! wind instruments where:
//! - The delay line represents the resonant air column
//! - Noise excitation simulates breath turbulence
//! - Damping models high-frequency absorption at the bore walls
//! - Resonance controls feedback (embouchure reflection)

use rand::Rng;
use rand_pcg::Pcg32;

use super::Synthesizer;

/// Waveguide synthesis parameters.
#[derive(Debug, Clone)]
pub struct WaveguideSynth {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Breath/excitation strength (0.0-1.0).
    pub breath: f64,
    /// Noise mix in excitation (0.0-1.0).
    pub noise: f64,
    /// Delay line damping (0.0-1.0).
    pub damping: f64,
    /// Feedback/resonance amount (0.0-1.0).
    pub resonance: f64,
}

impl WaveguideSynth {
    /// Creates a new waveguide synthesizer.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `breath` - Excitation strength (0.0-1.0)
    /// * `noise` - Noise mix (0.0-1.0)
    /// * `damping` - High-frequency damping (0.0-1.0)
    /// * `resonance` - Feedback amount (0.0-1.0)
    pub fn new(frequency: f64, breath: f64, noise: f64, damping: f64, resonance: f64) -> Self {
        Self {
            frequency,
            breath: breath.clamp(0.0, 1.0),
            noise: noise.clamp(0.0, 1.0),
            damping: damping.clamp(0.0, 1.0),
            resonance: resonance.clamp(0.0, 0.999),
        }
    }

    /// Creates a flute-like preset.
    pub fn flute(frequency: f64) -> Self {
        Self::new(frequency, 0.6, 0.3, 0.2, 0.7)
    }

    /// Creates a clarinet-like preset.
    pub fn clarinet(frequency: f64) -> Self {
        Self::new(frequency, 0.8, 0.15, 0.4, 0.85)
    }

    /// Creates a brass-like preset.
    pub fn brass(frequency: f64) -> Self {
        Self::new(frequency, 0.9, 0.1, 0.3, 0.9)
    }

    /// Creates a breathy/airy preset.
    pub fn breathy(frequency: f64) -> Self {
        Self::new(frequency, 0.5, 0.7, 0.5, 0.5)
    }
}

impl Synthesizer for WaveguideSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        // Handle zero/invalid frequency
        if self.frequency <= 0.0 || !self.frequency.is_finite() {
            return vec![0.0; num_samples];
        }

        // Calculate delay line length based on frequency
        let delay_length = (sample_rate / self.frequency).round() as usize;
        if delay_length == 0 {
            return vec![0.0; num_samples];
        }

        // Initialize delay line to zero
        let mut delay_line = vec![0.0; delay_length];
        let mut output = Vec::with_capacity(num_samples);

        // State for lowpass damping filter (one-pole)
        let mut filter_state = 0.0;

        // Damping coefficient: higher damping = more high-frequency absorption
        // Map damping [0,1] to filter coefficient [0.1, 0.9]
        let damping_coeff = 0.1 + self.damping * 0.8;

        // Phase for tonal excitation component
        let mut excitation_phase: f64 = 0.0;
        let phase_inc = 2.0 * std::f64::consts::PI * self.frequency / sample_rate;

        let mut write_pos = 0;

        for _ in 0..num_samples {
            // Read from delay line (one sample delayed from write position)
            let read_pos = (write_pos + 1) % delay_length;
            let delayed = delay_line[read_pos];

            // Apply one-pole lowpass filter for damping
            // y[n] = (1 - coeff) * x[n] + coeff * y[n-1]
            filter_state = (1.0 - damping_coeff) * delayed + damping_coeff * filter_state;

            // Generate excitation signal
            // Mix between noise and tonal components
            let noise_sample = rng.gen::<f64>() * 2.0 - 1.0;
            let tone_sample = excitation_phase.sin();
            let excitation = self.noise * noise_sample + (1.0 - self.noise) * tone_sample;

            // Scale excitation by breath parameter
            let scaled_excitation = excitation * self.breath;

            // Combine excitation with feedback
            // Apply nonlinearity (tanh saturation) to feedback for stability
            let feedback = (filter_state * self.resonance).tanh();
            let combined = scaled_excitation + feedback;

            // Write to delay line
            delay_line[write_pos] = combined;

            // Output is the delayed/filtered signal
            output.push(filter_state);

            // Advance positions
            write_pos = (write_pos + 1) % delay_length;
            excitation_phase += phase_inc;
            if excitation_phase > 2.0 * std::f64::consts::PI {
                excitation_phase -= 2.0 * std::f64::consts::PI;
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_waveguide_basic() {
        let synth = WaveguideSynth::new(440.0, 0.7, 0.3, 0.3, 0.8);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);
        // Should produce non-zero output
        let energy: f64 = samples.iter().map(|s| s.powi(2)).sum();
        assert!(energy > 0.0);
    }

    #[test]
    fn test_waveguide_presets() {
        let mut rng = create_rng(42);

        let flute = WaveguideSynth::flute(440.0);
        let flute_samples = flute.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(flute_samples.len(), 1000);

        let clarinet = WaveguideSynth::clarinet(220.0);
        let clarinet_samples = clarinet.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(clarinet_samples.len(), 1000);

        let brass = WaveguideSynth::brass(330.0);
        let brass_samples = brass.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(brass_samples.len(), 1000);

        let breathy = WaveguideSynth::breathy(550.0);
        let breathy_samples = breathy.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(breathy_samples.len(), 1000);
    }

    #[test]
    fn test_waveguide_determinism() {
        let synth = WaveguideSynth::new(440.0, 0.7, 0.3, 0.3, 0.8);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_waveguide_parameter_clamping() {
        // Test that out-of-range parameters are clamped
        let synth = WaveguideSynth::new(440.0, 2.0, -0.5, 1.5, 1.5);
        assert_eq!(synth.breath, 1.0);
        assert_eq!(synth.noise, 0.0);
        assert_eq!(synth.damping, 1.0);
        assert!(synth.resonance <= 0.999);
    }

    #[test]
    fn test_waveguide_zero_frequency_handling() {
        let synth = WaveguideSynth::new(0.0, 0.7, 0.3, 0.3, 0.8);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(100, 44100.0, &mut rng);

        // Should return silence for zero frequency
        assert_eq!(samples.len(), 100);
        assert!(samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_waveguide_high_resonance_stability() {
        // Ensure high resonance doesn't cause instability/explosion
        let synth = WaveguideSynth::new(440.0, 1.0, 0.5, 0.1, 0.99);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        // All samples should be finite
        assert!(samples.iter().all(|s| s.is_finite()));
        // Should not explode (bounded by tanh saturation)
        assert!(samples.iter().all(|s| s.abs() < 10.0));
    }
}
