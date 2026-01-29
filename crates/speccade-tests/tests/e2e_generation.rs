//! End-to-End Generation Tests for SpecCade
//!
//! Tests verify each asset type produces valid outputs.
//! Includes both Tier 1 (Rust backends) and Tier 2 (Blender backends) tests.
//!
//! ## Running Tests
//!
//! ```bash
//! # Run Tier 1 tests (no Blender required)
//! cargo test -p speccade-tests --test e2e_generation
//!
//! # Run Tier 2 tests (requires Blender)
//! SPECCADE_RUN_BLENDER_TESTS=1 cargo test -p speccade-tests --test e2e_generation --ignored
//! ```

use std::fs;

use speccade_spec::recipe::audio::{AudioLayer, AudioV1Params, Envelope, Synthesis, Waveform};
use speccade_spec::recipe::music::{
    ArrangementEntry, InstrumentSynthesis, MusicTrackerSongV1Params, PatternNote, TrackerFormat,
    TrackerInstrument, TrackerPattern,
};
use speccade_spec::recipe::texture::{
    NoiseAlgorithm, NoiseConfig, TextureProceduralNode, TextureProceduralOp,
    TextureProceduralV1Params,
};
use speccade_spec::{AssetType, OutputFormat, OutputKind, OutputSpec, Recipe, Spec};
use speccade_tests::fixtures::GoldenFixtures;
use speccade_tests::harness::{
    is_blender_available, parse_spec_file, should_run_blender_tests, validate_glb_file,
    validate_png_file, validate_wav_file, validate_xm_file, TestHarness,
};

// ============================================================================
// Tier 1: Generation Tests (Rust backends)
// ============================================================================

/// Test audio generation produces valid WAV output.
#[test]
fn test_generate_audio_wav() {
    let harness = TestHarness::new();

    // Create a minimal audio spec
    let spec = Spec::builder("test-beep-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(
            OutputFormat::Wav,
            "audio/test_beep.wav",
        ))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "sample_rate": 22050,
                "layers": [{
                    "synthesis": {
                        "type": "oscillator",
                        "waveform": "sine",
                        "frequency": 440.0
                    },
                    "envelope": {
                        "attack": 0.01,
                        "decay": 0.02,
                        "sustain": 0.5,
                        "release": 0.05
                    },
                    "volume": 0.8,
                    "pan": 0.0
                }]
            }),
        ))
        .build();

    // Generate using the backend directly
    let result = speccade_backend_audio::generate(&spec);
    assert!(
        result.is_ok(),
        "Audio generation failed: {:?}",
        result.err()
    );

    let gen_result = result.unwrap();
    assert!(!gen_result.wav.wav_data.is_empty());

    // Write output and validate
    let output_path = harness.path().join("audio").join("test_beep.wav");
    fs::create_dir_all(output_path.parent().unwrap()).unwrap();
    fs::write(&output_path, &gen_result.wav.wav_data).unwrap();

    let validation = validate_wav_file(&output_path);
    assert!(
        validation.is_ok(),
        "WAV validation failed: {:?}",
        validation.err()
    );
}

