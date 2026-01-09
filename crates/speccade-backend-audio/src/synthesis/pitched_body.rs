//! Pitched body synthesis for impact sounds.
//!
//! This module creates impact and hit sounds by sweeping frequency from high to low,
//! simulating the behavior of resonant bodies being struck.

use std::f64::consts::PI;

use rand_pcg::Pcg32;

use super::{FrequencySweep, SweepCurve, Synthesizer};

/// Pitched body synthesizer for impact sounds.
#[derive(Debug, Clone)]
pub struct PitchedBody {
    /// Starting frequency in Hz (usually higher).
    pub start_freq: f64,
    /// Ending frequency in Hz (usually lower).
    pub end_freq: f64,
    /// Sweep curve type.
    pub curve: SweepCurve,
    /// Number of harmonics to include.
    pub harmonics: usize,
    /// Harmonic decay rate (higher = faster harmonic decay).
    pub harmonic_decay: f64,
}

impl PitchedBody {
    /// Creates a new pitched body synthesizer.
    ///
    /// # Arguments
    /// * `start_freq` - Starting frequency (typically 100-500 Hz)
    /// * `end_freq` - Ending frequency (typically 20-100 Hz)
    pub fn new(start_freq: f64, end_freq: f64) -> Self {
        Self {
            start_freq,
            end_freq,
            curve: SweepCurve::Exponential,
            harmonics: 4,
            harmonic_decay: 0.5,
        }
    }

    /// Creates a kick drum-like preset.
    pub fn kick() -> Self {
        Self {
            start_freq: 200.0,
            end_freq: 40.0,
            curve: SweepCurve::Exponential,
            harmonics: 2,
            harmonic_decay: 0.7,
        }
    }

    /// Creates a tom drum-like preset.
    pub fn tom(pitch: f64) -> Self {
        Self {
            start_freq: pitch * 2.0,
            end_freq: pitch,
            curve: SweepCurve::Exponential,
            harmonics: 3,
            harmonic_decay: 0.5,
        }
    }

    /// Creates a wood block-like preset.
    pub fn wood_block(pitch: f64) -> Self {
        Self {
            start_freq: pitch * 1.2,
            end_freq: pitch,
            curve: SweepCurve::Linear,
            harmonics: 6,
            harmonic_decay: 0.3,
        }
    }

    /// Creates a thud/punch impact preset.
    pub fn thud() -> Self {
        Self {
            start_freq: 300.0,
            end_freq: 30.0,
            curve: SweepCurve::Exponential,
            harmonics: 1,
            harmonic_decay: 0.9,
        }
    }

    /// Sets the sweep curve type.
    pub fn with_curve(mut self, curve: SweepCurve) -> Self {
        self.curve = curve;
        self
    }

    /// Sets the number of harmonics.
    pub fn with_harmonics(mut self, harmonics: usize) -> Self {
        self.harmonics = harmonics.max(1);
        self
    }

    /// Sets the harmonic decay rate.
    pub fn with_harmonic_decay(mut self, decay: f64) -> Self {
        self.harmonic_decay = decay.clamp(0.0, 1.0);
        self
    }
}

impl Synthesizer for PitchedBody {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        let mut output = vec![0.0; num_samples];
        let sweep = FrequencySweep::new(self.start_freq, self.end_freq, self.curve);

        let dt = 1.0 / sample_rate;
        let two_pi = 2.0 * PI;

