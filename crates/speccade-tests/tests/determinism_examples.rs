//! Example determinism tests demonstrating the framework.
//!
//! These tests verify that asset generation produces byte-identical output
//! across multiple runs, which is a Tier 1 requirement for SpecCade.
//!
//! Run these tests with:
//! ```bash
//! cargo test -p speccade-tests --test determinism_examples
//! ```

use speccade_tests::determinism::verify_determinism;
use speccade_tests::test_determinism;

// ============================================================================
// Audio Determinism Tests
// ============================================================================

mod audio {
    use super::*;
    use speccade_backend_audio::generate_from_params;
    use speccade_spec::recipe::audio::{
        AudioLayer, AudioV1Params, Envelope, NoiseType, Synthesis, Waveform,
    };

    fn create_sine_params() -> AudioV1Params {
        AudioV1Params {
            base_note: None,
            duration_seconds: 0.1,
            sample_rate: 22050,
            master_filter: None,
            pitch_envelope: None,
            generate_loop_points: false,
            effects: vec![],
            layers: vec![AudioLayer {
                synthesis: Synthesis::Oscillator {
                    waveform: Waveform::Sine,
                    frequency: 440.0,
                    freq_sweep: None,
                    detune: None,
                    duty: None,
                },
                envelope: Envelope::default(),
                volume: 0.8,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
        }
    }

    fn create_noise_params() -> AudioV1Params {
        AudioV1Params {
            base_note: None,
            duration_seconds: 0.1,
            sample_rate: 22050,
            master_filter: None,
            pitch_envelope: None,
            generate_loop_points: false,
            effects: vec![],
            layers: vec![AudioLayer {
                synthesis: Synthesis::NoiseBurst {
                    noise_type: NoiseType::White,
                    filter: None,
                },
                envelope: Envelope::default(),
                volume: 0.8,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
        }
    }

    fn create_fm_params() -> AudioV1Params {
        AudioV1Params {
            base_note: None,
            duration_seconds: 0.2,
            sample_rate: 44100,
            master_filter: None,
            pitch_envelope: None,
            generate_loop_points: false,
            effects: vec![],
            layers: vec![AudioLayer {
                synthesis: Synthesis::FmSynth {
                    carrier_freq: 440.0,
                    modulator_freq: 880.0,
                    modulation_index: 2.5,
                    freq_sweep: None,
                },
                envelope: Envelope {
                    attack: 0.01,
                    decay: 0.05,
                    sustain: 0.5,
                    release: 0.1,
                },
                volume: 0.7,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
        }
    }

    #[test]
    fn test_sine_determinism() {
        let params = create_sine_params();
        let result = verify_determinism(
            || {
                generate_from_params(&params, 42)
                    .map(|r| r.wav.wav_data)
                    .unwrap_or_default()
            },
            5,
        );
        result.assert_deterministic();
    }

    #[test]
    fn test_noise_determinism() {
        // Noise uses RNG but should still be deterministic with same seed
        let params = create_noise_params();
        let result = verify_determinism(
            || {
                generate_from_params(&params, 12345)
                    .map(|r| r.wav.wav_data)
                    .unwrap_or_default()
            },
            5,
        );
        result.assert_deterministic();
    }

    #[test]
    fn test_fm_determinism() {
        let params = create_fm_params();
        let result = verify_determinism(
            || {
                generate_from_params(&params, 99)
                    .map(|r| r.wav.wav_data)
                    .unwrap_or_default()
            },
            3,
        );
        result.assert_deterministic();
    }

    #[test]
    fn test_different_seeds_produce_different_noise() {
        let params = create_noise_params();

        let result1 = generate_from_params(&params, 1).unwrap();
        let result2 = generate_from_params(&params, 2).unwrap();

        assert_ne!(
            result1.wav.wav_data, result2.wav.wav_data,
            "Different seeds should produce different noise output"
        );
    }

    // Using the test_determinism macro
    test_determinism!(macro_sine_test, {
        let params = AudioV1Params {
            base_note: None,
            duration_seconds: 0.05,
            sample_rate: 11025,
            master_filter: None,
            pitch_envelope: None,
            generate_loop_points: false,
            effects: vec![],
            layers: vec![AudioLayer {
                synthesis: Synthesis::Oscillator {
                    waveform: Waveform::Square,
                    frequency: 220.0,
                    freq_sweep: None,
                    detune: None,
                    duty: Some(0.5),
                },
                envelope: Envelope::default(),
                volume: 0.5,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
        };
        generate_from_params(&params, 42)
            .map(|r| r.wav.wav_data)
            .unwrap_or_default()
    });

    test_determinism!(macro_noise_5_runs, runs = 5, {
        let params = AudioV1Params {
            base_note: None,
            duration_seconds: 0.05,
            sample_rate: 11025,
            master_filter: None,
            pitch_envelope: None,
            generate_loop_points: false,
            effects: vec![],
            layers: vec![AudioLayer {
                synthesis: Synthesis::NoiseBurst {
                    noise_type: NoiseType::Pink,
                    filter: None,
                },
                envelope: Envelope::default(),
                volume: 0.5,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
        };
        generate_from_params(&params, 777)
            .map(|r| r.wav.wav_data)
            .unwrap_or_default()
    });
}

// ============================================================================
// Texture Determinism Tests
// ============================================================================

mod texture {
    use super::*;
    use speccade_backend_texture::generate_material_maps;
    use speccade_backend_texture::png::PngConfig;
    use speccade_backend_texture::{Color, TextureBuffer};
    use speccade_spec::recipe::texture::{
        BaseMaterial, MaterialType, TextureMapType, TextureMaterialV1Params,
    };

    fn create_metal_params() -> TextureMaterialV1Params {
        TextureMaterialV1Params {
            resolution: [64, 64],
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

    #[test]
    fn test_texture_albedo_determinism() {
        let params = create_metal_params();
        let result = verify_determinism(
            || {
                let tex_result = generate_material_maps(&params, 42).unwrap();
                tex_result
                    .maps
                    .get(&TextureMapType::Albedo)
                    .map(|m| m.data.clone())
                    .unwrap_or_default()
            },
            3,
        );
        result.assert_deterministic();
    }

    #[test]
    fn test_texture_normal_determinism() {
        let params = create_metal_params();
        let result = verify_determinism(
            || {
                let tex_result = generate_material_maps(&params, 42).unwrap();
                tex_result
                    .maps
                    .get(&TextureMapType::Normal)
                    .map(|m| m.data.clone())
                    .unwrap_or_default()
            },
            3,
        );
        result.assert_deterministic();
    }

    test_determinism!(macro_texture_roughness, {
        let params = TextureMaterialV1Params {
            resolution: [32, 32],
            tileable: false,
            maps: vec![TextureMapType::Roughness],
            base_material: Some(BaseMaterial {
                material_type: MaterialType::Stone,
                base_color: [0.5, 0.5, 0.5],
                roughness_range: Some([0.6, 0.9]),
                metallic: None,
                brick_pattern: None,
                normal_params: None,
            }),
            layers: vec![],
            palette: None,
            color_ramp: None,
        };
        let tex_result = generate_material_maps(&params, 123).unwrap();
        tex_result
            .maps
            .get(&TextureMapType::Roughness)
            .map(|m| m.data.clone())
            .unwrap_or_default()
    });

    // Test PNG encoding determinism using the texture backend's buffer types
    #[test]
    fn test_png_encoding_determinism() {
        let buffer = TextureBuffer::new(32, 32, Color::rgb(0.5, 0.3, 0.1));
        let config = PngConfig::default();

        let result = verify_determinism(
            || {
                speccade_backend_texture::png::write_rgba_to_vec_with_hash(&buffer, &config)
                    .map(|(data, _hash)| data)
                    .unwrap_or_default()
            },
            3,
        );
        result.assert_deterministic();
    }
}

// ============================================================================
// Hash Verification Tests
// ============================================================================

mod hash_verification {
    use speccade_tests::determinism::{compute_hash, verify_hash_determinism};

    #[test]
    fn test_hash_consistency() {
        let data = b"test data for hashing";

        let hashes: Vec<String> = (0..5).map(|_| compute_hash(data)).collect();

        assert!(verify_hash_determinism(&hashes));
    }

    #[test]
    fn test_hash_differs_for_different_data() {
        let hash1 = compute_hash(b"data A");
        let hash2 = compute_hash(b"data B");

        assert_ne!(hash1, hash2);
    }
}

// ============================================================================
// Framework API Tests
// ============================================================================

mod framework_api {
    use speccade_tests::determinism::{assert_deterministic, verify_determinism};

    #[test]
    fn test_verify_determinism_returns_detailed_result() {
        let result = verify_determinism(|| vec![1u8, 2, 3, 4, 5], 3);

        assert!(result.is_deterministic);
        assert_eq!(result.runs, 3);
        assert_eq!(result.output_size, 5);
        assert_eq!(result.hash.len(), 64); // BLAKE3 hash is 64 hex chars
        assert!(result.diff_info.is_none());
    }

    #[test]
    fn test_assert_deterministic_helper() {
        // Should not panic for deterministic output
        assert_deterministic(3, || vec![42u8; 100]);
    }

    #[test]
    #[should_panic(expected = "Non-deterministic output")]
    fn test_assert_deterministic_panics_on_non_determinism() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let counter = AtomicU32::new(0);

        assert_deterministic(3, || {
            let n = counter.fetch_add(1, Ordering::SeqCst);
            vec![n as u8]
        });
    }
}