/// Test audio generation with params struct.
#[test]
fn test_generate_audio_from_params() {
    let params = AudioV1Params {
        base_note: None,
        duration_seconds: 0.2,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 880.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope {
                attack: 0.01,
                decay: 0.05,
                sustain: 0.6,
                release: 0.1,
            },
            volume: 0.7,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        loop_config: None,
        generate_loop_points: false,
        master_filter: None,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let result = speccade_backend_audio::generate_from_params(&params, 42);
    assert!(
        result.is_ok(),
        "Audio generation from params failed: {:?}",
        result.err()
    );

    let gen_result = result.unwrap();
    assert!(!gen_result.wav.wav_data.is_empty());
}

/// Test audio generation with loop points enabled.
#[test]
fn test_generate_audio_with_loop_points() {
    let params = AudioV1Params {
        base_note: None,
        duration_seconds: 0.5,
        sample_rate: 22050,
        layers: vec![AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sawtooth,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.2,
            },
            volume: 1.0,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        pitch_envelope: None,
        loop_config: None,
        generate_loop_points: true,
        master_filter: None,
        effects: vec![],
        post_fx_lfos: vec![],
    };

    let result = speccade_backend_audio::generate_from_params(&params, 42);
    assert!(
        result.is_ok(),
        "Instrument generation failed: {:?}",
        result.err()
    );

    let gen_result = result.unwrap();
    assert!(!gen_result.wav.wav_data.is_empty());
}

/// Test procedural texture graph generation produces valid PNG output.
#[test]
fn test_generate_texture_procedural() {
    let harness = TestHarness::new();

    let params = TextureProceduralV1Params {
        resolution: [64, 64],
        tileable: true,
        nodes: vec![
            TextureProceduralNode {
                id: "n".to_string(),
                op: TextureProceduralOp::Noise {
                    noise: NoiseConfig {
                        algorithm: NoiseAlgorithm::Perlin,
                        scale: 0.1,
                        octaves: 3,
                        persistence: 0.5,
                        lacunarity: 2.0,
                    },
                },
            },
            TextureProceduralNode {
                id: "mask".to_string(),
                op: TextureProceduralOp::Threshold {
                    input: "n".to_string(),
                    threshold: 0.5,
                },
            },
        ],
    };

    let result = speccade_backend_texture::generate_graph(&params, 42);
    assert!(
        result.is_ok(),
        "Procedural graph generation failed: {:?}",
        result.err()
    );

    let nodes = result.unwrap();
    let mask = nodes.get("mask").expect("mask node missing");
    let (png_bytes, _) =
        speccade_backend_texture::encode_graph_value_png(mask).expect("encode failed");

    // Write output and validate
    let output_path = harness.path().join("test_mask.png");
    fs::write(&output_path, &png_bytes).unwrap();

    let validation = validate_png_file(&output_path);
    assert!(
        validation.is_ok(),
        "PNG validation failed: {:?}",
        validation.err()
    );
}

/// Test procedural normal-from-height generation.
#[test]
fn test_generate_texture_normal_from_height() {
    let harness = TestHarness::new();

    let params = TextureProceduralV1Params {
        resolution: [64, 64],
        tileable: true,
        nodes: vec![
            TextureProceduralNode {
                id: "height".to_string(),
                op: TextureProceduralOp::Noise {
                    noise: NoiseConfig {
                        algorithm: NoiseAlgorithm::Perlin,
                        scale: 0.12,
                        octaves: 2,
                        persistence: 0.5,
                        lacunarity: 2.0,
                    },
                },
            },
            TextureProceduralNode {
                id: "normal".to_string(),
                op: TextureProceduralOp::NormalFromHeight {
                    input: "height".to_string(),
                    strength: 1.0,
                },
            },
        ],
    };

    let nodes = speccade_backend_texture::generate_graph(&params, 7)
        .expect("Normal-from-height graph failed");
    let normal = nodes.get("normal").expect("normal node missing");
    let (png_bytes, _) =
        speccade_backend_texture::encode_graph_value_png(normal).expect("encode failed");

    let output_path = harness.path().join("test_normal.png");
    fs::write(&output_path, &png_bytes).unwrap();

    let validation = validate_png_file(&output_path);
    assert!(
        validation.is_ok(),
        "PNG validation failed: {:?}",
        validation.err()
    );
}

/// Test music generation produces valid XM output.
#[test]
fn test_generate_music_xm() {
    let harness = TestHarness::new();

    let mut patterns = std::collections::HashMap::new();
    patterns.insert(
        "intro".to_string(),
        TrackerPattern {
            rows: 16,
            data: Some(vec![PatternNote {
                row: 0,
                channel: Some(0),
                note: "C-4".to_string(),
                inst: 0,
                vol: None,
                ..Default::default()
            }]),
            ..Default::default()
        },
    );

    let params = MusicTrackerSongV1Params {
        format: TrackerFormat::Xm,
        bpm: 120,
        speed: 6,
        channels: 4,
        r#loop: false,
        instruments: vec![TrackerInstrument {
            name: "bass".to_string(),
            synthesis: Some(InstrumentSynthesis::Pulse { duty_cycle: 0.5 }),
            envelope: Envelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.2,
            },
            ..Default::default()
        }],
        patterns,
        arrangement: vec![ArrangementEntry {
            pattern: "intro".to_string(),
            repeat: 1,
        }],
        ..Default::default()
    };

    let result = speccade_backend_music::generate_music(&params, 42, harness.path());
    assert!(
        result.is_ok(),
        "Music generation failed: {:?}",
        result.err()
    );

    let gen_result = result.unwrap();
    assert_eq!(gen_result.extension, "xm");
    assert!(!gen_result.data.is_empty());

    // Write output and validate
    let output_path = harness.path().join("test_song.xm");
    fs::write(&output_path, &gen_result.data).unwrap();

    let validation = validate_xm_file(&output_path);
    assert!(
        validation.is_ok(),
        "XM validation failed: {:?}",
        validation.err()
    );
}

