//! AM (Amplitude Modulation) synthesis.
//!
//! AM synthesis creates sounds by modulating the amplitude of a carrier
//! oscillator with a modulator oscillator. This produces tremolo effects
//! and sidebands at (carrier +/- modulator) frequencies.
//!
//! Common AM sounds include tremolo guitar, ring modulation effects,
//! and broadcast-style amplitude modulation.

use std::f64::consts::PI;

use rand_pcg::Pcg32;

use super::{FrequencySweep, Synthesizer};

/// AM (Amplitude Modulation) synthesis parameters.
///
/// The output formula is: `output = carrier * (1.0 + depth * modulator)`
/// where both carrier and modulator are sine oscillators.
#[derive(Debug, Clone)]
pub struct AmSynth {
    /// Carrier frequency in Hz.
    pub carrier_freq: f64,
    /// Modulator frequency in Hz (typically lower than carrier for tremolo effects).
    pub modulator_freq: f64,
    /// Modulation depth (0.0 to 1.0, where 1.0 = 100% modulation).
    pub modulation_depth: f64,
    /// Optional carrier frequency sweep.
    pub freq_sweep: Option<FrequencySweep>,
}

impl AmSynth {
    /// Creates a new AM synthesizer.
    ///
    /// # Arguments
    /// * `carrier_freq` - Carrier frequency in Hz
    /// * `modulator_freq` - Modulator frequency in Hz
    /// * `modulation_depth` - Modulation depth (0.0 to 1.0)
    pub fn new(carrier_freq: f64, modulator_freq: f64, modulation_depth: f64) -> Self {
        Self {
            carrier_freq,
            modulator_freq,
            modulation_depth: modulation_depth.clamp(0.0, 1.0),
            freq_sweep: None,
        }
    }

    /// Creates an AM synthesizer using frequency ratio.
    ///
    /// # Arguments
    /// * `carrier_freq` - Carrier frequency in Hz
    /// * `ratio` - Modulator/carrier frequency ratio
    /// * `modulation_depth` - Modulation depth (0.0 to 1.0)
    pub fn with_ratio(carrier_freq: f64, ratio: f64, modulation_depth: f64) -> Self {
        Self {
            carrier_freq,
            modulator_freq: carrier_freq * ratio,
            modulation_depth: modulation_depth.clamp(0.0, 1.0),
            freq_sweep: None,
        }
    }

    /// Sets a frequency sweep on the carrier.
    pub fn with_sweep(mut self, sweep: FrequencySweep) -> Self {
        self.freq_sweep = Some(sweep);
        self
    }

    /// Creates a tremolo effect preset.
    ///
    /// Tremolo uses a low modulator frequency (typically 4-8 Hz) to create
    /// a pulsing amplitude variation on the carrier.
    ///
    /// # Arguments
    /// * `carrier_freq` - The main tone frequency in Hz
    /// * `tremolo_rate` - Tremolo rate in Hz (typical: 4-8 Hz)
    /// * `depth` - Tremolo depth (0.0 to 1.0)
    pub fn tremolo(carrier_freq: f64, tremolo_rate: f64, depth: f64) -> Self {
        Self {
            carrier_freq,
            modulator_freq: tremolo_rate,
            modulation_depth: depth.clamp(0.0, 1.0),
            freq_sweep: None,
        }
    }

    /// Creates a broadcast-style AM preset.
    ///
    /// Uses a moderate modulator frequency for classic AM radio-style modulation.
    ///
    /// # Arguments
    /// * `carrier_freq` - Carrier frequency in Hz
    /// * `audio_freq` - Audio/modulator frequency in Hz
    pub fn broadcast(carrier_freq: f64, audio_freq: f64) -> Self {
        Self {
            carrier_freq,
            modulator_freq: audio_freq,
            modulation_depth: 0.8,
            freq_sweep: None,
        }
    }

    /// Creates a siren-like effect preset.
    ///
    /// Uses a higher modulation rate for a warbling siren sound.
    ///
    /// # Arguments
    /// * `base_freq` - Base frequency in Hz
    pub fn siren(base_freq: f64) -> Self {
        Self {
            carrier_freq: base_freq,
            modulator_freq: 12.0,
            modulation_depth: 0.6,
            freq_sweep: None,
        }
    }
}

