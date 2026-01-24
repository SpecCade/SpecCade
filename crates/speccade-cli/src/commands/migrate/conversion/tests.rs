//! Tests for conversion module

use super::*;
use std::collections::HashMap;

use speccade_spec::AssetType;

#[test]
fn test_map_category_to_type() {
    let (asset_type, kind) = map_category_to_type("sounds").unwrap();
    assert_eq!(asset_type, AssetType::Audio);
    assert_eq!(kind, "audio_v1");

    let (asset_type, kind) = map_category_to_type("textures").unwrap();
    assert_eq!(asset_type, AssetType::Texture);
    assert_eq!(kind, "texture.procedural_v1");
}

#[test]
fn test_extract_asset_id() {
    let id = extract_asset_id(Path::new("laser_blast_01.spec.py")).unwrap();
    assert_eq!(id, "laser-blast-01");

    assert!(
        extract_asset_id(Path::new("AB.spec.py")).is_err(),
        "too short"
    );
    assert!(
        extract_asset_id(Path::new("INVALID!.spec.py")).is_err(),
        "invalid chars"
    );
}

#[test]
fn test_generate_seed_from_filename_is_deterministic() {
    let seed1 = generate_seed_from_filename(Path::new("a.spec.py"));
    let seed2 = generate_seed_from_filename(Path::new("a.spec.py"));
    let seed3 = generate_seed_from_filename(Path::new("b.spec.py"));

    assert_eq!(seed1, seed2);
    assert_ne!(seed1, seed3);
}

#[test]
fn test_generate_outputs_normals() {
    let outputs = generate_outputs("wall-01", &AssetType::Texture, "normals").unwrap();
    assert_eq!(outputs.len(), 1);
    assert!(outputs[0].path.ends_with("_normal.png"));
    assert_eq!(outputs[0].source.as_deref(), Some("normal"));
}

#[test]
fn test_generate_outputs_textures() {
    let outputs = generate_outputs("brick-01", &AssetType::Texture, "textures").unwrap();
    assert_eq!(outputs.len(), 1);
    assert!(outputs[0].path.ends_with(".png"));
    assert_eq!(outputs[0].source.as_deref(), Some("albedo"));
}

// ========================================================================
// Audio Mapping Tests
// ========================================================================

#[test]
fn test_map_sound_simple_beep() {
    let legacy = LegacySpec {
        dict_name: "SOUND".to_string(),
        category: "sounds".to_string(),
        data: HashMap::from([
            ("name".to_string(), serde_json::json!("simple_beep")),
            ("duration".to_string(), serde_json::json!(0.3)),
            ("sample_rate".to_string(), serde_json::json!(44100)),
            (
                "layers".to_string(),
                serde_json::json!([{
                    "type": "sine",
                    "freq": 880,
                    "amplitude": 0.8,
                    "envelope": {
                        "attack": 0.01,
                        "decay": 0.05,
                        "sustain": 0.6,
                        "release": 0.15
                    }
                }]),
            ),
        ]),
    };

    let (params, warnings) = map_legacy_keys_to_params(&legacy, "audio_v1").unwrap();

    assert_eq!(params["duration_seconds"].as_f64().unwrap(), 0.3);
    assert_eq!(params["sample_rate"].as_u64().unwrap(), 44100);

    let layers = params["layers"].as_array().unwrap();
    assert_eq!(layers.len(), 1);

    let synth = &layers[0]["synthesis"];
    assert_eq!(synth["type"].as_str().unwrap(), "oscillator");
    assert_eq!(synth["waveform"].as_str().unwrap(), "sine");
    assert_eq!(synth["frequency"].as_f64().unwrap(), 880.0);

    assert_eq!(layers[0]["volume"].as_f64().unwrap(), 0.8);
    assert!(warnings.is_empty() || warnings.iter().all(|w| !w.contains("error")));
}

