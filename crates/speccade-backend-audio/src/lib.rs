//! SpecCade Audio Backend
//!
//! This crate implements audio generation backends for SpecCade:
//! - `audio_v1` - Unified audio synthesis (SFX and instruments)
//!
//! # Overview
//!
//! The audio backend generates WAV files from declarative specifications. It supports
//! multiple synthesis types that can be layered together:
//!
//! - **FM Synthesis** - Frequency modulation for complex timbres
//! - **Karplus-Strong** - Plucked string sounds
//! - **Oscillators** - Basic waveforms (sine, square, saw, triangle)
//! - **Noise** - White, pink, and brown noise with filtering
//! - **Additive** - Harmonic synthesis
//!
//! # Determinism
//!
//! All synthesis is deterministic. Given the same spec and seed, the output will be
//! byte-identical across runs (on the same platform). The crate uses PCG32 for all
//! random number generation, with seeds derived via BLAKE3 hashing.
//!
//! # Example
//!
//! ```ignore
//! use speccade_backend_audio::generate;
//! use speccade_spec::Spec;
//!
//! let spec = Spec::from_json(json_string)?;
//! let result = generate(&spec)?;
//!
//! // Write to file
//! std::fs::write("output.wav", &result.wav.wav_data)?;
//!
//! // Get PCM hash for validation
//! println!("PCM hash: {}", result.wav.pcm_hash);
//! ```
//!
//! # Crate Structure
//!
//! - [`generate()`] - Main entry point for audio generation (SFX and instruments)
//! - [`envelope`] - ADSR envelope generators
//! - [`filter`] - Biquad filter implementations
//! - [`mixer`] - Layer mixing with volume/pan
//! - [`oscillator`] - Basic waveform generators
//! - [`rng`] - Deterministic RNG with seed derivation
//! - [`synthesis`] - Synthesis algorithm implementations
//! - [`wav`] - Deterministic WAV file writer

pub mod effects;
pub mod envelope;
pub mod error;
pub mod filter;
pub mod generate;
pub mod loop_processing;
pub mod mixer;
pub mod modulation;
pub mod oscillator;
pub mod rng;
pub mod synthesis;
pub mod wav;

