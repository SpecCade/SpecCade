//! Additive synthesis with multiple harmonics.
//!
//! This module implements additive synthesis, which builds complex sounds
//! by combining multiple sine waves (partials) at harmonic frequencies.

use std::f64::consts::PI;

use rand_pcg::Pcg32;

use super::Synthesizer;

/// Additive synthesis using harmonic partials.
#[derive(Debug, Clone)]
pub struct HarmonicSynth {
    /// Base frequency in Hz.
    pub base_freq: f64,
    /// Harmonic amplitudes (index 0 = fundamental, index 1 = 2nd harmonic, etc.).
    pub harmonics: Vec<f64>,
    /// Phase offsets for each harmonic (optional).
    pub phases: Vec<f64>,
}

impl HarmonicSynth {
    /// Creates a new harmonic synthesizer.
    ///
    /// # Arguments
    /// * `base_freq` - Fundamental frequency in Hz
    /// * `harmonics` - Amplitude of each harmonic (0 = fundamental)
    pub fn new(base_freq: f64, harmonics: Vec<f64>) -> Self {
        let num_harmonics = harmonics.len();
        Self {
            base_freq,
            harmonics,
            phases: vec![0.0; num_harmonics],
        }
    }

    /// Creates a synthesizer with specified frequencies and amplitudes.
    ///
    /// This allows for non-harmonic additive synthesis.
    ///
    /// # Arguments
    /// * `frequencies` - Frequency of each partial in Hz
    /// * `amplitudes` - Amplitude of each partial
    pub fn with_frequencies(frequencies: Vec<f64>, amplitudes: Vec<f64>) -> PartialSynth {
        PartialSynth::new(frequencies, amplitudes)
    }

    /// Creates a sawtooth-like timbre using harmonic series.
    pub fn sawtooth(base_freq: f64, num_harmonics: usize) -> Self {
        // Sawtooth: 1/n amplitude for each harmonic
        let harmonics: Vec<f64> = (1..=num_harmonics).map(|n| 1.0 / n as f64).collect();
        Self::new(base_freq, harmonics)
    }

    /// Creates a square-like timbre using odd harmonics.
    pub fn square(base_freq: f64, num_harmonics: usize) -> Self {
        // Square: 1/n amplitude for odd harmonics only
        let harmonics: Vec<f64> = (1..=num_harmonics * 2)
            .map(|n| if n % 2 == 1 { 1.0 / n as f64 } else { 0.0 })
            .collect();
        Self::new(base_freq, harmonics)
    }

    /// Creates a triangle-like timbre using odd harmonics with alternating signs.
    pub fn triangle(base_freq: f64, num_harmonics: usize) -> Self {
        // Triangle: 1/n^2 amplitude for odd harmonics
        let harmonics: Vec<f64> = (1..=num_harmonics * 2)
            .map(|n| {
                if n % 2 == 1 {
                    1.0 / (n as f64).powi(2)
                } else {
                    0.0
                }
            })
            .collect();
        Self::new(base_freq, harmonics)
    }

    /// Creates an organ-like timbre with specific drawbar settings.
    ///
    /// # Arguments
    /// * `base_freq` - Fundamental frequency
    /// * `drawbars` - 9 drawbar levels (0.0 to 1.0), representing:
    ///   0: 16' (sub-octave), 1: 5 1/3' (3rd harmonic - 1 octave),
    ///   2: 8' (fundamental), 3: 4' (2nd harmonic), 4: 2 2/3' (3rd harmonic),
    ///   5: 2' (4th harmonic), 6: 1 3/5' (5th harmonic), 7: 1 1/3' (6th harmonic),
    ///   8: 1' (8th harmonic)
    pub fn organ(base_freq: f64, drawbars: [f64; 9]) -> Self {
        // Drawbar positions map to these harmonic ratios
        let ratios = [0.5, 1.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0];

        // Build harmonic array
        let max_harmonic = 8;
        let mut harmonics = vec![0.0; max_harmonic];

        for (i, &level) in drawbars.iter().enumerate() {
            let ratio = ratios[i];
            if ratio >= 1.0 {
                let harmonic_index = ratio as usize - 1;
                if harmonic_index < max_harmonic {
                    harmonics[harmonic_index] = level.max(harmonics[harmonic_index]);
                }
            }
        }

        Self::new(base_freq, harmonics)
    }

    /// Creates a bell-like timbre with bright harmonics.
    pub fn bright_bell(base_freq: f64) -> Self {
        // Emphasis on higher harmonics for bright bell sound
        Self::new(base_freq, vec![1.0, 0.9, 0.7, 0.6, 0.5, 0.3, 0.2, 0.1])
    }

