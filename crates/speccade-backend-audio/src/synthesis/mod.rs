//! Synthesis modules for various sound generation techniques.
//!
//! Each module implements a specific synthesis technique:
//! - `sine` - Pure sine wave with optional frequency sweep
//! - `square` - Square/pulse wave with duty cycle
//! - `saw` - Sawtooth wave
//! - `triangle` - Triangle wave
//! - `noise` - White/pink/brown noise with optional filter
//! - `fm` - FM synthesis (carrier + modulator)
//! - `feedback_fm` - Feedback FM synthesis (self-modulating operator)
//! - `am` - AM synthesis (amplitude modulation)
//! - `ring_mod` - Ring modulation synthesis (carrier * modulator)
//! - `karplus` - Karplus-Strong plucked string synthesis
//! - `bowed_string` - Bowed string synthesis for violin/cello-like sounds
//! - `comb_synth` - Comb filter synthesis for resonant metallic tones
//! - `pitched_body` - Frequency sweep for impact sounds
//! - `metallic` - Inharmonic partials for metallic sounds
//! - `harmonics` - Additive synthesis with multiple harmonics
//! - `granular` - Granular synthesis with grain processing
//! - `wavetable` - Wavetable synthesis with morphing
//! - `phase_distortion` - Phase distortion (Casio CZ style) synthesis
//! - `modal` - Modal synthesis for struck/bowed objects (bells, chimes, bars)
//! - `membrane` - Membrane drum synthesis for toms, hand drums, congas, etc.
//! - `vocoder` - Vocoder synthesis with filter bank and formant animation
//! - `formant` - Formant synthesis for vowel and voice sounds
//! - `vector` - Vector synthesis with 2D crossfading between multiple sources
//! - `waveguide` - Waveguide synthesis for wind/brass physical modeling
//! - `pulsar` - Pulsar synthesis (synchronized grain trains for rhythmic/tonal granular)
//! - `vosim` - VOSIM synthesis (voice simulation with squared-sine pulse trains)
//! - `spectral` - Spectral freeze synthesis (FFT-based frozen spectral content)

pub mod am;
pub mod bowed_string;
pub mod comb_synth;
pub mod feedback_fm;
pub mod fm;
pub mod formant;
pub mod granular;
pub mod harmonics;
pub mod karplus;
pub mod membrane;
pub mod metallic;
pub mod modal;
pub mod noise;
pub mod oscillators;
pub mod phase_distortion;
pub mod pitched_body;
pub mod pulsar;
pub mod ring_mod;
pub mod spectral;
pub mod vector;
pub mod vocoder;
pub mod vosim;
pub mod waveguide;
pub mod wavetable;

use rand_pcg::Pcg32;

/// Common trait for all synthesis modules.
pub trait Synthesizer {
    /// Generates audio samples.
    ///
    /// # Arguments
    /// * `num_samples` - Number of samples to generate
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `rng` - Deterministic RNG for any randomness
    ///
    /// # Returns
    /// Vector of audio samples in range [-1.0, 1.0]
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64>;
}

/// Frequency sweep curve types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SweepCurve {
    /// Linear interpolation.
    Linear,
    /// Exponential interpolation (perceptually linear for pitch).
    Exponential,
    /// Logarithmic interpolation.
    Logarithmic,
}

impl SweepCurve {
    /// Interpolates between start and end values.
    ///
    /// # Arguments
    /// * `start` - Starting value
    /// * `end` - Ending value
    /// * `t` - Progress (0.0 to 1.0)
    ///
    /// # Returns
    /// Interpolated value
    pub fn interpolate(&self, start: f64, end: f64, t: f64) -> f64 {
        match self {
            SweepCurve::Linear => start + (end - start) * t,
            SweepCurve::Exponential => {
                if start <= 0.0 || end <= 0.0 {
                    // Fall back to linear for non-positive values
                    start + (end - start) * t
                } else {
                    start * (end / start).powf(t)
                }
            }
            SweepCurve::Logarithmic => {
                // Logarithmic curve (slow start, fast end)
                let log_t = if t > 0.0 { 1.0 - (-t * 3.0).exp() } else { 0.0 };
                start + (end - start) * log_t
            }
        }
    }
}

/// Frequency sweep parameters.
#[derive(Debug, Clone, Copy)]
pub struct FrequencySweep {
    /// Starting frequency in Hz.
    pub start_freq: f64,
    /// Ending frequency in Hz.
    pub end_freq: f64,
    /// Sweep curve type.
    pub curve: SweepCurve,
}

impl FrequencySweep {
    /// Creates a new frequency sweep.
    pub fn new(start_freq: f64, end_freq: f64, curve: SweepCurve) -> Self {
        Self {
            start_freq,
            end_freq,
            curve,
        }
    }

    /// Gets the frequency at a given progress point.
    ///
    /// # Arguments
    /// * `t` - Progress (0.0 to 1.0)
    ///
    /// # Returns
    /// Frequency in Hz
    pub fn at(&self, t: f64) -> f64 {
        self.curve.interpolate(self.start_freq, self.end_freq, t)
    }

    /// Generates a frequency curve for the given number of samples.
    pub fn generate(&self, num_samples: usize) -> Vec<f64> {
        let divisor = if num_samples > 1 { num_samples - 1 } else { 1 };
        (0..num_samples)
            .map(|i| self.at(i as f64 / divisor as f64))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sweep_curve_linear() {
        let curve = SweepCurve::Linear;
        assert!((curve.interpolate(100.0, 200.0, 0.0) - 100.0).abs() < 0.01);
        assert!((curve.interpolate(100.0, 200.0, 0.5) - 150.0).abs() < 0.01);
        assert!((curve.interpolate(100.0, 200.0, 1.0) - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_sweep_curve_exponential() {
        let curve = SweepCurve::Exponential;
        assert!((curve.interpolate(100.0, 400.0, 0.0) - 100.0).abs() < 0.01);
        // Geometric mean at 0.5
        assert!((curve.interpolate(100.0, 400.0, 0.5) - 200.0).abs() < 1.0);
        assert!((curve.interpolate(100.0, 400.0, 1.0) - 400.0).abs() < 0.01);
    }

    #[test]
    fn test_frequency_sweep() {
        let sweep = FrequencySweep::new(440.0, 880.0, SweepCurve::Linear);
        let freqs = sweep.generate(11);

        assert_eq!(freqs.len(), 11);
        assert!((freqs[0] - 440.0).abs() < 0.01);
        assert!((freqs[5] - 660.0).abs() < 0.01);
        assert!((freqs[10] - 880.0).abs() < 0.01);
    }
}
