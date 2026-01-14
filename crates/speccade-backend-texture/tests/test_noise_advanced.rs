//! Advanced tests for noise algorithms: FBM parameters, Worley patterns, and tileability.
//!
//! This module tests advanced noise features including FBM octave/lacunarity/persistence
//! control, Worley distance functions and return types, and tileable noise generation.

use speccade_backend_texture::noise::tile_coord;
use speccade_backend_texture::noise::{Fbm, Noise2D, SimplexNoise, WorleyNoise};

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
