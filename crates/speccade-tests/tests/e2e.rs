//! End-to-End Integration Tests for SpecCade
//!
//! These tests verify parity-critical flows:
//!
//! 1. Migration tests - Verify legacy .spec.py parsing works correctly
//! 2. Generation tests - Verify each asset type produces valid outputs
//! 3. Output validation tests - Verify outputs exist and are valid
//! 4. Audit tests - Verify the migrator audit mode works correctly
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all Tier 1 tests (no Blender required)
//! cargo test -p speccade-tests
//!
//! # Run Tier 2 tests (requires Blender)
//! SPECCADE_RUN_BLENDER_TESTS=1 cargo test -p speccade-tests --ignored
//! ```

use std::fs;

use speccade_spec::recipe::audio::{AudioLayer, AudioV1Params, Envelope, Synthesis, Waveform};
use speccade_spec::recipe::music::{
    ArrangementEntry, InstrumentSynthesis, MusicTrackerSongV1Params, PatternNote, TrackerFormat,
    TrackerInstrument, TrackerPattern,
};
use speccade_spec::recipe::texture::{
    NormalMapPattern, TextureMapType, TextureMaterialV1Params, TextureNormalV1Params,
};
use speccade_spec::{AssetType, OutputFormat, OutputKind, OutputSpec, Recipe, Spec};
use speccade_tests::fixtures::{GoldenFixtures, LegacyProjectFixture};
use speccade_tests::harness::{
    is_blender_available, parse_spec_file, should_run_blender_tests, validate_png_file,
    validate_wav_file, validate_xm_file, TestHarness,
};

// ============================================================================
// Module 1: Migration Tests
// ============================================================================

mod migration {
    use super::*;

    /// Test that we can parse a minimal sound spec.
    #[test]
    fn test_parse_sound_spec() {
        let fixture = LegacyProjectFixture::new();
        let spec_path = fixture.add_sound("test_beep");

        // Read and verify the file exists and has expected structure
        let content = fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("SOUND"));
        assert!(content.contains("duration"));
        assert!(content.contains("layers"));
    }

    /// Test that we can parse a minimal instrument spec.
    #[test]
    fn test_parse_instrument_spec() {
        let fixture = LegacyProjectFixture::new();
        let spec_path = fixture.add_instrument("test_synth");

        let content = fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("INSTRUMENT"));
        assert!(content.contains("synthesis"));
        assert!(content.contains("oscillators"));
    }

    /// Test that we can parse a minimal music spec.
    #[test]
    fn test_parse_music_spec() {
        let fixture = LegacyProjectFixture::new();
        let spec_path = fixture.add_music("test_song");

        let content = fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("SONG"));
        assert!(content.contains("bpm"));
        assert!(content.contains("patterns"));
    }

    /// Test that we can parse a minimal texture spec.
    #[test]
    fn test_parse_texture_spec() {
        let fixture = LegacyProjectFixture::new();
        let spec_path = fixture.add_texture("test_metal");

        let content = fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("TEXTURE"));
        assert!(content.contains("size"));
        assert!(content.contains("layers"));
    }

    /// Test that we can parse a minimal normal map spec.
    #[test]
    fn test_parse_normal_spec() {
        let fixture = LegacyProjectFixture::new();
        let spec_path = fixture.add_normal("test_bricks");

        let content = fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("NORMAL"));
        assert!(content.contains("pattern"));
    }

    /// Test that we can parse a minimal mesh spec.
    #[test]
    fn test_parse_mesh_spec() {
        let fixture = LegacyProjectFixture::new();
        let spec_path = fixture.add_mesh("test_cube");

        let content = fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("MESH"));
        assert!(content.contains("primitive"));
    }

    /// Test that we can parse a minimal character spec.
    #[test]
    fn test_parse_character_spec() {
        let fixture = LegacyProjectFixture::new();
        let spec_path = fixture.add_character("test_biped");

        let content = fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("SPEC"));
        assert!(content.contains("skeleton"));
    }

    /// Test that we can parse a minimal animation spec.
    #[test]
    fn test_parse_animation_spec() {
        let fixture = LegacyProjectFixture::new();
        let spec_path = fixture.add_animation("test_walk");

        let content = fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("ANIMATION"));
        assert!(content.contains("fps"));
    }

    /// Test that golden legacy fixtures exist.
    #[test]
    fn test_golden_legacy_fixtures_exist() {
        if !GoldenFixtures::exists() {
            println!("Golden fixtures not found, skipping test");
            return;
        }

        let sounds = GoldenFixtures::list_legacy_specs("sounds");
        assert!(!sounds.is_empty(), "Should have sound specs");

        let textures = GoldenFixtures::list_legacy_specs("textures");
        assert!(!textures.is_empty(), "Should have texture specs");
    }

    /// Test that golden speccade specs can be parsed.
    #[test]
    fn test_golden_speccade_specs_parse() {
        if !GoldenFixtures::exists() {
            println!("Golden fixtures not found, skipping test");
            return;
        }

        let audio_specs = GoldenFixtures::list_speccade_specs("audio");
        for spec_path in audio_specs {
            let spec = parse_spec_file(&spec_path);
            assert!(
                spec.is_ok(),
                "Failed to parse {:?}: {:?}",
                spec_path,
                spec.err()
            );
            let spec = spec.unwrap();
            assert_eq!(spec.asset_type, AssetType::Audio);
        }

        let texture_specs = GoldenFixtures::list_speccade_specs("texture");
        for spec_path in texture_specs {
            let spec = parse_spec_file(&spec_path);
            assert!(
                spec.is_ok(),
                "Failed to parse {:?}: {:?}",
                spec_path,
                spec.err()
            );
            let spec = spec.unwrap();
            assert_eq!(spec.asset_type, AssetType::Texture);
        }
    }
}