#[test]
fn test_map_sound_fm_synth() {
    let legacy = LegacySpec {
        dict_name: "SOUND".to_string(),
        category: "sounds".to_string(),
        data: HashMap::from([
            ("duration".to_string(), serde_json::json!(0.25)),
            (
                "layers".to_string(),
                serde_json::json!([{
                    "type": "fm_synth",
                    "carrier_freq": 1200,
                    "mod_ratio": 2.5,
                    "mod_index": 8.0,
                    "envelope": { "attack": 0.001, "decay": 0.1, "sustain": 0.3, "release": 0.1 }
                }]),
            ),
        ]),
    };

    let (params, _warnings) = map_legacy_keys_to_params(&legacy, "audio_v1").unwrap();

    let layers = params["layers"].as_array().unwrap();
    let synth = &layers[0]["synthesis"];
    assert_eq!(synth["type"].as_str().unwrap(), "fm_synth");
    assert_eq!(synth["carrier_freq"].as_f64().unwrap(), 1200.0);
    assert_eq!(synth["modulation_index"].as_f64().unwrap(), 8.0);
    // mod_ratio 2.5 * carrier 1200 = 3000
    assert_eq!(synth["modulator_freq"].as_f64().unwrap(), 3000.0);
}

#[test]
fn test_map_sound_noise_burst_with_filter() {
    let legacy = LegacySpec {
        dict_name: "SOUND".to_string(),
        category: "sounds".to_string(),
        data: HashMap::from([
            ("duration".to_string(), serde_json::json!(0.5)),
            (
                "layers".to_string(),
                serde_json::json!([{
                    "type": "noise_burst",
                    "color": "brown",
                    "amplitude": 1.0,
                    "envelope": { "attack": 0.005, "decay": 0.3, "sustain": 0.2, "release": 0.7 },
                    "filter": {
                        "type": "lowpass",
                        "cutoff": 800,
                        "cutoff_end": 200,
                        "q": 1.2
                    }
                }]),
            ),
        ]),
    };

    let (params, _warnings) = map_legacy_keys_to_params(&legacy, "audio_v1").unwrap();

    let layers = params["layers"].as_array().unwrap();
    let synth = &layers[0]["synthesis"];
    assert_eq!(synth["type"].as_str().unwrap(), "noise_burst");
    assert_eq!(synth["noise_type"].as_str().unwrap(), "brown");

    let filter = &layers[0]["filter"];
    assert_eq!(filter["type"].as_str().unwrap(), "lowpass");
    assert_eq!(filter["cutoff"].as_f64().unwrap(), 800.0);
    assert_eq!(filter["cutoff_end"].as_f64().unwrap(), 200.0);
    assert_eq!(filter["resonance"].as_f64().unwrap(), 1.2);
}

#[test]
fn test_map_instrument_karplus_strong() {
    let legacy = LegacySpec {
        dict_name: "INSTRUMENT".to_string(),
        category: "instruments".to_string(),
        data: HashMap::from([
            ("name".to_string(), serde_json::json!("bass_pluck")),
            ("base_note".to_string(), serde_json::json!("C2")),
            ("sample_rate".to_string(), serde_json::json!(44100)),
            (
                "synthesis".to_string(),
                serde_json::json!({
                    "type": "karplus_strong",
                    "damping": 0.998,
                    "brightness": 0.5
                }),
            ),
            (
                "envelope".to_string(),
                serde_json::json!({ "attack": 0.001, "decay": 0.2, "sustain": 0.3, "release": 0.4 }),
            ),
            (
                "output".to_string(),
                serde_json::json!({ "duration": 1.5, "bit_depth": 16 }),
            ),
        ]),
    };

    let (params, _warnings) = map_legacy_keys_to_params(&legacy, "audio_v1").unwrap();

    assert_eq!(params["duration_seconds"].as_f64().unwrap(), 1.5);
    assert_eq!(params["base_note"].as_str().unwrap(), "C2");

    let layers = params["layers"].as_array().unwrap();
    assert_eq!(layers.len(), 1);

    let synth = &layers[0]["synthesis"];
    assert_eq!(synth["type"].as_str().unwrap(), "karplus_strong");
    assert_eq!(synth["decay"].as_f64().unwrap(), 0.998);
    assert_eq!(synth["blend"].as_f64().unwrap(), 0.5);
}

#[test]
fn test_map_sound_warnings_for_deprecated_fields() {
    let legacy = LegacySpec {
        dict_name: "SOUND".to_string(),
        category: "sounds".to_string(),
        data: HashMap::from([
            ("duration".to_string(), serde_json::json!(0.5)),
            ("normalize".to_string(), serde_json::json!(true)),
            ("peak_db".to_string(), serde_json::json!(-3.0)),
            ("layers".to_string(), serde_json::json!([])),
        ]),
    };

    let (_params, warnings) = map_legacy_keys_to_params(&legacy, "audio_v1").unwrap();
    assert!(
        warnings.iter().any(|w| w.contains("normalize")),
        "should warn about normalize"
    );
}

