//! Tests for texture generation.

use super::*;
use ::png as png_crate;
use speccade_spec::recipe::texture::{
    BaseMaterial, GradientDirection, MaterialType, NoiseAlgorithm, NoiseConfig, StripeDirection,
    TextureLayer,
};
use std::collections::HashSet;
use std::io::Cursor;

fn make_params() -> TextureMaterialV1Params {
    TextureMaterialV1Params {
        resolution: [32, 32],
        tileable: true,
        maps: vec![
            TextureMapType::Albedo,
            TextureMapType::Normal,
            TextureMapType::Roughness,
        ],
        base_material: Some(BaseMaterial {
            material_type: MaterialType::Metal,
            base_color: [0.8, 0.2, 0.1],
            roughness_range: Some([0.2, 0.5]),
            metallic: Some(1.0),
            brick_pattern: None,
            normal_params: None,
        }),
        layers: vec![],
        palette: None,
        color_ramp: None,
    }
}

fn decode_png_bytes(data: &[u8]) -> (png_crate::ColorType, u32, u32, Vec<u8>) {
    let decoder = png_crate::Decoder::new(Cursor::new(data));
    let mut reader = decoder.read_info().expect("decode png header");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("decode png frame");
    (
        info.color_type,
        info.width,
        info.height,
        buf[..info.buffer_size()].to_vec(),
    )
}

// ========================================================================
// Determinism Tests
// ========================================================================

#[test]
fn test_generate_material_maps_deterministic() {
    let params = make_params();
    let result1 = generate_material_maps(&params, 42).unwrap();
    let result2 = generate_material_maps(&params, 42).unwrap();

    assert_eq!(result1.maps.len(), params.maps.len());
    assert_eq!(result2.maps.len(), params.maps.len());

    for map_type in &params.maps {
        let m1 = result1.maps.get(map_type).unwrap();
        let m2 = result2.maps.get(map_type).unwrap();
        assert_eq!(m1.hash, m2.hash);
        assert_eq!(m1.data, m2.data);
        assert_eq!(m1.width, params.resolution[0]);
        assert_eq!(m1.height, params.resolution[1]);
        assert!(!m1.data.is_empty());
    }
}

#[test]
fn test_generate_material_maps_seed_changes_output() {
    let params = make_params();
    let result1 = generate_material_maps(&params, 1).unwrap();
    let result2 = generate_material_maps(&params, 2).unwrap();

    let hash1 = &result1.maps.get(&TextureMapType::Albedo).unwrap().hash;
    let hash2 = &result2.maps.get(&TextureMapType::Albedo).unwrap().hash;
    assert_ne!(hash1, hash2);
}

// ========================================================================
// Validation Tests
// ========================================================================

#[test]
fn test_generate_material_maps_invalid_resolution() {
    let mut params = make_params();
    params.resolution = [0, 32];
    let err = generate_material_maps(&params, 42).unwrap_err();
    assert!(err.to_string().contains("resolution"));
}

#[test]
fn test_generate_material_maps_empty_maps_is_error() {
    let mut params = make_params();
    params.maps.clear();
    let err = generate_material_maps(&params, 42).unwrap_err();
    assert!(err.to_string().contains("maps"));
}

#[test]
fn test_generate_material_maps_duplicate_maps_is_error() {
    let mut params = make_params();
    params.maps = vec![TextureMapType::Albedo, TextureMapType::Albedo];
    let err = generate_material_maps(&params, 42).unwrap_err();
    assert!(err.to_string().contains("duplicate"));
}

// ========================================================================
// Layer Pattern Generation Tests
// ========================================================================

#[test]
fn test_layer_noise_pattern_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::NoisePattern {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        },
        affects: vec![TextureMapType::Roughness],
        strength: 0.5,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

#[test]
fn test_layer_gradient_horizontal_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::Gradient {
        direction: GradientDirection::Horizontal,
        start: Some(0.0),
        end: Some(1.0),
        center: None,
        inner: None,
        outer: None,
        affects: vec![TextureMapType::Albedo],
        strength: 0.75,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Albedo));
}

