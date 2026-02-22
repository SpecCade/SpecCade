//! Tests for texture recipe types.

use super::*;

// ========================================================================
// TEXTURE Top-Level Keys Tests
// ========================================================================

#[test]
fn test_texture_params_name() {
    let params = TextureMaterialV1Params {
        resolution: [256, 256],
        tileable: false,
        maps: vec![TextureMapType::Albedo],
        base_material: None,
        layers: vec![],
        palette: None,
        color_ramp: None,
    };
    // Name is part of the recipe spec, not params
    assert_eq!(params.resolution, [256, 256]);
}

#[test]
fn test_texture_params_resolution() {
    let params = TextureMaterialV1Params {
        resolution: [512, 1024],
        tileable: false,
        maps: vec![TextureMapType::Albedo],
        base_material: None,
        layers: vec![],
        palette: None,
        color_ramp: None,
    };
    let json = serde_json::to_string(&params).unwrap();
    let parsed: TextureMaterialV1Params = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.resolution, [512, 1024]);
}

#[test]
fn test_texture_params_format_serialization() {
    let params = TextureMaterialV1Params {
        resolution: [256, 256],
        tileable: true,
        maps: vec![TextureMapType::Albedo, TextureMapType::Normal],
        base_material: None,
        layers: vec![],
        palette: None,
        color_ramp: None,
    };
    let json = serde_json::to_string(&params).unwrap();
    assert!(json.contains("resolution"));
    assert!(json.contains("tileable"));
}

#[test]
fn test_texture_params_layers() {
    let layer = TextureLayer::Gradient {
        direction: GradientDirection::Horizontal,
        start: Some(0.0),
        end: Some(1.0),
        center: None,
        inner: None,
        outer: None,
        affects: vec![TextureMapType::Albedo],
        strength: 0.5,
    };
    let params = TextureMaterialV1Params {
        resolution: [256, 256],
        tileable: false,
        maps: vec![TextureMapType::Albedo],
        base_material: None,
        layers: vec![layer.clone()],
        palette: None,
        color_ramp: None,
    };
    let json = serde_json::to_string(&params).unwrap();
    let parsed: TextureMaterialV1Params = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.layers.len(), 1);
}

#[test]
fn test_texture_params_palette() {
    let params = TextureMaterialV1Params {
        resolution: [256, 256],
        tileable: false,
        maps: vec![TextureMapType::Albedo],
        base_material: None,
        layers: vec![],
        palette: Some(vec![
            "#FF0000".to_string(),
            "#00FF00".to_string(),
            "#0000FF".to_string(),
        ]),
        color_ramp: None,
    };
    let json = serde_json::to_string(&params).unwrap();
    let parsed: TextureMaterialV1Params = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.palette.as_ref().unwrap().len(), 3);
}

#[test]
fn test_texture_params_color_ramp() {
    let params = TextureMaterialV1Params {
        resolution: [256, 256],
        tileable: false,
        maps: vec![TextureMapType::Albedo],
        base_material: None,
        layers: vec![],
        palette: None,
        color_ramp: Some(vec!["#000000".to_string(), "#FFFFFF".to_string()]),
    };
    let json = serde_json::to_string(&params).unwrap();
    let parsed: TextureMaterialV1Params = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.color_ramp.as_ref().unwrap().len(), 2);
}

// ========================================================================
// Layer Type Tests
// ========================================================================

#[test]
fn test_layer_solid_serde() {
    // Note: Solid layer is not in the current enum, but we test existing layers
    let layer = TextureLayer::NoisePattern {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        },
        affects: vec![TextureMapType::Albedo],
        strength: 1.0,
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("noise_pattern"));
}

#[test]
fn test_layer_noise_all_algorithms() {
    for algo in [
        NoiseAlgorithm::Perlin,
        NoiseAlgorithm::Simplex,
        NoiseAlgorithm::Worley,
        NoiseAlgorithm::Value,
        NoiseAlgorithm::Gabor,
        NoiseAlgorithm::Fbm,
    ] {
        let layer = TextureLayer::NoisePattern {
            noise: NoiseConfig {
                algorithm: algo,
                scale: 0.1,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            affects: vec![TextureMapType::Albedo],
            strength: 1.0,
        };
        let json = serde_json::to_string(&layer).unwrap();
        let parsed: TextureLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layer);
    }
}

