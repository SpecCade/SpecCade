//! Tests for packed texture map generation.

use std::collections::HashMap;

use speccade_spec::recipe::texture::{MapDefinition, TexturePackedV1Params};

use crate::packing::{pack_channels, ChannelSource, PackedChannels};
use crate::png;

use super::generate_packed_maps;

fn make_orm_params(resolution: [u32; 2]) -> TexturePackedV1Params {
    let mut maps = HashMap::new();
    maps.insert(
        "height".to_string(),
        MapDefinition::Pattern {
            pattern: "noise".to_string(),
            noise_type: Some("fbm".to_string()),
            octaves: Some(4),
            axis: None,
            frequency: None,
            duty_cycle: None,
            phase: None,
            cells: None,
            line_width: None,
            start: None,
            end: None,
            jitter: None,
            distance_fn: None,
        },
    );
    maps.insert(
        "ao".to_string(),
        MapDefinition::Grayscale {
            value: None,
            from_height: Some(true),
            ao_strength: Some(0.5),
        },
    );
    maps.insert(
        "roughness".to_string(),
        MapDefinition::Grayscale {
            value: None,
            from_height: Some(true),
            ao_strength: None,
        },
    );
    maps.insert(
        "metallic".to_string(),
        MapDefinition::Grayscale {
            value: Some(1.0),
            from_height: None,
            ao_strength: None,
        },
    );

    TexturePackedV1Params {
        resolution,
        tileable: true,
        maps,
    }
}

#[test]
fn packed_maps_are_deterministic() {
    let params = make_orm_params([16, 16]);
    let maps_a = generate_packed_maps(&params, 123).unwrap();
    let maps_b = generate_packed_maps(&params, 123).unwrap();

    let channels = PackedChannels::rgb(
        ChannelSource::key("ao"),
        ChannelSource::key("roughness"),
        ChannelSource::key("metallic"),
    );

    let packed_a = pack_channels(&channels, &maps_a, 16, 16).unwrap();
    let packed_b = pack_channels(&channels, &maps_b, 16, 16).unwrap();

    let config = crate::png::PngConfig::default();
    let (_, hash_a) = png::write_rgba_to_vec_with_hash(&packed_a, &config).unwrap();
    let (_, hash_b) = png::write_rgba_to_vec_with_hash(&packed_b, &config).unwrap();

    assert_eq!(hash_a, hash_b);
}

#[test]
fn packed_channels_match_sources() {
    let params = make_orm_params([8, 8]);
    let maps = generate_packed_maps(&params, 77).unwrap();

    let channels = PackedChannels::rgb(
        ChannelSource::key("ao"),
        ChannelSource::key("roughness"),
        ChannelSource::key("metallic"),
    );

    let packed = pack_channels(&channels, &maps, 8, 8).unwrap();

    let ao = maps.get("ao").unwrap();
    let roughness = maps.get("roughness").unwrap();
    let metallic = maps.get("metallic").unwrap();

    for (x, y) in [(0, 0), (3, 5), (7, 2)] {
        let packed_pixel = packed.get(x, y);
        let ao_pixel = ao.get(x, y);
        let rough_pixel = roughness.get(x, y);
        let metal_pixel = metallic.get(x, y);

        assert!(
            (packed_pixel.r - ao_pixel.r).abs() < 1e-10,
            "AO channel mismatch at ({}, {})",
            x,
            y
        );
        assert!(
            (packed_pixel.g - rough_pixel.r).abs() < 1e-10,
            "Roughness channel mismatch at ({}, {})",
            x,
            y
        );
        assert!(
            (packed_pixel.b - metal_pixel.r).abs() < 1e-10,
            "Metallic channel mismatch at ({}, {})",
            x,
            y
        );
    }
}

#[test]
fn packed_inversion_matches_expected() {
    let mut maps = HashMap::new();
    maps.insert(
        "height".to_string(),
        MapDefinition::Pattern {
            pattern: "noise".to_string(),
            noise_type: Some("fbm".to_string()),
            octaves: Some(4),
            axis: None,
            frequency: None,
            duty_cycle: None,
            phase: None,
            cells: None,
            line_width: None,
            start: None,
            end: None,
            jitter: None,
            distance_fn: None,
        },
    );
    maps.insert(
        "rough".to_string(),
        MapDefinition::Grayscale {
            value: None,
            from_height: Some(true),
            ao_strength: None,
        },
    );

    let params = TexturePackedV1Params {
        resolution: [8, 8],
        tileable: true,
        maps,
    };

    let maps = generate_packed_maps(&params, 456).unwrap();
    let rough = maps.get("rough").unwrap();

    let invert = ChannelSource::extended("rough").invert(true).build();
    let channels = PackedChannels::rgb(invert.clone(), invert.clone(), invert);

    let packed = pack_channels(&channels, &maps, 8, 8).unwrap();

    for (x, y) in [(1, 1), (4, 3), (6, 7)] {
        let packed_pixel = packed.get(x, y);
        let rough_pixel = rough.get(x, y);
        let expected = 1.0 - rough_pixel.r;

        assert!(
            (packed_pixel.r - expected).abs() < 1e-10,
            "Invert mismatch at ({}, {})",
            x,
            y
        );
        assert!(
            (packed_pixel.g - expected).abs() < 1e-10,
            "Invert mismatch at ({}, {})",
            x,
            y
        );
        assert!(
            (packed_pixel.b - expected).abs() < 1e-10,
            "Invert mismatch at ({}, {})",
            x,
            y
        );
    }
}