/// Test that golden audio specs can be generated.
#[test]
fn test_generate_golden_audio() {
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping test");
        return;
    }

    let harness = TestHarness::new();
    let specs = GoldenFixtures::list_speccade_specs("audio");

    for spec_path in specs.iter().take(3) {
        // Test a subset for speed
        let spec = parse_spec_file(spec_path).expect("Failed to parse spec");

        if spec.recipe.is_none() {
            continue;
        }

        let result = speccade_backend_audio::generate(&spec);
        assert!(
            result.is_ok(),
            "Failed to generate {:?}: {:?}",
            spec_path,
            result.err()
        );

        let gen_result = result.unwrap();

        // Write and validate output
        let output_path = harness.path().join(format!("{}.wav", spec.asset_id));
        fs::write(&output_path, &gen_result.wav.wav_data).unwrap();

        let validation = validate_wav_file(&output_path);
        assert!(
            validation.is_ok(),
            "WAV validation failed for {:?}: {:?}",
            spec_path,
            validation.err()
        );
    }
}

/// Test that golden texture specs can be generated.
#[test]
fn test_generate_golden_textures() {
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping test");
        return;
    }

    let specs = GoldenFixtures::list_speccade_specs("texture");

    for spec_path in specs.iter().take(3) {
        let spec = parse_spec_file(spec_path).expect("Failed to parse spec");

        if spec.recipe.is_none() {
            continue;
        }

        let recipe = spec.recipe.as_ref().unwrap();

        if recipe.kind == "texture.procedural_v1" {
            let params = recipe.as_texture_procedural();
            if let Ok(params) = params {
                let nodes = speccade_backend_texture::generate_graph(&params, spec.seed);
                assert!(
                    nodes.is_ok(),
                    "Failed to generate procedural texture {:?}: {:?}",
                    spec_path,
                    nodes.err()
                );

                let nodes = nodes.unwrap();
                for output in spec
                    .outputs
                    .iter()
                    .filter(|o| o.kind == OutputKind::Primary)
                {
                    let source = output
                        .source
                        .as_ref()
                        .expect("procedural output missing source");
                    let value = nodes
                        .get(source)
                        .unwrap_or_else(|| panic!("node '{}' not found", source));
                    let encoded = speccade_backend_texture::encode_graph_value_png(value);
                    assert!(
                        encoded.is_ok(),
                        "Failed to encode procedural output {:?}: {:?}",
                        spec_path,
                        encoded.err()
                    );
                }
            }
        }
    }
}

// ============================================================================
// Tier 2: Generation Tests (Blender backends)
// ============================================================================

/// Test static mesh generation with Blender.
#[test]
#[ignore] // Run with SPECCADE_RUN_BLENDER_TESTS=1
fn test_generate_static_mesh() {
    if !should_run_blender_tests() {
        println!("Blender tests not enabled, skipping");
        return;
    }

    if !is_blender_available() {
        println!("Blender not available, skipping");
        return;
    }

    let harness = TestHarness::new();

    let spec = Spec::builder("test-cube-01", AssetType::StaticMesh)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(
            OutputFormat::Glb,
            "meshes/test_cube.glb",
        ))
        .recipe(Recipe::new(
            "static_mesh.blender_primitives_v1",
            serde_json::json!({
                "base_primitive": "cube",
                "dimensions": [1.0, 1.0, 1.0],
                "modifiers": [],
                "uv_projection": "box",
                "material_slots": [],
                "export": {
                    "apply_modifiers": true,
                    "triangulate": true,
                    "include_normals": true,
                    "include_uvs": true,
                    "include_vertex_colors": false
                }
            }),
        ))
        .build();

    let result = speccade_backend_blender::static_mesh::generate(&spec, harness.path());
    assert!(
        result.is_ok(),
        "Static mesh generation failed: {:?}",
        result.err()
    );

    let gen_result = result.unwrap();
    assert!(gen_result.metrics.triangle_count.is_some());
}