#[test]
fn test_layer_gradient_vertical_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::Gradient {
        direction: GradientDirection::Vertical,
        start: Some(0.2),
        end: Some(0.8),
        center: None,
        inner: None,
        outer: None,
        affects: vec![TextureMapType::Roughness],
        strength: 1.0,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

#[test]
fn test_layer_gradient_radial_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::Gradient {
        direction: GradientDirection::Radial,
        start: None,
        end: None,
        center: Some([0.5, 0.5]),
        inner: Some(1.0),
        outer: Some(0.0),
        affects: vec![TextureMapType::Albedo],
        strength: 0.8,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Albedo));
}

#[test]
fn test_layer_stripes_horizontal_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::Stripes {
        direction: StripeDirection::Horizontal,
        stripe_width: 4,
        color1: 0.0,
        color2: 1.0,
        affects: vec![TextureMapType::Albedo],
        strength: 1.0,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Albedo));
}

#[test]
fn test_layer_stripes_vertical_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::Stripes {
        direction: StripeDirection::Vertical,
        stripe_width: 8,
        color1: 0.3,
        color2: 0.7,
        affects: vec![TextureMapType::Roughness],
        strength: 0.5,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

#[test]
fn test_layer_checkerboard_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::Checkerboard {
        tile_size: 8,
        color1: 0.85,
        color2: 0.2,
        affects: vec![TextureMapType::Albedo, TextureMapType::Normal],
        strength: 1.0,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Albedo));
    assert!(result.maps.contains_key(&TextureMapType::Normal));
}

#[test]
fn test_layer_pitting_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::Pitting {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Worley,
            scale: 0.12,
            octaves: 3,
            persistence: 0.5,
            lacunarity: 2.0,
        },
        threshold: 0.6,
        depth: 0.2,
        affects: vec![TextureMapType::Roughness],
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

#[test]
fn test_layer_weave_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::Weave {
        thread_width: 6,
        gap: 2,
        depth: 0.3,
        affects: vec![TextureMapType::Normal],
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Normal));
}

#[test]
fn test_layer_scratches_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::Scratches {
        density: 0.2,
        length_range: [0.1, 0.4],
        width: 0.002,
        affects: vec![TextureMapType::Roughness],
        strength: 0.6,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

#[test]
fn test_layer_edge_wear_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::EdgeWear {
        amount: 0.3,
        affects: vec![TextureMapType::Roughness],
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

#[test]
fn test_layer_dirt_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::Dirt {
        density: 0.15,
        color: [0.3, 0.25, 0.2],
        affects: vec![TextureMapType::Albedo],
        strength: 0.4,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Albedo));
}

#[test]
fn test_layer_stains_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::Stains {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.08,
            octaves: 3,
            persistence: 0.5,
            lacunarity: 2.0,
        },
        threshold: 0.7,
        color: [0.2, 0.18, 0.15],
        affects: vec![TextureMapType::Albedo, TextureMapType::Roughness],
        strength: 0.6,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Albedo));
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

#[test]
fn test_layer_water_streaks_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::WaterStreaks {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Simplex,
            scale: 0.05,
            octaves: 4,
            persistence: 0.6,
            lacunarity: 2.2,
        },
        threshold: 0.65,
        direction: StripeDirection::Vertical,
        color: [0.15, 0.2, 0.25],
        affects: vec![TextureMapType::Albedo, TextureMapType::Roughness],
        strength: 0.5,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Albedo));
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

#[test]
fn test_layer_color_variation_generation() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::ColorVariation {
        hue_range: 10.0,
        saturation_range: 0.1,
        value_range: 0.15,
        noise_scale: 0.05,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Albedo));
}

// ========================================================================
// Material Type Tests
// ========================================================================

#[test]
fn test_material_type_metal_generation() {
    let mut params = make_params();
    params.base_material = Some(BaseMaterial {
        material_type: MaterialType::Metal,
        base_color: [0.8, 0.8, 0.8],
        roughness_range: Some([0.2, 0.4]),
        metallic: Some(1.0),
        brick_pattern: None,
        normal_params: None,
    });

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Albedo));
}

#[test]
fn test_material_type_wood_generation() {
    let mut params = make_params();
    params.base_material = Some(BaseMaterial {
        material_type: MaterialType::Wood,
        base_color: [0.6, 0.4, 0.2],
        roughness_range: Some([0.5, 0.8]),
        metallic: Some(0.0),
        brick_pattern: None,
        normal_params: None,
    });

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Albedo));
}

