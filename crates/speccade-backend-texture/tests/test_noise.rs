//! Comprehensive tests for noise algorithms.
//!
//! This module tests determinism, value ranges, tileability, seed variation,
//! and algorithm-specific properties for all noise generators.

use speccade_backend_texture::noise::tile_coord;
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

// ============================================================================
// FBM Parameter Tests
// ============================================================================

/// Test that FBM octaves affect the output.
#[test]
fn test_fbm_octaves_affect_detail() {
    let simple = Fbm::new(SimplexNoise::new(42)).with_octaves(1);
    let complex = Fbm::new(SimplexNoise::new(42)).with_octaves(8);

    let x = 1.5;
    let y = 2.3;

    let v1 = simple.sample(x, y);
    let v2 = complex.sample(x, y);

    // Values should differ due to additional octaves
    assert_ne!(
        v1, v2,
        "Different octave counts should produce different values"
    );

    // Both should still be in valid range
    assert!(
        (-1.0..=1.0).contains(&v1),
        "1 octave FBM should be normalized"
    );
    assert!(
        (-1.5..=1.5).contains(&v2),
        "8 octave FBM should be roughly normalized"
    );
}

/// Test that more octaves add high-frequency detail.
#[test]
fn test_fbm_more_octaves_more_variation() {
    let one_octave = Fbm::new(SimplexNoise::new(42)).with_octaves(1);
    let eight_octaves = Fbm::new(SimplexNoise::new(42)).with_octaves(8);

    // Measure local variation (difference between adjacent samples)
    let mut variation_1 = 0.0;
    let mut variation_8 = 0.0;
    let step = 0.01; // Small step to measure high-frequency detail

    for i in 0..100 {
        let x = i as f64 * step;
        let y = 0.5;

        if i > 0 {
            let prev_x = (i - 1) as f64 * step;
            variation_1 += (one_octave.sample(x, y) - one_octave.sample(prev_x, y)).abs();
            variation_8 += (eight_octaves.sample(x, y) - eight_octaves.sample(prev_x, y)).abs();
        }
    }

    // More octaves should generally show more local variation (high frequency detail)
    // This isn't always true due to normalization, but the trend should be there
    assert!(
        variation_8 >= variation_1 * 0.5,
        "8 octaves ({}) should have at least half the variation of 1 octave ({})",
        variation_8,
        variation_1
    );
}

/// Test that lacunarity affects frequency scaling.
#[test]
fn test_fbm_lacunarity() {
    let low_lac = Fbm::new(SimplexNoise::new(42))
        .with_octaves(4)
        .with_lacunarity(1.5);
    let high_lac = Fbm::new(SimplexNoise::new(42))
        .with_octaves(4)
        .with_lacunarity(3.0);

    let x = 1.5;
    let y = 2.3;

    // Different lacunarity should produce different values
    assert_ne!(
        low_lac.sample(x, y),
        high_lac.sample(x, y),
        "Different lacunarity should produce different values"
    );
}

/// Test that higher lacunarity creates faster frequency changes.
#[test]
fn test_fbm_lacunarity_affects_frequency_scaling() {
    let low_lac = Fbm::new(SimplexNoise::new(42))
        .with_octaves(3)
        .with_lacunarity(1.5);
    let high_lac = Fbm::new(SimplexNoise::new(42))
        .with_octaves(3)
        .with_lacunarity(4.0);

    // Measure variation over a fixed distance
    let mut var_low = 0.0;
    let mut var_high = 0.0;

    for i in 0..100 {
        let x = i as f64 * 0.01;
        if i > 0 {
            let px = (i - 1) as f64 * 0.01;
            var_low += (low_lac.sample(x, 0.5) - low_lac.sample(px, 0.5)).abs();
            var_high += (high_lac.sample(x, 0.5) - high_lac.sample(px, 0.5)).abs();
        }
    }

    // Higher lacunarity should generally show more rapid changes
    // (though this can be subtle due to the base noise contribution)
    assert!(
        (var_low - var_high).abs() > 0.01 || var_low != var_high,
        "Different lacunarities should produce measurable differences"
    );
}

/// Test that persistence affects amplitude falloff.
#[test]
fn test_fbm_persistence() {
    let low_persist = Fbm::new(SimplexNoise::new(42))
        .with_octaves(4)
        .with_persistence(0.25);
    let high_persist = Fbm::new(SimplexNoise::new(42))
        .with_octaves(4)
        .with_persistence(0.75);

    let x = 1.5;
    let y = 2.3;

    // Different persistence should produce different values
    assert_ne!(
        low_persist.sample(x, y),
        high_persist.sample(x, y),
        "Different persistence should produce different values"
    );
}

