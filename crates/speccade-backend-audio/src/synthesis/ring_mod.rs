//! Ring Modulation synthesis.
//!
//! Ring modulation multiplies two signals directly (carrier * modulator),
//! producing sum and difference frequencies. Unlike AM which has a DC offset
//! (output = carrier * (1 + modulator)), ring modulation creates pure
//! sidebands at (carrier + modulator) and (carrier - modulator) frequencies.
//!
//! This creates metallic, bell-like, and robotic timbres commonly used in
//! electronic music and sound design.

use std::f64::consts::PI;

use rand_pcg::Pcg32;

use super::{FrequencySweep, Synthesizer};

/// Ring Modulation synthesis parameters.
///
/// The output formula is: `output = carrier * modulator`
/// where both carrier and modulator are sine oscillators.
///
/// This produces frequency components at:
/// - (carrier_freq + modulator_freq) - sum frequency
/// - (carrier_freq - modulator_freq) - difference frequency
#[derive(Debug, Clone)]
pub struct RingModSynth {
    /// Carrier frequency in Hz.
    pub carrier_freq: f64,
    /// Modulator frequency in Hz.
    pub modulator_freq: f64,
    /// Wet/dry mix (0.0 = pure carrier, 1.0 = pure ring modulation).
    pub mix: f64,
    /// Optional carrier frequency sweep.
    pub freq_sweep: Option<FrequencySweep>,
}

impl RingModSynth {
    /// Creates a new Ring Modulation synthesizer.
    ///
    /// # Arguments
    /// * `carrier_freq` - Carrier frequency in Hz
    /// * `modulator_freq` - Modulator frequency in Hz
    /// * `mix` - Wet/dry mix (0.0 to 1.0)
    pub fn new(carrier_freq: f64, modulator_freq: f64, mix: f64) -> Self {
        Self {
            carrier_freq,
            modulator_freq,
            mix: mix.clamp(0.0, 1.0),
            freq_sweep: None,
        }
    }

    /// Creates a Ring Modulation synthesizer using frequency ratio.
    ///
    /// # Arguments
    /// * `carrier_freq` - Carrier frequency in Hz
    /// * `ratio` - Modulator/carrier frequency ratio
    /// * `mix` - Wet/dry mix (0.0 to 1.0)
    pub fn with_ratio(carrier_freq: f64, ratio: f64, mix: f64) -> Self {
        Self {
            carrier_freq,
            modulator_freq: carrier_freq * ratio,
            mix: mix.clamp(0.0, 1.0),
            freq_sweep: None,
        }
    }

    /// Sets a frequency sweep on the carrier.
    pub fn with_sweep(mut self, sweep: FrequencySweep) -> Self {
        self.freq_sweep = Some(sweep);
        self
    }

    /// Creates a metallic preset.
    ///
    /// Uses inharmonic frequency relationships to produce metallic timbres.
    /// The modulator is set to an irrational ratio of the carrier.
    ///
    /// # Arguments
    /// * `carrier_freq` - The main tone frequency in Hz
    pub fn metallic(carrier_freq: f64) -> Self {
        Self {
            carrier_freq,
            // Irrational ratio creates inharmonic sidebands
            modulator_freq: carrier_freq * 1.414, // sqrt(2)
            mix: 0.8,
            freq_sweep: None,
        }
    }

    /// Creates a robotic/vocoder-like preset.
    ///
    /// Uses a low modulator frequency to create a robotic, mechanical sound.
    ///
    /// # Arguments
    /// * `carrier_freq` - The main tone frequency in Hz
    pub fn robotic(carrier_freq: f64) -> Self {
        Self {
            carrier_freq,
            modulator_freq: 50.0, // Low frequency creates rough texture
            mix: 1.0,
            freq_sweep: None,
        }
    }

    /// Creates a bell-like preset.
    ///
    /// Uses a slightly detuned modulator to create bell-like inharmonic tones.
    ///
    /// # Arguments
    /// * `carrier_freq` - The main tone frequency in Hz
    pub fn bell(carrier_freq: f64) -> Self {
        Self {
            carrier_freq,
            // Golden ratio creates pleasant inharmonic relationship
            modulator_freq: carrier_freq * 1.618,
            mix: 0.7,
            freq_sweep: None,
        }
    }

    /// Creates a dissonant/atonal preset.
    ///
    /// Uses prime number ratio for maximum inharmonicity.
    ///
    /// # Arguments
    /// * `carrier_freq` - The main tone frequency in Hz
    pub fn dissonant(carrier_freq: f64) -> Self {
        Self {
            carrier_freq,
            // Non-octave ratio creates dissonance
            modulator_freq: carrier_freq * 1.333, // 4/3 ratio
            mix: 0.9,
            freq_sweep: None,
        }
    }

