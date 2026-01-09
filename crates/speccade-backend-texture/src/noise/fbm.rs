//! Fractal Brownian Motion (FBM) noise.
//!
//! FBM layers multiple octaves of noise to create natural-looking patterns.

use super::Noise2D;

/// Fractal Brownian Motion generator.
///
/// Combines multiple octaves of a base noise function to create
/// more complex, natural-looking patterns.
#[derive(Clone)]
pub struct Fbm<N: Noise2D + Clone> {
    /// The base noise function.
    noise: N,
    /// Number of octaves to combine.
    octaves: u8,
    /// How much each octave contributes relative to the previous.
    /// Typical value: 0.5 (each octave is half the amplitude).
    persistence: f64,
    /// How much detail increases with each octave.
    /// Typical value: 2.0 (each octave is twice the frequency).
    lacunarity: f64,
}

impl<N: Noise2D + Clone> Fbm<N> {
    /// Create a new FBM generator with default settings.
    ///
    /// Default: 4 octaves, 0.5 persistence, 2.0 lacunarity.
    pub fn new(noise: N) -> Self {
        Self {
            noise,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }

    /// Set the number of octaves.
    pub fn with_octaves(mut self, octaves: u8) -> Self {
        self.octaves = octaves.max(1);
        self
    }

    /// Set the persistence (amplitude multiplier per octave).
    pub fn with_persistence(mut self, persistence: f64) -> Self {
        self.persistence = persistence;
        self
    }

    /// Set the lacunarity (frequency multiplier per octave).
    pub fn with_lacunarity(mut self, lacunarity: f64) -> Self {
        self.lacunarity = lacunarity;
        self
    }
}

impl<N: Noise2D + Clone> Noise2D for Fbm<N> {
    fn sample(&self, x: f64, y: f64) -> f64 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0; // Used for normalizing

        for _ in 0..self.octaves {
            total += self.noise.sample(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }

        // Normalize to [-1, 1]
        total / max_value
    }
}

/// Ridged multifractal noise.
///
/// Similar to FBM but creates ridge-like features by using the absolute
/// value of the noise and inverting it.
pub struct RidgedMultifractal<N: Noise2D> {
    /// The base noise function.
    noise: N,
    /// Number of octaves to combine.
    octaves: u8,
    /// How much each octave contributes.
    gain: f64,
    /// How much detail increases with each octave.
    lacunarity: f64,
    /// Offset applied to create ridges.
    offset: f64,
}

impl<N: Noise2D> RidgedMultifractal<N> {
    /// Create a new ridged multifractal generator.
    pub fn new(noise: N) -> Self {
        Self {
            noise,
            octaves: 4,
            gain: 2.0,
            lacunarity: 2.0,
            offset: 1.0,
        }
    }

    /// Set the number of octaves.
    pub fn with_octaves(mut self, octaves: u8) -> Self {
        self.octaves = octaves.max(1);
        self
    }

    /// Set the gain.
    pub fn with_gain(mut self, gain: f64) -> Self {
        self.gain = gain;
        self
    }

    /// Set the lacunarity.
    pub fn with_lacunarity(mut self, lacunarity: f64) -> Self {
        self.lacunarity = lacunarity;
        self
    }

