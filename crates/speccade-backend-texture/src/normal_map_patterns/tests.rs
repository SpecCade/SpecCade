//! Tests for normal map pattern height generation.

use super::*;
use speccade_spec::recipe::texture::{NoiseAlgorithm, NoiseConfig};

#[test]
fn test_generate_grid_height() {
    let buffer = generate_grid_height(64, 64, 16, 2, 0.5);
    assert_eq!(buffer.width, 64);
    assert_eq!(buffer.height, 64);

    // Check that grid lines are recessed
    assert!(buffer.get(0, 0) < 0.5);
    assert!(buffer.get(1, 0) < 0.5);
}

#[test]
fn test_generate_brick_height() {
    let buffer = generate_brick_height(128, 128, 32, 16, 3, 0.5, 42);
    assert_eq!(buffer.width, 128);
    assert_eq!(buffer.height, 128);
}

#[test]
fn test_generate_hexagon_height() {
    let buffer = generate_hexagon_height(64, 64, 10, 2);
    assert_eq!(buffer.width, 64);
    assert_eq!(buffer.height, 64);
}

#[test]
fn test_generate_noise_height() {
    let config = NoiseConfig {
        algorithm: NoiseAlgorithm::Perlin,
        scale: 0.1,
        octaves: 4,
        persistence: 0.5,
        lacunarity: 2.0,
    };

    let buffer = generate_noise_height(64, 64, &config, 42, false);
    assert_eq!(buffer.width, 64);
    assert_eq!(buffer.height, 64);
}

#[test]
fn test_generate_noise_height_tileable() {
    let config = NoiseConfig {
        algorithm: NoiseAlgorithm::Simplex,
        scale: 0.05,
        octaves: 4,
        persistence: 0.5,
        lacunarity: 2.0,
    };

    let buffer = generate_noise_height(64, 64, &config, 42, true);
    assert_eq!(buffer.width, 64);
    assert_eq!(buffer.height, 64);
}

#[test]
fn test_generate_diamond_plate_height() {
    let buffer = generate_diamond_plate_height(64, 64, 16, 0.5);
    assert_eq!(buffer.width, 64);
    assert_eq!(buffer.height, 64);
}

#[test]
fn test_generate_tiles_height() {
    let buffer = generate_tiles_height(64, 64, 16, 2, 0.4, 42);
    assert_eq!(buffer.width, 64);
    assert_eq!(buffer.height, 64);
}

#[test]
fn test_generate_rivets_height() {
    let buffer = generate_rivets_height(64, 64, 16, 3, 0.5, 42);
    assert_eq!(buffer.width, 64);
    assert_eq!(buffer.height, 64);
}

#[test]
fn test_generate_weave_height() {
    let buffer = generate_weave_height(64, 64, 6, 1, 0.2);
    assert_eq!(buffer.width, 64);
    assert_eq!(buffer.height, 64);
}

#[test]
fn test_generate_height_from_pattern_grid() {
    let pattern = NormalMapPattern::Grid {
        cell_size: 16,
        line_width: 2,
        bevel: 0.5,
    };

    let buffer = generate_height_from_pattern(&pattern, 64, 64, 42, false);
    assert_eq!(buffer.width, 64);
    assert_eq!(buffer.height, 64);
}

#[test]
fn test_generate_height_from_pattern_all_types() {
    let patterns = vec![
        NormalMapPattern::Grid {
            cell_size: 16,
            line_width: 2,
            bevel: 0.5,
        },
        NormalMapPattern::Bricks {
            brick_width: 32,
            brick_height: 16,
            mortar_width: 3,
            offset: 0.5,
        },
        NormalMapPattern::Hexagons { size: 10, gap: 2 },
        NormalMapPattern::NoiseBumps {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Perlin,
                scale: 0.1,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
        },
        NormalMapPattern::DiamondPlate {
            diamond_size: 16,
            height: 0.5,
        },
        NormalMapPattern::Tiles {
            tile_size: 16,
            gap_width: 2,
            gap_depth: 0.4,
            seed: 42,
        },
        NormalMapPattern::Rivets {
            spacing: 16,
            radius: 3,
            height: 0.5,
            seed: 42,
        },
        NormalMapPattern::Weave {
            thread_width: 6,
            gap: 1,
            depth: 0.2,
        },
    ];

    for pattern in patterns {
        let buffer = generate_height_from_pattern(&pattern, 32, 32, 42, false);
        assert_eq!(buffer.width, 32);
        assert_eq!(buffer.height, 32);
    }
}

#[test]
fn test_determinism() {
    let pattern = NormalMapPattern::Bricks {
        brick_width: 32,
        brick_height: 16,
        mortar_width: 3,
        offset: 0.5,
    };

    let buffer1 = generate_height_from_pattern(&pattern, 64, 64, 42, false);
    let buffer2 = generate_height_from_pattern(&pattern, 64, 64, 42, false);

    assert_eq!(buffer1.data, buffer2.data);
}

#[test]
fn test_different_seeds_produce_different_output() {
    let pattern = NormalMapPattern::Bricks {
        brick_width: 32,
        brick_height: 16,
        mortar_width: 3,
        offset: 0.5,
    };

    let buffer1 = generate_height_from_pattern(&pattern, 64, 64, 42, false);
    let buffer2 = generate_height_from_pattern(&pattern, 64, 64, 100, false);

    assert_ne!(buffer1.data, buffer2.data);
}
