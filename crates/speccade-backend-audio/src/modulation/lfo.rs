//! LFO (Low Frequency Oscillator) implementation.
//!
//! Generates modulation signals for various synthesis parameters.

use crate::oscillator::{sawtooth, sine, square, triangle, PhaseAccumulator};
use rand_pcg::Pcg32;
use speccade_spec::recipe::audio::Waveform;

/// LFO generator.
#[derive(Debug, Clone)]
pub struct Lfo {
    /// LFO waveform type.
    waveform: Waveform,
    /// LFO rate in Hz.
    rate: f64,
    /// Modulation depth (0.0-1.0).
    depth: f64,
    /// Phase accumulator for waveform generation.
    phase_acc: PhaseAccumulator,
    /// Previous sample for sample-and-hold (random waveform).
    sh_value: f64,
    /// Phase at which to update sample-and-hold.
    sh_update_phase: f64,
}

impl Lfo {
    /// Creates a new LFO.
    ///
    /// # Arguments
    /// * `waveform` - The waveform type (sine, triangle, square, sawtooth, pulse)
    /// * `rate` - LFO rate in Hz (typically 0.1-20 Hz)
    /// * `depth` - Modulation depth (0.0-1.0)
    /// * `sample_rate` - Audio sample rate
    /// * `initial_phase` - Initial phase offset (0.0-1.0)
    pub fn new(
        waveform: Waveform,
        rate: f64,
        depth: f64,
        sample_rate: f64,
        initial_phase: f64,
    ) -> Self {
        let mut phase_acc = PhaseAccumulator::new(sample_rate);

        // Set initial phase by advancing the phase accumulator
        if initial_phase > 0.0 {
            // Advance the phase accumulator to the initial phase
            for _ in 0..(initial_phase * sample_rate / rate).floor() as usize {
                phase_acc.advance(rate);
            }
        }

        Self {
            waveform,
            rate,
            depth,
            phase_acc,
            sh_value: 0.0,
            sh_update_phase: 0.0,
        }
    }

    /// Generates the next LFO sample.
    ///
    /// Returns a value in the range [0.0, 1.0].
    ///
    /// # Arguments
    /// * `_rng` - Random number generator for sample-and-hold mode (unused for now)
    pub fn next_sample(&mut self, _rng: &mut Pcg32) -> f64 {
        let phase = self.phase_acc.advance(self.rate);

        // For pulse waveform, treat it as square with 50% duty
        let waveform = if self.waveform == Waveform::Pulse {
            Waveform::Square
        } else {
            self.waveform
        };

        let raw_value = match waveform {
            Waveform::Sine => sine(phase),
            Waveform::Square | Waveform::Pulse => square(phase, 0.5),
            Waveform::Sawtooth => sawtooth(phase),
            Waveform::Triangle => triangle(phase),
        };

        // Convert from [-1.0, 1.0] to [0.0, 1.0]
        let normalized = (raw_value + 1.0) * 0.5;

        // Apply depth: (1 - depth) represents center point, depth scales the modulation
        let modulated = (1.0 - self.depth) + normalized * self.depth;

        modulated.clamp(0.0, 1.0)
    }

    /// Generates a buffer of LFO samples.
    ///
    /// # Arguments
    /// * `num_samples` - Number of samples to generate
    /// * `rng` - Random number generator
    ///
    /// # Returns
    /// Vector of LFO values in range [0.0, 1.0]
    pub fn generate(&mut self, num_samples: usize, rng: &mut Pcg32) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            output.push(self.next_sample(rng));
        }
        output
    }
}

/// Applies pitch modulation to a frequency value.
///
/// # Arguments
/// * `frequency` - Base frequency in Hz
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `semitones` - Maximum pitch deviation in semitones
///
/// # Returns
/// Modulated frequency in Hz
pub fn apply_pitch_modulation(frequency: f64, lfo_value: f64, semitones: f64) -> f64 {
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    // Convert semitones to frequency multiplier
    let pitch_shift = bipolar * semitones;
    frequency * 2.0_f64.powf(pitch_shift / 12.0)
}

/// Applies volume modulation to an amplitude value.
///
/// # Arguments
/// * `amplitude` - Base amplitude
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `depth` - Modulation depth (0.0-1.0)
///
/// # Returns
/// Modulated amplitude
pub fn apply_volume_modulation(amplitude: f64, lfo_value: f64, depth: f64) -> f64 {
    // LFO value is already in [0.0, 1.0], scale by depth
    amplitude * ((1.0 - depth) + lfo_value * depth)
}

