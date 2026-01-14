//! Basic tests for noise algorithms: determinism, value ranges, and seed variation.
//!
//! This module tests fundamental properties of noise generators including
//! deterministic behavior, output value ranges, and seed-based variation.

use speccade_backend_texture::noise::{Fbm, Noise2D, PerlinNoise, SimplexNoise, WorleyNoise};
use speccade_backend_texture::rng::DeterministicRng;

// ============================================================================
// Determinism Tests
// ============================================================================

/// Verify that SimplexNoise produces identical output for identical inputs.
#[test]
fn test_simplex_determinism() {
    let noise = SimplexNoise::new(12345);
    let v1 = noise.sample(10.5, 20.3);
    let v2 = noise.sample(10.5, 20.3);
    assert_eq!(
        v1, v2,
        "SimplexNoise should return identical values for identical inputs"
    );
}

/// Verify that recreating SimplexNoise with same seed produces same output.
#[test]
fn test_simplex_determinism_across_instances() {
    let noise1 = SimplexNoise::new(12345);
    let noise2 = SimplexNoise::new(12345);

    for i in 0..100 {
        let x = i as f64 * 0.1;
        let y = i as f64 * 0.13;
        assert_eq!(
            noise1.sample(x, y),
            noise2.sample(x, y),
            "Two SimplexNoise instances with same seed should produce identical results"
        );
    }
}

/// Verify that PerlinNoise produces identical output for identical inputs.
#[test]
fn test_perlin_determinism() {
    let noise = PerlinNoise::new(12345);
    let v1 = noise.sample(10.5, 20.3);
    let v2 = noise.sample(10.5, 20.3);
    assert_eq!(
        v1, v2,
        "PerlinNoise should return identical values for identical inputs"
    );
}

/// Verify that recreating PerlinNoise with same seed produces same output.
#[test]
fn test_perlin_determinism_across_instances() {
    let noise1 = PerlinNoise::new(12345);
    let noise2 = PerlinNoise::new(12345);

    for i in 0..100 {
        let x = i as f64 * 0.1;
        let y = i as f64 * 0.13;
        assert_eq!(
            noise1.sample(x, y),
            noise2.sample(x, y),
            "Two PerlinNoise instances with same seed should produce identical results"
        );
    }
}

/// Verify that WorleyNoise produces identical output for identical inputs.
#[test]
fn test_worley_determinism() {
    let noise = WorleyNoise::new(12345);
    let v1 = noise.sample(10.5, 20.3);
    let v2 = noise.sample(10.5, 20.3);
    assert_eq!(
        v1, v2,
        "WorleyNoise should return identical values for identical inputs"
    );
}

/// Verify that recreating WorleyNoise with same seed produces same output.
#[test]
fn test_worley_determinism_across_instances() {
    let noise1 = WorleyNoise::new(12345);
    let noise2 = WorleyNoise::new(12345);

    for i in 0..100 {
        let x = i as f64 * 0.1;
        let y = i as f64 * 0.13;
        assert_eq!(
            noise1.sample(x, y),
            noise2.sample(x, y),
            "Two WorleyNoise instances with same seed should produce identical results"
        );
    }
}

/// Verify that FBM produces identical output for identical inputs.
#[test]
fn test_fbm_determinism() {
    let noise = Fbm::new(SimplexNoise::new(12345));
    let v1 = noise.sample(10.5, 20.3);
    let v2 = noise.sample(10.5, 20.3);
    assert_eq!(
        v1, v2,
        "FBM should return identical values for identical inputs"
    );
}

/// Verify that recreating FBM with same seed produces same output.
#[test]
fn test_fbm_determinism_across_instances() {
    let noise1 = Fbm::new(SimplexNoise::new(12345));
    let noise2 = Fbm::new(SimplexNoise::new(12345));

    for i in 0..100 {
        let x = i as f64 * 0.1;
        let y = i as f64 * 0.13;
        assert_eq!(
            noise1.sample(x, y),
            noise2.sample(x, y),
            "Two FBM instances with same underlying seed should produce identical results"
        );
    }
}

