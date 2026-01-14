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
    is_blender_available, parse_spec_file, should_run_blender_tests, validate_png_file,
    validate_wav_file, validate_xm_file, TestHarness,
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
        generate_loop_points: false,
        master_filter: None,
        effects: vec![],
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
        generate_loop_points: true,
        master_filter: None,
        effects: vec![],
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