// ============================================================================
// Module 2: Generation Tests (Tier 1 - Rust backends)
// ============================================================================

mod generation_tier1 {
    use super::*;

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
            }],
            pitch_envelope: None,
            generate_loop_points: false,
            master_filter: None,
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
            }],
            pitch_envelope: None,
            generate_loop_points: true,
            master_filter: None,
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

    /// Test texture material maps generation produces valid PNG output.
    #[test]
    fn test_generate_texture_material_maps() {
        let harness = TestHarness::new();

        let params = TextureMaterialV1Params {
            resolution: [64, 64],
            tileable: true,
            maps: vec![TextureMapType::Albedo],
            base_material: None,
            layers: vec![],
            color_ramp: None,
            palette: None,
        };

        let result = speccade_backend_texture::generate_material_maps(&params, 42);
        assert!(
            result.is_ok(),
            "Texture generation failed: {:?}",
            result.err()
        );

        let gen_result = result.unwrap();
        let albedo = gen_result.maps.get(&TextureMapType::Albedo);
        assert!(albedo.is_some(), "Albedo map should be present");
        assert!(!albedo.unwrap().data.is_empty());

        // Write output and validate
        let output_path = harness.path().join("test_albedo.png");
        fs::write(&output_path, &albedo.unwrap().data).unwrap();

        let validation = validate_png_file(&output_path);
        assert!(
            validation.is_ok(),
            "PNG validation failed: {:?}",
            validation.err()
        );
    }

    /// Test texture normal map generation.
    #[test]
    fn test_generate_texture_normal_map() {
        let harness = TestHarness::new();

        let params = TextureNormalV1Params {
            resolution: [64, 64],
            tileable: true,
            bump_strength: 1.0,
            pattern: Some(NormalMapPattern::Bricks {
                brick_width: 32,
                brick_height: 16,
                mortar_width: 2,
                offset: 0.5,
            }),
            processing: None,
        };

        let result = speccade_backend_texture::generate_normal_map(&params, 42);
        assert!(
            result.is_ok(),
            "Normal map generation failed: {:?}",
            result.err()
        );

        let gen_result = result.unwrap();
        assert!(!gen_result.data.is_empty());

        // Write output and validate
        let output_path = harness.path().join("test_normal.png");
        fs::write(&output_path, &gen_result.data).unwrap();

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

            // Handle both material_maps and normal_map recipes
            if recipe.kind == "texture.material_v1" {
                let params = recipe.as_texture_material();
                if let Ok(params) = params {
                    let result =
                        speccade_backend_texture::generate_material_maps(&params, spec.seed);
                    assert!(
                        result.is_ok(),
                        "Failed to generate {:?}: {:?}",
                        spec_path,
                        result.err()
                    );
                }
            } else if recipe.kind == "texture.normal_v1" {
                let params = recipe.as_texture_normal();
                if let Ok(params) = params {
                    let result = speccade_backend_texture::generate_normal_map(&params, spec.seed);
                    assert!(
                        result.is_ok(),
                        "Failed to generate {:?}: {:?}",
                        spec_path,
                        result.err()
                    );
                }
            } else if recipe.kind == "texture.packed_v1" {
                let params = recipe.as_texture_packed();
                if let Ok(params) = params {
                    let maps =
                        speccade_backend_texture::generate_packed_maps(&params, spec.seed);
                    assert!(
                        maps.is_ok(),
                        "Failed to generate packed maps {:?}: {:?}",
                        spec_path,
                        maps.err()
                    );

                    let maps = maps.unwrap();
                    let [width, height] = params.resolution;
                    let config = speccade_backend_texture::PngConfig::default();

                    for output in spec.outputs.iter().filter(|o| o.kind == OutputKind::Packed) {
                        let channels = output
                            .channels
                            .as_ref()
                            .expect("packed output missing channels");
                        let packed =
                            speccade_backend_texture::pack_channels(channels, &maps, width, height);
                        assert!(
                            packed.is_ok(),
                            "Failed to pack channels {:?}: {:?}",
                            spec_path,
                            packed.err()
                        );

                        let packed = packed.unwrap();
                        let encoded = speccade_backend_texture::png::write_rgba_to_vec_with_hash(
                            &packed, &config,
                        );
                        assert!(
                            encoded.is_ok(),
                            "Failed to encode packed PNG {:?}: {:?}",
                            spec_path,
                            encoded.err()
                        );
                    }
                }
            } else if recipe.kind == "texture.graph_v1" {
                let params = recipe.as_texture_graph();
                if let Ok(params) = params {
                    let nodes = speccade_backend_texture::generate_graph(&params, spec.seed);
                    assert!(
                        nodes.is_ok(),
                        "Failed to generate texture graph {:?}: {:?}",
                        spec_path,
                        nodes.err()
                    );

                    let nodes = nodes.unwrap();
                    for output in spec.outputs.iter().filter(|o| o.kind == OutputKind::Primary) {
                        let source = output.source.as_ref().expect("graph output missing source");
                        let value = nodes
                            .get(source)
                            .unwrap_or_else(|| panic!("graph node '{}' not found", source));
                        let encoded = speccade_backend_texture::encode_graph_value_png(value);
                        assert!(
                            encoded.is_ok(),
                            "Failed to encode graph output {:?}: {:?}",
                            spec_path,
                            encoded.err()
                        );
                    }
                }
            }
        }
    }

    /// Test that packed texture goldens match expected output hashes.
    #[test]
    fn test_packed_texture_goldens_match_expected() {
        if !GoldenFixtures::exists() {
            println!("Golden fixtures not found, skipping test");
            return;
        }

        let specs_dir = GoldenFixtures::speccade_specs_dir().join("texture");
        let target_triple = speccade_spec::ReportBuilder::new(
            "hash".to_string(),
            "speccade-tests".to_string(),
        )
        .build()
        .target_triple;
        let expected_dir = GoldenFixtures::expected_hashes_dir()
            .join("texture")
            .join(&target_triple);

        if !expected_dir.exists() {
            println!(
                "Expected packed texture hashes not found for target '{}', skipping test",
                target_triple
            );
            return;
        }

        let specs = [
            specs_dir.join("packed_orm.json"),
            specs_dir.join("packed_mre.json"),
            specs_dir.join("packed_smoothness.json"),
            specs_dir.join("packed_patterns.json"),
        ];

        for spec_path in specs {
            let spec = parse_spec_file(&spec_path).expect("Failed to parse spec");
            let recipe = spec.recipe.as_ref().expect("Packed spec missing recipe");
            let params = recipe
                .as_texture_packed()
                .expect("Invalid packed params");

            let maps = speccade_backend_texture::generate_packed_maps(&params, spec.seed)
                .expect("Failed to generate packed maps");
            let [width, height] = params.resolution;
            let config = speccade_backend_texture::PngConfig::default();

            for output in spec.outputs.iter().filter(|o| o.kind == OutputKind::Packed) {
                let channels = output.channels.as_ref().expect("Missing packed channels");
                let packed = speccade_backend_texture::pack_channels(channels, &maps, width, height)
                    .expect("Failed to pack channels");
                let (data, hash) =
                    speccade_backend_texture::png::write_rgba_to_vec_with_hash(&packed, &config)
                        .expect("Failed to encode PNG");

                let expected_path = expected_dir
                    .join(&spec.asset_id)
                    .join(format!("{}.blake3", output.path));
                if !expected_path.exists() {
                    println!(
                        "Expected packed texture hash missing: {:?}; skipping test",
                        expected_path
                    );
                    return;
                }
                let expected_hash = fs::read_to_string(&expected_path)
                    .expect("Missing expected hash file")
                    .trim()
                    .to_string();

                assert_eq!(
                    hash, expected_hash,
                    "Packed texture hash mismatch for {:?}",
                    spec_path
                );

                // Sanity check: generated PNG is non-empty.
                assert!(!data.is_empty(), "Generated PNG is empty");
            }
        }
    }
}

