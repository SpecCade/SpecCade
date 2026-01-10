//! ADSR envelope generator.
//!
//! This module provides an Attack-Decay-Sustain-Release envelope generator
//! for shaping the amplitude of audio signals over time.

/// ADSR envelope parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdsrParams {
    /// Attack time in seconds.
    pub attack: f64,
    /// Decay time in seconds.
    pub decay: f64,
    /// Sustain level (0.0 to 1.0).
    pub sustain: f64,
    /// Release time in seconds.
    pub release: f64,
}

impl Default for AdsrParams {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
        }
    }
}

impl AdsrParams {
    /// Creates new ADSR parameters.
    pub fn new(attack: f64, decay: f64, sustain: f64, release: f64) -> Self {
        Self {
            attack: attack.max(0.0),
            decay: decay.max(0.0),
            sustain: sustain.clamp(0.0, 1.0),
            release: release.max(0.0),
        }
    }

    /// Creates a percussive envelope (no sustain).
    pub fn percussive(attack: f64, decay: f64) -> Self {
        Self {
            attack,
            decay,
            sustain: 0.0,
            release: decay,
        }
    }

    /// Creates a pluck envelope (very fast attack, medium decay).
    pub fn pluck(decay: f64) -> Self {
        Self {
            attack: 0.001,
            decay,
            sustain: 0.0,
            release: decay,
        }
    }

    /// Creates a pad envelope (slow attack and release).
    pub fn pad(attack: f64, release: f64) -> Self {
        Self {
            attack,
            decay: 0.0,
            sustain: 1.0,
            release,
        }
    }
}

/// Envelope generator state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    /// Attack phase - amplitude rising from 0 to 1.
    Attack,
    /// Decay phase - amplitude falling from 1 to sustain level.
    Decay,
    /// Sustain phase - amplitude held at sustain level.
    Sustain,
    /// Release phase - amplitude falling from current level to 0.
    Release,
    /// Envelope completed - amplitude is 0.
    Idle,
}

/// ADSR envelope generator.
#[derive(Debug, Clone)]
pub struct AdsrEnvelope {
    params: AdsrParams,
    sample_rate: f64,
    state: EnvelopeState,
    time: f64,
    level: f64,
    release_level: f64,
}

impl AdsrEnvelope {
    /// Creates a new ADSR envelope.
    pub fn new(params: AdsrParams, sample_rate: f64) -> Self {
        Self {
            params,
            sample_rate,
            state: EnvelopeState::Attack,
            time: 0.0,
            level: 0.0,
            release_level: 0.0,
        }
    }

    /// Triggers the envelope to start (note on).
    pub fn trigger(&mut self) {
        self.state = EnvelopeState::Attack;
        self.time = 0.0;
        self.level = 0.0;
    }

    /// Releases the envelope (note off).
    pub fn release(&mut self) {
        if self.state != EnvelopeState::Idle && self.state != EnvelopeState::Release {
            self.release_level = self.level;
            self.state = EnvelopeState::Release;
            self.time = 0.0;
        }
    }

    /// Gets the current envelope state.
    pub fn state(&self) -> EnvelopeState {
        self.state
    }

    /// Returns true if the envelope has completed.
    pub fn is_idle(&self) -> bool {
        self.state == EnvelopeState::Idle
    }

