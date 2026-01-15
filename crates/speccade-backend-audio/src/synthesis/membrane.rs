//! Membrane drum synthesis for toms, hand drums, congas, bongos, etc.
//!
//! Uses modal synthesis based on circular membrane mode frequencies derived from
//! Bessel function zeros. The key insight is that a circular membrane (like a
//! drumhead) vibrates at specific frequencies determined by the zeros of Bessel
//! functions, creating the characteristic pitched/tonal sound of drums.
//!
//! The mode frequency ratios for a circular membrane are based on the zeros of
//! Bessel functions J_m(x) = 0. The first several ratios relative to the
//! fundamental (mode 0,1) are approximately:
//! - Mode (0,1): 1.000 (fundamental)
//! - Mode (1,1): 1.593
//! - Mode (2,1): 2.135
//! - Mode (0,2): 2.295
//! - Mode (3,1): 2.653
//! - Mode (1,2): 2.917
//! - Mode (4,1): 3.155
//! - Mode (2,2): 3.500
//! - Mode (0,3): 3.598

use std::f64::consts::PI;

use rand::Rng;
use rand_pcg::Pcg32;

use super::Synthesizer;

/// Circular membrane mode frequency ratios (Bessel function zeros).
/// These are the first 9 modes of a circular membrane, normalized to the fundamental.
const MEMBRANE_MODE_RATIOS: [f64; 9] = [
    1.000, // (0,1) fundamental
    1.593, // (1,1)
    2.135, // (2,1)
    2.295, // (0,2)
    2.653, // (3,1)
    2.917, // (1,2)
    3.155, // (4,1)
    3.500, // (2,2)
    3.598, // (0,3)
];

/// Membrane drum synthesizer using modal synthesis.
///
/// Creates pitched drum sounds like toms, congas, bongos, and hand drums
/// by modeling the resonant modes of a circular membrane.
#[derive(Debug, Clone)]
pub struct MembraneDrumSynth {
    /// Fundamental frequency in Hz.
    pub frequency: f64,
    /// Decay rate (0.0-1.0). Higher = faster decay.
    pub decay: f64,
    /// Tone/brightness (0.0-1.0). Controls overtone emphasis.
    pub tone: f64,
    /// Strike strength (0.0-1.0). Controls attack transient intensity.
    pub strike: f64,
}

impl MembraneDrumSynth {
    /// Creates a new membrane drum synthesizer.
    ///
    /// # Arguments
    /// * `frequency` - Fundamental frequency in Hz
    /// * `decay` - Decay rate (0.0-1.0), higher = faster decay
    /// * `tone` - Tone/brightness (0.0-1.0), higher = more overtones
    /// * `strike` - Strike strength (0.0-1.0), higher = stronger attack transient
    pub fn new(frequency: f64, decay: f64, tone: f64, strike: f64) -> Self {
        Self {
            frequency: frequency.max(20.0),
            decay: decay.clamp(0.0, 1.0),
            tone: tone.clamp(0.0, 1.0),
            strike: strike.clamp(0.0, 1.0),
        }
    }

    /// Creates a floor tom preset.
    pub fn floor_tom(frequency: f64) -> Self {
        Self::new(frequency, 0.4, 0.3, 0.7)
    }

    /// Creates a rack tom preset.
    pub fn rack_tom(frequency: f64) -> Self {
        Self::new(frequency, 0.5, 0.4, 0.8)
    }

    /// Creates a conga preset.
    pub fn conga(frequency: f64) -> Self {
        Self::new(frequency, 0.6, 0.5, 0.6)
    }

    /// Creates a bongo preset.
    pub fn bongo(frequency: f64) -> Self {
        Self::new(frequency, 0.7, 0.6, 0.5)
    }

    /// Creates a djembe preset.
    pub fn djembe(frequency: f64) -> Self {
        Self::new(frequency, 0.5, 0.7, 0.8)
    }

    /// Creates a timpani preset.
    pub fn timpani(frequency: f64) -> Self {
        Self::new(frequency, 0.3, 0.2, 0.6)
    }
}

