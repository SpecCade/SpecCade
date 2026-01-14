//! Tests for normal map generation.

use speccade_spec::recipe::texture::{
    NoiseAlgorithm, NoiseConfig, NormalMapPattern, NormalMapProcessing, TextureNormalV1Params,
};

use super::*;

#[test]
fn test_generate_flat_normal() {
    let params = TextureNormalV1Params {
        resolution: [64, 64],
        tileable: false,
        pattern: None,
        bump_strength: 1.0,
        processing: None,
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert_eq!(result.width, 64);
    assert_eq!(result.height, 64);
    assert!(!result.hash.is_empty());
}

#[test]
fn test_generate_grid_normal() {
    let params = TextureNormalV1Params {
        resolution: [128, 128],
        tileable: true,
        pattern: Some(NormalMapPattern::Grid {
            cell_size: 32,
            line_width: 4,
            bevel: 0.5,
        }),
        bump_strength: 1.0,
        processing: None,
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert_eq!(result.width, 128);
    assert_eq!(result.height, 128);
}

#[test]
fn test_generate_brick_normal() {
    let params = TextureNormalV1Params {
        resolution: [256, 256],
        tileable: true,
        pattern: Some(NormalMapPattern::Bricks {
            brick_width: 64,
            brick_height: 32,
            mortar_width: 4,
            offset: 0.5,
        }),
        bump_strength: 1.5,
        processing: None,
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert_eq!(result.width, 256);
    assert_eq!(result.height, 256);
}

#[test]
fn test_deterministic() {
    let params = TextureNormalV1Params {
        resolution: [64, 64],
        tileable: false,
        pattern: Some(NormalMapPattern::NoiseBumps {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Perlin,
                scale: 0.05,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
        }),
        bump_strength: 1.0,
        processing: None,
    };

    let result1 = generate_normal_map(&params, 42).unwrap();
    let result2 = generate_normal_map(&params, 42).unwrap();

    assert_eq!(result1.hash, result2.hash);
    assert_eq!(result1.data, result2.data);
}

#[test]
fn test_generate_normal_map_invalid_resolution() {
    let params = TextureNormalV1Params {
        resolution: [0, 64],
        tileable: false,
        pattern: None,
        bump_strength: 1.0,
        processing: None,
    };

    let err = generate_normal_map(&params, 42).unwrap_err();
    assert!(err.to_string().contains("resolution"));
}

#[test]
fn test_generate_normal_map_invalid_bump_strength() {
    let params = TextureNormalV1Params {
        resolution: [64, 64],
        tileable: false,
        pattern: None,
        bump_strength: -1.0,
        processing: None,
    };

    let err = generate_normal_map(&params, 42).unwrap_err();
    assert!(err.to_string().contains("bump_strength"));
}

// ========================================================================
// All Pattern Types Tests
// ========================================================================

#[test]
fn test_pattern_tiles() {
    let params = TextureNormalV1Params {
        resolution: [128, 128],
        tileable: true,
        pattern: Some(NormalMapPattern::Tiles {
            tile_size: 32,
            gap_width: 3,
            gap_depth: 0.4,
            seed: 42,
        }),
        bump_strength: 1.0,
        processing: None,
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert_eq!(result.width, 128);
    assert_eq!(result.height, 128);
    assert!(!result.hash.is_empty());
}

#[test]
fn test_pattern_hexagons() {
    let params = TextureNormalV1Params {
        resolution: [256, 256],
        tileable: true,
        pattern: Some(NormalMapPattern::Hexagons { size: 20, gap: 2 }),
        bump_strength: 1.2,
        processing: None,
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert_eq!(result.width, 256);
}

#[test]
fn test_pattern_rivets() {
    let params = TextureNormalV1Params {
        resolution: [128, 128],
        tileable: false,
        pattern: Some(NormalMapPattern::Rivets {
            spacing: 24,
            radius: 3,
            height: 0.25,
            seed: 123,
        }),
        bump_strength: 1.5,
        processing: None,
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert_eq!(result.width, 128);
}

#[test]
fn test_pattern_weave() {
    let params = TextureNormalV1Params {
        resolution: [128, 128],
        tileable: true,
        pattern: Some(NormalMapPattern::Weave {
            thread_width: 6,
            gap: 1,
            depth: 0.2,
        }),
        bump_strength: 1.0,
        processing: None,
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert_eq!(result.width, 128);
}

#[test]
fn test_pattern_diamond_plate() {
    let params = TextureNormalV1Params {
        resolution: [256, 256],
        tileable: true,
        pattern: Some(NormalMapPattern::DiamondPlate {
            diamond_size: 40,
            height: 0.35,
        }),
        bump_strength: 1.0,
        processing: None,
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert_eq!(result.width, 256);
}

#[test]
fn test_all_noise_algorithms() {
    for algo in [
        NoiseAlgorithm::Perlin,
        NoiseAlgorithm::Simplex,
        NoiseAlgorithm::Worley,
        NoiseAlgorithm::Value,
        NoiseAlgorithm::Fbm,
    ] {
        let params = TextureNormalV1Params {
            resolution: [64, 64],
            tileable: false,
            pattern: Some(NormalMapPattern::NoiseBumps {
                noise: NoiseConfig {
                    algorithm: algo,
                    scale: 0.05,
                    octaves: 4,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 64);
        assert_eq!(result.height, 64);
    }
}

// ========================================================================
// Processing Options Tests
// ========================================================================

#[test]
fn test_processing_blur() {
    let params = TextureNormalV1Params {
        resolution: [64, 64],
        tileable: false,
        pattern: Some(NormalMapPattern::Grid {
            cell_size: 16,
            line_width: 2,
            bevel: 0.3,
        }),
        bump_strength: 1.0,
        processing: Some(NormalMapProcessing {
            blur: Some(1.5),
            invert: false,
        }),
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert!(!result.data.is_empty());
}

#[test]
fn test_processing_invert() {
    let params = TextureNormalV1Params {
        resolution: [64, 64],
        tileable: false,
        pattern: Some(NormalMapPattern::Bricks {
            brick_width: 32,
            brick_height: 16,
            mortar_width: 3,
            offset: 0.5,
        }),
        bump_strength: 1.0,
        processing: Some(NormalMapProcessing {
            blur: None,
            invert: true,
        }),
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert!(!result.data.is_empty());
}

#[test]
fn test_processing_blur_and_invert() {
    let params = TextureNormalV1Params {
        resolution: [64, 64],
        tileable: false,
        pattern: Some(NormalMapPattern::Tiles {
            tile_size: 20,
            gap_width: 2,
            gap_depth: 0.3,
            seed: 42,
        }),
        bump_strength: 1.0,
        processing: Some(NormalMapProcessing {
            blur: Some(2.0),
            invert: true,
        }),
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert!(!result.data.is_empty());
}

// ========================================================================
// Bump Strength Tests
// ========================================================================

#[test]
fn test_various_bump_strengths() {
    for strength in [0.5, 1.0, 1.5, 2.0, 3.0] {
        let params = TextureNormalV1Params {
            resolution: [32, 32],
            tileable: false,
            pattern: Some(NormalMapPattern::Grid {
                cell_size: 8,
                line_width: 1,
                bevel: 0.5,
            }),
            bump_strength: strength,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 32);
        assert_eq!(result.height, 32);
    }
}

// ========================================================================
// Tileable Tests
// ========================================================================

#[test]
fn test_tileable_noise() {
    let params = TextureNormalV1Params {
        resolution: [128, 128],
        tileable: true,
        pattern: Some(NormalMapPattern::NoiseBumps {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Perlin,
                scale: 0.1,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
        }),
        bump_strength: 1.0,
        processing: None,
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert_eq!(result.width, 128);
}

#[test]
fn test_non_tileable_noise() {
    let params = TextureNormalV1Params {
        resolution: [128, 128],
        tileable: false,
        pattern: Some(NormalMapPattern::NoiseBumps {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Simplex,
                scale: 0.05,
                octaves: 6,
                persistence: 0.6,
                lacunarity: 2.2,
            },
        }),
        bump_strength: 1.0,
        processing: None,
    };

    let result = generate_normal_map(&params, 42).unwrap();
    assert_eq!(result.width, 128);
}

// ========================================================================
// Seed Variation Tests
// ========================================================================

#[test]
fn test_seed_affects_bricks() {
    let make_params = || TextureNormalV1Params {
        resolution: [64, 64],
        tileable: false,
        pattern: Some(NormalMapPattern::Bricks {
            brick_width: 32,
            brick_height: 16,
            mortar_width: 3,
            offset: 0.5,
        }),
        bump_strength: 1.0,
        processing: None,
    };

    let result1 = generate_normal_map(&make_params(), 1).unwrap();
    let result2 = generate_normal_map(&make_params(), 2).unwrap();

    assert_ne!(result1.hash, result2.hash);
}

#[test]
fn test_seed_affects_tiles() {
    let make_params = |seed| TextureNormalV1Params {
        resolution: [64, 64],
        tileable: false,
        pattern: Some(NormalMapPattern::Tiles {
            tile_size: 20,
            gap_width: 2,
            gap_depth: 0.3,
            seed,
        }),
        bump_strength: 1.0,
        processing: None,
    };

    let result1 = generate_normal_map(&make_params(42), 1).unwrap();
    let result2 = generate_normal_map(&make_params(100), 2).unwrap();

    assert_ne!(result1.hash, result2.hash);
}

// ========================================================================
// File Save Tests
// ========================================================================

#[test]
fn test_save_normal_map() {
    let params = TextureNormalV1Params {
        resolution: [64, 64],
        tileable: false,
        pattern: Some(NormalMapPattern::Grid {
            cell_size: 16,
            line_width: 2,
            bevel: 0.5,
        }),
        bump_strength: 1.0,
        processing: None,
    };

    let result = generate_normal_map(&params, 42).unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let output_path = tmp.path().join("test_normal.png");

    let saved = save_normal_map(&result, &output_path).unwrap();
    assert!(output_path.exists());
    assert!(saved.file_path.is_some());
}

// ========================================================================
// Pattern Parameter Variation Tests
// ========================================================================

#[test]
fn test_bricks_with_different_sizes() {
    for brick_width in [32, 64, 128] {
        let params = TextureNormalV1Params {
            resolution: [256, 256],
            tileable: true,
            pattern: Some(NormalMapPattern::Bricks {
                brick_width,
                brick_height: brick_width / 2,
                mortar_width: 4,
                offset: 0.5,
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 256);
    }
}

#[test]
fn test_tiles_with_different_gap_depths() {
    for gap_depth in [0.1, 0.3, 0.5, 0.7] {
        let params = TextureNormalV1Params {
            resolution: [128, 128],
            tileable: true,
            pattern: Some(NormalMapPattern::Tiles {
                tile_size: 32,
                gap_width: 3,
                gap_depth,
                seed: 42,
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 128);
    }
}

#[test]
fn test_weave_with_different_thread_widths() {
    for thread_width in [4, 8, 12, 16] {
        let params = TextureNormalV1Params {
            resolution: [128, 128],
            tileable: true,
            pattern: Some(NormalMapPattern::Weave {
                thread_width,
                gap: 2,
                depth: 0.15,
            }),
            bump_strength: 1.0,
            processing: None,
        };

        let result = generate_normal_map(&params, 42).unwrap();
        assert_eq!(result.width, 128);
    }
}