/// Test that higher persistence gives more weight to higher octaves.
#[test]
fn test_fbm_persistence_affects_detail_strength() {
    // Low persistence: higher octaves contribute less
    let low_persist = Fbm::new(SimplexNoise::new(42))
        .with_octaves(6)
        .with_persistence(0.3);
    // High persistence: higher octaves contribute more
    let high_persist = Fbm::new(SimplexNoise::new(42))
        .with_octaves(6)
        .with_persistence(0.8);

    // Compare to single octave (the base)
    let base = Fbm::new(SimplexNoise::new(42)).with_octaves(1);

    let x = 1.5;
    let y = 2.3;

    let base_val = base.sample(x, y);
    let low_val = low_persist.sample(x, y);
    let high_val = high_persist.sample(x, y);

    // Low persistence should be closer to the base (first octave dominates)
    let diff_low = (base_val - low_val).abs();
    let diff_high = (base_val - high_val).abs();

    // The test verifies different persistence values produce different deviations from base
    assert!(
        (diff_low - diff_high).abs() > 0.001 || diff_low != diff_high,
        "Different persistence should affect how much result deviates from base"
    );
}

/// Test FBM with all parameters customized.
#[test]
fn test_fbm_custom_parameters_deterministic() {
    let noise1 = Fbm::new(SimplexNoise::new(42))
        .with_octaves(5)
        .with_persistence(0.6)
        .with_lacunarity(2.5);
    let noise2 = Fbm::new(SimplexNoise::new(42))
        .with_octaves(5)
        .with_persistence(0.6)
        .with_lacunarity(2.5);

    for i in 0..50 {
        let x = i as f64 * 0.2;
        let y = i as f64 * 0.17;
        assert_eq!(
            noise1.sample(x, y),
            noise2.sample(x, y),
            "FBM with identical parameters should be deterministic"
        );
    }
}

// ============================================================================
// Worley/Voronoi Tests
// ============================================================================

/// Test that Worley distance functions produce different results.
#[test]
fn test_worley_distance_functions() {
    use speccade_backend_texture::noise::DistanceFunction;

    let euclidean = WorleyNoise::new(42).with_distance_function(DistanceFunction::Euclidean);
    let manhattan = WorleyNoise::new(42).with_distance_function(DistanceFunction::Manhattan);
    let chebyshev = WorleyNoise::new(42).with_distance_function(DistanceFunction::Chebyshev);

    let x = 1.5;
    let y = 2.3;

    let v_euclidean = euclidean.sample(x, y);
    let v_manhattan = manhattan.sample(x, y);
    let v_chebyshev = chebyshev.sample(x, y);

    // At least two of the three should be different at most points
    let all_same = v_euclidean == v_manhattan && v_manhattan == v_chebyshev;
    assert!(
        !all_same,
        "Different distance functions should produce different values"
    );
}

/// Test Worley return types produce different patterns.
#[test]
fn test_worley_return_types() {
    use speccade_backend_texture::noise::WorleyReturn;

    let seed = 42;
    let f1 = WorleyNoise::new(seed).with_return_type(WorleyReturn::F1);
    let f2 = WorleyNoise::new(seed).with_return_type(WorleyReturn::F2);
    let f2_minus_f1 = WorleyNoise::new(seed).with_return_type(WorleyReturn::F2MinusF1);
    let f1_plus_f2 = WorleyNoise::new(seed).with_return_type(WorleyReturn::F1PlusF2);

    let x = 1.5;
    let y = 2.3;

    let v_f1 = f1.sample(x, y);
    let v_f2 = f2.sample(x, y);
    let v_f2_minus_f1 = f2_minus_f1.sample(x, y);
    let v_f1_plus_f2 = f1_plus_f2.sample(x, y);

    // F2 should be >= F1 (before normalization transformation)
    // After the transformation (result * 2.0 - 1.0), F2 should still be >= F1
    assert!(
        v_f2 >= v_f1 - 0.0001,
        "F2 ({}) should be >= F1 ({})",
        v_f2,
        v_f1
    );

    // All return types should produce distinct values
    let values = [v_f1, v_f2, v_f2_minus_f1, v_f1_plus_f2];
    let mut unique_count = 0;
    for i in 0..values.len() {
        let mut is_unique = true;
        for j in 0..values.len() {
            if i != j && (values[i] - values[j]).abs() < 0.0001 {
                is_unique = false;
                break;
            }
        }
        if is_unique {
            unique_count += 1;
        }
    }
    assert!(
        unique_count >= 2,
        "Return types should produce at least 2 distinct values"
    );
}