#[test]
fn test_material_type_brick_generation() {
    let mut params = make_params();
    params.base_material = Some(BaseMaterial {
        material_type: MaterialType::Brick,
        base_color: [0.7, 0.3, 0.2],
        roughness_range: Some([0.6, 0.9]),
        metallic: Some(0.0),
        brick_pattern: None,
        normal_params: None,
    });

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Albedo));
}

#[test]
fn test_all_material_types() {
    for mat_type in [
        MaterialType::Metal,
        MaterialType::Wood,
        MaterialType::Stone,
        MaterialType::Fabric,
        MaterialType::Plastic,
        MaterialType::Concrete,
        MaterialType::Brick,
        MaterialType::Procedural,
    ] {
        let mut params = make_params();
        params.base_material = Some(BaseMaterial {
            material_type: mat_type,
            base_color: [0.5, 0.5, 0.5],
            roughness_range: None,
            metallic: None,
            brick_pattern: None,
            normal_params: None,
        });

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(!result.maps.is_empty());
    }
}

// ========================================================================
// Noise Algorithm Tests
// ========================================================================

#[test]
fn test_noise_algorithm_perlin() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::NoisePattern {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        },
        affects: vec![TextureMapType::Roughness],
        strength: 0.5,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

#[test]
fn test_noise_algorithm_simplex() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::NoisePattern {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Simplex,
            scale: 0.05,
            octaves: 6,
            persistence: 0.6,
            lacunarity: 2.2,
        },
        affects: vec![TextureMapType::Roughness],
        strength: 0.7,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