// ========================================================================
// Music Mapping Tests
// ========================================================================

#[test]
fn test_map_song_simple_loop() {
    let legacy = LegacySpec {
        dict_name: "SONG".to_string(),
        category: "music".to_string(),
        data: HashMap::from([
            ("name".to_string(), serde_json::json!("simple_loop")),
            ("title".to_string(), serde_json::json!("Simple Loop")),
            ("format".to_string(), serde_json::json!("xm")),
            ("bpm".to_string(), serde_json::json!(120)),
            ("speed".to_string(), serde_json::json!(6)),
            ("channels".to_string(), serde_json::json!(4)),
            ("restart_position".to_string(), serde_json::json!(0)),
            (
                "instruments".to_string(),
                serde_json::json!([
                    {
                        "name": "lead",
                        "synthesis": { "type": "karplus_strong", "damping": 0.996, "brightness": 0.7 },
                        "envelope": { "attack": 0.001, "decay": 0.1, "sustain": 0.5, "release": 0.2 }
                    }
                ]),
            ),
            (
                "patterns".to_string(),
                serde_json::json!({
                    "intro": {
                        "rows": 64,
                        "notes": {
                            "0": [
                                { "row": 0, "note": "C4", "inst": 0, "vol": 64 }
                            ]
                        }
                    }
                }),
            ),
            (
                "arrangement".to_string(),
                serde_json::json!([{ "pattern": "intro", "repeat": 4 }]),
            ),
        ]),
    };

    let (params, warnings) = map_legacy_keys_to_params(&legacy, "music.tracker_song_v1").unwrap();

    assert_eq!(params["format"].as_str().unwrap(), "xm");
    assert_eq!(params["bpm"].as_u64().unwrap(), 120);
    assert_eq!(params["speed"].as_u64().unwrap(), 6);
    assert_eq!(params["channels"].as_u64().unwrap(), 4);
    assert_eq!(params["name"].as_str().unwrap(), "simple_loop");
    assert_eq!(params["title"].as_str().unwrap(), "Simple Loop");

    let instruments = params["instruments"].as_array().unwrap();
    assert_eq!(instruments.len(), 1);
    assert_eq!(instruments[0]["name"].as_str().unwrap(), "lead");

    let patterns = params["patterns"].as_object().unwrap();
    assert!(patterns.contains_key("intro"));
    assert_eq!(patterns["intro"]["rows"].as_u64().unwrap(), 64);

    let arrangement = params["arrangement"].as_array().unwrap();
    assert_eq!(arrangement.len(), 1);
    assert_eq!(arrangement[0]["pattern"].as_str().unwrap(), "intro");
    assert_eq!(arrangement[0]["repeat"].as_u64().unwrap(), 4);

    assert!(warnings.is_empty());
}

// ========================================================================
// Mesh Mapping Tests
// ========================================================================

#[test]
fn test_map_mesh_simple_cube() {
    let legacy = LegacySpec {
        dict_name: "MESH".to_string(),
        category: "meshes".to_string(),
        data: HashMap::from([
            ("name".to_string(), serde_json::json!("simple_cube")),
            ("primitive".to_string(), serde_json::json!("cube")),
            ("params".to_string(), serde_json::json!({ "size": 1.0 })),
            ("shade".to_string(), serde_json::json!("smooth")),
            (
                "modifiers".to_string(),
                serde_json::json!([{ "type": "bevel", "width": 0.02, "segments": 2 }]),
            ),
            (
                "uv".to_string(),
                serde_json::json!({ "mode": "smart_project", "angle_limit": 66.0 }),
            ),
            (
                "export".to_string(),
                serde_json::json!({ "tangents": false }),
            ),
        ]),
    };

    let (params, warnings) =
        map_legacy_keys_to_params(&legacy, "static_mesh.blender_primitives_v1").unwrap();

    assert_eq!(params["base_primitive"].as_str().unwrap(), "cube");

    let dimensions = params["dimensions"].as_array().unwrap();
    assert_eq!(dimensions[0].as_f64().unwrap(), 1.0);
    assert_eq!(dimensions[1].as_f64().unwrap(), 1.0);
    assert_eq!(dimensions[2].as_f64().unwrap(), 1.0);

    let modifiers = params["modifiers"].as_array().unwrap();
    assert_eq!(modifiers.len(), 1);
    assert_eq!(modifiers[0]["type"].as_str().unwrap(), "bevel");
    assert_eq!(modifiers[0]["width"].as_f64().unwrap(), 0.02);

    let uv = &params["uv_projection"];
    assert_eq!(uv["method"].as_str().unwrap(), "smart");
    assert_eq!(uv["angle_limit"].as_f64().unwrap(), 66.0);

    let export = &params["export"];
    assert!(!export["tangents"].as_bool().unwrap());

    // Smooth shading should set normals preset
    assert!(params.get("normals").is_some());

    assert!(
        warnings.is_empty() || warnings.iter().all(|w| !w.contains("error")),
        "unexpected error warnings: {:?}",
        warnings
    );
}