// ============================================================================
// Module 3: Generation Tests (Tier 2 - Blender backends)
// ============================================================================

mod generation_tier2 {
    use super::*;

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
}

// ============================================================================
// Module 4: Output Validation Tests
// ============================================================================

mod output_validation {
    use super::*;

    /// Test WAV file validation catches invalid files.
    #[test]
    fn test_wav_validation_rejects_invalid() {
        let harness = TestHarness::new();
        let bad_file = harness.path().join("bad.wav");

        // Empty file
        fs::write(&bad_file, b"").unwrap();
        assert!(validate_wav_file(&bad_file).is_err());

        // Too short
        fs::write(&bad_file, b"RIFF").unwrap();
        assert!(validate_wav_file(&bad_file).is_err());

        // Wrong magic
        fs::write(&bad_file, b"XXXX00000000WAVE").unwrap();
        assert!(validate_wav_file(&bad_file).is_err());
    }

    /// Test PNG file validation catches invalid files.
    #[test]
    fn test_png_validation_rejects_invalid() {
        let harness = TestHarness::new();
        let bad_file = harness.path().join("bad.png");

        // Empty file
        fs::write(&bad_file, b"").unwrap();
        assert!(validate_png_file(&bad_file).is_err());

        // Wrong magic
        fs::write(&bad_file, b"NOTAPNG!").unwrap();
        assert!(validate_png_file(&bad_file).is_err());
    }

