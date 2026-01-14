//! Integration and edge case tests for noise algorithms.
//!
//! This module tests edge cases, clone behavior, and complex integration scenarios
//! including nested FBM, domain warping, and noise combination techniques.

use speccade_backend_texture::noise::{Fbm, Noise2D, PerlinNoise, SimplexNoise, WorleyNoise};
use speccade_backend_texture::rng::DeterministicRng;

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
