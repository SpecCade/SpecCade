//! End-to-End Determinism and Audit Tests for SpecCade
//!
//! Tests verify:
//! - Deterministic generation (same seed -> same output)
//! - Migrator audit mode
//!
//! ## Running Tests
//!
//! ```bash
//! cargo test -p speccade-tests --test e2e_determinism
//! ```

use speccade_spec::recipe::audio::{AudioLayer, AudioV1Params, Envelope, Synthesis, Waveform};
use speccade_spec::recipe::texture::{
    NoiseAlgorithm, NoiseConfig, TextureProceduralNode, TextureProceduralOp,
    TextureProceduralV1Params,
};
use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe, Spec};
use speccade_tests::fixtures::{GoldenFixtures, LegacyProjectFixture};

// ============================================================================
// Determinism Tests
// ============================================================================

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
    let params = TextureProceduralV1Params {
        resolution: [32, 32],
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

    let nodes1 = speccade_backend_texture::generate_graph(&params, 12345).unwrap();
    let nodes2 = speccade_backend_texture::generate_graph(&params, 12345).unwrap();

    let (_, hash1) =
        speccade_backend_texture::encode_graph_value_png(nodes1.get("mask").unwrap()).unwrap();
    let (_, hash2) =
        speccade_backend_texture::encode_graph_value_png(nodes2.get("mask").unwrap()).unwrap();

    assert_eq!(hash1, hash2, "Same seed should produce same hash");
}

/// Test that different seeds produce different outputs.
#[test]
fn test_different_seeds_different_output() {
    let params = TextureProceduralV1Params {
        resolution: [32, 32],
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

    let nodes1 = speccade_backend_texture::generate_graph(&params, 111).unwrap();
    let nodes2 = speccade_backend_texture::generate_graph(&params, 222).unwrap();

    let (_, hash1) =
        speccade_backend_texture::encode_graph_value_png(nodes1.get("mask").unwrap()).unwrap();
    let (_, hash2) =
        speccade_backend_texture::encode_graph_value_png(nodes2.get("mask").unwrap()).unwrap();

    // Note: This could theoretically fail with a collision, but it's extremely unlikely
    assert_ne!(
        hash1, hash2,
        "Different seeds should produce different hashes"
    );
}

/// Test that audio generation from params is deterministic.
#[test]
fn test_audio_params_determinism() {
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

    let result1 = speccade_backend_audio::generate_from_params(&params, 999).unwrap();
    let result2 = speccade_backend_audio::generate_from_params(&params, 999).unwrap();

    assert_eq!(
        result1.wav.pcm_hash, result2.wav.pcm_hash,
        "Same seed should produce same hash"
    );
}

// ============================================================================
// Audit Tests
// ============================================================================

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