/// Verify FBM determinism with Perlin base noise.
#[test]
fn test_fbm_perlin_determinism() {
    let noise1 = Fbm::new(PerlinNoise::new(12345));
    let noise2 = Fbm::new(PerlinNoise::new(12345));

    for i in 0..100 {
        let x = i as f64 * 0.1;
        let y = i as f64 * 0.13;
        assert_eq!(
            noise1.sample(x, y),
            noise2.sample(x, y),
            "FBM with Perlin should be deterministic"
        );
    }
}

// ============================================================================
// Value Range Tests
// ============================================================================

/// Test that SimplexNoise values stay within expected range.
#[test]
fn test_simplex_value_range() {
    let noise = SimplexNoise::new(42);
    let mut rng = DeterministicRng::new(999);

    for _ in 0..1000 {
        let x = rng.gen_f64() * 100.0 - 50.0;
        let y = rng.gen_f64() * 100.0 - 50.0;
        let v = noise.sample(x, y);
        assert!(
            (-1.5..=1.5).contains(&v),
            "SimplexNoise value {} at ({}, {}) out of expected range [-1.5, 1.5]",
            v,
            x,
            y
        );
    }
}

/// Test that SimplexNoise values actually use the full range.
#[test]
fn test_simplex_uses_full_range() {
    let noise = SimplexNoise::new(42);
    let mut min = f64::MAX;
    let mut max = f64::MIN;

    // Sample a grid to find range
    for i in 0..500 {
        for j in 0..500 {
            let x = i as f64 * 0.01;
            let y = j as f64 * 0.01;
            let v = noise.sample(x, y);
            min = min.min(v);
            max = max.max(v);
        }
    }

    // Should use a substantial portion of the [-1, 1] range
    assert!(
        min < -0.5,
        "SimplexNoise min ({}) should be below -0.5",
        min
    );
    assert!(max > 0.5, "SimplexNoise max ({}) should be above 0.5", max);
}

/// Test that PerlinNoise values stay within expected range.
#[test]
fn test_perlin_value_range() {
    let noise = PerlinNoise::new(42);
    let mut rng = DeterministicRng::new(999);

    for _ in 0..1000 {
        let x = rng.gen_f64() * 100.0 - 50.0;
        let y = rng.gen_f64() * 100.0 - 50.0;
        let v = noise.sample(x, y);
        assert!(
            (-1.5..=1.5).contains(&v),
            "PerlinNoise value {} at ({}, {}) out of expected range [-1.5, 1.5]",
            v,
            x,
            y
        );
    }
}

/// Test that PerlinNoise values actually use the full range.
#[test]
fn test_perlin_uses_full_range() {
    let noise = PerlinNoise::new(42);
    let mut min = f64::MAX;
    let mut max = f64::MIN;

    for i in 0..500 {
        for j in 0..500 {
            let x = i as f64 * 0.01;
            let y = j as f64 * 0.01;
            let v = noise.sample(x, y);
            min = min.min(v);
            max = max.max(v);
        }
    }

    assert!(min < -0.3, "PerlinNoise min ({}) should be below -0.3", min);
    assert!(max > 0.3, "PerlinNoise max ({}) should be above 0.3", max);
}

/// Test that FBM values stay within expected normalized range.
#[test]
fn test_fbm_value_range() {
    let noise = Fbm::new(SimplexNoise::new(42));
    let mut rng = DeterministicRng::new(999);

    for _ in 0..1000 {
        let x = rng.gen_f64() * 100.0 - 50.0;
        let y = rng.gen_f64() * 100.0 - 50.0;
        let v = noise.sample(x, y);
        assert!(
            (-1.5..=1.5).contains(&v),
            "FBM value {} at ({}, {}) out of expected range [-1.5, 1.5]",
            v,
            x,
            y
        );
    }
}

