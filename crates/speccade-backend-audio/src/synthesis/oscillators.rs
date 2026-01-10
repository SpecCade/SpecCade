//! Basic oscillator synthesis (sine, square, saw, triangle).
//!
//! These are the fundamental waveform generators that produce periodic signals.

use rand_pcg::Pcg32;

use crate::oscillator::{self, PhaseAccumulator, TWO_PI};

use super::{FrequencySweep, SweepCurve, Synthesizer};

/// Sine wave synthesizer parameters.
#[derive(Debug, Clone)]
pub struct SineSynth {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Optional frequency sweep.
    pub freq_sweep: Option<FrequencySweep>,
}

impl SineSynth {
    /// Creates a new sine synthesizer.
    pub fn new(frequency: f64) -> Self {
        Self {
            frequency,
            freq_sweep: None,
        }
    }

    /// Creates a sine synthesizer with frequency sweep.
    pub fn with_sweep(start_freq: f64, end_freq: f64, curve: SweepCurve) -> Self {
        Self {
            frequency: start_freq,
            freq_sweep: Some(FrequencySweep::new(start_freq, end_freq, curve)),
        }
    }
}

impl Synthesizer for SineSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);
        let mut phase_acc = PhaseAccumulator::new(sample_rate);

        for i in 0..num_samples {
            let freq = if let Some(ref sweep) = self.freq_sweep {
                sweep.at(i as f64 / num_samples as f64)
            } else {
                self.frequency
            };

            let phase = phase_acc.advance(freq);
            output.push(oscillator::sine(phase));
        }

        output
    }
}

/// Square wave synthesizer parameters.
#[derive(Debug, Clone)]
pub struct SquareSynth {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Duty cycle (0.0 to 1.0, 0.5 = standard square wave).
    pub duty: f64,
    /// Optional frequency sweep.
    pub freq_sweep: Option<FrequencySweep>,
    /// Use band-limited (PolyBLEP) algorithm for anti-aliasing.
    pub band_limited: bool,
}

impl SquareSynth {
    /// Creates a new square wave synthesizer.
    pub fn new(frequency: f64) -> Self {
        Self {
            frequency,
            duty: 0.5,
            freq_sweep: None,
            band_limited: true,
        }
    }

    /// Creates a pulse wave synthesizer with specified duty cycle.
    pub fn pulse(frequency: f64, duty: f64) -> Self {
        Self {
            frequency,
            duty: duty.clamp(0.01, 0.99),
            freq_sweep: None,
            band_limited: true,
        }
    }

    /// Creates a square wave synthesizer with frequency sweep.
    pub fn with_sweep(start_freq: f64, end_freq: f64, curve: SweepCurve) -> Self {
        Self {
            frequency: start_freq,
            duty: 0.5,
            freq_sweep: Some(FrequencySweep::new(start_freq, end_freq, curve)),
            band_limited: true,
        }
    }
}

impl Synthesizer for SquareSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);
        let mut phase = 0.0;

        for i in 0..num_samples {
            let freq = if let Some(ref sweep) = self.freq_sweep {
                sweep.at(i as f64 / num_samples as f64)
            } else {
                self.frequency
            };

            let dt = freq / sample_rate;

            let sample = if self.band_limited {
                oscillator::polyblep_square(phase, dt, self.duty)
            } else {
                oscillator::square(phase * TWO_PI, self.duty)
            };

            output.push(sample);

            phase += dt;
            if phase >= 1.0 {
                phase -= 1.0;
            }
        }

        output
    }
}

/// Sawtooth wave synthesizer parameters.
#[derive(Debug, Clone)]
pub struct SawSynth {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Optional frequency sweep.
    pub freq_sweep: Option<FrequencySweep>,
    /// Use band-limited (PolyBLEP) algorithm for anti-aliasing.
    pub band_limited: bool,
}

impl SawSynth {
    /// Creates a new sawtooth wave synthesizer.
    pub fn new(frequency: f64) -> Self {
        Self {
            frequency,
            freq_sweep: None,
            band_limited: true,
        }
    }

    /// Creates a sawtooth synthesizer with frequency sweep.
    pub fn with_sweep(start_freq: f64, end_freq: f64, curve: SweepCurve) -> Self {
        Self {
            frequency: start_freq,
            freq_sweep: Some(FrequencySweep::new(start_freq, end_freq, curve)),
            band_limited: true,
        }
    }
}

impl Synthesizer for SawSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);
        let mut phase = 0.0;

        for i in 0..num_samples {
            let freq = if let Some(ref sweep) = self.freq_sweep {
                sweep.at(i as f64 / num_samples as f64)
            } else {
                self.frequency
            };

            let dt = freq / sample_rate;

            let sample = if self.band_limited {
                oscillator::polyblep_saw(phase, dt)
            } else {
                oscillator::sawtooth(phase * TWO_PI)
            };

            output.push(sample);

            phase += dt;
            if phase >= 1.0 {
                phase -= 1.0;
            }
        }

        output
    }
}

/// Triangle wave synthesizer parameters.
#[derive(Debug, Clone)]
pub struct TriangleSynth {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Optional frequency sweep.
    pub freq_sweep: Option<FrequencySweep>,
}

impl TriangleSynth {
    /// Creates a new triangle wave synthesizer.
    pub fn new(frequency: f64) -> Self {
        Self {
            frequency,
            freq_sweep: None,
        }
    }

    /// Creates a triangle synthesizer with frequency sweep.
    pub fn with_sweep(start_freq: f64, end_freq: f64, curve: SweepCurve) -> Self {
        Self {
            frequency: start_freq,
            freq_sweep: Some(FrequencySweep::new(start_freq, end_freq, curve)),
        }
    }
}

impl Synthesizer for TriangleSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);
        let mut phase_acc = PhaseAccumulator::new(sample_rate);

        for i in 0..num_samples {
            let freq = if let Some(ref sweep) = self.freq_sweep {
                sweep.at(i as f64 / num_samples as f64)
            } else {
                self.frequency
            };

            let phase = phase_acc.advance(freq);
            output.push(oscillator::triangle(phase));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_sine_synth() {
        let synth = SineSynth::new(440.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Check that all samples are in range
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_sine_synth_with_sweep() {
        let synth = SineSynth::with_sweep(440.0, 880.0, SweepCurve::Linear);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_square_synth() {
        let synth = SquareSynth::new(440.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_square_synth_duty() {
        let synth = SquareSynth::pulse(440.0, 0.25);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_saw_synth() {
        let synth = SawSynth::new(440.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_triangle_synth() {
        let synth = TriangleSynth::new(440.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_synthesis_determinism() {
        let synth = SineSynth::new(440.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }
}