#[test]
fn test_noise_algorithm_worley() {
    let mut params = make_params();
    params.layers = vec![TextureLayer::NoisePattern {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Worley,
            scale: 0.02,
            octaves: 3,
            persistence: 0.5,
            lacunarity: 2.0,
        },
        affects: vec![TextureMapType::Roughness],
        strength: 0.4,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

#[test]
fn test_noise_algorithm_all() {
    for algo in [
        NoiseAlgorithm::Perlin,
        NoiseAlgorithm::Simplex,
        NoiseAlgorithm::Worley,
        NoiseAlgorithm::Value,
        NoiseAlgorithm::Gabor,
        NoiseAlgorithm::Fbm,
    ] {
        let mut params = make_params();
        params.layers = vec![TextureLayer::NoisePattern {
            noise: NoiseConfig {
                algorithm: algo,
                scale: 0.1,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            affects: vec![TextureMapType::Roughness],
            strength: 0.5,
        }];

        let result = generate_material_maps(&params, 42).unwrap();
        assert!(!result.maps.is_empty());
    }
}

// ========================================================================
// File I/O Tests
// ========================================================================

#[test]
fn test_save_texture_result_writes_files() {
    let params = make_params();
    let result = generate_material_maps(&params, 42).unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let paths = save_texture_result(&result, tmp.path(), "material").unwrap();

    assert!(paths.contains_key(&TextureMapType::Albedo));
    assert!(paths.contains_key(&TextureMapType::Normal));
    assert!(paths.contains_key(&TextureMapType::Roughness));

    for path in paths.values() {
        assert!(std::path::Path::new(path).exists());
    }
}

#[test]
fn test_save_all_map_types() {
    let mut params = make_params();
    params.maps = vec![
        TextureMapType::Albedo,
        TextureMapType::Normal,
        TextureMapType::Roughness,
        TextureMapType::Metallic,
        TextureMapType::Ao,
        TextureMapType::Emissive,
        TextureMapType::Height,
    ];

    let result = generate_material_maps(&params, 42).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let paths = save_texture_result(&result, tmp.path(), "test").unwrap();

    assert_eq!(paths.len(), 7);
    for map_type in params.maps {
        assert!(paths.contains_key(&map_type));
    }
}

// ========================================================================
// Multi-Layer Tests
// ========================================================================

#[test]
fn test_multiple_layers_combined() {
    let mut params = make_params();
    params.layers = vec![
        TextureLayer::NoisePattern {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Perlin,
                scale: 0.1,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            affects: vec![TextureMapType::Roughness],
            strength: 0.3,
        },
        TextureLayer::Scratches {
            density: 0.1,
            length_range: [0.1, 0.3],
            width: 0.001,
            affects: vec![TextureMapType::Roughness],
            strength: 0.5,
        },
        TextureLayer::EdgeWear {
            amount: 0.2,
            affects: vec![TextureMapType::Roughness],
        },
    ];

    let result = generate_material_maps(&params, 42).unwrap();
    assert!(result.maps.contains_key(&TextureMapType::Roughness));
}

// ========================================================================
// Palette and Color Ramp Tests
// ========================================================================

#[test]
fn test_palette_specified() {
    let mut params = make_params();
    params.maps = vec![TextureMapType::Albedo];
    params.layers = vec![TextureLayer::Checkerboard {
        tile_size: 4,
        color1: 0.0,
        color2: 1.0,
        affects: vec![TextureMapType::Albedo],
        strength: 1.0,
    }];
    params.palette = Some(vec!["#000000".to_string(), "#0000FF".to_string()]);

    let result = generate_material_maps(&params, 42).unwrap();
    let albedo = result.maps.get(&TextureMapType::Albedo).unwrap();
    let (color_type, width, height, bytes) = decode_png_bytes(&albedo.data);
    assert_eq!(color_type, png_crate::ColorType::Rgba);
    assert_eq!(width, params.resolution[0]);
    assert_eq!(height, params.resolution[1]);

    let mut colors: HashSet<[u8; 3]> = HashSet::new();
    for chunk in bytes.chunks_exact(4) {
        colors.insert([chunk[0], chunk[1], chunk[2]]);
    }

    assert!(
        colors.contains(&[0, 0, 0]) && colors.contains(&[0, 0, 255]),
        "expected black and blue pixels, got {:?}",
        colors
    );
}

#[test]
fn test_color_ramp_specified() {
    let mut params = make_params();
    params.maps = vec![TextureMapType::Albedo];
    params.layers = vec![TextureLayer::Checkerboard {
        tile_size: 4,
        color1: 0.0,
        color2: 1.0,
        affects: vec![TextureMapType::Albedo],
        strength: 1.0,
    }];
    params.color_ramp = Some(vec!["#000000".to_string(), "#FF0000".to_string()]);

    let result = generate_material_maps(&params, 42).unwrap();
    let albedo = result.maps.get(&TextureMapType::Albedo).unwrap();
    let (color_type, _, _, bytes) = decode_png_bytes(&albedo.data);
    assert_eq!(color_type, png_crate::ColorType::Rgba);

    let mut colors: HashSet<[u8; 3]> = HashSet::new();
    for chunk in bytes.chunks_exact(4) {
        colors.insert([chunk[0], chunk[1], chunk[2]]);
    }

    assert!(
        colors.contains(&[0, 0, 0]) && colors.contains(&[255, 0, 0]),
        "expected black and red pixels, got {:?}",
        colors
    );
}

#[test]
fn test_emissive_layers_produce_non_black_output() {
    let mut params = make_params();
    params.maps = vec![TextureMapType::Emissive];
    params.layers = vec![TextureLayer::Stains {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.1,
            octaves: 2,
            persistence: 0.5,
            lacunarity: 2.0,
        },
        threshold: 0.4,
        color: [1.0, 0.5, 0.0],
        affects: vec![TextureMapType::Emissive],
        strength: 1.0,
    }];

    let result = generate_material_maps(&params, 42).unwrap();
    let emissive = result.maps.get(&TextureMapType::Emissive).unwrap();
    let (color_type, _, _, bytes) = decode_png_bytes(&emissive.data);
    assert_eq!(color_type, png_crate::ColorType::Rgb);
    assert!(
        bytes.iter().any(|&b| b != 0),
        "expected some non-black pixels"
    );
}

#[test]
fn test_metallic_layers_affect_metallic_output() {
    let mut base = make_params();
    base.maps = vec![TextureMapType::Metallic];
    base.layers.clear();

    let mut striped = base.clone();
    striped.layers = vec![TextureLayer::Stripes {
        direction: StripeDirection::Vertical,
        stripe_width: 4,
        color1: 0.0,
        color2: 1.0,
        affects: vec![TextureMapType::Metallic],
        strength: 1.0,
    }];

    let base_result = generate_material_maps(&base, 42).unwrap();
    let striped_result = generate_material_maps(&striped, 42).unwrap();

    let base_hash = &base_result
        .maps
        .get(&TextureMapType::Metallic)
        .unwrap()
        .hash;
    let striped_hash = &striped_result
        .maps
        .get(&TextureMapType::Metallic)
        .unwrap()
        .hash;
    assert_ne!(base_hash, striped_hash);
}