#[test]
fn test_map_mesh_cylinder_dimensions() {
    let legacy = LegacySpec {
        dict_name: "MESH".to_string(),
        category: "meshes".to_string(),
        data: HashMap::from([
            ("primitive".to_string(), serde_json::json!("cylinder")),
            (
                "params".to_string(),
                serde_json::json!({ "radius": 0.3, "depth": 2.0, "vertices": 32 }),
            ),
        ]),
    };

    let (params, _warnings) =
        map_legacy_keys_to_params(&legacy, "static_mesh.blender_primitives_v1").unwrap();

    assert_eq!(params["base_primitive"].as_str().unwrap(), "cylinder");

    let dimensions = params["dimensions"].as_array().unwrap();
    // radius 0.3 -> diameter 0.6
    assert!((dimensions[0].as_f64().unwrap() - 0.6).abs() < 0.001);
    assert!((dimensions[1].as_f64().unwrap() - 0.6).abs() < 0.001);
    // depth 2.0
    assert_eq!(dimensions[2].as_f64().unwrap(), 2.0);
}

#[test]
fn test_map_mesh_sphere_dimensions() {
    let legacy = LegacySpec {
        dict_name: "MESH".to_string(),
        category: "meshes".to_string(),
        data: HashMap::from([
            ("primitive".to_string(), serde_json::json!("sphere")),
            (
                "params".to_string(),
                serde_json::json!({ "radius": 0.5, "segments": 48, "rings": 24 }),
            ),
        ]),
    };

    let (params, _warnings) =
        map_legacy_keys_to_params(&legacy, "static_mesh.blender_primitives_v1").unwrap();

    assert_eq!(params["base_primitive"].as_str().unwrap(), "sphere");

    let dimensions = params["dimensions"].as_array().unwrap();
    // radius 0.5 -> diameter 1.0
    assert_eq!(dimensions[0].as_f64().unwrap(), 1.0);
    assert_eq!(dimensions[1].as_f64().unwrap(), 1.0);
    assert_eq!(dimensions[2].as_f64().unwrap(), 1.0);
}

#[test]
fn test_map_mesh_unknown_primitive_warns() {
    let legacy = LegacySpec {
        dict_name: "MESH".to_string(),
        category: "meshes".to_string(),
        data: HashMap::from([("primitive".to_string(), serde_json::json!("unknown_shape"))]),
    };

    let (params, warnings) =
        map_legacy_keys_to_params(&legacy, "static_mesh.blender_primitives_v1").unwrap();

    // Should default to cube and warn
    assert_eq!(params["base_primitive"].as_str().unwrap(), "cube");
    assert!(warnings.iter().any(|w| w.contains("unknown_shape")));
}

#[test]
fn test_map_mesh_warns_about_transforms() {
    let legacy = LegacySpec {
        dict_name: "MESH".to_string(),
        category: "meshes".to_string(),
        data: HashMap::from([
            ("primitive".to_string(), serde_json::json!("cube")),
            ("location".to_string(), serde_json::json!([1, 2, 3])),
            ("rotation".to_string(), serde_json::json!([0, 0, 0])),
        ]),
    };

    let (_params, warnings) =
        map_legacy_keys_to_params(&legacy, "static_mesh.blender_primitives_v1").unwrap();

    assert!(warnings.iter().any(|w| w.contains("location")));
}
