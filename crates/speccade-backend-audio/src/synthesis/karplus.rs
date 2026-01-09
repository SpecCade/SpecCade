//! Karplus-Strong synthesis for plucked string sounds.
//!
//! The Karplus-Strong algorithm creates realistic plucked string sounds using
//! a simple feedback delay line with lowpass filtering. The initial noise burst
//! decays into a pitched tone.

use rand::Rng;
use rand_pcg::Pcg32;

use super::Synthesizer;

/// Karplus-Strong synthesis parameters.
#[derive(Debug, Clone)]
pub struct KarplusStrong {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Decay factor (0.0 to 1.0). Higher = longer sustain.
    pub decay: f64,
    /// Blend factor for lowpass filter (0.0 to 1.0). Higher = brighter.
    pub blend: f64,
    /// Stretch factor for pitch variation during decay.
    pub stretch: f64,
}

impl KarplusStrong {
    /// Creates a new Karplus-Strong synthesizer.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `decay` - Decay factor (0.9-0.999 typical)
    /// * `blend` - Brightness (0.5 = dark, 0.9 = bright)
    pub fn new(frequency: f64, decay: f64, blend: f64) -> Self {
        Self {
            frequency,
            decay: decay.clamp(0.0, 0.9999),
            blend: blend.clamp(0.0, 1.0),
            stretch: 1.0,
        }
    }

    /// Creates a guitar-like preset.
    pub fn guitar(frequency: f64) -> Self {
        Self {
            frequency,
            decay: 0.996,
            blend: 0.7,
            stretch: 1.0,
        }
    }

    /// Creates a bass-like preset.
    pub fn bass(frequency: f64) -> Self {
        Self {
            frequency,
            decay: 0.998,
            blend: 0.3,
            stretch: 1.0,
        }
    }

    /// Creates a harp-like preset.
    pub fn harp(frequency: f64) -> Self {
        Self {
            frequency,
            decay: 0.995,
            blend: 0.8,
            stretch: 1.0,
        }
    }

    /// Creates a drum-like preset (short decay).
    pub fn drum(frequency: f64) -> Self {
        Self {
            frequency,
            decay: 0.9,
            blend: 0.5,
            stretch: 0.5,
        }
    }

    /// Sets the stretch factor for pitch variation.
    pub fn with_stretch(mut self, stretch: f64) -> Self {
        self.stretch = stretch;
        self
    }
}

impl Synthesizer for KarplusStrong {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        // Calculate delay line length based on frequency
        let delay_length = (sample_rate / self.frequency).round() as usize;
        if delay_length == 0 {
            return vec![0.0; num_samples];
        }

        // Initialize delay line with noise burst
        let mut delay_line: Vec<f64> = (0..delay_length)
            .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
            .collect();

        let mut output = Vec::with_capacity(num_samples);
        let mut write_pos = 0;

        // Karplus-Strong with extended algorithm
        for _ in 0..num_samples {
            // Read from delay line
            let read_pos = write_pos;
            let next_pos = (write_pos + 1) % delay_length;

            // Two-point average lowpass filter with blend
            let current = delay_line[read_pos];
            let next = delay_line[next_pos];
            let filtered = self.blend * current + (1.0 - self.blend) * next;

            // Apply decay
            let new_sample = filtered * self.decay;

            // Stretch algorithm (optional pitch variation)
            if self.stretch != 1.0 && rng.gen::<f64>() > self.stretch {
                // Occasionally skip updating (creates pitch bend)
                output.push(current);
            } else {
                delay_line[write_pos] = new_sample;
                output.push(current);
            }

            write_pos = next_pos;
        }

        output
    }
}

/// Extended Karplus-Strong with more control over initial excitation.
#[derive(Debug, Clone)]
pub struct ExtendedKarplusStrong {
    /// Base parameters.
    pub base: KarplusStrong,
    /// Initial excitation type.
    pub excitation: ExcitationType,
    /// Dynamics (pick position) 0.0 = bridge, 1.0 = middle.
    pub pick_position: f64,
}

/// Type of initial excitation for the string.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExcitationType {
    /// Random noise burst (classic Karplus-Strong).
    Noise,
    /// Sawtooth-like excitation (more harmonic content).
    Sawtooth,
    /// Triangle excitation (softer sound).
    Triangle,
    /// Impulse excitation (sharp pluck).
    Impulse,
}