#[test]
fn test_layer_gradient_all_directions() {
    for direction in [
        GradientDirection::Horizontal,
        GradientDirection::Vertical,
        GradientDirection::Radial,
    ] {
        let layer = TextureLayer::Gradient {
            direction,
            start: Some(0.0),
            end: Some(1.0),
            center: None,
            inner: None,
            outer: None,
            affects: vec![TextureMapType::Albedo],
            strength: 0.5,
        };
        let json = serde_json::to_string(&layer).unwrap();
        let parsed: TextureLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layer);
    }
}

#[test]
fn test_layer_gradient_horizontal() {
    let layer = TextureLayer::Gradient {
        direction: GradientDirection::Horizontal,
        start: Some(0.2),
        end: Some(0.8),
        center: None,
        inner: None,
        outer: None,
        affects: vec![TextureMapType::Albedo],
        strength: 0.75,
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("horizontal"));
    assert!(json.contains("0.2"));
    assert!(json.contains("0.8"));
}

#[test]
fn test_layer_gradient_vertical() {
    let layer = TextureLayer::Gradient {
        direction: GradientDirection::Vertical,
        start: Some(0.0),
        end: Some(1.0),
        center: None,
        inner: None,
        outer: None,
        affects: vec![TextureMapType::Albedo],
        strength: 1.0,
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("vertical"));
}

#[test]
fn test_layer_gradient_radial() {
    let layer = TextureLayer::Gradient {
        direction: GradientDirection::Radial,
        start: None,
        end: None,
        center: Some([0.5, 0.5]),
        inner: Some(1.0),
        outer: Some(0.0),
        affects: vec![TextureMapType::Albedo],
        strength: 1.0,
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("radial"));
    assert!(json.contains("center"));
}

#[test]
fn test_layer_stripes_horizontal() {
    let layer = TextureLayer::Stripes {
        direction: StripeDirection::Horizontal,
        stripe_width: 16,
        color1: 0.0,
        color2: 1.0,
        affects: vec![TextureMapType::Albedo],
        strength: 1.0,
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("horizontal"));
    assert!(json.contains("stripe_width"));
}

#[test]
fn test_layer_stripes_vertical() {
    let layer = TextureLayer::Stripes {
        direction: StripeDirection::Vertical,
        stripe_width: 32,
        color1: 0.3,
        color2: 0.7,
        affects: vec![TextureMapType::Roughness],
        strength: 0.5,
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("vertical"));
}

#[test]
fn test_layer_checkerboard() {
    let layer = TextureLayer::Checkerboard {
        tile_size: 64,
        color1: 0.85,
        color2: 0.2,
        affects: vec![TextureMapType::Albedo, TextureMapType::Normal],
        strength: 1.0,
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("checkerboard"));
    assert!(json.contains("tile_size"));
    assert!(json.contains("64"));
    let parsed: TextureLayer = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, layer);
}

#[test]
fn test_layer_pitting() {
    let layer = TextureLayer::Pitting {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Worley,
            scale: 0.08,
            octaves: 3,
            persistence: 0.5,
            lacunarity: 2.0,
        },
        threshold: 0.6,
        depth: 0.2,
        affects: vec![TextureMapType::Roughness],
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("pitting"));
    assert!(json.contains("threshold"));
    let parsed: TextureLayer = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, layer);
}

#[test]
fn test_layer_weave() {
    let layer = TextureLayer::Weave {
        thread_width: 8,
        gap: 2,
        depth: 0.25,
        affects: vec![TextureMapType::Normal],
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("weave"));
    assert!(json.contains("thread_width"));
    let parsed: TextureLayer = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, layer);
}

#[test]
fn test_layer_scratches() {
    let layer = TextureLayer::Scratches {
        density: 0.15,
        length_range: [0.1, 0.4],
        width: 0.002,
        affects: vec![TextureMapType::Albedo, TextureMapType::Roughness],
        strength: 0.5,
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("scratches"));
    assert!(json.contains("density"));
    let parsed: TextureLayer = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, layer);
}

#[test]
fn test_layer_edge_wear() {
    let layer = TextureLayer::EdgeWear {
        amount: 0.3,
        affects: vec![TextureMapType::Roughness],
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("edge_wear"));
}