impl Synthesizer for AmSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = Vec::with_capacity(num_samples);

        let dt = 1.0 / sample_rate;
        let two_pi = 2.0 * PI;

        let mut carrier_phase: f64 = 0.0;
        let mut modulator_phase: f64 = 0.0;

        for i in 0..num_samples {
            let progress = i as f64 / num_samples as f64;

            // Get carrier frequency (with optional sweep)
            let carrier_freq = if let Some(ref sweep) = self.freq_sweep {
                sweep.at(progress)
            } else {
                self.carrier_freq
            };

            // Scale modulator frequency proportionally if sweeping
            let mod_ratio = self.modulator_freq / self.carrier_freq;
            let modulator_freq = carrier_freq * mod_ratio;

            // AM equation: output = carrier * (1.0 + depth * modulator)
            let carrier = carrier_phase.sin();
            let modulator = modulator_phase.sin();
            let amplitude = 1.0 + self.modulation_depth * modulator;
            let sample = carrier * amplitude;

            // Clamp output to [-1.0, 1.0] range
            // With depth <= 1.0, amplitude ranges from (1-depth) to (1+depth)
            // Maximum output magnitude is (1+depth) which can exceed 1.0
            // Normalize by dividing by (1+depth) to keep within range
            let normalized = sample / (1.0 + self.modulation_depth);
            output.push(normalized.clamp(-1.0, 1.0));

            // Update phases
            carrier_phase += two_pi * carrier_freq * dt;
            modulator_phase += two_pi * modulator_freq * dt;

            // Wrap phases to prevent precision loss
            if carrier_phase >= two_pi {
                carrier_phase -= two_pi;
            }
            if modulator_phase >= two_pi {
                modulator_phase -= two_pi;
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;
    use crate::synthesis::SweepCurve;

    #[test]
    fn test_basic_am() {
        let synth = AmSynth::new(440.0, 5.0, 0.5);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!(
                (-1.0..=1.0).contains(&s),
                "Sample {} out of range [-1.0, 1.0]",
                s
            );
        }
    }

    #[test]
    fn test_am_with_ratio() {
        let synth = AmSynth::with_ratio(440.0, 0.01, 0.8);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Modulator should be at 4.4 Hz (440 * 0.01)
        assert!((synth.modulator_freq - 4.4).abs() < 0.001);
    }

    #[test]
    fn test_am_tremolo_preset() {
        let synth = AmSynth::tremolo(440.0, 6.0, 0.7);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);
        // Check that tremolo creates amplitude variation
        // Find max and min in a segment
        let segment = &samples[0..4410]; // First 0.1 seconds
        let max_val = segment.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_val = segment.iter().cloned().fold(f64::INFINITY, f64::min);

        // With tremolo, there should be noticeable amplitude variation
        assert!(
            max_val > min_val,
            "Tremolo should create amplitude variation"
        );
    }

    #[test]
    fn test_am_with_sweep() {
        let sweep = FrequencySweep::new(440.0, 880.0, SweepCurve::Linear);
        let synth = AmSynth::new(440.0, 5.0, 0.5).with_sweep(sweep);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_am_full_depth() {
        // Test with maximum modulation depth
        let synth = AmSynth::new(440.0, 10.0, 1.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!(
                (-1.0..=1.0).contains(&s),
                "Sample {} out of range at full depth",
                s
            );
        }
    }

    #[test]
    fn test_am_zero_depth() {
        // With zero modulation depth, output should be pure carrier
        let synth = AmSynth::new(440.0, 10.0, 0.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        // Compare with a pure sine wave
        let sine_synth = AmSynth::new(440.0, 0.0, 0.0);
        let sine_samples = sine_synth.synthesize(1000, 44100.0, &mut create_rng(42));

        // They should be very similar (both are just sine waves)
        for (am, sine) in samples.iter().zip(sine_samples.iter()) {
            assert!(
                (am - sine).abs() < 0.001,
                "Zero depth should produce pure carrier"
            );
        }
    }

    #[test]
    fn test_am_determinism() {
        let synth = AmSynth::new(440.0, 5.0, 0.5);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2, "AM synthesis must be deterministic");
    }

    #[test]
    fn test_broadcast_preset() {
        let synth = AmSynth::broadcast(1000.0, 440.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        assert_eq!(synth.carrier_freq, 1000.0);
        assert_eq!(synth.modulator_freq, 440.0);
        assert!((synth.modulation_depth - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_siren_preset() {
        let synth = AmSynth::siren(800.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        assert_eq!(synth.carrier_freq, 800.0);
        assert_eq!(synth.modulator_freq, 12.0);
        assert!((synth.modulation_depth - 0.6).abs() < 0.001);
    }
}