/// Test static mesh generation with collision, navmesh analysis, and baking enabled.
#[test]
#[ignore] // Run with SPECCADE_RUN_BLENDER_TESTS=1
fn test_generate_static_mesh_with_collision_navmesh_baking() {
    if !should_run_blender_tests() {
        println!("Blender tests not enabled, skipping");
        return;
    }

    if !is_blender_available() {
        println!("Blender not available, skipping");
        return;
    }

    let harness = TestHarness::new();

    let spec = Spec::builder("test-static-advanced-01", AssetType::StaticMesh)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(
            OutputFormat::Glb,
            "meshes/test_static_advanced.glb",
        ))
        .recipe(Recipe::new(
            "static_mesh.blender_primitives_v1",
            serde_json::json!({
                "base_primitive": "cube",
                "dimensions": [1.0, 1.0, 1.0],
                "modifiers": [],
                "uv_projection": "smart",
                "material_slots": [{
                    "name": "mat0",
                    "base_color": [0.8, 0.8, 0.8, 1.0],
                    "metallic": 0.0,
                    "roughness": 0.5
                }],
                "export": {
                    "apply_modifiers": true,
                    "triangulate": true,
                    "include_normals": true,
                    "include_uvs": true,
                    "include_vertex_colors": false
                },
                "collision_mesh": {
                    "collision_type": "box",
                    "output_suffix": "_col"
                },
                "navmesh": {
                    "walkable_slope_max": 45.0,
                    "stair_detection": false
                },
                "baking": {
                    "bake_types": ["normal"],
                    "ray_distance": 0.1,
                    "margin": 2,
                    "resolution": [32, 32]
                }
            }),
        ))
        .build();

    let result = speccade_backend_blender::static_mesh::generate(&spec, harness.path());
    assert!(
        result.is_ok(),
        "Static mesh generation failed: {:?}",
        result.err()
    );

    let gen_result = result.unwrap();
    assert!(gen_result.metrics.collision_mesh.is_some());
    assert!(gen_result.metrics.collision_mesh_path.is_some());
    assert!(gen_result.metrics.navmesh.is_some());
    assert!(gen_result.metrics.baking.is_some());

    // baking_metrics should include at least one baked map entry.
    let baked_maps = gen_result
        .metrics
        .baking
        .as_ref()
        .unwrap()
        .baked_maps
        .as_slice();
    assert!(!baked_maps.is_empty());

    // Primary output exists
    let primary = harness
        .path()
        .join("meshes")
        .join("test_static_advanced.glb");
    assert!(
        primary.exists(),
        "Primary GLB missing: {}",
        primary.display()
    );

    // Collision mesh output exists (suffix-based)
    let collision = harness
        .path()
        .join("meshes")
        .join("test_static_advanced_col.glb");
    assert!(
        collision.exists(),
        "Collision GLB missing: {}",
        collision.display()
    );

    // Baked normal map output exists (asset_id-based)
    let baked = harness
        .path()
        .join("meshes")
        .join("test-static-advanced-01_normal.png");
    assert!(baked.exists(), "Baked texture missing: {}", baked.display());
}