    /// Sets phase offsets for harmonics.
    pub fn with_phases(mut self, phases: Vec<f64>) -> Self {
        self.phases = phases;
        self
    }

    /// Sets random initial phases (creates more natural sound).
    pub fn with_random_phases(mut self, rng: &mut Pcg32) -> Self {
        use rand::Rng;
        let two_pi = 2.0 * PI;
        self.phases = (0..self.harmonics.len())
            .map(|_| rng.gen::<f64>() * two_pi)
            .collect();
        self
    }
}

impl Synthesizer for HarmonicSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = vec![0.0; num_samples];
        let dt = 1.0 / sample_rate;
        let two_pi = 2.0 * PI;

        // Sum all harmonics
        for (h, &amp) in self.harmonics.iter().enumerate() {
            if amp.abs() < 1e-6 {
                continue; // Skip silent harmonics
            }

            let freq = self.base_freq * (h + 1) as f64;
            let phase_offset = self.phases.get(h).copied().unwrap_or(0.0);

            for (i, sample) in output.iter_mut().enumerate() {
                let t = i as f64 * dt;
                *sample += (two_pi * freq * t + phase_offset).sin() * amp;
            }
        }

        // Normalize
        let max = output
            .iter()
            .map(|s| s.abs())
            .fold(0.0_f64, |a, b| a.max(b));
        if max > 0.0 {
            for s in &mut output {
                *s /= max;
            }
        }

        output
    }
}

/// Partial synthesis with arbitrary frequencies.
///
/// Unlike HarmonicSynth which uses integer harmonic ratios, this allows
/// any set of frequencies to be combined.
#[derive(Debug, Clone)]
pub struct PartialSynth {
    /// Frequencies of each partial in Hz.
    pub frequencies: Vec<f64>,
    /// Amplitudes of each partial.
    pub amplitudes: Vec<f64>,
    /// Phase offsets for each partial.
    pub phases: Vec<f64>,
}

impl PartialSynth {
    /// Creates a new partial synthesizer.
    pub fn new(frequencies: Vec<f64>, amplitudes: Vec<f64>) -> Self {
        let num = frequencies.len().min(amplitudes.len());
        Self {
            frequencies: frequencies[..num].to_vec(),
            amplitudes: amplitudes[..num].to_vec(),
            phases: vec![0.0; num],
        }
    }

    /// Sets phase offsets.
    pub fn with_phases(mut self, phases: Vec<f64>) -> Self {
        self.phases = phases;
        self
    }

    /// Sets random initial phases.
    pub fn with_random_phases(mut self, rng: &mut Pcg32) -> Self {
        use rand::Rng;
        let two_pi = 2.0 * PI;
        self.phases = (0..self.frequencies.len())
            .map(|_| rng.gen::<f64>() * two_pi)
            .collect();
        self
    }
}

impl Synthesizer for PartialSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = vec![0.0; num_samples];
        let dt = 1.0 / sample_rate;
        let two_pi = 2.0 * PI;

        for (i, (&freq, &amp)) in self
            .frequencies
            .iter()
            .zip(self.amplitudes.iter())
            .enumerate()
        {
            if amp.abs() < 1e-6 {
                continue;
            }

            let phase_offset = self.phases.get(i).copied().unwrap_or(0.0);

            for (j, sample) in output.iter_mut().enumerate() {
                let t = j as f64 * dt;
                *sample += (two_pi * freq * t + phase_offset).sin() * amp;
            }
        }

        // Normalize
        let max = output
            .iter()
            .map(|s| s.abs())
            .fold(0.0_f64, |a, b| a.max(b));
        if max > 0.0 {
            for s in &mut output {
                *s /= max;
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
    fn test_harmonic_synth() {
        let synth = HarmonicSynth::new(440.0, vec![1.0, 0.5, 0.25, 0.125]);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_sawtooth_timbre() {
        let synth = HarmonicSynth::sawtooth(440.0, 8);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_square_timbre() {
        let synth = HarmonicSynth::square(440.0, 8);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Square wave should only have odd harmonics
        assert_eq!(synth.harmonics[1], 0.0); // 2nd harmonic should be 0
    }

    #[test]
    fn test_organ_timbre() {
        let drawbars = [0.0, 0.0, 1.0, 0.5, 0.3, 0.2, 0.0, 0.0, 0.1];
        let synth = HarmonicSynth::organ(440.0, drawbars);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_partial_synth() {
        let synth = PartialSynth::new(vec![440.0, 550.0, 660.0], vec![1.0, 0.5, 0.3]);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_harmonic_determinism() {
        let synth = HarmonicSynth::new(440.0, vec![1.0, 0.5, 0.25]);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }
}
