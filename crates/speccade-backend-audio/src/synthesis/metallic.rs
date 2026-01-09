//! Metallic synthesis with inharmonic partials.
//!
//! This module creates metallic, bell-like, and cymbal sounds using
//! inharmonic frequency ratios. Unlike harmonic sounds where partials
//! are integer multiples of the fundamental, metallic sounds use
//! non-integer ratios that create the characteristic metallic timbre.

use std::f64::consts::PI;

use rand::Rng;
use rand_pcg::Pcg32;

use super::Synthesizer;

/// Metallic sound synthesizer using inharmonic partials.
#[derive(Debug, Clone)]
pub struct MetallicSynth {
    /// Base frequency in Hz.
    pub base_freq: f64,
    /// Number of partials.
    pub num_partials: usize,
    /// Inharmonicity factor (1.0 = harmonic, >1.0 = increasingly inharmonic).
    pub inharmonicity: f64,
    /// Decay rate for higher partials.
    pub partial_decay: f64,
    /// Overall decay time in seconds.
    pub decay_time: f64,
    /// Random detune amount (0.0 to 1.0).
    pub detune: f64,
}

impl MetallicSynth {
    /// Creates a new metallic synthesizer.
    ///
    /// # Arguments
    /// * `base_freq` - Base frequency in Hz
    /// * `num_partials` - Number of inharmonic partials
    /// * `inharmonicity` - Inharmonicity factor (try 1.4 for bell, 2.0 for cymbal)
    pub fn new(base_freq: f64, num_partials: usize, inharmonicity: f64) -> Self {
        Self {
            base_freq,
            num_partials: num_partials.max(1),
            inharmonicity,
            partial_decay: 0.5,
            decay_time: 1.0,
            detune: 0.0,
        }
    }

    /// Creates a bell-like preset.
    pub fn bell(frequency: f64) -> Self {
        Self {
            base_freq: frequency,
            num_partials: 8,
            inharmonicity: 1.414, // sqrt(2)
            partial_decay: 0.6,
            decay_time: 2.0,
            detune: 0.01,
        }
    }

    /// Creates a small bell/chime preset.
    pub fn chime(frequency: f64) -> Self {
        Self {
            base_freq: frequency,
            num_partials: 6,
            inharmonicity: 1.3,
            partial_decay: 0.7,
            decay_time: 1.5,
            detune: 0.005,
        }
    }

    /// Creates a cymbal-like preset.
    pub fn cymbal(frequency: f64) -> Self {
        Self {
            base_freq: frequency,
            num_partials: 16,
            inharmonicity: 2.0,
            partial_decay: 0.3,
            decay_time: 0.5,
            detune: 0.1,
        }
    }

    /// Creates a hi-hat-like preset.
    pub fn hihat(frequency: f64) -> Self {
        Self {
            base_freq: frequency,
            num_partials: 12,
            inharmonicity: 2.2,
            partial_decay: 0.4,
            decay_time: 0.1,
            detune: 0.15,
        }
    }

    /// Creates a gong-like preset.
    pub fn gong(frequency: f64) -> Self {
        Self {
            base_freq: frequency,
            num_partials: 10,
            inharmonicity: 1.6,
            partial_decay: 0.5,
            decay_time: 4.0,
            detune: 0.02,
        }
    }

    /// Creates a metallic impact preset.
    pub fn metal_impact(frequency: f64) -> Self {
        Self {
            base_freq: frequency,
            num_partials: 6,
            inharmonicity: 1.5,
            partial_decay: 0.6,
            decay_time: 0.3,
            detune: 0.03,
        }
    }

    /// Sets the partial decay rate.
    pub fn with_partial_decay(mut self, decay: f64) -> Self {
        self.partial_decay = decay.clamp(0.0, 1.0);
        self
    }

    /// Sets the overall decay time.
    pub fn with_decay_time(mut self, time: f64) -> Self {
        self.decay_time = time.max(0.001);
        self
    }

    /// Sets the random detune amount.
    pub fn with_detune(mut self, detune: f64) -> Self {
        self.detune = detune.clamp(0.0, 1.0);
        self
    }
}