    /// Generates the next envelope sample.
    pub fn next_sample(&mut self) -> f64 {
        let dt = 1.0 / self.sample_rate;

        match self.state {
            EnvelopeState::Attack => {
                if self.params.attack > 0.0 {
                    self.level = self.time / self.params.attack;
                    if self.level >= 1.0 {
                        self.level = 1.0;
                        self.state = EnvelopeState::Decay;
                        self.time = 0.0;
                    } else {
                        self.time += dt;
                    }
                } else {
                    self.level = 1.0;
                    self.state = EnvelopeState::Decay;
                    self.time = 0.0;
                }
            }
            EnvelopeState::Decay => {
                if self.params.decay > 0.0 {
                    let progress = self.time / self.params.decay;
                    self.level = 1.0 - progress * (1.0 - self.params.sustain);
                    if progress >= 1.0 {
                        self.level = self.params.sustain;
                        self.state = EnvelopeState::Sustain;
                        self.time = 0.0;
                    } else {
                        self.time += dt;
                    }
                } else {
                    self.level = self.params.sustain;
                    self.state = EnvelopeState::Sustain;
                    self.time = 0.0;
                }
            }
            EnvelopeState::Sustain => {
                self.level = self.params.sustain;
                // Stay in sustain until release() is called
            }
            EnvelopeState::Release => {
                if self.params.release > 0.0 {
                    let progress = self.time / self.params.release;
                    self.level = self.release_level * (1.0 - progress);
                    if progress >= 1.0 {
                        self.level = 0.0;
                        self.state = EnvelopeState::Idle;
                    } else {
                        self.time += dt;
                    }
                } else {
                    self.level = 0.0;
                    self.state = EnvelopeState::Idle;
                }
            }
            EnvelopeState::Idle => {
                self.level = 0.0;
            }
        }

        self.level
    }

    /// Generates an envelope curve for a fixed duration.
    ///
    /// This is useful for one-shot sound effects where the envelope should
    /// complete within a specified duration.
    ///
    /// # Arguments
    /// * `params` - ADSR parameters
    /// * `sample_rate` - Audio sample rate
    /// * `duration` - Total duration in seconds
    ///
    /// # Returns
    /// Vector of envelope values (0.0 to 1.0)
    pub fn generate_fixed_duration(
        params: &AdsrParams,
        sample_rate: f64,
        duration: f64,
    ) -> Vec<f64> {
        let num_samples = (duration * sample_rate).ceil() as usize;
        let mut envelope = Vec::with_capacity(num_samples);

        // Calculate when to start release
        let attack_decay_duration = params.attack + params.decay;
        let release_start = duration - params.release;

        // Ensure release starts after attack+decay
        let release_start = release_start.max(attack_decay_duration);

        let release_start_sample = (release_start * sample_rate) as usize;

        let mut env = AdsrEnvelope::new(*params, sample_rate);
        env.trigger();

        for i in 0..num_samples {
            if i == release_start_sample {
                env.release();
            }
            envelope.push(env.next_sample());
        }

        envelope
    }
}

/// Simple exponential decay envelope.
///
/// Useful for percussive sounds where only decay matters.
pub struct DecayEnvelope {
    level: f64,
    decay_rate: f64,
}

impl DecayEnvelope {
    /// Creates a new decay envelope.
    ///
    /// # Arguments
    /// * `decay_time` - Time for the envelope to decay to ~37% (1/e) in seconds
    /// * `sample_rate` - Audio sample rate
    pub fn new(decay_time: f64, sample_rate: f64) -> Self {
        let decay_rate = if decay_time > 0.0 {
            (-1.0 / (decay_time * sample_rate)).exp()
        } else {
            0.0
        };

        Self {
            level: 1.0,
            decay_rate,
        }
    }

    /// Generates the next envelope sample.
    pub fn next_sample(&mut self) -> f64 {
        let current = self.level;
        self.level *= self.decay_rate;
        current
    }

    /// Resets the envelope to full level.
    pub fn reset(&mut self) {
        self.level = 1.0;
    }
}

/// Generates an exponential decay curve.
///
/// # Arguments
/// * `decay_time` - Time constant in seconds
/// * `sample_rate` - Audio sample rate
/// * `duration` - Total duration in seconds
///
/// # Returns
/// Vector of envelope values starting at 1.0 and decaying exponentially
pub fn exponential_decay(decay_time: f64, sample_rate: f64, duration: f64) -> Vec<f64> {
    let num_samples = (duration * sample_rate).ceil() as usize;
    let mut envelope = Vec::with_capacity(num_samples);

    let decay_rate = if decay_time > 0.0 {
        -1.0 / (decay_time * sample_rate)
    } else {
        f64::NEG_INFINITY
    };

    for i in 0..num_samples {
        envelope.push((i as f64 * decay_rate).exp());
    }

    envelope
}