#[test]
fn stripes_pattern_generates_expected_pixels() {
    let mut maps = HashMap::new();
    maps.insert(
        "stripes".to_string(),
        MapDefinition::Pattern {
            pattern: "stripes".to_string(),
            noise_type: None,
            octaves: None,
            axis: Some("x".to_string()),
            frequency: Some(2),
            duty_cycle: Some(0.5),
            phase: Some(0.0),
            cells: None,
            line_width: None,
            start: None,
            end: None,
            jitter: None,
            distance_fn: None,
        },
    );

    let params = TexturePackedV1Params {
        resolution: [8, 1],
        tileable: true,
        maps,
    };

    let maps = generate_packed_maps(&params, 0).unwrap();
    let stripes = maps.get("stripes").unwrap();

    let expected = [1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
    for (x, expected) in expected.into_iter().enumerate() {
        let actual = stripes.get(x as u32, 0).r;
        assert!(
            (actual - expected).abs() < 1e-12,
            "stripes mismatch at x={}: expected {}, got {}",
            x,
            expected,
            actual
        );
    }
}

#[test]
fn grid_pattern_generates_expected_lines() {
    let mut maps = HashMap::new();
    maps.insert(
        "grid".to_string(),
        MapDefinition::Pattern {
            pattern: "grid".to_string(),
            noise_type: None,
            octaves: None,
            axis: None,
            frequency: None,
            duty_cycle: None,
            phase: Some(0.0),
            cells: Some([2, 2]),
            line_width: Some(0.25),
            start: None,
            end: None,
            jitter: None,
            distance_fn: None,
        },
    );

    let params = TexturePackedV1Params {
        resolution: [8, 8],
        tileable: true,
        maps,
    };

    let maps = generate_packed_maps(&params, 0).unwrap();
    let grid = maps.get("grid").unwrap();

    assert!((grid.get(0, 0).r - 1.0).abs() < 1e-12);
    assert!((grid.get(1, 0).r - 1.0).abs() < 1e-12);
    assert!((grid.get(0, 1).r - 1.0).abs() < 1e-12);
    assert!((grid.get(1, 1).r - 0.0).abs() < 1e-12);
    assert!((grid.get(2, 2).r - 0.0).abs() < 1e-12);
    assert!((grid.get(4, 4).r - 1.0).abs() < 1e-12);
}

#[test]
fn gradient_pattern_generates_expected_ramp() {
    let mut maps = HashMap::new();
    maps.insert(
        "gradient".to_string(),
        MapDefinition::Pattern {
            pattern: "gradient".to_string(),
            noise_type: None,
            octaves: None,
            axis: Some("x".to_string()),
            frequency: None,
            duty_cycle: None,
            phase: Some(0.0),
            cells: None,
            line_width: None,
            start: Some(0.0),
            end: Some(1.0),
            jitter: None,
            distance_fn: None,
        },
    );

    let params = TexturePackedV1Params {
        resolution: [8, 1],
        tileable: true,
        maps,
    };

    let maps = generate_packed_maps(&params, 0).unwrap();
    let gradient = maps.get("gradient").unwrap();

    assert!((gradient.get(0, 0).r - 0.0).abs() < 1e-12);
    assert!((gradient.get(4, 0).r - 1.0).abs() < 1e-12);
    assert!((gradient.get(7, 0).r - 0.25).abs() < 1e-12);
}

#[test]
fn worley_edges_pattern_is_deterministic() {
    let mut maps = HashMap::new();
    maps.insert(
        "edges".to_string(),
        MapDefinition::Pattern {
            pattern: "worley_edges".to_string(),
            noise_type: None,
            octaves: None,
            axis: None,
            frequency: None,
            duty_cycle: None,
            phase: None,
            cells: None,
            line_width: None,
            start: None,
            end: None,
            jitter: Some(1.0),
            distance_fn: Some("manhattan".to_string()),
        },
    );

    let params = TexturePackedV1Params {
        resolution: [16, 16],
        tileable: true,
        maps,
    };

    let maps_a = generate_packed_maps(&params, 123).unwrap();
    let maps_b = generate_packed_maps(&params, 123).unwrap();

    let config = crate::png::PngConfig::default();
    let (_, hash_a) =
        png::write_rgba_to_vec_with_hash(maps_a.get("edges").unwrap(), &config).unwrap();
    let (_, hash_b) =
        png::write_rgba_to_vec_with_hash(maps_b.get("edges").unwrap(), &config).unwrap();

    assert_eq!(hash_a, hash_b);
}

#[test]
fn noise_pattern_rejects_non_noise_params() {
    let mut maps = HashMap::new();
    maps.insert(
        "height".to_string(),
        MapDefinition::Pattern {
            pattern: "noise".to_string(),
            noise_type: Some("perlin".to_string()),
            octaves: None,
            axis: None,
            frequency: Some(4),
            duty_cycle: None,
            phase: None,
            cells: None,
            line_width: None,
            start: None,
            end: None,
            jitter: None,
            distance_fn: None,
        },
    );

    let params = TexturePackedV1Params {
        resolution: [8, 8],
        tileable: true,
        maps,
    };

    let err = generate_packed_maps(&params, 0).unwrap_err();
    assert!(err.to_string().contains("does not accept"));
}