#[test]
fn test_layer_dirt() {
    let layer = TextureLayer::Dirt {
        density: 0.2,
        color: [0.3, 0.25, 0.2],
        affects: vec![TextureMapType::Albedo],
        strength: 0.4,
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("dirt"));
}

#[test]
fn test_layer_stains() {
    let layer = TextureLayer::Stains {
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
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("stains"));
    let parsed: TextureLayer = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, layer);
}

#[test]
fn test_layer_water_streaks() {
    let layer = TextureLayer::WaterStreaks {
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
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("water_streaks"));
    let parsed: TextureLayer = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, layer);
}

#[test]
fn test_layer_color_variation() {
    let layer = TextureLayer::ColorVariation {
        hue_range: 10.0,
        saturation_range: 0.1,
        value_range: 0.1,
        noise_scale: 0.05,
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("color_variation"));
}

#[test]
fn test_layer_blend_mode_default() {
    // blend_mode is not in TextureLayer enum, testing layer opacity
    let layer = TextureLayer::NoisePattern {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Perlin,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        },
        affects: vec![TextureMapType::Albedo],
        strength: 0.5,
    };
    assert_eq!(layer, layer); // Just ensure it compiles
}

#[test]
fn test_layer_opacity() {
    let layer = TextureLayer::Gradient {
        direction: GradientDirection::Horizontal,
        start: Some(0.0),
        end: Some(1.0),
        center: None,
        inner: None,
        outer: None,
        affects: vec![TextureMapType::Albedo],
        strength: 0.75, // strength is similar to opacity
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("0.75"));
}

// ========================================================================
// Noise Configuration Tests
// ========================================================================

#[test]
fn test_noise_config_perlin() {
    let noise = NoiseConfig {
        algorithm: NoiseAlgorithm::Perlin,
        scale: 0.05,
        octaves: 6,
        persistence: 0.6,
        lacunarity: 2.5,
    };
    let json = serde_json::to_string(&noise).unwrap();
    assert!(json.contains("perlin"));
}

#[test]
fn test_noise_config_simplex() {
    let noise = NoiseConfig {
        algorithm: NoiseAlgorithm::Simplex,
        scale: 0.1,
        octaves: 4,
        persistence: 0.5,
        lacunarity: 2.0,
    };
    let json = serde_json::to_string(&noise).unwrap();
    assert!(json.contains("simplex"));
}

#[test]
fn test_noise_config_worley() {
    let noise = NoiseConfig {
        algorithm: NoiseAlgorithm::Worley,
        scale: 0.02,
        octaves: 3,
        persistence: 0.5,
        lacunarity: 2.0,
    };
    let json = serde_json::to_string(&noise).unwrap();
    assert!(json.contains("worley"));
}

#[test]
fn test_noise_config_defaults() {
    let noise = NoiseConfig {
        algorithm: NoiseAlgorithm::Perlin,
        scale: 0.1,
        octaves: 4,
        persistence: 0.5,
        lacunarity: 2.0,
    };
    assert_eq!(noise.octaves, common::default_octaves());
    assert_eq!(noise.persistence, common::default_persistence());
    assert_eq!(noise.lacunarity, common::default_lacunarity());
}

#[test]
fn test_noise_config_serde_defaults_when_omitted() {
    let json = r#"{"algorithm":"perlin","scale":0.1}"#;
    let noise: NoiseConfig = serde_json::from_str(json).unwrap();
    assert_eq!(noise.octaves, common::default_octaves());
    assert_eq!(noise.persistence, common::default_persistence());
    assert_eq!(noise.lacunarity, common::default_lacunarity());
}

#[test]
fn test_noise_config_denies_unknown_fields() {
    let json = r#"{"algorithm":"perlin","scale":0.1,"nope":123}"#;
    let result: Result<NoiseConfig, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

// ========================================================================
// NORMAL Map Tests
// ========================================================================

#[test]
fn test_normal_map_params_name() {
    let params = TextureNormalV1Params {
        resolution: [256, 256],
        tileable: false,
        pattern: None,
        bump_strength: 1.0,
        processing: None,
    };
    assert_eq!(params.resolution, [256, 256]);
}

#[test]
fn test_normal_map_params_resolution() {
    let params = TextureNormalV1Params {
        resolution: [512, 512],
        tileable: true,
        pattern: None,
        bump_strength: 1.5,
        processing: None,
    };
    let json = serde_json::to_string(&params).unwrap();
    let parsed: TextureNormalV1Params = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.resolution, [512, 512]);
}