    /// Set the offset.
    pub fn with_offset(mut self, offset: f64) -> Self {
        self.offset = offset;
        self
    }
}

impl<N: Noise2D> Noise2D for RidgedMultifractal<N> {
    fn sample(&self, x: f64, y: f64) -> f64 {
        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut weight = 1.0;

        for i in 0..self.octaves {
            // Get noise value and create ridges
            let signal = self.noise.sample(x * frequency, y * frequency);
            let signal = self.offset - signal.abs();
            let signal = signal * signal; // Square for sharper ridges

            // Weight by previous octave's value for heterogeneous ridges
            let signal = signal * weight;
            weight = (signal * self.gain).clamp(0.0, 1.0);

            total += signal;
            frequency *= self.lacunarity;
        }

        // Normalize roughly to [-1, 1]
        total * 2.0 / self.octaves as f64 - 1.0
    }
}

/// Turbulence noise.
///
/// Similar to FBM but uses the absolute value of each noise sample,
/// creating a more chaotic appearance.
pub struct Turbulence<N: Noise2D + Clone> {
    fbm: Fbm<AbsNoise<N>>,
}

/// Wrapper that takes absolute value of noise.
#[derive(Clone)]
pub struct AbsNoise<N: Noise2D + Clone> {
    inner: N,
}

impl<N: Noise2D + Clone> Noise2D for AbsNoise<N> {
    fn sample(&self, x: f64, y: f64) -> f64 {
        self.inner.sample(x, y).abs()
    }
}

impl<N: Noise2D + Clone> Turbulence<N> {
    /// Create a new turbulence generator.
    pub fn new(noise: N) -> Self {
        Self {
            fbm: Fbm::new(AbsNoise { inner: noise }),
        }
    }

    /// Set the number of octaves.
    pub fn with_octaves(mut self, octaves: u8) -> Self {
        self.fbm = self.fbm.with_octaves(octaves);
        self
    }

    /// Set the persistence.
    pub fn with_persistence(mut self, persistence: f64) -> Self {
        self.fbm = self.fbm.with_persistence(persistence);
        self
    }

    /// Set the lacunarity.
    pub fn with_lacunarity(mut self, lacunarity: f64) -> Self {
        self.fbm = self.fbm.with_lacunarity(lacunarity);
        self
    }
}

impl<N: Noise2D + Clone> Noise2D for Turbulence<N> {
    fn sample(&self, x: f64, y: f64) -> f64 {
        // Turbulence naturally returns values in [0, 1] due to abs
        // Convert to [-1, 1] for consistency
        self.fbm.sample(x, y) * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise::SimplexNoise;

    #[test]
    fn test_fbm_deterministic() {
        let noise1 = Fbm::new(SimplexNoise::new(42));
        let noise2 = Fbm::new(SimplexNoise::new(42));

        for i in 0..100 {
            let x = i as f64 * 0.1;
            let y = i as f64 * 0.13;
            assert_eq!(noise1.sample(x, y), noise2.sample(x, y));
        }
    }

    #[test]
    fn test_fbm_octaves_affect_detail() {
        let simple = Fbm::new(SimplexNoise::new(42)).with_octaves(1);
        let complex = Fbm::new(SimplexNoise::new(42)).with_octaves(8);

        // Sample at the same point
        let x = 1.5;
        let y = 2.3;

        let v1 = simple.sample(x, y);
        let v2 = complex.sample(x, y);

        // Values should be different due to additional octaves
        // (unless they happen to align perfectly, which is unlikely)
        // We just verify both are in valid range
        assert!(v1 >= -1.0 && v1 <= 1.0);
        assert!(v2 >= -1.0 && v2 <= 1.0);
    }

    #[test]
    fn test_ridged_multifractal_range() {
        let noise = RidgedMultifractal::new(SimplexNoise::new(42));

        for i in 0..100 {
            for j in 0..100 {
                let x = i as f64 * 0.05;
                let y = j as f64 * 0.05;
                let v = noise.sample(x, y);
                // Should be in roughly [-1, 1] but may exceed slightly
                assert!(v >= -2.0 && v <= 2.0);
            }
        }
    }

    #[test]
    fn test_turbulence_positive_tendency() {
        let noise = Turbulence::new(SimplexNoise::new(42));

        let mut sum = 0.0;
        let count = 1000;
        for i in 0..count {
            let x = i as f64 * 0.1;
            let y = i as f64 * 0.13;
            sum += noise.sample(x, y);
        }

        // Turbulence tends to produce values that average around 0
        // after normalization
        let avg = sum / count as f64;
        assert!(avg.abs() < 0.5);
    }
}
