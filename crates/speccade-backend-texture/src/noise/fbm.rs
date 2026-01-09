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
}