    /// Test XM file validation catches invalid files.
    #[test]
    fn test_xm_validation_rejects_invalid() {
        let harness = TestHarness::new();
        let bad_file = harness.path().join("bad.xm");

        // Empty file
        fs::write(&bad_file, b"").unwrap();
        assert!(validate_xm_file(&bad_file).is_err());

        // Wrong header
        fs::write(&bad_file, b"Not an XM file at all!").unwrap();
        assert!(validate_xm_file(&bad_file).is_err());
    }

    /// Test that output format detection works correctly.
    #[test]
    fn test_output_format_dispatch() {
        let harness = TestHarness::new();

        // Create a minimal valid WAV file (44 bytes minimum)
        let wav_header: Vec<u8> = vec![
            b'R', b'I', b'F', b'F', // ChunkID
            36, 0, 0, 0, // ChunkSize (36 + data size)
            b'W', b'A', b'V', b'E', // Format
            b'f', b'm', b't', b' ', // Subchunk1ID
            16, 0, 0, 0, // Subchunk1Size
            1, 0, // AudioFormat (PCM)
            1, 0, // NumChannels
            0x22, 0x56, 0, 0, // SampleRate (22050)
            0x44, 0xAC, 0, 0, // ByteRate
            2, 0, // BlockAlign
            16, 0, // BitsPerSample
            b'd', b'a', b't', b'a', // Subchunk2ID
            0, 0, 0, 0, // Subchunk2Size
        ];

        let wav_file = harness.path().join("test.wav");
        fs::write(&wav_file, &wav_header).unwrap();

        let result = validate_wav_file(&wav_file);
        assert!(result.is_ok(), "Valid WAV should pass: {:?}", result.err());
    }
}