impl ExtendedKarplusStrong {
    /// Creates a new extended Karplus-Strong synthesizer.
    pub fn new(frequency: f64, decay: f64, blend: f64) -> Self {
        Self {
            base: KarplusStrong::new(frequency, decay, blend),
            excitation: ExcitationType::Noise,
            pick_position: 0.5,
        }
    }

    /// Sets the excitation type.
    pub fn with_excitation(mut self, excitation: ExcitationType) -> Self {
        self.excitation = excitation;
        self
    }

    /// Sets the pick position (0.0 = bridge, 1.0 = middle).
    pub fn with_pick_position(mut self, position: f64) -> Self {
        self.pick_position = position.clamp(0.0, 1.0);
        self
    }
}

impl Synthesizer for ExtendedKarplusStrong {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        // Calculate delay line length
        let delay_length = (sample_rate / self.base.frequency).round() as usize;
        if delay_length == 0 {
            return vec![0.0; num_samples];
        }

        // Initialize delay line based on excitation type
        let mut delay_line: Vec<f64> = match self.excitation {
            ExcitationType::Noise => (0..delay_length)
                .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
                .collect(),
            ExcitationType::Sawtooth => (0..delay_length)
                .map(|i| {
                    let t = i as f64 / delay_length as f64;
                    2.0 * t - 1.0
                })
                .collect(),
            ExcitationType::Triangle => (0..delay_length)
                .map(|i| {
                    let t = i as f64 / delay_length as f64;
                    if t < 0.5 {
                        4.0 * t - 1.0
                    } else {
                        3.0 - 4.0 * t
                    }
                })
                .collect(),
            ExcitationType::Impulse => {
                let mut line = vec![0.0; delay_length];
                let impulse_pos = (self.pick_position * delay_length as f64) as usize;
                if impulse_pos < delay_length {
                    line[impulse_pos] = 1.0;
                }
                line
            }
        };

        // Apply pick position filtering (simulate picking at different positions)
        if self.pick_position != 0.5 && self.excitation != ExcitationType::Impulse {
            let pick_node = (self.pick_position * delay_length as f64) as usize;
            if pick_node > 0 && pick_node < delay_length {
                // Zero out harmonics that have nodes at pick position
                for (i, sample) in delay_line.iter_mut().enumerate() {
                    let dist = ((i as f64 - pick_node as f64) / delay_length as f64).abs();
                    if dist < 0.1 {
                        *sample *= dist * 10.0;
                    }
                }
            }
        }

        let mut output = Vec::with_capacity(num_samples);
        let mut write_pos = 0;

        for _ in 0..num_samples {
            let read_pos = write_pos;
            let next_pos = (write_pos + 1) % delay_length;

            let current = delay_line[read_pos];
            let next = delay_line[next_pos];
            let filtered = self.base.blend * current + (1.0 - self.base.blend) * next;
            let new_sample = filtered * self.base.decay;

            delay_line[write_pos] = new_sample;
            output.push(current);

            write_pos = next_pos;
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_basic_karplus_strong() {
        let synth = KarplusStrong::new(440.0, 0.996, 0.5);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);
        // Should decay over time
        let early_energy: f64 = samples[0..1000].iter().map(|s| s.powi(2)).sum();
        let late_energy: f64 = samples[40000..41000].iter().map(|s| s.powi(2)).sum();
        assert!(early_energy > late_energy);
    }

    #[test]
    fn test_karplus_strong_presets() {
        let mut rng = create_rng(42);

        let guitar = KarplusStrong::guitar(220.0);
        let guitar_samples = guitar.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(guitar_samples.len(), 1000);

        let bass = KarplusStrong::bass(110.0);
        let bass_samples = bass.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(bass_samples.len(), 1000);

        let harp = KarplusStrong::harp(440.0);
        let harp_samples = harp.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(harp_samples.len(), 1000);
    }

    #[test]
    fn test_extended_karplus_strong() {
        let mut rng = create_rng(42);

        let synth = ExtendedKarplusStrong::new(440.0, 0.996, 0.7)
            .with_excitation(ExcitationType::Triangle)
            .with_pick_position(0.3);

        let samples = synth.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_karplus_strong_determinism() {
        let synth = KarplusStrong::new(440.0, 0.996, 0.5);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }
}