/// Test sample_01 normalizes to [0, 1] range.
#[test]
fn test_sample_01_range() {
    let simplex = SimplexNoise::new(42);
    let perlin = PerlinNoise::new(42);
    let worley = WorleyNoise::new(42);
    let fbm = Fbm::new(SimplexNoise::new(42));

    let mut rng = DeterministicRng::new(999);

    for _ in 0..500 {
        let x = rng.gen_f64() * 50.0;
        let y = rng.gen_f64() * 50.0;

        let v_simplex = simplex.sample_01(x, y);
        let v_perlin = perlin.sample_01(x, y);
        let v_worley = worley.sample_01(x, y);
        let v_fbm = fbm.sample_01(x, y);

        // sample_01 shifts from [-1,1] to [0,1], so slight overshoots become [~-0.25, ~1.25]
        assert!(
            (-0.5..=1.5).contains(&v_simplex),
            "Simplex sample_01 {} out of range",
            v_simplex
        );
        assert!(
            (-0.5..=1.5).contains(&v_perlin),
            "Perlin sample_01 {} out of range",
            v_perlin
        );
        // Worley has different characteristics, allow wider range
        assert!(
            (-2.0..=3.0).contains(&v_worley),
            "Worley sample_01 {} out of range",
            v_worley
        );
        assert!(
            (-0.5..=1.5).contains(&v_fbm),
            "FBM sample_01 {} out of range",
            v_fbm
        );
    }
}

// ============================================================================
// Seed Variation Tests
// ============================================================================

/// Test that different seeds produce different output for SimplexNoise.
#[test]
fn test_simplex_different_seeds_different_output() {
    let n1 = SimplexNoise::new(1);
    let n2 = SimplexNoise::new(2);
    assert_ne!(
        n1.sample(5.0, 5.0),
        n2.sample(5.0, 5.0),
        "Different seeds should produce different outputs"
    );
}

/// Test that different seeds produce different output for PerlinNoise.
#[test]
fn test_perlin_different_seeds_different_output() {
    let n1 = PerlinNoise::new(1);
    let n2 = PerlinNoise::new(2);
    // Use a non-integer point to avoid landing on grid points where all gradients cancel
    assert_ne!(
        n1.sample(5.7, 5.3),
        n2.sample(5.7, 5.3),
        "Different seeds should produce different outputs"
    );
}

/// Test that different seeds produce different output for WorleyNoise.
#[test]
fn test_worley_different_seeds_different_output() {
    let n1 = WorleyNoise::new(1);
    let n2 = WorleyNoise::new(2);
    assert_ne!(
        n1.sample(5.0, 5.0),
        n2.sample(5.0, 5.0),
        "Different seeds should produce different outputs"
    );
}

/// Test that different seeds produce different output for FBM.
#[test]
fn test_fbm_different_seeds_different_output() {
    let n1 = Fbm::new(SimplexNoise::new(1));
    let n2 = Fbm::new(SimplexNoise::new(2));
    assert_ne!(
        n1.sample(5.0, 5.0),
        n2.sample(5.0, 5.0),
        "Different seeds should produce different outputs"
    );
}

/// Test statistical difference between seeds across many samples.
#[test]
fn test_seeds_produce_statistically_different_output() {
    let n1 = SimplexNoise::new(100);
    let n2 = SimplexNoise::new(200);

    let mut differences = 0;
    let samples = 100;

    for i in 0..samples {
        let x = i as f64 * 0.1;
        let y = i as f64 * 0.17;
        if (n1.sample(x, y) - n2.sample(x, y)).abs() > 0.001 {
            differences += 1;
        }
    }

    // At least 90% of samples should be different
    assert!(
        differences > samples * 90 / 100,
        "Seeds should produce mostly different values, got {}/{} different",
        differences,
        samples
    );
}