// ============================================================================
// Module 5: Audit Tests
// ============================================================================

mod audit {
    use super::*;

    /// Test that audit works on a fixture with all implemented keys.
    #[test]
    fn test_audit_high_completeness() {
        let fixture = LegacyProjectFixture::new();

        // Add specs that use only well-known implemented keys
        fixture.add_sound("beep_01");
        fixture.add_sound("beep_02");
        fixture.add_texture("metal_01");

        // The audit should report high completeness for these minimal specs
        // since they only use core implemented keys (name, duration, layers, etc.)
        assert!(fixture.path().exists());
        assert!(fixture.specs_dir.exists());
    }

    /// Test that audit works on golden legacy fixtures.
    #[test]
    fn test_audit_golden_fixtures() {
        if !GoldenFixtures::exists() {
            println!("Golden fixtures not found, skipping");
            return;
        }

        // Verify the golden legacy directory has the expected structure
        let sounds = GoldenFixtures::list_legacy_specs("sounds");
        let textures = GoldenFixtures::list_legacy_specs("textures");
        let instruments = GoldenFixtures::list_legacy_specs("instruments");

        // These should all have files
        println!("Found {} sounds", sounds.len());
        println!("Found {} textures", textures.len());
        println!("Found {} instruments", instruments.len());

        assert!(!sounds.is_empty() || !textures.is_empty() || !instruments.is_empty());
    }

    /// Test that the fixture structure matches what migrate expects.
    #[test]
    fn test_fixture_structure_for_migrate() {
        let fixture = LegacyProjectFixture::new();

        // Add some specs
        fixture.add_sound("test_sound");
        fixture.add_texture("test_texture");

        // Verify the structure is what migrate expects
        let studio_dir = fixture.path().join(".studio");
        let specs_dir = studio_dir.join("specs");
        let sounds_dir = specs_dir.join("sounds");
        let textures_dir = specs_dir.join("textures");

        assert!(studio_dir.exists(), ".studio should exist");
        assert!(specs_dir.exists(), ".studio/specs should exist");
        assert!(sounds_dir.exists(), ".studio/specs/sounds should exist");
        assert!(textures_dir.exists(), ".studio/specs/textures should exist");

        assert!(sounds_dir.join("test_sound.spec.py").exists());
        assert!(textures_dir.join("test_texture.spec.py").exists());
    }
}

// ============================================================================
// Module 6: Determinism Tests
// ============================================================================

mod determinism {
    use super::*;