/// Generates a linear attack-release curve.
///
/// # Arguments
/// * `attack_time` - Attack time in seconds
/// * `release_time` - Release time in seconds
/// * `sample_rate` - Audio sample rate
///
/// # Returns
/// Vector of envelope values
pub fn linear_ar(attack_time: f64, release_time: f64, sample_rate: f64) -> Vec<f64> {
    let attack_samples = (attack_time * sample_rate).ceil() as usize;
    let release_samples = (release_time * sample_rate).ceil() as usize;
    let total_samples = attack_samples + release_samples;

    let mut envelope = Vec::with_capacity(total_samples);

    // Attack
    for i in 0..attack_samples {
        let t = if attack_samples > 0 {
            i as f64 / attack_samples as f64
        } else {
            1.0
        };
        envelope.push(t);
    }

    // Release
    for i in 0..release_samples {
        let t = if release_samples > 0 {
            1.0 - (i as f64 / release_samples as f64)
        } else {
            0.0
        };
        envelope.push(t);
    }

    envelope
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adsr_default() {
        let params = AdsrParams::default();
        assert_eq!(params.attack, 0.01);
        assert_eq!(params.decay, 0.1);
        assert_eq!(params.sustain, 0.5);
        assert_eq!(params.release, 0.2);
    }

    #[test]
    fn test_adsr_attack_phase() {
        let params = AdsrParams::new(0.1, 0.0, 1.0, 0.0);
        let mut env = AdsrEnvelope::new(params, 1000.0);
        env.trigger();

        // After 50ms (50 samples at 1kHz), should be at 50%
        for _ in 0..50 {
            env.next_sample();
        }
        assert!((env.level - 0.5).abs() < 0.02);
    }

    #[test]
    fn test_adsr_decay_to_sustain() {
        let params = AdsrParams::new(0.0, 0.1, 0.5, 0.0);
        let mut env = AdsrEnvelope::new(params, 1000.0);
        env.trigger();

        // After attack (instant) and full decay
        for _ in 0..150 {
            env.next_sample();
        }
        assert!((env.level - 0.5).abs() < 0.02);
        assert_eq!(env.state(), EnvelopeState::Sustain);
    }

    #[test]
    fn test_adsr_release() {
        let params = AdsrParams::new(0.0, 0.0, 1.0, 0.1);
        let mut env = AdsrEnvelope::new(params, 1000.0);
        env.trigger();
        env.next_sample(); // Get to sustain

        env.release();
        // After 50ms of release, should be at 50%
        for _ in 0..50 {
            env.next_sample();
        }
        assert!((env.level - 0.5).abs() < 0.02);

        // After full release
        for _ in 0..100 {
            env.next_sample();
        }
        assert!(env.is_idle());
    }

    #[test]
    fn test_fixed_duration_envelope() {
        let params = AdsrParams::new(0.01, 0.05, 0.5, 0.1);
        let envelope = AdsrEnvelope::generate_fixed_duration(&params, 1000.0, 0.5);

        assert_eq!(envelope.len(), 500);
        // Should start at 0
        assert!(envelope[0] < 0.1);
        // Should end near 0
        assert!(envelope[499] < 0.1);
    }

    #[test]
    fn test_exponential_decay() {
        let envelope = exponential_decay(0.1, 1000.0, 0.5);

        // Should start at 1.0
        assert!((envelope[0] - 1.0).abs() < 0.001);
        // After one time constant (100 samples), should be at ~37%
        assert!((envelope[100] - 0.368).abs() < 0.01);
    }

    #[test]
    fn test_linear_ar() {
        let envelope = linear_ar(0.1, 0.1, 1000.0);

        // 100 attack + 100 release = 200 samples
        assert_eq!(envelope.len(), 200);
        // Should start at 0
        assert!(envelope[0].abs() < 0.01);
        // Peak should be at 1.0
        assert!((envelope[100] - 1.0).abs() < 0.01);
        // Should end at 0
        assert!(envelope[199].abs() < 0.02);
    }
}