        // Generate each harmonic
        for h in 1..=self.harmonics {
            let harmonic_amp = self.harmonic_decay.powi(h as i32 - 1);
            let mut phase: f64 = 0.0;

            for (i, sample) in output.iter_mut().enumerate() {
                let progress = i as f64 / num_samples as f64;
                let base_freq = sweep.at(progress);
                let freq = base_freq * h as f64;

                let wave = phase.sin() * harmonic_amp;
                *sample += wave;

                phase += two_pi * freq * dt;
                if phase >= two_pi {
                    phase -= two_pi;
                }
            }
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

/// Impact sound synthesizer combining pitched body with noise attack.
#[derive(Debug, Clone)]
pub struct ImpactSound {
    /// Pitched body component.
    pub body: PitchedBody,
    /// Noise attack duration in seconds.
    pub noise_attack: f64,
    /// Noise to body ratio (0.0 = all body, 1.0 = all noise).
    pub noise_mix: f64,
    /// Body decay time in seconds.
    pub body_decay: f64,
}

impl ImpactSound {
    /// Creates a new impact sound.
    pub fn new(body: PitchedBody) -> Self {
        Self {
            body,
            noise_attack: 0.01,
            noise_mix: 0.3,
            body_decay: 0.2,
        }
    }

    /// Creates a kick drum impact.
    pub fn kick() -> Self {
        Self {
            body: PitchedBody::kick(),
            noise_attack: 0.005,
            noise_mix: 0.2,
            body_decay: 0.3,
        }
    }

    /// Creates a snare-like impact.
    pub fn snare() -> Self {
        Self {
            body: PitchedBody::new(250.0, 150.0),
            noise_attack: 0.02,
            noise_mix: 0.6,
            body_decay: 0.15,
        }
    }

    /// Creates a punch/hit impact.
    pub fn punch() -> Self {
        Self {
            body: PitchedBody::thud(),
            noise_attack: 0.003,
            noise_mix: 0.15,
            body_decay: 0.1,
        }
    }

    /// Sets the noise attack duration.
    pub fn with_noise_attack(mut self, duration: f64) -> Self {
        self.noise_attack = duration.max(0.0);
        self
    }

    /// Sets the noise mix ratio.
    pub fn with_noise_mix(mut self, mix: f64) -> Self {
        self.noise_mix = mix.clamp(0.0, 1.0);
        self
    }

    /// Sets the body decay time.
    pub fn with_body_decay(mut self, decay: f64) -> Self {
        self.body_decay = decay.max(0.001);
        self
    }
}

impl Synthesizer for ImpactSound {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        use crate::oscillator::white_noise;

        // Generate body
        let mut body = self.body.synthesize(num_samples, sample_rate, rng);

        // Apply decay envelope to body
        let decay_samples = (self.body_decay * sample_rate) as usize;
        for (i, sample) in body.iter_mut().enumerate() {
            let env = if i < decay_samples {
                (-3.0 * i as f64 / decay_samples as f64).exp()
            } else {
                0.0
            };
            *sample *= env;
        }

        // Generate noise attack
        let noise_samples = (self.noise_attack * sample_rate) as usize;
        if noise_samples > 0 && self.noise_mix > 0.0 {
            let noise = white_noise(rng, noise_samples.min(num_samples));

            // Mix noise into body
            for (i, &n) in noise.iter().enumerate() {
                let env = 1.0 - (i as f64 / noise_samples as f64);
                body[i] += n * self.noise_mix * env;
            }
        }

        // Normalize
        let max = body.iter().map(|s| s.abs()).fold(0.0_f64, |a, b| a.max(b));
        if max > 0.0 {
            for s in &mut body {
                *s /= max;
            }
        }

        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_pitched_body() {
        let synth = PitchedBody::new(200.0, 50.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        for &s in &samples {
            assert!(s >= -1.0 && s <= 1.0);
        }
    }

    #[test]
    fn test_pitched_body_presets() {
        let mut rng = create_rng(42);

        let kick = PitchedBody::kick();
        let kick_samples = kick.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(kick_samples.len(), 1000);

        let tom = PitchedBody::tom(100.0);
        let tom_samples = tom.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(tom_samples.len(), 1000);

        let wood = PitchedBody::wood_block(800.0);
        let wood_samples = wood.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(wood_samples.len(), 1000);
    }

    #[test]
    fn test_impact_sound() {
        let synth = ImpactSound::kick();
        let mut rng = create_rng(42);
        let samples = synth.synthesize(22050, 44100.0, &mut rng);

        assert_eq!(samples.len(), 22050);
        for &s in &samples {
            assert!(s >= -1.0 && s <= 1.0);
        }
    }

    #[test]
    fn test_pitched_body_determinism() {
        let synth = PitchedBody::new(200.0, 50.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }
}