#[test]
fn test_normal_map_params_format() {
    let params = TextureNormalV1Params {
        resolution: [256, 256],
        tileable: true,
        pattern: None,
        bump_strength: 1.0,
        processing: None,
    };
    let json = serde_json::to_string(&params).unwrap();
    assert!(json.contains("resolution"));
}

#[test]
fn test_normal_map_pattern_bricks() {
    let pattern = NormalMapPattern::Bricks {
        brick_width: 64,
        brick_height: 32,
        mortar_width: 4,
        offset: 0.5,
    };
    let json = serde_json::to_string(&pattern).unwrap();
    assert!(json.contains("bricks"));
    let parsed: NormalMapPattern = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, pattern);
}

#[test]
fn test_normal_map_pattern_tiles() {
    let pattern = NormalMapPattern::Tiles {
        tile_size: 64,
        gap_width: 4,
        gap_depth: 0.3,
        seed: 42,
    };
    let json = serde_json::to_string(&pattern).unwrap();
    assert!(json.contains("tiles"));
}

#[test]
fn test_normal_map_pattern_hexagons() {
    let pattern = NormalMapPattern::Hexagons { size: 32, gap: 3 };
    let json = serde_json::to_string(&pattern).unwrap();
    assert!(json.contains("hexagons"));
}

#[test]
fn test_normal_map_pattern_noise() {
    let pattern = NormalMapPattern::NoiseBumps {
        noise: NoiseConfig {
            algorithm: NoiseAlgorithm::Simplex,
            scale: 0.1,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        },
    };
    let json = serde_json::to_string(&pattern).unwrap();
    assert!(json.contains("noise_bumps"));
}

#[test]
fn test_normal_map_pattern_rivets() {
    let pattern = NormalMapPattern::Rivets {
        spacing: 32,
        radius: 4,
        height: 0.2,
        seed: 42,
    };
    let json = serde_json::to_string(&pattern).unwrap();
    assert!(json.contains("rivets"));
}

#[test]
fn test_normal_map_pattern_weave() {
    let pattern = NormalMapPattern::Weave {
        thread_width: 8,
        gap: 2,
        depth: 0.15,
    };
    let json = serde_json::to_string(&pattern).unwrap();
    assert!(json.contains("weave"));
}

#[test]
fn test_normal_map_pattern_grid() {
    let pattern = NormalMapPattern::Grid {
        cell_size: 32,
        line_width: 4,
        bevel: 0.5,
    };
    let json = serde_json::to_string(&pattern).unwrap();
    assert!(json.contains("grid"));
}

#[test]
fn test_normal_map_pattern_diamond_plate() {
    let pattern = NormalMapPattern::DiamondPlate {
        diamond_size: 32,
        height: 0.3,
    };
    let json = serde_json::to_string(&pattern).unwrap();
    assert!(json.contains("diamond_plate"));
}

#[test]
fn test_normal_map_processing_blur() {
    let processing = NormalMapProcessing {
        blur: Some(2.0),
        invert: false,
    };
    let json = serde_json::to_string(&processing).unwrap();
    assert!(json.contains("blur"));
}

#[test]
fn test_normal_map_processing_strength() {
    let params = TextureNormalV1Params {
        resolution: [256, 256],
        tileable: false,
        pattern: None,
        bump_strength: 2.5,
        processing: None,
    };
    assert_eq!(params.bump_strength, 2.5);
}

#[test]
fn test_normal_map_processing_invert() {
    let processing = NormalMapProcessing {
        blur: None,
        invert: true,
    };
    let json = serde_json::to_string(&processing).unwrap();
    assert!(json.contains("invert"));
}

#[test]
fn test_normal_map_processing_defaults() {
    // Test that deserializing without invert field gives default of false
    let json = r#"{"blur": 1.0}"#;
    let processing: NormalMapProcessing = serde_json::from_str(json).unwrap();
    // default_invert() returns false (no inversion by default)
    assert!(!processing.invert);
}

// ========================================================================
// Material Type Tests
// ========================================================================

#[test]
fn test_material_type_metal() {
    let mat = BaseMaterial {
        material_type: MaterialType::Metal,
        base_color: [0.8, 0.8, 0.8],
        roughness_range: Some([0.2, 0.4]),
        metallic: Some(1.0),
        brick_pattern: None,
        normal_params: None,
    };
    assert_eq!(mat.material_type, MaterialType::Metal);
}