/// Test static mesh generation with LOD chain enabled surfaces per-LOD metrics.
#[test]
#[ignore] // Run with SPECCADE_RUN_BLENDER_TESTS=1
fn test_generate_static_mesh_with_lod_chain_metrics() {
    if !should_run_blender_tests() {
        println!("Blender tests not enabled, skipping");
        return;
    }

    if !is_blender_available() {
        println!("Blender not available, skipping");
        return;
    }

    let harness = TestHarness::new();

    let spec = Spec::builder("test-static-lod-01", AssetType::StaticMesh)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(
            OutputFormat::Glb,
            "meshes/test_static_lod.glb",
        ))
        .recipe(Recipe::new(
            "static_mesh.blender_primitives_v1",
            serde_json::json!({
                "base_primitive": "cube",
                "dimensions": [1.0, 1.0, 1.0],
                "modifiers": [{
                    "type": "subdivision",
                    "levels": 3,
                    "render_levels": 3
                }],
                "uv_projection": {
                    "method": "smart",
                    "texel_density": 256.0
                },
                "material_slots": [],
                "lod_chain": {
                    "levels": [
                        { "level": 0 },
                        { "level": 1, "target_tris": 200 }
                    ],
                    "decimate_method": "collapse"
                },
                "export": {
                    "apply_modifiers": true,
                    "triangulate": true,
                    "include_normals": true,
                    "include_uvs": true,
                    "include_vertex_colors": false
                }
            }),
        ))
        .build();

    let result = speccade_backend_blender::static_mesh::generate(&spec, harness.path());
    assert!(
        result.is_ok(),
        "Static mesh generation failed: {:?}",
        result.err()
    );

    let gen_result = result.unwrap();
    assert_eq!(gen_result.metrics.lod_count, Some(2));

    let levels = gen_result
        .metrics
        .lod_levels
        .as_ref()
        .expect("missing lod_levels");
    assert_eq!(levels.len(), 2);
    assert_eq!(levels[0].lod_level, 0);
    assert_eq!(levels[1].lod_level, 1);

    // Ensure texel density and UV layer count were computed on LOD0 metrics.
    assert!(levels[0].uv_layer_count.is_some());
    assert!(levels[0].texel_density.is_some());
}

/// Test skeletal mesh generation with Blender.
#[test]
#[ignore] // Run with SPECCADE_RUN_BLENDER_TESTS=1
fn test_generate_skeletal_mesh() {
    if !should_run_blender_tests() {
        println!("Blender tests not enabled, skipping");
        return;
    }

    if !is_blender_available() {
        println!("Blender not available, skipping");
        return;
    }

    let harness = TestHarness::new();

    // Load a golden skeletal mesh spec
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping");
        return;
    }

    let specs = GoldenFixtures::list_speccade_specs("skeletal_mesh");
    if specs.is_empty() {
        println!("No skeletal mesh specs found, skipping");
        return;
    }

    let spec = parse_spec_file(&specs[0]).expect("Failed to parse spec");

    let result = speccade_backend_blender::skeletal_mesh::generate(&spec, harness.path());
    assert!(
        result.is_ok(),
        "Skeletal mesh generation failed: {:?}",
        result.err()
    );

    let gen_result = result.unwrap();
    assert!(gen_result.metrics.bone_count.is_some());
}

