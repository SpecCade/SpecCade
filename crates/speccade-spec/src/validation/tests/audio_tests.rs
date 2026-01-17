//! Audio validation tests.

use crate::output::{OutputFormat, OutputSpec};
use crate::recipe::Recipe;
use crate::spec::AssetType;
use crate::validation::*;

fn make_valid_spec() -> crate::spec::Spec {
    crate::spec::Spec::builder("test-asset-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .description("Test asset")
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .build()
}

#[test]
fn test_audio_requires_wav_primary_output() {
    let mut spec = make_valid_spec();
    spec.outputs = vec![OutputSpec::primary(OutputFormat::Xm, "sounds/test.xm")];
    spec.recipe = Some(crate::recipe::Recipe::new(
        "audio_v1",
        serde_json::json!({
            "duration_seconds": 0.1,
            "layers": []
        }),
    ));

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::OutputValidationFailed));
}

#[test]
fn test_audio_lfo_rejects_depth_out_of_range() {
    let spec = crate::spec::Spec::builder("test-audio-lfo-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 2.0 },
                            "target": { "target": "volume", "amount": 1.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams));
}

#[test]
fn test_audio_lfo_allows_pitch_lfo_on_non_oscillator() {
    let spec = crate::spec::Spec::builder("test-audio-lfo-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "fm_synth", "carrier_freq": 440.0, "modulator_freq": 880.0, "modulation_index": 2.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 1.0 },
                            "target": { "target": "pitch", "semitones": 1.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_audio_lfo_rejects_filter_cutoff_lfo_without_filter() {
    let spec = crate::spec::Spec::builder("test-audio-lfo-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 1.0 },
                            "target": { "target": "filter_cutoff", "amount": 100.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("filter_cutoff LFO requires")));
}

#[test]
fn test_audio_lfo_allows_fm_index_on_fm_synth() {
    let spec = crate::spec::Spec::builder("test-audio-fm-index-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "fm_synth", "carrier_freq": 440.0, "modulator_freq": 880.0, "modulation_index": 4.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 1.0 },
                            "target": { "target": "fm_index", "amount": 2.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_audio_lfo_rejects_fm_index_on_non_fm_synth() {
    let spec = crate::spec::Spec::builder("test-audio-fm-index-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 1.0 },
                            "target": { "target": "fm_index", "amount": 2.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("fm_index LFO target is only valid for FmSynth")));
}

#[test]
fn test_audio_lfo_rejects_fm_index_with_zero_amount() {
    let spec = crate::spec::Spec::builder("test-audio-fm-index-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "fm_synth", "carrier_freq": 440.0, "modulator_freq": 880.0, "modulation_index": 4.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 5.0, "depth": 1.0 },
                            "target": { "target": "fm_index", "amount": 0.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams));
}

#[test]
fn test_audio_lfo_allows_grain_size_on_granular() {
    let spec = crate::spec::Spec::builder("test-audio-grain-size-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": {
                            "type": "granular",
                            "source": { "type": "tone", "waveform": "sine", "frequency": 440.0 },
                            "grain_size_ms": 50.0,
                            "grain_density": 20.0
                        },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                            "target": { "target": "grain_size", "amount_ms": 30.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_audio_lfo_rejects_grain_size_on_non_granular() {
    let spec = crate::spec::Spec::builder("test-audio-grain-size-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                            "target": { "target": "grain_size", "amount_ms": 30.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("grain_size LFO target is only valid for Granular")));
}

#[test]
fn test_audio_lfo_rejects_grain_size_with_zero_amount() {
    let spec = crate::spec::Spec::builder("test-audio-grain-size-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": {
                            "type": "granular",
                            "source": { "type": "tone", "waveform": "sine", "frequency": 440.0 },
                            "grain_size_ms": 50.0,
                            "grain_density": 20.0
                        },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                            "target": { "target": "grain_size", "amount_ms": 0.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams));
}

#[test]
fn test_audio_lfo_allows_grain_density_on_granular() {
    let spec = crate::spec::Spec::builder("test-audio-grain-dens-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": {
                            "type": "granular",
                            "source": { "type": "noise", "noise_type": "white" },
                            "grain_size_ms": 50.0,
                            "grain_density": 20.0
                        },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "triangle", "rate": 1.0, "depth": 0.8 },
                            "target": { "target": "grain_density", "amount": 15.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_audio_lfo_rejects_grain_density_on_non_granular() {
    let spec = crate::spec::Spec::builder("test-audio-grain-dens-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": { "type": "fm_synth", "carrier_freq": 440.0, "modulator_freq": 880.0, "modulation_index": 2.0 },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                            "target": { "target": "grain_density", "amount": 15.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("grain_density LFO target is only valid for Granular")));
}

#[test]
fn test_audio_lfo_rejects_grain_density_with_zero_amount() {
    let spec = crate::spec::Spec::builder("test-audio-grain-dens-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "layers": [
                    {
                        "synthesis": {
                            "type": "granular",
                            "source": { "type": "noise", "noise_type": "white" },
                            "grain_size_ms": 50.0,
                            "grain_density": 20.0
                        },
                        "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                        "volume": 1.0,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                            "target": { "target": "grain_density", "amount": 0.0 }
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams));
}

// ============================================================================
// Post-FX LFO Tests
// ============================================================================

#[test]
fn test_post_fx_lfo_delay_time_valid() {
    let spec = crate::spec::Spec::builder("test-post-fx-lfo-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.5,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.1 },
                        "volume": 0.8,
                        "pan": 0.0
                    }
                ],
                "effects": [
                    { "type": "delay", "time_ms": 250.0, "feedback": 0.4, "wet": 0.3 }
                ],
                "post_fx_lfos": [
                    {
                        "config": { "waveform": "sine", "rate": 0.5, "depth": 1.0 },
                        "target": { "target": "delay_time", "amount_ms": 25.0 }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_post_fx_lfo_rejects_delay_time_on_layer() {
    let spec = crate::spec::Spec::builder("test-post-fx-lfo-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.5,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.1 },
                        "volume": 0.8,
                        "pan": 0.0,
                        "lfo": {
                            "config": { "waveform": "sine", "rate": 0.5, "depth": 1.0 },
                            "target": { "target": "delay_time", "amount_ms": 25.0 }
                        }
                    }
                ],
                "effects": [
                    { "type": "delay", "time_ms": 250.0, "feedback": 0.4, "wet": 0.3 }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("delay_time LFO target is only valid in post_fx_lfos")));
}

#[test]
fn test_post_fx_lfo_rejects_layer_target_on_post_fx() {
    let spec = crate::spec::Spec::builder("test-post-fx-lfo-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.5,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.1 },
                        "volume": 0.8,
                        "pan": 0.0
                    }
                ],
                "effects": [],
                "post_fx_lfos": [
                    {
                        "config": { "waveform": "sine", "rate": 2.0, "depth": 1.0 },
                        "target": { "target": "pitch", "semitones": 2.0 }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("pitch LFO target is only valid in layer LFOs")));
}

#[test]
fn test_post_fx_lfo_rejects_duplicate_target() {
    let spec = crate::spec::Spec::builder("test-post-fx-lfo-04", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.5,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.1 },
                        "volume": 0.8,
                        "pan": 0.0
                    }
                ],
                "effects": [
                    { "type": "delay", "time_ms": 250.0, "feedback": 0.4, "wet": 0.3 }
                ],
                "post_fx_lfos": [
                    {
                        "config": { "waveform": "sine", "rate": 0.5, "depth": 1.0 },
                        "target": { "target": "delay_time", "amount_ms": 25.0 }
                    },
                    {
                        "config": { "waveform": "triangle", "rate": 1.0, "depth": 0.5 },
                        "target": { "target": "delay_time", "amount_ms": 10.0 }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("duplicate delay_time target in post_fx_lfos")));
}

#[test]
fn test_post_fx_lfo_rejects_delay_time_without_delay_effect() {
    let spec = crate::spec::Spec::builder("test-post-fx-lfo-05", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.5,
                "layers": [
                    {
                        "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                        "envelope": { "attack": 0.01, "decay": 0.1, "sustain": 0.5, "release": 0.1 },
                        "volume": 0.8,
                        "pan": 0.0
                    }
                ],
                "effects": [
                    { "type": "reverb", "room_size": 0.7, "damping": 0.5, "wet": 0.3 }
                ],
                "post_fx_lfos": [
                    {
                        "config": { "waveform": "sine", "rate": 0.5, "depth": 1.0 },
                        "target": { "target": "delay_time", "amount_ms": 25.0 }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_spec(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e
        .message
        .contains("delay_time LFO requires at least one delay effect")));
}