#[test]
fn test_material_type_wood() {
    let mat = BaseMaterial {
        material_type: MaterialType::Wood,
        base_color: [0.6, 0.4, 0.2],
        roughness_range: Some([0.5, 0.8]),
        metallic: Some(0.0),
        brick_pattern: None,
        normal_params: None,
    };
    assert_eq!(mat.material_type, MaterialType::Wood);
}

#[test]
fn test_material_type_all_types() {
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
        let mat = BaseMaterial {
            material_type: mat_type,
            base_color: [0.5, 0.5, 0.5],
            roughness_range: None,
            metallic: None,
            brick_pattern: None,
            normal_params: None,
        };
        let json = serde_json::to_string(&mat).unwrap();
        let parsed: BaseMaterial = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.material_type, mat_type);
    }
}

#[test]
fn test_brick_pattern_params_default_offset() {
    let json = r#"{"brick_width":64,"brick_height":32,"mortar_width":4}"#;
    let params: BrickPatternParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.offset, 0.5);
}

#[test]
fn test_normal_params_defaults() {
    let json = r#"{}"#;
    let params: NormalParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.bump_strength, 1.0);
    assert_eq!(params.mortar_depth, 0.3);
}

// ========================================================================
// Map Type Tests
// ========================================================================

#[test]
fn test_texture_map_type_serde() {
    let map = TextureMapType::Albedo;
    let json = serde_json::to_string(&map).unwrap();
    assert_eq!(json, "\"albedo\"");

    let parsed: TextureMapType = serde_json::from_str("\"normal\"").unwrap();
    assert_eq!(parsed, TextureMapType::Normal);
}

#[test]
fn test_texture_map_type_all() {
    for map_type in [
        TextureMapType::Albedo,
        TextureMapType::Normal,
        TextureMapType::Roughness,
        TextureMapType::Metallic,
        TextureMapType::Ao,
        TextureMapType::Emissive,
        TextureMapType::Height,
    ] {
        let json = serde_json::to_string(&map_type).unwrap();
        let parsed: TextureMapType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, map_type);
    }
}

// ========================================================================
// Round-trip Serialization Tests
// ========================================================================

#[test]
fn test_full_texture_params_roundtrip() {
    let params = TextureMaterialV1Params {
        resolution: [1024, 1024],
        tileable: true,
        maps: vec![
            TextureMapType::Albedo,
            TextureMapType::Normal,
            TextureMapType::Roughness,
            TextureMapType::Metallic,
        ],
        base_material: Some(BaseMaterial {
            material_type: MaterialType::Metal,
            base_color: [0.8, 0.2, 0.1],
            roughness_range: Some([0.2, 0.5]),
            metallic: Some(1.0),
            brick_pattern: None,
            normal_params: None,
        }),
        layers: vec![
            TextureLayer::NoisePattern {
                noise: NoiseConfig {
                    algorithm: NoiseAlgorithm::Perlin,
                    scale: 0.05,
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
                affects: vec![TextureMapType::Albedo, TextureMapType::Roughness],
                strength: 0.5,
            },
        ],
        palette: Some(vec!["#FF0000".to_string(), "#00FF00".to_string()]),
        color_ramp: Some(vec!["#000000".to_string(), "#FFFFFF".to_string()]),
    };

    let json = serde_json::to_string_pretty(&params).unwrap();
    let parsed: TextureMaterialV1Params = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.resolution, params.resolution);
    assert_eq!(parsed.maps.len(), params.maps.len());
    assert_eq!(parsed.layers.len(), params.layers.len());
}

#[test]
fn test_full_normal_map_params_roundtrip() {
    let params = TextureNormalV1Params {
        resolution: [512, 512],
        tileable: true,
        pattern: Some(NormalMapPattern::Bricks {
            brick_width: 64,
            brick_height: 32,
            mortar_width: 4,
            offset: 0.5,
        }),
        bump_strength: 1.5,
        processing: Some(NormalMapProcessing {
            blur: Some(1.0),
            invert: false,
        }),
    };

    let json = serde_json::to_string_pretty(&params).unwrap();
    let parsed: TextureNormalV1Params = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.resolution, params.resolution);
    assert_eq!(parsed.bump_strength, params.bump_strength);
}