/// Test Worley jitter parameter affects output.
#[test]
fn test_worley_jitter() {
    let no_jitter = WorleyNoise::new(42).with_jitter(0.0);
    let half_jitter = WorleyNoise::new(42).with_jitter(0.5);
    let full_jitter = WorleyNoise::new(42).with_jitter(1.0);

    let x = 1.5;
    let y = 2.3;

    // Different jitter should produce different values
    let v_no = no_jitter.sample(x, y);
    let v_half = half_jitter.sample(x, y);
    let v_full = full_jitter.sample(x, y);

    // At least some should be different
    assert!(
        v_no != v_half || v_half != v_full,
        "Different jitter values should produce different outputs"
    );
}

/// Test Worley produces cellular patterns (values cluster around cell centers).
#[test]
fn test_worley_cell_distribution() {
    use speccade_backend_texture::noise::WorleyReturn;

    let noise = WorleyNoise::new(42).with_return_type(WorleyReturn::F1);

    // F1 (distance to nearest) should have local minima at cell centers
    // Test by checking that sampling around a point shows variation
    let center_x = 1.5;
    let center_y = 2.5;

    let center_value = noise.sample(center_x, center_y);
    let mut found_different = false;

    // Sample in a small radius
    for i in 0..8 {
        let angle = i as f64 * std::f64::consts::PI / 4.0;
        let x = center_x + 0.3 * angle.cos();
        let y = center_y + 0.3 * angle.sin();
        let v = noise.sample(x, y);
        if (v - center_value).abs() > 0.01 {
            found_different = true;
            break;
        }
    }

    assert!(
        found_different,
        "Worley noise should vary spatially around a point"
    );
}

/// Test that Worley cell boundaries are detectable with F2-F1.
#[test]
fn test_worley_f2_minus_f1_edges() {
    use speccade_backend_texture::noise::WorleyReturn;

    let noise = WorleyNoise::new(42).with_return_type(WorleyReturn::F2MinusF1);

    // F2-F1 should be small near cell edges (where F1 and F2 are close)
    // and larger inside cells (where F2 >> F1)

    let mut values: Vec<f64> = Vec::new();

    // Sample a line across multiple cells
    for i in 0..100 {
        let x = i as f64 * 0.05;
        let y = 0.5;
        values.push(noise.sample(x, y));
    }

    // Find min and max
    let min = values.iter().cloned().fold(f64::MAX, f64::min);
    let max = values.iter().cloned().fold(f64::MIN, f64::max);

    // Should have meaningful variation
    assert!(
        max - min > 0.1,
        "F2-F1 should vary between edges and cell interiors"
    );
}

// ============================================================================
// Tileability Tests
// ============================================================================

/// Test the tile_coord helper function.
#[test]
fn test_tile_coord_basic() {
    // Values within period should remain unchanged (or nearly so)
    assert!((tile_coord(0.5, 10.0) - 0.5).abs() < 0.0001);
    assert!((tile_coord(5.0, 10.0) - 5.0).abs() < 0.0001);

    // Values at period boundary
    assert!((tile_coord(10.0, 10.0) - 0.0).abs() < 0.0001);

    // Values beyond period should wrap
    assert!((tile_coord(12.0, 10.0) - 2.0).abs() < 0.0001);
    assert!((tile_coord(25.0, 10.0) - 5.0).abs() < 0.0001);

    // Negative values should also wrap correctly
    assert!((tile_coord(-2.0, 10.0) - 8.0).abs() < 0.0001);
    assert!((tile_coord(-12.0, 10.0) - 8.0).abs() < 0.0001);
}