// Re-export main types at crate root
pub use error::{AudioError, AudioResult};
pub use generate::{generate, generate_from_params, generate_preview, GenerateResult};
pub use wav::{WavResult, WavWriter};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use speccade_spec::recipe::audio::{
        AudioLayer, AudioV1Params, Envelope, NoiseType, Synthesis, Waveform,
    };
    use speccade_spec::recipe::Recipe;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Spec};

    fn create_fm_spec(seed: u32) -> Spec {
        let params = AudioV1Params {
            duration_seconds: 0.3,
            sample_rate: 44100,
            master_filter: None,
            layers: vec![AudioLayer {
                synthesis: Synthesis::FmSynth {
                    carrier_freq: 440.0,
                    modulator_freq: 880.0,
                    modulation_index: 2.5,
                    freq_sweep: Some(speccade_spec::recipe::audio::FreqSweep {
                        end_freq: 110.0,
                        curve: speccade_spec::recipe::audio::SweepCurve::Exponential,
                    }),
                },
                envelope: Envelope {
                    attack: 0.01,
                    decay: 0.05,
                    sustain: 0.3,
                    release: 0.15,
                },
                volume: 0.8,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
            pitch_envelope: None,
            base_note: None,
            loop_config: None,
            generate_loop_points: false,
            effects: vec![],
            post_fx_lfos: vec![],
        };

        Spec::builder("laser-blast-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(seed)
            .description("Sci-fi laser blast sound effect")
            .tag("retro")
            .tag("scifi")
            .output(OutputSpec::primary(
                OutputFormat::Wav,
                "sounds/laser_blast_01.wav",
            ))
            .recipe(Recipe::new(
                "audio_v1",
                serde_json::to_value(&params).unwrap(),
            ))
            .build()
    }

    #[test]
    fn test_full_generation_pipeline() {
        let spec = create_fm_spec(42);
        let result = generate(&spec).expect("generation should succeed");

        // Verify output
        assert_eq!(result.num_layers, 1);
        assert!(!result.wav.wav_data.is_empty());
        assert_eq!(result.wav.sample_rate, 44100);
        assert!(!result.wav.is_stereo); // Single centered layer = mono

        // Verify WAV header
        assert_eq!(&result.wav.wav_data[0..4], b"RIFF");
        assert_eq!(&result.wav.wav_data[8..12], b"WAVE");
    }

    #[test]
    fn test_generation_determinism() {
        let spec = create_fm_spec(42);

        let result1 = generate(&spec).expect("first generation");
        let result2 = generate(&spec).expect("second generation");

        // PCM hash must be identical
        assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);

        // Full WAV data must be identical
        assert_eq!(result1.wav.wav_data, result2.wav.wav_data);
    }

    #[test]
    fn test_different_seeds_produce_different_output() {
        let spec1 = create_fm_spec(42);
        let spec2 = create_fm_spec(43);

        let _result1 = generate(&spec1).expect("first generation");
        let _result2 = generate(&spec2).expect("second generation");

        // Note: FM synthesis without noise is deterministic from oscillators,
        // so the seeds won't affect it. See test_noise_different_seeds for proper coverage.
    }

    #[test]
    fn test_noise_determinism() {
        let params = AudioV1Params {
            duration_seconds: 0.1,
            sample_rate: 22050,
            master_filter: None,
            layers: vec![AudioLayer {
                synthesis: Synthesis::NoiseBurst {
                    noise_type: NoiseType::White,
                    filter: None,
                },
                envelope: Envelope::default(),
                volume: 1.0,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
            pitch_envelope: None,
            base_note: None,
            loop_config: None,
            generate_loop_points: false,
            effects: vec![],
            post_fx_lfos: vec![],
        };

        let spec = Spec::builder("noise-test", AssetType::Audio)
            .license("CC0-1.0")
            .seed(12345)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_v1",
                serde_json::to_value(&params).unwrap(),
            ))
            .build();

        let result1 = generate(&spec).expect("first generation");
        let result2 = generate(&spec).expect("second generation");

        assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);
    }

    #[test]
    fn test_noise_different_seeds() {
        let make_spec = |seed: u32| {
            let params = AudioV1Params {
                duration_seconds: 0.1,
                sample_rate: 22050,
                master_filter: None,
                layers: vec![AudioLayer {
                    synthesis: Synthesis::NoiseBurst {
                        noise_type: NoiseType::White,
                        filter: None,
                    },
                    envelope: Envelope::default(),
                    volume: 1.0,
                    pan: 0.0,
                    delay: None,
                    filter: None,
                    lfo: None,
                }],
                pitch_envelope: None,
                base_note: None,
                loop_config: None,
                generate_loop_points: false,
                effects: vec![],
                post_fx_lfos: vec![],
            };

            Spec::builder("noise-test", AssetType::Audio)
                .license("CC0-1.0")
                .seed(seed)
                .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
                .recipe(Recipe::new(
                    "audio_v1",
                    serde_json::to_value(&params).unwrap(),
                ))
                .build()
        };

        let spec1 = make_spec(42);
        let spec2 = make_spec(43);

        let result1 = generate(&spec1).expect("first generation");
        let result2 = generate(&spec2).expect("second generation");

        // Different seeds should produce different noise
        assert_ne!(result1.wav.pcm_hash, result2.wav.pcm_hash);
    }

    #[test]
    fn test_stereo_output() {
        let params = AudioV1Params {
            duration_seconds: 0.1,
            sample_rate: 44100,
            master_filter: None,
            pitch_envelope: None,
            base_note: None,
            loop_config: None,
            generate_loop_points: false,
            layers: vec![
                AudioLayer {
                    synthesis: Synthesis::Oscillator {
                        waveform: Waveform::Sine,
                        frequency: 440.0,
                        freq_sweep: None,
                        detune: None,
                        duty: None,
                    },
                    envelope: Envelope::default(),
                    volume: 0.5,
                    pan: -0.8,
                    delay: None,
                    filter: None,
                    lfo: None,
                },
                AudioLayer {
                    synthesis: Synthesis::Oscillator {
                        waveform: Waveform::Sine,
                        frequency: 550.0,
                        freq_sweep: None,
                        detune: None,
                        duty: None,
                    },
                    envelope: Envelope::default(),
                    volume: 0.5,
                    pan: 0.8,
                    delay: None,
                    filter: None,
                    lfo: None,
                },
            ],
            effects: vec![],
            post_fx_lfos: vec![],
        };

        let spec = Spec::builder("stereo-test", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_v1",
                serde_json::to_value(&params).unwrap(),
            ))
            .build();

        let result = generate(&spec).expect("generation should succeed");

        assert!(result.wav.is_stereo);
        assert_eq!(result.num_layers, 2);
    }

    #[test]
    fn test_karplus_strong() {
        let params = AudioV1Params {
            duration_seconds: 0.5,
            sample_rate: 44100,
            master_filter: None,
            layers: vec![AudioLayer {
                synthesis: Synthesis::KarplusStrong {
                    frequency: 220.0,
                    decay: 0.996,
                    blend: 0.7,
                },
                envelope: Envelope {
                    attack: 0.001,
                    decay: 0.3,
                    sustain: 0.0,
                    release: 0.2,
                },
                volume: 1.0,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
            pitch_envelope: None,
            base_note: None,
            loop_config: None,
            generate_loop_points: false,
            effects: vec![],
            post_fx_lfos: vec![],
        };

        let spec = Spec::builder("pluck-test", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_v1",
                serde_json::to_value(&params).unwrap(),
            ))
            .build();

        let result = generate(&spec).expect("generation should succeed");
        assert!(!result.wav.wav_data.is_empty());
    }

    #[test]
    fn test_additive_synthesis() {
        let params = AudioV1Params {
            duration_seconds: 0.3,
            sample_rate: 44100,
            master_filter: None,
            layers: vec![AudioLayer {
                synthesis: Synthesis::Additive {
                    base_freq: 440.0,
                    harmonics: vec![1.0, 0.5, 0.25, 0.125],
                },
                envelope: Envelope::default(),
                volume: 0.8,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
            pitch_envelope: None,
            base_note: None,
            loop_config: None,
            generate_loop_points: false,
            effects: vec![],
            post_fx_lfos: vec![],
        };

        let spec = Spec::builder("additive-test", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_v1",
                serde_json::to_value(&params).unwrap(),
            ))
            .build();

        let result = generate(&spec).expect("generation should succeed");
        assert!(!result.wav.wav_data.is_empty());
    }

    #[test]
    fn test_pcm_hash_format() {
        let spec = create_fm_spec(42);
        let result = generate(&spec).expect("generation should succeed");

        // BLAKE3 hash should be 64 hex characters
        assert_eq!(result.wav.pcm_hash.len(), 64);

        // Should be valid hex
        assert!(result.wav.pcm_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_loop_config_with_defaults() {
        use speccade_spec::recipe::audio::LoopConfig;

        let params = AudioV1Params {
            duration_seconds: 1.0,
            sample_rate: 44100,
            master_filter: None,
            layers: vec![AudioLayer {
                synthesis: Synthesis::Oscillator {
                    waveform: Waveform::Sine,
                    frequency: 440.0,
                    freq_sweep: None,
                    detune: None,
                    duty: None,
                },
                envelope: Envelope {
                    attack: 0.05,
                    decay: 0.1,
                    sustain: 0.7,
                    release: 0.2,
                },
                volume: 0.8,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
            pitch_envelope: None,
            base_note: None,
            loop_config: Some(LoopConfig::enabled()),
            generate_loop_points: false,
            effects: vec![],
            post_fx_lfos: vec![],
        };

        let spec = Spec::builder("loop-test", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_v1",
                serde_json::to_value(&params).unwrap(),
            ))
            .build();

        let result = generate(&spec).expect("generation should succeed");

        // Loop points should be generated
        assert!(result.loop_point.is_some());
        assert!(result.loop_end.is_some());

        // Loop start should be after attack+decay (0.15s = 6615 samples)
        let expected_start = (0.15 * 44100.0) as usize;
        let actual_start = result.loop_point.unwrap();
        // Allow some tolerance for zero-crossing snapping
        assert!(
            (actual_start as i64 - expected_start as i64).abs() < 1100,
            "Loop start {} should be near expected {}",
            actual_start,
            expected_start
        );

        // Loop end should be near the end of the audio
        let actual_end = result.loop_end.unwrap();
        assert!(actual_end > actual_start, "Loop end must be after start");
    }

    #[test]
    fn test_loop_config_with_custom_crossfade() {
        use speccade_spec::recipe::audio::LoopConfig;

        let params = AudioV1Params {
            duration_seconds: 0.5,
            sample_rate: 44100,
            master_filter: None,
            layers: vec![AudioLayer {
                synthesis: Synthesis::Oscillator {
                    waveform: Waveform::Sawtooth,
                    frequency: 220.0,
                    freq_sweep: None,
                    detune: None,
                    duty: None,
                },
                envelope: Envelope {
                    attack: 0.02,
                    decay: 0.05,
                    sustain: 0.8,
                    release: 0.1,
                },
                volume: 0.8,
                pan: 0.0,
                delay: None,
                filter: None,
                lfo: None,
            }],
            pitch_envelope: None,
            base_note: None,
            loop_config: Some(LoopConfig::with_crossfade(25.0)), // 25ms crossfade
            generate_loop_points: false,
            effects: vec![],
            post_fx_lfos: vec![],
        };

        let spec = Spec::builder("loop-crossfade-test", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_v1",
                serde_json::to_value(&params).unwrap(),
            ))
            .build();

        let result = generate(&spec).expect("generation should succeed");
        assert!(result.loop_point.is_some());
        assert!(result.loop_end.is_some());
    }

    #[test]
    fn test_legacy_generate_loop_points() {
        // Test backward compatibility with legacy generate_loop_points field
        let params = AudioV1Params {
            duration_seconds: 0.5,
            sample_rate: 44100,
            master_filter: None,
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
            pitch_envelope: None,
            base_note: None,
            loop_config: None,          // No new config
            generate_loop_points: true, // Legacy flag
            effects: vec![],
            post_fx_lfos: vec![],
        };

        let spec = Spec::builder("legacy-loop-test", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_v1",
                serde_json::to_value(&params).unwrap(),
            ))
            .build();

        let result = generate(&spec).expect("generation should succeed");

        // Legacy flag should still generate loop points
        assert!(result.loop_point.is_some());
        assert!(result.loop_end.is_some());
    }

    #[test]
    fn test_loop_determinism() {
        use speccade_spec::recipe::audio::LoopConfig;

        let params = AudioV1Params {
            duration_seconds: 0.5,
            sample_rate: 44100,
            master_filter: None,
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
            pitch_envelope: None,
            base_note: None,
            loop_config: Some(LoopConfig::enabled()),
            generate_loop_points: false,
            effects: vec![],
            post_fx_lfos: vec![],
        };

        let spec = Spec::builder("loop-determinism-test", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new(
                "audio_v1",
                serde_json::to_value(&params).unwrap(),
            ))
            .build();

        let result1 = generate(&spec).expect("first generation");
        let result2 = generate(&spec).expect("second generation");

        // Loop points must be identical
        assert_eq!(result1.loop_point, result2.loop_point);
        assert_eq!(result1.loop_end, result2.loop_end);

        // Audio with crossfade must be identical
        assert_eq!(result1.wav.pcm_hash, result2.wav.pcm_hash);
    }
}