impl Synthesizer for MetallicSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        let mut output = vec![0.0; num_samples];
        let dt = 1.0 / sample_rate;
        let two_pi = 2.0 * PI;

        // Pre-calculate partial frequencies and amplitudes
        let mut partials: Vec<(f64, f64, f64)> = Vec::with_capacity(self.num_partials);

        for n in 0..self.num_partials {
            // Inharmonic frequency calculation
            // f_n = f_base * (n+1)^inharmonicity
            let partial_num = (n + 1) as f64;
            let freq = self.base_freq * partial_num.powf(self.inharmonicity);

            // Add random detune
            let detune_factor = if self.detune > 0.0 {
                1.0 + (rng.gen::<f64>() * 2.0 - 1.0) * self.detune
            } else {
                1.0
            };
            let freq = freq * detune_factor;

            // Amplitude decreases for higher partials
            let amp = self.partial_decay.powf(n as f64);

            // Random phase for each partial
            let phase = rng.gen::<f64>() * two_pi;

            partials.push((freq, amp, phase));
        }

        // Generate samples
        let decay_rate = -3.0 / (self.decay_time * sample_rate);

        for (i, sample) in output.iter_mut().enumerate() {
            let t = i as f64 * dt;
            let env = (decay_rate * i as f64).exp();

            let mut sum = 0.0;
            for &(freq, amp, phase) in &partials {
                // Each partial decays at different rates
                let partial_env = env.powf(1.0 + (freq / self.base_freq - 1.0) * 0.5);
                sum += (two_pi * freq * t + phase).sin() * amp * partial_env;
            }

            *sample = sum;
        }

        // Normalize
        let max = output.iter().map(|s| s.abs()).fold(0.0_f64, |a, b| a.max(b));
        if max > 0.0 {
            for s in &mut output {
                *s /= max;
            }
        }

        output
    }
}

/// Ring modulation for additional metallic character.
#[derive(Debug, Clone)]
pub struct RingMod {
    /// Carrier frequency in Hz.
    pub carrier_freq: f64,
    /// Modulator frequency in Hz.
    pub modulator_freq: f64,
    /// Mix between original and ring-modulated signal (0.0 = original, 1.0 = fully modulated).
    pub mix: f64,
}

impl RingMod {
    /// Creates a new ring modulator.
    pub fn new(carrier_freq: f64, modulator_freq: f64, mix: f64) -> Self {
        Self {
            carrier_freq,
            modulator_freq,
            mix: mix.clamp(0.0, 1.0),
        }
    }

    /// Applies ring modulation to a signal.
    pub fn process(&self, input: &[f64], sample_rate: f64) -> Vec<f64> {
        let two_pi = 2.0 * PI;
        let dt = 1.0 / sample_rate;

        input
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let t = i as f64 * dt;
                let carrier = (two_pi * self.carrier_freq * t).sin();
                let modulator = (two_pi * self.modulator_freq * t).sin();
                let ring = carrier * modulator;
                s * (1.0 - self.mix) + ring * self.mix
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_metallic_synth() {
        let synth = MetallicSynth::new(440.0, 6, 1.414);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!(s >= -1.0 && s <= 1.0);
        }
    }

    #[test]
    fn test_metallic_presets() {
        let mut rng = create_rng(42);

        let bell = MetallicSynth::bell(440.0);
        let bell_samples = bell.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(bell_samples.len(), 1000);

        let cymbal = MetallicSynth::cymbal(800.0);
        let cymbal_samples = cymbal.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(cymbal_samples.len(), 1000);

        let gong = MetallicSynth::gong(200.0);
        let gong_samples = gong.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(gong_samples.len(), 1000);
    }

    #[test]
    fn test_metallic_determinism() {
        let synth = MetallicSynth::bell(440.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_ring_mod() {
        let ring = RingMod::new(440.0, 300.0, 0.5);
        let input: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin()).collect();
        let output = ring.process(&input, 44100.0);

        assert_eq!(output.len(), 100);
    }
}