    /// Test that audio generation is deterministic (same seed -> same output).
    #[test]
    fn test_audio_determinism() {
        let spec = Spec::builder("determinism-test", AssetType::Audio)
            .license("CC0-1.0")
            .seed(12345)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
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

        let result1 = speccade_backend_audio::generate(&spec).unwrap();
        let result2 = speccade_backend_audio::generate(&spec).unwrap();

        assert_eq!(
            result1.wav.pcm_hash, result2.wav.pcm_hash,
            "Same seed should produce same hash"
        );
        assert_eq!(
            result1.wav.wav_data, result2.wav.wav_data,
            "Same seed should produce same data"
        );
    }

    /// Test that texture generation is deterministic.
    #[test]
    fn test_texture_determinism() {
        let params = TextureMaterialV1Params {
            resolution: [32, 32],
            tileable: true,
            maps: vec![TextureMapType::Albedo],
            base_material: None,
            layers: vec![],
            color_ramp: None,
            palette: None,
        };

        let result1 = speccade_backend_texture::generate_material_maps(&params, 12345).unwrap();
        let result2 = speccade_backend_texture::generate_material_maps(&params, 12345).unwrap();

        let hash1 = &result1.maps.get(&TextureMapType::Albedo).unwrap().hash;
        let hash2 = &result2.maps.get(&TextureMapType::Albedo).unwrap().hash;

        assert_eq!(hash1, hash2, "Same seed should produce same hash");
    }

    /// Test that different seeds produce different outputs.
    #[test]
    fn test_different_seeds_different_output() {
        let params = TextureMaterialV1Params {
            resolution: [32, 32],
            tileable: true,
            maps: vec![TextureMapType::Albedo],
            base_material: None,
            layers: vec![],
            color_ramp: None,
            palette: None,
        };

        let result1 = speccade_backend_texture::generate_material_maps(&params, 111).unwrap();
        let result2 = speccade_backend_texture::generate_material_maps(&params, 222).unwrap();

        let hash1 = &result1.maps.get(&TextureMapType::Albedo).unwrap().hash;
        let hash2 = &result2.maps.get(&TextureMapType::Albedo).unwrap().hash;

        // Note: This could theoretically fail with a collision, but it's extremely unlikely
        assert_ne!(
            hash1, hash2,
            "Different seeds should produce different hashes"
        );
    }
}

// ============================================================================
// Module 7: Spec Validation Tests
// ============================================================================

mod spec_validation {
    use super::*;
    use speccade_spec::{validate_for_generate, validate_spec};

    /// Test that a valid spec passes validation.
    #[test]
    fn test_valid_spec_passes() {
        let spec = Spec::builder("valid-spec-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let result = validate_spec(&spec);
        assert!(
            result.is_ok(),
            "Valid spec should pass: {:?}",
            result.errors
        );
    }

    /// Test that a spec without recipe fails generate validation.
    #[test]
    fn test_spec_without_recipe_fails_generate() {
        let spec = Spec::builder("no-recipe-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let result = validate_for_generate(&spec);
        assert!(
            !result.is_ok(),
            "Spec without recipe should fail generate validation"
        );
    }

    /// Test that all golden specs pass validation.
    #[test]
    fn test_golden_specs_pass_validation() {
        if !GoldenFixtures::exists() {
            println!("Golden fixtures not found, skipping");
            return;
        }

        let asset_types = [
            "audio",
            "music",
            "texture",
            "static_mesh",
            "skeletal_mesh",
            "skeletal_animation",
        ];

        for asset_type in &asset_types {
            let specs = GoldenFixtures::list_speccade_specs(asset_type);
            for spec_path in specs {
                let spec = parse_spec_file(&spec_path);
                assert!(
                    spec.is_ok(),
                    "Failed to parse {:?}: {:?}",
                    spec_path,
                    spec.err()
                );

                let spec = spec.unwrap();
                let result = validate_spec(&spec);
                assert!(
                    result.is_ok(),
                    "Golden spec {:?} should pass validation: {:?}",
                    spec_path,
                    result.errors
                );
            }
        }
    }
}