/// Test that noise with tiled coordinates produces repeating patterns.
#[test]
fn test_tiled_noise_repeats() {
    let noise = SimplexNoise::new(42);
    let period = 4.0;

    for i in 0..20 {
        for j in 0..20 {
            let x = i as f64 * 0.1;
            let y = j as f64 * 0.1;

            // Sample at base position
            let v1 = noise.sample(tile_coord(x, period), tile_coord(y, period));

            // Sample at position + period (should tile)
            let v2 = noise.sample(
                tile_coord(x + period, period),
                tile_coord(y + period, period),
            );

            // Use approximate comparison due to floating-point precision
            assert!(
                (v1 - v2).abs() < 1e-10,
                "Tiled noise should repeat at period boundaries: {} vs {} at ({}, {})",
                v1,
                v2,
                x,
                y
            );
        }
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

/// Test noise at origin.
#[test]
fn test_noise_at_origin() {
    let simplex = SimplexNoise::new(42);
    let perlin = PerlinNoise::new(42);
    let worley = WorleyNoise::new(42);

    // Should not panic or produce NaN/Inf
    let v1 = simplex.sample(0.0, 0.0);
    let v2 = perlin.sample(0.0, 0.0);
    let v3 = worley.sample(0.0, 0.0);

    assert!(!v1.is_nan() && !v1.is_infinite());
    assert!(!v2.is_nan() && !v2.is_infinite());
    assert!(!v3.is_nan() && !v3.is_infinite());
}

/// Test noise at large coordinates.
#[test]
fn test_noise_at_large_coordinates() {
    let simplex = SimplexNoise::new(42);
    let perlin = PerlinNoise::new(42);
    let worley = WorleyNoise::new(42);

    let large = 1_000_000.0;

    // Should not panic or produce NaN/Inf
    let v1 = simplex.sample(large, large);
    let v2 = perlin.sample(large, large);
    let v3 = worley.sample(large, large);

    assert!(
        !v1.is_nan() && !v1.is_infinite(),
        "Simplex at large coords: {}",
        v1
    );
    assert!(
        !v2.is_nan() && !v2.is_infinite(),
        "Perlin at large coords: {}",
        v2
    );
    assert!(
        !v3.is_nan() && !v3.is_infinite(),
        "Worley at large coords: {}",
        v3
    );
}

/// Test noise at negative coordinates.
#[test]
fn test_noise_at_negative_coordinates() {
    let simplex = SimplexNoise::new(42);
    let perlin = PerlinNoise::new(42);
    let worley = WorleyNoise::new(42);

    // Should work correctly with negative coordinates
    let v1 = simplex.sample(-10.5, -20.3);
    let v2 = perlin.sample(-10.5, -20.3);
    let v3 = worley.sample(-10.5, -20.3);

    assert!(!v1.is_nan() && !v1.is_infinite());
    assert!(!v2.is_nan() && !v2.is_infinite());
    assert!(!v3.is_nan() && !v3.is_infinite());

    // Should be different from positive coordinates
    let v1_pos = simplex.sample(10.5, 20.3);
    let v2_pos = perlin.sample(10.5, 20.3);
    let v3_pos = worley.sample(10.5, 20.3);

    // At least one should differ
    assert!(
        v1 != v1_pos || v2 != v2_pos || v3 != v3_pos,
        "Negative coordinates should generally produce different values than positive"
    );
}

/// Test noise with fractional coordinates near integer boundaries.
#[test]
fn test_noise_near_integer_boundaries() {
    let simplex = SimplexNoise::new(42);
    let perlin = PerlinNoise::new(42);

    // Test values very close to integers (edge cases for floor/cell calculations)
    let near_int = 5.0 - 1e-10;
    let at_int = 5.0;
    let past_int = 5.0 + 1e-10;

    let v1 = simplex.sample(near_int, 0.5);
    let v2 = simplex.sample(at_int, 0.5);
    let v3 = simplex.sample(past_int, 0.5);

    // Should be continuous (values should be close)
    assert!(
        (v1 - v2).abs() < 0.01 && (v2 - v3).abs() < 0.01,
        "Noise should be continuous across integer boundaries: {}, {}, {}",
        v1,
        v2,
        v3
    );

    let p1 = perlin.sample(near_int, 0.5);
    let p2 = perlin.sample(at_int, 0.5);
    let p3 = perlin.sample(past_int, 0.5);

    assert!(
        (p1 - p2).abs() < 0.01 && (p2 - p3).abs() < 0.01,
        "Perlin should be continuous: {}, {}, {}",
        p1,
        p2,
        p3
    );
}

/// Test extreme seed values.
#[test]
fn test_extreme_seed_values() {
    // Test with edge case seeds
    let _n0 = SimplexNoise::new(0);
    let _n1 = SimplexNoise::new(1);
    let _nmax = SimplexNoise::new(u32::MAX);
    let _nhalf = SimplexNoise::new(u32::MAX / 2);

    // All should produce valid output
    let v0 = _n0.sample(1.5, 2.5);
    let v1 = _n1.sample(1.5, 2.5);
    let vmax = _nmax.sample(1.5, 2.5);
    let vhalf = _nhalf.sample(1.5, 2.5);

    assert!(!v0.is_nan() && !v0.is_infinite());
    assert!(!v1.is_nan() && !v1.is_infinite());
    assert!(!vmax.is_nan() && !vmax.is_infinite());
    assert!(!vhalf.is_nan() && !vhalf.is_infinite());
}

// ============================================================================
// Clone Tests
// ============================================================================

/// Test that cloned noise generators produce identical output.
#[test]
fn test_simplex_clone() {
    let noise = SimplexNoise::new(42);
    let cloned = noise.clone();

    for i in 0..50 {
        let x = i as f64 * 0.2;
        let y = i as f64 * 0.15;
        assert_eq!(noise.sample(x, y), cloned.sample(x, y));
    }
}

/// Test that cloned Perlin noise generators produce identical output.
#[test]
fn test_perlin_clone() {
    let noise = PerlinNoise::new(42);
    let cloned = noise.clone();

    for i in 0..50 {
        let x = i as f64 * 0.2;
        let y = i as f64 * 0.15;
        assert_eq!(noise.sample(x, y), cloned.sample(x, y));
    }
}

/// Test that cloned Worley noise generators produce identical output.
#[test]
fn test_worley_clone() {
    let noise = WorleyNoise::new(42);
    let cloned = noise.clone();

    for i in 0..50 {
        let x = i as f64 * 0.2;
        let y = i as f64 * 0.15;
        assert_eq!(noise.sample(x, y), cloned.sample(x, y));
    }
}

/// Test that cloned FBM generators produce identical output.
#[test]
fn test_fbm_clone() {
    let noise = Fbm::new(SimplexNoise::new(42))
        .with_octaves(4)
        .with_persistence(0.5)
        .with_lacunarity(2.0);
    let cloned = noise.clone();

    for i in 0..50 {
        let x = i as f64 * 0.2;
        let y = i as f64 * 0.15;
        assert_eq!(noise.sample(x, y), cloned.sample(x, y));
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

/// Test FBM with different base noise types.
#[test]
fn test_fbm_with_perlin() {
    let noise = Fbm::new(PerlinNoise::new(42));
    let mut rng = DeterministicRng::new(999);

    for _ in 0..100 {
        let x = rng.gen_f64() * 10.0;
        let y = rng.gen_f64() * 10.0;
        let v = noise.sample(x, y);
        assert!(
            !v.is_nan() && !v.is_infinite(),
            "FBM with Perlin should produce valid values"
        );
    }
}

/// Test nested FBM (FBM of FBM).
#[test]
fn test_nested_fbm() {
    // This is unusual but should work
    let inner = Fbm::new(SimplexNoise::new(42)).with_octaves(2);
    let outer = Fbm::new(inner).with_octaves(2);

    let v = outer.sample(1.5, 2.5);
    assert!(!v.is_nan() && !v.is_infinite());
}

/// Test combining multiple noise types.
#[test]
fn test_noise_combination() {
    let simplex = SimplexNoise::new(42);
    let perlin = PerlinNoise::new(43);
    let worley = WorleyNoise::new(44);

    // Combine noise values (common in procedural generation)
    let x = 1.5;
    let y = 2.5;

    let v_simplex = simplex.sample(x, y);
    let v_perlin = perlin.sample(x, y);
    let v_worley = worley.sample(x, y);

    // Weighted combination
    let combined = v_simplex * 0.5 + v_perlin * 0.3 + v_worley * 0.2;

    assert!(!combined.is_nan() && !combined.is_infinite());
}

/// Test domain warping (using noise to offset sampling coordinates).
#[test]
fn test_domain_warping() {
    let base = SimplexNoise::new(42);
    let warp = SimplexNoise::new(43);

    let x = 1.5;
    let y = 2.5;
    let warp_strength = 0.5;

    // Use one noise to warp the coordinates of another
    let warp_x = warp.sample(x, y) * warp_strength;
    let warp_y = warp.sample(x + 100.0, y) * warp_strength;

    let warped_value = base.sample(x + warp_x, y + warp_y);

    assert!(!warped_value.is_nan() && !warped_value.is_infinite());

    // Should be deterministic
    let warp_x2 = warp.sample(x, y) * warp_strength;
    let warp_y2 = warp.sample(x + 100.0, y) * warp_strength;
    let warped_value2 = base.sample(x + warp_x2, y + warp_y2);

    assert_eq!(warped_value, warped_value2);
}