/// Applies filter cutoff modulation.
///
/// # Arguments
/// * `base_cutoff` - Base cutoff frequency in Hz
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `amount` - Maximum cutoff change in Hz
///
/// # Returns
/// Modulated cutoff frequency in Hz
pub fn apply_filter_cutoff_modulation(base_cutoff: f64, lfo_value: f64, amount: f64) -> f64 {
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    (base_cutoff + bipolar * amount).max(20.0) // Ensure cutoff doesn't go below 20 Hz
}

/// Applies pan modulation.
///
/// # Arguments
/// * `base_pan` - Base pan position (-1.0 to 1.0)
/// * `lfo_value` - LFO modulation value (0.0-1.0)
/// * `depth` - Modulation depth (0.0-1.0)
///
/// # Returns
/// Modulated pan position (-1.0 to 1.0)
pub fn apply_pan_modulation(base_pan: f64, lfo_value: f64, depth: f64) -> f64 {
    // Convert LFO value from [0.0, 1.0] to [-1.0, 1.0]
    let bipolar = (lfo_value - 0.5) * 2.0;
    (base_pan + bipolar * depth).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_lfo_sine_generation() {
        let mut rng = create_rng(42);
        let mut lfo = Lfo::new(Waveform::Sine, 1.0, 1.0, 44100.0, 0.0);

        // Generate some samples
        let samples = lfo.generate(100, &mut rng);

        // All samples should be in range [0.0, 1.0]
        for sample in &samples {
            assert!(
                (0.0..=1.0).contains(sample),
                "Sample {} out of range",
                sample
            );
        }
    }

    #[test]
    fn test_lfo_determinism() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let mut lfo1 = Lfo::new(Waveform::Sine, 2.0, 0.8, 44100.0, 0.0);
        let mut lfo2 = Lfo::new(Waveform::Sine, 2.0, 0.8, 44100.0, 0.0);

        let samples1 = lfo1.generate(100, &mut rng1);
        let samples2 = lfo2.generate(100, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_pitch_modulation() {
        let base_freq = 440.0; // A4
        let lfo_value = 0.5; // Center (no modulation)
        let semitones = 12.0; // One octave range

        let modulated = apply_pitch_modulation(base_freq, lfo_value, semitones);

        // At center, should be close to base frequency
        assert!((modulated - base_freq).abs() < 0.1);

        // At max (lfo_value = 1.0), should be one octave up
        let modulated_max = apply_pitch_modulation(base_freq, 1.0, semitones);
        assert!((modulated_max / base_freq - 2.0).abs() < 0.01);

        // At min (lfo_value = 0.0), should be one octave down
        let modulated_min = apply_pitch_modulation(base_freq, 0.0, semitones);
        assert!((modulated_min / base_freq - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_volume_modulation() {
        let amplitude = 1.0;
        let depth = 1.0;

        // At LFO max, should be full amplitude
        let mod_max = apply_volume_modulation(amplitude, 1.0, depth);
        assert!((mod_max - 1.0).abs() < 0.01);

        // At LFO min, should be zero with full depth
        let mod_min = apply_volume_modulation(amplitude, 0.0, depth);
        assert!(mod_min < 0.01);

        // At LFO center with half depth, should be 0.75
        let mod_center = apply_volume_modulation(amplitude, 0.5, 0.5);
        assert!((mod_center - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_filter_cutoff_modulation() {
        let base_cutoff = 1000.0;
        let amount = 500.0;

        // At center, should be base cutoff
        let mod_center = apply_filter_cutoff_modulation(base_cutoff, 0.5, amount);
        assert!((mod_center - base_cutoff).abs() < 0.1);

        // At max, should be base + amount
        let mod_max = apply_filter_cutoff_modulation(base_cutoff, 1.0, amount);
        assert!((mod_max - (base_cutoff + amount)).abs() < 0.1);

        // At min, should be base - amount
        let mod_min = apply_filter_cutoff_modulation(base_cutoff, 0.0, amount);
        assert!((mod_min - (base_cutoff - amount)).abs() < 0.1);
    }

    #[test]
    fn test_pan_modulation() {
        let base_pan = 0.0; // Center
        let depth = 1.0;

        // At center, should be close to base
        let mod_center = apply_pan_modulation(base_pan, 0.5, depth);
        assert!(mod_center.abs() < 0.01);

        // At max, should be right
        let mod_max = apply_pan_modulation(base_pan, 1.0, depth);
        assert!((mod_max - 1.0).abs() < 0.01);

        // At min, should be left
        let mod_min = apply_pan_modulation(base_pan, 0.0, depth);
        assert!((mod_min + 1.0).abs() < 0.01);
    }
}