/// Full-features smoke test for `skeletal_mesh.armature_driven_v1`.
///
/// This is intentionally ignored by default because it requires Blender.
#[test]
#[ignore] // Run with SPECCADE_RUN_BLENDER_TESTS=1
fn test_generate_skeletal_mesh_armature_driven_full_features_smoke() {
    if !should_run_blender_tests() {
        println!("Blender tests not enabled, skipping");
        return;
    }

    if !is_blender_available() {
        println!("Blender not available, skipping");
        return;
    }

    let harness = TestHarness::new();

    let spec = Spec::builder(
        "test-armature-driven-full-features-01",
        AssetType::SkeletalMesh,
    )
    .license("CC0-1.0")
    .seed(42)
    .output(OutputSpec::primary(
        OutputFormat::Glb,
        "characters/armature_driven_full_features.glb",
    ))
    .recipe(Recipe::new(
        "skeletal_mesh.armature_driven_v1",
        serde_json::json!({
            "skeleton_preset": "humanoid_basic_v1",
            "material_slots": [
                {
                    "name": "skin",
                    "base_color": [0.80, 0.60, 0.50, 1.0],
                    "metallic": 0.0,
                    "roughness": 0.75
                },
                {
                    "name": "metal",
                    "base_color": [0.65, 0.66, 0.70, 1.0],
                    "metallic": 1.0,
                    "roughness": 0.25,
                    "emissive": [0.05, 0.05, 0.05],
                    "emissive_strength": 1.5
                }
            ],
            "export": {
                "include_armature": true,
                "include_normals": true,
                "include_uvs": true,
                "triangulate": true,
                "include_skin_weights": true,
                "save_blend": false
            },
            "constraints": {
                "max_triangles": 20000,
                "max_bones": 64,
                "max_materials": 8
            },
            "bool_shapes": {
                "chest_cutout": {
                    "primitive": "cube",
                    "dimensions": [0.18, 0.12, 0.10],
                    "position": [0.0, 0.20, 0.35],
                    "bone": "chest"
                },
                "arm_socket_l": {
                    "primitive": "sphere",
                    "dimensions": [0.12, 0.12, 0.12],
                    "position": [0.10, 0.08, 0.12],
                    "bone": "shoulder_l"
                },
                "arm_socket_r": { "mirror": "arm_socket_l" }
            },
            "bone_meshes": {
                "spine": {
                    "profile": "circle(8)",
                    "profile_radius": 0.06,
                    "taper": 0.90,
                    "translate": [0.0, 0.0, 0.0],
                    "rotate": [0.0, 0.0, 0.0],
                    "bulge": [
                        { "at": 0.0, "scale": 1.20 },
                        { "at": 0.5, "scale": 1.00 },
                        { "at": 1.0, "scale": 0.95 }
                    ],
                    "twist": 10.0,
                    "cap_start": true,
                    "cap_end": true,
                    "modifiers": [
                        { "bevel": { "width": 0.01, "segments": 2 } },
                        { "subdivide": { "cuts": 1 } }
                    ],
                    "material_index": 0,
                    "attachments": [
                        {
                            "primitive": "cube",
                            "dimensions": [0.14, 0.10, 0.06],
                            "offset": [0.0, 0.03, 0.02],
                            "rotation": [0.0, 30.0, 0.0],
                            "material_index": 1
                        }
                    ]
                },
                "chest": {
                    "profile": "circle(10)",
                    "profile_radius": [0.07, 0.09],
                    "taper": 0.85,
                    "bulge": [
                        { "at": 0.0, "scale": 1.15 },
                        { "at": 0.7, "scale": 1.00 },
                        { "at": 1.0, "scale": 0.92 }
                    ],
                    "modifiers": [
                        { "bool": { "operation": "difference", "target": "chest_cutout" } },
                        { "bevel": { "width": 0.008, "segments": 1 } }
                    ],
                    "material_index": 0,
                    "attachments": [
                        {
                            "extrude": {
                                "profile": "circle(6)",
                                "start": [0.0, 0.0, 0.0],
                                "end": [0.0, 0.0, 0.25],
                                "profile_radius": { "absolute": 0.015 },
                                "taper": 0.4
                            }
                        }
                    ]
                },
                "upper_arm_l": {
                    "profile": "hexagon(6)",
                    "profile_radius": 0.05,
                    "taper": 0.80,
                    "twist": -15.0,
                    "cap_start": true,
                    "cap_end": true,
                    "modifiers": [
                        { "subdivide": { "cuts": 1 } }
                    ],
                    "material_index": 0
                },
                "upper_arm_r": { "mirror": "upper_arm_l" },
                "lower_arm_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.045,
                    "taper": 0.75,
                    "rotate": [0.0, 0.0, 20.0],
                    "modifiers": [
                        { "bevel": { "width": 0.006, "segments": 1 } }
                    ],
                    "material_index": 0
                },
                "lower_arm_r": { "mirror": "lower_arm_l" },
                "upper_leg_l": {
                    "profile": "circle(10)",
                    "profile_radius": 0.07,
                    "taper": 0.78,
                    "bulge": [
                        { "at": 0.2, "scale": 1.10 },
                        { "at": 0.8, "scale": 0.95 }
                    ],
                    "material_index": 0
                },
                "upper_leg_r": { "mirror": "upper_leg_l" }
            }
        }),
    ))
    .build();

    let result = speccade_backend_blender::skeletal_mesh::generate(&spec, harness.path());
    assert!(
        result.is_ok(),
        "Armature-driven skeletal mesh generation failed: {:?}",
        result.err()
    );

    let gen_result = result.unwrap();

    let expected_out = harness
        .path()
        .join("characters")
        .join("armature_driven_full_features.glb");
    assert!(
        expected_out.exists(),
        "Primary GLB missing: {}",
        expected_out.display()
    );

    let validation = validate_glb_file(&expected_out);
    assert!(
        validation.is_ok(),
        "GLB validation failed: {:?}",
        validation.err()
    );

    assert!(
        gen_result.metrics.bone_count.is_some(),
        "Expected bone_count metric"
    );
    assert!(
        gen_result.metrics.material_slot_count.is_some(),
        "Expected material_slot_count metric"
    );
    assert!(
        gen_result.metrics.uv_layer_count.is_some(),
        "Expected uv_layer_count metric"
    );

    let uv_layers = gen_result.metrics.uv_layer_count.unwrap_or(0);
    assert!(uv_layers > 0, "Expected uv_layer_count > 0");

    let unweighted = gen_result
        .metrics
        .unweighted_vertex_count
        .unwrap_or(u32::MAX);
    assert_eq!(unweighted, 0, "Expected unweighted_vertex_count == 0");

    let max_influences = gen_result.metrics.max_bone_influences.unwrap_or(0);
    assert_eq!(max_influences, 1, "Expected max_bone_influences == 1");
}