impl Synthesizer for MembraneDrumSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        if num_samples == 0 {
            return vec![];
        }

        let mut output = vec![0.0; num_samples];
        let dt = 1.0 / sample_rate;
        let two_pi = 2.0 * PI;

        // Calculate mode amplitudes based on tone parameter
        // Low tone = emphasize fundamental, high tone = emphasize overtones
        let mode_amplitudes: Vec<f64> = MEMBRANE_MODE_RATIOS
            .iter()
            .enumerate()
            .map(|(i, _)| {
                // Base amplitude falls off with mode number
                let base_amp = 1.0 / (1.0 + i as f64 * 0.5);
                // Tone parameter shifts emphasis
                let tone_factor = if i == 0 {
                    // Fundamental is always present, reduced by tone
                    1.0 - self.tone * 0.5
                } else {
                    // Higher modes are enhanced by tone
                    0.3 + self.tone * 0.7
                };
                base_amp * tone_factor
            })
            .collect();

        // Calculate decay times for each mode
        // Higher modes decay faster (realistic membrane behavior)
        // decay parameter scales overall decay rate
        let base_decay_time = 0.1 + (1.0 - self.decay) * 2.0; // 0.1 to 2.1 seconds
        let mode_decay_rates: Vec<f64> = MEMBRANE_MODE_RATIOS
            .iter()
            .enumerate()
            .map(|(i, ratio)| {
                // Decay rate proportional to frequency ratio (higher modes decay faster)
                let mode_decay_time =
                    base_decay_time / (1.0 + i as f64 * 0.3 + (*ratio - 1.0) * 0.2);
                // Convert to decay rate (time to reach ~5% amplitude)
                3.0 / mode_decay_time
            })
            .collect();

        // Generate excitation signal based on strike parameter
        let strike_samples = ((sample_rate * 0.005) as usize).min(num_samples); // 5ms max
        let excitation: Vec<f64> = (0..num_samples)
            .map(|i| {
                if i < strike_samples {
                    let t = i as f64 / strike_samples as f64;
                    // Impulse-like excitation with quick decay
                    let impulse = if i == 0 { 1.0 } else { 0.0 };
                    // Noise component scaled by strike strength
                    let noise = (rng.gen::<f64>() * 2.0 - 1.0) * (1.0 - t);
                    let noise_amount = self.strike * 0.4;
                    impulse * (1.0 - noise_amount) + noise * noise_amount
                } else {
                    0.0
                }
            })
            .collect();

        // Random initial phases for natural sound (deterministic from RNG)
        let initial_phases: Vec<f64> = (0..MEMBRANE_MODE_RATIOS.len())
            .map(|_| rng.gen::<f64>() * two_pi)
            .collect();

        // Synthesize each mode
        for (mode_idx, &freq_ratio) in MEMBRANE_MODE_RATIOS.iter().enumerate() {
            let mode_freq = self.frequency * freq_ratio;
            let amplitude = mode_amplitudes[mode_idx];
            let decay_rate = mode_decay_rates[mode_idx];
            let mut phase = initial_phases[mode_idx];

            for (i, sample) in output.iter_mut().enumerate() {
                let t = i as f64 * dt;

                // Decaying sine for this mode
                let envelope = (-decay_rate * t).exp();
                let mode_output = phase.sin() * amplitude * envelope;

                // Convolve with excitation (simplified: modulate by excitation envelope)
                let exc_factor = excitation[i];
                let exc_influence = if exc_factor.abs() > 0.0001 {
                    1.0 + exc_factor * self.strike
                } else {
                    1.0
                };

                *sample += mode_output * exc_influence;

                // Advance phase
                phase += two_pi * mode_freq * dt;
                if phase >= two_pi {
                    phase -= two_pi;
                }
            }
        }

        // Normalize output to [-1.0, 1.0]
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
    fn test_membrane_drum_basic() {
        let synth = MembraneDrumSynth::new(100.0, 0.5, 0.5, 0.5);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s), "Sample out of range: {}", s);
        }
    }

    #[test]
    fn test_membrane_drum_presets() {
        let presets = [
            MembraneDrumSynth::floor_tom(80.0),
            MembraneDrumSynth::rack_tom(120.0),
            MembraneDrumSynth::conga(200.0),
            MembraneDrumSynth::bongo(300.0),
            MembraneDrumSynth::djembe(150.0),
            MembraneDrumSynth::timpani(60.0),
        ];

        for synth in presets {
            let mut rng = create_rng(42);
            let samples = synth.synthesize(1000, 44100.0, &mut rng);

            assert_eq!(samples.len(), 1000);
            for &s in &samples {
                assert!((-1.0..=1.0).contains(&s));
            }
        }
    }

    #[test]
    fn test_membrane_drum_determinism() {
        let synth = MembraneDrumSynth::new(100.0, 0.5, 0.5, 0.5);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(500, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(500, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_membrane_drum_different_seeds() {
        let synth = MembraneDrumSynth::new(100.0, 0.5, 0.5, 0.5);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(43);

        let samples1 = synth.synthesize(500, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(500, 44100.0, &mut rng2);

        // Should be different due to random phases
        assert_ne!(samples1, samples2);
    }

    #[test]
    fn test_membrane_drum_zero_samples() {
        let synth = MembraneDrumSynth::new(100.0, 0.5, 0.5, 0.5);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(0, 44100.0, &mut rng);

        assert!(samples.is_empty());
    }

    #[test]
    fn test_membrane_drum_clamping() {
        // Test that parameters are clamped correctly
        let synth = MembraneDrumSynth::new(-10.0, 2.0, -0.5, 1.5);
        assert!(synth.frequency >= 20.0);
        assert!((0.0..=1.0).contains(&synth.decay));
        assert!((0.0..=1.0).contains(&synth.tone));
        assert!((0.0..=1.0).contains(&synth.strike));
    }

    #[test]
    fn test_membrane_drum_tone_variation() {
        // Low tone should emphasize fundamental
        let low_tone = MembraneDrumSynth::new(100.0, 0.5, 0.0, 0.5);
        // High tone should emphasize overtones
        let high_tone = MembraneDrumSynth::new(100.0, 0.5, 1.0, 0.5);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = low_tone.synthesize(1000, 44100.0, &mut rng1);
        let samples2 = high_tone.synthesize(1000, 44100.0, &mut rng2);

        // They should be different due to different tone settings
        assert_ne!(samples1, samples2);
    }

    #[test]
    fn test_membrane_drum_decay_variation() {
        // Fast decay
        let fast_decay = MembraneDrumSynth::new(100.0, 1.0, 0.5, 0.5);
        // Slow decay
        let slow_decay = MembraneDrumSynth::new(100.0, 0.0, 0.5, 0.5);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = fast_decay.synthesize(2000, 44100.0, &mut rng1);
        let samples2 = slow_decay.synthesize(2000, 44100.0, &mut rng2);

        // Fast decay should have lower RMS energy in the tail
        let tail_start = 1000;
        let rms1: f64 = (samples1[tail_start..].iter().map(|s| s * s).sum::<f64>()
            / (samples1.len() - tail_start) as f64)
            .sqrt();
        let rms2: f64 = (samples2[tail_start..].iter().map(|s| s * s).sum::<f64>()
            / (samples2.len() - tail_start) as f64)
            .sqrt();

        assert!(rms1 < rms2, "Fast decay should have less tail energy");
    }
}