    /// Creates a sci-fi/alien preset.
    ///
    /// Uses a high modulator frequency for otherworldly tones.
    ///
    /// # Arguments
    /// * `carrier_freq` - The main tone frequency in Hz
    pub fn scifi(carrier_freq: f64) -> Self {
        Self {
            carrier_freq,
            modulator_freq: carrier_freq * 2.5,
            mix: 1.0,
            freq_sweep: None,
        }
    }
}

impl Synthesizer for RingModSynth {
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

            // Generate oscillators
            let carrier = carrier_phase.sin();
            let modulator = modulator_phase.sin();

            // Ring modulation: carrier * modulator
            // This produces frequencies at (carrier + modulator) and (carrier - modulator)
            let ring_mod = carrier * modulator;

            // Mix between pure carrier and ring modulated signal
            let sample = carrier * (1.0 - self.mix) + ring_mod * self.mix;

            // Output is already in [-1.0, 1.0] range since:
            // - carrier * modulator is in [-1.0, 1.0] (product of two sine waves)
            // - linear interpolation between two values in [-1.0, 1.0] stays in range
            output.push(sample.clamp(-1.0, 1.0));

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
    fn test_basic_ring_mod() {
        let synth = RingModSynth::new(440.0, 300.0, 1.0);
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
    fn test_ring_mod_with_ratio() {
        let synth = RingModSynth::with_ratio(440.0, 1.5, 0.8);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Modulator should be at 660 Hz (440 * 1.5)
        assert!((synth.modulator_freq - 660.0).abs() < 0.001);
    }

    #[test]
    fn test_ring_mod_metallic_preset() {
        let synth = RingModSynth::metallic(440.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(4410, 44100.0, &mut rng);

        assert_eq!(samples.len(), 4410);
        // Check that we have varied output (not a flat line)
        let max_val = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_val = samples.iter().cloned().fold(f64::INFINITY, f64::min);
        assert!(
            max_val - min_val > 0.1,
            "Ring mod should produce amplitude variation"
        );
    }

    #[test]
    fn test_ring_mod_with_sweep() {
        let sweep = FrequencySweep::new(440.0, 880.0, SweepCurve::Linear);
        let synth = RingModSynth::new(440.0, 300.0, 1.0).with_sweep(sweep);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_ring_mod_zero_mix() {
        // With zero mix, output should be pure carrier (sine wave)
        let synth = RingModSynth::new(440.0, 300.0, 0.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(100, 44100.0, &mut rng);

        // Generate a pure sine for comparison
        let two_pi = 2.0 * PI;
        let dt = 1.0 / 44100.0;
        for (i, &sample) in samples.iter().enumerate() {
            let expected = (two_pi * 440.0 * i as f64 * dt).sin();
            assert!(
                (sample - expected).abs() < 0.001,
                "Zero mix should produce pure carrier at sample {}: got {}, expected {}",
                i,
                sample,
                expected
            );
        }
    }

    #[test]
    fn test_ring_mod_full_mix() {
        // With full mix, output should be carrier * modulator
        let synth = RingModSynth::new(440.0, 300.0, 1.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(100, 44100.0, &mut rng);

        // Verify the ring modulation formula
        let two_pi = 2.0 * PI;
        let dt = 1.0 / 44100.0;
        for (i, &sample) in samples.iter().enumerate() {
            let carrier = (two_pi * 440.0 * i as f64 * dt).sin();
            let modulator = (two_pi * 300.0 * i as f64 * dt).sin();
            let expected = carrier * modulator;
            assert!(
                (sample - expected).abs() < 0.001,
                "Full mix should produce carrier * modulator at sample {}: got {}, expected {}",
                i,
                sample,
                expected
            );
        }
    }

    #[test]
    fn test_ring_mod_presets() {
        let mut rng = create_rng(42);

        let robotic = RingModSynth::robotic(440.0);
        let samples = robotic.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(samples.len(), 1000);
        assert_eq!(robotic.modulator_freq, 50.0);

        let bell = RingModSynth::bell(440.0);
        let samples = bell.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(samples.len(), 1000);
        assert!((bell.modulator_freq - 440.0 * 1.618).abs() < 0.001);

        let scifi = RingModSynth::scifi(440.0);
        let samples = scifi.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_ring_mod_determinism() {
        let synth = RingModSynth::new(440.0, 300.0, 0.7);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2, "Ring modulation synthesis must be deterministic");
    }
}