/// `translate` / `rotate` on armature-driven bone meshes should affect output bounds.
///
/// This is intentionally ignored by default because it requires Blender.
#[test]
#[ignore] // Run with SPECCADE_RUN_BLENDER_TESTS=1
fn test_generate_skeletal_mesh_armature_driven_translate_rotate_affects_bounds() {
    if !should_run_blender_tests() {
        println!("Blender tests not enabled, skipping");
        return;
    }

    if !is_blender_available() {
        println!("Blender not available, skipping");
        return;
    }

    let base_harness = TestHarness::new();
    let base_spec = Spec::builder(
        "test-armature-driven-bounds-baseline-01",
        AssetType::SkeletalMesh,
    )
    .license("CC0-1.0")
    .seed(1)
    .output(OutputSpec::primary(
        OutputFormat::Glb,
        "characters/armature_driven_bounds_baseline.glb",
    ))
    .recipe(Recipe::new(
        "skeletal_mesh.armature_driven_v1",
        serde_json::json!({
            "skeleton_preset": "humanoid_basic_v1",
            "export": {
                "include_armature": true,
                "include_normals": true,
                "include_uvs": true,
                "triangulate": true,
                "include_skin_weights": true,
                "save_blend": false
            },
            "bone_meshes": {
                "spine": {
                    "profile": "rectangle",
                    "profile_radius": [0.14, 0.06],
                    "cap_start": true,
                    "cap_end": true
                }
            }
        }),
    ))
    .build();

    let base_result =
        speccade_backend_blender::skeletal_mesh::generate(&base_spec, base_harness.path());
    assert!(
        base_result.is_ok(),
        "Armature-driven skeletal mesh generation failed: {:?}",
        base_result.err()
    );
    let base_result = base_result.unwrap();

    assert!(
        base_result.metrics.bounds_min.is_some(),
        "Expected bounds_min metric"
    );
    assert!(
        base_result.metrics.bounds_max.is_some(),
        "Expected bounds_max metric"
    );
    let base_bounds_min = base_result.metrics.bounds_min.unwrap();
    let base_bounds_max = base_result.metrics.bounds_max.unwrap();

    let moved_harness = TestHarness::new();
    let moved_spec = Spec::builder(
        "test-armature-driven-bounds-moved-01",
        AssetType::SkeletalMesh,
    )
    .license("CC0-1.0")
    .seed(1)
    .output(OutputSpec::primary(
        OutputFormat::Glb,
        "characters/armature_driven_bounds_moved.glb",
    ))
    .recipe(Recipe::new(
        "skeletal_mesh.armature_driven_v1",
        serde_json::json!({
            "skeleton_preset": "humanoid_basic_v1",
            "export": {
                "include_armature": true,
                "include_normals": true,
                "include_uvs": true,
                "triangulate": true,
                "include_skin_weights": true,
                "save_blend": false
            },
            "bone_meshes": {
                "spine": {
                    "profile": "rectangle",
                    "profile_radius": [0.14, 0.06],
                    "cap_start": true,
                    "cap_end": true,
                    "translate": [0.75, 0.0, 0.0],
                    "rotate": [10.0, 0.0, 35.0]
                }
            }
        }),
    ))
    .build();

    let moved_result =
        speccade_backend_blender::skeletal_mesh::generate(&moved_spec, moved_harness.path());
    assert!(
        moved_result.is_ok(),
        "Armature-driven skeletal mesh generation failed: {:?}",
        moved_result.err()
    );
    let moved_result = moved_result.unwrap();

    assert!(
        moved_result.metrics.bounds_min.is_some(),
        "Expected bounds_min metric"
    );
    assert!(
        moved_result.metrics.bounds_max.is_some(),
        "Expected bounds_max metric"
    );
    let moved_bounds_min = moved_result.metrics.bounds_min.unwrap();
    let moved_bounds_max = moved_result.metrics.bounds_max.unwrap();

    assert_ne!(
        base_bounds_min, moved_bounds_min,
        "Expected bounds_min to differ between baseline and translated/rotated specs"
    );
    assert_ne!(
        base_bounds_max, moved_bounds_max,
        "Expected bounds_max to differ between baseline and translated/rotated specs"
    );
}

/// Test animation generation with Blender.
#[test]
#[ignore] // Run with SPECCADE_RUN_BLENDER_TESTS=1
fn test_generate_animation() {
    if !should_run_blender_tests() {
        println!("Blender tests not enabled, skipping");
        return;
    }

    if !is_blender_available() {
        println!("Blender not available, skipping");
        return;
    }

    let harness = TestHarness::new();

    // Load a golden animation spec
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping");
        return;
    }

    let specs = GoldenFixtures::list_speccade_specs("skeletal_animation");
    if specs.is_empty() {
        println!("No skeletal animation specs found, skipping");
        return;
    }

    let spec = parse_spec_file(&specs[0]).expect("Failed to parse spec");

    let result = speccade_backend_blender::animation::generate(&spec, harness.path());
    assert!(
        result.is_ok(),
        "Animation generation failed: {:?}",
        result.err()
    );

    let gen_result = result.unwrap();
    assert!(gen_result.metrics.animation_frame_count.is_some());
}

/// Test rigged (IK-aware) animation generation with Blender.
#[test]
#[ignore] // Run with SPECCADE_RUN_BLENDER_TESTS=1
fn test_generate_rigged_animation_motion_metrics() {
    if !should_run_blender_tests() {
        println!("Blender tests not enabled, skipping");
        return;
    }

    if !is_blender_available() {
        println!("Blender not available, skipping");
        return;
    }

    // Load a golden rigged animation spec
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping");
        return;
    }

    let spec_path = GoldenFixtures::speccade_specs_dir()
        .join("skeletal_animation")
        .join("walk_cycle_ik.json");
    if !spec_path.exists() {
        println!("Rigged animation fixture not found, skipping");
        return;
    }

    let harness = TestHarness::new();
    let spec = parse_spec_file(&spec_path).expect("Failed to parse rigged animation spec");

    let result = speccade_backend_blender::rigged_animation::generate(&spec, harness.path());
    assert!(
        result.is_ok(),
        "Rigged animation generation failed: {:?}",
        result.err()
    );

    let gen_result = result.unwrap();

    // Motion verification metrics (MESHVER-005) should be present in reports.
    assert!(gen_result.metrics.hinge_axis_violations.is_some());
    assert!(gen_result.metrics.range_violations.is_some());
    assert!(gen_result.metrics.velocity_spikes.is_some());
    assert!(gen_result.metrics.root_motion_delta.is_some());
}

/// Test golden static mesh specs can be generated.
#[test]
#[ignore] // Run with SPECCADE_RUN_BLENDER_TESTS=1
fn test_generate_golden_static_meshes() {
    if !should_run_blender_tests() {
        println!("Blender tests not enabled, skipping");
        return;
    }

    if !is_blender_available() {
        println!("Blender not available, skipping");
        return;
    }

    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping");
        return;
    }

    let harness = TestHarness::new();
    let specs = GoldenFixtures::list_speccade_specs("static_mesh");

    for spec_path in specs.iter().take(2) {
        let spec = parse_spec_file(spec_path).expect("Failed to parse spec");

        if spec.recipe.is_none() {
            continue;
        }

        let result = speccade_backend_blender::static_mesh::generate(&spec, harness.path());
        assert!(
            result.is_ok(),
            "Failed to generate {:?}: {:?}",
            spec_path,
            result.err()
        );
    }
}
